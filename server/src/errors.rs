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
