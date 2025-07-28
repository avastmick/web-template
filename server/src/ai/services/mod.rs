//! AI business logic services

pub mod schema_validator;

pub use schema_validator::{SchemaValidator, schemas};

// For now, re-export existing services to avoid breaking changes
pub use crate::services::ai_service::AiService;
