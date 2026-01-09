//! Authentication API Contract Snapshots
//!
//! Tests for auth response schema stability.

use chrono::{TimeZone, Utc};
use insta::assert_json_snapshot;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Simplified AuthUser for contract testing (mirrors server::models::auth::AuthUser)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub email: String,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

/// Simplified PaymentUser for contract testing (mirrors server::models::auth::PaymentUser)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaymentUser {
    pub payment_required: bool,
    pub payment_status: Option<String>,
    pub subscription_end_date: Option<chrono::DateTime<Utc>>,
    pub has_valid_invite: bool,
    pub invite_expires_at: Option<chrono::DateTime<Utc>>,
}

/// Simplified UnifiedAuthResponse for contract testing
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnifiedAuthResponse {
    pub auth_token: String,
    pub auth_user: AuthUser,
    pub payment_user: PaymentUser,
}

/// OAuth callback parameters for contract testing
#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthCallbackParams {
    pub token: String,
    pub user_id: Uuid,
    pub email: String,
    pub is_new_user: bool,
    pub payment_required: bool,
    pub has_valid_invite: bool,
}

// Fixed timestamps and UUIDs for deterministic snapshots
fn fixed_uuid() -> Uuid {
    Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").expect("valid uuid")
}

fn fixed_timestamp() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2024, 1, 15, 12, 0, 0).unwrap()
}

fn fixed_future_timestamp() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2025, 1, 15, 12, 0, 0).unwrap()
}

#[test]
fn test_unified_auth_response_new_user() {
    let response = UnifiedAuthResponse {
        auth_token: "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.example_token".to_string(),
        auth_user: AuthUser {
            id: fixed_uuid(),
            email: "user@example.com".to_string(),
            created_at: fixed_timestamp(),
            updated_at: fixed_timestamp(),
        },
        payment_user: PaymentUser {
            payment_required: true,
            payment_status: None,
            subscription_end_date: None,
            has_valid_invite: false,
            invite_expires_at: None,
        },
    };

    assert_json_snapshot!("unified_auth_response_new_user", response);
}

#[test]
fn test_unified_auth_response_paid_user() {
    let response = UnifiedAuthResponse {
        auth_token: "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.example_token".to_string(),
        auth_user: AuthUser {
            id: fixed_uuid(),
            email: "paid@example.com".to_string(),
            created_at: fixed_timestamp(),
            updated_at: fixed_timestamp(),
        },
        payment_user: PaymentUser {
            payment_required: false,
            payment_status: Some("active".to_string()),
            subscription_end_date: Some(fixed_future_timestamp()),
            has_valid_invite: false,
            invite_expires_at: None,
        },
    };

    assert_json_snapshot!("unified_auth_response_paid_user", response);
}

#[test]
fn test_unified_auth_response_invited_user() {
    let response = UnifiedAuthResponse {
        auth_token: "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.example_token".to_string(),
        auth_user: AuthUser {
            id: fixed_uuid(),
            email: "invited@example.com".to_string(),
            created_at: fixed_timestamp(),
            updated_at: fixed_timestamp(),
        },
        payment_user: PaymentUser {
            payment_required: false,
            payment_status: None,
            subscription_end_date: None,
            has_valid_invite: true,
            invite_expires_at: Some(fixed_future_timestamp()),
        },
    };

    assert_json_snapshot!("unified_auth_response_invited_user", response);
}

#[test]
fn test_auth_user_schema() {
    let user = AuthUser {
        id: fixed_uuid(),
        email: "test@example.com".to_string(),
        created_at: fixed_timestamp(),
        updated_at: fixed_timestamp(),
    };

    assert_json_snapshot!("auth_user", user);
}

#[test]
fn test_payment_user_no_payment() {
    let payment_user = PaymentUser {
        payment_required: true,
        payment_status: None,
        subscription_end_date: None,
        has_valid_invite: false,
        invite_expires_at: None,
    };

    assert_json_snapshot!("payment_user_no_payment", payment_user);
}

#[test]
fn test_payment_user_active_subscription() {
    let payment_user = PaymentUser {
        payment_required: false,
        payment_status: Some("active".to_string()),
        subscription_end_date: Some(fixed_future_timestamp()),
        has_valid_invite: false,
        invite_expires_at: None,
    };

    assert_json_snapshot!("payment_user_active_subscription", payment_user);
}

#[test]
fn test_payment_user_cancelled_subscription() {
    let payment_user = PaymentUser {
        payment_required: true,
        payment_status: Some("cancelled".to_string()),
        subscription_end_date: Some(fixed_timestamp()),
        has_valid_invite: false,
        invite_expires_at: None,
    };

    assert_json_snapshot!("payment_user_cancelled_subscription", payment_user);
}

#[test]
fn test_oauth_callback_params_new_user() {
    let params = OAuthCallbackParams {
        token: "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.example_token".to_string(),
        user_id: fixed_uuid(),
        email: "oauth@example.com".to_string(),
        is_new_user: true,
        payment_required: true,
        has_valid_invite: false,
    };

    assert_json_snapshot!("oauth_callback_params_new_user", params);
}

#[test]
fn test_oauth_callback_params_existing_user() {
    let params = OAuthCallbackParams {
        token: "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.example_token".to_string(),
        user_id: fixed_uuid(),
        email: "existing@example.com".to_string(),
        is_new_user: false,
        payment_required: false,
        has_valid_invite: true,
    };

    assert_json_snapshot!("oauth_callback_params_existing_user", params);
}
