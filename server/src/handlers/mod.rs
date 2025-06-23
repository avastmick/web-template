// web-template/server/src/handlers/mod.rs

pub mod ai_handler;
pub mod auth_handler;
pub mod health_handler;
pub mod oauth_handler;
pub mod user_handler;

// Re-export handlers for easier access from the routing module, if preferred.
// For example:
// pub use auth_handler::register_user_handler;
