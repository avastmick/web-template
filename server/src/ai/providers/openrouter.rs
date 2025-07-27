//! `OpenRouter` AI provider implementation

use super::traits::AiProvider;
use crate::ai::{AiResult, ChatMessage, ChatRequest, ChatResponse, ChatRole};
use async_trait::async_trait;
use openai_api_rs::v1::api::OpenAIClient;
use openai_api_rs::v1::chat_completion::{
    ChatCompletionMessage, ChatCompletionRequest, Content, MessageRole,
};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct OpenRouterProvider {
    client: Arc<Mutex<OpenAIClient>>,
    default_model: String,
}

impl OpenRouterProvider {
    /// Process and format JSON schema for `OpenAI` structured outputs
    fn format_json_schema(json_schema: &serde_json::Value) -> serde_json::Value {
        // Ensure consistent key ordering in JSON schema
        let ordered_schema = if let Ok(schema_str) = serde_json::to_string(json_schema) {
            let ordered: serde_json::Map<String, serde_json::Value> =
                serde_json::from_str(&schema_str).unwrap_or_else(|_| serde_json::Map::new());
            let sorted: serde_json::Map<String, serde_json::Value> = ordered
                .into_iter()
                .collect::<std::collections::BTreeMap<_, _>>()
                .into_iter()
                .collect();
            serde_json::Value::Object(sorted)
        } else {
            json_schema.clone()
        };

        // Extract the actual schema object if it's nested
        let final_schema = if let Some(obj) = ordered_schema.as_object() {
            if let Some(schema_field) = obj.get("schema") {
                // The schema is nested, extract it
                schema_field.clone()
            } else {
                // Use as-is
                ordered_schema.clone()
            }
        } else {
            ordered_schema.clone()
        };

        // Pretty print the given JSON schema for debugging
        if tracing::enabled!(tracing::Level::DEBUG) {
            if let Ok(json) = serde_json::to_string_pretty(&final_schema) {
                tracing::debug!("Final JSON Schema:\n{}", json);
            }
        }

        // Create the OpenAI-compatible response_format structure
        serde_json::json!({
            "type": "json_schema",
            "json_schema": {
                "name": "response_schema",
                "strict": true,
                "schema": final_schema
            }
        })
    }
    /// Create a new `OpenRouter` provider from environment configuration
    ///
    /// Reads configuration from:
    /// - `OPENROUTER_API_KEY` - Required API key
    /// - `AI_DEFAULT_MODEL`   - Required default model
    /// - `OPENROUTER_ENDPOINT` - Optional endpoint (defaults to `OpenRouter`)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Required environment variables are missing
    /// - The `OpenAI` client cannot be created
    pub fn new() -> AiResult<Self> {
        // Get configuration from environment
        let api_key = std::env::var("OPENROUTER_API_KEY").map_err(|_| {
            crate::ai::AiError::Provider(
                "OPENROUTER_API_KEY environment variable is required".to_string(),
            )
        })?;
        let default_model = std::env::var("AI_DEFAULT_MODEL").map_err(|_| {
            crate::ai::AiError::Provider(
                "AI_DEFAULT_MODEL environment variable is required".to_string(),
            )
        })?;

        let endpoint = std::env::var("OPENROUTER_ENDPOINT")
            .unwrap_or_else(|_| "https://openrouter.ai/api/v1".to_string());

        tracing::info!("Using provider endpoint: {endpoint}");
        tracing::info!("Using AI model: {default_model}");

        let client = OpenAIClient::builder()
            .with_endpoint(&endpoint)
            .with_api_key(api_key)
            .build()
            .map_err(|e| crate::ai::AiError::Provider(format!("Failed to create client: {e}")))?;

        Ok(Self {
            client: Arc::new(Mutex::new(client)),
            default_model,
        })
    }

    /// Create a new `OpenRouter` provider with explicit configuration
    ///
    /// This method is kept for backwards compatibility and testing
    ///
    /// # Errors
    ///
    /// Returns an error if the `OpenAI` client cannot be created
    pub fn with_config(api_key: String, default_model: String) -> AiResult<Self> {
        let endpoint = std::env::var("OPENROUTER_ENDPOINT")
            .unwrap_or_else(|_| "https://openrouter.ai/api/v1".to_string());

        let client = OpenAIClient::builder()
            .with_endpoint(&endpoint)
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

    fn model(&self) -> &str {
        &self.default_model
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
                    ChatRole::Function => MessageRole::function,
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

        // Add response_format if provided
        if let Some(response_format) = request.response_format {
            if let Some(json_schema) = response_format.json_schema {
                // Use the helper function to format the schema
                let response_format_value = Self::format_json_schema(&json_schema);
                // OpenAI-api-rs expects response_format as a serde_json::Value
                req.response_format = Some(response_format_value);
            } else {
                return Err(AiError::Provider(
                    "response_format requested but json_schema is None".to_string(),
                ));
            }
        }

        // Add function definitions if provided
        if let Some(_functions) = request.functions {
            // TODO: Add function calling support when the OpenAI client supports it
            tracing::warn!(
                "function calling requested but not yet supported by current OpenAI client"
            );
        }

        // Make the API call
        match self.client.lock().await.chat_completion(req).await {
            Ok(response) => {
                // Convert OpenAI response to our ChatResponse
                let choices = response
                    .choices
                    .into_iter()
                    .map(|choice| crate::ai::models::ChatChoice {
                        index: u32::try_from(choice.index).unwrap_or(0),
                        message: ChatMessage {
                            role: match choice.message.role {
                                MessageRole::system => ChatRole::System,
                                MessageRole::user => ChatRole::User,
                                MessageRole::assistant => ChatRole::Assistant,
                                MessageRole::function | MessageRole::tool => ChatRole::Function,
                            },
                            content: choice.message.content.unwrap_or_default(),
                        },
                        finish_reason: choice.finish_reason.map(|fr| format!("{fr:?}")),
                    })
                    .collect();

                let usage = Some(crate::ai::models::TokenUsage {
                    prompt: u32::try_from(response.usage.prompt_tokens).unwrap_or(0),
                    completion: u32::try_from(response.usage.completion_tokens).unwrap_or(0),
                    total: u32::try_from(response.usage.total_tokens).unwrap_or(0),
                });

                Ok(ChatResponse {
                    id: response.id.unwrap_or_default(),
                    model: response.model,
                    choices,
                    usage,
                    created: response.created,
                    function_call: None, // TODO: Extract function calls from response
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
