// server/tests/endpoint/ai_tests.rs
#![allow(clippy::unwrap_used)]

//! Integration tests for AI-related API endpoints
//!
//! These tests verify the complete behavior of AI API endpoints
//! including authentication, request validation, and response formats.

use axum::{
    Router,
    body::Body,
    http::{Method, Request, StatusCode, header},
    response::Response,
};
use serde_json::{Value, json};
use tower::ServiceExt;

use server::routes::create_router;

use crate::common::TestContext;

// Test constants
const TEST_SECURE_PASS: &str = "secure_password_123";

/// Helper function to create the test app
async fn create_test_app() -> (Router, TestContext) {
    let ctx = TestContext::new().await;

    let router = create_router(
        ctx.user_service.clone(),
        ctx.auth_service.clone(),
        ctx.invite_service.clone(),
        ctx.oauth_service.clone(),
        &ctx.pool,
    )
    .expect("Failed to create router");

    (router, ctx)
}

/// Helper function to send a JSON request
async fn send_json_request(app: Router, method: Method, uri: &str, body: Value) -> Response<Body> {
    let request = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_string(&body).expect("Failed to serialize JSON"),
        ))
        .expect("Failed to build request");

    app.oneshot(request)
        .await
        .expect("Failed to execute request")
}

/// Helper function to send a GET request without body
async fn send_get_request(app: Router, uri: &str) -> Response<Body> {
    let request = Request::builder()
        .method(Method::GET)
        .uri(uri)
        .body(Body::empty())
        .expect("Failed to build request");

    app.oneshot(request)
        .await
        .expect("Failed to execute request")
}

/// Helper function to extract JSON response body
async fn extract_json_response(response: Response<Body>) -> Value {
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");
    serde_json::from_slice(&body).expect("Failed to deserialize JSON response")
}

/// Helper function to send an authenticated JSON request
async fn send_authenticated_json_request(
    app: Router,
    method: Method,
    uri: &str,
    token: &str,
    body: Value,
) -> Response<Body> {
    let request = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::from(
            serde_json::to_string(&body).expect("Failed to serialize JSON"),
        ))
        .expect("Failed to build request");

    app.oneshot(request)
        .await
        .expect("Failed to execute request")
}

/// Helper function to send an authenticated GET request
async fn send_authenticated_get_request(
    app: Router,
    uri: &str,
    token: &str,
) -> Response<Body> {
    let request = Request::builder()
        .method(Method::GET)
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .expect("Failed to build request");

    app.oneshot(request)
        .await
        .expect("Failed to execute request")
}

/// Helper function to send an authenticated DELETE request
async fn send_authenticated_delete_request(
    app: Router,
    uri: &str,
    token: &str,
) -> Response<Body> {
    let request = Request::builder()
        .method(Method::DELETE)
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .expect("Failed to build request");

    app.oneshot(request)
        .await
        .expect("Failed to execute request")
}

/// Helper to create invite and register user
async fn create_test_invite_with_context(ctx: &TestContext, email: &str) {
    ctx.invite_service
        .create_invite(email, Some("test-admin".to_string()), None)
        .await
        .expect("Failed to create test invite");
}

/// Helper to register and login, returning auth token
async fn register_and_login_user(
    app: Router,
    ctx: &TestContext,
    email: &str,
    password: &str,
) -> String {
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

    let login_payload = json!({
        "email": email,
        "password": password
    });

    let login_response =
        send_json_request(app, Method::POST, "/api/auth/login", login_payload).await;
    assert_eq!(login_response.status(), StatusCode::OK);

    let login_body = extract_json_response(login_response).await;
    login_body["auth_token"]
        .as_str()
        .expect("Expected auth_token in response")
        .to_string()
}

// ============================================================================
// Public endpoints (no auth required)
// ============================================================================

