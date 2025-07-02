mod cli;
mod config;
mod errors;
mod git;
mod template;
mod update;
mod utils;
mod wizard;

use clap::Parser;
use cli::{Cli, Commands, ConfigAction};
use config::{PostProcessStep, TemplateConfig};
use errors::{Result, TemplateError};
use git::GitOperations;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use template::TemplateProcessor;
use update::{ProjectMetadata, UpdateManager};
use utils::{find_project_root, print_error, print_info, print_success, print_warning};
use wizard::{ProjectConfig, SetupWizard};

struct CreateProjectOptions {
    name: String,
    path: Option<PathBuf>,
    no_interactive: bool,
    features: Option<Vec<String>>,
    template_source: Option<String>,
    dry_run: bool,
    verbose: bool,
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        print_error(&format!("Error: {e}"));
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New {
            name,
            path,
            no_interactive,
            features,
            template: template_source,
        } => {
            let options = CreateProjectOptions {
                name,
                path,
                no_interactive,
                features,
                template_source,
                dry_run: cli.dry_run,
                verbose: cli.verbose,
            };
            create_project(options).await?;
        }

        Commands::Update {
            path,
            force,
            only,
            exclude,
        } => {
            update_project(path, force, only, exclude, cli.dry_run, cli.verbose).await?;
        }

        Commands::Config { action } => {
            handle_config_command(action)?;
        }
    }

    Ok(())
}

async fn create_project(options: CreateProjectOptions) -> Result<()> {
    // Find template root
    let template_root = if let Some(source) = &options.template_source {
        PathBuf::from(source)
    } else {
        find_project_root()
            .ok_or_else(|| TemplateError::Config("Could not find template root. Run from within the template directory or specify --template".to_string()))?
    };

    // Load template configuration
    let config_path = template_root.join("template.config.json");
    let template_config = TemplateConfig::load_from_file(&config_path)?;

    // Run wizard or use CLI args
    let project_config = if options.no_interactive {
        build_non_interactive_config(&options, &template_config)?
    } else {
        let wizard = SetupWizard::new(template_config.clone());
        let mut config = wizard.run(Some(options.name))?;

        // Override path if provided
        if let Some(path) = options.path {
            config.path = path;
        }

        config
    };

    if options.dry_run {
        print_warning("Running in dry-run mode - no files will be created");
    }

    // Process template
    let mut processor = TemplateProcessor::new(template_config.clone(), template_root.clone());
    processor.process(&project_config, options.dry_run)?;

    if !options.dry_run {
        // Run post-processing steps
        for step in &template_config.post_processing {
            match step {
                PostProcessStep::RemoveFiles { patterns } => {
                    GitOperations::clean_template_files(&project_config.path, patterns)?;
                }
                PostProcessStep::GitInit {
                    initial_commit,
                    message,
                } => {
                    GitOperations::init_repository(
                        &project_config.path,
                        *initial_commit,
                        message.clone(),
                    )?;
                }
                PostProcessStep::Command {
                    command,
                    args,
                    working_dir: _,
                } => {
                    // TODO: Implement command execution
                    if options.verbose {
                        print_info(&format!("Would run: {command} {}", args.join(" ")));
                    }
                }
            }
        }

        // Save project metadata
        save_project_metadata(&project_config, &template_config)?;

        // Install dependencies
        install_dependencies(&project_config.path, options.verbose).await?;
    }

    // Success message
    print_success(&format!(
        "✨ Project '{}' created successfully!",
        project_config.name
    ));
    println!();
    println!("Next steps:");
    println!("  cd {}", project_config.path.display());
    println!("  direnv allow");
    println!("  just dev");
    println!();

    Ok(())
}

async fn install_dependencies(project_path: &Path, verbose: bool) -> Result<()> {
    print_info("Installing dependencies...");

    // Check if we should install
    let should_install = dialoguer::Confirm::new()
        .with_prompt("Install dependencies now?")
        .default(true)
        .interact()
        .unwrap_or(false);

    if !should_install {
        print_info("Skipping dependency installation");
        return Ok(());
    }

    // Run just setup
    let output = tokio::process::Command::new("just")
        .arg("setup")
        .current_dir(project_path)
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(TemplateError::CommandFailed(format!(
            "just setup failed: {stderr}"
        )));
    }

    if verbose {
        println!("{}", String::from_utf8_lossy(&output.stdout));
    }

    print_success("Dependencies installed successfully");
    Ok(())
}

