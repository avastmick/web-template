pub mod ai_models;
pub mod ai_persona;
pub mod ai_session;
pub mod auth;
pub mod invite;
pub mod oauth;
pub mod payment;
pub mod user;

pub use ai_models::{
    AiConversation, AiMessage, AiUsage, ConversationResponse, ConversationWithMessages,
    CreateConversationRequest, CreateMessageRequest, MessageResponse, UsageStatsResponse,
};
pub use ai_persona::AiPersona;
pub use ai_session::{
    AiSession, AiSessionAsset, AiSessionMessage, CreateSessionRequest, CreateSessionResponse,
    FinalizeAction, FinalizeSessionRequest, FinalizeSessionResponse, IssueDraft, IssuePreview,
    MessageRole, SendMessageRequest, SendMessageResponse, SessionContext, SessionStatus,
};
// Public API exports
pub use auth::{AuthUser, OAuthCallbackParams, PaymentUser, UnifiedAuthResponse};
pub use invite::UserInvite;
// Payment models exported internally to modules
// Individual modules import directly from payment::
pub use user::{User, UserConversionError, UserFromDb};
