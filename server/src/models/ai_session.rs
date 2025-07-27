//! AI session models for conversational issue creation

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::ai_persona::AiPersona;

/// Session status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    #[sqlx(rename = "active")]
    Active,
    #[sqlx(rename = "completed")]
    Completed,
    #[sqlx(rename = "cancelled")]
    Cancelled,
    #[sqlx(rename = "expired")]
    Expired,
}

/// Message role in conversation
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    #[sqlx(rename = "user")]
    User,
    #[sqlx(rename = "assistant")]
    Assistant,
    #[sqlx(rename = "function")]
    Function,
}

/// AI session for conversational interactions
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AiSession {
    pub id: String,
    pub user_id: String,
    #[serde(rename = "type")]
    #[sqlx(rename = "type")]
    pub status: SessionStatus,
    pub context: String,       // JSON string
    pub draft: Option<String>, // JSON string
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
}

/// AI session message
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AiSessionMessage {
    pub id: String,
    pub session_id: String,
    pub role: MessageRole,
    pub content: String,
    pub function_call: Option<String>, // JSON string
    pub created_at: DateTime<Utc>,
}

/// AI session asset (uploaded file)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AiSessionAsset {
    pub id: String,
    pub session_id: String,
    pub file_path: String,
    pub content_type: String,
    pub size: i32,
    pub metadata: Option<String>, // JSON string
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Request to create a new AI session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    pub initial_input: Option<String>,
    pub board_id: Option<String>, // For issue creation context
    #[serde(default)]
    pub persona: AiPersona, // AI persona to use for the conversation
}

/// Response after creating a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionResponse {
    pub session_id: String,
    pub status: SessionStatus,
    pub assistant_response: String,
    pub expires_at: DateTime<Utc>,
}

/// Request to send a message in a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub message: String,
}

/// Response after sending a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageResponse {
    pub message_id: String,
    pub assistant_response: String,
    pub function_calls: Vec<FunctionCall>,
    pub draft_updated: bool,
}

/// Function call made by the assistant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub parameters: serde_json::Value,
}

/// Session context structure (stored as JSON)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionContext {
    #[serde(default)]
    pub requirements: Vec<String>,
    #[serde(default)]
    pub technical_notes: Vec<String>,
    #[serde(default)]
    pub design_notes: Vec<String>,
    #[serde(default)]
    pub other_notes: Vec<String>,
    #[serde(default)]
    pub board_id: Option<String>,
    #[serde(default)]
    pub persona: AiPersona,
}

/// Issue draft structure (stored as JSON)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IssueDraft {
    pub title: Option<String>,
    pub description: Option<String>,
    pub acceptance_criteria: Vec<String>,
    pub technical_notes: Option<String>,
    pub assets: Vec<String>, // Asset IDs
    pub estimated_hours: Option<f32>,
    pub priority: Option<String>,
    pub tags: Vec<String>,
}

/// Request to finalize a session and create an issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalizeSessionRequest {
    pub action: FinalizeAction,
    pub board_id: String,
}

/// Actions available when finalizing a session
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FinalizeAction {
    CreateIssue,
    Cancel,
}

/// Response after finalizing a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalizeSessionResponse {
    pub issue_id: Option<String>,
    pub board_id: Option<String>,
    pub status: SessionStatus,
    pub url: Option<String>,
    pub assets_moved: usize,
}

/// Preview of the issue to be created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuePreview {
    pub title: String,
    pub description: String,
    pub acceptance_criteria: Vec<String>,
    pub technical_notes: Option<String>,
    pub estimated_hours: Option<f32>,
    pub priority: String,
    pub tags: Vec<String>,
    pub asset_count: usize,
}

impl AiSession {
    /// Create a new AI session
    ///
    /// # Panics
    ///
    /// Panics if the default `SessionContext` cannot be serialized to JSON (should never happen)
    #[must_use]
    pub fn new(user_id: String) -> Self {
        let now = Utc::now();
        Self {
            id: format!("sess_{}", Uuid::new_v4()),
            user_id,
            status: SessionStatus::Active,
            context: serde_json::to_string(&SessionContext::default())
                .expect("Failed to serialize default context"),
            draft: None,
            created_at: now,
            updated_at: now,
            completed_at: None,
            expires_at: now + chrono::Duration::minutes(30), // 30 minute expiry
        }
    }

    /// Parse context from JSON
    ///
    /// # Errors
    ///
    /// Returns `serde_json::Error` if the context JSON is invalid
    pub fn get_context(&self) -> Result<SessionContext, serde_json::Error> {
        serde_json::from_str(&self.context)
    }

    /// Parse draft from JSON
    ///
    /// # Errors
    ///
    /// Returns `serde_json::Error` if the draft JSON is invalid
    pub fn get_draft(&self) -> Result<Option<IssueDraft>, serde_json::Error> {
        match &self.draft {
            Some(draft_json) => Ok(Some(serde_json::from_str(draft_json)?)),
            None => Ok(None),
        }
    }
}

impl AiSessionMessage {
    /// Create a new message
    #[must_use]
    pub fn new(session_id: String, role: MessageRole, content: String) -> Self {
        Self {
            id: format!("msg_{}", Uuid::new_v4()),
            session_id,
            role,
            content,
            function_call: None,
            created_at: Utc::now(),
        }
    }
}

impl AiSessionAsset {
    /// Create a new asset
    #[must_use]
    pub fn new(
        session_id: String,
        file_path: String,
        content_type: String,
        size: i32,
        description: Option<String>,
    ) -> Self {
        Self {
            id: format!("asset_{}", Uuid::new_v4()),
            session_id,
            file_path,
            content_type,
            size,
            metadata: None,
            description,
            created_at: Utc::now(),
        }
    }
}
