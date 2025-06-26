// web-template/server/src/handlers/oauth_handler.rs

use axum::{
    extract::{Query, State},
    response::Redirect,
};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use uuid::Uuid;

use crate::{
    core::AppState,
    errors::AppError,
    models::{
        User,
        oauth::{OAuthProvider, OAuthUserInfo},
    },
    services::OAuthService,
};

/// Request payload for OAuth login initiation
#[derive(Debug, Deserialize)]
pub struct OAuthInitRequest {
    /// Optional state parameter for CSRF protection
    pub state: Option<String>,
}

/// Query parameters for OAuth callback
#[derive(Debug, Deserialize)]
pub struct OAuthCallbackQuery {
    /// Authorization code from OAuth provider
    pub code: String,
    /// State parameter for CSRF protection
    pub state: Option<String>,
    /// Error parameter if OAuth failed
    pub error: Option<String>,
}

/// Response for successful OAuth login
#[derive(Debug, Serialize)]
pub struct OAuthLoginResponse {
    pub token: String,
    pub user: User,
    pub is_new_user: bool,
}

/// Extended application state for OAuth handlers
#[derive(Clone)]
pub struct OAuthAppState {
    pub app_state: std::sync::Arc<AppState>,
    pub oauth_service: std::sync::Arc<OAuthService>,
}

/// Initiate Google OAuth login flow
///
/// Redirects the user to Google's OAuth authorization URL
///
/// # Errors
///
/// Returns an error if OAuth configuration is invalid or redirect URL construction fails
#[instrument(skip(state), err(Debug))]
pub async fn google_login_init(
    State(state): State<OAuthAppState>,
    Query(params): Query<OAuthInitRequest>,
) -> Result<Redirect, AppError> {
    // Generate a random state for CSRF protection if not provided
    let csrf_state = params.state.unwrap_or_else(|| Uuid::new_v4().to_string());

    // Store the state for later validation
    state
        .oauth_service
        .store_oauth_state(
            &csrf_state,
            crate::models::oauth::OAuthProvider::Google,
            None,
        )
        .await?;

    // Get Google OAuth authorization URL
    let auth_url = state.oauth_service.get_google_auth_url(&csrf_state);

    tracing::info!("Initiating Google OAuth login with state: {}", csrf_state);

    Ok(Redirect::permanent(&auth_url))
}

/// Initiate GitHub OAuth login flow
///
/// Redirects the user to GitHub's OAuth authorization URL
///
/// # Errors
///
/// Returns an error if OAuth configuration is invalid or redirect URL construction fails
#[instrument(skip(state), err(Debug))]
pub async fn github_login_init(
    State(state): State<OAuthAppState>,
    Query(params): Query<OAuthInitRequest>,
) -> Result<Redirect, AppError> {
    // Generate a random state for CSRF protection if not provided
    let csrf_state = params.state.unwrap_or_else(|| Uuid::new_v4().to_string());

    // Store the state for later validation
    state
        .oauth_service
        .store_oauth_state(
            &csrf_state,
            crate::models::oauth::OAuthProvider::GitHub,
            None,
        )
        .await?;

    // Get GitHub OAuth authorization URL
    let auth_url = state.oauth_service.get_github_auth_url(&csrf_state);

    tracing::info!("Initiating GitHub OAuth login with state: {}", csrf_state);

    Ok(Redirect::permanent(&auth_url))
}

/// Handle Google OAuth callback
///
/// Exchanges the authorization code for user information and either creates a new user
/// or logs in an existing user
///
/// # Errors
///
/// Returns an error if OAuth exchange fails, user info retrieval fails, or JWT generation fails
#[instrument(skip(state), fields(code = %params.code), err(Debug))]
pub async fn google_oauth_callback(
    State(state): State<OAuthAppState>,
    Query(params): Query<OAuthCallbackQuery>,
) -> Result<Redirect, AppError> {
    handle_oauth_callback(state, params, OAuthProvider::Google).await
}

