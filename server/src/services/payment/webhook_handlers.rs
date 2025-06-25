use crate::errors::{AppError, AppResult};
use crate::models::payment::PaymentStatus;
use crate::services::payment::{PaymentDbOperations, PaymentService};
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
        self.update_payment_after_checkout(
            payment_id,
            None,
            Some(payment_intent.id.as_str()),
            i32::try_from(payment_intent.amount).unwrap_or(i32::MAX),
            &payment_intent.currency.to_string(),
            Some(now),
            Some(expiry),
        )
        .await?;

        Ok(())
    }
}
