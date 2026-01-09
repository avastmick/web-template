//! Middleware tests runner
//!
//! This module includes all middleware test submodules and makes them
//! discoverable as a single test binary by Cargo.

mod common;
mod middleware;

// Re-export all middleware test modules so they can be run
pub use middleware::*;
