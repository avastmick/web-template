// kanbain/server/src/handlers/auth_handler.rs

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::Deserialize;
use std::sync::Arc;
use validator::Validate;

use crate::{
    core::{AppState, build_unified_auth_response, password_utils::verify_password},
    errors::{AppError, AppResult},
};

#[derive(Debug, Deserialize, Validate, Clone)]
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
    let invite = state.invite.get_valid_invite(&payload.email).await?;

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
    match state.user.create_user(&payload).await {
        Ok(created_user) => {
            // Mark invite as used
            // FIXME: should check state and log as info if INVITENOTFOUND, only log error if not
            // that
            if let Err(e) = state.invite.mark_invite_used(&payload.email).await {
                tracing::error!(
                    "Failed to mark invite as used for email {}: {:?}",
                    payload.email,
                    e
                );
                // We don't fail the registration since the user is already created
            }

            // Generate JWT token for immediate login (matches OAuth behavior)
            let token = state
                .auth
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
    let user = match state.user.find_by_email(&payload.email).await {
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
        .auth
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::create_test_app_state;
    use sqlx::SqlitePool;
    use uuid::Uuid;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("Failed to create test database");

        // Run migrations to ensure test database matches production schema
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");

        pool
    }

    #[tokio::test]
    async fn test_register_user_success() {
        let pool = setup_test_db().await;
        let state = create_test_app_state(&pool);

        let payload = RegisterUserPayload {
            email: format!("test_{}@example.com", Uuid::new_v4()),
            password: "strongPassword123!".to_string(),
        };

        let result = register_user_handler(State(state), Json(payload.clone())).await;
        assert!(result.is_ok());

        match result {
            Ok(response) => {
                // Since the handler returns impl IntoResponse, we need to test it differently
                // We know it returns (StatusCode, Json<UnifiedAuthResponse>)
                // But we can't destructure it directly in tests
                // For now, just verify the request succeeded
                let _ = response;
            }
            Err(e) => panic!("Failed to register user: {e:?}"),
        }
    }

    #[tokio::test]
    async fn test_register_user_with_invite() {
        let pool = setup_test_db().await;
        let state = create_test_app_state(&pool);

        let email = format!("invited_{}@example.com", Uuid::new_v4());

        // Create an invite first
        state
            .invite
            .create_invite(&email, Some("Test invite".to_string()), None)
            .await
            .expect("Failed to create invite");

        let payload = RegisterUserPayload {
            email: email.clone(),
            password: "strongPassword123!".to_string(),
        };

        let result = register_user_handler(State(state), Json(payload)).await;
        assert!(result.is_ok());

        assert!(result.is_ok(), "Failed to register user with invite");
    }

    #[tokio::test]
    async fn test_register_user_invalid_email() {
        let pool = setup_test_db().await;
        let state = create_test_app_state(&pool);

        let payload = RegisterUserPayload {
            email: "not-an-email".to_string(),
            password: "strongPassword123!".to_string(),
        };

        let result = register_user_handler(State(state), Json(payload)).await;
        assert!(result.is_err());

        match result {
            Err(AppError::ValidationError(msg)) => {
                assert!(msg.contains("Email must be a valid email address"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[tokio::test]
    async fn test_register_user_short_password() {
        let pool = setup_test_db().await;
        let state = create_test_app_state(&pool);

        let payload = RegisterUserPayload {
            email: format!("test_{}@example.com", Uuid::new_v4()),
            password: "short".to_string(),
        };

        let result = register_user_handler(State(state), Json(payload)).await;
        assert!(result.is_err());

        match result {
            Err(AppError::ValidationError(msg)) => {
                assert!(msg.contains("Password must be at least 12 characters long"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[tokio::test]
    async fn test_register_user_duplicate_email() {
        let pool = setup_test_db().await;
        let state = create_test_app_state(&pool);

        let email = format!("duplicate_{}@example.com", Uuid::new_v4());
        let payload = RegisterUserPayload {
            email: email.clone(),
            password: "strongPassword123!".to_string(),
        };

        // Register first user
        let result1 = register_user_handler(State(state.clone()), Json(payload.clone())).await;
        assert!(result1.is_ok());

        // Try to register with same email
        let result2 = register_user_handler(State(state), Json(payload)).await;
        assert!(result2.is_err());

        match result2 {
            Err(AppError::UserAlreadyExists { email: err_email }) => {
                assert_eq!(err_email, email);
            }
            _ => panic!("Expected UserAlreadyExists error"),
        }
    }

    #[tokio::test]
    async fn test_login_user_success() {
        let pool = setup_test_db().await;
        let state = create_test_app_state(&pool);

        let email = format!("login_test_{}@example.com", Uuid::new_v4());
        let password = "strongPassword123!";

        // Register user first
        let register_payload = RegisterUserPayload {
            email: email.clone(),
            password: password.to_string(),
        };
        register_user_handler(State(state.clone()), Json(register_payload))
            .await
            .expect("Failed to register user");

        // Now login
        let login_payload = LoginUserPayload {
            email: email.clone(),
            password: password.to_string(),
        };

        let result = login_user_handler(State(state), Json(login_payload)).await;
        assert!(result.is_ok());

        assert!(result.is_ok(), "Failed to login user");
    }

    #[tokio::test]
    async fn test_login_user_wrong_password() {
        let pool = setup_test_db().await;
        let state = create_test_app_state(&pool);

        let email = format!("wrong_pwd_{}@example.com", Uuid::new_v4());
        let password = "strongPassword123!";

        // Register user first
        let register_payload = RegisterUserPayload {
            email: email.clone(),
            password: password.to_string(),
        };
        register_user_handler(State(state.clone()), Json(register_payload))
            .await
            .expect("Failed to register user");

        // Try to login with wrong password
        let login_payload = LoginUserPayload {
            email,
            password: "wrongPassword123!".to_string(),
        };

        let result = login_user_handler(State(state), Json(login_payload)).await;
        assert!(result.is_err());

        match result {
            Err(AppError::InvalidCredentials) => {}
            _ => panic!("Expected InvalidCredentials error"),
        }
    }

    #[tokio::test]
    async fn test_login_user_non_existent() {
        let pool = setup_test_db().await;
        let state = create_test_app_state(&pool);

        let login_payload = LoginUserPayload {
            email: "nonexistent@example.com".to_string(),
            password: "somePassword123!".to_string(),
        };

        let result = login_user_handler(State(state), Json(login_payload)).await;
        assert!(result.is_err());

        match result {
            Err(AppError::InvalidCredentials) => {}
            _ => panic!("Expected InvalidCredentials error"),
        }
    }

    #[tokio::test]
    async fn test_login_user_invalid_email() {
        let pool = setup_test_db().await;
        let state = create_test_app_state(&pool);

        let login_payload = LoginUserPayload {
            email: "not-an-email".to_string(),
            password: "somePassword123!".to_string(),
        };

        let result = login_user_handler(State(state), Json(login_payload)).await;
        assert!(result.is_err());

        match result {
            Err(AppError::ValidationError(msg)) => {
                assert!(msg.contains("Email must be a valid email address"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[tokio::test]
    async fn test_login_user_empty_password() {
        let pool = setup_test_db().await;
        let state = create_test_app_state(&pool);

        let login_payload = LoginUserPayload {
            email: "test@example.com".to_string(),
            password: String::new(),
        };

        let result = login_user_handler(State(state), Json(login_payload)).await;
        assert!(result.is_err());

        match result {
            Err(AppError::ValidationError(msg)) => {
                assert!(msg.contains("Password is required"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }

    #[tokio::test]
    async fn test_register_and_login_flow() {
        let pool = setup_test_db().await;
        let state = create_test_app_state(&pool);

        let email = format!("flow_test_{}@example.com", Uuid::new_v4());
        let password = "strongPassword123!";

        // Register
        let register_payload = RegisterUserPayload {
            email: email.clone(),
            password: password.to_string(),
        };
        register_user_handler(State(state.clone()), Json(register_payload))
            .await
            .expect("Failed to register user");

        // Login
        let login_payload = LoginUserPayload {
            email: email.clone(),
            password: password.to_string(),
        };
        let login_result = login_user_handler(State(state), Json(login_payload)).await;
        assert!(login_result.is_ok(), "Failed to login user");

        // Both register and login should succeed
        // We can't easily test the response details with impl IntoResponse
    }
}
