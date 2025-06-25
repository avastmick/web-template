// web-template/server/src/services/mod.rs

pub mod ai_data_service;
pub mod ai_service;
pub mod auth_service;
pub mod invite_service;
pub mod oauth_service;
pub mod payment;
pub mod user_service;

// Re-export for convenience
pub use ai_data_service::AiDataService;
pub use ai_service::AiService;
pub use auth_service::AuthService;
pub use invite_service::InviteService;
pub use oauth_service::OAuthService;
pub use payment::PaymentService;
pub use user_service::UserServiceImpl;
