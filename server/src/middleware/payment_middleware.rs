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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::auth_handler::RegisterUserPayload;
    use crate::models::User;
    use crate::services::payment::PaymentDbOperations;
    use crate::test_helpers::create_test_app_state;
    use sqlx::SqlitePool;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("Failed to create test database");

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");

        pool
    }

    async fn create_test_user(state: &Arc<AppState>, email: &str) -> User {
        state
            .user
            .create_user(&RegisterUserPayload {
                email: email.to_string(),
                password: "test_password123".to_string(),
            })
            .await
            .expect("Failed to create test user")
    }

    #[tokio::test]
    async fn test_check_payment_status_with_valid_invite() {
        let pool = setup_test_db().await;
        let state = create_test_app_state(&pool);

        let email = "test-invite@example.com".to_string();

        // Create invite
        state
            .invite
            .create_invite(&email, Some("Test invite".to_string()), None)
            .await
            .expect("Failed to create invite");

        // Create user using the service
        let user = create_test_user(&state, &email).await;

        let auth_user = AuthenticatedUser {
            user_id: user.id,
            email: user.email,
        };

        let result = check_payment_status(&state, &auth_user).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_check_payment_status_with_active_payment() {
        let pool = setup_test_db().await;
        let state = create_test_app_state(&pool);

        let email = "test-active@example.com".to_string();

        // Create user using the service
        let user = create_test_user(&state, &email).await;

        // Create payment using the service
        let payment = state
            .payment
            .create_payment(user.id, crate::models::payment::PaymentType::Subscription)
            .await
            .expect("Failed to create payment");

        // Update to active status
        state
            .payment
            .update_payment_status(payment.id, crate::models::payment::PaymentStatus::Active)
            .await
            .expect("Failed to update payment status");

        let auth_user = AuthenticatedUser {
            user_id: user.id,
            email: user.email,
        };

        let result = check_payment_status(&state, &auth_user).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_check_payment_status_no_invite_no_payment() {
        let pool = setup_test_db().await;
        let state = create_test_app_state(&pool);

        let email = "test-no-access@example.com".to_string();

        // Create user but no invite or payment
        let user = create_test_user(&state, &email).await;

        let auth_user = AuthenticatedUser {
            user_id: user.id,
            email: user.email,
        };

        let result = check_payment_status(&state, &auth_user).await;
        assert!(result.is_err());
        assert!(matches!(
            result.expect_err("Expected PaymentRequired error"),
            AppError::PaymentRequired
        ));
    }

    #[tokio::test]
    async fn test_check_payment_status_with_active_payment_no_invite() {
        let pool = setup_test_db().await;
        let state = create_test_app_state(&pool);

        let email = "test-active-no-invite@example.com".to_string();

        // Create user using the service
        let user = create_test_user(&state, &email).await;

        // Create payment using the service
        let payment = state
            .payment
            .create_payment(user.id, crate::models::payment::PaymentType::Subscription)
            .await
            .expect("Failed to create payment");

        // Update to active status
        state
            .payment
            .update_payment_status(payment.id, crate::models::payment::PaymentStatus::Active)
            .await
            .expect("Failed to update payment status");

        // This test verifies that active payments (without invites) are allowed

        let auth_user = AuthenticatedUser {
            user_id: user.id,
            email: user.email,
        };

        let result = check_payment_status(&state, &auth_user).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_payment_required_response_serialization() {
        let response = PaymentRequiredResponse {
            error: "Payment required".to_string(),
            payment_required: true,
        };

        let json = serde_json::to_string(&response).expect("Failed to serialize");
        assert!(json.contains("Payment required"));
        assert!(json.contains("\"payment_required\":true"));
    }

    #[tokio::test]
    async fn test_check_payment_status_with_cancelled_payment() {
        let pool = setup_test_db().await;
        let state = create_test_app_state(&pool);

        let email = "test-cancelled@example.com".to_string();

        // Create user using the service
        let user = create_test_user(&state, &email).await;

        // Create payment using the service
        let payment = state
            .payment
            .create_payment(user.id, crate::models::payment::PaymentType::Subscription)
            .await
            .expect("Failed to create payment");

        // Update to cancelled status - get_user_payment_status only returns active payments
        // so this will return has_active_payment: false, causing PaymentRequired error
        state
            .payment
            .update_payment_status(payment.id, crate::models::payment::PaymentStatus::Cancelled)
            .await
            .expect("Failed to update payment status");

        let auth_user = AuthenticatedUser {
            user_id: user.id,
            email: user.email,
        };

        let result = check_payment_status(&state, &auth_user).await;
        assert!(result.is_err());
        assert!(matches!(
            result.expect_err("Expected PaymentRequired error"),
            AppError::PaymentRequired
        ));
    }

    #[tokio::test]
    async fn test_check_payment_status_trialing_payment() {
        let pool = setup_test_db().await;
        let state = create_test_app_state(&pool);

        let email = "test-trial@example.com".to_string();

        // Create user using the service
        let user = create_test_user(&state, &email).await;

        // Create trialing payment using the service
        let payment = state
            .payment
            .create_payment(user.id, crate::models::payment::PaymentType::Subscription)
            .await
            .expect("Failed to create payment");

        // Update to active status (trialing payments are also "active")
        state
            .payment
            .update_payment_status(payment.id, crate::models::payment::PaymentStatus::Active)
            .await
            .expect("Failed to update payment status");

        let auth_user = AuthenticatedUser {
            user_id: user.id,
            email: user.email,
        };

        let result = check_payment_status(&state, &auth_user).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_payment_required_response_debug() {
        let response = PaymentRequiredResponse {
            error: "Test error".to_string(),
            payment_required: false,
        };

        let debug_str = format!("{response:?}");
        assert!(debug_str.contains("PaymentRequiredResponse"));
        assert!(debug_str.contains("Test error"));
    }
}
