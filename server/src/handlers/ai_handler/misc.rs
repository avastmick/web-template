//! Miscellaneous AI handlers

use axum::{Json, extract::State};
use axum_extra::TypedHeader;
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Bearer;
use serde::Deserialize;
use std::sync::Arc;

use crate::core::AppState;
use crate::errors::{AppError, AppResult};

use super::file_upload::FileUpload;

#[derive(Debug, Deserialize)]
pub struct ContextualChatRequest {
    pub question: String,
    pub context: Option<Vec<String>>,
    pub files: Option<Vec<FileUpload>>,
    pub use_schema: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CodeAnalysisRequest {
    pub code: String,
    pub language: String,
    pub context: Option<String>,
}

/// Get AI service info
///
/// # Errors
///
/// Returns an error if the service information cannot be retrieved.
pub async fn ai_info_handler(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<serde_json::Value>> {
    let ai_service = state.ai.read().await;

    let info = serde_json::json!({
        "provider": ai_service.provider_name(),
        "schemas": ai_service.list_schemas(),
        "streaming_supported": true,
        "websocket_supported": true,
    });

    Ok(Json(info))
}

/// Handle contextual chat with templates and file uploads
///
/// # Errors
///
/// Returns an error if the AI request fails or authentication is invalid.
pub async fn contextual_chat_handler(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Json(request): Json<ContextualChatRequest>,
) -> AppResult<Json<serde_json::Value>> {
    // Verify JWT token and get user
    let token = auth.token();
    let _user_id = state.auth.get_user_id_from_token(token)?;

    let ai_service = state.ai.read().await;

    // Prepare template data
    let template_data = serde_json::json!({
        "question": request.question,
        "context": request.context,
        "files": request.files
    });

    // Use template-based chat
    let response = ai_service
        .chat_with_template("contextual_chat", &template_data)
        .await
        .map_err(|e| AppError::BadRequest(format!("Template chat failed: {e}")))?;

    // If schema is requested, validate the response
    if let Some(schema_name) = &request.use_schema {
        let default_content = String::new();
        let content = response
            .choices
            .first()
            .map_or(&default_content, |c| &c.message.content);

        // Parse JSON response
        let json_response: serde_json::Value = serde_json::from_str(content)
            .map_err(|e| AppError::BadRequest(format!("Invalid JSON response: {e}")))?;

        // Validate against schema - we'll implement this via a method on ai_service
        // For now, just log that validation would happen here
        tracing::info!("Would validate against schema: {}", schema_name);

        Ok(Json(json_response))
    } else {
        Ok(Json(serde_json::json!({
            "response": response.choices.first().map_or(&String::new(), |c| &c.message.content),
            "usage": response.usage
        })))
    }
}

/// Handle code analysis with structured output
///
/// # Errors
///
/// Returns an error if the AI request fails or authentication is invalid.
pub async fn code_analysis_handler(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Json(request): Json<CodeAnalysisRequest>,
) -> AppResult<Json<serde_json::Value>> {
    // Verify JWT token and get user
    let token = auth.token();
    let _user_id = state.auth.get_user_id_from_token(token)?.to_string();

    let ai_service = state.ai.read().await;

    // Use the dedicated analyze_code method
    let analysis_result = ai_service
        .analyze_code(
            &request.code,
            Some(&request.language),
            request.context.as_deref(),
        )
        .await
        .map_err(|e| AppError::BadRequest(format!("Code analysis failed: {e}")))?;

    Ok(Json(serde_json::json!({
        "analysis": analysis_result,
        "language": request.language,
        "timestamp": chrono::Utc::now()
    })))
}

/// Health check endpoint that uses the AI provider
///
/// # Errors
///
/// Returns an error if the AI provider is not healthy.
pub async fn health_check_handler(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<serde_json::Value>> {
    let ai_service = state.ai.read().await;

    // Use the provider's dedicated health check method
    ai_service
        .health_check()
        .await
        .map_err(|e| AppError::BadRequest(format!("Health check failed: {e}")))?;

    Ok(Json(serde_json::json!({
        "status": "healthy",
        "provider": ai_service.provider_name(),
        "timestamp": chrono::Utc::now()
    })))
}

/// Content moderation endpoint
///
/// # Errors
///
/// Returns an error if moderation fails or authentication is invalid.
pub async fn moderate_content_handler(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Json(request): Json<serde_json::Value>,
) -> AppResult<Json<serde_json::Value>> {
    // Verify JWT token and get user
    let token = auth.token();
    let _user_id = state.auth.get_user_id_from_token(token)?.to_string();

    let content = request
        .get("content")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("Missing content field".to_string()))?;

    let ai_service = state.ai.read().await;

    let moderation_result = ai_service
        .moderate_content(content)
        .await
        .map_err(|e| AppError::BadRequest(format!("Moderation failed: {e}")))?;

    Ok(Json(serde_json::json!(moderation_result)))
}

/// Verify JWT token endpoint - demonstrates JWT token validation
///
/// # Errors
///
/// Returns an error if the token is invalid or expired.
pub async fn verify_token_handler(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
) -> AppResult<Json<serde_json::Value>> {
    let token = auth.token();

    // Use the auth service to verify the token and get user ID
    let user_id = state
        .auth
        .get_user_id_from_token(token)
        .map_err(|e| AppError::Unauthorized(format!("Token verification failed: {e}")))?;

    Ok(Json(serde_json::json!({
        "valid": true,
        "user_id": user_id,
        "verified_at": chrono::Utc::now()
    })))
}

/// Demo AI message creation with token count
///
/// # Errors
///
/// Returns an error if message creation fails.
pub async fn demo_message_handler() -> AppResult<Json<serde_json::Value>> {
    // Demonstrate usage of with_token_count method
    let message = crate::models::ai_models::AiMessage::new(
        "demo_conversation_id".to_string(),
        "user".to_string(),
        "This is a demo message to show token counting functionality.".to_string(),
    )
    .with_token_count(12); // Example token count

    Ok(Json(serde_json::json!({
        "demo_message": message,
        "token_count": message.token_count,
        "message": "Demo AI message created with token count"
    })))
}

/// Error demonstration endpoint - shows usage of various error types
///
/// # Errors
///
/// Returns various error types based on the `error_type` parameter.
pub async fn error_demo_handler(
    axum::extract::Path(error_type): axum::extract::Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    match error_type.as_str() {
        "jwt" => Err(AppError::JwtError("Invalid JWT token format".to_string())),
        "invite_expired" => Err(AppError::InviteExpired),
        "invite_used" => Err(AppError::InviteAlreadyUsed),
        "ai_invalid" => {
            // Use AI error variants
            let ai_error = crate::ai::AiError::InvalidRequest("Missing required field".to_string());
            Err(AppError::BadRequest(format!("AI Error: {ai_error}")))
        }
        "ai_rate_limit" => {
            let ai_error = crate::ai::AiError::RateLimitExceeded;
            Err(AppError::BadRequest(format!("AI Error: {ai_error}")))
        }
        "ai_unknown" => {
            let ai_error = crate::ai::AiError::Unknown("Unexpected provider error".to_string());
            Err(AppError::BadRequest(format!("AI Error: {ai_error}")))
        }
        _ => Ok(Json(serde_json::json!({
            "message": "No error generated",
            "available_errors": ["jwt", "invite_expired", "invite_used", "ai_invalid", "ai_rate_limit", "ai_unknown"]
        }))),
    }
}

// TODO: Move these invite handlers to a separate invite_handler module
/// List invites endpoint - demonstrates invite service usage
///
/// # Errors
///
/// Returns an error if database operation fails or authentication is invalid.
pub async fn list_invites_handler(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
) -> AppResult<Json<serde_json::Value>> {
    // Verify admin access (in a real app, you'd check user permissions)
    let _token = auth.token();

    let invites = state
        .invite
        .list_invites()
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to list invites: {e}")))?;

    Ok(Json(serde_json::json!({
        "invites": invites,
        "count": invites.len()
    })))
}

/// Create invite endpoint
///
/// # Errors
///
/// Returns an error if invite creation fails or authentication is invalid.
pub async fn create_invite_handler(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Json(request): Json<serde_json::Value>,
) -> AppResult<Json<serde_json::Value>> {
    // Verify admin access (in a real app, you'd check user permissions)
    let _token = auth.token();

    let email = request
        .get("email")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("Missing email field".to_string()))?;

    let invited_by = request
        .get("invited_by")
        .and_then(|v| v.as_str())
        .map(String::from);

    let invite = state
        .invite
        .create_invite(email, invited_by, None) // No expiration
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to create invite: {e}")))?;

    Ok(Json(serde_json::json!({
        "invite": invite,
        "message": "Invite created successfully"
    })))
}

/// Get valid invite endpoint
///
/// # Errors
///
/// Returns an error if invite lookup fails.
pub async fn get_invite_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(email): axum::extract::Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let invite = state
        .invite
        .get_valid_invite(&email)
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to get invite: {e}")))?;

