//! AI provider trait and implementations

use crate::ai::{AiResult, ChatMessage, ChatRequest, ChatResponse, ChatRole};
use async_trait::async_trait;
use openai_api_rs::v1::api::OpenAIClient;
use openai_api_rs::v1::chat_completion::{
    ChatCompletionMessage, ChatCompletionRequest, Content, MessageRole,
};
use std::sync::Arc;
use tokio::sync::Mutex;

#[async_trait]
pub trait AiProvider: Send + Sync {
    /// Get the provider name
    fn name(&self) -> &str;

    /// Send a chat completion request
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The request is invalid
    /// - The provider is unreachable
    /// - The response cannot be parsed
    async fn chat(&self, request: ChatRequest) -> AiResult<ChatResponse>;

    /// Check if the provider is healthy
    ///
    /// # Errors
    ///
    /// Returns an error if the provider is not responding
    async fn health_check(&self) -> AiResult<()>;
}

pub struct OpenRouterProvider {
    client: Arc<Mutex<OpenAIClient>>,
    default_model: String,
}

impl OpenRouterProvider {
    /// Create a new `OpenRouter` provider with API key and default model
    ///
    /// # Errors
    ///
    /// Returns an error if the `OpenAI` client cannot be created
    pub fn new(api_key: String, default_model: String) -> AiResult<Self> {
        let client = OpenAIClient::builder()
            .with_endpoint("https://openrouter.ai/api/v1")
            .with_api_key(api_key)
            .build()
            .map_err(|e| crate::ai::AiError::Provider(format!("Failed to create client: {e}")))?;

        Ok(Self {
            client: Arc::new(Mutex::new(client)),
            default_model,
        })
    }
}

#[async_trait]
impl AiProvider for OpenRouterProvider {
    fn name(&self) -> &'static str {
        "openrouter"
    }

    async fn chat(&self, request: ChatRequest) -> AiResult<ChatResponse> {
        use crate::ai::AiError;

        // Convert our ChatRequest to OpenAI ChatCompletionRequest
        let model = request.model.unwrap_or_else(|| self.default_model.clone());

        let messages: Vec<ChatCompletionMessage> = request
            .messages
            .into_iter()
            .map(|msg| ChatCompletionMessage {
                role: match msg.role {
                    ChatRole::System => MessageRole::system,
                    ChatRole::User => MessageRole::user,
                    ChatRole::Assistant => MessageRole::assistant,
                },
                content: Content::Text(msg.content),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            })
            .collect();

        let mut req = ChatCompletionRequest::new(model, messages);

        if let Some(temp) = request.temperature {
            req.temperature = Some(temp.into());
        }

        if let Some(max_tokens) = request.max_tokens {
            req.max_tokens = Some(max_tokens.into());
        }

        // Make the API call
        match self.client.lock().await.chat_completion(req).await {
            Ok(response) => {
                // Convert OpenAI response to our ChatResponse
                let choices = response
                    .choices
                    .into_iter()
                    .map(|choice| {
                        crate::ai::models::ChatChoice {
                            index: u32::try_from(choice.index).unwrap_or(0),
                            message: ChatMessage {
                                role: match choice.message.role {
                                    MessageRole::system => ChatRole::System,
                                    MessageRole::user => ChatRole::User,
                                    _ => ChatRole::Assistant, // Default fallback
                                },
                                content: choice.message.content.unwrap_or_default(),
                            },
                            finish_reason: choice.finish_reason.map(|fr| format!("{fr:?}")),
                        }
                    })
                    .collect();

                let usage = Some(crate::ai::models::TokenUsage {
                    prompt_tokens: u32::try_from(response.usage.prompt_tokens).unwrap_or(0),
                    completion_tokens: u32::try_from(response.usage.completion_tokens).unwrap_or(0),
                    total_tokens: u32::try_from(response.usage.total_tokens).unwrap_or(0),
                });

                Ok(ChatResponse {
                    id: response.id.unwrap_or_default(),
                    model: response.model,
                    choices,
                    usage,
                    created: response.created,
                })
            }
            Err(e) => Err(AiError::Provider(format!("Chat request failed: {e}"))),
        }
    }

    async fn health_check(&self) -> AiResult<()> {
        // Simple health check by sending a basic message
        let messages = vec![ChatMessage {
            role: ChatRole::User,
            content: "Hi".to_string(),
        }];
        let request = crate::ai::ChatRequest::new(messages);
        self.chat(request).await.map(|_| ())
    }
}
