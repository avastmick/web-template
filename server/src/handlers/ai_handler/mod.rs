//! AI endpoint handlers module

pub mod chat;
pub mod conversations;
pub mod file_upload;
pub mod misc;
pub mod streaming;

// Re-export all public handlers
pub use chat::chat_handler;
pub use conversations::{
    archive_conversation_handler, get_conversation_handler, get_conversations_handler,
    get_usage_stats_handler,
};
pub use file_upload::upload_file_handler;
pub use misc::{
    ai_info_handler, code_analysis_handler, contextual_chat_handler, demo_message_handler,
    error_demo_handler, health_check_handler, moderate_content_handler, verify_token_handler,
};
pub use streaming::chat_stream_handler;

// Re-export handlers that belong elsewhere
// These should be moved to separate handler modules
pub use misc::{
    create_invite_handler, delete_invite_handler, get_invite_handler, list_invites_handler,
};