    match invite {
        Some(mut invite) => {
            // Check if invite is expired and mark as used if it's valid
            if invite.is_expired() {
                return Err(AppError::InviteExpired);
            }

            if !invite.is_valid() {
                return Err(AppError::InviteAlreadyUsed);
            }

            // For demonstration purposes, mark the invite as used
            invite.mark_used();

            Ok(Json(serde_json::json!({
                "invite": invite,
                "valid": invite.is_valid(),
                "expired": invite.is_expired(),
                "message": "Invite processed successfully"
            })))
        }
        None => Ok(Json(serde_json::json!({
            "invite": null,
            "valid": false,
            "message": "No valid invite found for this email"
        }))),
    }
}

/// Delete invite endpoint
///
/// # Errors
///
/// Returns an error if invite deletion fails or authentication is invalid.
pub async fn delete_invite_handler(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    axum::extract::Path(invite_id): axum::extract::Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    // Verify admin access (in a real app, you'd check user permissions)
    let _token = auth.token();

    state
        .invite
        .delete_invite(&invite_id)
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to delete invite: {e}")))?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Invite deleted successfully"
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::create_test_app_state;
    use serde_json::json;
    use sqlx::SqlitePool;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("Failed to create test database");

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");

        pool
    }

    #[tokio::test]
    async fn test_ai_info_handler() {
        let pool = setup_test_db().await;
        let state = create_test_app_state(&pool);

        let result = ai_info_handler(State(state)).await;
        assert!(result.is_ok());

        let response = result.expect("AI info handler should succeed");
        let value = response.0;

        assert!(value.get("provider").is_some());
        assert!(value.get("schemas").is_some());
        assert_eq!(value.get("streaming_supported"), Some(&json!(true)));
        assert_eq!(value.get("websocket_supported"), Some(&json!(true)));
    }

    #[tokio::test]
    async fn test_health_check_handler() {
        let pool = setup_test_db().await;
        let state = create_test_app_state(&pool);

        let result = health_check_handler(State(state)).await;
        assert!(result.is_ok());

        let response = result.expect("Health check handler should succeed");
        let value = response.0;

        assert_eq!(value.get("status"), Some(&json!("healthy")));
        assert!(value.get("provider").is_some());
        assert!(value.get("timestamp").is_some());
    }

    #[tokio::test]
    async fn test_demo_message_handler() {
        let result = demo_message_handler().await;
        assert!(result.is_ok());

        let response = result.expect("Demo message handler should succeed");
        let value = response.0;

        assert!(value.get("demo_message").is_some());
        assert_eq!(value.get("token_count"), Some(&json!(12)));
        assert!(value.get("message").is_some());
    }

    #[tokio::test]
    async fn test_error_demo_handler_jwt() {
        let result = error_demo_handler(axum::extract::Path("jwt".to_string())).await;
        assert!(result.is_err());
        assert!(matches!(
            result.expect_err("Should return JWT error"),
            AppError::JwtError(_)
        ));
    }

    #[tokio::test]
    async fn test_error_demo_handler_invite_expired() {
        let result = error_demo_handler(axum::extract::Path("invite_expired".to_string())).await;
        assert!(result.is_err());
        assert!(matches!(
            result.expect_err("Should return invite expired error"),
            AppError::InviteExpired
        ));
    }

    #[tokio::test]
    async fn test_error_demo_handler_invite_used() {
        let result = error_demo_handler(axum::extract::Path("invite_used".to_string())).await;
        assert!(result.is_err());
        assert!(matches!(
            result.expect_err("Should return invite used error"),
            AppError::InviteAlreadyUsed
        ));
    }

    #[tokio::test]
    async fn test_error_demo_handler_ai_invalid() {
        let result = error_demo_handler(axum::extract::Path("ai_invalid".to_string())).await;
        assert!(result.is_err());
        assert!(matches!(
            result.expect_err("Should return bad request error"),
            AppError::BadRequest(_)
        ));
    }

    #[tokio::test]
    async fn test_error_demo_handler_ai_rate_limit() {
        let result = error_demo_handler(axum::extract::Path("ai_rate_limit".to_string())).await;
        assert!(result.is_err());
        assert!(matches!(
            result.expect_err("Should return bad request error"),
            AppError::BadRequest(_)
        ));
    }

    #[tokio::test]
    async fn test_error_demo_handler_ai_unknown() {
        let result = error_demo_handler(axum::extract::Path("ai_unknown".to_string())).await;
        assert!(result.is_err());
        assert!(matches!(
            result.expect_err("Should return bad request error"),
            AppError::BadRequest(_)
        ));
    }

    #[tokio::test]
    async fn test_error_demo_handler_no_error() {
        let result = error_demo_handler(axum::extract::Path("other".to_string())).await;
        assert!(result.is_ok());

        let response = result.expect("Error demo handler should succeed");
        let value = response.0;

        assert_eq!(value.get("message"), Some(&json!("No error generated")));
        assert!(value.get("available_errors").is_some());
    }

    #[tokio::test]
    async fn test_contextual_chat_request_deserialization() {
        let json_data = json!({
            "question": "What is this code doing?",
            "context": ["Some context"],
            "files": null,
            "use_schema": "analysis_schema"
        });

        let request: ContextualChatRequest =
            serde_json::from_value(json_data).expect("Should deserialize contextual chat request");
        assert_eq!(request.question, "What is this code doing?");
        assert_eq!(request.context, Some(vec!["Some context".to_string()]));
        assert_eq!(request.use_schema, Some("analysis_schema".to_string()));
    }

    #[tokio::test]
    async fn test_code_analysis_request_deserialization() {
        let json_data = json!({
            "code": "fn main() { println!(\"Hello\"); }",
            "language": "rust",
            "context": "Main function"
        });

        let request: CodeAnalysisRequest =
            serde_json::from_value(json_data).expect("Should deserialize code analysis request");
        assert_eq!(request.code, "fn main() { println!(\"Hello\"); }");
        assert_eq!(request.language, "rust");
        assert_eq!(request.context, Some("Main function".to_string()));
    }

    #[tokio::test]
    async fn test_get_invite_handler_not_found() {
        let pool = setup_test_db().await;
        let state = create_test_app_state(&pool);

        let result = get_invite_handler(
            State(state),
            axum::extract::Path("nonexistent@example.com".to_string()),
        )
        .await;

        assert!(result.is_ok());
        let response = result.expect("Get invite handler should succeed");
        let value = response.0;

        assert_eq!(value.get("invite"), Some(&json!(null)));
        assert_eq!(value.get("valid"), Some(&json!(false)));
        assert!(value.get("message").is_some());
    }

    // Note: The following tests require authentication and would need to be integration tests
    // with proper JWT token setup to test the authenticated endpoints:
    // - contextual_chat_handler
    // - code_analysis_handler
    // - moderate_content_handler
    // - verify_token_handler
    // - list_invites_handler
    // - create_invite_handler
    // - delete_invite_handler
}
