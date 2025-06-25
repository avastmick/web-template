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
