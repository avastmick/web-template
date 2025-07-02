use crate::errors::{Result, TemplateError};
use console::style;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};

pub fn validate_project_name(name: &str) -> Result<()> {
    let re = Regex::new(r"^[a-zA-Z][a-zA-Z0-9_-]*$").unwrap();

    if !re.is_match(name) {
        return Err(TemplateError::InvalidProjectName(
            "Project name must start with a letter and contain only letters, numbers, hyphens, and underscores".to_string()
        ));
    }

    if name.len() > 214 {
        return Err(TemplateError::InvalidProjectName(
            "Project name must be 214 characters or less".to_string(),
        ));
    }

    Ok(())
}

pub fn ensure_directory(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

pub fn find_project_root() -> Option<PathBuf> {
    let current_dir = std::env::current_dir().ok()?;

    for ancestor in current_dir.ancestors() {
        if ancestor.join("template.config.json").exists() {
            return Some(ancestor.to_path_buf());
        }
    }

    None
}

pub fn print_success(message: &str) {
    println!("{} {}", style("✓").green().bold(), message);
}

pub fn print_error(message: &str) {
    eprintln!("{} {}", style("✗").red().bold(), message);
}

pub fn print_info(message: &str) {
    println!("{} {}", style("ℹ").blue().bold(), message);
}

pub fn print_warning(message: &str) {
    println!("{} {}", style("⚠").yellow().bold(), message);
}

pub fn is_binary_file(path: &Path) -> Result<bool> {
    let data = fs::read(path)?;

    // Check for null bytes in first 8KB
    let check_len = std::cmp::min(8192, data.len());
    Ok(data[..check_len].contains(&0))
}
