use crate::errors::{Result, TemplateError};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    pub template: TemplateMetadata,
    pub variables: HashMap<String, Variable>,
    pub features: HashMap<String, Feature>,
    pub file_mappings: Vec<FileMapping>,
    pub post_processing: Vec<PostProcessStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub repository: Option<String>,
    pub license: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub description: String,
    pub default: Option<String>,
    pub required: bool,
    pub validation: Option<String>, // Regex pattern
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    pub description: String,
    pub default: bool,
    pub depends_on: Vec<String>,
    pub conflicts_with: Vec<String>,
    pub includes: Vec<String>,              // File patterns to include
    pub excludes: Vec<String>,              // File patterns to exclude
    pub variables: HashMap<String, String>, // Additional variables when enabled
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMapping {
    pub from: String,
    pub to: String,
    pub template: bool,                // Whether to process as template
    pub features: Option<Vec<String>>, // Required features
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PostProcessStep {
    Command {
        command: String,
        args: Vec<String>,
        working_dir: Option<String>,
    },
    GitInit {
        initial_commit: bool,
        message: Option<String>,
    },
    RemoveFiles {
        patterns: Vec<String>,
    },
}

impl TemplateConfig {
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: TemplateConfig = serde_json::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<()> {
        // Validate feature dependencies and conflicts
        for (name, feature) in &self.features {
            // Check dependencies exist
            for dep in &feature.depends_on {
                if !self.features.contains_key(dep) {
                    return Err(TemplateError::Config(format!(
                        "Feature '{name}' depends on unknown feature '{dep}'"
                    )));
                }
            }

            // Check conflicts exist
            for conflict in &feature.conflicts_with {
                if !self.features.contains_key(conflict) {
                    return Err(TemplateError::Config(format!(
                        "Feature '{name}' conflicts with unknown feature '{conflict}'"
                    )));
                }
            }

            // Check for circular dependencies
            if self.has_circular_dependency(name, &mut HashSet::new())? {
                return Err(TemplateError::Config(format!(
                    "Feature '{name}' has circular dependencies"
                )));
            }
        }

        Ok(())
    }

    fn has_circular_dependency(
        &self,
        feature: &str,
        visited: &mut HashSet<String>,
    ) -> Result<bool> {
        if visited.contains(feature) {
            return Ok(true);
        }

        visited.insert(feature.to_string());

        if let Some(f) = self.features.get(feature) {
            for dep in &f.depends_on {
                if self.has_circular_dependency(dep, visited)? {
                    return Ok(true);
                }
            }
        }

        visited.remove(feature);
        Ok(false)
    }

    pub fn resolve_features(&self, selected: &[String]) -> Result<HashSet<String>> {
        let mut resolved = HashSet::new();
        let mut to_process: Vec<String> = selected.to_vec();

        while let Some(feature) = to_process.pop() {
            if resolved.contains(&feature) {
                continue;
            }

            let f = self
                .features
                .get(&feature)
                .ok_or_else(|| TemplateError::Config(format!("Unknown feature: {feature}")))?;

            // Check conflicts
            for conflict in &f.conflicts_with {
                if resolved.contains(conflict) || to_process.contains(conflict) {
                    return Err(TemplateError::FeatureConflict(format!(
                        "Feature '{feature}' conflicts with '{conflict}'"
                    )));
                }
            }

            // Add dependencies
            for dep in &f.depends_on {
                if !resolved.contains(dep) {
                    to_process.push(dep.clone());
                }
            }

            resolved.insert(feature);
        }

        Ok(resolved)
    }
}