/// Handle GitHub OAuth callback
///
/// Exchanges the authorization code for user information and either creates a new user
/// or logs in an existing user
///
/// # Errors
///
/// Returns an error if OAuth exchange fails, user info retrieval fails, or JWT generation fails
#[instrument(skip(state), fields(code = %params.code), err(Debug))]
pub async fn github_oauth_callback(
    State(state): State<OAuthAppState>,
    Query(params): Query<OAuthCallbackQuery>,
) -> Result<Redirect, AppError> {
    handle_oauth_callback(state, params, OAuthProvider::GitHub).await
}

/// Generic OAuth callback handler
async fn handle_oauth_callback(
    state: OAuthAppState,
    params: OAuthCallbackQuery,
    provider: OAuthProvider,
) -> Result<Redirect, AppError> {
    // Check for OAuth error
    if let Some(error) = params.error {
        return Ok(redirect_with_error(&state, &error));
    }

    // Validate state parameter for CSRF protection
    if let Some(state_param) = &params.state {
        state
            .oauth_service
            .validate_oauth_state(state_param, provider.clone())
            .await
            .map_err(|_| AppError::Unauthorized("Invalid or expired OAuth state".to_string()))?;

        tracing::info!("OAuth state validated successfully");
    } else {
        // State is required for security
        return Err(AppError::Unauthorized(
            "Missing OAuth state parameter".to_string(),
        ));
    }

    // Exchange authorization code for user info
    let Ok(oauth_user_info) = exchange_oauth_code(&state, &params.code, provider).await else {
        return Ok(redirect_with_error(&state, "oauth_exchange_failed"));
    };

    // Get or create user (this will handle invite validation for new users only)
    let (user, is_new_user) = match get_or_create_user(&state, &oauth_user_info).await {
        Ok(result) => result,
        Err(error_code) => return Ok(redirect_with_error(&state, &error_code)),
    };

    // Generate JWT token
    let Ok(token) = generate_jwt_token(&state, &user) else {
        return Ok(redirect_with_error(&state, "token_generation_failed"));
    };

    tracing::info!(
        "OAuth login successful for user: {} (new_user: {})",
        user.email,
        is_new_user
    );

    // Redirect to client with success data
    Ok(redirect_with_success(&state, &token, &user, is_new_user))
}

/// Redirect to client with error
fn redirect_with_error(state: &OAuthAppState, error: &str) -> Redirect {
    tracing::warn!("OAuth callback error: {}", error);
    let client_url = state.oauth_service.get_client_url();
    let redirect_url = format!(
        "{}/auth/oauth/callback?error={}",
        client_url,
        urlencoding::encode(error)
    );
    Redirect::permanent(&redirect_url)
}

/// Redirect to client with success data
fn redirect_with_success(
    state: &OAuthAppState,
    token: &str,
    user: &User,
    is_new_user: bool,
) -> Redirect {
    let client_url = state.oauth_service.get_client_url();
    let redirect_url = format!(
        "{}/auth/oauth/callback?token={}&user_id={}&email={}&is_new_user={}",
        client_url,
        urlencoding::encode(token),
        user.id,
        urlencoding::encode(&user.email),
        is_new_user
    );
    Redirect::permanent(&redirect_url)
}

/// Exchange OAuth authorization code for user info
async fn exchange_oauth_code(
    state: &OAuthAppState,
    code: &str,
    provider: OAuthProvider,
) -> Result<OAuthUserInfo, AppError> {
    let oauth_user_info = match provider {
        OAuthProvider::Google => state.oauth_service.handle_google_callback(code).await?,
        OAuthProvider::GitHub => state.oauth_service.handle_github_callback(code).await?,
    };

    tracing::info!(
        "OAuth callback successful for user: {} (provider: {})",
        oauth_user_info.email,
        provider
    );

    Ok(oauth_user_info)
}

