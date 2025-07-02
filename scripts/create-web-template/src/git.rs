use crate::errors::Result;
use crate::utils::{print_info, print_success};
use git2::{Repository, Signature};
use std::path::Path;

pub struct GitOperations;

impl GitOperations {
    pub fn init_repository(
        path: &Path,
        initial_commit: bool,
        message: Option<String>,
    ) -> Result<()> {
        print_info("Initializing git repository...");

        // Initialize repository
        let repo = Repository::init(path)?;

        if initial_commit {
            // Create initial commit
            let message = message.unwrap_or_else(|| "Initial commit from web-template".to_string());
            Self::create_initial_commit(&repo, &message)?;
        }

        // Create .gitignore if it doesn't exist
        Self::create_gitignore(path)?;

        print_success("Git repository initialized");
        Ok(())
    }

    fn create_initial_commit(repo: &Repository, message: &str) -> Result<()> {
        // Get signature
        let signature = Self::get_signature()?;

        // Get the index
        let mut index = repo.index()?;

        // Add all files to the index
        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;

        // Write the index as a tree
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;

        // Create the commit
        repo.commit(Some("HEAD"), &signature, &signature, message, &tree, &[])?;

        Ok(())
    }

    fn get_signature() -> Result<Signature<'static>> {
        // Try to get from git config
        let config = git2::Config::open_default()?;

        let name = config
            .get_string("user.name")
            .unwrap_or_else(|_| "Web Template User".to_string());

        let email = config
            .get_string("user.email")
            .unwrap_or_else(|_| "user@example.com".to_string());

        Ok(Signature::now(&name, &email)?)
    }

    fn create_gitignore(path: &Path) -> Result<()> {
        let gitignore_path = path.join(".gitignore");

        if !gitignore_path.exists() {
            let content = r"# Dependencies
node_modules/
.bun/

# Build outputs
target/
build/
dist/
.svelte-kit/

# Environment
.env
.envrc
!.envrc.example

# Logs
logs/
*.log

# OS
.DS_Store
Thumbs.db

# IDE
.vscode/
.idea/
*.swp
*.swo

# Testing
coverage/
.nyc_output/
test-results/

# Temporary files
*.tmp
*.temp
.cache/

# Database
*.db
*.sqlite
data/

# Generated files
generated/

# Rust
Cargo.lock
!Cargo.lock

# Process management
.overmind.sock
dump.rdb
";

            std::fs::write(gitignore_path, content)?;
        }

        Ok(())
    }

    pub fn clean_template_files(path: &Path, patterns: &[String]) -> Result<()> {
        print_info("Cleaning up template-specific files...");

        for pattern in patterns {
            let glob_pattern = path.join(pattern).to_string_lossy().to_string();

            for path in glob::glob(&glob_pattern).unwrap().flatten() {
                if path.is_file() {
                    std::fs::remove_file(&path)?;
                } else if path.is_dir() {
                    std::fs::remove_dir_all(&path)?;
                }
            }
        }

        Ok(())
    }
}
