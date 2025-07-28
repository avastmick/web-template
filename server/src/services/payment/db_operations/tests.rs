#![allow(clippy::unwrap_used)]

use super::*;
use crate::{
    handlers::auth_handler::RegisterUserPayload,
    models::payment::{PaymentStatus, PaymentType, UserPaymentFromDb},
    services::UserServiceImpl,
};
use sqlx::SqlitePool;
use std::{str::FromStr, sync::Arc};

struct TestContext {
    pool: SqlitePool,
    user_service: Arc<UserServiceImpl>,
}

impl TestContext {
    async fn new() -> Self {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        // Run migrations to ensure test database matches production schema
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        let user_service = Arc::new(UserServiceImpl::new(pool.clone()));

        Self { pool, user_service }
    }

    async fn create_test_user(&self, email: &str) -> Uuid {
        let user = self
            .user_service
            .create_user(&RegisterUserPayload {
                email: email.to_string(),
                password: "test_password123".to_string(),
            })
            .await
            .unwrap();
        user.id
    }
}

fn create_test_service(pool: SqlitePool) -> PaymentService {
    // Set required env vars for the test
    // SAFETY: Tests are single-threaded and we're setting test values
    #[allow(unsafe_code)]
    unsafe {
        std::env::set_var("STRIPE_SECRET_KEY", "test_key");
        std::env::set_var("STRIPE_WEBHOOK_ENDPOINT_SECRET", "test_secret");
    }
    PaymentService::new(pool).unwrap()
}

#[tokio::test]
async fn test_create_payment() {
    let ctx = TestContext::new().await;
    let service = create_test_service(ctx.pool.clone());

    let user_id = ctx.create_test_user("test_create@example.com").await;
    let payment = service
        .create_payment(user_id, PaymentType::Subscription)
        .await
        .unwrap();

    assert_eq!(payment.user_id, user_id);
    assert_eq!(payment.payment_type, PaymentType::Subscription);
    assert_eq!(payment.payment_status, PaymentStatus::Pending);
    assert_eq!(payment.currency, "usd");
}

