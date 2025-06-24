//! Chat-related handlers

use axum::{Json, extract::State};
use axum_extra::TypedHeader;
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Bearer;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::ai::{ChatMessage, ChatRole};
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
    let user_id = extract_user_id_from_auth(auth_header, &state.auth_service)?;

    // Set up conversation
    let (conversation_id, model) = create_conversation(&state, &user_id, &request).await?;

    // Process messages and save to database
    let messages = process_and_save_messages(&state, &request, &conversation_id, &user_id).await?;

    // Get AI response
    let response = get_ai_response(&state, messages, &request, &model).await?;

    // Save AI response and record usage
    save_response_and_usage(&state, &response, &conversation_id, &user_id, &model).await?;

    // Convert to API response
    Ok(Json(convert_to_chat_response(response)))
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
        .ai_data_service
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
                .ai_data_service
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
    model: &str,
) -> AppResult<crate::ai::ChatResponse> {
    let ai_service = state.ai_service.read().await;

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
        let schema_response = ai_service
            .chat_with_schema(messages, schema_name)
            .await
            .map_err(|e| AppError::BadRequest(format!("Schema-based chat failed: {e}")))?;

        Ok(crate::ai::ChatResponse {
            id: uuid::Uuid::new_v4().to_string(),
            model: model.to_string(),
            choices: vec![crate::ai::models::ChatChoice {
                index: 0,
                message: ChatMessage {
                    role: ChatRole::Assistant,
                    content: schema_response.to_string(),
                },
                finish_reason: Some("stop".to_string()),
            }],
            usage: Some(crate::ai::models::TokenUsage {
                prompt_tokens: 100,
                completion_tokens: 200,
                total_tokens: 300,
            }),
            created: chrono::Utc::now().timestamp(),
        })
    } else {
        // Use context and parameters if provided
        let mut enhanced_messages = messages;

        if let Some(context) = &request.context {
            let context_message = ChatMessage {
                role: ChatRole::System,
                content: format!("Additional context: {}", context.join(", ")),
            };
            enhanced_messages.insert(0, context_message);
        }

        ai_service
            .chat(enhanced_messages)
            .await
            .map_err(|e| AppError::BadRequest(format!("AI request failed: {e}")))
    }
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
        .ai_data_service
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
        let total_tokens = usage.prompt_tokens + usage.completion_tokens;
        let estimated_cost_cents = (i64::from(total_tokens) * 3) / 100; // Rough estimate

        let ai_usage = crate::models::ai_models::AiUsage::new(
            user_id.to_string(),
            model.to_string(),
            i64::from(usage.prompt_tokens),
            i64::from(usage.completion_tokens),
        )
        .with_conversation(conversation_id.to_string())
        .with_cost(estimated_cost_cents);

        let conv_id = ai_usage.conversation_id.clone().unwrap_or_default();
        let usage_record = crate::services::ai_data_service::UsageRecord {
            user_id,
            model,
            conversation_id: Some(&conv_id),
            prompt_tokens: i64::from(usage.prompt_tokens),
            completion_tokens: i64::from(usage.completion_tokens),
            request_id: Some(&response.id),
            duration_ms: Some(0),
        };

        state
            .ai_data_service
            .record_usage(usage_record)
            .await
            .map_err(|e| AppError::BadRequest(format!("Failed to record usage: {e}")))?;
    }

    Ok(())
}

/// Convert internal AI response to API response format
fn convert_to_chat_response(response: crate::ai::ChatResponse) -> ChatResponse {
    ChatResponse {
        id: response.id,
        message: MessageOutput {
            role: "assistant".to_string(),
            content: response
                .choices
                .first()
                .map_or(String::new(), |c| c.message.content.clone()),
        },
        usage: response.usage.map(|u| crate::ai::models::TokenUsage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        }),
    }
}
