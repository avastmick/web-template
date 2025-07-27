mod db_operations;
mod stripe_integration;
mod webhook_handlers;

use crate::errors::{AppError, AppResult};
use crate::models::payment::{
    CreatePaymentIntentRequest, CreatePaymentIntentResponse, UserPaymentStatusResponse,
};
use sqlx::SqlitePool;
use std::env;
use stripe::Client as StripeClient;
use tracing::instrument;
use uuid::Uuid;

pub use db_operations::PaymentDbOperations;
pub use stripe_integration::StripeIntegration;
pub use webhook_handlers::WebhookHandlers;

#[derive(Clone)]
pub struct PaymentService {
    db_pool: SqlitePool,
    stripe: StripeClient,
    webhook_secret: String,
}

impl PaymentService {
    /// Create a new payment service instance
    ///
    /// # Errors
    ///
    /// Returns an error if required environment variables are not set
    pub fn new(db_pool: SqlitePool) -> AppResult<Self> {
        let stripe_secret_key = env::var("STRIPE_SECRET_KEY").map_err(|_| {
            AppError::ConfigError("STRIPE_SECRET_KEY environment variable not set".to_string())
        })?;
        let webhook_secret = env::var("STRIPE_WEBHOOK_ENDPOINT_SECRET").map_err(|_| {
            AppError::ConfigError(
                "STRIPE_WEBHOOK_ENDPOINT_SECRET environment variable not set".to_string(),
            )
        })?;

        let stripe = StripeClient::new(stripe_secret_key);

        Ok(Self {
            db_pool,
            stripe,
            webhook_secret,
        })
    }

    /// Get user payment status
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails
    #[instrument(skip(self), err(Debug))]
    pub async fn get_user_payment_status(
        &self,
        user_id: Uuid,
    ) -> AppResult<UserPaymentStatusResponse> {
        let payment = self.get_active_payment_for_user(user_id).await?;

        match payment {
            Some(p) => Ok(UserPaymentStatusResponse {
                has_active_payment: p.payment_status
                    == crate::models::payment::PaymentStatus::Active,
                payment_status: Some(p.payment_status),
                payment_type: Some(p.payment_type),
                subscription_end_date: p.subscription_end_date,
            }),
            None => Ok(UserPaymentStatusResponse {
                has_active_payment: false,
                payment_status: None,
                payment_type: None,
                subscription_end_date: None,
            }),
        }
    }

    /// Create a Stripe payment intent
    ///
    /// # Errors
    ///
    /// Returns an error if the Stripe API call or database operation fails
    #[instrument(skip(self), fields(user_email = %user_email), err(Debug))]
    pub async fn create_payment_intent(
        &self,
        user_id: Uuid,
        user_email: &str,
        request: CreatePaymentIntentRequest,
    ) -> AppResult<CreatePaymentIntentResponse> {
        self.create_payment_intent_impl(user_id, user_email, request)
            .await
    }

