//! Mock Stripe client for testing payment flows
//!
//! This module provides mock implementations of Stripe operations
//! for testing payment intent creation, webhook handling, and customer management.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Mock response for creating a Stripe customer
#[derive(Debug, Clone)]
pub struct MockCustomerResponse {
    pub id: String,
    pub email: Option<String>,
    pub metadata: HashMap<String, String>,
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
pub struct MockPaymentIntentResponse {
    pub id: String,
    pub client_secret: String,
    pub amount: i64,
    pub currency: String,
    pub customer: Option<String>,
    pub metadata: HashMap<String, String>,
    pub status: String,
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
            status: "requires_payment_method".to_string(),
        }
    }
}

/// Configuration for mock behavior
#[derive(Debug, Clone, Default)]
pub struct MockStripeConfig {
    /// Whether customer creation should fail
    pub fail_customer_creation: bool,
    /// Whether payment intent creation should fail
    pub fail_payment_intent_creation: bool,
    /// Custom error message for failures
    pub error_message: Option<String>,
    /// Preset customer response
    pub customer_response: Option<MockCustomerResponse>,
    /// Preset payment intent response
    pub payment_intent_response: Option<MockPaymentIntentResponse>,
}

/// Mock Stripe client that records calls and returns configurable responses
#[derive(Debug, Default)]
pub struct MockStripeClient {
    config: MockStripeConfig,
    /// Recorded customer creation calls
    pub customer_calls: Arc<Mutex<Vec<CustomerCreateCall>>>,
    /// Recorded payment intent creation calls
    pub payment_intent_calls: Arc<Mutex<Vec<PaymentIntentCreateCall>>>,
}

