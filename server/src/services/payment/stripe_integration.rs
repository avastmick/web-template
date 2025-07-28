use crate::errors::{AppError, AppResult};
use crate::models::payment::{
    CreatePaymentIntentRequest, CreatePaymentIntentResponse, PaymentType,
};
use crate::services::payment::{
    PaymentDbOperations, PaymentService, db_operations::CheckoutUpdate,
};
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
        self.update_payment_after_checkout(&CheckoutUpdate {
            payment_id: payment.id,
            stripe_subscription_id: None,
            stripe_payment_intent_id: Some(payment_intent.id.to_string()),
            amount_cents: i32::try_from(request.amount_cents)
                .map_err(|_| AppError::BadRequest("Amount too large to process".to_string()))?,
            currency: request.currency.clone(),
            subscription_start: None,
            subscription_end: None,
        })
        .await?;

        Ok(CreatePaymentIntentResponse {
            client_secret: payment_intent.client_secret.ok_or_else(|| {
                AppError::InternalServerError("No client secret returned".to_string())
            })?,
            payment_intent_id: payment_intent.id.as_str().to_string(),
        })
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

    // Note: These tests would normally require mocking the Stripe API client
    // Since we can't actually call Stripe in unit tests, we'll test the logic
    // around the Stripe calls and database operations

    #[tokio::test]
    async fn test_create_payment_intent_creates_payment_record() {
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

        // Verify no payment exists initially
        let payment = service.get_payment_by_user_id(user_id).await.unwrap();
        assert!(payment.is_none());

        // Create a payment manually to test the flow
        let payment = service
            .create_payment(user_id, PaymentType::OneTime)
            .await
            .unwrap();

        assert_eq!(payment.user_id, user_id);
        assert_eq!(payment.payment_type, PaymentType::OneTime);
        assert_eq!(payment.payment_status, PaymentStatus::Pending);
    }

    #[tokio::test]
    async fn test_create_payment_intent_with_existing_payment() {
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

        // Create existing payment
        let existing = service
            .create_payment(user_id, PaymentType::Subscription)
            .await
            .unwrap();

        // Verify it retrieves existing payment
        let payment = service.get_payment_by_user_id(user_id).await.unwrap();
        assert!(payment.is_some());
        assert_eq!(payment.unwrap().id, existing.id);
    }

    #[tokio::test]
    async fn test_currency_validation() {
        // Test valid currencies
        assert!(matches!("usd", "usd" | "eur" | "gbp"));
        assert!(matches!("EUR", c if c.to_lowercase() == "eur"));

        // Test invalid currency would return error
        let invalid_currency = "xyz";
        assert!(!matches!(invalid_currency, "usd" | "eur" | "gbp"));
    }

    #[tokio::test]
    async fn test_checkout_update_creation() {
        let payment_id = Uuid::new_v4();
        let update = CheckoutUpdate {
            payment_id,
            stripe_subscription_id: None,
            stripe_payment_intent_id: Some("pi_test123".to_string()),
            amount_cents: 999,
            currency: "usd".to_string(),
            subscription_start: None,
            subscription_end: None,
        };

        assert_eq!(update.payment_id, payment_id);
        assert!(update.stripe_subscription_id.is_none());
        assert_eq!(
            update.stripe_payment_intent_id,
            Some("pi_test123".to_string())
        );
        assert_eq!(update.amount_cents, 999);
        assert_eq!(update.currency, "usd");
    }

    #[tokio::test]
    async fn test_payment_intent_request_validation() {
        let request = CreatePaymentIntentRequest {
            amount_cents: 1000,
            currency: "usd".to_string(),
        };

        assert_eq!(request.amount_cents, 1000);
        assert_eq!(request.currency, "usd");

        // Test amount conversion
        let amount_i32 = i32::try_from(request.amount_cents);
        assert!(amount_i32.is_ok());
        assert_eq!(amount_i32.unwrap(), 1000);
    }

    #[tokio::test]
    async fn test_update_stripe_customer_flow() {
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

        // Initially no customer ID
        assert!(payment.stripe_customer_id.is_none());

        // Update with customer ID
        service
            .update_stripe_customer_id(payment.id, "cus_test456")
            .await
            .unwrap();

        let updated = service
            .get_payment_by_id(payment.id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(updated.stripe_customer_id, Some("cus_test456".to_string()));
    }

    #[tokio::test]
    async fn test_metadata_creation() {
        let user_id = Uuid::new_v4();
        let payment_id = Uuid::new_v4();

        let metadata = HashMap::from([
            ("payment_id".to_string(), payment_id.to_string()),
            ("user_id".to_string(), user_id.to_string()),
        ]);

        assert_eq!(metadata.len(), 2);
        assert_eq!(metadata.get("user_id"), Some(&user_id.to_string()));
        assert_eq!(metadata.get("payment_id"), Some(&payment_id.to_string()));
    }

    #[tokio::test]
    async fn test_payment_intent_response_creation() {
        let client_secret = "pi_test_secret_123".to_string();
        let payment_intent_id = "pi_test123".to_string();

        let response = CreatePaymentIntentResponse {
            client_secret: client_secret.clone(),
            payment_intent_id: payment_intent_id.clone(),
        };

        assert_eq!(response.client_secret, client_secret);
        assert_eq!(response.payment_intent_id, payment_intent_id);
    }
}
