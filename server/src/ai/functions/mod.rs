//! Function calling definitions for AI assistants
//!
//! This module provides OpenAI-compatible function definitions that AI assistants
//! can use to perform structured actions during conversations.

pub mod definitions;

pub use definitions::{
    FunctionCall, FunctionDefinition, FunctionParameter, FunctionResult,
    get_business_analyst_functions,
};
