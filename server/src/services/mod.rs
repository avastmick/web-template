// web-template/server/src/services/mod.rs

pub mod auth_service;
pub mod invite_service;
pub mod user_service;

// Re-export for convenience
pub use auth_service::AuthService;
pub use invite_service::InviteService;
pub use user_service::UserServiceImpl;
