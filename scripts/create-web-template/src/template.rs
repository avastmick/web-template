use crate::config::{FileMapping, TemplateConfig};
use crate::errors::{Result, TemplateError};
use crate::utils::{ensure_directory, is_binary_file, print_info};
use crate::wizard::ProjectConfig;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::{Path, PathBuf};
use tera::{Context, Tera};
use walkdir::WalkDir;

pub struct TemplateProcessor {
    template_config: TemplateConfig,
    tera: Tera,
    source_path: PathBuf,
}

impl TemplateProcessor {
    pub fn new(template_config: TemplateConfig, source_path: PathBuf) -> Self {
        let mut tera = Tera::default();
        tera.autoescape_on(vec![]);

        Self {
            template_config,
            tera,
            source_path,
        }
    }

    pub fn process(&mut self, project_config: &ProjectConfig, dry_run: bool) -> Result<()> {
        print_info("Processing template files...");

        // Create tera context from variables
        let context = Self::create_context(project_config);

        // Get list of files to process
        let files = self.collect_files(project_config)?;

        // Create progress bar
        let pb = ProgressBar::new(files.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );

        // Process files
        for (source, target) in files {
            pb.set_message(format!("Processing {}", target.display()));

            if dry_run {
                pb.inc(1);
                continue;
            }

            self.process_file(&source, &target, &context)?;
            pb.inc(1);
        }

        pb.finish_with_message("Template processing complete");

        Ok(())
    }

    fn create_context(project_config: &ProjectConfig) -> Context {
        let mut context = Context::new();

        // Add all variables
        for (key, value) in &project_config.variables {
            context.insert(key, value);
        }

        // Add feature flags
        for feature in &project_config.features {
            context.insert(format!("feature_{feature}"), &true);
        }

        // Add database type
        context.insert("db_sqlite", &(project_config.database == "sqlite"));
        context.insert("db_postgresql", &(project_config.database == "postgresql"));

        context
    }

    fn collect_files(&self, project_config: &ProjectConfig) -> Result<Vec<(PathBuf, PathBuf)>> {
        let mut files = Vec::new();

        // Get exclude patterns based on features
        let exclude_patterns = self.get_exclude_patterns(&project_config.features);

        // Walk the source directory
        for entry in WalkDir::new(&self.source_path).follow_links(true) {
            let entry = entry?;
            let path = entry.path();

            // Skip directories
            if entry.file_type().is_dir() {
                continue;
            }

            // Get relative path
            let relative_path = path.strip_prefix(&self.source_path).unwrap();
            let relative_str = relative_path.to_string_lossy();

            // Check if file should be excluded
            if Self::should_exclude(&relative_str, &exclude_patterns) {
                continue;
            }

            // Check for explicit file mappings
            let target_path = if let Some(mapping) = self.find_file_mapping(&relative_str) {
                // Check if mapping requires specific features
                if let Some(required_features) = &mapping.features {
                    let has_all_features = required_features
                        .iter()
                        .all(|f| project_config.features.contains(f));

                    if !has_all_features {
                        continue;
                    }
                }

                project_config.path.join(&mapping.to)
            } else {
                // Default mapping
                project_config.path.join(relative_path)
            };

            files.push((path.to_path_buf(), target_path));
        }

        Ok(files)
    }

    fn get_exclude_patterns(&self, enabled_features: &[String]) -> Vec<String> {
        let mut patterns = vec![
            // Version control
            r"\.git/.*".to_string(),
            r"\.gitignore".to_string(),
            // Build artifacts
            r"target/.*".to_string(),
            r"node_modules/.*".to_string(),
            r"\.svelte-kit/.*".to_string(),
            r"build/.*".to_string(),
            r"dist/.*".to_string(),
            // IDE
            r"\.vscode/.*".to_string(),
            r"\.idea/.*".to_string(),
            // Logs and temp files
            r"logs/.*".to_string(),
            r".*\.log".to_string(),
            r"\.DS_Store".to_string(),
            // Template-specific files
            r"scripts/create-web-template/.*".to_string(),
            r"template\.config\.json".to_string(),
            r"CLAUDE\.md".to_string(),
            r"INSTRUCTIONS\.md".to_string(),
            r"CURRENT_TASKS\.md".to_string(),
            // Documentation directory - template-specific
            r"documentation/.*".to_string(),
            // Other template-specific files
            r"scratchpad\.md".to_string(),
            r"check_auth\.js".to_string(),
            // Test artifacts
            r"test-results/.*".to_string(),
            r"test-project/.*".to_string(),
        ];

        // Add feature-specific excludes
        for (feature_name, feature) in &self.template_config.features {
            if !enabled_features.contains(feature_name) {
                patterns.extend(feature.excludes.clone());
            }
        }

        patterns
    }

