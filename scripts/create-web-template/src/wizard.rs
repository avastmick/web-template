use crate::config::TemplateConfig;
use crate::errors::{Result, TemplateError};
use crate::utils::{print_info, validate_project_name};
use console::{Term, style};
use dialoguer::{Confirm, Input, MultiSelect, Select, theme::ColorfulTheme};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct ProjectConfig {
    pub name: String,
    pub path: PathBuf,
    // These fields are used to populate the variables HashMap
    #[allow(dead_code)] // Used via variables HashMap
    pub description: String,
    #[allow(dead_code)] // Used via variables HashMap
    pub author_name: String,
    #[allow(dead_code)] // Used via variables HashMap
    pub author_email: String,
    pub features: Vec<String>,
    pub database: String,
    pub variables: HashMap<String, String>,
}

pub struct SetupWizard {
    template_config: TemplateConfig,
    theme: ColorfulTheme,
    term: Term,
}

impl SetupWizard {
    pub fn new(template_config: TemplateConfig) -> Self {
        Self {
            template_config,
            theme: ColorfulTheme::default(),
            term: Term::stdout(),
        }
    }

    pub fn run(&self, initial_name: Option<String>) -> Result<ProjectConfig> {
        self.term.clear_screen().ok();

        // Welcome message
        println!("{}", style("🚀 Web Template Project Setup").bold().cyan());
        println!("{}", style("──────────────────────────────").dim());
        println!();

        // Project name
        let name = self.get_project_name(initial_name)?;

        // Project path
        let path = self.get_project_path(&name)?;

        // Project description
        let description = self.get_project_description()?;

        // Author information
        let (author_name, author_email) = self.get_author_info()?;

        // Feature selection
        let features = self.select_features()?;

        // Database selection
        let database = self.select_database(&features)?;

        // Collect all variables
        let variables = self.collect_variables(
            &name,
            &description,
            &author_name,
            &author_email,
            &features,
            &database,
        )?;

        // Summary
        Self::show_summary(&name, &path, &description, &features, &database);

        // Confirmation
        if !self.confirm_setup()? {
            return Err(TemplateError::Config("Setup cancelled by user".to_string()));
        }

        Ok(ProjectConfig {
            name,
            path,
            description,
            author_name,
            author_email,
            features,
            database,
            variables,
        })
    }

    fn get_project_name(&self, initial: Option<String>) -> Result<String> {
        let name: String = Input::with_theme(&self.theme)
            .with_prompt("Project name")
            .default(initial.unwrap_or_default())
            .validate_with(|input: &String| validate_project_name(input).map_err(|e| e.to_string()))
            .interact_text()?;

        Ok(name)
    }

    fn get_project_path(&self, project_name: &str) -> Result<PathBuf> {
        let default_path = std::env::current_dir()?.join(project_name);

        let path_str: String = Input::with_theme(&self.theme)
            .with_prompt("Project directory")
            .default(default_path.to_string_lossy().to_string())
            .interact_text()?;

        let path = PathBuf::from(path_str);

        // Check if directory exists
        if path.exists() {
            if path.is_file() {
                return Err(TemplateError::DirectoryExists(
                    "Target path exists and is a file".to_string(),
                ));
            }

            if path.read_dir()?.next().is_some() {
                let overwrite = Confirm::with_theme(&self.theme)
                    .with_prompt("Directory exists and is not empty. Continue anyway?")
                    .default(false)
                    .interact()?;

                if !overwrite {
                    return Err(TemplateError::DirectoryExists(
                        "Directory exists and is not empty".to_string(),
                    ));
                }
            }
        }

        Ok(path)
    }

    fn get_project_description(&self) -> Result<String> {
        let description: String = Input::with_theme(&self.theme)
            .with_prompt("Project description")
            .default("A web application built with web-template".to_string())
            .interact_text()?;

        Ok(description)
    }

    fn get_author_info(&self) -> Result<(String, String)> {
        // Try to get from git config
        let git_name = std::process::Command::new("git")
            .args(["config", "user.name"])
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        let git_email = std::process::Command::new("git")
            .args(["config", "user.email"])
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        let author_name: String = Input::with_theme(&self.theme)
            .with_prompt("Author name")
            .default(git_name)
            .allow_empty(true)
            .interact_text()?;

        let author_email: String = Input::with_theme(&self.theme)
            .with_prompt("Author email")
            .default(git_email)
            .allow_empty(true)
            .interact_text()?;

        Ok((author_name, author_email))
    }

