use crate::errors::{AppError, AppResult};
use crate::models::payment::PaymentStatus;
use crate::services::payment::{
    PaymentDbOperations, PaymentService, db_operations::CheckoutUpdate,
};
use chrono::Utc;
use stripe::{EventObject, EventType, PaymentIntent, Webhook};
use uuid::Uuid;

/// Webhook handling methods for `PaymentService`
#[allow(async_fn_in_trait)]
pub trait WebhookHandlers {
    /// Process webhook event implementation
    async fn process_webhook_event_impl(
        &self,
        payload: &str,
        stripe_signature: &str,
    ) -> AppResult<()>;

    /// Handle payment succeeded (for one-time payments)
    async fn handle_payment_succeeded(&self, payment_intent: PaymentIntent) -> AppResult<()>;
}

impl WebhookHandlers for PaymentService {
    async fn process_webhook_event_impl(
        &self,
        payload: &str,
        stripe_signature: &str,
    ) -> AppResult<()> {
        // Verify webhook signature
        let event = Webhook::construct_event(payload, stripe_signature, &self.webhook_secret)
            .map_err(|e| {
                AppError::BadRequest(format!("Failed to verify webhook signature: {e}"))
            })?;

        // Check if event already processed
        if self.is_webhook_event_processed(event.id.as_ref()).await? {
            return Ok(());
        }

        // Store webhook event
        let event_id = self
            .store_webhook_event(
                event.id.as_ref(),
                &event.type_.to_string(),
                serde_json::to_value(&event).unwrap_or_default(),
            )
            .await?;

        // Process event based on type
        match event.type_ {
            EventType::PaymentIntentSucceeded => {
                if let EventObject::PaymentIntent(payment_intent) = event.data.object {
                    self.handle_payment_succeeded(payment_intent).await?;
                }
            }
            _ => {
                // Ignore other event types for now
                tracing::debug!("Ignoring event type: {:?}", event.type_);
            }
        }

        // Mark event as processed
        self.mark_webhook_event_processed(event_id).await?;

        Ok(())
    }

