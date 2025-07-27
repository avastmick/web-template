//! Prompt template management
//!
//! This module handles loading and managing prompt templates for different AI personas
//! and use cases using Handlebars templates.

pub mod renderer;
pub mod templates;

pub use renderer::PromptRenderer;
