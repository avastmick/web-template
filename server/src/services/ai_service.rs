//! AI service that integrates provider, prompts, and schema validation

use crate::ai::{
    AiError, AiProvider, AiResult, ChatMessage, ChatRequest, ChatResponse, ChatRole,
    OpenRouterProvider, PromptManager, SchemaValidator, schemas,
};
use std::env;
use std::path::PathBuf;
use std::sync::Arc;

/// Main AI service that coordinates all AI functionality
pub struct AiService {
    provider: Arc<dyn AiProvider>,
    prompt_manager: PromptManager,
    schema_validator: SchemaValidator,
}

impl AiService {
    /// Create a new AI service with the given configuration
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The AI provider cannot be initialized
    /// - The prompt templates cannot be loaded
    ///
    /// # Panics
    ///
    /// Panics if `OPENROUTER_API_KEY` environment variable is not set in production
    pub async fn new() -> AiResult<Self> {
        // Get configuration from environment variables
        // For testing, use a placeholder API key if not provided
        let api_key = env::var("OPENROUTER_API_KEY").unwrap_or_else(|_| {
            if cfg!(test) {
                "test_api_key_placeholder".to_string()
            } else {
                panic!("OPENROUTER_API_KEY environment variable is required")
            }
        });

        let default_model =
            env::var("AI_DEFAULT_MODEL").unwrap_or_else(|_| "openai/gpt-4o-mini".to_string());

        // Check if this is a test environment
        let is_test = api_key == "test_api_key_placeholder" || api_key == "test_api_key_not_real";

        // Initialize provider
        let provider: Arc<dyn AiProvider> =
            Arc::new(OpenRouterProvider::new(api_key, default_model)?);

        // Initialize prompt manager
        let prompts_dir = if is_test {
            // Use a dummy directory path for tests
            PathBuf::from("/tmp/test_prompts_not_exist")
        } else {
            PathBuf::from("prompts")
        };
        let mut prompt_manager = PromptManager::new(prompts_dir);
        prompt_manager.load_templates().await?;

        // Initialize schema validator with common schemas
        let mut schema_validator = SchemaValidator::new();
        schema_validator.register_schema("moderation", schemas::moderation_response())?;
        schema_validator.register_schema("code_analysis", schemas::code_analysis())?;

        Ok(Self {
            provider,
            prompt_manager,
            schema_validator,
        })
    }

    /// Send a simple chat message
    ///
    /// # Errors
    ///
    /// Returns an error if the chat request fails
    pub async fn chat(&self, messages: Vec<ChatMessage>) -> AiResult<ChatResponse> {
        let request = ChatRequest::new(messages);
        self.provider.chat(request).await
    }

    /// Send a chat message with a specific prompt template
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The template doesn't exist
    /// - The template rendering fails
    /// - The chat request fails
    pub async fn chat_with_template<T: serde::Serialize>(
        &self,
        template_name: &str,
        data: &T,
    ) -> AiResult<ChatResponse> {
        let json_data = serde_json::to_value(data).map_err(AiError::Serialization)?;
        let prompt = self.prompt_manager.render(template_name, &json_data)?;

        let messages = vec![ChatMessage {
            role: ChatRole::User,
            content: prompt,
        }];

        let request = ChatRequest::new(messages);
        self.provider.chat(request).await
    }

    /// Send a chat message and validate the response against a schema
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The chat request fails
    /// - The response doesn't match the schema
    pub async fn chat_with_schema(
        &self,
        messages: Vec<ChatMessage>,
        schema_name: &str,
    ) -> AiResult<serde_json::Value> {
        // Verify the schema exists
        if !self
            .schema_validator
            .list_schemas()
            .contains(&schema_name.to_string())
        {
            return Err(AiError::SchemaValidation(format!(
                "Schema '{schema_name}' not found"
            )));
        }

        // Create request with JSON response format
        let request = ChatRequest::new(messages).with_json_schema(serde_json::json!({
            "name": schema_name,
            "strict": true
        }));

        let response = self.provider.chat(request).await?;

        // Extract and validate the response
        if let Some(choice) = response.choices.first() {
            let content = &choice.message.content;
            let json_value: serde_json::Value = serde_json::from_str(content)
                .map_err(|e| AiError::SchemaValidation(format!("Invalid JSON response: {e}")))?;

            // Validate against schema
            self.schema_validator.validate(schema_name, &json_value)?;

            Ok(json_value)
        } else {
            Err(AiError::Provider(
                "No response from AI provider".to_string(),
            ))
        }
    }

    /// Moderate content using AI
    ///
    /// # Errors
    ///
    /// Returns an error if the moderation request fails
    pub async fn moderate_content(&self, content: &str) -> AiResult<ModerationResult> {
        let data = serde_json::json!({
            "content": content
        });

        let response = self
            .chat_with_template("moderation/content_check", &data)
            .await?;

        if let Some(choice) = response.choices.first() {
            let result: ModerationResult =
                serde_json::from_str(&choice.message.content).map_err(|e| {
                    AiError::SchemaValidation(format!("Invalid moderation response: {e}"))
                })?;
            Ok(result)
        } else {
            Err(AiError::Provider("No response from moderation".to_string()))
        }
    }

    /// Analyze code using AI
    ///
    /// # Errors
    ///
    /// Returns an error if the code analysis request fails
    pub async fn analyze_code(
        &self,
        code: &str,
        language: Option<&str>,
        specific_question: Option<&str>,
    ) -> AiResult<CodeAnalysisResult> {
        let data = serde_json::json!({
            "code": code,
            "language": language,
            "specific_question": specific_question
        });

        let response = self.chat_with_template("code/explain", &data).await?;

        if let Some(choice) = response.choices.first() {
            // For code analysis, we return the text response directly
            Ok(CodeAnalysisResult {
                explanation: choice.message.content.clone(),
            })
        } else {
            Err(AiError::Provider(
                "No response from code analysis".to_string(),
            ))
        }
    }

    /// Get the current provider name
    #[must_use]
    pub fn provider_name(&self) -> &str {
        self.provider.name()
    }

    /// Perform a health check on the AI provider
    ///
    /// # Errors
    ///
    /// Returns an error if the provider is not responding or unhealthy
    pub async fn health_check(&self) -> AiResult<()> {
        self.provider.health_check().await
    }

    /// List available prompt templates
    #[must_use]
    pub fn list_templates(&self) -> Vec<String> {
        self.prompt_manager.list_templates()
    }

    /// List available schemas
    #[must_use]
    pub fn list_schemas(&self) -> Vec<String> {
        self.schema_validator.list_schemas()
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModerationResult {
    pub safe: bool,
    pub issues: Vec<String>,
    pub severity: String,
    pub recommendation: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CodeAnalysisResult {
    pub explanation: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ai_service_creation() {
        // This test would require mock settings
        // For now, just verify the types compile
        let _ = std::marker::PhantomData::<AiService>;
    }
}
