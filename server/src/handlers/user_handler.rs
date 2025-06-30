// web-template/server/src/handlers/user_handler.rs

//! User-related HTTP handlers
//!
//! This module contains handlers for user-specific operations such as
//! retrieving user profile information.

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use std::sync::Arc;

use crate::{
    core::{AppState, build_unified_auth_response_no_token},
    errors::AppResult,
    middleware::JwtAuth,
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

    // Create unified auth response using shared function (without token)
    let response = build_unified_auth_response_no_token(&state, &user).await?;

    tracing::info!(
        "Successfully retrieved profile for user: {} (payment_required: {})",
        response.auth_user.email,
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
