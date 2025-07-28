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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_conversation_new() {
        let user_id = "user_123".to_string();
        let model = "gpt-4".to_string();
        let conversation = AiConversation::new(user_id.clone(), model.clone());

        assert!(!conversation.id.is_empty());
        assert_eq!(conversation.user_id, user_id);
        assert_eq!(conversation.model, model);
        assert!(conversation.title.is_none());
        assert!(conversation.system_prompt.is_none());
        assert!(conversation.archived_at.is_none());
        assert!(conversation.metadata.is_none());
        assert!(!conversation.created_at.is_empty());
        assert_eq!(conversation.created_at, conversation.updated_at);
    }

    #[test]
    fn test_ai_conversation_with_title() {
        let conversation = AiConversation::new("user_123".to_string(), "gpt-4".to_string())
            .with_title("Test Conversation".to_string());

        assert_eq!(conversation.title, Some("Test Conversation".to_string()));
    }

    #[test]
    fn test_ai_conversation_with_system_prompt() {
        let conversation = AiConversation::new("user_123".to_string(), "gpt-4".to_string())
            .with_system_prompt("You are a helpful assistant".to_string());

        assert_eq!(
            conversation.system_prompt,
            Some("You are a helpful assistant".to_string())
        );
    }

    #[test]
    fn test_ai_conversation_archive() {
        let mut conversation = AiConversation::new("user_123".to_string(), "gpt-4".to_string());
        let original_updated_at = conversation.updated_at.clone();

        // Small delay to ensure timestamps differ
        std::thread::sleep(std::time::Duration::from_millis(10));

        conversation.archive();

        assert!(conversation.archived_at.is_some());
        assert_ne!(conversation.updated_at, original_updated_at);
    }

    #[test]
    fn test_ai_conversation_update_timestamp() {
        let mut conversation = AiConversation::new("user_123".to_string(), "gpt-4".to_string());
        let original_updated_at = conversation.updated_at.clone();

        // Small delay to ensure timestamps differ
        std::thread::sleep(std::time::Duration::from_millis(10));

        conversation.update_timestamp();

        assert_ne!(conversation.updated_at, original_updated_at);
        assert!(conversation.archived_at.is_none());
    }

    #[test]
    fn test_ai_message_new() {
        let conversation_id = "conv_123".to_string();
        let role = "user".to_string();
        let content = "Hello, AI!".to_string();
        let message = AiMessage::new(conversation_id.clone(), role.clone(), content.clone());

        assert!(!message.id.is_empty());
        assert_eq!(message.conversation_id, conversation_id);
        assert_eq!(message.role, role);
        assert_eq!(message.content, content);
        assert!(message.token_count.is_none());
        assert!(!message.created_at.is_empty());
        assert!(message.metadata.is_none());
    }

    #[test]
    fn test_ai_message_with_token_count() {
        let message = AiMessage::new(
            "conv_123".to_string(),
            "assistant".to_string(),
            "I can help with that!".to_string(),
        )
        .with_token_count(42);

        assert_eq!(message.token_count, Some(42));
    }

    #[test]
    fn test_ai_usage_new() {
        let user_id = "user_123".to_string();
        let model = "gpt-4".to_string();
        let prompt_tokens = 100;
        let completion_tokens = 150;

        let usage = AiUsage::new(
            user_id.clone(),
            model.clone(),
            prompt_tokens,
            completion_tokens,
        );

        assert!(!usage.id.is_empty());
        assert_eq!(usage.user_id, user_id);
        assert_eq!(usage.model, model);
        assert_eq!(usage.prompt_tokens, prompt_tokens);
        assert_eq!(usage.completion_tokens, completion_tokens);
        assert_eq!(usage.total_tokens, 250);
        assert!(usage.conversation_id.is_none());
        assert!(usage.cost_cents.is_none());
        assert!(!usage.created_at.is_empty());
        assert!(usage.request_id.is_none());
        assert!(usage.duration_ms.is_none());
    }

    #[test]
    fn test_ai_usage_with_conversation() {
        let usage = AiUsage::new("user_123".to_string(), "gpt-4".to_string(), 100, 150)
            .with_conversation("conv_456".to_string());

        assert_eq!(usage.conversation_id, Some("conv_456".to_string()));
    }

    #[test]
    fn test_ai_usage_with_cost() {
        let usage =
            AiUsage::new("user_123".to_string(), "gpt-4".to_string(), 100, 150).with_cost(325); // $3.25 in cents

        assert_eq!(usage.cost_cents, Some(325));
    }

    #[test]
    fn test_ai_usage_with_request_id() {
        let usage = AiUsage::new("user_123".to_string(), "gpt-4".to_string(), 100, 150)
            .with_request_id("req_789".to_string());

        assert_eq!(usage.request_id, Some("req_789".to_string()));
    }

    #[test]
    fn test_ai_usage_with_duration() {
        let usage =
            AiUsage::new("user_123".to_string(), "gpt-4".to_string(), 100, 150).with_duration(1234);

        assert_eq!(usage.duration_ms, Some(1234));
    }

    #[test]
    fn test_ai_usage_builder_chain() {
        let usage = AiUsage::new("user_123".to_string(), "gpt-4".to_string(), 100, 150)
            .with_conversation("conv_456".to_string())
            .with_cost(325)
            .with_request_id("req_789".to_string())
            .with_duration(1234);

        assert_eq!(usage.conversation_id, Some("conv_456".to_string()));
        assert_eq!(usage.cost_cents, Some(325));
        assert_eq!(usage.request_id, Some("req_789".to_string()));
        assert_eq!(usage.duration_ms, Some(1234));
    }

    #[test]
    fn test_create_conversation_request_serialization() {
        let request = CreateConversationRequest {
            title: Some("Test Chat".to_string()),
            model: "gpt-4".to_string(),
            system_prompt: Some("Be helpful".to_string()),
        };

        let json = serde_json::to_string(&request).expect("Failed to serialize");
        assert!(json.contains("Test Chat"));
        assert!(json.contains("gpt-4"));
        assert!(json.contains("Be helpful"));
    }

    #[test]
    fn test_conversation_response_serialization() {
        let response = ConversationResponse {
            id: "conv_123".to_string(),
            title: Some("Test Chat".to_string()),
            model: "gpt-4".to_string(),
            system_prompt: Some("Be helpful".to_string()),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
            message_count: Some(5),
            last_message_at: Some("2024-01-01T01:00:00Z".to_string()),
        };

        let json = serde_json::to_string(&response).expect("Failed to serialize");
        let deserialized: ConversationResponse =
            serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(deserialized.id, response.id);
        assert_eq!(deserialized.message_count, Some(5));
    }

    #[test]
    fn test_message_response_serialization() {
        let response = MessageResponse {
            id: "msg_123".to_string(),
            role: "assistant".to_string(),
            content: "Hello!".to_string(),
            token_count: Some(10),
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&response).expect("Failed to serialize");
        assert!(json.contains("msg_123"));
        assert!(json.contains("assistant"));
    }

    #[test]
    fn test_conversation_with_messages_serialization() {
        let conv_response = ConversationResponse {
            id: "conv_123".to_string(),
            title: Some("Test".to_string()),
            model: "gpt-4".to_string(),
            system_prompt: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
            message_count: Some(2),
            last_message_at: None,
        };

        let messages = vec![
            MessageResponse {
                id: "msg_1".to_string(),
                role: "user".to_string(),
                content: "Hello".to_string(),
                token_count: Some(5),
                created_at: "2024-01-01T00:00:00Z".to_string(),
            },
            MessageResponse {
                id: "msg_2".to_string(),
                role: "assistant".to_string(),
                content: "Hi there!".to_string(),
                token_count: Some(8),
                created_at: "2024-01-01T00:00:01Z".to_string(),
            },
        ];

        let with_messages = ConversationWithMessages {
            conversation: conv_response,
            messages,
        };

        let json = serde_json::to_string(&with_messages).expect("Failed to serialize");
        assert!(json.contains("conv_123"));
        assert!(json.contains("msg_1"));
        assert!(json.contains("msg_2"));
    }

    #[test]
    fn test_create_message_request_serialization() {
        let request = CreateMessageRequest {
            role: "user".to_string(),
            content: "What is Rust?".to_string(),
        };

        let json = serde_json::to_string(&request).expect("Failed to serialize");
        assert!(json.contains("user"));
        assert!(json.contains("What is Rust?"));
    }

    #[test]
    fn test_usage_stats_response_serialization() {
        let response = UsageStatsResponse {
            total_conversations: 10,
            total_messages: 150,
            total_tokens: 50000,
            total_cost_cents: Some(1250),
            models_used: vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()],
        };

        let json = serde_json::to_string(&response).expect("Failed to serialize");
        let deserialized: UsageStatsResponse =
            serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(deserialized.total_conversations, 10);
        assert_eq!(deserialized.total_tokens, 50000);
        assert_eq!(deserialized.models_used.len(), 2);
    }
}
