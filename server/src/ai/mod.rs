//! AI provider abstraction and implementations

pub mod error;
pub mod models;
pub mod prompt;
pub mod provider;
pub mod schema;

pub use error::{AiError, AiResult};
pub use models::{ChatMessage, ChatRequest, ChatResponse, ChatRole};
pub use prompt::PromptManager;
pub use provider::{AiProvider, OpenRouterProvider};
pub use schema::{SchemaValidator, schemas};
