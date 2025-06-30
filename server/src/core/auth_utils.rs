// web-template/server/src/core/auth_utils.rs

use std::sync::Arc;

use crate::{
    core::AppState,
    errors::AppResult,
    models::{AuthUser, PaymentUser, UnifiedAuthResponse, User},
    services::payment::PaymentDbOperations,
};

/// Build a unified auth response for any authentication flow
/// This ensures consistent response format across all auth methods
///
/// # Arguments
/// * `state` - Application state with services
/// * `user` - The authenticated user
/// * `token` - JWT token for the user
///
/// # Returns
/// * `UnifiedAuthResponse` with user, token, and payment information
///
/// # Errors
/// Returns an error if:
/// * Failed to fetch invite information
/// * Failed to fetch payment information
pub async fn build_unified_auth_response(
    state: &Arc<AppState>,
    user: &User,
    token: String,
) -> AppResult<UnifiedAuthResponse> {
    // Get invite information - check if user has EVER had an invite
    let invite = state.invite_service.get_user_invite(&user.email).await?;

    // Get active payment information
    let payment = state
        .payment_service
        .get_active_payment_for_user(user.id)
        .await?;

    // Create unified response components
    let auth_user = AuthUser::from(user.clone());
    let payment_user = PaymentUser::from_payment_and_invite(payment.as_ref(), invite.as_ref());

    tracing::debug!(
        "Built unified auth response for user: {} (payment_required: {}, has_valid_invite: {})",
        user.email,
        payment_user.payment_required,
        payment_user.has_valid_invite
    );

    Ok(UnifiedAuthResponse {
        auth_token: token,
        auth_user,
        payment_user,
    })
}

/// Build a unified auth response without a token (for endpoints where user is already authenticated)
/// Used by the /api/users/me endpoint
///
/// # Errors
/// Returns an error if:
/// * Failed to fetch invite information
/// * Failed to fetch payment information
pub async fn build_unified_auth_response_no_token(
    state: &Arc<AppState>,
    user: &User,
) -> AppResult<UnifiedAuthResponse> {
    let mut response = build_unified_auth_response(state, user, String::new()).await?;
    response.auth_token = String::new(); // Ensure token is empty
    Ok(response)
}
