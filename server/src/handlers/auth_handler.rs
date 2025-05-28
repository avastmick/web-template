// web-template/server/src/handlers/auth_handler.rs

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use validator::Validate;

use crate::{
    core::password_utils::verify_password,
    errors::{AppError, AppResult},
    models::User,
    services::{AuthService, UserServiceImpl},
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

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: uuid::Uuid,
    pub email: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserResponse,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        UserResponse {
            id: user.id,
            email: user.email,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

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

    // 2. Call the user service to attempt user creation
    match state.user_service.create_user(&payload).await {
        Ok(created_user) => {
            tracing::info!(
                "User successfully created with email: {}",
                created_user.email
            );
            let user_response = UserResponse::from(created_user);
            Ok((StatusCode::CREATED, Json(user_response)))
        }
        Err(app_error) => {
            // Log the AppError variant, but not necessarily the full detail if it's sensitive
            // or already logged more deeply (e.g., SqlxError in AppError::into_response)
            tracing::error!("Failed to create user (handler level): {:?}", app_error);
            Err(app_error) // Propagate the AppError
        }
    }
}

/// Application state for handlers that need both services
pub struct AppState {
    pub user_service: Arc<UserServiceImpl>,
    pub auth_service: Arc<AuthService>,
}

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

    tracing::info!("User logged in successfully: {}", user.email);

    let response = LoginResponse {
        token,
        user: UserResponse::from(user),
    };

    Ok((StatusCode::OK, Json(response)))
}
