// web-template/server/tests/auth_integration_tests.rs

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
use sqlx::{Pool, Sqlite};
use std::sync::Arc;
use tower::ServiceExt; // for `oneshot` and `ready`
use uuid::Uuid;

// Import the application modules we need for testing
use server::{
    routes::create_router,
    services::{AuthService, InviteService, OAuthService, UserServiceImpl},
};

// Test constants to avoid gitleaks false positives
const TEST_EMAIL: &str = "test@example.com";
const TEST_SECURE_PASS: &str = "secure_password_123";
const TEST_WEAK_PASS: &str = "weak";
const TEST_WRONG_PASS: &str = "wrong_password_123";
const TEST_CORRECT_PASS: &str = "correct_password_123";
const TEST_EMPTY_PASS: &str = "";

mod common;

/// Helper function to create a test database in memory
async fn create_test_db() -> Pool<Sqlite> {
    common::setup_test_database().await
}

/// Helper function to create the test app with database pool
async fn create_test_app() -> Router {
    let pool = create_test_db().await;
    create_test_app_with_pool(pool).await
}

/// Helper function to create the test app with a specific database pool
async fn create_test_app_with_pool(pool: Pool<Sqlite>) -> Router {
    // Set up JWT secret for testing
    unsafe {
        std::env::set_var(
            "JWT_SECRET",
            "test_secret_key_that_is_long_enough_for_testing",
        );
        // Set OAuth environment variables for testing
        std::env::set_var("GOOGLE_CLIENT_ID", "test_client_id");
        std::env::set_var("GOOGLE_CLIENT_SECRET", "test_client_secret");
        std::env::set_var("SERVER_URL", "http://localhost:8081");
        // Set static directory for testing (use /tmp as it always exists)
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
        .body(Body::from(
            serde_json::to_string(&body).expect("Failed to serialize JSON"),
        ))
        .expect("Failed to build request");

    app.oneshot(request).await.expect("Failed to send request")
}

/// Helper function to extract JSON response body
async fn extract_json_response(response: Response<Body>) -> Value {
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");
    serde_json::from_slice(&body).expect("Failed to parse JSON response")
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

    app.oneshot(request).await.expect("Failed to send request")
}

/// Helper function to create an invite in the test database
async fn create_test_invite(email: &str) -> Pool<Sqlite> {
    use chrono::Utc;

    let pool = create_test_db().await;

    // Create an invite for the test user
    let now = Utc::now();
    sqlx::query(
        r"
        INSERT INTO user_invites (id, email, invited_by, invited_at, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        ",
    )
    .bind(format!("test-invite-{}", Uuid::new_v4()))
    .bind(email.to_lowercase())
    .bind("test-admin")
    .bind(now)
    .bind(now)
    .bind(now)
    .execute(&pool)
    .await
    .expect("Failed to create test invite");

    pool
}

/// Helper function to register a user and get a valid JWT token
/// Note: This assumes the invite has already been created
async fn register_and_login_user(app: Router, email: &str, password: &str) -> String {
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
    login_body["auth_token"]
        .as_str()
        .expect("auth_token not found in response")
        .to_string()
}

#[tokio::test]
async fn test_register_user_success() {
    // Create invite first
    let pool = create_test_invite(TEST_EMAIL).await;
    let app = create_test_app_with_pool(pool).await;

    let test_email = TEST_EMAIL;
    let test_pass = TEST_SECURE_PASS;
    let payload = json!({
        "email": test_email,
        "password": test_pass
    });

    let response = send_json_request(app, Method::POST, "/api/auth/register", payload).await;

    assert_eq!(response.status(), StatusCode::CREATED);

    let json_body = extract_json_response(response).await;
    assert!(json_body["auth_user"]["id"].is_string());
    assert_eq!(json_body["auth_user"]["email"], "test@example.com");
    assert!(json_body["auth_user"]["created_at"].is_string());
    assert!(json_body["auth_user"]["updated_at"].is_string());
    assert!(json_body["payment_user"]["payment_required"].is_boolean());
}

#[tokio::test]
async fn test_register_user_without_invite() {
    let app = create_test_app().await;

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
}

#[tokio::test]
async fn test_register_user_invalid_email() {
    let app = create_test_app().await;

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
            .expect("error field not found")
            .contains("Email must be a valid email address")
    );
}

#[tokio::test]
async fn test_register_user_weak_password() {
    let app = create_test_app().await;

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
            .expect("error field not found")
            .contains("Password must be at least 12 characters long")
    );
}

