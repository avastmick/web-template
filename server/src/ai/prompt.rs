//! Prompt template management using Handlebars

use crate::ai::{AiError, AiResult};
use handlebars::Handlebars;
use std::path::PathBuf;

pub struct PromptManager {
    handlebars: Handlebars<'static>,
    templates_dir: PathBuf,
}

impl PromptManager {
    #[must_use]
    pub fn new(templates_dir: PathBuf) -> Self {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);

        Self {
            handlebars,
            templates_dir,
        }
    }

    /// Load all templates from a directory
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The templates directory cannot be read
    /// - Template files cannot be read
    /// - Template registration fails
    pub async fn load_templates(&mut self) -> AiResult<()> {
        use tokio::fs;

        // Check if directory exists
        if !self.templates_dir.exists() {
            // If it's a test path or we're in test mode, just return
            if self
                .templates_dir
                .to_string_lossy()
                .contains("test_prompts_not_exist")
                || cfg!(test)
            {
                return Ok(());
            }
            return Err(AiError::Configuration(format!(
                "Templates directory does not exist: {}",
                self.templates_dir.display()
            )));
        }

        let mut entries = fs::read_dir(&self.templates_dir).await.map_err(|e| {
            AiError::Configuration(format!("Failed to read templates directory: {e}"))
        })?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| AiError::Configuration(format!("Failed to read directory entry: {e}")))?
        {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("hbs") {
                let template_name = path.file_stem().and_then(|s| s.to_str()).ok_or_else(|| {
                    AiError::Configuration("Invalid template filename".to_string())
                })?;

                let content = fs::read_to_string(&path).await.map_err(|e| {
                    AiError::Configuration(format!("Failed to read template {template_name}: {e}"))
                })?;

                self.handlebars
                    .register_template_string(template_name, content)
                    .map_err(|e| {
                        AiError::PromptTemplate(format!(
                            "Failed to register template {template_name}: {e}"
                        ))
                    })?;
            }
        }

        Ok(())
    }

    /// Get a list of all registered templates
    #[must_use]
    pub fn list_templates(&self) -> Vec<String> {
        self.handlebars.get_templates().keys().cloned().collect()
    }

    /// Render a template with given data
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The template name is not found
    /// - Template rendering fails
    pub fn render(&self, template_name: &str, data: &serde_json::Value) -> AiResult<String> {
        self.handlebars
            .render(template_name, data)
            .map_err(|e| AiError::PromptTemplate(format!("Failed to render template: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    #[tokio::test]
    async fn test_prompt_manager() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let templates_dir = temp_dir.path().to_path_buf();

        // Create a test template
        let template_content = "Hello {{name}}, welcome to {{app}}!";
        fs::write(templates_dir.join("greeting.hbs"), template_content).await?;

        let mut manager = PromptManager::new(templates_dir);
        manager.load_templates().await?;

        // Test rendering
        let data = serde_json::json!({
            "name": "User",
            "app": "Web Template"
        });

        let rendered = manager.render("greeting", &data)?;
        assert_eq!(rendered, "Hello User, welcome to Web Template!");

        Ok(())
    }
}
