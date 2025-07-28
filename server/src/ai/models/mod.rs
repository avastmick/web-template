//! AI-specific data models

pub mod analysis;
pub mod chat;
pub mod responses;
pub mod usage;

// Re-export all types from chat module for backward compatibility
pub use chat::{
    ChatChoice, ChatMessage, ChatRequest, ChatResponse, ChatRole, ResponseFormat, StreamEvent,
};
pub use responses::{
    ComplexityFactor, ConfidenceLevel, DuplicateDetection, DuplicateRecommendation,
    EffortEstimation, ImpactLevel, IssueAnalysis, IssueSuggestions, Priority, QualityAssessment,
    SimilarIssue, SimilarityLevel, SplitSuggestion, Suggestion, SuggestionCategory, TaskBreakdown,
};
pub use usage::TokenUsage;