/// Validate that user has an invite
async fn validate_user_invite(state: &OAuthAppState, email: &str) -> Result<(), String> {
    let has_invite = state
        .app_state
        .invite_service
        .check_invite_exists(email)
        .await
        .map_err(|e| {
            tracing::error!("Failed to check invite: {:?}", e);
            "invite_check_failed"
        })?;

    if !has_invite {
        tracing::warn!("OAuth user {} does not have an invite", email);
        return Err("no_invite".to_string());
    }

    Ok(())
}

/// Get existing user or create new one from OAuth info
async fn get_or_create_user(
    state: &OAuthAppState,
    oauth_user_info: &OAuthUserInfo,
) -> Result<(User, bool), String> {
    match state
        .app_state
        .user_service
        .find_by_email(&oauth_user_info.email)
        .await
    {
        Ok(existing_user) => {
            tracing::info!(
                "Found existing user for OAuth login: {}",
                oauth_user_info.email
            );
            Ok((existing_user, false))
        }
        Err(AppError::UserNotFound) => {
            tracing::info!(
                "Creating new user from OAuth info: {}",
                oauth_user_info.email
            );

            // For new users, validate they have an invite
            validate_user_invite(state, &oauth_user_info.email).await?;

            let new_user = create_user_from_oauth(state, oauth_user_info)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to create user from OAuth: {:?}", e);
                    "user_creation_failed"
                })?;

            // Check if there is an invite
            // FIXME: it is not an error if a user does not have an invite
            // If a the call returns a INVITENOTFOUND error, then they have to pay
            // else log as info that they are an invited user (add email)
            if let Err(e) = state
                .app_state
                .invite_service
                .mark_invite_used(&oauth_user_info.email)
                .await
            {
                tracing::error!("Failed to mark invite as used: {:?}", e);
                // Continue anyway since user was created successfully
            }

            Ok((new_user, true))
        }
        Err(e) => {
            tracing::error!("Error finding user: {:?}", e);
            Err("user_lookup_failed".to_string())
        }
    }
}

/// Generate JWT token for user
fn generate_jwt_token(state: &OAuthAppState, user: &User) -> Result<String, AppError> {
    state
        .app_state
        .auth_service
        .generate_token(user.id, &user.email)
}

/// Create a new user from OAuth user information
async fn create_user_from_oauth(
    state: &OAuthAppState,
    oauth_info: &OAuthUserInfo,
) -> Result<User, AppError> {
    use chrono::Utc;
    use sqlx::query;

    let user_id = Uuid::new_v4();
    let current_time = Utc::now();
    let provider = oauth_info.provider.to_string();

    // Insert user with OAuth provider information
    // Note: Using a dummy password since this is OAuth user
    let dummy_password = format!("oauth_user_{}", Uuid::new_v4());
    let hashed_dummy_password = crate::core::password_utils::hash_password(&dummy_password)
        .map_err(|e| {
            tracing::error!("Failed to hash dummy password for OAuth user: {}", e);
            AppError::PasswordUtilError(e)
        })?;

    let user_id_str = user_id.to_string();

    query!(
        r#"
        INSERT INTO users (id, email, hashed_password, provider, provider_user_id, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
        user_id_str,
        oauth_info.email,
        hashed_dummy_password,
        provider,
        oauth_info.id,
        current_time,
        current_time
    )
    .execute(&state.app_state.user_service.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to insert OAuth user {}: {}", oauth_info.email, e);
        AppError::SqlxError(e)
    })?;

    // Fetch the created user
    state
        .app_state
        .user_service
        .find_by_email(&oauth_info.email)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth_init_request_parsing() {
        let json = r#"{"state": "test_state"}"#;
        let req: OAuthInitRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.state, Some("test_state".to_string()));
    }

    #[test]
    fn test_oauth_callback_query_parsing() {
        // Test basic query parsing - this would normally be handled by axum
        let query = OAuthCallbackQuery {
            code: "test_code".to_string(),
            state: Some("test_state".to_string()),
            error: None,
        };
        assert_eq!(query.code, "test_code");
        assert_eq!(query.state, Some("test_state".to_string()));
    }
}
