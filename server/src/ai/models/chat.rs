//! Chat-related models for AI interactions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    System,
    User,
    Assistant,
    Function,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    pub model: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stream: Option<bool>,
    pub response_format: Option<ResponseFormat>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub functions: Option<Vec<serde_json::Value>>,
    pub function_call: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseFormat {
    #[serde(rename = "type")]
    pub format_type: String,
    pub json_schema: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub id: String,
    pub model: String,
    pub choices: Vec<ChatChoice>,
    pub usage: Option<crate::ai::models::usage::TokenUsage>,
    pub created: i64,
    pub function_call: Option<FunctionCall>,
}

impl ChatResponse {
    /// Get the content from the first choice
    #[must_use]
    pub fn content(&self) -> String {
        self.choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChoice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    Start {
        id: String,
        model: String,
    },
    Delta {
        content: String,
        index: u32,
    },
    Error {
        message: String,
    },
    Done {
        finish_reason: String,
        usage: Option<crate::ai::models::usage::TokenUsage>,
    },
}

impl ChatRequest {
    #[must_use]
    pub fn new(messages: Vec<ChatMessage>) -> Self {
        Self {
            messages,
            model: None,
            temperature: None,
            max_tokens: None,
            stream: None,
            response_format: None,
            metadata: None,
            functions: None,
            function_call: None,
        }
    }

    /// Add JSON schema for structured response
    #[must_use]
    pub fn with_json_schema(mut self, schema: serde_json::Value) -> Self {
        self.response_format = Some(ResponseFormat {
            format_type: "json_schema".to_string(),
            json_schema: Some(schema),
        });
        self
    }
}
