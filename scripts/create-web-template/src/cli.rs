use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "create-web-template",
    about = "Create new projects from the web-template",
    version,
    author
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short, long, global = true, help = "Enable verbose output")]
    pub verbose: bool,

    #[arg(long, global = true, help = "Run in dry-run mode (preview changes)")]
    pub dry_run: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Create a new project from the template")]
    New {
        #[arg(help = "Name of the new project")]
        name: String,

        #[arg(short, long, help = "Target directory (defaults to project name)")]
        path: Option<PathBuf>,

        #[arg(long, help = "Skip interactive setup")]
        no_interactive: bool,

        #[arg(
            long,
            value_delimiter = ',',
            help = "Enable features (comma-separated)"
        )]
        features: Option<Vec<String>>,

        #[arg(long, help = "Template source path or URL")]
        template: Option<String>,
    },

    #[command(about = "Update an existing project with latest template changes")]
    Update {
        #[arg(
            short,
            long,
            help = "Project directory to update (defaults to current)"
        )]
        path: Option<PathBuf>,

        #[arg(long, help = "Force update, overwriting local changes")]
        force: bool,

        #[arg(long, help = "Only update specific files (glob patterns)")]
        only: Option<Vec<String>>,

        #[arg(long, help = "Exclude files from update (glob patterns)")]
        exclude: Option<Vec<String>>,
    },

    #[command(about = "Show or modify template configuration")]
    Config {
        #[command(subcommand)]
        action: Option<ConfigAction>,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    #[command(about = "Show current configuration")]
    Show,

    #[command(about = "List available features")]
    Features,

    #[command(about = "Validate configuration")]
    Validate {
        #[arg(short, long, help = "Configuration file path")]
        file: Option<PathBuf>,
    },
}
