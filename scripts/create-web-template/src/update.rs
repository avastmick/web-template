use crate::config::TemplateConfig;
use crate::errors::Result;
use crate::template::TemplateProcessor;
use crate::utils::{print_info, print_success, print_warning};
use crate::wizard::ProjectConfig;
use glob::Pattern;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub template_version: String,
    pub project_name: String,
    pub features: Vec<String>,
    pub database: String,
    pub created_at: String,
    pub last_updated: String,
}

pub struct UpdateManager {
    template_config: TemplateConfig,
    template_root: PathBuf,
    project_path: PathBuf,
}

impl UpdateManager {
    pub fn new(template_root: PathBuf, project_path: PathBuf) -> Result<Self> {
        // Load template configuration
        let config_path = template_root.join("template.config.json");
        let template_config = TemplateConfig::load_from_file(&config_path)?;

        Ok(Self {
            template_config,
            template_root,
            project_path,
        })
    }

    pub fn update(
        &self,
        force: bool,
        only_patterns: Option<Vec<String>>,
        exclude_patterns: Option<Vec<String>>,
        dry_run: bool,
    ) -> Result<()> {
        print_info("Checking project metadata...");

        // Load project metadata
        let metadata = self.load_project_metadata()?;

        // Check template version
        if metadata.template_version == self.template_config.template.version {
            print_info("Project is already up to date with the latest template version.");
            return Ok(());
        }

        print_info(&format!(
            "Updating from template version {} to {}",
            metadata.template_version, self.template_config.template.version
        ));

        // Get list of files to update
        let files_to_update = self.get_files_to_update(&only_patterns, &exclude_patterns)?;

        if files_to_update.is_empty() {
            print_info("No files to update based on the provided patterns.");
            return Ok(());
        }

        // Show files that will be updated
        println!("\nFiles to be updated:");
        for (_template_file, project_file) in &files_to_update {
            let relative_path = project_file
                .strip_prefix(&self.project_path)
                .unwrap_or(project_file);
            println!("  - {}", relative_path.display());
        }

        if dry_run {
            print_warning("Running in dry-run mode - no files will be modified");
            return Ok(());
        }

        // Confirm update
        if !force {
            println!("\n⚠️  This will overwrite local changes in the files listed above.");
            let confirm = dialoguer::Confirm::new()
                .with_prompt("Continue with update?")
                .default(false)
                .interact()
                .unwrap_or(false);

            if !confirm {
                print_info("Update cancelled.");
                return Ok(());
            }
        }

        // Create project config from metadata
        let project_config = self.create_project_config(&metadata)?;

        // Process updates
        let _processor =
            TemplateProcessor::new(self.template_config.clone(), self.template_root.clone());
        let context = Self::create_context(&project_config);

        let mut updated_count = 0;
        for (template_file, project_file) in files_to_update {
            if self.update_file(&template_file, &project_file, &context)? {
                updated_count += 1;
            }
        }

        // Update metadata
        self.update_project_metadata(&metadata)?;

        print_success(&format!("✨ Updated {} files successfully!", updated_count));

        // Show post-update instructions
        println!("\nNext steps:");
        println!("  1. Review the changes: git diff");
        println!("  2. Run checks: just check");
        println!("  3. Test your application");
        println!("  4. Commit the updates");

        Ok(())
    }

    fn load_project_metadata(&self) -> Result<ProjectMetadata> {
        let metadata_path = self.project_path.join(".web-template.json");

        if !metadata_path.exists() {
            // Try to infer metadata from existing project
            return self.infer_project_metadata();
        }

        let content = fs::read_to_string(&metadata_path)?;
        let metadata: ProjectMetadata = serde_json::from_str(&content)?;

        Ok(metadata)
    }

    fn infer_project_metadata(&self) -> Result<ProjectMetadata> {
        print_warning("No project metadata found. Inferring from project structure...");

        // Try to get project name from Cargo.toml
        let cargo_path = self.project_path.join("server/Cargo.toml");
        let project_name = if cargo_path.exists() {
            let content = fs::read_to_string(&cargo_path)?;
            Self::extract_project_name_from_cargo(&content).unwrap_or_else(|| "unknown".to_string())
        } else {
            "unknown".to_string()
        };

        // Detect features based on file existence
        let mut features = Vec::new();

        if self
            .project_path
            .join("server/src/handlers/auth_handler.rs")
            .exists()
        {
            features.push("local_auth".to_string());
        }
        if self
            .project_path
            .join("server/src/handlers/oauth_handler.rs")
            .exists()
        {
            features.push("google_auth".to_string());
            features.push("github_auth".to_string());
        }
        if self
            .project_path
            .join("server/src/handlers/payment_handler.rs")
            .exists()
        {
            features.push("stripe_payment".to_string());
        }
        if self.project_path.join("client/src/routes/chat").exists() {
            features.push("chat".to_string());
        }

        Ok(ProjectMetadata {
            template_version: "0.0.0".to_string(), // Unknown version
            project_name,
            features,
            database: "sqlite".to_string(), // Default assumption
            created_at: "unknown".to_string(),
            last_updated: chrono::Utc::now().to_rfc3339(),
        })
    }

