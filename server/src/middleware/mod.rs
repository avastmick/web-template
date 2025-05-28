// web-template/server/src/middleware/mod.rs

pub mod auth_middleware;

// Re-export for convenience
pub use auth_middleware::JwtAuth;
