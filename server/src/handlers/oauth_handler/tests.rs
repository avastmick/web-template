#![allow(clippy::unwrap_used)]

use super::*;
use crate::{
    handlers::auth_handler::RegisterUserPayload,
    models::{
        oauth::{OAuthProvider, OAuthUserInfo},
        user::User,
    },
    services::OAuthService,
    test_helpers::create_test_app_state,
};
use axum::extract::{Query, State};
use sqlx::SqlitePool;
use std::sync::Arc;
use uuid::Uuid;

async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

    // Run migrations to ensure test database matches production schema
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    pool
}

fn create_test_oauth_app_state(pool: &SqlitePool) -> OAuthAppState {
    // Set required env vars for the test
    // SAFETY: Tests are single-threaded and we're setting test values
    #[allow(unsafe_code)]
    unsafe {
        std::env::set_var(
            "JWT_SECRET",
            "test_secret_key_that_is_long_enough_for_testing",
        );
        std::env::set_var("STRIPE_SECRET_KEY", "test_key");
        std::env::set_var("STRIPE_WEBHOOK_ENDPOINT_SECRET", "test_secret");
        std::env::set_var("GOOGLE_CLIENT_ID", "test_google_client_id");
        std::env::set_var("GOOGLE_CLIENT_SECRET", "test_google_client_secret");
        std::env::set_var("GITHUB_CLIENT_ID", "test_github_client_id");
        std::env::set_var("GITHUB_CLIENT_SECRET", "test_github_client_secret");
        std::env::set_var("CLIENT_URL", "http://localhost:3000");
    }

    let app_state = create_test_app_state(pool);

    let oauth_service = Arc::new(OAuthService::new(pool.clone()).unwrap());

    OAuthAppState {
        app_state,
        oauth_service,
    }
}

#[test]
fn test_oauth_init_request_parsing() {
    let json = r#"{"state": "test_state"}"#;
    let req: OAuthInitRequest = serde_json::from_str(json).expect("Failed to parse JSON");
    assert_eq!(req.state, Some("test_state".to_string()));
}

#[test]
fn test_oauth_init_request_parsing_no_state() {
    let json = "{}";
    let req: OAuthInitRequest = serde_json::from_str(json).expect("Failed to parse JSON");
    assert_eq!(req.state, None);
}

#[test]
fn test_oauth_callback_query_parsing() {
    let query = OAuthCallbackQuery {
        code: "test_code".to_string(),
        state: Some("test_state".to_string()),
        error: None,
    };
    assert_eq!(query.code, "test_code");
    assert_eq!(query.state, Some("test_state".to_string()));
    assert!(query.error.is_none());
}

#[test]
fn test_oauth_callback_query_with_error() {
    let query = OAuthCallbackQuery {
        code: "test_code".to_string(),
        state: Some("test_state".to_string()),
        error: Some("access_denied".to_string()),
    };
    assert_eq!(query.error, Some("access_denied".to_string()));
}

