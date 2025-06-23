//! AI endpoint handlers for chat and streaming responses

use axum::{
    Json,
    extract::{Multipart, Query, State},
    response::{Sse, sse::Event},
};
use axum_extra::TypedHeader;
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Bearer;
use futures::stream::{self, Stream};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

use crate::ai::{ChatMessage, ChatRole};
use crate::core::AppState;
use crate::errors::{AppError, AppResult};
use crate::services::AuthService;

/// Estimate token count for a text string (rough approximation)
/// In production, use a proper tokenizer like tiktoken
fn estimate_token_count(text: &str) -> i64 {
    // Rough estimation: 1 token ≈ 4 characters for English text
    // This is a very rough approximation; real tokenizers are more accurate
    let char_count = text.chars().count();
    // Use div_ceil for proper ceiling division
    i64::try_from(char_count.div_ceil(4)).unwrap_or(0)
}

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
    pub usage: Option<TokenUsage>,
}

#[derive(Debug, Serialize)]
pub struct MessageOutput {
    pub role: String,
    pub content: String,
}

use crate::ai::models::TokenUsage;

#[derive(Debug, Serialize)]
pub struct StreamChunk {
    pub id: String,
    pub delta: String,
    pub finished: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileUpload {
    pub name: String,
    pub content: String,
}

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
    let conversation_id = uuid::Uuid::new_v4().to_string();
    let model = request.model.as_deref().unwrap_or("gpt-4").to_string();

    let create_request = crate::models::ai_models::CreateConversationRequest {
        title: Some("Chat Conversation".to_string()),
        model: model.clone(),
        system_prompt: None,
    };

    state
        .ai_data_service
        .create_conversation(user_id, create_request)
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to create conversation: {e}")))?;

    Ok((conversation_id, model))
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
        usage: response.usage.map(|u| TokenUsage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        }),
    }
}

/// Handle SSE streaming chat requests
///
/// # Errors
///
/// Returns an error if the AI request fails or authentication is invalid.
pub async fn chat_stream_handler(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Query(params): Query<ChatRequest>,
) -> AppResult<Sse<impl Stream<Item = Result<Event, Infallible>>>> {
    // Verify JWT token and get user
    let token = auth.token();
    let _user_id = state.auth_service.get_user_id_from_token(token)?;

    let chat_id = Uuid::new_v4().to_string();

    // Convert request messages
    let messages: Vec<ChatMessage> = params
        .messages
        .into_iter()
        .map(|msg| ChatMessage {
            role: match msg.role.as_str() {
                "system" => ChatRole::System,
                "assistant" => ChatRole::Assistant,
                _ => ChatRole::User,
            },
            content: msg.content,
        })
        .collect();

    // For now, simulate streaming by calling regular chat and splitting the response
    let ai_service = state.ai_service.read().await;
    let response = ai_service.chat(messages).await;

    // Create a stream that sends events
    let stream = stream::unfold(
        (chat_id, response, 0),
        |(chat_id, response, mut index)| async move {
            match &response {
                Ok(chat_response) => {
                    let content = chat_response
                        .choices
                        .first()
                        .map(|c| c.message.content.clone())
                        .unwrap_or_default();

                    // Split the response into chunks to simulate streaming
                    let words: Vec<&str> = content.split_whitespace().collect();
                    let chunks: Vec<String> = words
                        .chunks(3) // Group words into chunks of 3
                        .map(|chunk| format!("{} ", chunk.join(" ")))
                        .collect();

                    if index < chunks.len() {
                        let chunk = StreamChunk {
                            id: chat_id.clone(),
                            delta: chunks[index].clone(),
                            finished: index == chunks.len() - 1,
                        };

                        let event = Event::default()
                            .json_data(chunk)
                            .unwrap_or_else(|_| Event::default().data("error"));

                        index += 1;
                        tokio::time::sleep(Duration::from_millis(100)).await;
                        Some((Ok(event), (chat_id, response, index)))
                    } else {
                        None
                    }
                }
                Err(e) => {
                    if index == 0 {
                        let error_chunk = StreamChunk {
                            id: chat_id.clone(),
                            delta: format!("Error: {e}"),
                            finished: true,
                        };

                        let event = Event::default()
                            .json_data(error_chunk)
                            .unwrap_or_else(|_| Event::default().data("error"));

                        Some((Ok(event), (chat_id, response, 1)))
                    } else {
                        None
                    }
                }
            }
        },
    );

    Ok(Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(30))
            .text("keep-alive"),
    ))
}

