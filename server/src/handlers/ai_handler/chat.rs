//! Chat-related handlers

use axum::{Json, extract::State};
use axum_extra::TypedHeader;
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Bearer;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::ai::{ChatMessage, ChatRequest as AiChatRequest, ChatRole};
use crate::core::AppState;
use crate::errors::{AppError, AppResult};
use crate::services::AuthService;

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<MessageInput>,
    pub stream: Option<bool>,
    pub model: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub context: Option<Vec<String>>,
    pub use_schema: Option<String>,
    pub template: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MessageInput {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub id: String,
    pub conversation_id: String,
    pub message: MessageOutput,
    pub usage: Option<crate::ai::models::TokenUsage>,
}

#[derive(Debug, Serialize)]
pub struct MessageOutput {
    pub role: String,
    pub content: String,
}

/// Estimate token count for a text string (rough approximation)
/// In production, use a proper tokenizer like tiktoken
fn estimate_token_count(text: &str) -> i64 {
    // Rough estimation: 1 token ≈ 4 characters for English text
    // This is a very rough approximation; real tokenizers are more accurate
    let char_count = text.chars().count();
    // Use div_ceil for proper ceiling division
    i64::try_from(char_count.div_ceil(4)).unwrap_or(0)
}

/// Handle non-streaming chat requests with full context and database persistence
///
/// # Errors
///
/// Returns an error if the AI request fails or authentication is invalid.
pub async fn chat_handler(
    State(state): State<Arc<AppState>>,
    auth_header: Option<TypedHeader<Authorization<Bearer>>>,
    Json(request): Json<ChatRequest>,
) -> AppResult<Json<ChatResponse>> {
    // Check if streaming is requested - redirect to streaming handler
    if request.stream.unwrap_or(false) {
        return Err(AppError::BadRequest(
            "Use /api/ai/chat/stream endpoint for streaming responses".to_string(),
        ));
    }

    // Extract user info from auth
    let user_id = extract_user_id_from_auth(auth_header, &state.auth)?;

    // Set up conversation
    let (conversation_id, model) = create_conversation(&state, &user_id, &request).await?;

    // Process messages and save to database
    let messages = process_and_save_messages(&state, &request, &conversation_id, &user_id).await?;

    // Get AI response
    let response = get_ai_response(&state, messages, &request, &model).await?;

    // Save AI response and record usage
    save_response_and_usage(&state, &response, &conversation_id, &user_id, &model).await?;

    // Convert to API response
    Ok(Json(convert_to_chat_response(response, conversation_id)))
}

/// Extract user ID from authorization header
fn extract_user_id_from_auth(
    auth_header: Option<TypedHeader<Authorization<Bearer>>>,
    auth_service: &Arc<AuthService>,
) -> AppResult<String> {
    let auth = auth_header
        .ok_or_else(|| AppError::Unauthorized("Missing authorization header".to_string()))?;

    let token = auth.0.token();

    // Verify JWT token and extract user ID
    let user_id = auth_service.get_user_id_from_token(token)?;
    Ok(user_id.to_string())
}

/// Create a new conversation in the database
async fn create_conversation(
    state: &Arc<AppState>,
    user_id: &str,
    request: &ChatRequest,
) -> AppResult<(String, String)> {
    let model = request.model.as_deref().unwrap_or("gpt-4").to_string();

    let create_request = crate::models::ai_models::CreateConversationRequest {
        title: Some("Chat Conversation".to_string()),
        model: model.clone(),
        system_prompt: None,
    };

    let conversation_response = state
        .ai_data
        .create_conversation(user_id, create_request)
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to create conversation: {e}")))?;

    Ok((conversation_response.id, model))
}

/// Process request messages and save user messages to database
async fn process_and_save_messages(
    state: &Arc<AppState>,
    request: &ChatRequest,
    conversation_id: &str,
    user_id: &str,
) -> AppResult<Vec<ChatMessage>> {
    let mut messages = Vec::new();

    for msg in &request.messages {
        let role = match msg.role.as_str() {
            "system" => ChatRole::System,
            "assistant" => ChatRole::Assistant,
            _ => ChatRole::User,
        };

        let chat_message = ChatMessage {
            role: role.clone(),
            content: msg.content.clone(),
        };
        messages.push(chat_message);

        // Save user message to database with token count
        if matches!(role, ChatRole::User) {
            let token_count = estimate_token_count(&msg.content);
            let message_request = crate::models::ai_models::CreateMessageRequest {
                role: "user".to_string(),
                content: msg.content.clone(),
            };

            // Note: In real implementation, we'd enhance CreateMessageRequest to include token_count
            // For now, we'll create the message and then update it
            let saved_message = state
                .ai_data
                .add_message(conversation_id, user_id, message_request)
                .await
                .map_err(|e| AppError::BadRequest(format!("Failed to save message: {e}")))?;

            // Log the token count for now (in real implementation, store in DB)
            tracing::info!(
                "Message {} has estimated {} tokens",
                saved_message.id,
                token_count
            );
        }
    }

    Ok(messages)
}