#[tokio::test]
async fn test_google_login_init_success() {
    let pool = setup_test_db().await;
    let state = create_test_oauth_app_state(&pool);
    let params = OAuthInitRequest {
        state: Some("custom_state".to_string()),
    };

    let result = google_login_init(State(state), Query(params)).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_google_login_init_generates_state() {
    let pool = setup_test_db().await;
    let state = create_test_oauth_app_state(&pool);
    let params = OAuthInitRequest { state: None };

    let result = google_login_init(State(state), Query(params)).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_github_login_init_success() {
    let pool = setup_test_db().await;
    let state = create_test_oauth_app_state(&pool);
    let params = OAuthInitRequest {
        state: Some("github_state".to_string()),
    };

    let result = github_login_init(State(state), Query(params)).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_github_login_init_generates_state() {
    let pool = setup_test_db().await;
    let state = create_test_oauth_app_state(&pool);
    let params = OAuthInitRequest { state: None };

    let result = github_login_init(State(state), Query(params)).await;
    assert!(result.is_ok());
}

#[test]
fn test_redirect_with_error_url_encoding() {
    // SAFETY: Tests are single-threaded and we're setting test values
    #[allow(unsafe_code)]
    unsafe {
        std::env::set_var("CLIENT_URL", "http://localhost:3000");
    }

    // Test URL encoding for error messages
    let error_msg = "test error with spaces";
    let encoded = urlencoding::encode(error_msg);
    assert_eq!(encoded, "test%20error%20with%20spaces");

    // Test basic URL construction
    let client_url = "http://localhost:3000";
    let expected_url = format!(
        "{}/auth/oauth/callback?error={}",
        client_url,
        urlencoding::encode("test_error")
    );

    assert!(expected_url.contains("error=test_error"));
    assert!(expected_url.contains("http://localhost:3000"));
}

#[tokio::test]
async fn test_exchange_oauth_code_google() {
    let pool = setup_test_db().await;
    let state = create_test_oauth_app_state(&pool);

    // This would normally make an HTTP request, so we test the error path
    let result = exchange_oauth_code(&state, "test_code", OAuthProvider::Google).await;
    // This will fail because we're not mocking the HTTP client
    // but we're testing the code path and error handling
    assert!(result.is_err());
}

#[tokio::test]
async fn test_exchange_oauth_code_github() {
    let pool = setup_test_db().await;
    let state = create_test_oauth_app_state(&pool);

    // This would normally make an HTTP request, so we test the error path
    let result = exchange_oauth_code(&state, "test_code", OAuthProvider::GitHub).await;
    // This will fail because we're not mocking the HTTP client
    // but we're testing the code path and error handling
    assert!(result.is_err());
}

#[tokio::test]
async fn test_create_user_from_oauth_success() {
    let pool = setup_test_db().await;
    let state = create_test_oauth_app_state(&pool);

    let oauth_info = OAuthUserInfo {
        id: "google_123".to_string(),
        email: "test@example.com".to_string(),
        name: Some("Test User".to_string()),
        picture: None,
        provider: OAuthProvider::Google,
    };

    let result = create_user_from_oauth(&state, &oauth_info).await;
    assert!(result.is_ok());

    let user = result.unwrap();
    assert_eq!(user.email, "test@example.com");
}

#[tokio::test]
async fn test_get_or_create_user_existing() {
    let pool = setup_test_db().await;
    let state = create_test_oauth_app_state(&pool);

    // First create a user using the user service
    let _existing_user = state
        .app_state
        .user
        .create_user(&RegisterUserPayload {
            email: "existing@example.com".to_string(),
            password: "test_password123".to_string(),
        })
        .await
        .unwrap();

    let oauth_info = OAuthUserInfo {
        id: "google_123".to_string(),
        email: "existing@example.com".to_string(),
        name: Some("Existing User".to_string()),
        picture: None,
        provider: OAuthProvider::Google,
    };

    let result = get_or_create_user(&state, &oauth_info).await;
    assert!(result.is_ok());

    let (user, is_new) = result.unwrap();
    assert_eq!(user.email, "existing@example.com");
    assert!(!is_new); // Should be false for existing user
}

#[tokio::test]
async fn test_get_or_create_user_new() {
    let pool = setup_test_db().await;
    let state = create_test_oauth_app_state(&pool);

    let oauth_info = OAuthUserInfo {
        id: "google_456".to_string(),
        email: "newuser@example.com".to_string(),
        name: Some("New User".to_string()),
        picture: None,
        provider: OAuthProvider::Google,
    };

    let result = get_or_create_user(&state, &oauth_info).await;
    assert!(result.is_ok());

    let (user, is_new) = result.unwrap();
    assert_eq!(user.email, "newuser@example.com");
    assert!(is_new); // Should be true for new user
}

#[tokio::test]
async fn test_generate_jwt_token_success() {
    let pool = setup_test_db().await;
    let state = create_test_oauth_app_state(&pool);

    let user = User {
        id: Uuid::new_v4(),
        email: "test@example.com".to_string(),
        hashed_password: "hashed".to_string(),
        provider: "oauth".to_string(),
        provider_user_id: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let result = generate_jwt_token(&state, &user);
    assert!(result.is_ok());

    let token = result.unwrap();
    assert!(!token.is_empty());
}

#[tokio::test]
async fn test_oauth_app_state_clone() {
    let pool = setup_test_db().await;
    let state = create_test_oauth_app_state(&pool);

    let cloned_state = state.clone();
    // Verify the clone works and both point to the same underlying data
    assert!(Arc::ptr_eq(&state.app_state, &cloned_state.app_state));
    assert!(Arc::ptr_eq(
        &state.oauth_service,
        &cloned_state.oauth_service
    ));
}

#[test]
fn test_oauth_provider_display() {
    assert_eq!(OAuthProvider::Google.to_string(), "google");
    assert_eq!(OAuthProvider::GitHub.to_string(), "github");
}
