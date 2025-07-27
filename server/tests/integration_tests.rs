//! Integration tests runner
//!
//! This module includes all integration test submodules and makes them
//! discoverable as a single test binary by Cargo.

mod common;
mod integration;

// Re-export all integration test modules so they can be run
pub use integration::*;
