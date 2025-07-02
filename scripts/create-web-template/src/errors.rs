use thiserror::Error;

#[derive(Error, Debug)]
pub enum TemplateError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Template rendering error: {0}")]
    Template(#[from] tera::Error),

    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("Walk directory error: {0}")]
    WalkDir(#[from] walkdir::Error),

    #[error("Dialogue error: {0}")]
    Dialogue(#[from] dialoguer::Error),

    #[error("Invalid project name: {0}")]
    InvalidProjectName(String),

    #[error("Target directory already exists: {0}")]
    DirectoryExists(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Feature conflict: {0}")]
    FeatureConflict(String),

    #[error("Command execution failed: {0}")]
    CommandFailed(String),
}

pub type Result<T> = std::result::Result<T, TemplateError>;