/// Recorded customer creation call
#[derive(Debug, Clone)]
pub struct CustomerCreateCall {
    pub email: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Recorded payment intent creation call
#[derive(Debug, Clone)]
pub struct PaymentIntentCreateCall {
    pub amount: i64,
    pub currency: String,
    pub customer: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl MockStripeClient {
    /// Create a new mock client with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a mock client with custom configuration
    #[must_use]
    pub fn with_config(config: MockStripeConfig) -> Self {
        Self {
            config,
            customer_calls: Arc::new(Mutex::new(Vec::new())),
            payment_intent_calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Simulate creating a Stripe customer
    ///
    /// # Errors
    ///
    /// Returns an error if configured to fail
    pub fn create_customer(
        &self,
        email: Option<&str>,
        metadata: HashMap<String, String>,
    ) -> Result<MockCustomerResponse, String> {
        // Record the call
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

        Ok(self.config.customer_response.clone().unwrap_or_else(|| {
            let mut response = MockCustomerResponse::default();
            response.email = email.map(ToString::to_string);
            response.metadata = metadata;
            response
        }))
    }

    /// Simulate creating a payment intent
    ///
    /// # Errors
    ///
    /// Returns an error if configured to fail
    pub fn create_payment_intent(
        &self,
        amount: i64,
        currency: &str,
        customer: Option<&str>,
        metadata: HashMap<String, String>,
    ) -> Result<MockPaymentIntentResponse, String> {
        // Record the call
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

        Ok(self
            .config
            .payment_intent_response
            .clone()
            .unwrap_or_else(|| {
                let mut response = MockPaymentIntentResponse::default();
                response.amount = amount;
                response.currency = currency.to_string();
                response.customer = customer.map(ToString::to_string);
                response.metadata = metadata;
                response
            }))
    }

    /// Get the number of customer creation calls
    #[must_use]
    pub fn customer_call_count(&self) -> usize {
        self.customer_calls.lock().expect("Lock poisoned").len()
    }

    /// Get the number of payment intent creation calls
    #[must_use]
    pub fn payment_intent_call_count(&self) -> usize {
        self.payment_intent_calls.lock().expect("Lock poisoned").len()
    }

    /// Get recorded customer calls
    #[must_use]
    pub fn get_customer_calls(&self) -> Vec<CustomerCreateCall> {
        self.customer_calls.lock().expect("Lock poisoned").clone()
    }

    /// Get recorded payment intent calls
    #[must_use]
    pub fn get_payment_intent_calls(&self) -> Vec<PaymentIntentCreateCall> {
        self.payment_intent_calls
            .lock()
            .expect("Lock poisoned")
            .clone()
    }
}

/// Builder for creating mock webhook events
#[derive(Debug, Default)]
pub struct MockWebhookEventBuilder {
    event_id: Option<String>,
    event_type: Option<String>,
    payment_intent_id: Option<String>,
    payment_id: Option<String>,
    user_id: Option<String>,
    amount: Option<i64>,
    currency: Option<String>,
}

impl MockWebhookEventBuilder {
    /// Create a new webhook event builder
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the event ID
    #[must_use]
    pub fn event_id(mut self, id: &str) -> Self {
        self.event_id = Some(id.to_string());
        self
    }

    /// Set the event type
    #[must_use]
    pub fn event_type(mut self, event_type: &str) -> Self {
        self.event_type = Some(event_type.to_string());
        self
    }

    /// Set the payment intent ID
    #[must_use]
    pub fn payment_intent_id(mut self, id: &str) -> Self {
        self.payment_intent_id = Some(id.to_string());
        self
    }

    /// Set the payment ID (from metadata)
    #[must_use]
    pub fn payment_id(mut self, id: &str) -> Self {
        self.payment_id = Some(id.to_string());
        self
    }

    /// Set the user ID (from metadata)
    #[must_use]
    pub fn user_id(mut self, id: &str) -> Self {
        self.user_id = Some(id.to_string());
        self
    }

    /// Set the amount
    #[must_use]
    pub fn amount(mut self, amount: i64) -> Self {
        self.amount = Some(amount);
        self
    }

    /// Set the currency
    #[must_use]
    pub fn currency(mut self, currency: &str) -> Self {
        self.currency = Some(currency.to_string());
        self
    }

    /// Build the mock event as JSON
    #[must_use]
    pub fn build_json(&self) -> serde_json::Value {
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

    /// Build a `payment_intent.succeeded` event
    #[must_use]
    pub fn build_payment_succeeded(&self) -> serde_json::Value {
        Self {
            event_type: Some("payment_intent.succeeded".to_string()),
            ..self.clone()
        }
        .build_json()
    }

    /// Build a `payment_intent.payment_failed` event
    #[must_use]
    pub fn build_payment_failed(&self) -> serde_json::Value {
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

impl Clone for MockWebhookEventBuilder {
    fn clone(&self) -> Self {
        Self {
            event_id: self.event_id.clone(),
            event_type: self.event_type.clone(),
            payment_intent_id: self.payment_intent_id.clone(),
            payment_id: self.payment_id.clone(),
            user_id: self.user_id.clone(),
            amount: self.amount,
            currency: self.currency.clone(),
        }
    }
}

/// Load a webhook fixture from the fixtures directory
///
/// # Errors
///
/// Returns an error if the file cannot be read or parsed
pub fn load_webhook_fixture(fixture_name: &str) -> Result<serde_json::Value, std::io::Error> {
    let path = format!("tests/fixtures/stripe/{fixture_name}");
    let content = std::fs::read_to_string(&path)?;
    serde_json::from_str(&content)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_customer_creation() {
        let mock = MockStripeClient::new();

        let mut metadata = HashMap::new();
        metadata.insert("user_id".to_string(), "test-user-123".to_string());

        let result = mock.create_customer(Some("test@example.com"), metadata);
        assert!(result.is_ok());

        let customer = result.expect("Expected success");
        assert!(customer.id.starts_with("cus_mock_"));
        assert_eq!(customer.email, Some("test@example.com".to_string()));

        assert_eq!(mock.customer_call_count(), 1);
    }

    #[test]
    fn test_mock_customer_creation_failure() {
        let config = MockStripeConfig {
            fail_customer_creation: true,
            error_message: Some("Card declined".to_string()),
            ..Default::default()
        };
        let mock = MockStripeClient::with_config(config);

        let result = mock.create_customer(Some("test@example.com"), HashMap::new());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Card declined");
    }

    #[test]
    fn test_mock_payment_intent_creation() {
        let mock = MockStripeClient::new();

        let mut metadata = HashMap::new();
        metadata.insert("payment_id".to_string(), "pay-123".to_string());

        let result =
            mock.create_payment_intent(2500, "eur", Some("cus_123"), metadata);
        assert!(result.is_ok());

        let intent = result.expect("Expected success");
        assert!(intent.id.starts_with("pi_mock_"));
        assert!(intent.client_secret.contains("_secret_mock"));
        assert_eq!(intent.amount, 2500);
        assert_eq!(intent.currency, "eur");
        assert_eq!(intent.customer, Some("cus_123".to_string()));

        assert_eq!(mock.payment_intent_call_count(), 1);
    }

    #[test]
    fn test_mock_payment_intent_failure() {
        let config = MockStripeConfig {
            fail_payment_intent_creation: true,
            ..Default::default()
        };
        let mock = MockStripeClient::with_config(config);

        let result = mock.create_payment_intent(1000, "usd", None, HashMap::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_webhook_event_builder() {
        let event = MockWebhookEventBuilder::new()
            .event_id("evt_test123")
            .event_type("payment_intent.succeeded")
            .payment_intent_id("pi_test456")
            .payment_id("pay_789")
            .user_id("user_abc")
            .amount(5000)
            .currency("gbp")
            .build_json();

        assert_eq!(event["id"], "evt_test123");
        assert_eq!(event["type"], "payment_intent.succeeded");
        assert_eq!(event["data"]["object"]["id"], "pi_test456");
        assert_eq!(event["data"]["object"]["amount"], 5000);
        assert_eq!(event["data"]["object"]["currency"], "gbp");
        assert_eq!(event["data"]["object"]["metadata"]["payment_id"], "pay_789");
        assert_eq!(event["data"]["object"]["metadata"]["user_id"], "user_abc");
    }

    #[test]
    fn test_build_payment_succeeded() {
        let event = MockWebhookEventBuilder::new()
            .payment_id("pay_123")
            .build_payment_succeeded();

        assert_eq!(event["type"], "payment_intent.succeeded");
        assert_eq!(event["data"]["object"]["status"], "succeeded");
    }

    #[test]
    fn test_build_payment_failed() {
        let event = MockWebhookEventBuilder::new()
            .payment_id("pay_123")
            .build_payment_failed();

        assert_eq!(event["type"], "payment_intent.payment_failed");
        assert_eq!(
            event["data"]["object"]["status"],
            "requires_payment_method"
        );
        assert!(event["data"]["object"]["last_payment_error"].is_object());
    }

    #[test]
    fn test_call_recording() {
        let mock = MockStripeClient::new();

        // Make multiple calls
        let _ = mock.create_customer(Some("user1@test.com"), HashMap::new());
        let _ = mock.create_customer(Some("user2@test.com"), HashMap::new());
        let _ = mock.create_payment_intent(1000, "usd", None, HashMap::new());

        assert_eq!(mock.customer_call_count(), 2);
        assert_eq!(mock.payment_intent_call_count(), 1);

        let customer_calls = mock.get_customer_calls();
        assert_eq!(customer_calls[0].email, Some("user1@test.com".to_string()));
        assert_eq!(customer_calls[1].email, Some("user2@test.com".to_string()));
    }
}
