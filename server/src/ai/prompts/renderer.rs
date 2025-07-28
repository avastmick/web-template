//! Handlebars-based prompt renderer for generating JSON API requests

use crate::ai::error::{AiError, AiResult};
use handlebars::Handlebars;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

/// Renders prompt templates using Handlebars to generate valid JSON requests
pub struct PromptRenderer {
    handlebars: Handlebars<'static>,
}

impl PromptRenderer {
    /// Create a new prompt renderer with all templates registered
    ///
    /// # Errors
    ///
    /// Returns an error if templates cannot be registered
    pub fn new() -> AiResult<Self> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);

        // Register escape function to prevent JSON escaping for schema insertion
        handlebars.register_escape_fn(handlebars::no_escape);

        // Register JSON helper for proper JSON string escaping
        handlebars.register_helper(
            "json",
            Box::new(
                |h: &handlebars::Helper,
                 _: &Handlebars,
                 _: &handlebars::Context,
                 _: &mut handlebars::RenderContext,
                 out: &mut dyn handlebars::Output|
                 -> handlebars::HelperResult {
                    if let Some(param) = h.param(0) {
                        // Use unwrap_or to provide a fallback for serialization errors
                        let json_str = serde_json::to_string(&param.value())
                            .unwrap_or_else(|_| "null".to_string());
                        out.write(&json_str)?;
                    }
                    Ok(())
                },
            ),
        );

        Ok(Self { handlebars })
    }

    /// Render a request template with the given context
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The template is not found
    /// - The template fails to render
    /// - The rendered JSON is invalid
    pub fn render_request<T: Serialize>(
        &self,
        template_name: &str,
        context: &T,
    ) -> AiResult<Value> {
        let json_str = self
            .handlebars
            .render(template_name, context)
            .map_err(|e| {
                AiError::Configuration(format!("Failed to render template {template_name}: {e}"))
            })?;

        serde_json::from_str(&json_str)
            .map_err(|e| AiError::Configuration(format!("Failed to parse rendered JSON: {e}")))
    }

    /// Render a request template with a `HashMap` context
    ///
    /// # Errors
    ///
    /// Returns an error if the template cannot be rendered
    pub fn render_request_with_map(
        &self,
        template_name: &str,
        context: &HashMap<String, Value>,
    ) -> AiResult<Value> {
        self.render_request(template_name, context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renderer_creation() {
        let result = PromptRenderer::new();
        assert!(
            result.is_ok(),
            "Failed to create PromptRenderer: {}",
            result.as_ref().err().map_or(
                "unknown error".to_string(),
                std::string::ToString::to_string
            )
        );
    }
}
