//! API Contract Snapshot Tests
//!
//! This module contains snapshot tests for API response schemas.
//! These tests ensure API response formats remain stable and
//! any changes are intentional and documented.
//!
//! Use `cargo insta review` to review and accept snapshot changes.

pub mod ai_contracts;
pub mod auth_contracts;
pub mod payment_contracts;
