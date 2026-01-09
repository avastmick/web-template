//! Payment API Contract Snapshots
//!
//! Tests for payment response schema stability.

use chrono::{TimeZone, Utc};
use insta::assert_json_snapshot;
use serde::{Deserialize, Serialize};

/// Payment status enum (mirrors server::models::payment::PaymentStatus)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatus {
    Pending,
    Active,
    Cancelled,
    Expired,
    Failed,
}

/// Payment type enum (mirrors server::models::payment::PaymentType)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PaymentType {
    Subscription,
    OneTime,
}

/// Create payment intent response (mirrors server::models::payment::CreatePaymentIntentResponse)
#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePaymentIntentResponse {
    pub client_secret: String,
    pub payment_intent_id: String,
}

/// User payment status response (mirrors server::models::payment::UserPaymentStatusResponse)
#[derive(Debug, Serialize, Deserialize)]
pub struct UserPaymentStatusResponse {
    pub has_active_payment: bool,
    pub payment_status: Option<PaymentStatus>,
    pub payment_type: Option<PaymentType>,
    pub subscription_end_date: Option<chrono::DateTime<Utc>>,
}

// Fixed timestamp for deterministic snapshots
fn fixed_future_timestamp() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2025, 1, 15, 12, 0, 0).unwrap()
}

#[test]
fn test_create_payment_intent_response() {
    let response = CreatePaymentIntentResponse {
        client_secret: "pi_1234567890_secret_abcdefghij".to_string(),
        payment_intent_id: "pi_1234567890".to_string(),
    };

    assert_json_snapshot!("create_payment_intent_response", response);
}

#[test]
fn test_user_payment_status_no_payment() {
    let response = UserPaymentStatusResponse {
        has_active_payment: false,
        payment_status: None,
        payment_type: None,
        subscription_end_date: None,
    };

    assert_json_snapshot!("user_payment_status_no_payment", response);
}

#[test]
fn test_user_payment_status_active_subscription() {
    let response = UserPaymentStatusResponse {
        has_active_payment: true,
        payment_status: Some(PaymentStatus::Active),
        payment_type: Some(PaymentType::Subscription),
        subscription_end_date: Some(fixed_future_timestamp()),
    };

    assert_json_snapshot!("user_payment_status_active_subscription", response);
}

#[test]
fn test_user_payment_status_one_time_payment() {
    let response = UserPaymentStatusResponse {
        has_active_payment: true,
        payment_status: Some(PaymentStatus::Active),
        payment_type: Some(PaymentType::OneTime),
        subscription_end_date: None,
    };

    assert_json_snapshot!("user_payment_status_one_time_payment", response);
}

#[test]
fn test_user_payment_status_pending() {
    let response = UserPaymentStatusResponse {
        has_active_payment: false,
        payment_status: Some(PaymentStatus::Pending),
        payment_type: Some(PaymentType::Subscription),
        subscription_end_date: None,
    };

    assert_json_snapshot!("user_payment_status_pending", response);
}

#[test]
fn test_user_payment_status_cancelled() {
    let response = UserPaymentStatusResponse {
        has_active_payment: false,
        payment_status: Some(PaymentStatus::Cancelled),
        payment_type: Some(PaymentType::Subscription),
        subscription_end_date: Some(fixed_future_timestamp()),
    };

    assert_json_snapshot!("user_payment_status_cancelled", response);
}

#[test]
fn test_user_payment_status_expired() {
    let response = UserPaymentStatusResponse {
        has_active_payment: false,
        payment_status: Some(PaymentStatus::Expired),
        payment_type: Some(PaymentType::Subscription),
        subscription_end_date: None,
    };

    assert_json_snapshot!("user_payment_status_expired", response);
}

#[test]
fn test_user_payment_status_failed() {
    let response = UserPaymentStatusResponse {
        has_active_payment: false,
        payment_status: Some(PaymentStatus::Failed),
        payment_type: Some(PaymentType::OneTime),
        subscription_end_date: None,
    };

    assert_json_snapshot!("user_payment_status_failed", response);
}

#[test]
fn test_payment_status_enum_serialization() {
    // Test all payment status variants serialize correctly
    assert_json_snapshot!("payment_status_pending", PaymentStatus::Pending);
    assert_json_snapshot!("payment_status_active", PaymentStatus::Active);
    assert_json_snapshot!("payment_status_cancelled", PaymentStatus::Cancelled);
    assert_json_snapshot!("payment_status_expired", PaymentStatus::Expired);
    assert_json_snapshot!("payment_status_failed", PaymentStatus::Failed);
}

#[test]
fn test_payment_type_enum_serialization() {
    // Test all payment type variants serialize correctly
    assert_json_snapshot!("payment_type_subscription", PaymentType::Subscription);
    assert_json_snapshot!("payment_type_one_time", PaymentType::OneTime);
}
