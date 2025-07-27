//! AI Persona definitions for different assistant types

use serde::{Deserialize, Serialize};

/// Available AI personas for different conversation contexts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiPersona {
    /// Business Analyst for requirements gathering and issue creation
    BusinessAnalyst,
    /// Technical Support for troubleshooting
    TechnicalSupport,
    /// Code Reviewer for code analysis
    CodeReviewer,
    /// Project Manager for planning and estimation
    ProjectManager,
}

impl AiPersona {
    /// Get the persona's display name
    #[must_use]
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::BusinessAnalyst => "Business Analyst",
            Self::TechnicalSupport => "Technical Support",
            Self::CodeReviewer => "Code Reviewer",
            Self::ProjectManager => "Project Manager",
        }
    }

    /// Get the persona identifier for template selection
    #[must_use]
    pub fn template_name(&self) -> &'static str {
        match self {
            Self::BusinessAnalyst => "business_analyst_conversation",
            Self::TechnicalSupport => "technical_support_conversation",
            Self::CodeReviewer => "code_reviewer_conversation",
            Self::ProjectManager => "project_manager_conversation",
        }
    }

    /// Check if this persona supports function calling
    #[must_use]
    pub fn supports_functions(&self) -> bool {
        match self {
            Self::BusinessAnalyst | Self::ProjectManager => true,
            Self::TechnicalSupport | Self::CodeReviewer => false,
        }
    }
}

impl Default for AiPersona {
    fn default() -> Self {
        Self::BusinessAnalyst
    }
}
