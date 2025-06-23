// web-template/server/src/core/mod.rs

pub mod password_utils;
pub mod state;

// Re-export for easier access if desired
// pub use password_utils::{hash_password, verify_password, PasswordError};
pub use state::AppState;