    fn extract_project_name_from_cargo(content: &str) -> Option<String> {
        for line in content.lines() {
            if line.starts_with("name") {
                let parts: Vec<&str> = line.split('=').collect();
                if parts.len() == 2 {
                    return Some(
                        parts[1]
                            .trim()
                            .trim_matches('"')
                            .trim_matches('\'')
                            .to_string(),
                    );
                }
            }
        }
        None
    }

    fn get_files_to_update(
        &self,
        only_patterns: &Option<Vec<String>>,
        exclude_patterns: &Option<Vec<String>>,
    ) -> Result<Vec<(PathBuf, PathBuf)>> {
        let mut files = Vec::new();

        // Convert patterns to glob patterns
        let only_globs: Option<Vec<Pattern>> = only_patterns.as_ref().map(|patterns| {
            patterns
                .iter()
                .filter_map(|p| Pattern::new(p).ok())
                .collect()
        });

        let exclude_globs: Option<Vec<Pattern>> = exclude_patterns.as_ref().map(|patterns| {
            patterns
                .iter()
                .filter_map(|p| Pattern::new(p).ok())
                .collect()
        });

        // Walk template directory
        for entry in WalkDir::new(&self.template_root).follow_links(true) {
            let entry = entry?;
            let template_path = entry.path();

            // Skip directories
            if entry.file_type().is_dir() {
                continue;
            }

            // Get relative path
            let relative_path = template_path
                .strip_prefix(&self.template_root)
                .unwrap_or(template_path);

            // Skip template-specific files
            if self.should_skip_file(&relative_path) {
                continue;
            }

            // Apply only patterns
            if let Some(ref globs) = only_globs {
                let matches = globs.iter().any(|g| g.matches_path(&relative_path));
                if !matches {
                    continue;
                }
            }

            // Apply exclude patterns
            if let Some(ref globs) = exclude_globs {
                let matches = globs.iter().any(|g| g.matches_path(&relative_path));
                if matches {
                    continue;
                }
            }

            let project_path = self.project_path.join(&relative_path);

            // Only update files that exist in the project
            if project_path.exists() {
                files.push((template_path.to_path_buf(), project_path));
            }
        }

        Ok(files)
    }

    fn should_skip_file(&self, path: &Path) -> bool {
        let skip_patterns = vec![
            "template.config.json",
            "CLAUDE.md",
            "INSTRUCTIONS.md",
            "CURRENT_TASKS.md",
            "documentation/",
            "scripts/create-web-template/",
            ".git/",
            "target/",
            "node_modules/",
            "test-project/",
        ];

        let path_str = path.to_string_lossy();
        skip_patterns
            .iter()
            .any(|pattern| path_str.contains(pattern))
    }

    fn create_project_config(&self, metadata: &ProjectMetadata) -> Result<ProjectConfig> {
        let mut variables = HashMap::new();
        variables.insert("project_name".to_string(), metadata.project_name.clone());
        variables.insert(
            "project_description".to_string(),
            "A web application built with web-template".to_string(),
        );

        Ok(ProjectConfig {
            name: metadata.project_name.clone(),
            path: self.project_path.clone(),
            description: String::new(),
            author_name: String::new(),
            author_email: String::new(),
            features: metadata.features.clone(),
            database: metadata.database.clone(),
            variables,
        })
    }

    fn create_context(project_config: &ProjectConfig) -> tera::Context {
        let mut context = tera::Context::new();

        // Add all variables
        for (key, value) in &project_config.variables {
            context.insert(key, value);
        }

        // Add feature flags
        for feature in &project_config.features {
            context.insert(&format!("feature_{}", feature), &true);
        }

        // Add database type
        context.insert("db_sqlite", &(project_config.database == "sqlite"));
        context.insert("db_postgresql", &(project_config.database == "postgresql"));

        context
    }

    fn update_file(
        &self,
        template_file: &Path,
        project_file: &Path,
        context: &tera::Context,
    ) -> Result<bool> {
        // Read template file
        let template_content = fs::read_to_string(template_file)?;

        // Read existing file for comparison
        let existing_content = fs::read_to_string(project_file)?;

        // Process template (using simple substitution like in template.rs)
        let processed_content = Self::simple_variable_substitution(&template_content, context);

        // Only update if content is different
        if existing_content != processed_content {
            fs::write(project_file, processed_content)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn simple_variable_substitution(content: &str, ctx: &tera::Context) -> String {
        let mut result = content.to_string();

        // Perform simple variable substitution for common variables
        if let Some(project_name) = ctx.get("project_name").and_then(|v| v.as_str()) {
            result = result.replace("web-template", project_name);
            result = result.replace("web_template", &project_name.replace('-', "_"));
        }

        if let Some(description) = ctx.get("project_description").and_then(|v| v.as_str()) {
            result = result.replace(
                "A high-performance, secure web application template with Svelte frontend and Rust backend",
                description,
            );
        }

        result
    }

    fn update_project_metadata(&self, metadata: &ProjectMetadata) -> Result<()> {
        let metadata_path = self.project_path.join(".web-template.json");

        let updated_metadata = ProjectMetadata {
            template_version: self.template_config.template.version.clone(),
            project_name: metadata.project_name.clone(),
            features: metadata.features.clone(),
            database: metadata.database.clone(),
            created_at: metadata.created_at.clone(),
            last_updated: chrono::Utc::now().to_rfc3339(),
        };

        let content = serde_json::to_string_pretty(&updated_metadata)?;
        fs::write(metadata_path, content)?;

        Ok(())
    }
}
