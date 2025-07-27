//! AI module - Centralized AI functionality for Kanbain
//!
//! This module provides:
//! - AI provider abstraction (`OpenRouter` as primary, configurable endpoint)
//! - Business logic services (issue analysis, duplicate detection, sizing)
//! - Prompt management and templating
//! - Structured response validation
//!
//! See `src/ai/README.md` for detailed architecture documentation.

pub mod error;
pub mod functions;
pub mod models;
pub mod prompts;
pub mod providers;
pub mod services;

// Re-export commonly used types
pub use error::{AiError, AiResult};
pub use functions::{
    FunctionCall, FunctionDefinition, FunctionResult, get_business_analyst_functions,
};
pub use models::chat::{ChatMessage, ChatRequest, ChatResponse, ChatRole};
pub use providers::{AiProvider, OpenRouterProvider};
pub use services::{SchemaValidator, schemas};
