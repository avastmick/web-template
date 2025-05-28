// web-template/server/src/lib.rs

//! Server library crate
//!
//! This module exposes the core server functionality for use in integration tests
//! and other consumers of the server as a library.

pub mod core;
pub mod errors;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod routes;
pub mod services;
