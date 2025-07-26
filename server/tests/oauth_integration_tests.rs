use sqlx::SqlitePool;
use std::env;

use server::models::oauth::OAuthProvider;
use server::services::OAuthService;

mod common;

async fn setup_test_db() -> SqlitePool {
    common::setup_test_database().await
}

fn setup_test_env() {
    #[allow(unsafe_code)]
    unsafe {
        env::set_var("CLIENT_URL", "http://localhost:8080");
        env::set_var("GOOGLE_CLIENT_ID", "test_google_client_id");
        env::set_var("GOOGLE_CLIENT_SECRET", "test_google_client_secret");
        env::set_var("GITHUB_CLIENT_ID", "test_github_client_id");
        env::set_var("GITHUB_CLIENT_SECRET", "test_github_client_secret");
        env::set_var(
            "GOOGLE_REDIRECT_URI",
            "http://localhost:8000/api/auth/google/callback",
        );
        env::set_var(
            "GITHUB_REDIRECT_URI",
            "http://localhost:8000/api/auth/github/callback",
        );
    }
}

#[tokio::test]
async fn test_oauth_service_creation() {
    setup_test_env();
    let pool = setup_test_db().await;
    let service = OAuthService::new(pool);
    assert!(service.is_ok());
}

#[tokio::test]
async fn test_oauth_service_creation_missing_config() {
    // Clear all OAuth env vars
    #[allow(unsafe_code)]
    unsafe {
        env::remove_var("GOOGLE_CLIENT_ID");
        env::remove_var("GOOGLE_CLIENT_SECRET");
        env::remove_var("GITHUB_CLIENT_ID");
        env::remove_var("GITHUB_CLIENT_SECRET");
    }

    let pool = setup_test_db().await;
    let service = OAuthService::new(pool);
    assert!(service.is_err());
}

#[tokio::test]
async fn test_get_google_auth_url() {
    setup_test_env();
    let pool = setup_test_db().await;
    let service = OAuthService::new(pool.clone()).expect("Failed to create OAuthService");
    let auth_url = service.get_google_auth_url("test_state");

    assert!(auth_url.contains("accounts.google.com"));
    assert!(auth_url.contains("test_state"));
}

#[tokio::test]
async fn test_get_github_auth_url() {
    setup_test_env();
    let pool = setup_test_db().await;
    let service = OAuthService::new(pool.clone()).expect("Failed to create OAuthService");
    let auth_url = service.get_github_auth_url("test_state");

    assert!(auth_url.contains("github.com"));
    assert!(auth_url.contains("test_state"));
}

#[tokio::test]
async fn test_oauth_state_storage_and_validation() {
    setup_test_env();
    let pool = setup_test_db().await;
    let service = OAuthService::new(pool.clone()).expect("Failed to create OAuthService");

    let state = "test_state_123";
    let provider = OAuthProvider::Google;

    // Store state
    service
        .store_oauth_state(state, provider.clone(), None)
        .await
        .expect("Failed to store OAuth state");

    // Validate state - should succeed
    service
        .validate_oauth_state(state, provider.clone())
        .await
        .expect("Failed to validate OAuth state");

    // Try to validate again - should fail (one-time use)
    let result = service.validate_oauth_state(state, provider).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_oauth_state_wrong_provider() {
    setup_test_env();
    let pool = setup_test_db().await;
    let service = OAuthService::new(pool.clone()).expect("Failed to create OAuthService");

    let state = "test_state_456";

    // Store state for Google
    service
        .store_oauth_state(state, OAuthProvider::Google, None)
        .await
        .expect("Failed to store OAuth state");

    // Try to validate with GitHub - should fail
    let result = service
        .validate_oauth_state(state, OAuthProvider::GitHub)
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_oauth_state_expiration() {
    use chrono::{Duration, Utc};

    setup_test_env();
    let pool = setup_test_db().await;
    let service = OAuthService::new(pool.clone()).expect("Failed to create OAuthService");

    let state = "test_state_789";

    // Store state with immediate expiration using chrono
    let now = Utc::now();
    let expired_time = now - Duration::minutes(1);
    sqlx::query!(
        r#"
        INSERT INTO oauth_states (state, provider, redirect_uri, expires_at, created_at)
        VALUES (?1, ?2, ?3, ?4, ?5)
        "#,
        state,
        "google",
        None::<String>,
        expired_time,
        now
    )
    .execute(&pool)
    .await
    .expect("Failed to insert expired state");

    // Try to validate expired state - should fail
    let result = service
        .validate_oauth_state(state, OAuthProvider::Google)
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_cleanup_expired_states() {
    use chrono::{Duration, Utc};

    setup_test_env();
    let pool = setup_test_db().await;
    let service = OAuthService::new(pool.clone()).expect("Failed to create OAuthService");

    // Insert some expired and non-expired states
    let now = Utc::now();
    let expired_time = now - Duration::hours(1);
    let valid_time = now + Duration::hours(1);

    sqlx::query!(
        r#"
        INSERT INTO oauth_states (state, provider, redirect_uri, expires_at, created_at)
        VALUES (?1, ?2, ?3, ?4, ?5)
        "#,
        "expired1",
        "google",
        None::<String>,
        expired_time,
        now
    )
    .execute(&pool)
    .await
    .expect("Failed to insert expired state");

    sqlx::query!(
        r#"
        INSERT INTO oauth_states (state, provider, redirect_uri, expires_at, created_at)
        VALUES (?1, ?2, ?3, ?4, ?5)
        "#,
        "valid1",
        "google",
        None::<String>,
        valid_time,
        now
    )
    .execute(&pool)
    .await
    .expect("Failed to insert valid state");

    // Run cleanup
    let deleted = service
        .cleanup_expired_states()
        .await
        .expect("Failed to cleanup expired states");

    assert_eq!(deleted, 1);

    // Verify only valid state remains
    let count = sqlx::query_scalar!("SELECT COUNT(*) FROM oauth_states")
        .fetch_one(&pool)
        .await
        .expect("Failed to count states");

    assert_eq!(count, 1);
}
