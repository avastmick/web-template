// kanbain/server/tests/auth_integration_tests.rs
#![allow(clippy::unwrap_used)]

//! Integration tests for authentication endpoints
//!
//! These tests verify the complete behavior of auth-related API endpoints
//! including request validation, business logic, and database interactions.

use axum::{
    Router,
    body::Body,
    http::{Method, Request, StatusCode, header},
    response::Response,
};
use serde_json::{Value, json};
use tower::ServiceExt; // for `oneshot` and `ready`

// Import the application modules we need for testing
use server::routes::create_router;

// Test constants to avoid gitleaks false positives
const TEST_EMAIL: &str = "test@example.com";
const TEST_SECURE_PASS: &str = "secure_password_123";
const TEST_WEAK_PASS: &str = "weak";
const TEST_WRONG_PASS: &str = "wrong_password_123";
const TEST_CORRECT_PASS: &str = "correct_password_123";
const TEST_EMPTY_PASS: &str = "";

use crate::common::TestContext;

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

/// Helper function to send a JSON request to the test app
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

/// Helper function to extract JSON response body
async fn extract_json_response(response: Response<Body>) -> Value {
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");
    serde_json::from_slice(&body).expect("Failed to deserialize JSON response")
}

/// Helper function to send a request with authorization header
async fn send_authenticated_request(
    app: Router,
    method: Method,
    uri: &str,
    token: &str,
) -> Response<Body> {
    let request = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .expect("Failed to build authenticated request");

    app.oneshot(request)
        .await
        .expect("Failed to execute authenticated request")
}

/// Helper function to create an invite in the test context
async fn create_test_invite_with_context(ctx: &TestContext, email: &str) {
    ctx.invite_service
        .create_invite(email, Some("test-admin".to_string()), None)
        .await
        .expect("Failed to create test invite");
}

/// Helper function to register a user and get a valid JWT token
/// Note: This assumes the invite has already been created
async fn register_and_login_user(
    app: Router,
    ctx: &TestContext,
    email: &str,
    password: &str,
) -> String {
    // Register the user
    let email_val = email;
    let pass_val = password;
    let register_payload = json!({
        "email": email_val,
        "password": pass_val
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
        "email": email_val,
        "password": pass_val
    });

    let login_response =
        send_json_request(app, Method::POST, "/api/auth/login", login_payload).await;
    assert_eq!(login_response.status(), StatusCode::OK);

    let login_body = extract_json_response(login_response).await;
    let auth_token = login_body["auth_token"]
        .as_str()
        .expect("Expected auth_token in response")
        .to_string();

    // Database verification: Check user exists and was updated
    let updated_at =
        sqlx::query_scalar::<_, String>("SELECT updated_at FROM users WHERE email = ?")
            .bind(email_val)
            .fetch_one(&ctx.pool)
            .await
            .expect("User should exist");

    assert!(!updated_at.is_empty(), "Updated timestamp should be set");

    auth_token
}

