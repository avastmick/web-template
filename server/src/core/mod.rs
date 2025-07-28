// kanbain/server/src/core/mod.rs

pub mod auth_utils;
pub mod password_utils;
pub mod state;

// Re-export for easier access if desired
// pub use password_utils::{hash_password, verify_password, PasswordError};
pub use auth_utils::{build_unified_auth_response, build_unified_auth_response_no_token};
pub use state::AppState;
