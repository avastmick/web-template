// web-template/server/src/models/mod.rs

pub mod ai_models;
pub mod invite;
pub mod oauth;
pub mod payment;
pub mod user;

pub use ai_models::{
    AiConversation, AiMessage, AiUsage, ConversationResponse, ConversationWithMessages,
    CreateConversationRequest, CreateMessageRequest, MessageResponse, UsageStatsResponse,
};
pub use invite::UserInvite;
// Payment models exported internally to modules
// Individual modules import directly from payment::
pub use user::{User, UserConversionError, UserFromDb};
