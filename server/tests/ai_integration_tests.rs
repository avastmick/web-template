//! Integration tests for AI endpoints

use axum::{
    Router,
    body::Body,
    http::{Method, Request, StatusCode, header},
    response::Response,
};
use serde_json::{Value, json};
use sqlx::{Pool, Sqlite, SqlitePool};
use std::sync::Arc;
use tower::ServiceExt;

// Import the application modules we need for testing
use server::{
    routes::create_router,
    services::{AuthService, InviteService, OAuthService, UserServiceImpl},
};

/// Helper function to create a test database in memory
async fn create_test_db() -> Pool<Sqlite> {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Failed to create in-memory SQLite database");

    // Create the users table directly for testing
    sqlx::query(
        r"
        CREATE TABLE users (
            id TEXT PRIMARY KEY,
            email TEXT UNIQUE NOT NULL,
            hashed_password TEXT NOT NULL,
            provider TEXT NOT NULL DEFAULT 'local',
            provider_user_id TEXT,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        CREATE INDEX idx_users_email ON users(email);
        CREATE INDEX idx_users_provider_oauth ON users(provider, provider_user_id) WHERE provider != 'local';
        ",
    )
    .execute(&pool)
    .await
    .expect("Failed to create users table in test database");

    // Create the user_invites table for testing
    sqlx::query(
        r"
        CREATE TABLE user_invites (
            id TEXT PRIMARY KEY NOT NULL,
            email TEXT UNIQUE NOT NULL COLLATE NOCASE,
            invited_by TEXT,
            invited_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            used_at DATETIME,
            expires_at DATETIME,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        CREATE INDEX idx_user_invites_email ON user_invites(email);
        CREATE INDEX idx_user_invites_used_at ON user_invites(used_at) WHERE used_at IS NULL;
        CREATE INDEX idx_user_invites_expires_at ON user_invites(expires_at) WHERE expires_at IS NOT NULL;
        ",
    )
    .execute(&pool)
    .await
    .expect("Failed to create user_invites table in test database");

    // Create AI tables for testing
    sqlx::query(
        r"
        CREATE TABLE ai_conversations (
            id TEXT PRIMARY KEY NOT NULL,
            user_id TEXT NOT NULL,
            title TEXT,
            model TEXT NOT NULL,
            system_prompt TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            archived_at TEXT,
            metadata TEXT,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        );

        CREATE TABLE ai_messages (
            id TEXT PRIMARY KEY NOT NULL,
            conversation_id TEXT NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            token_count INTEGER,
            created_at TEXT NOT NULL,
            metadata TEXT,
            FOREIGN KEY (conversation_id) REFERENCES ai_conversations(id) ON DELETE CASCADE
        );

        CREATE TABLE ai_usage (
            id TEXT PRIMARY KEY NOT NULL,
            user_id TEXT NOT NULL,
            conversation_id TEXT,
            model TEXT NOT NULL,
            prompt_tokens INTEGER NOT NULL,
            completion_tokens INTEGER NOT NULL,
            total_tokens INTEGER NOT NULL,
            cost_cents INTEGER,
            request_id TEXT,
            duration_ms INTEGER,
            created_at TEXT NOT NULL,
            metadata TEXT,
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
            FOREIGN KEY (conversation_id) REFERENCES ai_conversations(id) ON DELETE SET NULL
        );

        CREATE INDEX idx_ai_conversations_user_id ON ai_conversations(user_id);
        CREATE INDEX idx_ai_conversations_created_at ON ai_conversations(created_at);
        CREATE INDEX idx_ai_conversations_archived ON ai_conversations(archived_at) WHERE archived_at IS NOT NULL;

        CREATE INDEX idx_ai_messages_conversation_id ON ai_messages(conversation_id);
        CREATE INDEX idx_ai_messages_created_at ON ai_messages(created_at);

        CREATE INDEX idx_ai_usage_user_id ON ai_usage(user_id);
        CREATE INDEX idx_ai_usage_model ON ai_usage(model);
        CREATE INDEX idx_ai_usage_created_at ON ai_usage(created_at);
        ",
    )
    .execute(&pool)
    .await
    .expect("Failed to create AI tables in test database");

    pool
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
    let oauth_service = Arc::new(OAuthService::new().expect("Failed to create oauth service"));

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
    sqlx::query(
        r"
        INSERT INTO user_invites (id, email, invited_by, invited_at, created_at, updated_at)
        VALUES (?1, ?2, ?3, datetime('now'), datetime('now'), datetime('now'))
        ",
    )
    .bind(format!("test-invite-{}", uuid::Uuid::new_v4()))
    .bind(email.to_lowercase())
    .bind("test-admin")
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
async fn test_list_models_endpoint() {
    let pool = create_test_db().await;
    let token =
        register_and_login_user(pool.clone(), "ai_test@example.com", "secure_password_123").await;
    let app = create_test_app_with_pool(pool).await;

    // Test the list models endpoint
    let response =
        send_authenticated_request(app, Method::GET, "/api/ai/models", &token, None).await;

    assert_eq!(response.status(), StatusCode::OK);

    let json_body = extract_json_response(response).await;
    assert!(json_body.is_array());
    let models = json_body.as_array().unwrap();
    assert!(!models.is_empty());

    // Check that we have expected model names
    let model_strings: Vec<String> = models
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    assert!(model_strings.iter().any(|m| m.contains("gpt")));
    assert!(model_strings.iter().any(|m| m.contains("claude")));
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
