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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_name() {
        assert_eq!(
            AiPersona::BusinessAnalyst.display_name(),
            "Business Analyst"
        );
        assert_eq!(
            AiPersona::TechnicalSupport.display_name(),
            "Technical Support"
        );
        assert_eq!(AiPersona::CodeReviewer.display_name(), "Code Reviewer");
        assert_eq!(AiPersona::ProjectManager.display_name(), "Project Manager");
    }

    #[test]
    fn test_template_name() {
        assert_eq!(
            AiPersona::BusinessAnalyst.template_name(),
            "business_analyst_conversation"
        );
        assert_eq!(
            AiPersona::TechnicalSupport.template_name(),
            "technical_support_conversation"
        );
        assert_eq!(
            AiPersona::CodeReviewer.template_name(),
            "code_reviewer_conversation"
        );
        assert_eq!(
            AiPersona::ProjectManager.template_name(),
            "project_manager_conversation"
        );
    }

    #[test]
    fn test_supports_functions() {
        assert!(AiPersona::BusinessAnalyst.supports_functions());
        assert!(AiPersona::ProjectManager.supports_functions());
        assert!(!AiPersona::TechnicalSupport.supports_functions());
        assert!(!AiPersona::CodeReviewer.supports_functions());
    }

    #[test]
    fn test_default() {
        assert_eq!(AiPersona::default(), AiPersona::BusinessAnalyst);
    }

    #[test]
    fn test_equality() {
        assert_eq!(AiPersona::BusinessAnalyst, AiPersona::BusinessAnalyst);
        assert_ne!(AiPersona::BusinessAnalyst, AiPersona::TechnicalSupport);
    }

    #[test]
    fn test_clone() {
        let persona = AiPersona::ProjectManager;
        #[allow(clippy::clone_on_copy)]
        let cloned = persona.clone();
        assert_eq!(persona, cloned);
    }

    #[test]
    fn test_copy() {
        let persona = AiPersona::CodeReviewer;
        let copied = persona;
        assert_eq!(persona, copied);
    }

    #[test]
    fn test_debug() {
        let persona = AiPersona::TechnicalSupport;
        let debug_str = format!("{persona:?}");
        assert_eq!(debug_str, "TechnicalSupport");
    }

    #[test]
    fn test_serialize() {
        let persona = AiPersona::BusinessAnalyst;
        let serialized = serde_json::to_string(&persona).expect("Failed to serialize AiPersona");
        assert_eq!(serialized, r#""business_analyst""#);
    }

    #[test]
    fn test_deserialize() {
        let json = r#""project_manager""#;
        let persona: AiPersona =
            serde_json::from_str(json).expect("Failed to deserialize AiPersona");
        assert_eq!(persona, AiPersona::ProjectManager);
    }

    #[test]
    fn test_serialize_deserialize_all_variants() {
        let personas = vec![
            AiPersona::BusinessAnalyst,
            AiPersona::TechnicalSupport,
            AiPersona::CodeReviewer,
            AiPersona::ProjectManager,
        ];

        for persona in personas {
            let serialized =
                serde_json::to_string(&persona).expect("Failed to serialize AiPersona");
            let deserialized: AiPersona =
                serde_json::from_str(&serialized).expect("Failed to deserialize AiPersona");
            assert_eq!(persona, deserialized);
        }
    }
}
