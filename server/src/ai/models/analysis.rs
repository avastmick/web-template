//! Analysis result models for AI services

use serde::{Deserialize, Serialize};

/// Result of AI issue analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueAnalysis {
    /// Suggested issue title
    pub title: Option<String>,

    /// Enhanced description with acceptance criteria
    pub enhanced_description: String,

    /// Suggested issue type
    pub suggested_type: Option<IssueType>,

    /// Suggested priority based on impact
    pub suggested_priority: Option<IssuePriority>,

    /// List of clarifying questions if more info needed
    pub clarifying_questions: Vec<String>,

    /// Technical considerations identified
    pub technical_notes: Vec<String>,

    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
}

/// Simplified issue type for AI analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueType {
    Feature,
    Enhancement,
    Bug,
    Task,
}

/// Simplified issue priority for AI analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssuePriority {
    Critical,
    High,
    Medium,
    Low,
}

/// Result of duplicate detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateCheckResult {
    /// List of potentially duplicate issues
    pub duplicates: Vec<DuplicateIssue>,

    /// Overall uniqueness score (0.0 = exact duplicate, 1.0 = completely unique)
    pub uniqueness_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateIssue {
    pub issue_id: String,
    pub title: String,
    pub similarity_score: f32,
    pub status: String,
}

/// Result of issue sizing analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueSizeEstimate {
    /// Estimated hours to complete
    pub estimated_hours: f32,

    /// Confidence in the estimate (0.0 - 1.0)
    pub confidence: f32,

    /// Factors that influenced the estimate
    pub factors: Vec<String>,

    /// Suggestions for breaking down if too large
    pub breakdown_suggestions: Vec<String>,

    /// Whether the issue should be split
    pub should_split: bool,
}
