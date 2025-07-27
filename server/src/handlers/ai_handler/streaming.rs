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
