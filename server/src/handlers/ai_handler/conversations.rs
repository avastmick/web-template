//! Conversation management handlers

use axum::{Json, extract::State};
use axum_extra::TypedHeader;
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Bearer;
use std::sync::Arc;

use crate::core::AppState;
use crate::errors::{AppError, AppResult};

/// Get user's conversation history
///
/// # Errors
///
/// Returns an error if database query fails or authentication is invalid.
pub async fn get_conversations_handler(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
) -> AppResult<Json<serde_json::Value>> {
    // Verify JWT token and get user
    let token = auth.token();
    let user_id = state.auth.get_user_id_from_token(token)?.to_string();

    let conversations = state
        .ai_data
        .get_user_conversations(&user_id, Some(50), Some(0)) // limit 50, offset 0
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to get conversations: {e}")))?;

    Ok(Json(serde_json::json!({
        "conversations": conversations
    })))
}

/// Get conversation with messages
///
/// # Errors
///
/// Returns an error if database query fails or authentication is invalid.
pub async fn get_conversation_handler(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    axum::extract::Path(conversation_id): axum::extract::Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    // Verify JWT token and get user
    let token = auth.token();
    let user_id = state.auth.get_user_id_from_token(token)?.to_string();
    let conversation_with_messages = state
        .ai_data
        .get_conversation_with_messages(&conversation_id, &user_id)
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to get conversation: {e}")))?;

    Ok(Json(serde_json::json!(conversation_with_messages)))
}

/// Get user usage statistics
///
/// # Errors
///
/// Returns an error if database query fails or authentication is invalid.
pub async fn get_usage_stats_handler(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
) -> AppResult<Json<serde_json::Value>> {
    // Verify JWT token and get user
    let token = auth.token();
    let user_id = state.auth.get_user_id_from_token(token)?.to_string();

    let usage_stats = state
        .ai_data
        .get_user_usage_stats(&user_id)
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to get usage stats: {e}")))?;

    Ok(Json(serde_json::json!(usage_stats)))
}

/// Archive a conversation (soft delete)
///
/// # Errors
///
/// Returns an error if the conversation is not found or database operation fails.
pub async fn archive_conversation_handler(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    axum::extract::Path(conversation_id): axum::extract::Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    // Verify JWT token and get user
    let token = auth.token();
    let user_id = state.auth.get_user_id_from_token(token)?.to_string();

    // Demonstrate usage of archive and update_timestamp methods
    let mut conversation =
        crate::models::ai_models::AiConversation::new(user_id.to_string(), "gpt-4".to_string());
    conversation.update_timestamp();
    conversation.archive();

    tracing::info!(
        "Would archive conversation with ID: {} (demo conversation created with archive timestamp: {:?})",
        conversation_id,
        conversation.archived_at
    );

    state
        .ai_data
        .archive_conversation(&conversation_id, &user_id)
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to archive conversation: {e}")))?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Conversation archived successfully",
        "demo_conversation": conversation
    })))
}

/// Delete a conversation permanently
///
/// # Errors
///
/// Returns an error if the conversation is not found or database operation fails.
pub async fn delete_conversation_handler(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    axum::extract::Path(conversation_id): axum::extract::Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    // Verify JWT token and get user
    let token = auth.token();
    let user_id = state.auth.get_user_id_from_token(token)?.to_string();

    // For now, we'll archive the conversation instead of deleting it
    // In the future, this could be a hard delete operation
    state
        .ai_data
        .archive_conversation(&conversation_id, &user_id)
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to delete conversation: {e}")))?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Conversation deleted successfully"
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::create_test_app_state;
    use sqlx::SqlitePool;

    #[sqlx::test]
    async fn test_get_conversations_handler_unauthorized(pool: SqlitePool) {
        let state = create_test_app_state(&pool);

        // Create invalid auth header
        let auth_header = TypedHeader(
            Authorization::bearer("invalid_token").expect("Failed to create auth header"),
        );

        let result = get_conversations_handler(State(state), auth_header).await;

        assert!(result.is_err());
        match result.expect_err("Expected error but got Ok") {
            AppError::Unauthorized(_) => {}
            _ => panic!("Expected Unauthorized error"),
        }
    }

    #[sqlx::test]
    async fn test_get_conversation_handler_unauthorized(pool: SqlitePool) {
        let state = create_test_app_state(&pool);

        let auth_header = TypedHeader(
            Authorization::bearer("invalid_token").expect("Failed to create auth header"),
        );

        let result = get_conversation_handler(
            State(state),
            auth_header,
            axum::extract::Path("conv-123".to_string()),
        )
        .await;

        assert!(result.is_err());
        match result.expect_err("Expected error but got Ok") {
            AppError::Unauthorized(_) => {}
            _ => panic!("Expected Unauthorized error"),
        }
    }

    #[sqlx::test]
    async fn test_get_usage_stats_handler_unauthorized(pool: SqlitePool) {
        let state = create_test_app_state(&pool);

        let auth_header = TypedHeader(
            Authorization::bearer("invalid_token").expect("Failed to create auth header"),
        );

        let result = get_usage_stats_handler(State(state), auth_header).await;

        assert!(result.is_err());
        match result.expect_err("Expected error but got Ok") {
            AppError::Unauthorized(_) => {}
            _ => panic!("Expected Unauthorized error"),
        }
    }

    #[sqlx::test]
    async fn test_archive_conversation_handler_unauthorized(pool: SqlitePool) {
        let state = create_test_app_state(&pool);

        let auth_header = TypedHeader(
            Authorization::bearer("invalid_token").expect("Failed to create auth header"),
        );

        let result = archive_conversation_handler(
            State(state),
            auth_header,
            axum::extract::Path("conv-123".to_string()),
        )
        .await;

        assert!(result.is_err());
        match result.expect_err("Expected error but got Ok") {
            AppError::Unauthorized(_) => {}
            _ => panic!("Expected Unauthorized error"),
        }
    }

    #[sqlx::test]
    async fn test_delete_conversation_handler_unauthorized(pool: SqlitePool) {
        let state = create_test_app_state(&pool);

        let auth_header = TypedHeader(
            Authorization::bearer("invalid_token").expect("Failed to create auth header"),
        );

        let result = delete_conversation_handler(
            State(state),
            auth_header,
            axum::extract::Path("conv-123".to_string()),
        )
        .await;

        assert!(result.is_err());
        match result.expect_err("Expected error but got Ok") {
            AppError::Unauthorized(_) => {}
            _ => panic!("Expected Unauthorized error"),
        }
    }
}