fn build_non_interactive_config(
    options: &CreateProjectOptions,
    template_config: &TemplateConfig,
) -> Result<ProjectConfig> {
    let path = options
        .path
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap().join(&options.name));

    let features = if let Some(feat) = &options.features {
        // Validate features
        template_config.resolve_features(feat)?;
        feat.clone()
    } else {
        // Use default features
        template_config
            .features
            .iter()
            .filter(|(_, f)| f.default)
            .map(|(name, _)| name.clone())
            .collect()
    };

    let mut variables = HashMap::new();
    variables.insert("project_name".to_string(), options.name.clone());
    variables.insert(
        "project_description".to_string(),
        "A web application built with web-template".to_string(),
    );
    variables.insert("author_name".to_string(), String::new());
    variables.insert("author_email".to_string(), String::new());
    variables.insert(
        "database_url".to_string(),
        "sqlite://data/app.db".to_string(),
    );

    Ok(ProjectConfig {
        name: options.name.clone(),
        path,
        description: variables["project_description"].clone(),
        author_name: String::new(),
        author_email: String::new(),
        features,
        database: "sqlite".to_string(),
        variables,
    })
}

fn handle_config_command(action: Option<ConfigAction>) -> Result<()> {
    // Find template root
    let template_root = find_project_root()
        .ok_or_else(|| TemplateError::Config("Could not find template root".to_string()))?;

    let config_path = template_root.join("template.config.json");
    let template_config = TemplateConfig::load_from_file(&config_path)?;

    match action {
        Some(ConfigAction::Show) | None => {
            println!("{}", serde_json::to_string_pretty(&template_config)?);
        }
        Some(ConfigAction::Features) => {
            println!("Available features:");
            println!();

            for (name, feature) in &template_config.features {
                let status = if feature.default { "(default)" } else { "" };
                println!("  {} {} - {}", name, status, feature.description);

                if !feature.depends_on.is_empty() {
                    println!("    Depends on: {}", feature.depends_on.join(", "));
                }

                if !feature.conflicts_with.is_empty() {
                    println!("    Conflicts with: {}", feature.conflicts_with.join(", "));
                }
            }
        }
        Some(ConfigAction::Validate { file }) => {
            let path = file.unwrap_or(config_path);
            let config = TemplateConfig::load_from_file(&path)?;
            config.validate()?;
            print_success("Configuration is valid!");
        }
    }

    Ok(())
}

fn save_project_metadata(
    project_config: &ProjectConfig,
    template_config: &TemplateConfig,
) -> Result<()> {
    let metadata_path = project_config.path.join(".web-template.json");

    let metadata = ProjectMetadata {
        template_version: template_config.template.version.clone(),
        project_name: project_config.name.clone(),
        features: project_config.features.clone(),
        database: project_config.database.clone(),
        created_at: chrono::Utc::now().to_rfc3339(),
        last_updated: chrono::Utc::now().to_rfc3339(),
    };

    let content = serde_json::to_string_pretty(&metadata)?;
    std::fs::write(metadata_path, content)?;

    Ok(())
}

async fn update_project(
    path: Option<PathBuf>,
    force: bool,
    only: Option<Vec<String>>,
    exclude: Option<Vec<String>>,
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    // Get project path (default to current directory)
    let project_path = path.unwrap_or_else(|| std::env::current_dir().unwrap());

    // Validate it's a valid project directory
    if !project_path.join("justfile").exists() {
        return Err(TemplateError::Config(
            "Not a valid web-template project directory (justfile not found)".to_string(),
        ));
    }

    // Find template root
    let template_root = find_project_root()
        .ok_or_else(|| TemplateError::Config("Could not find template root. Run from within the template directory or specify --template".to_string()))?;

    if verbose {
        print_info(&format!("Template root: {}", template_root.display()));
        print_info(&format!("Project path: {}", project_path.display()));
    }

    // Create update manager
    let update_manager = UpdateManager::new(template_root, project_path)?;

    // Run update
    update_manager.update(force, only, exclude, dry_run)?;

    Ok(())
}
