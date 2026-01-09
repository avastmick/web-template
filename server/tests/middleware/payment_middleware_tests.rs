#![allow(clippy::unwrap_used)]

//! Payment middleware integration tests
//!
//! Tests for payment verification middleware covering:
//! - Paid user allowed
//! - Unpaid user blocked
//! - Payment bypass for invite holders

use axum::{
    Router,
    body::Body,
    http::{Method, Request, StatusCode, header},
    response::Response,
    routing::get,
};
use chrono::{Duration, Utc};
use serde_json::Value;
use server::core::AppState;
use server::handlers::auth_handler::RegisterUserPayload;
use server::middleware::payment_middleware::PaymentRequired;
use server::models::payment::{PaymentStatus, PaymentType};
use server::services::payment::PaymentDbOperations;
use server::services::{AuthService, InviteService, PaymentService, UserServiceImpl};
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceExt;
use uuid::Uuid;

// ============================================================================
// Test Setup
// ============================================================================

async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    pool
}

fn setup_test_env() {
    #[allow(unsafe_code)]
    unsafe {
        std::env::set_var("JWT_SECRET", "test_secret_key_that_is_long_enough_for_testing");
        std::env::set_var("STRIPE_SECRET_KEY", "sk_test_mock");
        std::env::set_var("STRIPE_WEBHOOK_ENDPOINT_SECRET", "whsec_test_mock");
        std::env::set_var("OPENROUTER_API_KEY", "test_api_key");
        std::env::set_var("AI_DEFAULT_MODEL", "gpt-4o");
    }
}

fn create_test_app_state(pool: &SqlitePool) -> Arc<AppState> {
    setup_test_env();

    let user_service = Arc::new(UserServiceImpl::new(pool.clone()));
    let auth_service = Arc::new(AuthService::new().expect("Failed to create auth service"));
    let invite_service = Arc::new(InviteService::new(pool.clone()));
    let ai_service_inner = server::services::AiService::new().expect("Failed to create AI service");
    let ai_service = Arc::new(RwLock::new(ai_service_inner));
    let ai_data_service = Arc::new(server::services::AiDataService::new(pool.clone()));
    let payment_service = Arc::new(PaymentService::new(pool.clone()).expect("Failed to create payment service"));

    Arc::new(AppState {
        user: user_service,
        auth: auth_service,
        invite: invite_service,
        ai: ai_service,
        ai_data: ai_data_service,
        payment: payment_service,
    })
}

/// Create a test router with a payment-protected endpoint
fn create_test_router(state: Arc<AppState>) -> Router {
    async fn payment_protected_handler(_payment: PaymentRequired) -> String {
        "Premium content".to_string()
    }

    Router::new()
        .route("/premium", get(payment_protected_handler))
        .with_state(state)
}

async fn create_test_user(state: &Arc<AppState>, email: &str) -> server::models::User {
    state
        .user
        .create_user(&RegisterUserPayload {
            email: email.to_string(),
            password: "test_password123".to_string(),
        })
        .await
        .expect("Failed to create test user")
}

fn generate_token(state: &Arc<AppState>, user_id: Uuid, email: &str) -> String {
    state.auth.generate_token(user_id, email).unwrap()
}

async fn send_request(app: Router, token: Option<&str>) -> Response {
    let mut builder = Request::builder()
        .method(Method::GET)
        .uri("/premium");

    if let Some(t) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {t}"));
    }

    let request = builder.body(Body::empty()).unwrap();
    app.oneshot(request).await.unwrap()
}

async fn get_response_body(response: Response) -> Value {
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&body).unwrap_or(Value::Null)
}

// ============================================================================
// Paid User Allowed Tests
// ============================================================================

#[tokio::test]
async fn test_paid_user_allowed_with_active_subscription() {
    let pool = setup_test_db().await;
    let state = create_test_app_state(&pool);
    let app = create_test_router(state.clone());

    let email = "paid_sub@test.com";
    let user = create_test_user(&state, email).await;

    // Create active subscription payment
    let payment = state
        .payment
        .create_payment(user.id, PaymentType::Subscription)
        .await
        .unwrap();

    state
        .payment
        .update_payment_status(payment.id, PaymentStatus::Active)
        .await
        .unwrap();

    let token = generate_token(&state, user.id, email);
    let response = send_request(app, Some(&token)).await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(body_str.contains("Premium content"));
}