/// Get AI response either with schema or regular chat
async fn get_ai_response(
    state: &Arc<AppState>,
    messages: Vec<ChatMessage>,
    request: &ChatRequest,
    _model: &str,
) -> AppResult<crate::ai::ChatResponse> {
    let ai_service = state.ai.read().await;

    // Use template if specified
    if let Some(template_name) = &request.template {
        let template_data = serde_json::json!({
            "messages": request.messages,
            "context": request.context,
            "temperature": request.temperature,
            "max_tokens": request.max_tokens
        });

        let template_response = ai_service
            .chat_with_template(template_name, &template_data)
            .await
            .map_err(|e| AppError::BadRequest(format!("Template chat failed: {e}")))?;

        return Ok(template_response);
    }

    if let Some(schema_name) = &request.use_schema {
        return ai_service
            .chat_with_schema(messages, schema_name)
            .await
            .map_err(|e| AppError::BadRequest(format!("Schema-based chat failed: {e}")));
    }

    // Use context and parameters if provided
    let mut enhanced_messages = messages;

    if let Some(context) = &request.context {
        let context_message = ChatMessage {
            role: ChatRole::System,
            content: format!("Additional context: {}", context.join(", ")),
        };
        enhanced_messages.insert(0, context_message);
    }

    let chat_request = AiChatRequest::new(enhanced_messages);
    ai_service
        .chat(chat_request)
        .await
        .map_err(|e| AppError::BadRequest(format!("AI request failed: {e}")))
}

/// Save AI response to database and record usage statistics
async fn save_response_and_usage(
    state: &Arc<AppState>,
    response: &crate::ai::ChatResponse,
    conversation_id: &str,
    user_id: &str,
    model: &str,
) -> AppResult<()> {
    // Save AI response to database with token count
    let ai_response_content = response
        .choices
        .first()
        .map_or(String::new(), |c| c.message.content.clone());
    let response_token_count = estimate_token_count(&ai_response_content);

    let message_request = crate::models::ai_models::CreateMessageRequest {
        role: "assistant".to_string(),
        content: ai_response_content,
    };

    let saved_response = state
        .ai_data
        .add_message(conversation_id, user_id, message_request)
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to save AI response: {e}")))?;

    // Log the response token count (in real implementation, store in DB)
    tracing::info!(
        "AI response {} has estimated {} tokens",
        saved_response.id,
        response_token_count
    );

    // Record usage statistics
    if let Some(usage) = &response.usage {
        // Calculate estimated cost (example: $0.03 per 1K tokens for GPT-4)
        let total_tokens = usage.prompt + usage.completion;
        let estimated_cost_cents = (i64::from(total_tokens) * 3) / 100; // Rough estimate

        let ai_usage = crate::models::ai_models::AiUsage::new(
            user_id.to_string(),
            model.to_string(),
            i64::from(usage.prompt),
            i64::from(usage.completion),
        )
        .with_conversation(conversation_id.to_string())
        .with_cost(estimated_cost_cents);

        let conv_id = ai_usage.conversation_id.clone().unwrap_or_default();
        let usage_record = crate::services::ai_data_service::UsageRecord {
            user_id,
            model,
            conversation_id: Some(&conv_id),
            prompt_tokens: i64::from(usage.prompt),
            completion_tokens: i64::from(usage.completion),
            request_id: Some(&response.id),
            duration_ms: Some(0),
        };

        state
            .ai_data
            .record_usage(usage_record)
            .await
            .map_err(|e| AppError::BadRequest(format!("Failed to record usage: {e}")))?;
    }

    Ok(())
}