#[tokio::test]
async fn test_register_user_success() {
    let (app, ctx) = create_test_app().await;

    // Use a unique email for this test
    let test_email = "test_register_success@example.com";

    // Create invite first
    create_test_invite_with_context(&ctx, test_email).await;

    let test_pass = TEST_SECURE_PASS;
    let payload = json!({
        "email": test_email,
        "password": test_pass
    });

    let response = send_json_request(app, Method::POST, "/api/auth/register", payload).await;

    assert_eq!(response.status(), StatusCode::CREATED);

    let json_body = extract_json_response(response).await;
    assert!(json_body["auth_user"]["id"].is_string());
    assert_eq!(json_body["auth_user"]["email"], test_email);
    assert!(json_body["auth_user"]["created_at"].is_string());
    assert!(json_body["auth_user"]["updated_at"].is_string());
    assert!(json_body["payment_user"]["payment_required"].is_boolean());

    // Database verification: Ensure user was actually created
    let user_id = json_body["auth_user"]["id"].as_str().unwrap();
    let db_user = sqlx::query_as::<_, (String, String)>("SELECT id, email FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_one(&ctx.pool)
        .await
        .expect("User should exist in database");

    assert_eq!(db_user.0, user_id);
    assert_eq!(db_user.1, test_email);

    // Verify invite was marked as used
    let used_at =
        sqlx::query_scalar::<_, Option<String>>("SELECT used_at FROM user_invites WHERE email = ?")
            .bind(test_email)
            .fetch_one(&ctx.pool)
            .await
            .expect("Invite should exist");

    assert!(used_at.is_some(), "Invite should be marked as used");
}

#[tokio::test]
async fn test_register_user_without_invite() {
    let (app, ctx) = create_test_app().await;

    let test_email = "noinvite@example.com";
    let test_pass = TEST_SECURE_PASS;
    let payload = json!({
        "email": test_email,
        "password": test_pass
    });

    let response = send_json_request(app, Method::POST, "/api/auth/register", payload).await;

    assert_eq!(response.status(), StatusCode::CREATED);

    let json_body = extract_json_response(response).await;
    assert!(json_body["auth_user"]["id"].is_string());
    assert_eq!(json_body["auth_user"]["email"], "noinvite@example.com");
    assert!(json_body["auth_user"]["created_at"].is_string());
    assert!(json_body["auth_user"]["updated_at"].is_string());
    // Without invite, payment should be required
    assert_eq!(json_body["payment_user"]["payment_required"], true);

    // Database verification: Ensure user was actually created
    let user_id = json_body["auth_user"]["id"].as_str().unwrap();
    let db_user = sqlx::query_as::<_, (String, String)>("SELECT id, email FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_one(&ctx.pool)
        .await
        .expect("User should exist in database");

    assert_eq!(db_user.0, user_id);
    assert_eq!(db_user.1, test_email);
    // Payment required status is in the response, not the database
    assert!(
        json_body["payment_user"]["payment_required"]
            .as_bool()
            .unwrap(),
        "Payment should be required for users without invite"
    );
}

#[tokio::test]
async fn test_register_user_invalid_email() {
    let (app, _ctx) = create_test_app().await;

    let invalid_email = "invalid-email";
    let test_pass = TEST_SECURE_PASS;
    let payload = json!({
        "email": invalid_email,
        "password": test_pass
    });

    let response = send_json_request(app, Method::POST, "/api/auth/register", payload).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let json_body = extract_json_response(response).await;
    assert!(
        json_body["error"]
            .as_str()
            .expect("Expected error field to be a string")
            .contains("Email must be a valid email address")
    );
}

#[tokio::test]
async fn test_register_user_weak_password() {
    let (app, _ctx) = create_test_app().await;

    let test_email = TEST_EMAIL;
    let weak_pass = TEST_WEAK_PASS;
    let payload = json!({
        "email": test_email,
        "password": weak_pass
    });

    let response = send_json_request(app, Method::POST, "/api/auth/register", payload).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let json_body = extract_json_response(response).await;
    assert!(
        json_body["error"]
            .as_str()
            .expect("Expected error field to be a string")
            .contains("Password must be at least 12 characters long")
    );
}

#[tokio::test]
async fn test_register_user_duplicate_email() {
    let dup_email = "duplicate@example.com";
    let (app, ctx) = create_test_app().await;

    // Create an invite for the test
    create_test_invite_with_context(&ctx, dup_email).await;

    let test_pass = TEST_SECURE_PASS;
    let payload = json!({
        "email": dup_email,
        "password": test_pass
    });

    // First registration should succeed
    let response1 = send_json_request(
        app.clone(),
        Method::POST,
        "/api/auth/register",
        payload.clone(),
    )
    .await;
    assert_eq!(response1.status(), StatusCode::CREATED);

    // Second registration with same email should fail
    let response2 = send_json_request(app, Method::POST, "/api/auth/register", payload).await;
    // Should fail because user already exists (409 Conflict)
    assert_eq!(response2.status(), StatusCode::CONFLICT);

    let json_body = extract_json_response(response2).await;
    assert!(
        json_body["error"]
            .as_str()
            .expect("Expected error field to be a string")
            .contains("already exists")
            || json_body["error"]
                .as_str()
                .expect("Expected error field to be a string")
                .contains("User already exists")
    );

    // Database verification: Ensure only one user exists with this email
    let user_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE email = ?")
        .bind(dup_email)
        .fetch_one(&ctx.pool)
        .await
        .expect("Query should succeed");

    assert_eq!(user_count, 1, "Only one user should exist with this email");
}

#[tokio::test]
async fn test_register_user_missing_fields() {
    let (app, _ctx) = create_test_app().await;

    // Test missing email
    let test_pass = TEST_SECURE_PASS;
    let payload = json!({
        "password": test_pass
    });

    let response =
        send_json_request(app.clone(), Method::POST, "/api/auth/register", payload).await;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    // Test missing password
    let test_email = TEST_EMAIL;
    let payload = json!({
        "email": test_email
    });

    let response = send_json_request(app, Method::POST, "/api/auth/register", payload).await;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_register_user_empty_body() {
    let (app, _ctx) = create_test_app().await;

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/register")
        .header("content-type", "application/json")
        .body(Body::empty())
        .expect("Failed to build request");

    let response = app
        .oneshot(request)
        .await
        .expect("Failed to execute request");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

// Login endpoint tests

#[tokio::test]
async fn test_login_user_success() {
    let login_email = "login_test@example.com";
    let (app, ctx) = create_test_app().await;

    // Create invite for the test user
    create_test_invite_with_context(&ctx, login_email).await;

    // First, register a user
    let test_pass = TEST_SECURE_PASS;
    let register_payload = json!({
        "email": login_email,
        "password": test_pass
    });

    let register_response = send_json_request(
        app.clone(),
        Method::POST,
        "/api/auth/register",
        register_payload,
    )
    .await;
    assert_eq!(register_response.status(), StatusCode::CREATED);

    // Now try to login
    let login_payload = json!({
        "email": login_email,
        "password": test_pass
    });

    let login_response =
        send_json_request(app, Method::POST, "/api/auth/login", login_payload).await;

    assert_eq!(login_response.status(), StatusCode::OK);

    let json_body = extract_json_response(login_response).await;
    assert!(json_body["auth_token"].is_string());
    assert!(json_body["auth_user"]["id"].is_string());
    assert_eq!(json_body["auth_user"]["email"], "login_test@example.com");
    assert!(json_body["auth_user"]["created_at"].is_string());
    assert!(json_body["auth_user"]["updated_at"].is_string());
}

#[tokio::test]
async fn test_login_user_invalid_email() {
    let (app, _ctx) = create_test_app().await;

    let invalid_email = "invalid-email";
    let test_pass = TEST_SECURE_PASS;
    let login_payload = json!({
        "email": invalid_email,
        "password": test_pass
    });

    let response = send_json_request(app, Method::POST, "/api/auth/login", login_payload).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let json_body = extract_json_response(response).await;
    assert!(
        json_body["error"]
            .as_str()
            .expect("Expected error field to be a string")
            .contains("Email must be a valid email address")
    );
}

#[tokio::test]
async fn test_login_user_nonexistent_email() {
    let (app, _ctx) = create_test_app().await;

    let nonexistent_email = "nonexistent@example.com";
    let test_pass = TEST_SECURE_PASS;
    let login_payload = json!({
        "email": nonexistent_email,
        "password": test_pass
    });

    let response = send_json_request(app, Method::POST, "/api/auth/login", login_payload).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let json_body = extract_json_response(response).await;
    assert!(
        json_body["error"]
            .as_str()
            .expect("Expected error field to be a string")
            .contains("Invalid email or password")
    );
}

#[tokio::test]
async fn test_login_user_wrong_password() {
    let wrong_pass_email = "wrong_password_test@example.com";
    let (app, ctx) = create_test_app().await;

    // Create invite for the test user
    create_test_invite_with_context(&ctx, wrong_pass_email).await;

    // First, register a user
    let correct_pass = TEST_CORRECT_PASS;
    let register_payload = json!({
        "email": wrong_pass_email,
        "password": correct_pass
    });

    let register_response = send_json_request(
        app.clone(),
        Method::POST,
        "/api/auth/register",
        register_payload,
    )
    .await;
    assert_eq!(register_response.status(), StatusCode::CREATED);

    // Now try to login with wrong password
    let wrong_pass = TEST_WRONG_PASS;
    let login_payload = json!({
        "email": wrong_pass_email,
        "password": wrong_pass
    });

    let login_response =
        send_json_request(app, Method::POST, "/api/auth/login", login_payload).await;

    assert_eq!(login_response.status(), StatusCode::UNAUTHORIZED);

    let json_body = extract_json_response(login_response).await;
    assert!(
        json_body["error"]
            .as_str()
            .expect("Expected error field to be a string")
            .contains("Invalid email or password")
    );
}

#[tokio::test]
async fn test_login_user_missing_fields() {
    let (app, _ctx) = create_test_app().await;

    // Test missing email
    let test_pass = TEST_SECURE_PASS;
    let payload = json!({
        "password": test_pass
    });

    let response = send_json_request(app.clone(), Method::POST, "/api/auth/login", payload).await;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    // Test missing password
    let test_email = TEST_EMAIL;
    let payload = json!({
        "email": test_email
    });

    let response = send_json_request(app, Method::POST, "/api/auth/login", payload).await;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_login_user_empty_password() {
    let (app, _ctx) = create_test_app().await;

    let test_email = TEST_EMAIL;
    let empty_pass = TEST_EMPTY_PASS;
    let login_payload = json!({
        "email": test_email,
        "password": empty_pass
    });

    let response = send_json_request(app, Method::POST, "/api/auth/login", login_payload).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let json_body = extract_json_response(response).await;
    assert!(
        json_body["error"]
            .as_str()
            .expect("Expected error field to be a string")
            .contains("Password is required")
    );
}

// Protected endpoint tests

#[tokio::test]
async fn test_get_current_user_success() {
    let protected_email = "protected_test@example.com";
    let (app, ctx) = create_test_app().await;

    // Create invite for the test user
    create_test_invite_with_context(&ctx, protected_email).await;

    // Register and login to get a valid token
    let test_pass = TEST_SECURE_PASS;
    let token = register_and_login_user(app.clone(), &ctx, protected_email, test_pass).await;

    // Access the protected endpoint
    let response = send_authenticated_request(app, Method::GET, "/api/users/me", &token).await;

    assert_eq!(response.status(), StatusCode::OK);

    let json_body = extract_json_response(response).await;
    assert!(json_body["auth_user"]["id"].is_string());
    assert_eq!(
        json_body["auth_user"]["email"],
        "protected_test@example.com"
    );
    assert!(json_body["auth_user"]["created_at"].is_string());
    assert!(json_body["auth_user"]["updated_at"].is_string());
}

#[tokio::test]
async fn test_get_current_user_missing_auth_header() {
    let (app, _ctx) = create_test_app().await;

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/users/me")
        .body(Body::empty())
        .expect("Failed to build request");

    let response = app
        .oneshot(request)
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let json_body = extract_json_response(response).await;
    assert!(
        json_body["error"]
            .as_str()
            .expect("Expected error field to be a string")
            .contains("Missing or invalid authorization header")
    );
}

#[tokio::test]
async fn test_get_current_user_invalid_token() {
    let (app, _ctx) = create_test_app().await;

    let response =
        send_authenticated_request(app, Method::GET, "/api/users/me", "invalid_token").await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let json_body = extract_json_response(response).await;
    assert!(
        json_body["error"]
            .as_str()
            .expect("Expected error field to be a string")
            .contains("Invalid or expired token")
    );
}

#[tokio::test]
async fn test_get_current_user_malformed_token() {
    let (app, _ctx) = create_test_app().await;

    let response =
        send_authenticated_request(app, Method::GET, "/api/users/me", "Bearer malformed").await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let json_body = extract_json_response(response).await;
    assert!(
        json_body["error"]
            .as_str()
            .expect("Expected error field to be a string")
            .contains("Invalid or expired token")
    );
}

#[tokio::test]
async fn test_get_current_user_empty_bearer_token() {
    let (app, _ctx) = create_test_app().await;

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/users/me")
        .header(header::AUTHORIZATION, "Bearer ")
        .body(Body::empty())
        .expect("Failed to build request");

    let response = app
        .oneshot(request)
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let json_body = extract_json_response(response).await;
    assert!(
        json_body["error"]
            .as_str()
            .expect("Expected error field to be a string")
            .contains("Invalid or expired token")
    );
}

#[tokio::test]
async fn test_get_current_user_wrong_auth_scheme() {
    let (app, _ctx) = create_test_app().await;

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/users/me")
        .header(header::AUTHORIZATION, "Basic dGVzdDp0ZXN0")
        .body(Body::empty())
        .expect("Failed to build request");

    let response = app
        .oneshot(request)
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let json_body = extract_json_response(response).await;
    assert!(
        json_body["error"]
            .as_str()
            .expect("Expected error field to be a string")
            .contains("Missing or invalid authorization header")
    );
}
