//! AI API Contract Snapshots
//!
//! Tests for AI/conversation response schema stability.

use insta::assert_json_snapshot;
use serde::{Deserialize, Serialize};

/// Conversation response (mirrors server::models::ai_models::ConversationResponse)
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

/// Message response (mirrors server::models::ai_models::MessageResponse)
#[derive(Debug, Serialize, Deserialize)]
pub struct MessageResponse {
    pub id: String,
    pub role: String,
    pub content: String,
    pub token_count: Option<i64>,
    pub created_at: String,
}

/// Conversation with messages (mirrors server::models::ai_models::ConversationWithMessages)
#[derive(Debug, Serialize, Deserialize)]
pub struct ConversationWithMessages {
    pub conversation: ConversationResponse,
    pub messages: Vec<MessageResponse>,
}

/// Usage stats response (mirrors server::models::ai_models::UsageStatsResponse)
#[derive(Debug, Serialize, Deserialize)]
pub struct UsageStatsResponse {
    pub total_conversations: i64,
    pub total_messages: i64,
    pub total_tokens: i64,
    pub total_cost_cents: Option<i64>,
    pub models_used: Vec<String>,
}

/// Chat response from handler (simplified from server::handlers::ai_handler::chat)
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatResponse {
    pub id: String,
    pub conversation_id: String,
    pub message: MessageOutput,
    pub usage: Option<TokenUsage>,
}

/// Message output (simplified)
#[derive(Debug, Serialize, Deserialize)]
pub struct MessageOutput {
    pub role: String,
    pub content: String,
}

/// Token usage (simplified)
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt: u32,
    pub completion: u32,
    pub total: u32,
}

// Fixed values for deterministic snapshots
const FIXED_CONVERSATION_ID: &str = "conv_550e8400-e29b-41d4-a716-446655440000";
const FIXED_MESSAGE_ID: &str = "msg_550e8400-e29b-41d4-a716-446655440001";
const FIXED_TIMESTAMP: &str = "2024-01-15T12:00:00Z";
const FIXED_UPDATED_TIMESTAMP: &str = "2024-01-15T13:00:00Z";

#[test]
fn test_conversation_response_minimal() {
    let response = ConversationResponse {
        id: FIXED_CONVERSATION_ID.to_string(),
        title: None,
        model: "gpt-4".to_string(),
        system_prompt: None,
        created_at: FIXED_TIMESTAMP.to_string(),
        updated_at: FIXED_TIMESTAMP.to_string(),
        message_count: Some(0),
        last_message_at: None,
    };

    assert_json_snapshot!("conversation_response_minimal", response);
}

#[test]
fn test_conversation_response_full() {
    let response = ConversationResponse {
        id: FIXED_CONVERSATION_ID.to_string(),
        title: Some("Project Discussion".to_string()),
        model: "gpt-4-turbo".to_string(),
        system_prompt: Some("You are a helpful software engineering assistant.".to_string()),
        created_at: FIXED_TIMESTAMP.to_string(),
        updated_at: FIXED_UPDATED_TIMESTAMP.to_string(),
        message_count: Some(42),
        last_message_at: Some(FIXED_UPDATED_TIMESTAMP.to_string()),
    };

    assert_json_snapshot!("conversation_response_full", response);
}

#[test]
fn test_message_response_user() {
    let response = MessageResponse {
        id: FIXED_MESSAGE_ID.to_string(),
        role: "user".to_string(),
        content: "How do I implement authentication in Rust?".to_string(),
        token_count: Some(10),
        created_at: FIXED_TIMESTAMP.to_string(),
    };

    assert_json_snapshot!("message_response_user", response);
}

#[test]
fn test_message_response_assistant() {
    let response = MessageResponse {
        id: FIXED_MESSAGE_ID.to_string(),
        role: "assistant".to_string(),
        content: "To implement authentication in Rust, you have several options...".to_string(),
        token_count: Some(150),
        created_at: FIXED_TIMESTAMP.to_string(),
    };

    assert_json_snapshot!("message_response_assistant", response);
}

