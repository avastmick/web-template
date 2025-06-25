use crate::errors::{AppError, AppResult};
use crate::models::payment::{PaymentStatus, PaymentType, UserPayment, UserPaymentFromDb};
use crate::services::payment::PaymentService;
use chrono::Utc;
use std::convert::TryFrom;
use uuid::Uuid;

/// Database operations for `PaymentService`
#[allow(async_fn_in_trait)]
pub trait PaymentDbOperations {
    /// Create a new payment record
    async fn create_payment(
        &self,
        user_id: Uuid,
        payment_type: PaymentType,
    ) -> AppResult<UserPayment>;

    /// Get payment by ID
    async fn get_payment_by_id(&self, id: Uuid) -> AppResult<Option<UserPayment>>;

    /// Get payment by user ID
    async fn get_payment_by_user_id(&self, user_id: Uuid) -> AppResult<Option<UserPayment>>;

    /// Get active payment for user
    async fn get_active_payment_for_user(&self, user_id: Uuid) -> AppResult<Option<UserPayment>>;

    /// Update Stripe customer ID
    async fn update_stripe_customer_id(
        &self,
        payment_id: Uuid,
        stripe_customer_id: &str,
    ) -> AppResult<()>;

    /// Update payment after successful checkout
    #[allow(clippy::too_many_arguments)]
    async fn update_payment_after_checkout(
        &self,
        payment_id: Uuid,
        stripe_subscription_id: Option<&str>,
        stripe_payment_intent_id: Option<&str>,
        amount_cents: i32,
        currency: &str,
        subscription_start: Option<chrono::DateTime<Utc>>,
        subscription_end: Option<chrono::DateTime<Utc>>,
    ) -> AppResult<()>;

    /// Update payment status
    async fn update_payment_status(&self, payment_id: Uuid, status: PaymentStatus)
    -> AppResult<()>;

    /// Store webhook event
    async fn store_webhook_event(
        &self,
        stripe_event_id: &str,
        event_type: &str,
        event_data: serde_json::Value,
    ) -> AppResult<Uuid>;

    /// Check if webhook event already processed
    async fn is_webhook_event_processed(&self, stripe_event_id: &str) -> AppResult<bool>;

    /// Mark webhook event as processed
    async fn mark_webhook_event_processed(&self, event_id: Uuid) -> AppResult<()>;

    /// Get payment by Stripe customer ID
    #[allow(dead_code)]
    async fn get_payment_by_stripe_customer_id(
        &self,
        stripe_customer_id: &str,
    ) -> AppResult<Option<UserPayment>>;
}

impl PaymentDbOperations for PaymentService {
    async fn create_payment(
        &self,
        user_id: Uuid,
        payment_type: PaymentType,
    ) -> AppResult<UserPayment> {
        let id = Uuid::new_v4();
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r"
			INSERT INTO user_payments (id, user_id, payment_type, payment_status, created_at, updated_at)
			VALUES (?, ?, ?, ?, ?, ?)
			",
        )
        .bind(id.to_string())
        .bind(user_id.to_string())
        .bind(payment_type.as_str())
        .bind(PaymentStatus::Pending.as_str())
        .bind(&now)
        .bind(&now)
        .execute(&self.db_pool)
        .await?;