    fn select_features(&self) -> Result<Vec<String>> {
        println!();
        print_info("Select features to include in your project:");

        let feature_list: Vec<(&str, &str, bool)> = vec![
            ("local_auth", "Email/password authentication", true),
            ("google_auth", "Google OAuth authentication", true),
            ("github_auth", "GitHub OAuth authentication", true),
            ("stripe_payment", "Stripe payment integration", true),
            ("chat", "Chat functionality", true),
        ];

        let defaults: Vec<bool> = feature_list
            .iter()
            .map(|(_, _, default)| *default)
            .collect();
        let items: Vec<&str> = feature_list.iter().map(|(_, desc, _)| *desc).collect();

        let selections = MultiSelect::with_theme(&self.theme)
            .with_prompt("Features")
            .items(&items)
            .defaults(&defaults)
            .interact()?;

        let selected_features: Vec<String> = selections
            .into_iter()
            .map(|i| feature_list[i].0.to_string())
            .collect();

        // Validate features
        self.template_config.resolve_features(&selected_features)?;

        Ok(selected_features)
    }

    fn select_database(&self, _features: &[String]) -> Result<String> {
        println!();
        let databases = vec!["SQLite (recommended for development)", "PostgreSQL"];

        let selection = Select::with_theme(&self.theme)
            .with_prompt("Database")
            .items(&databases)
            .default(0)
            .interact()?;

        Ok(match selection {
            0 => "sqlite".to_string(),
            1 => "postgresql".to_string(),
            _ => unreachable!(),
        })
    }

    fn collect_variables(
        &self,
        name: &str,
        description: &str,
        author_name: &str,
        author_email: &str,
        features: &[String],
        database: &str,
    ) -> Result<HashMap<String, String>> {
        let mut variables = HashMap::new();

        // Base variables
        variables.insert("project_name".to_string(), name.to_string());
        variables.insert("project_description".to_string(), description.to_string());
        variables.insert("author_name".to_string(), author_name.to_string());
        variables.insert("author_email".to_string(), author_email.to_string());

        // Database URL
        let database_url = match database {
            "sqlite" => "sqlite://data/app.db".to_string(),
            "postgresql" => {
                println!();
                Input::with_theme(&self.theme)
                    .with_prompt("PostgreSQL connection URL")
                    .default("postgresql://user:password@localhost/dbname".to_string())
                    .interact_text()?
            }
            _ => unreachable!(),
        };
        variables.insert("database_url".to_string(), database_url);

        // Feature-specific variables
        if features.contains(&"google_auth".to_string()) {
            println!();
            print_info("Google OAuth configuration (leave empty to configure later):");

            let client_id: String = Input::with_theme(&self.theme)
                .with_prompt("Google Client ID")
                .allow_empty(true)
                .interact_text()?;

            let client_secret: String = Input::with_theme(&self.theme)
                .with_prompt("Google Client Secret")
                .allow_empty(true)
                .interact_text()?;

            variables.insert("GOOGLE_CLIENT_ID".to_string(), client_id);
            variables.insert("GOOGLE_CLIENT_SECRET".to_string(), client_secret);
        }

        if features.contains(&"github_auth".to_string()) {
            println!();
            print_info("GitHub OAuth configuration (leave empty to configure later):");

            let client_id: String = Input::with_theme(&self.theme)
                .with_prompt("GitHub Client ID")
                .allow_empty(true)
                .interact_text()?;

            let client_secret: String = Input::with_theme(&self.theme)
                .with_prompt("GitHub Client Secret")
                .allow_empty(true)
                .interact_text()?;

            variables.insert("GITHUB_CLIENT_ID".to_string(), client_id);
            variables.insert("GITHUB_CLIENT_SECRET".to_string(), client_secret);
        }

        if features.contains(&"stripe_payment".to_string()) {
            println!();
            print_info("Stripe configuration (leave empty to configure later):");

            let secret_key: String = Input::with_theme(&self.theme)
                .with_prompt("Stripe Secret Key")
                .allow_empty(true)
                .interact_text()?;

            let webhook_secret: String = Input::with_theme(&self.theme)
                .with_prompt("Stripe Webhook Secret")
                .allow_empty(true)
                .interact_text()?;

            variables.insert("STRIPE_SECRET_KEY".to_string(), secret_key);
            variables.insert("STRIPE_WEBHOOK_SECRET".to_string(), webhook_secret);
        }

        Ok(variables)
    }

    fn show_summary(
        name: &str,
        path: &Path,
        description: &str,
        features: &[String],
        database: &str,
    ) {
        println!();
        println!("{}", style("📋 Project Summary").bold().green());
        println!("{}", style("─────────────────").dim());
        println!("  {} {}", style("Name:").bold(), name);
        println!("  {} {}", style("Path:").bold(), path.display());
        println!("  {} {}", style("Description:").bold(), description);
        println!("  {} {}", style("Database:").bold(), database);
        println!("  {} {}", style("Features:").bold(), features.join(", "));
        println!();
    }

    fn confirm_setup(&self) -> Result<bool> {
        let confirm = Confirm::with_theme(&self.theme)
            .with_prompt("Create project with these settings?")
            .default(true)
            .interact()?;

        Ok(confirm)
    }
}
