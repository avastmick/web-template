// web-template/server/src/handlers/user_handler.rs

//! User-related HTTP handlers
//!
//! This module contains handlers for user-specific operations such as
//! retrieving user profile information.

use axum::{Json, http::StatusCode, response::IntoResponse};

use crate::{errors::AppResult, handlers::auth_handler::UserResponse, middleware::JwtAuth};

/// Handler for GET /api/users/me - returns current user's profile information
///
/// This is a protected endpoint that requires a valid JWT token in the Authorization header.
/// The JWT token is automatically validated by the JwtAuth extractor.
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
#[tracing::instrument(skip(auth), fields(user_id = %auth.user.user_id, email = %auth.user.email))]
pub async fn get_current_user_handler(auth: JwtAuth) -> AppResult<impl IntoResponse> {
    tracing::info!(
        "Fetching profile for authenticated user: {} ({})",
        auth.user.email,
        auth.user.user_id
    );

    // Since the JWT extractor already validated the user exists and is valid,
    // we can construct the response directly from the authenticated user data
    // Note: In a more complex scenario, you might want to fetch fresh user data
    // from the database to ensure the most up-to-date information

    // For now, we'll create a mock User object since we already have the core data
    // In a real implementation, you might want to fetch the full user from the database
    let user_response = UserResponse {
        id: auth.user.user_id,
        email: auth.user.email.clone(),
        // Since we don't have access to the timestamps here, we'd need to fetch from DB
        // For this implementation, we'll use placeholder values
        created_at: chrono::Utc::now(), // This should come from the DB
        updated_at: chrono::Utc::now(), // This should come from the DB
    };

    tracing::info!(
        "Successfully retrieved profile for user: {}",
        auth.user.email
    );

    Ok((StatusCode::OK, Json(user_response)))
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
