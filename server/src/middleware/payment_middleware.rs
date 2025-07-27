//! Payment status verification for protecting routes that require active payment

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use std::sync::Arc;

use crate::{
    core::AppState,
    errors::{AppError, AppResult},
    middleware::auth_middleware::AuthenticatedUser,
};

#[derive(Debug, Serialize)]
pub struct PaymentRequiredResponse {
    pub error: String,
    pub payment_required: bool,
}

/// Check if user has valid payment or invite status
/// Returns Ok(()) if user has access, or Err with payment required response
///
/// # Errors
///
/// Returns `AppError::PaymentRequired` if user has no valid invite and no active payment
pub async fn check_payment_status(
    state: &Arc<AppState>,
    user: &AuthenticatedUser,
) -> AppResult<()> {
    let user_id = user.user_id;
    let user_email = &user.email;

    // Check if user has a valid invite
    let has_invite = state.invite.check_invite_exists(user_email).await?;

    // Check user's payment status
    let payment_status = state.payment.get_user_payment_status(user_id).await?;

    // User needs either a valid invite OR an active payment
    if !has_invite && !payment_status.has_active_payment {
        tracing::warn!(
            "Access denied for user {} ({}): no valid invite or active payment",
            user_email,
            user_id
        );

        return Err(AppError::PaymentRequired);
    }

    // Check if payment subscription has expired
    if let Some(subscription_end_date) = payment_status.subscription_end_date {
        if subscription_end_date < chrono::Utc::now() {
            tracing::warn!(
                "Access denied for user {} ({}): payment subscription expired at {}",
                user_email,
                user_id,
                subscription_end_date
            );

            return Err(AppError::PaymentRequired);
        }
    }

    Ok(())
}

/// Extractor that validates payment status after JWT authentication
pub struct PaymentRequired;

impl axum::extract::FromRequestParts<Arc<AppState>> for PaymentRequired {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        // First, extract the authenticated user using JWT
        let jwt_auth = crate::middleware::JwtAuth::from_request_parts(parts, state).await?;

        // Then check payment status
        check_payment_status(state, &jwt_auth.user)
            .await
            .map_err(|e| {
                if matches!(e, AppError::PaymentRequired) {
                    let response = PaymentRequiredResponse {
                        error: "Payment required to access this resource".to_string(),
                        payment_required: true,
                    };
                    (StatusCode::PAYMENT_REQUIRED, Json(response)).into_response()
                } else {
                    e.into_response()
                }
            })?;

        Ok(PaymentRequired)
    }
}