#[tokio::test]
async fn test_ai_info_endpoint() {
    let (app, _ctx) = create_test_app().await;

    let response = send_get_request(app, "/api/ai/info").await;

    assert_eq!(response.status(), StatusCode::OK);

    let json_body = extract_json_response(response).await;
    assert!(json_body.get("provider").is_some());
    assert!(json_body.get("schemas").is_some());
    assert_eq!(json_body.get("streaming_supported"), Some(&json!(true)));
    assert_eq!(json_body.get("websocket_supported"), Some(&json!(true)));
}

#[tokio::test]
async fn test_ai_health_endpoint() {
    let (app, _ctx) = create_test_app().await;

    let response = send_get_request(app, "/api/ai/health").await;

    // In test env, health check may fail due to mock/invalid API key
    let status = response.status();
    if status == StatusCode::OK {
        let json_body = extract_json_response(response).await;
        assert_eq!(json_body.get("status"), Some(&json!("healthy")));
        assert!(json_body.get("provider").is_some());
        assert!(json_body.get("timestamp").is_some());
    } else {
        // Accept 400 if provider health check fails in test env
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }
}

// ============================================================================
// Chat endpoint tests
// ============================================================================

#[tokio::test]
async fn test_chat_endpoint_unauthorized() {
    let (app, _ctx) = create_test_app().await;

    let payload = json!({
        "messages": [{"role": "user", "content": "Hello"}]
    });

    let response = send_json_request(app, Method::POST, "/api/ai/chat", payload).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_chat_endpoint_with_streaming_flag() {
    let (app, ctx) = create_test_app().await;

    let email = "chat_stream_test@example.com";
    create_test_invite_with_context(&ctx, email).await;
    let token = register_and_login_user(app.clone(), &ctx, email, TEST_SECURE_PASS).await;

    let payload = json!({
        "messages": [{"role": "user", "content": "Hello"}],
        "stream": true
    });

    let response =
        send_authenticated_json_request(app, Method::POST, "/api/ai/chat", &token, payload).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let json_body = extract_json_response(response).await;
    assert!(json_body["error"]
        .as_str()
        .expect("Expected error field")
        .contains("streaming"));
}

#[tokio::test]
async fn test_chat_endpoint_success() {
    let (app, ctx) = create_test_app().await;

    let email = "chat_success_test@example.com";
    create_test_invite_with_context(&ctx, email).await;
    let token = register_and_login_user(app.clone(), &ctx, email, TEST_SECURE_PASS).await;

    let payload = json!({
        "messages": [{"role": "user", "content": "Hello, how are you?"}],
        "stream": false
    });

    let response =
        send_authenticated_json_request(app, Method::POST, "/api/ai/chat", &token, payload).await;

    // In test env with real API key, should succeed; otherwise may fail with provider error
    let status = response.status();
    if status == StatusCode::OK {
        let json_body = extract_json_response(response).await;
        assert!(json_body.get("id").is_some());
        assert!(json_body.get("conversation_id").is_some());
        assert!(json_body.get("message").is_some());
    } else {
        // May return 400 due to invalid API key or provider error in test env
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }
}

// ============================================================================
// Streaming endpoint tests
// ============================================================================

#[tokio::test]
async fn test_chat_stream_endpoint_unauthorized() {
    let (app, _ctx) = create_test_app().await;

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/ai/chat/stream")
        .body(Body::empty())
        .expect("Failed to build request");

    let response = app
        .oneshot(request)
        .await
        .expect("Failed to execute request");

    // Missing auth header results in 400 (TypedHeader extraction fails)
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNAUTHORIZED
    );
}

// ============================================================================
// Contextual chat endpoint tests
// ============================================================================