#[tokio::test]
async fn test_paid_user_allowed_with_one_time_payment() {
    let pool = setup_test_db().await;
    let state = create_test_app_state(&pool);
    let app = create_test_router(state.clone());

    let email = "paid_onetime@test.com";
    let user = create_test_user(&state, email).await;

    // Create active one-time payment
    let payment = state
        .payment
        .create_payment(user.id, PaymentType::OneTime)
        .await
        .unwrap();

    state
        .payment
        .update_payment_status(payment.id, PaymentStatus::Active)
        .await
        .unwrap();

    let token = generate_token(&state, user.id, email);
    let response = send_request(app, Some(&token)).await;

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_paid_user_with_active_subscription_and_future_expiry() {
    let pool = setup_test_db().await;
    let state = create_test_app_state(&pool);
    let app = create_test_router(state.clone());

    let email = "paid_future@test.com";
    let user = create_test_user(&state, email).await;

    let now = Utc::now();
    let end_date = now + Duration::days(30);
    let payment_id = Uuid::new_v4();

    // Create active payment with future expiry via SQL
    sqlx::query(
        r"INSERT INTO user_payments
         (id, user_id, payment_type, payment_status, subscription_end_date, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(payment_id.to_string())
    .bind(user.id.to_string())
    .bind("subscription")
    .bind("active")
    .bind(end_date.to_rfc3339())
    .bind(now.to_rfc3339())
    .bind(now.to_rfc3339())
    .execute(&pool)
    .await
    .unwrap();

    let token = generate_token(&state, user.id, email);
    let response = send_request(app, Some(&token)).await;

    assert_eq!(response.status(), StatusCode::OK);
}

// ============================================================================
// Unpaid User Blocked Tests
// ============================================================================

#[tokio::test]
async fn test_unpaid_user_blocked_no_payment() {
    let pool = setup_test_db().await;
    let state = create_test_app_state(&pool);
    let app = create_test_router(state.clone());

    let email = "unpaid@test.com";
    let user = create_test_user(&state, email).await;

    // User exists but has no payment record
    let token = generate_token(&state, user.id, email);
    let response = send_request(app, Some(&token)).await;

    assert_eq!(response.status(), StatusCode::PAYMENT_REQUIRED);

    let body = get_response_body(response).await;
    assert_eq!(body["payment_required"], true);
}

#[tokio::test]
async fn test_unpaid_user_blocked_pending_payment() {
    let pool = setup_test_db().await;
    let state = create_test_app_state(&pool);
    let app = create_test_router(state.clone());

    let email = "pending_pay@test.com";
    let user = create_test_user(&state, email).await;

    // Create pending payment (not active)
    let _payment = state
        .payment
        .create_payment(user.id, PaymentType::Subscription)
        .await
        .unwrap();

    // Payment status defaults to Pending

    let token = generate_token(&state, user.id, email);
    let response = send_request(app, Some(&token)).await;

    assert_eq!(response.status(), StatusCode::PAYMENT_REQUIRED);
}

#[tokio::test]
async fn test_unpaid_user_blocked_cancelled_payment() {
    let pool = setup_test_db().await;
    let state = create_test_app_state(&pool);
    let app = create_test_router(state.clone());

    let email = "cancelled@test.com";
    let user = create_test_user(&state, email).await;

    // Create and cancel payment
    let payment = state
        .payment
        .create_payment(user.id, PaymentType::Subscription)
        .await
        .unwrap();

    state
        .payment
        .update_payment_status(payment.id, PaymentStatus::Cancelled)
        .await
        .unwrap();

    let token = generate_token(&state, user.id, email);
    let response = send_request(app, Some(&token)).await;

    assert_eq!(response.status(), StatusCode::PAYMENT_REQUIRED);
}

#[tokio::test]
async fn test_unpaid_user_blocked_expired_payment() {
    let pool = setup_test_db().await;
    let state = create_test_app_state(&pool);
    let app = create_test_router(state.clone());

    let email = "expired_pay@test.com";
    let user = create_test_user(&state, email).await;

    // Create and expire payment
    let payment = state
        .payment
        .create_payment(user.id, PaymentType::Subscription)
        .await
        .unwrap();

    state
        .payment
        .update_payment_status(payment.id, PaymentStatus::Expired)
        .await
        .unwrap();

    let token = generate_token(&state, user.id, email);
    let response = send_request(app, Some(&token)).await;

    assert_eq!(response.status(), StatusCode::PAYMENT_REQUIRED);
}

#[tokio::test]
async fn test_unpaid_user_blocked_failed_payment() {
    let pool = setup_test_db().await;
    let state = create_test_app_state(&pool);
    let app = create_test_router(state.clone());

    let email = "failed_pay@test.com";
    let user = create_test_user(&state, email).await;

    // Create and fail payment
    let payment = state
        .payment
        .create_payment(user.id, PaymentType::OneTime)
        .await
        .unwrap();

    state
        .payment
        .update_payment_status(payment.id, PaymentStatus::Failed)
        .await
        .unwrap();

    let token = generate_token(&state, user.id, email);
    let response = send_request(app, Some(&token)).await;

    assert_eq!(response.status(), StatusCode::PAYMENT_REQUIRED);
}

#[tokio::test]
async fn test_unpaid_user_blocked_expired_subscription_date() {
    let pool = setup_test_db().await;
    let state = create_test_app_state(&pool);
    let app = create_test_router(state.clone());

    let email = "expired_sub_date@test.com";
    let user = create_test_user(&state, email).await;

    let now = Utc::now();
    let past_date = now - Duration::days(7);
    let payment_id = Uuid::new_v4();

    // Create payment with past subscription_end_date via SQL
    sqlx::query(
        r"INSERT INTO user_payments
         (id, user_id, payment_type, payment_status, subscription_end_date, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(payment_id.to_string())
    .bind(user.id.to_string())
    .bind("subscription")
    .bind("active")
    .bind(past_date.to_rfc3339())
    .bind(now.to_rfc3339())
    .bind(now.to_rfc3339())
    .execute(&pool)
    .await
    .unwrap();

    let token = generate_token(&state, user.id, email);
    let response = send_request(app, Some(&token)).await;

    // Should be blocked because subscription_end_date is in the past
    assert_eq!(response.status(), StatusCode::PAYMENT_REQUIRED);
}

// ============================================================================
// Invite Bypass Tests
// ============================================================================

#[tokio::test]
async fn test_invite_holder_allowed_without_payment() {
    let pool = setup_test_db().await;
    let state = create_test_app_state(&pool);
    let app = create_test_router(state.clone());

    let email = "invited@test.com";

    // Create invite first
    state
        .invite
        .create_invite(email, Some("admin".to_string()), None)
        .await
        .unwrap();

    // Create user (no payment)
    let user = create_test_user(&state, email).await;

    let token = generate_token(&state, user.id, email);
    let response = send_request(app, Some(&token)).await;

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(body_str.contains("Premium content"));
}

#[tokio::test]
async fn test_invite_holder_with_future_expiry_allowed() {
    let pool = setup_test_db().await;
    let state = create_test_app_state(&pool);
    let app = create_test_router(state.clone());

    let email = "invite_future@test.com";
    let future_expiry = Utc::now() + Duration::days(30);

    // Create invite with future expiry
    state
        .invite
        .create_invite(email, Some("admin".to_string()), Some(future_expiry))
        .await
        .unwrap();

    let user = create_test_user(&state, email).await;

    let token = generate_token(&state, user.id, email);
    let response = send_request(app, Some(&token)).await;

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_expired_invite_blocks_without_payment() {
    let pool = setup_test_db().await;
    let state = create_test_app_state(&pool);
    let app = create_test_router(state.clone());

    let email = "expired_invite@test.com";
    let past_expiry = Utc::now() - Duration::hours(1);

    // Create invite with past expiry
    state
        .invite
        .create_invite(email, Some("admin".to_string()), Some(past_expiry))
        .await
        .unwrap();

    let user = create_test_user(&state, email).await;

    let token = generate_token(&state, user.id, email);
    let response = send_request(app, Some(&token)).await;

    // Expired invite should not grant access without payment
    assert_eq!(response.status(), StatusCode::PAYMENT_REQUIRED);
}

#[tokio::test]
async fn test_used_invite_with_active_payment_allowed() {
    let pool = setup_test_db().await;
    let state = create_test_app_state(&pool);
    let app = create_test_router(state.clone());

    let email = "used_invite@test.com";

    // Create and use invite
    state
        .invite
        .create_invite(email, Some("admin".to_string()), None)
        .await
        .unwrap();

    state.invite.mark_invite_used(email).await.unwrap();

    let user = create_test_user(&state, email).await;

    // Create active payment
    let payment = state
        .payment
        .create_payment(user.id, PaymentType::Subscription)
        .await
        .unwrap();

    state
        .payment
        .update_payment_status(payment.id, PaymentStatus::Active)
        .await
        .unwrap();

    let token = generate_token(&state, user.id, email);
    let response = send_request(app, Some(&token)).await;

    // Should be allowed because of active payment (invite was used)
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_case_insensitive_invite_email() {
    let pool = setup_test_db().await;
    let state = create_test_app_state(&pool);
    let app = create_test_router(state.clone());

    let invite_email = "CaseTest@Test.Com";
    let user_email = "casetest@test.com"; // Different case

    // Create invite with different case
    state
        .invite
        .create_invite(invite_email, Some("admin".to_string()), None)
        .await
        .unwrap();

    let user = create_test_user(&state, user_email).await;

    let token = generate_token(&state, user.id, user_email);
    let response = send_request(app, Some(&token)).await;

    // Should work because invite lookup is case-insensitive
    assert_eq!(response.status(), StatusCode::OK);
}

// ============================================================================
// Combined Scenarios
// ============================================================================

#[tokio::test]
async fn test_invite_holder_with_payment_uses_both() {
    let pool = setup_test_db().await;
    let state = create_test_app_state(&pool);
    let app = create_test_router(state.clone());

    let email = "invite_and_payment@test.com";

    // Create invite
    state
        .invite
        .create_invite(email, Some("admin".to_string()), None)
        .await
        .unwrap();

    let user = create_test_user(&state, email).await;

    // Also create active payment
    let payment = state
        .payment
        .create_payment(user.id, PaymentType::Subscription)
        .await
        .unwrap();

    state
        .payment
        .update_payment_status(payment.id, PaymentStatus::Active)
        .await
        .unwrap();

    let token = generate_token(&state, user.id, email);
    let response = send_request(app, Some(&token)).await;

    // Should be allowed (either condition satisfied)
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_no_token_returns_unauthorized_before_payment_check() {
    let pool = setup_test_db().await;
    let state = create_test_app_state(&pool);
    let app = create_test_router(state);

    // No token - should fail at auth step, not payment step
    let response = send_request(app, None).await;

    // Should be 401 Unauthorized, not 402 Payment Required
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_invalid_token_returns_unauthorized_before_payment_check() {
    let pool = setup_test_db().await;
    let state = create_test_app_state(&pool);
    let app = create_test_router(state);

    let response = send_request(app, Some("invalid.token.here")).await;

    // Should be 401 Unauthorized, not 402 Payment Required
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

// ============================================================================
// Response Format Tests
// ============================================================================

#[tokio::test]
async fn test_payment_required_response_format() {
    let pool = setup_test_db().await;
    let state = create_test_app_state(&pool);
    let app = create_test_router(state.clone());

    let email = "response_test@test.com";
    let user = create_test_user(&state, email).await;

    let token = generate_token(&state, user.id, email);
    let response = send_request(app, Some(&token)).await;

    assert_eq!(response.status(), StatusCode::PAYMENT_REQUIRED);

    let body = get_response_body(response).await;

    // Check response structure
    assert!(body.is_object());
    assert!(body.get("error").is_some());
    assert!(body.get("payment_required").is_some());
    assert_eq!(body["payment_required"], true);
    assert!(body["error"]
        .as_str()
        .unwrap_or("")
        .contains("Payment required"));
}
