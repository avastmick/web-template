//! AI-related error types

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AiError {
    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Provider error: {0}")]
    Provider(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("OpenAI client error: {0}")]
    OpenAIClient(#[from] Box<dyn std::error::Error + Send + Sync>),

    #[error("Prompt template error: {0}")]
    PromptTemplate(String),

    #[error("Schema validation error: {0}")]
    SchemaValidation(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type AiResult<T> = Result<T, AiError>;
