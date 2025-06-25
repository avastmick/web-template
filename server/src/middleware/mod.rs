// web-template/server/src/middleware/mod.rs

pub mod auth_middleware;
pub mod payment_middleware;

// Re-export for convenience
pub use auth_middleware::JwtAuth;
// PaymentRequired will be used when we update the AI handlers
// pub use payment_middleware::PaymentRequired;