#[tokio::test]
async fn test_register_user_duplicate_email() {
    let dup_email = "duplicate@example.com";

    // Create an invite for the test
    let pool = create_test_invite(dup_email).await;
    let app = create_test_app_with_pool(pool).await;

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
            .expect("error field not found")
            .contains("already exists")
            || json_body["error"]
                .as_str()
                .expect("error field not found")
                .contains("User already exists")
    );
}

#[tokio::test]
async fn test_register_user_missing_fields() {
    let app = create_test_app().await;

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
    let app = create_test_app().await;

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/register")
        .header("content-type", "application/json")
        .body(Body::empty())
        .expect("Failed to build request");

    let response = app.oneshot(request).await.expect("Failed to send request");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

// Login endpoint tests

#[tokio::test]
async fn test_login_user_success() {
    // Create invite for the test user
    let login_email = "login_test@example.com";
    let pool = create_test_invite(login_email).await;
    let app = create_test_app_with_pool(pool).await;

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
    let app = create_test_app().await;

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
            .expect("error field not found")
            .contains("Email must be a valid email address")
    );
}

#[tokio::test]
async fn test_login_user_nonexistent_email() {
    let app = create_test_app().await;

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
            .expect("error field not found")
            .contains("Invalid email or password")
    );
}

#[tokio::test]
async fn test_login_user_wrong_password() {
    // Create invite for the test user
    let wrong_pass_email = "wrong_password_test@example.com";
    let pool = create_test_invite(wrong_pass_email).await;
    let app = create_test_app_with_pool(pool).await;

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
            .expect("error field not found")
            .contains("Invalid email or password")
    );
}

#[tokio::test]
async fn test_login_user_missing_fields() {
    let app = create_test_app().await;

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
    let app = create_test_app().await;

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
            .expect("error field not found")
            .contains("Password is required")
    );
}

// Protected endpoint tests

#[tokio::test]
async fn test_get_current_user_success() {
    // Create invite for the test user
    let protected_email = "protected_test@example.com";
    let pool = create_test_invite(protected_email).await;
    let app = create_test_app_with_pool(pool).await;

    // Register and login to get a valid token
    let test_pass = TEST_SECURE_PASS;
    let token = register_and_login_user(app.clone(), protected_email, test_pass).await;

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
    let app = create_test_app().await;

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/users/me")
        .body(Body::empty())
        .expect("Failed to build request");

    let response = app.oneshot(request).await.expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let json_body = extract_json_response(response).await;
    assert!(
        json_body["error"]
            .as_str()
            .expect("error field not found")
            .contains("Missing or invalid authorization header")
    );
}

#[tokio::test]
async fn test_get_current_user_invalid_token() {
    let app = create_test_app().await;

    let response =
        send_authenticated_request(app, Method::GET, "/api/users/me", "invalid_token").await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let json_body = extract_json_response(response).await;
    assert!(
        json_body["error"]
            .as_str()
            .expect("error field not found")
            .contains("Invalid or expired token")
    );
}

#[tokio::test]
async fn test_get_current_user_malformed_token() {
    let app = create_test_app().await;

    let response =
        send_authenticated_request(app, Method::GET, "/api/users/me", "Bearer malformed").await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let json_body = extract_json_response(response).await;
    assert!(
        json_body["error"]
            .as_str()
            .expect("error field not found")
            .contains("Invalid or expired token")
    );
}

#[tokio::test]
async fn test_get_current_user_empty_bearer_token() {
    let app = create_test_app().await;

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/users/me")
        .header(header::AUTHORIZATION, "Bearer ")
        .body(Body::empty())
        .expect("Failed to build request");

    let response = app.oneshot(request).await.expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let json_body = extract_json_response(response).await;
    assert!(
        json_body["error"]
            .as_str()
            .expect("error field not found")
            .contains("Invalid or expired token")
    );
}

#[tokio::test]
async fn test_get_current_user_wrong_auth_scheme() {
    let app = create_test_app().await;

    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/users/me")
        .header(header::AUTHORIZATION, "Basic dGVzdDp0ZXN0")
        .body(Body::empty())
        .expect("Failed to build request");

    let response = app.oneshot(request).await.expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let json_body = extract_json_response(response).await;
    assert!(
        json_body["error"]
            .as_str()
            .expect("error field not found")
            .contains("Missing or invalid authorization header")
    );
}
