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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_status_serialization() {
        assert_eq!(
            serde_json::to_string(&SessionStatus::Active).expect("Failed to serialize"),
            r#""active""#
        );
        assert_eq!(
            serde_json::to_string(&SessionStatus::Completed).expect("Failed to serialize"),
            r#""completed""#
        );
        assert_eq!(
            serde_json::to_string(&SessionStatus::Cancelled).expect("Failed to serialize"),
            r#""cancelled""#
        );
        assert_eq!(
            serde_json::to_string(&SessionStatus::Expired).expect("Failed to serialize"),
            r#""expired""#
        );
    }

    #[test]
    fn test_session_status_deserialization() {
        assert_eq!(
            serde_json::from_str::<SessionStatus>(r#""active""#).expect("Failed to deserialize"),
            SessionStatus::Active
        );
        assert_eq!(
            serde_json::from_str::<SessionStatus>(r#""completed""#).expect("Failed to deserialize"),
            SessionStatus::Completed
        );
    }

    #[test]
    fn test_message_role_serialization() {
        assert_eq!(
            serde_json::to_string(&MessageRole::User).expect("Failed to serialize"),
            r#""user""#
        );
        assert_eq!(
            serde_json::to_string(&MessageRole::Assistant).expect("Failed to serialize"),
            r#""assistant""#
        );
        assert_eq!(
            serde_json::to_string(&MessageRole::Function).expect("Failed to serialize"),
            r#""function""#
        );
    }

    #[test]
    fn test_ai_session_new() {
        let user_id = "user_123".to_string();
        let session = AiSession::new(user_id.clone());

        assert!(session.id.starts_with("sess_"));
        assert_eq!(session.user_id, user_id);
        assert_eq!(session.status, SessionStatus::Active);
        assert!(session.draft.is_none());
        assert!(session.completed_at.is_none());
        assert!(session.expires_at > session.created_at);
    }

    #[test]
    fn test_ai_session_get_context() {
        let session = AiSession::new("user_123".to_string());
        let context = session.get_context().expect("Failed to get context");

        assert!(context.requirements.is_empty());
        assert!(context.technical_notes.is_empty());
        assert!(context.design_notes.is_empty());
        assert!(context.other_notes.is_empty());
        assert!(context.board_id.is_none());
        assert_eq!(context.persona, AiPersona::BusinessAnalyst);
    }

    #[test]
    fn test_ai_session_get_draft_none() {
        let session = AiSession::new("user_123".to_string());
        let draft = session.get_draft().expect("Failed to get draft");
        assert!(draft.is_none());
    }

    #[test]
    fn test_ai_session_get_draft_some() {
        let mut session = AiSession::new("user_123".to_string());
        let draft = IssueDraft {
            title: Some("Test Issue".to_string()),
            description: Some("Test Description".to_string()),
            acceptance_criteria: vec!["AC1".to_string()],
            technical_notes: None,
            assets: vec![],
            estimated_hours: Some(3.5),
            priority: Some("high".to_string()),
            tags: vec!["test".to_string()],
        };
        session.draft = Some(serde_json::to_string(&draft).expect("Failed to serialize draft"));

        let retrieved_draft = session.get_draft().expect("Failed to get draft");
        assert!(retrieved_draft.is_some());
        let retrieved_draft = retrieved_draft.expect("Draft should exist");
        assert_eq!(retrieved_draft.title, Some("Test Issue".to_string()));
        assert_eq!(retrieved_draft.estimated_hours, Some(3.5));
    }

    #[test]
    fn test_ai_session_message_new() {
        let session_id = "sess_123".to_string();
        let content = "Hello AI".to_string();
        let message = AiSessionMessage::new(session_id.clone(), MessageRole::User, content.clone());

        assert!(message.id.starts_with("msg_"));
        assert_eq!(message.session_id, session_id);
        assert_eq!(message.role, MessageRole::User);
        assert_eq!(message.content, content);
        assert!(message.function_call.is_none());
    }

    #[test]
    fn test_ai_session_asset_new() {
        let session_id = "sess_123".to_string();
        let file_path = "/tmp/file.pdf".to_string();
        let content_type = "application/pdf".to_string();
        let size = 1024;
        let description = Some("Test PDF".to_string());

        let asset = AiSessionAsset::new(
            session_id.clone(),
            file_path.clone(),
            content_type.clone(),
            size,
            description.clone(),
        );

        assert!(asset.id.starts_with("asset_"));
        assert_eq!(asset.session_id, session_id);
        assert_eq!(asset.file_path, file_path);
        assert_eq!(asset.content_type, content_type);
        assert_eq!(asset.size, size);
        assert_eq!(asset.description, description);
        assert!(asset.metadata.is_none());
    }

    #[test]
    fn test_create_session_request_serialization() {
        let request = CreateSessionRequest {
            initial_input: Some("Create a login feature".to_string()),
            board_id: Some("board_123".to_string()),
            persona: AiPersona::BusinessAnalyst,
        };

        let json = serde_json::to_string(&request).expect("Failed to serialize");
        assert!(json.contains("initial_input"));
        assert!(json.contains("board_id"));
        assert!(json.contains("persona"));
    }

    #[test]
    fn test_session_context_default() {
        let context = SessionContext::default();
        assert!(context.requirements.is_empty());
        assert!(context.technical_notes.is_empty());
        assert!(context.design_notes.is_empty());
        assert!(context.other_notes.is_empty());
        assert!(context.board_id.is_none());
        assert_eq!(context.persona, AiPersona::BusinessAnalyst);
    }

    #[test]
    fn test_issue_draft_default() {
        let draft = IssueDraft::default();
        assert!(draft.title.is_none());
        assert!(draft.description.is_none());
        assert!(draft.acceptance_criteria.is_empty());
        assert!(draft.technical_notes.is_none());
        assert!(draft.assets.is_empty());
        assert!(draft.estimated_hours.is_none());
        assert!(draft.priority.is_none());
        assert!(draft.tags.is_empty());
    }

    #[test]
    fn test_finalize_action_serialization() {
        assert_eq!(
            serde_json::to_string(&FinalizeAction::CreateIssue).expect("Failed to serialize"),
            r#""create_issue""#
        );
        assert_eq!(
            serde_json::to_string(&FinalizeAction::Cancel).expect("Failed to serialize"),
            r#""cancel""#
        );
    }

    #[test]
    fn test_function_call_serialization() {
        let func_call = FunctionCall {
            name: "create_issue".to_string(),
            parameters: serde_json::json!({
                "title": "Test Issue",
                "priority": "high"
            }),
        };

        let json = serde_json::to_string(&func_call).expect("Failed to serialize");
        assert!(json.contains("create_issue"));
        assert!(json.contains("Test Issue"));
    }

    #[test]
    fn test_issue_preview_serialization() {
        let preview = IssuePreview {
            title: "Test Issue".to_string(),
            description: "Test Description".to_string(),
            acceptance_criteria: vec!["AC1".to_string(), "AC2".to_string()],
            technical_notes: Some("Tech notes".to_string()),
            estimated_hours: Some(5.0),
            priority: "medium".to_string(),
            tags: vec!["backend".to_string()],
            asset_count: 2,
        };

        let json = serde_json::to_string(&preview).expect("Failed to serialize");
        let deserialized: IssuePreview =
            serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(deserialized.title, preview.title);
        assert_eq!(deserialized.acceptance_criteria.len(), 2);
        assert_eq!(deserialized.asset_count, 2);
    }

    #[test]
    fn test_ai_session_expiry_duration() {
        let session = AiSession::new("user_123".to_string());
        let duration = session.expires_at - session.created_at;
        assert_eq!(duration.num_minutes(), 30);
    }

    #[test]
    fn test_session_status_equality() {
        assert_eq!(SessionStatus::Active, SessionStatus::Active);
        assert_ne!(SessionStatus::Active, SessionStatus::Completed);
    }

    #[test]
    fn test_message_role_equality() {
        assert_eq!(MessageRole::User, MessageRole::User);
        assert_ne!(MessageRole::User, MessageRole::Assistant);
    }

    #[test]
    fn test_create_session_response_serialization() {
        let response = CreateSessionResponse {
            session_id: "sess_123".to_string(),
            status: SessionStatus::Active,
            assistant_response: "Hello! How can I help?".to_string(),
            expires_at: Utc::now() + chrono::Duration::minutes(30),
        };

        let json = serde_json::to_string(&response).expect("Failed to serialize");
        assert!(json.contains("sess_123"));
        assert!(json.contains("active"));
    }

    #[test]
    fn test_send_message_response_with_function_calls() {
        let response = SendMessageResponse {
            message_id: "msg_123".to_string(),
            assistant_response: "I'll help you with that.".to_string(),
            function_calls: vec![FunctionCall {
                name: "analyze_requirements".to_string(),
                parameters: serde_json::json!({"text": "login feature"}),
            }],
            draft_updated: true,
        };

        let json = serde_json::to_string(&response).expect("Failed to serialize");
        assert!(json.contains("analyze_requirements"));
        assert!(json.contains("draft_updated"));
    }

    #[test]
    fn test_finalize_session_response_serialization() {
        let response = FinalizeSessionResponse {
            issue_id: Some("issue_123".to_string()),
            board_id: Some("board_456".to_string()),
            status: SessionStatus::Completed,
            url: Some("https://example.com/issue/123".to_string()),
            assets_moved: 3,
        };

        let json = serde_json::to_string(&response).expect("Failed to serialize");
        let deserialized: FinalizeSessionResponse =
            serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(deserialized.assets_moved, 3);
        assert_eq!(deserialized.status, SessionStatus::Completed);
    }
}
