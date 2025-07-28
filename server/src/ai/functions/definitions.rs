//! Function definitions for AI assistant capabilities
//!
//! Defines the structured function calls that AI assistants can make
//! to interact with the system during conversations.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// OpenAI-compatible function definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    /// Function name (e.g., `save_context`, `link_asset`)
    pub name: String,
    /// Human-readable description of what the function does
    pub description: String,
    /// JSON Schema for the function parameters
    pub parameters: FunctionParameter,
}

/// Function parameter schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionParameter {
    /// Always "object" for function parameters
    #[serde(rename = "type")]
    pub param_type: String,
    /// Map of parameter name to parameter schema
    pub properties: HashMap<String, ParameterProperty>,
    /// List of required parameter names
    pub required: Vec<String>,
}

/// Individual parameter property schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterProperty {
    /// Parameter type (string, number, boolean, array, object)
    #[serde(rename = "type")]
    pub prop_type: String,
    /// Human-readable description
    pub description: String,
    /// Enum values if applicable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#enum: Option<Vec<String>>,
    /// Items schema for arrays
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<ParameterProperty>>,
}

/// Structured function calls the AI can make
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "name", content = "arguments")]
pub enum FunctionCall {
    /// Save important context from the conversation
    #[serde(rename = "save_context")]
    SaveContext {
        /// Type of context being saved
        context_type: ContextType,
        /// The content to save
        content: String,
    },

    /// Link an uploaded asset to the issue draft
    #[serde(rename = "link_asset")]
    LinkAsset {
        /// ID of the asset to link
        asset_id: String,
        /// Description of why this asset is relevant
        description: String,
    },

    /// Update the issue draft with new information
    #[serde(rename = "update_issue_draft")]
    UpdateIssueDraft {
        /// Fields to update in the draft
        updates: IssueDraftUpdate,
    },

    /// Create the issue from the current draft
    #[serde(rename = "create_issue")]
    CreateIssue {
        /// Whether to create the issue immediately or preview first
        confirm: bool,
    },
}

/// Types of context that can be saved
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContextType {
    /// Business requirements and user needs
    Requirements,
    /// Technical implementation details
    Technical,
    /// Design decisions and rationale
    Design,
    /// General notes and observations
    Notes,
}

/// Partial update to an issue draft
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IssueDraftUpdate {
    /// Issue title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Issue description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Acceptance criteria (replaces existing if provided)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acceptance_criteria: Option<Vec<String>>,

    /// Technical notes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub technical_notes: Option<String>,

    /// Estimated hours
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_hours: Option<f32>,

    /// Priority (low, medium, high, critical)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,

    /// Tags to add
    #[serde(skip_serializing_if = "Option::is_none")]
    pub add_tags: Option<Vec<String>>,

    /// Tags to remove
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove_tags: Option<Vec<String>>,
}

/// Result of a function call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionResult {
    /// Whether the function call succeeded
    pub success: bool,
    /// Result message to show to the user
    pub message: String,
    /// Additional data if needed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Get function definitions for the Business Analyst assistant
#[must_use]
pub fn get_business_analyst_functions() -> Vec<FunctionDefinition> {
    vec![
        FunctionDefinition {
            name: "save_context".to_string(),
            description: "Save important context from the conversation for later reference".to_string(),
            parameters: FunctionParameter {
                param_type: "object".to_string(),
                properties: HashMap::from([
                    ("context_type".to_string(), ParameterProperty {
                        prop_type: "string".to_string(),
                        description: "Type of context being saved".to_string(),
                        r#enum: Some(vec![
                            "requirements".to_string(),
                            "technical".to_string(),
                            "design".to_string(),
                            "notes".to_string(),
                        ]),
                        items: None,
                    }),
                    ("content".to_string(), ParameterProperty {
                        prop_type: "string".to_string(),
                        description: "The context content to save".to_string(),
                        r#enum: None,
                        items: None,
                    }),
                ]),
                required: vec!["context_type".to_string(), "content".to_string()],
            },
        },

        FunctionDefinition {
            name: "link_asset".to_string(),
            description: "Link an uploaded file or image to the issue being created".to_string(),
            parameters: FunctionParameter {
                param_type: "object".to_string(),
                properties: HashMap::from([
                    ("asset_id".to_string(), ParameterProperty {
                        prop_type: "string".to_string(),
                        description: "ID of the uploaded asset".to_string(),
                        r#enum: None,
                        items: None,
                    }),
                    ("description".to_string(), ParameterProperty {
                        prop_type: "string".to_string(),
                        description: "Description of why this asset is relevant to the issue".to_string(),
                        r#enum: None,
                        items: None,
                    }),
                ]),
                required: vec!["asset_id".to_string(), "description".to_string()],
            },
        },

        FunctionDefinition {
            name: "update_issue_draft".to_string(),
            description: "Update the draft issue with new or modified information".to_string(),
            parameters: FunctionParameter {
                param_type: "object".to_string(),
                properties: HashMap::from([
                    ("updates".to_string(), ParameterProperty {
                        prop_type: "object".to_string(),
                        description: "Object containing the fields to update".to_string(),
                        r#enum: None,
                        items: None,
                    }),
                ]),
                required: vec!["updates".to_string()],
            },
        },

        FunctionDefinition {
            name: "create_issue".to_string(),
            description: "Create the issue from the current draft when all information is complete".to_string(),
            parameters: FunctionParameter {
                param_type: "object".to_string(),
                properties: HashMap::from([
                    ("confirm".to_string(), ParameterProperty {
                        prop_type: "boolean".to_string(),
                        description: "Whether to create the issue immediately (true) or show a preview first (false)".to_string(),
                        r#enum: None,
                        items: None,
                    }),
                ]),
                required: vec!["confirm".to_string()],
            },
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_definitions() {
        let functions = get_business_analyst_functions();
        assert_eq!(functions.len(), 4);

        // Check function names
        let names: Vec<_> = functions.iter().map(|f| &f.name).collect();
        assert!(names.contains(&&"save_context".to_string()));
        assert!(names.contains(&&"link_asset".to_string()));
        assert!(names.contains(&&"update_issue_draft".to_string()));
        assert!(names.contains(&&"create_issue".to_string()));
    }

    #[test]
    fn test_context_type_serialization() {
        assert_eq!(
            serde_json::to_string(&ContextType::Requirements).expect("serialize should work"),
            "\"requirements\""
        );
        assert_eq!(
            serde_json::to_string(&ContextType::Technical).expect("serialize should work"),
            "\"technical\""
        );
    }

    #[test]
    fn test_function_call_serialization() {
        let call = FunctionCall::SaveContext {
            context_type: ContextType::Requirements,
            content: "User needs feature X".to_string(),
        };

        let json = serde_json::to_string(&call).expect("serialize should work");
        assert!(json.contains("save_context"));
        assert!(json.contains("requirements"));
        assert!(json.contains("User needs feature X"));
    }

    #[test]
    fn test_issue_draft_update_partial() {
        let update = IssueDraftUpdate {
            title: Some("New Title".to_string()),
            description: None,
            ..Default::default()
        };

        let json = serde_json::to_string(&update).expect("serialize should work");
        assert!(json.contains("title"));
        assert!(!json.contains("description"));
        assert!(!json.contains("acceptance_criteria"));
    }
}
