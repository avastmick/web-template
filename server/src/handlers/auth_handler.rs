// web-template/server/src/handlers/auth_handler.rs

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::Deserialize;
use std::sync::Arc;
use validator::Validate;

use crate::{
    core::{AppState, build_unified_auth_response, password_utils::verify_password},
    errors::{AppError, AppResult},
};

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterUserPayload {
    #[validate(email(message = "Email must be a valid email address."))]
    pub email: String,

    #[validate(length(min = 12, message = "Password must be at least 12 characters long."))]
    // TODO: Implement more robust password complexity validation
    // e.g., using a custom validation function:
    // #[validate(custom(function = "validate_password_complexity"))]
    // fn validate_password_complexity(password: &str) -> Result<(), validator::ValidationError> { ... }
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginUserPayload {
    #[validate(email(message = "Email must be a valid email address."))]
    pub email: String,

    #[validate(length(min = 1, message = "Password is required."))]
    pub password: String,
}

// Note: We now use UnifiedAuthResponse for all auth endpoints
// The old UserResponse, RegisterResponse, and LoginResponse types have been replaced

/// Register a new user
///
/// # Errors
///
/// Returns an error if validation fails, user already exists, or database operation fails
#[tracing::instrument(skip(state, payload), fields(email = %payload.email), err(Debug))]
pub async fn register_user_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterUserPayload>,
) -> AppResult<impl IntoResponse> {
    // 1. Validate the payload
    if let Err(validation_errors) = payload.validate() {
        tracing::warn!(
            "Validation failed for registration payload: {:?}",
            validation_errors
        );
        // Convert validator::ValidationErrors to a user-friendly string.
        // For a production app, you might want a more structured error response.
        let first_error = validation_errors
            .field_errors()
            .into_iter()
            .next()
            .map_or_else(
                || "Invalid input.".to_string(),
                |(_, errors)| {
                    errors
                        .first()
                        .and_then(|err| err.message.as_ref().map(std::string::ToString::to_string))
                        .unwrap_or_else(|| "Unknown validation error".to_string())
                },
            );
        return Err(AppError::ValidationError(first_error));
    }

    tracing::info!(
        "Registration payload validated successfully for email: {}",
        payload.email
    );

    // 2. Check if user has a valid invite and get the invite details
    let invite = state
        .invite_service
        .get_valid_invite(&payload.email)
        .await?;

    // For MVP, we allow registration without invite but flag that payment is required
    if invite.is_some() {
        tracing::info!("Valid invite found for email: {}", payload.email);
    } else {
        tracing::info!(
            "Registration without invite for email: {} - payment will be required",
            payload.email
        );
    }

    // 3. Call the user service to attempt user creation
    match state.user_service.create_user(&payload).await {
        Ok(created_user) => {
            // Mark invite as used
            // FIXME: should check state and log as info if INVITENOTFOUND, only log error if not
            // that
            if let Err(e) = state.invite_service.mark_invite_used(&payload.email).await {
                tracing::error!(
                    "Failed to mark invite as used for email {}: {:?}",
                    payload.email,
                    e
                );
                // We don't fail the registration since the user is already created
            }

            // Generate JWT token for immediate login (matches OAuth behavior)
            let token = state
                .auth_service
                .generate_token(created_user.id, &created_user.email)
                .map_err(|e| {
                    tracing::error!(
                        "Failed to generate token for user {}: {:?}",
                        created_user.email,
                        e
                    );
                    e
                })?;

            tracing::info!(
                "User successfully created with email: {}",
                created_user.email
            );

            // Create unified auth response using shared function
            let response = build_unified_auth_response(&state, &created_user, token).await?;

            Ok((StatusCode::CREATED, Json(response)))
        }
        Err(app_error) => {
            // Log the AppError variant, but not necessarily the full detail if it's sensitive
            // or already logged more deeply (e.g., SqlxError in AppError::into_response)
            tracing::error!("Failed to create user (handler level): {:?}", app_error);
            Err(app_error) // Propagate the AppError
        }
    }
}

/// Login user
///
/// # Errors
///
/// Returns an error if validation fails, user not found, password incorrect, or JWT generation fails
#[tracing::instrument(skip(state, payload), fields(email = %payload.email), err(Debug))]
pub async fn login_user_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginUserPayload>,
) -> AppResult<impl IntoResponse> {
    // 1. Validate the payload
    if let Err(validation_errors) = payload.validate() {
        tracing::warn!(
            "Validation failed for login payload: {:?}",
            validation_errors
        );
        let first_error = validation_errors
            .field_errors()
            .into_iter()
            .next()
            .map_or_else(
                || "Invalid input.".to_string(),
                |(_, errors)| {
                    errors
                        .first()
                        .and_then(|err| err.message.as_ref().map(std::string::ToString::to_string))
                        .unwrap_or_else(|| "Unknown validation error".to_string())
                },
            );
        return Err(AppError::ValidationError(first_error));
    }

    tracing::info!("Login attempt for email: {}", payload.email);

    // 2. Find user by email
    let user = match state.user_service.find_by_email(&payload.email).await {
        Ok(user) => user,
        Err(AppError::UserNotFound) => {
            tracing::warn!("Login attempt with non-existent email: {}", payload.email);
            return Err(AppError::InvalidCredentials);
        }
        Err(e) => return Err(e),
    };

    // 3. Verify password
    if let Err(e) = verify_password(&payload.password, &user.hashed_password) {
        tracing::warn!(
            "Invalid password attempt for user: {} - error: {:?}",
            payload.email,
            e
        );
        return Err(AppError::InvalidCredentials);
    }

    // 4. Generate JWT token
    let token = state
        .auth_service
        .generate_token(user.id, &user.email)
        .map_err(|e| {
            tracing::error!("Failed to generate token for user {}: {:?}", user.email, e);
            e
        })?;

    // 5. Create unified auth response using shared function
    let response = build_unified_auth_response(&state, &user, token).await?;

    if response.payment_user.payment_required {
        tracing::info!(
            "User {} requires payment (no invite and no active payment)",
            response.auth_user.email
        );
    }

    tracing::info!("User logged in successfully: {}", response.auth_user.email);

    Ok((StatusCode::OK, Json(response)))
}
