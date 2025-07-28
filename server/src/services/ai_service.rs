//! AI service that integrates provider and schema validation

use crate::ai::{
    AiError, AiProvider, AiResult, ChatMessage, ChatRequest, ChatResponse, ChatRole,
    OpenRouterProvider, SchemaValidator, prompts::PromptRenderer, schemas,
};
use std::sync::Arc;

/// Main AI service that coordinates all AI functionality
pub struct AiService {
    provider: Arc<dyn AiProvider>,
    schema_validator: SchemaValidator,
    prompt_renderer: PromptRenderer,
}

impl AiService {
    /// Create a new AI service with the given configuration
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The AI provider cannot be initialized
    ///
    /// # Panics
    ///
    /// Panics if `OPENROUTER_API_KEY` environment variable is not set in production
    pub fn new() -> AiResult<Self> {
        // Initialize provider using environment configuration
        let provider: Arc<dyn AiProvider> = Arc::new(OpenRouterProvider::new()?);

        // Initialize schema validator with common schemas
        let mut schema_validator = SchemaValidator::new();
        schema_validator.register_schema("moderation_response", schemas::moderation_response())?;
        schema_validator.register_schema("code_analysis", schemas::code_analysis())?;

        // Initialize prompt renderer
        let prompt_renderer = PromptRenderer::new()?;

        Ok(Self {
            provider,
            schema_validator,
            prompt_renderer,
        })
    }

    /// Get the AI provider
    #[must_use]
    pub fn provider(&self) -> Arc<dyn AiProvider> {
        Arc::clone(&self.provider)
    }

    /// Send a chat message to the AI provider
    ///
    /// # Errors
    ///
    /// Returns an error if the provider fails
    pub async fn chat(&self, request: ChatRequest) -> AiResult<ChatResponse> {
        self.provider.chat(request).await
    }

    /// Send a system message to the AI
    ///
    /// # Errors
    ///
    /// Returns an error if the provider fails
    pub async fn send_system_message(
        &self,
        system_content: String,
        user_content: String,
    ) -> AiResult<String> {
        let request = ChatRequest {
            model: Some(self.provider.model().to_string()),
            messages: vec![
                ChatMessage {
                    role: ChatRole::System,
                    content: system_content,
                },
                ChatMessage {
                    role: ChatRole::User,
                    content: user_content,
                },
            ],
            temperature: Some(0.7),
            max_tokens: Some(1000),
            response_format: None,
            stream: None,
            metadata: None,
            functions: None,
            function_call: None,
        };

        let response = self.chat(request).await?;

        response
            .choices
            .into_iter()
            .next()
            .map(|choice| choice.message.content)
            .ok_or_else(|| AiError::InvalidRequest("No message content in response".to_string()))
    }

    /// Get the provider name
    #[must_use]
    pub fn provider_name(&self) -> &str {
        self.provider.name()
    }

    /// List available schemas
    #[must_use]
    pub fn list_schemas(&self) -> Vec<String> {
        self.schema_validator.list_schemas()
    }

    /// Chat with a template
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The template cannot be rendered
    /// - The AI provider fails
    pub async fn chat_with_template(
        &self,
        template_name: &str,
        context: &serde_json::Value,
    ) -> AiResult<ChatResponse> {
        // Render the prompt using the template
        let rendered = self
            .prompt_renderer
            .render_request(template_name, context)?;

        // Convert rendered messages to ChatMessage
        let messages = if let Some(messages_array) = rendered.as_array() {
            messages_array
                .iter()
                .filter_map(|msg| {
                    let role = msg.get("role")?.as_str()?;
                    let msg_content = msg.get("content")?.as_str()?;
                    let role = match role {
                        "system" => ChatRole::System,
                        "user" => ChatRole::User,
                        "assistant" => ChatRole::Assistant,
                        _ => return None,
                    };
                    Some(ChatMessage {
                        role,
                        content: msg_content.to_string(),
                    })
                })
                .collect()
        } else {
            return Err(AiError::InvalidRequest(
                "Template did not render to an array of messages".to_string(),
            ));
        };

        let request = ChatRequest::new(messages);
        self.chat(request).await
    }

    /// Chat with schema validation
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The schema doesn't exist
    /// - The AI provider fails
    /// - The response doesn't match the schema
    pub async fn chat_with_schema(
        &self,
        messages: Vec<ChatMessage>,
        schema_name: &str,
    ) -> AiResult<ChatResponse> {
        // Get the schema
        let schemas_list = self.schema_validator.list_schemas();
        if !schemas_list.contains(&schema_name.to_string()) {
            return Err(AiError::InvalidRequest(format!(
                "Schema '{schema_name}' not found"
            )));
        }

        // Get the actual schema value
        let schema = match schema_name {
            "moderation_response" => schemas::moderation_response(),
            "code_analysis" => schemas::code_analysis(),
            _ => {
                return Err(AiError::InvalidRequest(format!(
                    "Unknown schema: {schema_name}"
                )));
            }
        };

        // Create request with JSON schema
        let request = ChatRequest::new(messages).with_json_schema(schema);

        // Send to AI provider
        let response = self.chat(request).await?;

        // Validate the response
        if let Some(choice) = response.choices.first() {
            let content_value: serde_json::Value = serde_json::from_str(&choice.message.content)
                .map_err(|e| AiError::InvalidRequest(format!("Invalid JSON response: {e}")))?;
            self.schema_validator
                .validate(schema_name, &content_value)?;
        }

        Ok(response)
    }

    /// Analyze code
    ///
    /// # Errors
    ///
    /// Returns an error if the AI provider fails
    pub async fn analyze_code(
        &self,
        code: &str,
        language: Option<&str>,
        context: Option<&str>,
    ) -> AiResult<serde_json::Value> {
        let template_data = serde_json::json!({
            "code": code,
            "language": language.unwrap_or("unknown"),
            "context": context.unwrap_or("")
        });

        let response = self
            .chat_with_template("code_analysis", &template_data)
            .await?;

        // Extract and parse the response
        let response_content = response
            .choices
            .first()
            .map(|c| &c.message.content)
            .ok_or_else(|| AiError::InvalidRequest("No response content".to_string()))?;

        serde_json::from_str(response_content)
            .map_err(|e| AiError::InvalidRequest(format!("Invalid JSON response: {e}")))
    }

    /// Check provider health
    ///
    /// # Errors
    ///
    /// Returns an error if the provider is unhealthy
    pub async fn health_check(&self) -> AiResult<()> {
        self.provider.health_check().await
    }

    /// Moderate content
    ///
    /// # Errors
    ///
    /// Returns an error if the AI provider fails
    pub async fn moderate_content(&self, content: &str) -> AiResult<serde_json::Value> {
        let template_data = serde_json::json!({
            "content": content
        });

        let response = self
            .chat_with_template("content_moderation", &template_data)
            .await?;

        // Extract and parse the response
        let response_content = response
            .choices
            .first()
            .map(|c| &c.message.content)
            .ok_or_else(|| AiError::InvalidRequest("No response content".to_string()))?;

        serde_json::from_str(response_content)
            .map_err(|e| AiError::InvalidRequest(format!("Invalid JSON response: {e}")))
    }
}
