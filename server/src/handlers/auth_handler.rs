// web-template/server/src/handlers/auth_handler.rs

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use validator::Validate;

use crate::{
    errors::{AppError, AppResult},
    models::User,
    services::UserServiceImpl,
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

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: uuid::Uuid,
    pub email: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
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

#[tracing::instrument(skip(user_service, payload), fields(email = %payload.email), err(Debug))]
pub async fn register_user_handler(
    State(user_service): State<Arc<UserServiceImpl>>,
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
    match user_service.create_user(&payload).await {
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
