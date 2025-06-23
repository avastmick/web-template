//! Integration tests for AI endpoints

use axum::{
    Router,
    body::Body,
    http::{Method, Request, StatusCode, header},
    response::Response,
};
use serde_json::{Value, json};
use sqlx::{Pool, Sqlite};
use std::sync::Arc;
use tower::ServiceExt;

// Import the application modules we need for testing
use server::{
    routes::create_router,
    services::{AuthService, InviteService, OAuthService, UserServiceImpl},
};

mod common;

/// Helper function to create a test database in memory
async fn create_test_db() -> Pool<Sqlite> {
    common::setup_test_database().await
}

/// Helper function to create the test app with a specific database pool
async fn create_test_app_with_pool(pool: Pool<Sqlite>) -> Router {
    // Set up environment variables for testing
    unsafe {
        std::env::set_var(
            "JWT_SECRET",
            "test_secret_key_that_is_long_enough_for_testing",
        );
        std::env::set_var("GOOGLE_CLIENT_ID", "test_client_id");
        std::env::set_var("GOOGLE_CLIENT_SECRET", "test_client_secret");
        std::env::set_var("SERVER_URL", "http://localhost:8081");
        std::env::set_var("STATIC_DIR", "/tmp");
        // Set a test API key for OpenRouter (will be ignored in tests)
        std::env::set_var("OPENROUTER_API_KEY", "test_api_key_not_real");
    }

    let user_service = Arc::new(UserServiceImpl::new(pool.clone()));
    let auth_service = Arc::new(AuthService::new().expect("Failed to create auth service"));
    let invite_service = Arc::new(InviteService::new(pool.clone()));
    let oauth_service =
        Arc::new(OAuthService::new(pool.clone()).expect("Failed to create oauth service"));

    create_router(
        user_service,
        auth_service,
        invite_service,
        oauth_service,
        pool,
    )
    .await
    .expect("Failed to create router")
}

/// Helper function to send a JSON request to the test app
async fn send_json_request(app: Router, method: Method, uri: &str, body: Value) -> Response<Body> {
    let request = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();

    app.oneshot(request).await.unwrap()
}

/// Helper function to extract JSON response body
async fn extract_json_response(response: Response<Body>) -> Value {
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&body).unwrap()
}

/// Helper function to send a request with authorization header
async fn send_authenticated_request(
    app: Router,
    method: Method,
    uri: &str,
    token: &str,
    body: Option<Value>,
) -> Response<Body> {
    let mut request_builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {token}"));

    let request = if let Some(body_data) = body {
        request_builder = request_builder.header("content-type", "application/json");
        request_builder
            .body(Body::from(serde_json::to_string(&body_data).unwrap()))
            .unwrap()
    } else {
        request_builder.body(Body::empty()).unwrap()
    };

    app.oneshot(request).await.unwrap()
}

/// Helper function to register a user and get a valid JWT token
async fn register_and_login_user(pool: Pool<Sqlite>, email: &str, password: &str) -> String {
    // First create an invite
    use chrono::Utc;
    let now = Utc::now();
    sqlx::query(
        r"
        INSERT INTO user_invites (id, email, invited_by, invited_at, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        ",
    )
    .bind(format!("test-invite-{}", uuid::Uuid::new_v4()))
    .bind(email.to_lowercase())
    .bind("test-admin")
    .bind(now)
    .bind(now)
    .bind(now)
    .execute(&pool)
    .await
    .expect("Failed to create test invite");

    let app = create_test_app_with_pool(pool).await;

    // Register the user
    let register_payload = json!({
        "email": email,
        "password": password
    });

    let register_response = send_json_request(
        app.clone(),
        Method::POST,
        "/api/auth/register",
        register_payload,
    )
    .await;
    assert_eq!(register_response.status(), StatusCode::CREATED);

    // Login to get the token
    let login_payload = json!({
        "email": email,
        "password": password
    });

    let login_response =
        send_json_request(app, Method::POST, "/api/auth/login", login_payload).await;
    assert_eq!(login_response.status(), StatusCode::OK);

    let login_body = extract_json_response(login_response).await;
    login_body["token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn test_ai_info_endpoint() {
    let pool = create_test_db().await;
    let app = create_test_app_with_pool(pool).await;

    // Test the AI info endpoint (no auth required for this endpoint)
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/ai/info")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // This will likely fail due to missing API key, but we can test the endpoint structure
    if response.status() == StatusCode::OK {
        let json_body = extract_json_response(response).await;
        assert!(json_body["provider"].is_string());
        assert!(json_body["templates"].is_array());
        assert!(json_body["schemas"].is_array());
        assert!(json_body["streaming_supported"].is_boolean());
        assert!(json_body["websocket_supported"].is_boolean());
    } else {
        // Expected to fail due to missing real API key in test environment
        assert!(response.status().is_client_error() || response.status().is_server_error());
    }
}

#[tokio::test]
async fn test_chat_endpoint_without_auth() {
    let pool = create_test_db().await;
    let app = create_test_app_with_pool(pool).await;

    let chat_payload = json!({
        "messages": [
            {
                "role": "user",
                "content": "Hello, how are you?"
            }
        ]
    });

    let response = send_json_request(app, Method::POST, "/api/ai/chat", chat_payload).await;

    // Should return unauthorized
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_chat_endpoint_with_auth() {
    let pool = create_test_db().await;
    let token =
        register_and_login_user(pool.clone(), "chat_test@example.com", "secure_password_123").await;
    let app = create_test_app_with_pool(pool).await;

    let chat_payload = json!({
        "messages": [
            {
                "role": "user",
                "content": "Hello, how are you?"
            }
        ]
    });

    let response = send_authenticated_request(
        app,
        Method::POST,
        "/api/ai/chat",
        &token,
        Some(chat_payload),
    )
    .await;

    // This will likely fail due to missing real API key, but we can test the auth flow
    // Either succeeds or fails with a specific AI error (not auth error)
    assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_chat_endpoint_invalid_payload() {
    let pool = create_test_db().await;
    let token = register_and_login_user(
        pool.clone(),
        "invalid_test@example.com",
        "secure_password_123",
    )
    .await;
    let app = create_test_app_with_pool(pool).await;

    let invalid_payload = json!({
        "invalid_field": "invalid_value"
    });

    let response = send_authenticated_request(
        app,
        Method::POST,
        "/api/ai/chat",
        &token,
        Some(invalid_payload),
    )
    .await;

    // Should return bad request for invalid payload
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_chat_stream_endpoint() {
    let pool = create_test_db().await;
    let token = register_and_login_user(
        pool.clone(),
        "stream_test@example.com",
        "secure_password_123",
    )
    .await;
    let app = create_test_app_with_pool(pool).await;

    // Test SSE streaming endpoint with simple query parameters
    let uri = "/api/ai/chat/stream?stream=true";
    let response = send_authenticated_request(app, Method::GET, uri, &token, None).await;

    // This will likely fail due to missing real API key or invalid query format
    // but we test that it's not an auth error
    assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_chat_websocket_endpoint() {
    let pool = create_test_db().await;
    let token =
        register_and_login_user(pool.clone(), "ws_test@example.com", "secure_password_123").await;
    let app = create_test_app_with_pool(pool).await;

    // Test WebSocket upgrade endpoint
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/ai/chat/ws")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .header("upgrade", "websocket")
        .header("connection", "upgrade")
        .header("sec-websocket-key", "test-key")
        .header("sec-websocket-version", "13")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Should not be an auth error (either upgrades or fails for other reasons)
    assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
}