#[test]
fn test_message_response_system() {
    let response = MessageResponse {
        id: FIXED_MESSAGE_ID.to_string(),
        role: "system".to_string(),
        content: "You are a helpful software engineering assistant.".to_string(),
        token_count: Some(8),
        created_at: FIXED_TIMESTAMP.to_string(),
    };

    assert_json_snapshot!("message_response_system", response);
}

#[test]
fn test_conversation_with_messages() {
    let conversation = ConversationResponse {
        id: FIXED_CONVERSATION_ID.to_string(),
        title: Some("Rust Auth Discussion".to_string()),
        model: "gpt-4".to_string(),
        system_prompt: Some("You are a helpful assistant.".to_string()),
        created_at: FIXED_TIMESTAMP.to_string(),
        updated_at: FIXED_UPDATED_TIMESTAMP.to_string(),
        message_count: Some(3),
        last_message_at: Some(FIXED_UPDATED_TIMESTAMP.to_string()),
    };

    let messages = vec![
        MessageResponse {
            id: "msg_001".to_string(),
            role: "system".to_string(),
            content: "You are a helpful assistant.".to_string(),
            token_count: Some(6),
            created_at: FIXED_TIMESTAMP.to_string(),
        },
        MessageResponse {
            id: "msg_002".to_string(),
            role: "user".to_string(),
            content: "How do I use JWT in Rust?".to_string(),
            token_count: Some(8),
            created_at: FIXED_TIMESTAMP.to_string(),
        },
        MessageResponse {
            id: "msg_003".to_string(),
            role: "assistant".to_string(),
            content: "You can use the jsonwebtoken crate for JWT in Rust...".to_string(),
            token_count: Some(120),
            created_at: FIXED_UPDATED_TIMESTAMP.to_string(),
        },
    ];

    let response = ConversationWithMessages {
        conversation,
        messages,
    };

    assert_json_snapshot!("conversation_with_messages", response);
}

#[test]
fn test_usage_stats_response_minimal() {
    let response = UsageStatsResponse {
        total_conversations: 0,
        total_messages: 0,
        total_tokens: 0,
        total_cost_cents: None,
        models_used: vec![],
    };

    assert_json_snapshot!("usage_stats_response_minimal", response);
}

#[test]
fn test_usage_stats_response_full() {
    let response = UsageStatsResponse {
        total_conversations: 25,
        total_messages: 342,
        total_tokens: 125_000,
        total_cost_cents: Some(4250),
        models_used: vec![
            "gpt-4".to_string(),
            "gpt-4-turbo".to_string(),
            "gpt-3.5-turbo".to_string(),
        ],
    };

    assert_json_snapshot!("usage_stats_response_full", response);
}

#[test]
fn test_chat_response() {
    let response = ChatResponse {
        id: FIXED_MESSAGE_ID.to_string(),
        conversation_id: FIXED_CONVERSATION_ID.to_string(),
        message: MessageOutput {
            role: "assistant".to_string(),
            content: "Here's how you can solve that problem...".to_string(),
        },
        usage: Some(TokenUsage {
            prompt: 150,
            completion: 200,
            total: 350,
        }),
    };

    assert_json_snapshot!("chat_response", response);
}

#[test]
fn test_chat_response_no_usage() {
    let response = ChatResponse {
        id: FIXED_MESSAGE_ID.to_string(),
        conversation_id: FIXED_CONVERSATION_ID.to_string(),
        message: MessageOutput {
            role: "assistant".to_string(),
            content: "I understand your question...".to_string(),
        },
        usage: None,
    };

    assert_json_snapshot!("chat_response_no_usage", response);
}

#[test]
fn test_token_usage() {
    let usage = TokenUsage {
        prompt: 100,
        completion: 250,
        total: 350,
    };

    assert_json_snapshot!("token_usage", usage);
}

#[test]
fn test_message_output() {
    let message = MessageOutput {
        role: "assistant".to_string(),
        content: "This is the assistant's response.".to_string(),
    };

    assert_json_snapshot!("message_output", message);
}