    fn should_exclude(path: &str, patterns: &[String]) -> bool {
        for pattern in patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if re.is_match(path) {
                    return true;
                }
            }
        }
        false
    }

    fn find_file_mapping(&self, path: &str) -> Option<&FileMapping> {
        self.template_config
            .file_mappings
            .iter()
            .find(|m| m.from == path)
    }

    fn process_file(&mut self, source: &Path, target: &Path, context: &Context) -> Result<()> {
        // Ensure target directory exists
        if let Some(parent) = target.parent() {
            ensure_directory(parent)?;
        }

        // Check if file is binary
        if is_binary_file(source)? {
            // Just copy binary files
            fs::copy(source, target)?;
        } else {
            // Process as template
            let file_content = fs::read_to_string(source)?;

            // Check if this file should be processed as template
            let should_template = Self::should_template_file(source);

            let processed = if should_template {
                // Use simple variable substitution for all files to avoid Tera parsing issues
                Self::simple_variable_substitution(&file_content, context)
            } else {
                file_content
            };

            fs::write(target, processed)?;
        }

        // Preserve permissions on Unix
        #[cfg(unix)]
        {
            let metadata = fs::metadata(source)?;
            let permissions = metadata.permissions();
            fs::set_permissions(target, permissions)?;
        }

        Ok(())
    }

    fn should_template_file(path: &Path) -> bool {
        // List of file extensions that should NOT be templated
        let non_template_extensions = vec!["hbs", "handlebars"];

        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            if non_template_extensions.contains(&ext_str.as_str()) {
                return false;
            }
        }

        // List of file extensions that should be templated
        let template_extensions = vec![
            "rs", "toml", "json", "md", "txt", "yml", "yaml", "js", "ts", "jsx", "tsx", "svelte",
            "vue", "css", "scss", "html", "xml", "sh", "bash", "env", "example",
        ];

        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            return template_extensions.contains(&ext_str.as_str());
        }

        // Check for specific filenames without extensions
        if let Some(name) = path.file_name() {
            let name_str = name.to_string_lossy();
            return matches!(
                name_str.as_ref(),
                "Makefile" | "Dockerfile" | "README" | "LICENSE" | "Procfile"
            );
        }

        false
    }

    fn might_contain_template_syntax(content: &str) -> bool {
        // Check if content contains Handlebars-style templates or other template syntax
        // that might conflict with Tera
        content.contains("{{") && content.contains("}}")
    }

    fn simple_variable_substitution(content: &str, context: &Context) -> String {
        let mut result = content.to_string();

        // Perform simple variable substitution for common variables
        // This avoids Tera parsing issues with files containing template syntax
        if let Some(project_name) = context.get("project_name").and_then(|v| v.as_str()) {
            result = result.replace("web-template", project_name);
            result = result.replace("web_template", &project_name.replace('-', "_"));
        }

        if let Some(description) = context.get("project_description").and_then(|v| v.as_str()) {
            result = result.replace("A high-performance, secure web application template with Svelte frontend and Rust backend", description);
        }

        if let Some(author_name) = context.get("author_name").and_then(|v| v.as_str()) {
            if !author_name.is_empty() {
                result = result.replace("Your Name", author_name);
            }
        }

        if let Some(author_email) = context.get("author_email").and_then(|v| v.as_str()) {
            if !author_email.is_empty() {
                result = result.replace("your-email@example.com", author_email);
            }
        }

        result
    }
}
