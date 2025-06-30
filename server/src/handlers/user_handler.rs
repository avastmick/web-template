// web-template/server/src/handlers/user_handler.rs

//! User-related HTTP handlers
//!
//! This module contains handlers for user-specific operations such as
//! retrieving user profile information.

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use std::sync::Arc;

use crate::{
    core::AppState,
    errors::AppResult,
    middleware::JwtAuth,
    models::{AuthUser, PaymentUser, UnifiedAuthResponse},
    services::payment::PaymentDbOperations,
};

/// Handler for GET /api/users/me - returns current user's profile information
///
/// This is a protected endpoint that requires a valid JWT token in the Authorization header.
/// The JWT token is automatically validated by the `JwtAuth` extractor.
///
/// # Arguments
/// * `auth` - JWT authentication extractor containing the authenticated user
///
/// # Returns
/// Returns the user's profile information (excluding sensitive data like password)
///
/// # Errors
/// Returns appropriate HTTP error responses:
/// * 401 Unauthorized - If JWT token is missing, invalid, or expired
/// * 404 Not Found - If the user referenced in the JWT no longer exists
/// * 500 Internal Server Error - For database or other server errors
/// Get the current user's profile information
///
/// # Errors
/// Returns appropriate HTTP error responses:
/// * 401 Unauthorized - If JWT token is missing, invalid, or expired
/// * 500 Internal Server Error - For database or other server errors
#[tracing::instrument(skip(auth, state), fields(user_id = %auth.user.user_id, email = %auth.user.email))]
pub async fn get_current_user_handler(
    auth: JwtAuth,
    State(state): State<Arc<AppState>>,
) -> AppResult<impl IntoResponse> {
    tracing::info!(
        "Fetching profile for authenticated user: {} ({})",
        auth.user.email,
        auth.user.user_id
    );

    // Fetch the full user from the database
    let user = state.user_service.find_by_email(&auth.user.email).await?;

    // Check if user has valid invite or active payment
    let invite = state.invite_service.get_valid_invite(&user.email).await?;

    let payment = state
        .payment_service
        .get_active_payment_for_user(user.id)
        .await?;

    // Create unified auth response (without token since user is already authenticated)
    let auth_user = AuthUser::from(user);
    let payment_user = PaymentUser::from_payment_and_invite(payment.as_ref(), invite.as_ref());

    if payment_user.payment_required {
        tracing::info!(
            "User {} requires payment (no invite and no active payment)",
            auth_user.email
        );
    }

    let response = UnifiedAuthResponse {
        auth_token: String::new(), // Empty token for /me endpoint since user is already authenticated
        auth_user: auth_user.clone(),
        payment_user,
    };

    tracing::info!(
        "Successfully retrieved profile for user: {} (payment_required: {})",
        auth_user.email,
        response.payment_user.payment_required
    );

    Ok((StatusCode::OK, Json(response)))
}

// Alternative implementation that fetches fresh data from the database
// This would be used if you want the most up-to-date user information
//
// Note: This is commented out as an example of how you might implement
// a version that fetches fresh data from the database
/*
use axum::extract::State;
use std::sync::Arc;
use crate::handlers::auth_handler::AppState;

#[tracing::instrument(skip(auth, state), fields(user_id = %auth.user.user_id, email = %auth.user.email))]
pub async fn get_current_user_handler_with_db_fetch(
    auth: JwtAuth,
    State(state): State<Arc<AppState>>,
) -> AppResult<impl IntoResponse> {
    tracing::info!(
        "Fetching fresh profile data for authenticated user: {} ({})",
        auth.user.email,
        auth.user.user_id
    );

    // Fetch the most up-to-date user data from the database
    let user = state
        .user_service
        .find_by_email(&auth.user.email)
        .await?;

    let user_response = UserResponse::from(user);

    tracing::info!(
        "Successfully retrieved fresh profile for user: {}",
        auth.user.email
    );

    Ok((StatusCode::OK, Json(user_response)))
}
*/
