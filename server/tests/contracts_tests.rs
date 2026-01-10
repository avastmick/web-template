//! API Contract Snapshot Tests Runner
//!
//! This module runs all API contract snapshot tests.
//! These tests ensure API response formats remain stable.
//!
//! Run with: cargo test --test contracts_tests
//! Review snapshots with: cargo insta review

mod contracts;

// Re-export all contract test modules so they run as tests
pub use contracts::*;
