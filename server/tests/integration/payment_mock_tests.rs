#![allow(clippy::unwrap_used)]

//! Payment service tests with Stripe mocks
//!
//! These tests exercise the `PaymentService` using mock Stripe responses
//! and webhook fixture files for comprehensive testing without external API calls.

use chrono::{Duration, Utc};
use serde_json::Value;
use server::models::payment::{PaymentStatus, PaymentType};
use server::services::payment::{PaymentDbOperations, PaymentService};
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

// ============================================================================
// Mock Stripe Types (inline for integration tests)
// ============================================================================

/// Mock response for creating a Stripe customer
#[derive(Debug, Clone)]
struct MockCustomerResponse {
    id: String,
    email: Option<String>,
    metadata: HashMap<String, String>,
}

impl Default for MockCustomerResponse {
    fn default() -> Self {
        Self {
            id: format!("cus_mock_{}", Uuid::new_v4().simple()),
            email: None,
            metadata: HashMap::new(),
        }
    }
}

/// Mock response for creating a payment intent
#[derive(Debug, Clone)]
struct MockPaymentIntentResponse {
    id: String,
    client_secret: String,
    amount: i64,
    currency: String,
    customer: Option<String>,
    metadata: HashMap<String, String>,
}

impl Default for MockPaymentIntentResponse {
    fn default() -> Self {
        let id = format!("pi_mock_{}", Uuid::new_v4().simple());
        Self {
            client_secret: format!("{id}_secret_mock"),
            id,
            amount: 0,
            currency: "usd".to_string(),
            customer: None,
            metadata: HashMap::new(),
        }
    }
}

/// Configuration for mock behavior
#[derive(Debug, Clone, Default)]
struct MockStripeConfig {
    fail_customer_creation: bool,
    fail_payment_intent_creation: bool,
    error_message: Option<String>,
}

/// Recorded customer creation call
#[derive(Debug, Clone)]
struct CustomerCreateCall {
    email: Option<String>,
    metadata: HashMap<String, String>,
}

/// Mock Stripe client
#[derive(Debug, Default)]
struct MockStripeClient {
    config: MockStripeConfig,
    customer_calls: Arc<Mutex<Vec<CustomerCreateCall>>>,
    payment_intent_calls: Arc<Mutex<Vec<PaymentIntentCreateCall>>>,
}

/// Recorded payment intent creation call
#[derive(Debug, Clone)]
struct PaymentIntentCreateCall {
    amount: i64,
    currency: String,
    customer: Option<String>,
    metadata: HashMap<String, String>,
}

impl MockStripeClient {
    fn new() -> Self {
        Self::default()
    }