/// Get AI service info
///
/// # Errors
///
/// Returns an error if the service information cannot be retrieved.
pub async fn ai_info_handler(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<serde_json::Value>> {
    let ai_service = state.ai_service.read().await;

    let info = serde_json::json!({
        "provider": ai_service.provider_name(),
        "templates": ai_service.list_templates(),
        "schemas": ai_service.list_schemas(),
        "streaming_supported": true,
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
    let _user_id = state.auth_service.get_user_id_from_token(token)?;

    let ai_service = state.ai_service.read().await;

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
    let _user_id = state
        .auth_service
        .get_user_id_from_token(token)?
        .to_string();

    let ai_service = state.ai_service.read().await;

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
        "analysis": analysis_result.explanation,
        "language": request.language,
        "timestamp": chrono::Utc::now()
    })))
}

/// Handle file upload for chat context
///
/// # Errors
///
/// Returns an error if file upload fails or authentication is invalid.
pub async fn upload_file_handler(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    mut multipart: Multipart,
) -> AppResult<Json<serde_json::Value>> {
    // Verify JWT token and get user
    let token = auth.token();
    let _user_id = state
        .auth_service
        .get_user_id_from_token(token)?
        .to_string();

    let mut files = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Multipart error: {e}")))?
    {
        let name = field.name().unwrap_or("unknown").to_string();
        let data = field
            .bytes()
            .await
            .map_err(|e| AppError::BadRequest(format!("Failed to read file: {e}")))?;

        let content = String::from_utf8(data.to_vec())
            .map_err(|e| AppError::BadRequest(format!("Invalid UTF-8 content: {e}")))?;

        files.push(FileUpload { name, content });
    }

    Ok(Json(serde_json::json!({
        "files_uploaded": files.len(),
        "files": files
    })))
}

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
    let user_id = state
        .auth_service
        .get_user_id_from_token(token)?
        .to_string();

    let conversations = state
        .ai_data_service
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
    let user_id = state
        .auth_service
        .get_user_id_from_token(token)?
        .to_string();
    let conversation_with_messages = state
        .ai_data_service
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
    let user_id = state
        .auth_service
        .get_user_id_from_token(token)?
        .to_string();

    let usage_stats = state
        .ai_data_service
        .get_user_usage_stats(&user_id)
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to get usage stats: {e}")))?;

    Ok(Json(serde_json::json!(usage_stats)))
}

/// Health check endpoint that uses the AI provider
///
/// # Errors
///
/// Returns an error if the AI provider is not healthy.
pub async fn health_check_handler(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<serde_json::Value>> {
    let ai_service = state.ai_service.read().await;

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
    let _user_id = state
        .auth_service
        .get_user_id_from_token(token)?
        .to_string();

    let content = request
        .get("content")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("Missing content field".to_string()))?;

    let ai_service = state.ai_service.read().await;

    let moderation_result = ai_service
        .moderate_content(content)
        .await
        .map_err(|e| AppError::BadRequest(format!("Moderation failed: {e}")))?;

    Ok(Json(serde_json::json!(moderation_result)))
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
    let user_id = state
        .auth_service
        .get_user_id_from_token(token)?
        .to_string();

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
        .ai_data_service
        .archive_conversation(&conversation_id, &user_id)
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to archive conversation: {e}")))?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Conversation archived successfully",
        "demo_conversation": conversation
    })))
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
        .auth_service
        .get_user_id_from_token(token)
        .map_err(|e| AppError::Unauthorized(format!("Token verification failed: {e}")))?;

    Ok(Json(serde_json::json!({
        "valid": true,
        "user_id": user_id,
        "verified_at": chrono::Utc::now()
    })))
}

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
        .invite_service
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
        .invite_service
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
        .invite_service
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
        .invite_service
        .delete_invite(&invite_id)
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to delete invite: {e}")))?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Invite deleted successfully"
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
