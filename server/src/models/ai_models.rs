//! AI-related database models

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AiConversation {
    pub id: String,      // UUID as string
    pub user_id: String, // UUID as string, references users.id
    pub title: Option<String>,
    pub model: String,
    pub system_prompt: Option<String>,
    pub created_at: String,          // ISO8601 timestamp
    pub updated_at: String,          // ISO8601 timestamp
    pub archived_at: Option<String>, // ISO8601 timestamp for soft delete
    pub metadata: Option<String>,    // JSON string
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AiMessage {
    pub id: String,              // UUID as string
    pub conversation_id: String, // UUID as string, references ai_conversations.id
    pub role: String,            // 'system', 'user', 'assistant'
    pub content: String,
    pub token_count: Option<i64>,
    pub created_at: String,       // ISO8601 timestamp
    pub metadata: Option<String>, // JSON string
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AiUsage {
    pub id: String,                      // UUID as string
    pub conversation_id: Option<String>, // UUID as string, references ai_conversations.id
    pub user_id: String,                 // UUID as string, references users.id
    pub model: String,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub total_tokens: i64,
    pub cost_cents: Option<i64>,
    pub created_at: String, // ISO8601 timestamp
    pub request_id: Option<String>,
    pub duration_ms: Option<i64>,
}

// DTOs for API requests/responses
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateConversationRequest {
    pub title: Option<String>,
    pub model: String,
    pub system_prompt: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConversationResponse {
    pub id: String,
    pub title: Option<String>,
    pub model: String,
    pub system_prompt: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub message_count: Option<i64>,
    pub last_message_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageResponse {
    pub id: String,
    pub role: String,
    pub content: String,
    pub token_count: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConversationWithMessages {
    pub conversation: ConversationResponse,
    pub messages: Vec<MessageResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateMessageRequest {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsageStatsResponse {
    pub total_conversations: i64,
    pub total_messages: i64,
    pub total_tokens: i64,
    pub total_cost_cents: Option<i64>,
    pub models_used: Vec<String>,
}

// Implementations for conversions and utility methods
impl AiConversation {
    #[must_use]
    pub fn new(user_id: String, model: String) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            title: None,
            model,
            system_prompt: None,
            created_at: now.clone(),
            updated_at: now,
            archived_at: None,
            metadata: None,
        }
    }

    #[must_use]
    pub fn with_title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    #[must_use]
    pub fn with_system_prompt(mut self, system_prompt: String) -> Self {
        self.system_prompt = Some(system_prompt);
        self
    }

    pub fn archive(&mut self) {
        self.archived_at = Some(Utc::now().to_rfc3339());
        self.updated_at = Utc::now().to_rfc3339();
    }

    pub fn update_timestamp(&mut self) {
        self.updated_at = Utc::now().to_rfc3339();
    }
}

impl AiMessage {
    #[must_use]
    pub fn new(conversation_id: String, role: String, content: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            conversation_id,
            role,
            content,
            token_count: None,
            created_at: Utc::now().to_rfc3339(),
            metadata: None,
        }
    }

    #[must_use]
    pub fn with_token_count(mut self, token_count: i64) -> Self {
        self.token_count = Some(token_count);
        self
    }
}

impl AiUsage {
    #[must_use]
    pub fn new(user_id: String, model: String, prompt_tokens: i64, completion_tokens: i64) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            conversation_id: None,
            user_id,
            model,
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
            cost_cents: None,
            created_at: Utc::now().to_rfc3339(),
            request_id: None,
            duration_ms: None,
        }
    }

    #[must_use]
    pub fn with_conversation(mut self, conversation_id: String) -> Self {
        self.conversation_id = Some(conversation_id);
        self
    }

    #[must_use]
    pub fn with_cost(mut self, cost_cents: i64) -> Self {
        self.cost_cents = Some(cost_cents);
        self
    }

    #[must_use]
    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }

    #[must_use]
    pub fn with_duration(mut self, duration_ms: i64) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }
}
