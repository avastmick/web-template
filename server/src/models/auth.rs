// web-template/server/src/models/auth.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{invite::UserInvite, payment::UserPayment, user::User};

/// Unified authentication response for all auth methods (OAuth, email/password)
/// This ensures consistent client-side handling regardless of auth method
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnifiedAuthResponse {
    /// JWT authentication token
    pub auth_token: String,

    /// User information
    pub auth_user: AuthUser,

    /// Payment/subscription information
    pub payment_user: PaymentUser,
}

/// User information included in auth responses
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Payment/subscription status information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaymentUser {
    /// Whether payment is required to access the application
    pub payment_required: bool,

    /// Current payment status if payment exists
    pub payment_status: Option<String>,

    /// Subscription end date if applicable
    pub subscription_end_date: Option<DateTime<Utc>>,

    /// Whether user has a valid invite
    pub has_valid_invite: bool,

    /// Invite expiry date if applicable
    pub invite_expires_at: Option<DateTime<Utc>>,
}

impl From<User> for AuthUser {
    fn from(user: User) -> Self {
        AuthUser {
            id: user.id,
            email: user.email,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

impl PaymentUser {
    /// Create `PaymentUser` from optional payment and invite data
    #[must_use]
    pub fn from_payment_and_invite(
        payment: Option<&UserPayment>,
        invite: Option<&UserInvite>,
    ) -> Self {
        let has_valid_invite = invite.is_some_and(|inv| !inv.is_expired() && inv.used_at.is_none());

        let payment_required = !has_valid_invite && payment.is_none_or(|p| !p.is_active());

        PaymentUser {
            payment_required,
            payment_status: payment.map(|p| p.payment_status.as_str().to_string()),
            subscription_end_date: payment.and_then(|p| p.subscription_end_date),
            has_valid_invite,
            invite_expires_at: invite.and_then(|inv| inv.expires_at),
        }
    }
}

/// OAuth callback response parameters
/// These are passed as query parameters in the redirect URL
#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthCallbackParams {
    pub token: String,
    pub user_id: Uuid,
    pub email: String,
    pub is_new_user: bool,
    pub payment_required: bool,
    pub has_valid_invite: bool,
}

impl OAuthCallbackParams {
    #[must_use]
    pub fn from_unified_response(response: &UnifiedAuthResponse, is_new_user: bool) -> Self {
        OAuthCallbackParams {
            token: response.auth_token.clone(),
            user_id: response.auth_user.id,
            email: response.auth_user.email.clone(),
            is_new_user,
            payment_required: response.payment_user.payment_required,
            has_valid_invite: response.payment_user.has_valid_invite,
        }
    }
}
