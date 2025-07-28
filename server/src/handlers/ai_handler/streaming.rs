//! Streaming chat handlers

use axum::{
    extract::{Query, State},
    response::{Sse, sse::Event},
};
use axum_extra::TypedHeader;
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Bearer;
use futures::stream::{self, Stream};
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

use crate::ai::{ChatMessage, ChatRequest as AiChatRequest, ChatRole};
use crate::core::AppState;
use crate::errors::AppResult;

use super::chat::ChatRequest;

#[derive(Debug, serde::Serialize)]
pub struct StreamChunk {
    pub id: String,
    pub delta: String,
    pub finished: bool,
}

/// Handle SSE streaming chat requests
///
/// # Errors
///
/// Returns an error if the AI request fails or authentication is invalid.
pub async fn chat_stream_handler(
    State(state): State<Arc<AppState>>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Query(params): Query<ChatRequest>,
) -> AppResult<Sse<impl Stream<Item = Result<Event, Infallible>>>> {
    // Verify JWT token and get user
    let token = auth.token();
    let _user_id = state.auth.get_user_id_from_token(token)?;

    let chat_id = Uuid::new_v4().to_string();

    // Convert request messages
    let messages: Vec<ChatMessage> = params
        .messages
        .into_iter()
        .map(|msg| ChatMessage {
            role: match msg.role.as_str() {
                "system" => ChatRole::System,
                "assistant" => ChatRole::Assistant,
                _ => ChatRole::User,
            },
            content: msg.content,
        })
        .collect();

    // For now, simulate streaming by calling regular chat and splitting the response
    let ai_service = state.ai.read().await;
    let chat_request = AiChatRequest::new(messages);
    let response = ai_service.chat(chat_request).await;

    // Create a stream that sends events
    let stream = stream::unfold(
        (chat_id, response, 0),
        |(chat_id, response, mut index)| async move {
            match &response {
                Ok(chat_response) => {
                    let content = chat_response
                        .choices
                        .first()
                        .map(|c| c.message.content.clone())
                        .unwrap_or_default();

                    // Split the response into chunks to simulate streaming
                    let words: Vec<&str> = content.split_whitespace().collect();
                    let chunks: Vec<String> = words
                        .chunks(3) // Group words into chunks of 3
                        .map(|chunk| format!("{} ", chunk.join(" ")))
                        .collect();

                    if index < chunks.len() {
                        let chunk = StreamChunk {
                            id: chat_id.clone(),
                            delta: chunks[index].clone(),
                            finished: index == chunks.len() - 1,
                        };

                        let event = Event::default()
                            .json_data(chunk)
                            .unwrap_or_else(|_| Event::default().data("error"));

                        index += 1;
                        tokio::time::sleep(Duration::from_millis(100)).await;
                        Some((Ok(event), (chat_id, response, index)))
                    } else {
                        None
                    }
                }
                Err(e) => {
                    if index == 0 {
                        let error_chunk = StreamChunk {
                            id: chat_id.clone(),
                            delta: format!("Error: {e}"),
                            finished: true,
                        };

                        let event = Event::default()
                            .json_data(error_chunk)
                            .unwrap_or_else(|_| Event::default().data("error"));

                        Some((Ok(event), (chat_id, response, 1)))
                    } else {
                        None
                    }
                }
            }
        },
    );

    Ok(Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(30))
            .text("keep-alive"),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::ai_handler::chat::MessageInput;

    #[test]
    fn test_stream_chunk_serialization() {
        let chunk = StreamChunk {
            id: "test_id".to_string(),
            delta: "Hello world".to_string(),
            finished: false,
        };

        let json = serde_json::to_value(&chunk).expect("Should serialize stream chunk");
        assert_eq!(json["id"], "test_id");
        assert_eq!(json["delta"], "Hello world");
        assert_eq!(json["finished"], false);
    }

    #[test]
    fn test_stream_chunk_finished() {
        let chunk = StreamChunk {
            id: "test_id".to_string(),
            delta: "Final chunk".to_string(),
            finished: true,
        };

        let json = serde_json::to_value(&chunk).expect("Should serialize finished chunk");
        assert_eq!(json["finished"], true);
    }

    #[tokio::test]
    async fn test_chat_request_message_conversion() {
        // Test the conversion logic from MessageInput to ChatMessage
        let messages = vec![
            MessageInput {
                role: "system".to_string(),
                content: "You are a helpful assistant".to_string(),
            },
            MessageInput {
                role: "user".to_string(),
                content: "Hello".to_string(),
            },
            MessageInput {
                role: "assistant".to_string(),
                content: "Hi there!".to_string(),
            },
            MessageInput {
                role: "invalid".to_string(), // Should default to User
                content: "Test".to_string(),
            },
        ];

        // Convert messages like the handler does
        let converted: Vec<ChatMessage> = messages
            .into_iter()
            .map(|msg| ChatMessage {
                role: match msg.role.as_str() {
                    "system" => ChatRole::System,
                    "assistant" => ChatRole::Assistant,
                    _ => ChatRole::User,
                },
                content: msg.content,
            })
            .collect();

        assert_eq!(converted.len(), 4);
        assert!(matches!(converted[0].role, ChatRole::System));
        assert!(matches!(converted[1].role, ChatRole::User));
        assert!(matches!(converted[2].role, ChatRole::Assistant));
        assert!(matches!(converted[3].role, ChatRole::User)); // Invalid role defaults to User
    }

    // Note: Testing the actual chat_stream_handler requires authentication setup
    // and would be better suited for integration tests due to the complexity
    // of mocking SSE streaming responses and JWT tokens.

    #[tokio::test]
    async fn test_uuid_generation() {
        // Test that we can generate unique chat IDs
        let id1 = Uuid::new_v4().to_string();
        let id2 = Uuid::new_v4().to_string();

        assert_ne!(id1, id2);
        assert!(!id1.is_empty());
        assert!(!id2.is_empty());
    }

    #[test]
    fn test_word_chunking_logic() {
        // Test the chunking logic used in the streaming handler
        let content = "This is a test message with multiple words";
        let words: Vec<&str> = content.split_whitespace().collect();
        let chunks: Vec<String> = words
            .chunks(3) // Group words into chunks of 3
            .map(|chunk| format!("{} ", chunk.join(" ")))
            .collect();

        assert_eq!(chunks.len(), 3); // 9 words / 3 = 3 chunks
        assert_eq!(chunks[0], "This is a ");
        assert_eq!(chunks[1], "test message with ");
        assert_eq!(chunks[2], "multiple words ");
    }

    #[test]
    fn test_word_chunking_edge_cases() {
        // Test empty content
        let content = "";
        let words: Vec<&str> = content.split_whitespace().collect();
        let chunks: Vec<String> = words
            .chunks(3)
            .map(|chunk| format!("{} ", chunk.join(" ")))
            .collect();
        assert_eq!(chunks.len(), 0);

        // Test single word
        let content = "Hello";
        let words: Vec<&str> = content.split_whitespace().collect();
        let chunks: Vec<String> = words
            .chunks(3)
            .map(|chunk| format!("{} ", chunk.join(" ")))
            .collect();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], "Hello ");

        // Test two words
        let content = "Hello world";
        let words: Vec<&str> = content.split_whitespace().collect();
        let chunks: Vec<String> = words
            .chunks(3)
            .map(|chunk| format!("{} ", chunk.join(" ")))
            .collect();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], "Hello world ");
    }

    #[test]
    fn test_stream_chunk_debug() {
        let chunk = StreamChunk {
            id: "debug_test".to_string(),
            delta: "Debug content".to_string(),
            finished: false,
        };

        let debug_str = format!("{chunk:?}");
        assert!(debug_str.contains("StreamChunk"));
        assert!(debug_str.contains("debug_test"));
        assert!(debug_str.contains("Debug content"));
        assert!(debug_str.contains("false"));
    }
}