    /// Process Stripe webhook event
    ///
    /// # Errors
    ///
    /// Returns an error if webhook signature verification fails or event processing fails
    #[instrument(skip(self, payload), err(Debug))]
    pub async fn process_webhook_event(
        &self,
        payload: &str,
        stripe_signature: &str,
    ) -> AppResult<()> {
        Box::pin(self.process_webhook_event_impl(payload, stripe_signature)).await
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::models::payment::{PaymentStatus, PaymentType};
    use sqlx::SqlitePool;

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
    async fn test_payment_service_creation() {
        // Save current env vars
        let saved_key = std::env::var("STRIPE_SECRET_KEY").ok();
        let saved_secret = std::env::var("STRIPE_WEBHOOK_ENDPOINT_SECRET").ok();

        // Test that service creation fails without env vars
        // SAFETY: Tests are single-threaded and we're cleaning up test values
        #[allow(unsafe_code)]
        unsafe {
            std::env::remove_var("STRIPE_SECRET_KEY");
            std::env::remove_var("STRIPE_WEBHOOK_ENDPOINT_SECRET");
        }

        let pool = setup_test_db().await;
        let result = PaymentService::new(pool);
        assert!(result.is_err());

        // Set one var but not the other
        // SAFETY: Tests are single-threaded and we're setting test values
        #[allow(unsafe_code)]
        unsafe {
            std::env::set_var("STRIPE_SECRET_KEY", "test_key");
        }
        let pool = setup_test_db().await;
        let result = PaymentService::new(pool);
        assert!(result.is_err());

        // Set both vars
        #[allow(unsafe_code)]
        unsafe {
            std::env::set_var("STRIPE_WEBHOOK_ENDPOINT_SECRET", "test_secret");
        }
        let pool = setup_test_db().await;
        let result = PaymentService::new(pool);
        assert!(result.is_ok());

        // Restore original env vars
        #[allow(unsafe_code)]
        unsafe {
            if let Some(key) = saved_key {
                std::env::set_var("STRIPE_SECRET_KEY", key);
            }
            if let Some(secret) = saved_secret {
                std::env::set_var("STRIPE_WEBHOOK_ENDPOINT_SECRET", secret);
            }
        }
    }

    #[tokio::test]
    async fn test_get_user_payment_status_no_payment() {
        let pool = setup_test_db().await;
        let service = create_test_service(pool);

        let user_id = Uuid::new_v4();
        let status = service.get_user_payment_status(user_id).await.unwrap();

        assert!(!status.has_active_payment);
        assert!(status.payment_status.is_none());
        assert!(status.payment_type.is_none());
        assert!(status.subscription_end_date.is_none());
    }

    #[tokio::test]
    async fn test_get_user_payment_status_with_active_payment() {
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

        // Create a payment
        let payment = service
            .create_payment(user_id, PaymentType::Subscription)
            .await
            .unwrap();

        // Update to active
        service
            .update_payment_status(payment.id, PaymentStatus::Active)
            .await
            .unwrap();

        let status = service.get_user_payment_status(user_id).await.unwrap();

        assert!(status.has_active_payment);
        assert_eq!(status.payment_status, Some(PaymentStatus::Active));
        assert_eq!(status.payment_type, Some(PaymentType::Subscription));
    }

    #[tokio::test]
    async fn test_get_user_payment_status_with_expired_payment() {
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

        // Create an expired payment
        let payment = service
            .create_payment(user_id, PaymentType::OneTime)
            .await
            .unwrap();

        service
            .update_payment_status(payment.id, PaymentStatus::Expired)
            .await
            .unwrap();

        // Note: get_user_payment_status only returns active payments
        // So for expired payment, it returns None
        let status = service.get_user_payment_status(user_id).await.unwrap();

        assert!(!status.has_active_payment);
        assert!(status.payment_status.is_none());
        assert!(status.payment_type.is_none());

        // Verify the payment exists and is expired using get_payment_by_user_id
        let payment_record = service
            .get_payment_by_user_id(user_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(payment_record.payment_status, PaymentStatus::Expired);
        assert_eq!(payment_record.payment_type, PaymentType::OneTime);
    }

    #[tokio::test]
    async fn test_create_payment_intent_request() {
        let request = CreatePaymentIntentRequest {
            amount_cents: 2500,
            currency: "eur".to_string(),
        };

        assert_eq!(request.amount_cents, 2500);
        assert_eq!(request.currency, "eur");
    }

    #[tokio::test]
    async fn test_user_payment_status_response() {
        let response = UserPaymentStatusResponse {
            has_active_payment: true,
            payment_status: Some(PaymentStatus::Active),
            payment_type: Some(PaymentType::Subscription),
            subscription_end_date: Some(chrono::Utc::now() + chrono::Duration::days(30)),
        };

        assert!(response.has_active_payment);
        assert_eq!(response.payment_status, Some(PaymentStatus::Active));
        assert_eq!(response.payment_type, Some(PaymentType::Subscription));
        assert!(response.subscription_end_date.is_some());
    }

    #[tokio::test]
    async fn test_payment_service_clone() {
        let pool = setup_test_db().await;
        // SAFETY: Tests are single-threaded and we're setting test values
        #[allow(unsafe_code)]
        unsafe {
            std::env::set_var("STRIPE_SECRET_KEY", "test_key");
            std::env::set_var("STRIPE_WEBHOOK_ENDPOINT_SECRET", "test_secret");
        }
        let service = PaymentService::new(pool).unwrap();

        // Test that service can be cloned
        let _cloned = service.clone();
    }
}
