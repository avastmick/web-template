use crate::errors::{AppError, AppResult};
use crate::models::payment::{
    CreatePaymentIntentRequest, CreatePaymentIntentResponse, PaymentType,
};
use crate::services::payment::{PaymentDbOperations, PaymentService};
use std::collections::HashMap;
use stripe::{CreateCustomer, CreatePaymentIntent, Currency, Customer, PaymentIntent};
use uuid::Uuid;

/// Stripe integration methods for `PaymentService`
#[allow(async_fn_in_trait)]
pub trait StripeIntegration {
    /// Create a Stripe payment intent implementation
    async fn create_payment_intent_impl(
        &self,
        user_id: Uuid,
        user_email: &str,
        request: CreatePaymentIntentRequest,
    ) -> AppResult<CreatePaymentIntentResponse>;
}

impl StripeIntegration for PaymentService {
    async fn create_payment_intent_impl(
        &self,
        user_id: Uuid,
        user_email: &str,
        request: CreatePaymentIntentRequest,
    ) -> AppResult<CreatePaymentIntentResponse> {
        // Check if user already has a payment record
        let existing_payment = self.get_payment_by_user_id(user_id).await?;

        let payment = match existing_payment {
            Some(p) => p,
            None => {
                // Create new payment record
                self.create_payment(user_id, PaymentType::OneTime).await?
            }
        };

        // Create or retrieve Stripe customer
        let customer_id = if let Some(id) = &payment.stripe_customer_id {
            id.clone()
        } else {
            // Create new Stripe customer
            let customer = Customer::create(
                &self.stripe,
                CreateCustomer {
                    email: Some(user_email),
                    metadata: Some(HashMap::from([(
                        "user_id".to_string(),
                        user_id.to_string(),
                    )])),
                    ..Default::default()
                },
            )
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("Failed to create Stripe customer: {e}"))
            })?;

            // Update payment record with customer ID
            self.update_stripe_customer_id(payment.id, customer.id.as_str())
                .await?;

            customer.id.as_str().to_string()
        };

        // Create payment intent
        let currency = match request.currency.to_lowercase().as_str() {
            "usd" => Currency::USD,
            "eur" => Currency::EUR,
            "gbp" => Currency::GBP,
            _ => return Err(AppError::BadRequest("Unsupported currency".to_string())),
        };

        let mut params = CreatePaymentIntent::new(request.amount_cents, currency);

        params.customer = Some(customer_id.parse().map_err(|_| {
            AppError::InternalServerError("Invalid customer ID format".to_string())
        })?);

        params.metadata = Some(HashMap::from([
            ("payment_id".to_string(), payment.id.to_string()),
            ("user_id".to_string(), user_id.to_string()),
        ]));

        let payment_intent = PaymentIntent::create(&self.stripe, params)
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("Failed to create payment intent: {e}"))
            })?;

        // Update payment record with payment intent ID
        self.update_payment_after_checkout(
            payment.id,
            None,
            Some(payment_intent.id.as_str()),
            i32::try_from(request.amount_cents)
                .map_err(|_| AppError::BadRequest("Amount too large to process".to_string()))?,
            &request.currency,
            None,
            None,
        )
        .await?;

        Ok(CreatePaymentIntentResponse {
            client_secret: payment_intent.client_secret.ok_or_else(|| {
                AppError::InternalServerError("No client secret returned".to_string())
            })?,
            payment_intent_id: payment_intent.id.as_str().to_string(),
        })
    }
}
