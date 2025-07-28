// kanbain/server/src/errors.rs
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

// Assuming crate::core::password_utils::PasswordError exists and is made public
// If not, you might need to adjust this path or define PasswordError differently.
pub use crate::core::password_utils::PasswordError;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error("Password handling error: {0}")]
    PasswordUtilError(#[from] PasswordError),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("User with email '{email}' already exists")]
    UserAlreadyExists { email: String },

    #[error("User not found")]
    UserNotFound,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("An internal server error occurred: {0}")]
    InternalServerError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("JWT error: {0}")]
    JwtError(String),

    #[error("Invite not found")]
    InviteNotFound,

    #[error("Invite expired")]
    InviteExpired,

    #[error("Invite already used")]
    InviteAlreadyUsed,

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Payment required")]
    PaymentRequired,

    #[error("Not found: {0}")]
    NotFound(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message, error_detail) = match self {
            AppError::SqlxError(e) => {
                tracing::error!("Database error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "A database error occurred.".to_string(),
                    Some(e.to_string()),
                )
            }
            AppError::PasswordUtilError(e) => {
                tracing::error!("Password handling error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "A password processing error occurred.".to_string(),
                    Some(e.to_string()),
                )
            }
            AppError::ValidationError(msg) | AppError::BadRequest(msg) => {
                (StatusCode::BAD_REQUEST, msg, None)
            }
            AppError::UserAlreadyExists { email } => (
                StatusCode::CONFLICT,
                format!("User with email '{email}' already exists."),
                None,
            ),
            AppError::UserNotFound => (StatusCode::NOT_FOUND, "User not found.".to_string(), None),
            AppError::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                "Invalid email or password.".to_string(),
                None,
            ),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg, None),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg, None),
            AppError::InternalServerError(ref msg) => {
                tracing::error!("Internal Server Error Context: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An internal server error occurred.".to_string(),
                    Some(msg.clone()),
                )
            }
            AppError::ConfigError(ref msg) => {
                tracing::error!("Configuration error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "A configuration error occurred.".to_string(),
                    Some(msg.clone()),
                )
            }
            AppError::ConfigurationError(ref msg) => {
                tracing::error!("Configuration error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "A configuration error occurred.".to_string(),
                    Some(msg.clone()),
                )
            }
            AppError::JwtError(ref msg) => {
                tracing::warn!("JWT error: {}", msg);
                (
                    StatusCode::UNAUTHORIZED,
                    format!("Authentication error: {msg}"),
                    None,
                )
            }
            AppError::InviteNotFound => (
                StatusCode::FORBIDDEN,
                "Registration is by invitation only.".to_string(),
                None,
            ),
            AppError::InviteExpired => (
                StatusCode::FORBIDDEN,
                "Your invitation has expired.".to_string(),
                None,
            ),
            AppError::InviteAlreadyUsed => (
                StatusCode::FORBIDDEN,
                "Your invitation has already been used.".to_string(),
                None,
            ),
            AppError::PaymentRequired => (
                StatusCode::PAYMENT_REQUIRED,
                "Payment required to access this resource.".to_string(),
                None,
            ),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg, None),
        };

        let mut body = json!({ "error": error_message });

        // Include detailed error in debug builds only for non-user-facing errors
        if cfg!(debug_assertions) {
            if let Some(detail) = error_detail {
                match status {
                    StatusCode::INTERNAL_SERVER_ERROR | StatusCode::BAD_REQUEST => {
                        body["detail"] = json!(detail);
                    }
                    _ => {} // Don't add detail for auth errors etc.
                }
            }
        }

        (status, Json(body)).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    #[tokio::test]
    async fn test_validation_error_response() {
        let error = AppError::ValidationError("Invalid input".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_user_already_exists_response() {
        let error = AppError::UserAlreadyExists {
            email: "test@example.com".to_string(),
        };
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn test_user_not_found_response() {
        let error = AppError::UserNotFound;
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_invalid_credentials_response() {
        let error = AppError::InvalidCredentials;
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_unauthorized_response() {
        let error = AppError::Unauthorized("Access denied".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_forbidden_response() {
        let error = AppError::Forbidden("Forbidden action".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_internal_server_error_response() {
        let error = AppError::InternalServerError("Something went wrong".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_config_error_response() {
        let error = AppError::ConfigError("Missing configuration".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_configuration_error_response() {
        let error = AppError::ConfigurationError("Invalid config".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_jwt_error_response() {
        let error = AppError::JwtError("Invalid token".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_invite_not_found_response() {
        let error = AppError::InviteNotFound;
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_invite_expired_response() {
        let error = AppError::InviteExpired;
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_invite_already_used_response() {
        let error = AppError::InviteAlreadyUsed;
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_bad_request_response() {
        let error = AppError::BadRequest("Bad input".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_payment_required_response() {
        let error = AppError::PaymentRequired;
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::PAYMENT_REQUIRED);
    }

    #[tokio::test]
    async fn test_not_found_response() {
        let error = AppError::NotFound("Resource not found".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_sqlx_error_response() {
        // We can't easily create a real SqlxError, so we'll test via the From trait
        let error = AppError::SqlxError(sqlx::Error::RowNotFound);
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_password_util_error_response() {
        use crate::core::password_utils::PasswordError;
        use argon2::password_hash::Error as PasswordHashError;

        let password_error = PasswordError::HashingError(PasswordHashError::Password);
        let error = AppError::PasswordUtilError(password_error);
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_error_display_messages() {
        // Test Display trait implementation
        let error = AppError::UserAlreadyExists {
            email: "test@example.com".to_string(),
        };
        assert_eq!(
            error.to_string(),
            "User with email 'test@example.com' already exists"
        );

        let error = AppError::ValidationError("Invalid data".to_string());
        assert_eq!(error.to_string(), "Validation error: Invalid data");

        let error = AppError::UserNotFound;
        assert_eq!(error.to_string(), "User not found");

        let error = AppError::InvalidCredentials;
        assert_eq!(error.to_string(), "Invalid credentials");

        let error = AppError::InviteNotFound;
        assert_eq!(error.to_string(), "Invite not found");

        let error = AppError::InviteExpired;
        assert_eq!(error.to_string(), "Invite expired");

        let error = AppError::InviteAlreadyUsed;
        assert_eq!(error.to_string(), "Invite already used");

        let error = AppError::PaymentRequired;
        assert_eq!(error.to_string(), "Payment required");
    }

    #[test]
    fn test_app_result_type() {
        // Test that AppResult works correctly
        let success: AppResult<i32> = Ok(42);
        assert!(success.is_ok());
        if let Ok(value) = success {
            assert_eq!(value, 42);
        }

        let failure: AppResult<i32> = Err(AppError::UserNotFound);
        assert!(failure.is_err());
    }
}
