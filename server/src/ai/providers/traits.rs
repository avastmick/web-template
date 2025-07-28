//! AI provider trait definitions

use crate::ai::{
    error::AiResult,
    models::chat::{ChatRequest, ChatResponse},
};
use async_trait::async_trait;

/// Trait that all AI providers must implement
#[async_trait]
pub trait AiProvider: Send + Sync {
    /// Get the provider name
    fn name(&self) -> &str;

    /// Get the current model being used
    fn model(&self) -> &str;

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

    /// Get usage statistics for the current session
    ///
    /// # Errors
    ///
    /// Returns an error if usage stats cannot be retrieved
    fn get_usage_stats(&self) -> AiResult<UsageStats> {
        Ok(UsageStats::default())
    }
}

/// Usage statistics for AI providers
#[derive(Debug, Default, Clone)]
pub struct UsageStats {
    pub total_tokens: u32,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub requests_count: u32,
}