        self.get_payment_by_id(id).await?.ok_or_else(|| {
            AppError::InternalServerError("Failed to retrieve created payment".to_string())
        })
    }

    async fn get_payment_by_id(&self, id: Uuid) -> AppResult<Option<UserPayment>> {
        let payment_row =
            sqlx::query_as::<_, UserPaymentFromDb>("SELECT * FROM user_payments WHERE id = ?")
                .bind(id.to_string())
                .fetch_optional(&self.db_pool)
                .await?;

        match payment_row {
            Some(row) => Ok(Some(UserPayment::try_from(row).map_err(|e| {
                AppError::InternalServerError(format!("Failed to convert payment: {e}"))
            })?)),
            None => Ok(None),
        }
    }

    async fn get_payment_by_user_id(&self, user_id: Uuid) -> AppResult<Option<UserPayment>> {
        let payment_row = sqlx::query_as::<_, UserPaymentFromDb>(
            "SELECT * FROM user_payments WHERE user_id = ? ORDER BY created_at DESC LIMIT 1",
        )
        .bind(user_id.to_string())
        .fetch_optional(&self.db_pool)
        .await?;

        match payment_row {
            Some(row) => Ok(Some(UserPayment::try_from(row).map_err(|e| {
                AppError::InternalServerError(format!("Failed to convert payment: {e}"))
            })?)),
            None => Ok(None),
        }
    }

    async fn get_active_payment_for_user(&self, user_id: Uuid) -> AppResult<Option<UserPayment>> {
        let payment_row = sqlx::query_as::<_, UserPaymentFromDb>(
            r"
			SELECT * FROM user_payments
			WHERE user_id = ? AND payment_status = 'active'
			ORDER BY created_at DESC LIMIT 1
			",
        )
        .bind(user_id.to_string())
        .fetch_optional(&self.db_pool)
        .await?;

        match payment_row {
            Some(row) => Ok(Some(UserPayment::try_from(row).map_err(|e| {
                AppError::InternalServerError(format!("Failed to convert payment: {e}"))
            })?)),
            None => Ok(None),
        }
    }

    async fn update_stripe_customer_id(
        &self,
        payment_id: Uuid,
        stripe_customer_id: &str,
    ) -> AppResult<()> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r"
			UPDATE user_payments
			SET stripe_customer_id = ?, updated_at = ?
			WHERE id = ?
			",
        )
        .bind(stripe_customer_id)
        .bind(now)
        .bind(payment_id.to_string())
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn update_payment_after_checkout(
        &self,
        payment_id: Uuid,
        stripe_subscription_id: Option<&str>,
        stripe_payment_intent_id: Option<&str>,
        amount_cents: i32,
        currency: &str,
        subscription_start: Option<chrono::DateTime<Utc>>,
        subscription_end: Option<chrono::DateTime<Utc>>,
    ) -> AppResult<()> {
        let now = Utc::now().to_rfc3339();
        let last_payment_date = Utc::now().to_rfc3339();

        let subscription_start_str = subscription_start.map(|dt| dt.to_rfc3339());
        let subscription_end_str = subscription_end.map(|dt| dt.to_rfc3339());

        sqlx::query(
            r"
			UPDATE user_payments
			SET payment_status = ?,
				stripe_subscription_id = ?,
				stripe_payment_intent_id = ?,
				amount_cents = ?,
				currency = ?,
				subscription_start_date = ?,
				subscription_end_date = ?,
				last_payment_date = ?,
				updated_at = ?
			WHERE id = ?
			",
        )
        .bind(PaymentStatus::Active.as_str())
        .bind(stripe_subscription_id)
        .bind(stripe_payment_intent_id)
        .bind(amount_cents)
        .bind(currency)
        .bind(subscription_start_str)
        .bind(subscription_end_str)
        .bind(last_payment_date)
        .bind(now)
        .bind(payment_id.to_string())
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn update_payment_status(
        &self,
        payment_id: Uuid,
        status: PaymentStatus,
    ) -> AppResult<()> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r"
			UPDATE user_payments
			SET payment_status = ?, updated_at = ?
			WHERE id = ?
			",
        )
        .bind(status.as_str())
        .bind(now)
        .bind(payment_id.to_string())
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn store_webhook_event(
        &self,
        stripe_event_id: &str,
        event_type: &str,
        event_data: serde_json::Value,
    ) -> AppResult<Uuid> {
        let id = Uuid::new_v4();
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r"
			INSERT INTO stripe_webhook_events (id, stripe_event_id, event_type, event_data, created_at)
			VALUES (?, ?, ?, ?, ?)
			",
        )
        .bind(id.to_string())
        .bind(stripe_event_id)
        .bind(event_type)
        .bind(event_data.to_string())
        .bind(now)
        .execute(&self.db_pool)
        .await?;

        Ok(id)
    }

    async fn is_webhook_event_processed(&self, stripe_event_id: &str) -> AppResult<bool> {
        let result = sqlx::query_scalar::<_, i32>(
            r"
			SELECT COUNT(*) FROM stripe_webhook_events
			WHERE stripe_event_id = ? AND processed = TRUE
			",
        )
        .bind(stripe_event_id)
        .fetch_one(&self.db_pool)
        .await?;

        Ok(result > 0)
    }

    async fn mark_webhook_event_processed(&self, event_id: Uuid) -> AppResult<()> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r"
			UPDATE stripe_webhook_events
			SET processed = TRUE, processed_at = ?
			WHERE id = ?
			",
        )
        .bind(now)
        .bind(event_id.to_string())
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    async fn get_payment_by_stripe_customer_id(
        &self,
        stripe_customer_id: &str,
    ) -> AppResult<Option<UserPayment>> {
        let payment_row = sqlx::query_as::<_, UserPaymentFromDb>(
            "SELECT * FROM user_payments WHERE stripe_customer_id = ?",
        )
        .bind(stripe_customer_id)
        .fetch_optional(&self.db_pool)
        .await?;

        match payment_row {
            Some(row) => Ok(Some(UserPayment::try_from(row).map_err(|e| {
                AppError::InternalServerError(format!("Failed to convert payment: {e}"))
            })?)),
            None => Ok(None),
        }
    }
}