/// Convert internal AI response to API response format
fn convert_to_chat_response(
    response: crate::ai::ChatResponse,
    conversation_id: String,
) -> ChatResponse {
    ChatResponse {
        id: response.id,
        conversation_id,
        message: MessageOutput {
            role: "assistant".to_string(),
            content: response
                .choices
                .first()
                .map_or(String::new(), |c| c.message.content.clone()),
        },
        usage: response.usage.map(|u| crate::ai::models::TokenUsage {
            prompt: u.prompt,
            completion: u.completion,
            total: u.total,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::models::usage::TokenUsage as AiTokenUsage;
    use crate::ai::models::{ChatChoice, chat::ChatResponse as AiChatResponse};
    use crate::test_helpers::create_test_app_state;
    use sqlx::SqlitePool;

    #[test]
    fn test_estimate_token_count() {
        assert_eq!(estimate_token_count(""), 0);
        assert_eq!(estimate_token_count("test"), 1);
        assert_eq!(estimate_token_count("hello world"), 3);
        assert_eq!(
            estimate_token_count("This is a longer sentence with more tokens"),
            11
        );
        assert_eq!(estimate_token_count("🦀🚀"), 1); // Unicode chars
    }

    #[sqlx::test]
    async fn test_extract_user_id_from_auth_missing_header(pool: SqlitePool) {
        let state = create_test_app_state(&pool);
        let result = extract_user_id_from_auth(None, &state.auth);
        assert!(result.is_err());
        match result.expect_err("Expected an error but got Ok") {
            AppError::Unauthorized(msg) => {
                assert_eq!(msg, "Missing authorization header");
            }
            _ => panic!("Expected Unauthorized error"),
        }
    }

    #[sqlx::test]
    async fn test_extract_user_id_from_auth_invalid_token(pool: SqlitePool) {
        let state = create_test_app_state(&pool);
        let auth_header = TypedHeader(
            Authorization::bearer("invalid_token").expect("Failed to create auth header"),
        );
        let result = extract_user_id_from_auth(Some(auth_header), &state.auth);
        assert!(result.is_err());
    }

    #[sqlx::test]
    async fn test_chat_handler_with_streaming_request(pool: SqlitePool) {
        let state = create_test_app_state(&pool);

        let request = ChatRequest {
            messages: vec![MessageInput {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }],
            stream: Some(true),
            model: None,
            temperature: None,
            max_tokens: None,
            context: None,
            use_schema: None,
            template: None,
        };

        let result = chat_handler(State(state), None, Json(request)).await;

        assert!(result.is_err());
        match result.expect_err("Expected an error but got Ok") {
            AppError::BadRequest(msg) => {
                assert_eq!(
                    msg,
                    "Use /api/ai/chat/stream endpoint for streaming responses"
                );
            }
            _ => panic!("Expected BadRequest error"),
        }
    }

    #[sqlx::test]
    async fn test_chat_handler_unauthorized(pool: SqlitePool) {
        let state = create_test_app_state(&pool);

        let request = ChatRequest {
            messages: vec![MessageInput {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }],
            stream: Some(false),
            model: None,
            temperature: None,
            max_tokens: None,
            context: None,
            use_schema: None,
            template: None,
        };

        let result = chat_handler(State(state), None, Json(request)).await;

        assert!(result.is_err());
        match result.expect_err("Expected an error but got Ok") {
            AppError::Unauthorized(_) => {}
            _ => panic!("Expected Unauthorized error"),
        }
    }

    #[test]
    fn test_convert_to_chat_response() {
        let ai_response = AiChatResponse {
            id: "test-id".to_string(),
            created: 1_234_567_890,
            model: "gpt-4".to_string(),
            choices: vec![ChatChoice {
                index: 0,
                message: ChatMessage {
                    role: ChatRole::Assistant,
                    content: "Hello, how can I help you?".to_string(),
                },
                finish_reason: Some("stop".to_string()),
            }],
            usage: Some(AiTokenUsage {
                prompt: 10,
                completion: 5,
                total: 15,
            }),
            function_call: None,
        };

        let conversation_id = "conv-123".to_string();
        let result = convert_to_chat_response(ai_response, conversation_id.clone());

        assert_eq!(result.id, "test-id");
        assert_eq!(result.conversation_id, conversation_id);
        assert_eq!(result.message.role, "assistant");
        assert_eq!(result.message.content, "Hello, how can I help you?");
        assert!(result.usage.is_some());

        let usage = result.usage.expect("Expected usage but got None");
        assert_eq!(usage.prompt, 10);
        assert_eq!(usage.completion, 5);
        assert_eq!(usage.total, 15);
    }

    #[test]
    fn test_convert_to_chat_response_empty_choices() {
        let ai_response = AiChatResponse {
            id: "test-id".to_string(),
            created: 1_234_567_890,
            model: "gpt-4".to_string(),
            choices: vec![],
            usage: None,
            function_call: None,
        };

        let result = convert_to_chat_response(ai_response, "conv-123".to_string());
        assert_eq!(result.message.content, "");
        assert!(result.usage.is_none());
    }
}