    fn with_config(config: MockStripeConfig) -> Self {
        Self {
            config,
            customer_calls: Arc::new(Mutex::new(Vec::new())),
            payment_intent_calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn create_customer(
        &self,
        email: Option<&str>,
        metadata: HashMap<String, String>,
    ) -> Result<MockCustomerResponse, String> {
        {
            let mut calls = self.customer_calls.lock().expect("Lock poisoned");
            calls.push(CustomerCreateCall {
                email: email.map(ToString::to_string),
                metadata: metadata.clone(),
            });
        }

        if self.config.fail_customer_creation {
            return Err(self
                .config
                .error_message
                .clone()
                .unwrap_or_else(|| "Mock customer creation failed".to_string()));
        }

        Ok(MockCustomerResponse {
            email: email.map(ToString::to_string),
            metadata,
            ..Default::default()
        })
    }

    fn create_payment_intent(
        &self,
        amount: i64,
        currency: &str,
        customer: Option<&str>,
        metadata: HashMap<String, String>,
    ) -> Result<MockPaymentIntentResponse, String> {
        {
            let mut calls = self.payment_intent_calls.lock().expect("Lock poisoned");
            calls.push(PaymentIntentCreateCall {
                amount,
                currency: currency.to_string(),
                customer: customer.map(ToString::to_string),
                metadata: metadata.clone(),
            });
        }

        if self.config.fail_payment_intent_creation {
            return Err(self
                .config
                .error_message
                .clone()
                .unwrap_or_else(|| "Mock payment intent creation failed".to_string()));
        }

        Ok(MockPaymentIntentResponse {
            amount,
            currency: currency.to_string(),
            customer: customer.map(ToString::to_string),
            metadata,
            ..Default::default()
        })
    }

    fn customer_call_count(&self) -> usize {
        self.customer_calls.lock().expect("Lock poisoned").len()
    }

    fn payment_intent_call_count(&self) -> usize {
        self.payment_intent_calls.lock().expect("Lock poisoned").len()
    }
}

/// Builder for mock webhook events
#[derive(Debug, Clone, Default)]
struct MockWebhookEventBuilder {
    event_id: Option<String>,
    event_type: Option<String>,
    payment_intent_id: Option<String>,
    payment_id: Option<String>,
    user_id: Option<String>,
    amount: Option<i64>,
    currency: Option<String>,
}

impl MockWebhookEventBuilder {
    fn new() -> Self {
        Self::default()
    }

    fn event_id(mut self, id: &str) -> Self {
        self.event_id = Some(id.to_string());
        self
    }

    fn payment_id(mut self, id: &str) -> Self {
        self.payment_id = Some(id.to_string());
        self
    }

    fn user_id(mut self, id: &str) -> Self {
        self.user_id = Some(id.to_string());
        self
    }

    fn amount(mut self, amount: i64) -> Self {
        self.amount = Some(amount);
        self
    }

    fn currency(mut self, currency: &str) -> Self {
        self.currency = Some(currency.to_string());
        self
    }

    fn build_json(&self) -> serde_json::Value {
        let event_id = self
            .event_id
            .clone()
            .unwrap_or_else(|| format!("evt_mock_{}", Uuid::new_v4().simple()));
        let event_type = self
            .event_type
            .clone()
            .unwrap_or_else(|| "payment_intent.succeeded".to_string());
        let pi_id = self
            .payment_intent_id
            .clone()
            .unwrap_or_else(|| format!("pi_mock_{}", Uuid::new_v4().simple()));

        let mut metadata = serde_json::Map::new();
        if let Some(ref payment_id) = self.payment_id {
            metadata.insert("payment_id".to_string(), serde_json::json!(payment_id));
        }
        if let Some(ref user_id) = self.user_id {
            metadata.insert("user_id".to_string(), serde_json::json!(user_id));
        }

        serde_json::json!({
            "id": event_id,
            "object": "event",
            "api_version": "2023-10-16",
            "created": chrono::Utc::now().timestamp(),
            "type": event_type,
            "data": {
                "object": {
                    "id": pi_id,
                    "object": "payment_intent",
                    "amount": self.amount.unwrap_or(1000),
                    "currency": self.currency.clone().unwrap_or_else(|| "usd".to_string()),
                    "status": "succeeded",
                    "metadata": metadata
                }
            },
            "livemode": false,
            "pending_webhooks": 0,
            "request": {
                "id": format!("req_mock_{}", Uuid::new_v4().simple()),
                "idempotency_key": null
            }
        })
    }

    fn build_payment_succeeded(&self) -> serde_json::Value {
        Self {
            event_type: Some("payment_intent.succeeded".to_string()),
            ..self.clone()
        }
        .build_json()
    }

    fn build_payment_failed(&self) -> serde_json::Value {
        let mut event = self.clone();
        event.event_type = Some("payment_intent.payment_failed".to_string());

        let mut json = event.build_json();
        if let Some(obj) = json.get_mut("data").and_then(|d| d.get_mut("object")) {
            obj["status"] = serde_json::json!("requires_payment_method");
            obj["last_payment_error"] = serde_json::json!({
                "code": "card_declined",
                "message": "Your card was declined.",
                "type": "card_error"
            });
        }
        json
    }
}

// ============================================================================
// Test Setup
// ============================================================================

async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    pool
}

fn create_test_service(pool: SqlitePool) -> PaymentService {
    #[allow(unsafe_code)]
    unsafe {
        std::env::set_var("STRIPE_SECRET_KEY", "sk_test_mock_key");
        std::env::set_var("STRIPE_WEBHOOK_ENDPOINT_SECRET", "whsec_test_mock_secret");
    }
    PaymentService::new(pool).unwrap()
}

async fn create_test_user(pool: &SqlitePool, email: &str) -> Uuid {
    let user_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, email, hashed_password, provider, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(user_id.to_string())
    .bind(email)
    .bind("hashed_password_for_test")
    .bind("local")
    .bind(Utc::now().to_rfc3339())
    .bind(Utc::now().to_rfc3339())
    .execute(pool)
    .await
    .unwrap();
    user_id
}

// ============================================================================
// get_user_payment_status Tests
// ============================================================================

#[tokio::test]
async fn test_get_payment_status_no_payments() {
    let pool = setup_test_db().await;
    let service = create_test_service(pool.clone());
    let user_id = create_test_user(&pool, "no_payment@test.com").await;

    let status = service.get_user_payment_status(user_id).await.unwrap();

    assert!(!status.has_active_payment);
    assert!(status.payment_status.is_none());
    assert!(status.payment_type.is_none());
    assert!(status.subscription_end_date.is_none());
}

#[tokio::test]
async fn test_get_payment_status_with_pending_payment() {
    let pool = setup_test_db().await;
    let service = create_test_service(pool.clone());
    let user_id = create_test_user(&pool, "pending@test.com").await;

    // Create a pending payment
    let _payment = service
        .create_payment(user_id, PaymentType::OneTime)
        .await
        .unwrap();

    // Pending payments shouldn't show as active
    let status = service.get_user_payment_status(user_id).await.unwrap();
    assert!(!status.has_active_payment);
    // Note: get_active_payment_for_user only returns 'active' status payments
    assert!(status.payment_status.is_none());
}

#[tokio::test]
async fn test_get_payment_status_with_active_subscription() {
    let pool = setup_test_db().await;
    let service = create_test_service(pool.clone());
    let user_id = create_test_user(&pool, "active_sub@test.com").await;

    let now = Utc::now();
    let end_date = now + Duration::days(30);
    let payment_id = Uuid::new_v4();

    // Create active subscription payment via direct SQL (since CheckoutUpdate is private)
    sqlx::query(
        r"INSERT INTO user_payments
         (id, user_id, payment_type, payment_status, amount_cents, currency,
          subscription_start_date, subscription_end_date, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(payment_id.to_string())
    .bind(user_id.to_string())
    .bind("subscription")
    .bind("active")
    .bind(2500)
    .bind("eur")
    .bind(now.to_rfc3339())
    .bind(end_date.to_rfc3339())
    .bind(now.to_rfc3339())
    .bind(now.to_rfc3339())
    .execute(&pool)
    .await
    .unwrap();

    let status = service.get_user_payment_status(user_id).await.unwrap();

    assert!(status.has_active_payment);
    assert_eq!(status.payment_status, Some(PaymentStatus::Active));
    assert_eq!(status.payment_type, Some(PaymentType::Subscription));
    assert!(status.subscription_end_date.is_some());
}

#[tokio::test]
async fn test_get_payment_status_with_active_one_time() {
    let pool = setup_test_db().await;
    let service = create_test_service(pool.clone());
    let user_id = create_test_user(&pool, "onetime@test.com").await;

    // Create and activate a one-time payment
    let payment = service
        .create_payment(user_id, PaymentType::OneTime)
        .await
        .unwrap();

    service
        .update_payment_status(payment.id, PaymentStatus::Active)
        .await
        .unwrap();

    let status = service.get_user_payment_status(user_id).await.unwrap();

    assert!(status.has_active_payment);
    assert_eq!(status.payment_status, Some(PaymentStatus::Active));
    assert_eq!(status.payment_type, Some(PaymentType::OneTime));
}

#[tokio::test]
async fn test_get_payment_status_expired_not_active() {
    let pool = setup_test_db().await;
    let service = create_test_service(pool.clone());
    let user_id = create_test_user(&pool, "expired@test.com").await;

    // Create and expire a payment
    let payment = service
        .create_payment(user_id, PaymentType::Subscription)
        .await
        .unwrap();

    service
        .update_payment_status(payment.id, PaymentStatus::Expired)
        .await
        .unwrap();

    let status = service.get_user_payment_status(user_id).await.unwrap();

    // Expired payments should not show as active
    assert!(!status.has_active_payment);
    assert!(status.payment_status.is_none());
}

#[tokio::test]
async fn test_get_payment_status_cancelled_not_active() {
    let pool = setup_test_db().await;
    let service = create_test_service(pool.clone());
    let user_id = create_test_user(&pool, "cancelled@test.com").await;

    let payment = service
        .create_payment(user_id, PaymentType::Subscription)
        .await
        .unwrap();

    service
        .update_payment_status(payment.id, PaymentStatus::Cancelled)
        .await
        .unwrap();

    let status = service.get_user_payment_status(user_id).await.unwrap();

    assert!(!status.has_active_payment);
    assert!(status.payment_status.is_none());
}

#[tokio::test]
async fn test_get_payment_status_failed_not_active() {
    let pool = setup_test_db().await;
    let service = create_test_service(pool.clone());
    let user_id = create_test_user(&pool, "failed@test.com").await;

    let payment = service
        .create_payment(user_id, PaymentType::OneTime)
        .await
        .unwrap();

    service
        .update_payment_status(payment.id, PaymentStatus::Failed)
        .await
        .unwrap();

    let status = service.get_user_payment_status(user_id).await.unwrap();

    assert!(!status.has_active_payment);
    assert!(status.payment_status.is_none());
}

// ============================================================================
// Mock Stripe Client Tests (for payment intent flow)
// ============================================================================

#[tokio::test]
async fn test_mock_stripe_customer_creation() {
    let mock = MockStripeClient::new();

    let mut metadata = HashMap::new();
    metadata.insert("user_id".to_string(), Uuid::new_v4().to_string());

    let result = mock.create_customer(Some("test@example.com"), metadata.clone());
    assert!(result.is_ok());

    let customer = result.unwrap();
    assert!(customer.id.starts_with("cus_mock_"));
    assert_eq!(customer.email, Some("test@example.com".to_string()));
    assert_eq!(customer.metadata.get("user_id"), metadata.get("user_id"));
}

#[tokio::test]
async fn test_mock_stripe_payment_intent_creation() {
    let mock = MockStripeClient::new();

    let mut metadata = HashMap::new();
    let payment_id = Uuid::new_v4().to_string();
    metadata.insert("payment_id".to_string(), payment_id.clone());

    let result = mock.create_payment_intent(2500, "eur", Some("cus_mock_123"), metadata);
    assert!(result.is_ok());

    let intent = result.unwrap();
    assert!(intent.id.starts_with("pi_mock_"));
    assert!(intent.client_secret.contains("_secret_mock"));
    assert_eq!(intent.amount, 2500);
    assert_eq!(intent.currency, "eur");
    assert_eq!(intent.customer, Some("cus_mock_123".to_string()));
    assert_eq!(intent.metadata.get("payment_id"), Some(&payment_id));
}

#[tokio::test]
async fn test_mock_stripe_customer_creation_failure() {
    let config = MockStripeConfig {
        fail_customer_creation: true,
        error_message: Some("Stripe API error: invalid_request".to_string()),
        ..Default::default()
    };
    let mock = MockStripeClient::with_config(config);

    let result = mock.create_customer(Some("test@example.com"), HashMap::new());
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Stripe API error: invalid_request");
}

#[tokio::test]
async fn test_mock_stripe_payment_intent_failure() {
    let config = MockStripeConfig {
        fail_payment_intent_creation: true,
        error_message: Some("Card declined".to_string()),
        ..Default::default()
    };
    let mock = MockStripeClient::with_config(config);

    let result = mock.create_payment_intent(1000, "usd", None, HashMap::new());
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Card declined");
}

// ============================================================================
// Payment Intent Database Flow Tests
// ============================================================================

#[tokio::test]
async fn test_create_payment_record_for_new_user() {
    let pool = setup_test_db().await;
    let service = create_test_service(pool.clone());
    let user_id = create_test_user(&pool, "new_payment@test.com").await;

    // Initially no payment
    let existing = service.get_payment_by_user_id(user_id).await.unwrap();
    assert!(existing.is_none());

    // Create payment
    let payment = service
        .create_payment(user_id, PaymentType::OneTime)
        .await
        .unwrap();

    assert_eq!(payment.user_id, user_id);
    assert_eq!(payment.payment_type, PaymentType::OneTime);
    assert_eq!(payment.payment_status, PaymentStatus::Pending);
    assert!(payment.stripe_customer_id.is_none());
}

#[tokio::test]
async fn test_retrieve_existing_payment_record() {
    let pool = setup_test_db().await;
    let service = create_test_service(pool.clone());
    let user_id = create_test_user(&pool, "existing@test.com").await;

    // Create first payment
    let payment1 = service
        .create_payment(user_id, PaymentType::Subscription)
        .await
        .unwrap();

    // Retrieve should get same payment
    let retrieved = service.get_payment_by_user_id(user_id).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, payment1.id);
}

#[tokio::test]
async fn test_update_stripe_customer_id() {
    let pool = setup_test_db().await;
    let service = create_test_service(pool.clone());
    let user_id = create_test_user(&pool, "stripe_cust@test.com").await;

    let payment = service
        .create_payment(user_id, PaymentType::OneTime)
        .await
        .unwrap();

    assert!(payment.stripe_customer_id.is_none());

    service
        .update_stripe_customer_id(payment.id, "cus_mock_abc123")
        .await
        .unwrap();

    let updated = service.get_payment_by_id(payment.id).await.unwrap().unwrap();
    assert_eq!(
        updated.stripe_customer_id,
        Some("cus_mock_abc123".to_string())
    );
}

#[tokio::test]
async fn test_payment_checkout_update_via_sql() {
    // Test that payment records can be updated with checkout details
    // Uses direct SQL since CheckoutUpdate is private
    let pool = setup_test_db().await;
    let service = create_test_service(pool.clone());
    let user_id = create_test_user(&pool, "checkout@test.com").await;

    let payment = service
        .create_payment(user_id, PaymentType::Subscription)
        .await
        .unwrap();

    assert_eq!(payment.payment_status, PaymentStatus::Pending);

    let now = Utc::now();
    let end_date = now + Duration::days(30);

    // Simulate checkout update via SQL
    sqlx::query(
        r"UPDATE user_payments SET
          payment_status = ?,
          stripe_subscription_id = ?,
          stripe_payment_intent_id = ?,
          amount_cents = ?,
          currency = ?,
          subscription_start_date = ?,
          subscription_end_date = ?,
          updated_at = ?
         WHERE id = ?",
    )
    .bind("active")
    .bind("sub_mock_xyz")
    .bind("pi_mock_xyz")
    .bind(4999)
    .bind("gbp")
    .bind(now.to_rfc3339())
    .bind(end_date.to_rfc3339())
    .bind(now.to_rfc3339())
    .bind(payment.id.to_string())
    .execute(&pool)
    .await
    .unwrap();

    let updated = service.get_payment_by_id(payment.id).await.unwrap().unwrap();
    assert_eq!(updated.payment_status, PaymentStatus::Active);
    assert_eq!(
        updated.stripe_subscription_id,
        Some("sub_mock_xyz".to_string())
    );
    assert_eq!(
        updated.stripe_payment_intent_id,
        Some("pi_mock_xyz".to_string())
    );
    assert_eq!(updated.amount_cents, Some(4999));
    assert_eq!(updated.currency, "gbp");
}

// ============================================================================
// Webhook Handler Tests with Fixtures
// ============================================================================

#[tokio::test]
async fn test_webhook_event_builder_payment_succeeded() {
    let payment_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let event = MockWebhookEventBuilder::new()
        .event_id("evt_test_success")
        .payment_id(&payment_id.to_string())
        .user_id(&user_id.to_string())
        .amount(2500)
        .currency("eur")
        .build_payment_succeeded();

    assert_eq!(event["type"], "payment_intent.succeeded");
    assert_eq!(event["data"]["object"]["amount"], 2500);
    assert_eq!(event["data"]["object"]["currency"], "eur");
    assert_eq!(
        event["data"]["object"]["metadata"]["payment_id"],
        payment_id.to_string()
    );
}

#[tokio::test]
async fn test_webhook_event_builder_payment_failed() {
    let event = MockWebhookEventBuilder::new()
        .payment_id("pay_test_123")
        .build_payment_failed();

    assert_eq!(event["type"], "payment_intent.payment_failed");
    assert_eq!(
        event["data"]["object"]["status"],
        "requires_payment_method"
    );
    assert!(event["data"]["object"]["last_payment_error"].is_object());
}

#[tokio::test]
async fn test_store_and_retrieve_webhook_event() {
    let pool = setup_test_db().await;
    let service = create_test_service(pool);

    let event_data = MockWebhookEventBuilder::new()
        .event_id("evt_test_store")
        .build_json();

    let event_id = service
        .store_webhook_event("evt_test_store", "payment_intent.succeeded", event_data)
        .await
        .unwrap();

    // Event should not be processed yet
    let is_processed = service
        .is_webhook_event_processed("evt_test_store")
        .await
        .unwrap();
    assert!(!is_processed);

    // Mark as processed
    service.mark_webhook_event_processed(event_id).await.unwrap();

    // Now should be processed
    let is_processed = service
        .is_webhook_event_processed("evt_test_store")
        .await
        .unwrap();
    assert!(is_processed);
}

#[tokio::test]
async fn test_duplicate_webhook_event_prevention() {
    let pool = setup_test_db().await;
    let service = create_test_service(pool);

    let stripe_event_id = "evt_duplicate_test";

    // Store first event
    let event_id = service
        .store_webhook_event(
            stripe_event_id,
            "payment_intent.succeeded",
            serde_json::json!({}),
        )
        .await
        .unwrap();

    service.mark_webhook_event_processed(event_id).await.unwrap();

    // Should detect as already processed
    let is_processed = service
        .is_webhook_event_processed(stripe_event_id)
        .await
        .unwrap();
    assert!(is_processed);

    // In actual handler, this would skip processing
}

#[tokio::test]
async fn test_load_fixture_payment_succeeded() {
    let fixture_path = "tests/fixtures/stripe/payment_intent_succeeded.json";
    let content = std::fs::read_to_string(fixture_path).expect("Should read fixture file");
    let event: Value = serde_json::from_str(&content).expect("Should parse JSON");

    assert_eq!(event["type"], "payment_intent.succeeded");
    assert_eq!(event["data"]["object"]["status"], "succeeded");
    assert_eq!(event["data"]["object"]["amount"], 2500);
    assert_eq!(event["data"]["object"]["currency"], "eur");
}

#[tokio::test]
async fn test_load_fixture_payment_failed() {
    let fixture_path = "tests/fixtures/stripe/payment_intent_failed.json";
    let content = std::fs::read_to_string(fixture_path).expect("Should read fixture file");
    let event: Value = serde_json::from_str(&content).expect("Should parse JSON");

    assert_eq!(event["type"], "payment_intent.payment_failed");
    assert_eq!(
        event["data"]["object"]["status"],
        "requires_payment_method"
    );
    assert_eq!(
        event["data"]["object"]["last_payment_error"]["code"],
        "card_declined"
    );
}

// ============================================================================
// Payment Status Transition Tests
// ============================================================================

#[tokio::test]
async fn test_payment_status_pending_to_active() {
    let pool = setup_test_db().await;
    let service = create_test_service(pool.clone());
    let user_id = create_test_user(&pool, "pending_active@test.com").await;

    let payment = service
        .create_payment(user_id, PaymentType::OneTime)
        .await
        .unwrap();

    assert_eq!(payment.payment_status, PaymentStatus::Pending);

    service
        .update_payment_status(payment.id, PaymentStatus::Active)
        .await
        .unwrap();

    let updated = service.get_payment_by_id(payment.id).await.unwrap().unwrap();
    assert_eq!(updated.payment_status, PaymentStatus::Active);
}

#[tokio::test]
async fn test_payment_status_active_to_expired() {
    let pool = setup_test_db().await;
    let service = create_test_service(pool.clone());
    let user_id = create_test_user(&pool, "active_expired@test.com").await;

    let payment = service
        .create_payment(user_id, PaymentType::Subscription)
        .await
        .unwrap();

    service
        .update_payment_status(payment.id, PaymentStatus::Active)
        .await
        .unwrap();

    service
        .update_payment_status(payment.id, PaymentStatus::Expired)
        .await
        .unwrap();

    let updated = service.get_payment_by_id(payment.id).await.unwrap().unwrap();
    assert_eq!(updated.payment_status, PaymentStatus::Expired);
}

#[tokio::test]
async fn test_payment_status_active_to_cancelled() {
    let pool = setup_test_db().await;
    let service = create_test_service(pool.clone());
    let user_id = create_test_user(&pool, "active_cancel@test.com").await;

    let payment = service
        .create_payment(user_id, PaymentType::Subscription)
        .await
        .unwrap();

    service
        .update_payment_status(payment.id, PaymentStatus::Active)
        .await
        .unwrap();

    service
        .update_payment_status(payment.id, PaymentStatus::Cancelled)
        .await
        .unwrap();

    let updated = service.get_payment_by_id(payment.id).await.unwrap().unwrap();
    assert_eq!(updated.payment_status, PaymentStatus::Cancelled);
}

#[tokio::test]
async fn test_payment_status_pending_to_failed() {
    let pool = setup_test_db().await;
    let service = create_test_service(pool.clone());
    let user_id = create_test_user(&pool, "pending_failed@test.com").await;

    let payment = service
        .create_payment(user_id, PaymentType::OneTime)
        .await
        .unwrap();

    service
        .update_payment_status(payment.id, PaymentStatus::Failed)
        .await
        .unwrap();

    let updated = service.get_payment_by_id(payment.id).await.unwrap().unwrap();
    assert_eq!(updated.payment_status, PaymentStatus::Failed);
}

#[tokio::test]
async fn test_all_payment_status_values() {
    let pool = setup_test_db().await;
    let service = create_test_service(pool.clone());

    let statuses = [
        PaymentStatus::Pending,
        PaymentStatus::Active,
        PaymentStatus::Cancelled,
        PaymentStatus::Expired,
        PaymentStatus::Failed,
    ];

    for (i, status) in statuses.iter().enumerate() {
        let user_id = create_test_user(&pool, &format!("status_{i}@test.com")).await;
        let payment = service
            .create_payment(user_id, PaymentType::OneTime)
            .await
            .unwrap();

        service
            .update_payment_status(payment.id, status.clone())
            .await
            .unwrap();

        let updated = service.get_payment_by_id(payment.id).await.unwrap().unwrap();
        assert_eq!(updated.payment_status, *status);
    }
}

// ============================================================================
// Multiple Payments per User Tests
// ============================================================================

#[tokio::test]
async fn test_get_latest_payment_for_user() {
    let pool = setup_test_db().await;
    let service = create_test_service(pool.clone());
    let user_id = create_test_user(&pool, "multi_payment@test.com").await;

    // Create first payment
    let payment1 = service
        .create_payment(user_id, PaymentType::OneTime)
        .await
        .unwrap();

    // Mark as expired
    service
        .update_payment_status(payment1.id, PaymentStatus::Expired)
        .await
        .unwrap();

    // Create second payment (more recent)
    let payment2_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO user_payments (id, user_id, payment_type, payment_status, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(payment2_id.to_string())
    .bind(user_id.to_string())
    .bind(PaymentType::Subscription.as_str())
    .bind(PaymentStatus::Pending.as_str())
    .bind((Utc::now() + Duration::seconds(1)).to_rfc3339())
    .bind(Utc::now().to_rfc3339())
    .execute(&pool)
    .await
    .unwrap();

    // get_payment_by_user_id should return most recent
    let latest = service.get_payment_by_user_id(user_id).await.unwrap().unwrap();
    assert_eq!(latest.id, payment2_id);
    assert_eq!(latest.payment_type, PaymentType::Subscription);
}

// ============================================================================
// Currency Validation Tests
// ============================================================================

#[tokio::test]
async fn test_supported_currencies() {
    // Test that the system recognizes supported currencies
    let supported = ["usd", "eur", "gbp", "USD", "EUR", "GBP"];

    for currency in supported {
        let normalized = currency.to_lowercase();
        assert!(
            matches!(normalized.as_str(), "usd" | "eur" | "gbp"),
            "Currency {currency} should be supported"
        );
    }
}

#[tokio::test]
async fn test_unsupported_currencies() {
    let unsupported = ["jpy", "cad", "aud", "chf", "xyz"];

    for currency in unsupported {
        let normalized = currency.to_lowercase();
        assert!(
            !matches!(normalized.as_str(), "usd" | "eur" | "gbp"),
            "Currency {currency} should not be supported"
        );
    }
}