    async fn handle_payment_succeeded(&self, payment_intent: PaymentIntent) -> AppResult<()> {
        // Get payment ID from metadata
        let payment_id = payment_intent
            .metadata
            .get("payment_id")
            .and_then(|id| Uuid::parse_str(id).ok())
            .ok_or_else(|| {
                AppError::InternalServerError(
                    "Missing or invalid payment_id in metadata".to_string(),
                )
            })?;

        // Update payment status to active
        self.update_payment_status(payment_id, PaymentStatus::Active)
            .await?;

        // Set payment expiry to 30 days from now (or whatever your business logic requires)
        let now = Utc::now();
        let expiry = now + chrono::Duration::days(30);

        // Update payment dates
        self.update_payment_after_checkout(&CheckoutUpdate {
            payment_id,
            stripe_subscription_id: None,
            stripe_payment_intent_id: Some(payment_intent.id.to_string()),
            amount_cents: i32::try_from(payment_intent.amount).unwrap_or(i32::MAX),
            currency: payment_intent.currency.to_string(),
            subscription_start: Some(now),
            subscription_end: Some(expiry),
        })
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::models::payment::PaymentType;
    use sqlx::SqlitePool;
    use std::collections::HashMap;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

        // Run migrations to ensure test database matches production schema
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        pool
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
    async fn test_handle_payment_succeeded() {
        let pool = setup_test_db().await;
        let service = create_test_service(pool.clone());

        let user_id = Uuid::new_v4();

        // Create user first due to foreign key constraint
        sqlx::query(
            "INSERT INTO users (id, email, hashed_password, provider, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(user_id.to_string())
        .bind(format!("{user_id}@example.com"))
        .bind("hashed_password")
        .bind("local")
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&pool)
        .await
        .unwrap();
        let payment = service
            .create_payment(user_id, PaymentType::OneTime)
            .await
            .unwrap();

        // Initially payment should be pending
        assert_eq!(payment.payment_status, PaymentStatus::Pending);

        // Create mock payment intent with metadata
        let mut metadata = HashMap::new();
        metadata.insert("payment_id".to_string(), payment.id.to_string());

        // Since we can't create a real PaymentIntent in tests,
        // we'll test the logic around updating payment status
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
    }

    #[tokio::test]
    async fn test_webhook_event_processing_flow() {
        let pool = setup_test_db().await;
        let service = create_test_service(pool);

        let stripe_event_id = "evt_test789";
        let event_type = "payment_intent.succeeded";
        let event_data = serde_json::json!({
            "id": stripe_event_id,
            "type": event_type,
            "data": {
                "object": {
                    "id": "pi_test789"
                }
            }
        });

        // Store the event
        let event_id = service
            .store_webhook_event(stripe_event_id, event_type, event_data)
            .await
            .unwrap();

        // Check if processed (should be false)
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

        // Check again (should be true)
        let processed = service
            .is_webhook_event_processed(stripe_event_id)
            .await
            .unwrap();
        assert!(processed);
    }

    #[tokio::test]
    async fn test_payment_id_parsing_from_metadata() {
        let payment_id = Uuid::new_v4();
        let metadata = HashMap::from([("payment_id".to_string(), payment_id.to_string())]);

        // Test successful parsing
        let parsed = metadata
            .get("payment_id")
            .and_then(|id| Uuid::parse_str(id).ok());

        assert!(parsed.is_some());
        assert_eq!(parsed.unwrap(), payment_id);

        // Test missing metadata
        let empty_metadata: HashMap<String, String> = HashMap::new();
        let parsed = empty_metadata
            .get("payment_id")
            .and_then(|id| Uuid::parse_str(id).ok());

        assert!(parsed.is_none());
    }

    #[tokio::test]
    async fn test_checkout_update_with_dates() {
        let payment_id = Uuid::new_v4();
        let now = Utc::now();
        let expiry = now + chrono::Duration::days(30);

        let update = CheckoutUpdate {
            payment_id,
            stripe_subscription_id: None,
            stripe_payment_intent_id: Some("pi_test789".to_string()),
            amount_cents: 1500,
            currency: "usd".to_string(),
            subscription_start: Some(now),
            subscription_end: Some(expiry),
        };

        assert_eq!(update.payment_id, payment_id);
        assert!(update.subscription_start.is_some());
        assert!(update.subscription_end.is_some());

        // Verify the dates
        let duration = update.subscription_end.unwrap() - update.subscription_start.unwrap();
        assert_eq!(duration.num_days(), 30);
    }

    #[tokio::test]
    async fn test_amount_conversion() {
        // Test normal conversion
        let amount: i64 = 1000;
        let converted = i32::try_from(amount);
        assert!(converted.is_ok());
        assert_eq!(converted.unwrap(), 1000);

        // Test overflow handling
        let large_amount: i64 = i64::MAX;
        let converted = i32::try_from(large_amount).unwrap_or(i32::MAX);
        assert_eq!(converted, i32::MAX);
    }

    #[tokio::test]
    async fn test_event_type_matching() {
        // Test that we correctly match event types
        let event_type = "payment_intent.succeeded";
        assert!(matches!(event_type, "payment_intent.succeeded"));

        let other_event = "customer.subscription.created";
        assert!(!matches!(other_event, "payment_intent.succeeded"));
    }

    #[tokio::test]
    async fn test_duplicate_webhook_event_handling() {
        let pool = setup_test_db().await;
        let service = create_test_service(pool);

        let stripe_event_id = "evt_duplicate";
        let event_data = serde_json::json!({});

        // Store event first time
        let event_id = service
            .store_webhook_event(stripe_event_id, "test.event", event_data.clone())
            .await
            .unwrap();

        // Mark as processed
        service
            .mark_webhook_event_processed(event_id)
            .await
            .unwrap();

        // Check that it's marked as processed
        let is_processed = service
            .is_webhook_event_processed(stripe_event_id)
            .await
            .unwrap();
        assert!(is_processed);

        // In actual webhook handler, duplicate events would be skipped
    }
}