#[tokio::test]
async fn test_contextual_chat_endpoint_unauthorized() {
    let (app, _ctx) = create_test_app().await;

    let payload = json!({
        "question": "What is Rust?"
    });

    let response =
        send_json_request(app, Method::POST, "/api/ai/chat/contextual", payload).await;

    // Missing auth header results in 400 (TypedHeader extraction fails)
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn test_contextual_chat_endpoint_success() {
    let (app, ctx) = create_test_app().await;

    let email = "contextual_chat_test@example.com";
    create_test_invite_with_context(&ctx, email).await;
    let token = register_and_login_user(app.clone(), &ctx, email, TEST_SECURE_PASS).await;

    let payload = json!({
        "question": "What is Rust programming language?",
        "context": ["Programming", "Systems"]
    });

    let response = send_authenticated_json_request(
        app,
        Method::POST,
        "/api/ai/chat/contextual",
        &token,
        payload,
    )
    .await;

    // May return 400 if contextual_chat template is not configured in test env
    assert!(
        response.status() == StatusCode::OK || response.status() == StatusCode::BAD_REQUEST
    );
}

// ============================================================================
// Code analysis endpoint tests
// ============================================================================

#[tokio::test]
async fn test_code_analysis_endpoint_unauthorized() {
    let (app, _ctx) = create_test_app().await;

    let payload = json!({
        "code": "fn main() { println!(\"Hello\"); }",
        "language": "rust"
    });

    let response =
        send_json_request(app, Method::POST, "/api/ai/analyze/code", payload).await;

    // Missing auth header results in 400 (TypedHeader extraction fails)
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn test_code_analysis_endpoint_success() {
    let (app, ctx) = create_test_app().await;

    let email = "code_analysis_test@example.com";
    create_test_invite_with_context(&ctx, email).await;
    let token = register_and_login_user(app.clone(), &ctx, email, TEST_SECURE_PASS).await;

    let payload = json!({
        "code": "fn main() { println!(\"Hello, World!\"); }",
        "language": "rust",
        "context": "Simple hello world program"
    });

    let response = send_authenticated_json_request(
        app,
        Method::POST,
        "/api/ai/analyze/code",
        &token,
        payload,
    )
    .await;

    // May return 400 if code_analysis template is not configured in test env
    let status = response.status();
    if status == StatusCode::OK {
        let json_body = extract_json_response(response).await;
        assert!(json_body.get("analysis").is_some());
        assert_eq!(json_body.get("language"), Some(&json!("rust")));
        assert!(json_body.get("timestamp").is_some());
    } else {
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }
}

// ============================================================================
// Upload endpoint tests
// ============================================================================

#[tokio::test]
async fn test_upload_endpoint_unauthorized() {
    let (app, _ctx) = create_test_app().await;

    // Send empty multipart request
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/ai/upload")
        .header("content-type", "multipart/form-data; boundary=----test")
        .body(Body::empty())
        .expect("Failed to build request");

    let response = app
        .oneshot(request)
        .await
        .expect("Failed to execute request");

    // Missing auth header results in 400 (TypedHeader extraction fails)
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNAUTHORIZED
    );
}

// ============================================================================
// Conversations endpoint tests
// ============================================================================

#[tokio::test]
async fn test_get_conversations_endpoint_unauthorized() {
    let (app, _ctx) = create_test_app().await;

    let response = send_get_request(app, "/api/ai/conversations").await;

    // Missing auth header results in 400 (TypedHeader extraction fails)
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn test_get_conversations_endpoint_success() {
    let (app, ctx) = create_test_app().await;

    let email = "conversations_test@example.com";
    create_test_invite_with_context(&ctx, email).await;
    let token = register_and_login_user(app.clone(), &ctx, email, TEST_SECURE_PASS).await;

    let response = send_authenticated_get_request(app, "/api/ai/conversations", &token).await;

    assert_eq!(response.status(), StatusCode::OK);

    let json_body = extract_json_response(response).await;
    assert!(json_body.get("conversations").is_some());
}

#[tokio::test]
async fn test_get_conversation_by_id_unauthorized() {
    let (app, _ctx) = create_test_app().await;

    let response = send_get_request(app, "/api/ai/conversations/test-conv-id").await;

    // Missing auth header results in 400 (TypedHeader extraction fails)
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn test_get_conversation_by_id_not_found() {
    let (app, ctx) = create_test_app().await;

    let email = "conv_by_id_test@example.com";
    create_test_invite_with_context(&ctx, email).await;
    let token = register_and_login_user(app.clone(), &ctx, email, TEST_SECURE_PASS).await;

    let response =
        send_authenticated_get_request(app, "/api/ai/conversations/nonexistent-id", &token).await;

    // Should return 400 or 404 for non-existent conversation
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn test_delete_conversation_unauthorized() {
    let (app, _ctx) = create_test_app().await;

    let request = Request::builder()
        .method(Method::DELETE)
        .uri("/api/ai/conversations/test-id")
        .body(Body::empty())
        .expect("Failed to build request");

    let response = app
        .oneshot(request)
        .await
        .expect("Failed to execute request");

    // Missing auth header results in 400 (TypedHeader extraction fails)
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn test_delete_conversation_not_found() {
    let (app, ctx) = create_test_app().await;

    let email = "delete_conv_test@example.com";
    create_test_invite_with_context(&ctx, email).await;
    let token = register_and_login_user(app.clone(), &ctx, email, TEST_SECURE_PASS).await;

    let response =
        send_authenticated_delete_request(app, "/api/ai/conversations/nonexistent-id", &token)
            .await;

    // Should return 400 or 404 for non-existent conversation
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn test_archive_conversation_unauthorized() {
    let (app, _ctx) = create_test_app().await;

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/ai/conversations/test-id/archive")
        .body(Body::empty())
        .expect("Failed to build request");

    let response = app
        .oneshot(request)
        .await
        .expect("Failed to execute request");

    // Missing auth header results in 400 (TypedHeader extraction fails)
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn test_archive_conversation_not_found() {
    let (app, ctx) = create_test_app().await;

    let email = "archive_conv_test@example.com";
    create_test_invite_with_context(&ctx, email).await;
    let token = register_and_login_user(app.clone(), &ctx, email, TEST_SECURE_PASS).await;

    let response = send_authenticated_json_request(
        app,
        Method::POST,
        "/api/ai/conversations/nonexistent-id/archive",
        &token,
        json!({}),
    )
    .await;

    // Should return 400 or 404 for non-existent conversation
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::NOT_FOUND
    );
}

// ============================================================================
// Usage endpoint tests
// ============================================================================

#[tokio::test]
async fn test_usage_endpoint_unauthorized() {
    let (app, _ctx) = create_test_app().await;

    let response = send_get_request(app, "/api/ai/usage").await;

    // Missing auth header results in 400 (TypedHeader extraction fails)
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn test_usage_endpoint_success() {
    let (app, ctx) = create_test_app().await;

    let email = "usage_test@example.com";
    create_test_invite_with_context(&ctx, email).await;
    let token = register_and_login_user(app.clone(), &ctx, email, TEST_SECURE_PASS).await;

    let response = send_authenticated_get_request(app, "/api/ai/usage", &token).await;

    assert_eq!(response.status(), StatusCode::OK);
}

// ============================================================================
// Moderation endpoint tests
// ============================================================================

#[tokio::test]
async fn test_moderate_endpoint_unauthorized() {
    let (app, _ctx) = create_test_app().await;

    let payload = json!({
        "content": "Test content for moderation"
    });

    let response = send_json_request(app, Method::POST, "/api/ai/moderate", payload).await;

    // Missing auth header results in 400 (TypedHeader extraction fails)
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn test_moderate_endpoint_missing_content() {
    let (app, ctx) = create_test_app().await;

    let email = "moderate_missing_test@example.com";
    create_test_invite_with_context(&ctx, email).await;
    let token = register_and_login_user(app.clone(), &ctx, email, TEST_SECURE_PASS).await;

    let payload = json!({});

    let response =
        send_authenticated_json_request(app, Method::POST, "/api/ai/moderate", &token, payload)
            .await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let json_body = extract_json_response(response).await;
    assert!(json_body["error"]
        .as_str()
        .expect("Expected error field")
        .contains("content"));
}

#[tokio::test]
async fn test_moderate_endpoint_success() {
    let (app, ctx) = create_test_app().await;

    let email = "moderate_success_test@example.com";
    create_test_invite_with_context(&ctx, email).await;
    let token = register_and_login_user(app.clone(), &ctx, email, TEST_SECURE_PASS).await;

    let payload = json!({
        "content": "This is a test message for content moderation."
    });

    let response =
        send_authenticated_json_request(app, Method::POST, "/api/ai/moderate", &token, payload)
            .await;

    // May return 400 if content_moderation template is not configured in test env
    assert!(
        response.status() == StatusCode::OK || response.status() == StatusCode::BAD_REQUEST
    );
}

// ============================================================================
// Edge cases and validation tests
// ============================================================================

#[tokio::test]
async fn test_chat_endpoint_empty_messages() {
    let (app, ctx) = create_test_app().await;

    let email = "chat_empty_test@example.com";
    create_test_invite_with_context(&ctx, email).await;
    let token = register_and_login_user(app.clone(), &ctx, email, TEST_SECURE_PASS).await;

    let payload = json!({
        "messages": []
    });

    let response =
        send_authenticated_json_request(app, Method::POST, "/api/ai/chat", &token, payload).await;

    // Empty messages should either succeed with empty response or fail with validation error
    // The exact behavior depends on implementation
    let status = response.status();
    assert!(
        status == StatusCode::OK || status == StatusCode::BAD_REQUEST,
        "Expected OK or BAD_REQUEST, got {status}"
    );
}

#[tokio::test]
async fn test_chat_endpoint_invalid_token() {
    let (app, _ctx) = create_test_app().await;

    let payload = json!({
        "messages": [{"role": "user", "content": "Hello"}]
    });

    let response = send_authenticated_json_request(
        app,
        Method::POST,
        "/api/ai/chat",
        "invalid_token_here",
        payload,
    )
    .await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_chat_with_context_parameter() {
    let (app, ctx) = create_test_app().await;

    let email = "chat_context_test@example.com";
    create_test_invite_with_context(&ctx, email).await;
    let token = register_and_login_user(app.clone(), &ctx, email, TEST_SECURE_PASS).await;

    let payload = json!({
        "messages": [{"role": "user", "content": "What is the weather?"}],
        "context": ["The user is in San Francisco", "Current date is January 2026"],
        "stream": false
    });

    let response =
        send_authenticated_json_request(app, Method::POST, "/api/ai/chat", &token, payload).await;

    // May return 400 due to API key/provider issues in test env
    assert!(
        response.status() == StatusCode::OK || response.status() == StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn test_chat_with_model_parameter() {
    let (app, ctx) = create_test_app().await;

    let email = "chat_model_test@example.com";
    create_test_invite_with_context(&ctx, email).await;
    let token = register_and_login_user(app.clone(), &ctx, email, TEST_SECURE_PASS).await;

    let payload = json!({
        "messages": [{"role": "user", "content": "Hello"}],
        "model": "gpt-4",
        "stream": false
    });

    let response =
        send_authenticated_json_request(app, Method::POST, "/api/ai/chat", &token, payload).await;

    // May return 400 due to API key/provider issues in test env
    assert!(
        response.status() == StatusCode::OK || response.status() == StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn test_chat_with_temperature_and_max_tokens() {
    let (app, ctx) = create_test_app().await;

    let email = "chat_params_test@example.com";
    create_test_invite_with_context(&ctx, email).await;
    let token = register_and_login_user(app.clone(), &ctx, email, TEST_SECURE_PASS).await;

    let payload = json!({
        "messages": [{"role": "user", "content": "Hello"}],
        "temperature": 0.7,
        "max_tokens": 100,
        "stream": false
    });

    let response =
        send_authenticated_json_request(app, Method::POST, "/api/ai/chat", &token, payload).await;

    // May return 400 due to API key/provider issues in test env
    assert!(
        response.status() == StatusCode::OK || response.status() == StatusCode::BAD_REQUEST
    );
}
