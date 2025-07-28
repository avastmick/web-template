//! Typed response models for structured AI outputs

use serde::{Deserialize, Serialize};

/// Response from issue analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueAnalysis {
    pub title: String,
    pub description: String,
    pub acceptance_criteria: Vec<String>,
    #[serde(default)]
    pub technical_considerations: Vec<String>,
    pub estimated_hours: f32,
    pub priority: Priority,
    #[serde(default)]
    pub tags: Vec<String>,
    pub reasoning: String,
}

/// Priority level for issues
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

/// Response from suggestion generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueSuggestions {
    pub suggestions: Vec<Suggestion>,
    pub overall_quality: QualityAssessment,
    pub reasoning: String,
}

/// Individual suggestion for improvement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub category: SuggestionCategory,
    pub suggestion: String,
    pub rationale: String,
    pub example: String,
}

/// Category of suggestion
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SuggestionCategory {
    Clarity,
    Completeness,
    Technical,
    Scope,
    AcceptanceCriteria,
}

/// Overall quality assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityAssessment {
    pub score: u8,
    pub strengths: Vec<String>,
    pub improvements_needed: Vec<String>,
}

/// Response from effort estimation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffortEstimation {
    pub estimated_hours: f32,
    pub confidence_level: ConfidenceLevel,
    pub complexity_factors: Vec<ComplexityFactor>,
    pub breakdown: Vec<TaskBreakdown>,
    pub requires_splitting: bool,
    pub splitting_suggestions: Vec<SplitSuggestion>,
    pub reasoning: String,
}

/// Confidence level in estimate
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ConfidenceLevel {
    Low,
    Medium,
    High,
}

/// Factor contributing to complexity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityFactor {
    pub factor: String,
    pub impact: ImpactLevel,
    pub description: String,
}

/// Impact level
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ImpactLevel {
    Low,
    Medium,
    High,
}

/// Breakdown of task into subtasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskBreakdown {
    pub task: String,
    pub hours: f32,
}

/// Suggestion for splitting a large issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitSuggestion {
    pub title: String,
    pub description: String,
    pub estimated_hours: f32,
}

/// Response from duplicate detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateDetection {
    pub similar_issues: Vec<SimilarIssue>,
    pub has_duplicates: bool,
    pub summary: String,
}

/// Information about a similar issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarIssue {
    pub issue_id: String,
    pub similarity_level: SimilarityLevel,
    pub similarity_score: f32,
    pub overlapping_aspects: Vec<String>,
    pub recommendation: DuplicateRecommendation,
    pub reasoning: String,
}

/// Level of similarity between issues
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum SimilarityLevel {
    Duplicate,
    High,
    Medium,
    Low,
}

/// Recommended action for duplicate/similar issues
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DuplicateRecommendation {
    Merge,
    Link,
    KeepSeparate,
    CloseAsDuplicate,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_issue_analysis_deserialization() -> Result<(), Box<dyn std::error::Error>> {
        let json = json!({
            "title": "Add user authentication",
            "description": "Implement secure user authentication system",
            "acceptance_criteria": ["Users can register", "Users can login"],
            "estimated_hours": 3.5,
            "priority": "high",
            "reasoning": "Essential security feature"
        });

        let analysis: IssueAnalysis = serde_json::from_value(json)?;
        assert_eq!(analysis.title, "Add user authentication");
        assert_eq!(analysis.priority, Priority::High);
        assert!((analysis.estimated_hours - 3.5).abs() < f32::EPSILON);
        Ok(())
    }

    #[test]
    fn test_similarity_level_serialization() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            serde_json::to_string(&SimilarityLevel::Duplicate)?,
            "\"DUPLICATE\""
        );
        assert_eq!(serde_json::to_string(&SimilarityLevel::High)?, "\"HIGH\"");
        Ok(())
    }
}