#[tokio::test]
async fn test_get_payment_by_id() {
    let ctx = TestContext::new().await;
    let service = create_test_service(ctx.pool.clone());

    let user_id = ctx.create_test_user("test_get_by_id@example.com").await;
    let created_payment = service
        .create_payment(user_id, PaymentType::OneTime)
        .await
        .unwrap();

    let fetched_payment = service
        .get_payment_by_id(created_payment.id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(fetched_payment.id, created_payment.id);
    assert_eq!(fetched_payment.user_id, user_id);
    assert_eq!(fetched_payment.payment_type, PaymentType::OneTime);
}

#[tokio::test]
async fn test_get_payment_by_id_not_found() {
    let ctx = TestContext::new().await;
    let service = create_test_service(ctx.pool);

    let result = service.get_payment_by_id(Uuid::new_v4()).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_get_payment_by_user_id() {
    let ctx = TestContext::new().await;
    let service = create_test_service(ctx.pool.clone());

    let user_id = ctx.create_test_user("test_get_by_user@example.com").await;

    // Create multiple payments
    for _ in 0..3 {
        service
            .create_payment(user_id, PaymentType::Subscription)
            .await
            .unwrap();
    }

    let payment = service
        .get_payment_by_user_id(user_id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(payment.user_id, user_id);
}

#[tokio::test]
async fn test_get_active_payment_for_user() {
    let ctx = TestContext::new().await;
    let service = create_test_service(ctx.pool.clone());

    let user_id = ctx
        .create_test_user("test_active_payment@example.com")
        .await;

    // Create a pending payment
    let pending_payment = service
        .create_payment(user_id, PaymentType::Subscription)
        .await
        .unwrap();

    // No active payment yet
    let active = service.get_active_payment_for_user(user_id).await.unwrap();
    assert!(active.is_none());

    // Update payment to active
    service
        .update_payment_status(pending_payment.id, PaymentStatus::Active)
        .await
        .unwrap();

    // Now should find active payment
    let active = service
        .get_active_payment_for_user(user_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(active.payment_status, PaymentStatus::Active);
}

#[tokio::test]
async fn test_update_stripe_customer_id() {
    let ctx = TestContext::new().await;
    let service = create_test_service(ctx.pool.clone());

    let user_id = ctx
        .create_test_user("test_stripe_customer@example.com")
        .await;
    let payment = service
        .create_payment(user_id, PaymentType::Subscription)
        .await
        .unwrap();

    assert!(payment.stripe_customer_id.is_none());

    service
        .update_stripe_customer_id(payment.id, "cus_test123")
        .await
        .unwrap();

    let updated = service
        .get_payment_by_id(payment.id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(updated.stripe_customer_id, Some("cus_test123".to_string()));
}

#[tokio::test]
async fn test_update_payment_after_checkout() {
    let ctx = TestContext::new().await;
    let service = create_test_service(ctx.pool.clone());

    let user_id = ctx.create_test_user("test_checkout@example.com").await;
    let payment = service
        .create_payment(user_id, PaymentType::Subscription)
        .await
        .unwrap();

    let update = CheckoutUpdate {
        payment_id: payment.id,
        stripe_subscription_id: Some("sub_test123".to_string()),
        stripe_payment_intent_id: Some("pi_test123".to_string()),
        amount_cents: 999,
        currency: "usd".to_string(),
        subscription_start: Some(Utc::now()),
        subscription_end: Some(Utc::now() + chrono::Duration::days(30)),
    };

    service
        .update_payment_after_checkout(&update)
        .await
        .unwrap();

    let updated = service
        .get_payment_by_id(payment.id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(updated.payment_status, PaymentStatus::Active);
    assert_eq!(
        updated.stripe_subscription_id,
        Some("sub_test123".to_string())
    );
    assert_eq!(
        updated.stripe_payment_intent_id,
        Some("pi_test123".to_string())
    );
    assert_eq!(updated.amount_cents, Some(999));
    assert!(updated.subscription_start_date.is_some());
    assert!(updated.subscription_end_date.is_some());
}

#[tokio::test]
async fn test_update_payment_status() {
    let ctx = TestContext::new().await;
    let service = create_test_service(ctx.pool.clone());

    let user_id = ctx.create_test_user("test_update_status@example.com").await;
    let payment = service
        .create_payment(user_id, PaymentType::OneTime)
        .await
        .unwrap();

    assert_eq!(payment.payment_status, PaymentStatus::Pending);

    // Update to active
    service
        .update_payment_status(payment.id, PaymentStatus::Active)
        .await
        .unwrap();

    let updated = service
        .get_payment_by_id(payment.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updated.payment_status, PaymentStatus::Active);

    // Update to cancelled
    service
        .update_payment_status(payment.id, PaymentStatus::Cancelled)
        .await
        .unwrap();

    let updated = service
        .get_payment_by_id(payment.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updated.payment_status, PaymentStatus::Cancelled);
}

#[tokio::test]
async fn test_store_webhook_event() {
    let ctx = TestContext::new().await;
    let service = create_test_service(ctx.pool);

    let event_data = serde_json::json!({
        "id": "evt_test123",
        "type": "payment_intent.succeeded",
        "data": {
            "object": {
                "id": "pi_test123"
            }
        }
    });

    let event_id = service
        .store_webhook_event("evt_test123", "payment_intent.succeeded", event_data)
        .await
        .unwrap();

    assert!(!event_id.is_nil());
}

#[tokio::test]
async fn test_is_webhook_event_processed() {
    let ctx = TestContext::new().await;
    let service = create_test_service(ctx.pool);

    let stripe_event_id = "evt_test456";
    let event_data = serde_json::json!({});

    // Store event
    let event_id = service
        .store_webhook_event(stripe_event_id, "test.event", event_data)
        .await
        .unwrap();

    // Should not be processed yet
    let processed = service
        .is_webhook_event_processed(stripe_event_id)
        .await
        .unwrap();
    assert!(!processed);

    // Mark as processed
    service
        .mark_webhook_event_processed(event_id)
        .await
        .unwrap();

    // Should now be processed
    let processed = service
        .is_webhook_event_processed(stripe_event_id)
        .await
        .unwrap();
    assert!(processed);
}

#[tokio::test]
async fn test_payment_conversion_from_db() {
    let db_row = UserPaymentFromDb {
        id: Some(Uuid::new_v4().to_string()),
        user_id: Some(Uuid::new_v4().to_string()),
        stripe_customer_id: Some("cus_test".to_string()),
        stripe_subscription_id: None,
        stripe_payment_intent_id: None,
        payment_status: Some("active".to_string()),
        payment_type: Some("subscription".to_string()),
        amount_cents: Some(1000),
        currency: Some("usd".to_string()),
        subscription_start_date: Some(Utc::now().to_rfc3339()),
        subscription_end_date: Some((Utc::now() + chrono::Duration::days(30)).to_rfc3339()),
        subscription_cancelled_at: None,
        last_payment_date: Some(Utc::now().to_rfc3339()),
        created_at: Some(Utc::now().to_rfc3339()),
        updated_at: Some(Utc::now().to_rfc3339()),
    };

    let payment = UserPayment::try_from(db_row).unwrap();
    assert_eq!(payment.payment_status, PaymentStatus::Active);
    assert_eq!(payment.payment_type, PaymentType::Subscription);
    assert_eq!(payment.amount_cents, Some(1000));
    assert!(payment.is_active());
}

#[tokio::test]
async fn test_payment_status_from_str() {
    assert_eq!(
        PaymentStatus::from_str("pending").unwrap(),
        PaymentStatus::Pending
    );
    assert_eq!(
        PaymentStatus::from_str("active").unwrap(),
        PaymentStatus::Active
    );
    assert_eq!(
        PaymentStatus::from_str("cancelled").unwrap(),
        PaymentStatus::Cancelled
    );
    assert_eq!(
        PaymentStatus::from_str("expired").unwrap(),
        PaymentStatus::Expired
    );
    assert_eq!(
        PaymentStatus::from_str("failed").unwrap(),
        PaymentStatus::Failed
    );
    assert!(PaymentStatus::from_str("invalid").is_err());
}

#[tokio::test]
async fn test_payment_type_from_str() {
    assert_eq!(
        PaymentType::from_str("subscription").unwrap(),
        PaymentType::Subscription
    );
    assert_eq!(
        PaymentType::from_str("one_time").unwrap(),
        PaymentType::OneTime
    );
    assert!(PaymentType::from_str("invalid").is_err());
}
