// web-template/server/src/models/user.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::convert::TryFrom;
use std::str::FromStr;
use uuid::Uuid;

/// Represents a user in the system (domain model).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub hashed_password: String,
    pub provider: String,
    pub provider_user_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Represents a user row as fetched directly from the `SQLite` database.
/// All fields are Option<String> to accommodate sqlx-macros' conservative inference for `SQLite` TEXT columns.
#[derive(Debug, Clone, FromRow)]
pub struct UserFromDb {
    pub id: Option<String>,
    pub email: Option<String>,
    pub hashed_password: Option<String>,
    pub provider: Option<String>,
    pub provider_user_id: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

/// Error type for conversion from `UserFromDb` to `User`.
#[derive(Debug, thiserror::Error)]
pub enum UserConversionError {
    #[error("Missing expected database value for field: {0}")]
    MissingDatabaseValue(String),
    #[error("Failed to parse UUID from string '{value}': {source}")]
    UuidParseError { value: String, source: uuid::Error },
    #[error("Failed to parse DateTime from string '{value}': {source}")]
    DateTimeParseError {
        value: String,
        source: chrono::ParseError,
    },
}

impl TryFrom<UserFromDb> for User {
    type Error = UserConversionError;

    fn try_from(db_row: UserFromDb) -> Result<Self, Self::Error> {
        let id_str = db_row
            .id
            .ok_or_else(|| UserConversionError::MissingDatabaseValue("id".to_string()))?;
        let email = db_row
            .email
            .ok_or_else(|| UserConversionError::MissingDatabaseValue("email".to_string()))?;
        let hashed_password = db_row.hashed_password.ok_or_else(|| {
            UserConversionError::MissingDatabaseValue("hashed_password".to_string())
        })?;
        let provider = db_row.provider.unwrap_or_else(|| "local".to_string());
        let provider_user_id = db_row.provider_user_id;
        let created_at_str = db_row
            .created_at
            .ok_or_else(|| UserConversionError::MissingDatabaseValue("created_at".to_string()))?;
        let updated_at_str = db_row
            .updated_at
            .ok_or_else(|| UserConversionError::MissingDatabaseValue("updated_at".to_string()))?;

        let id = Uuid::from_str(&id_str).map_err(|e| UserConversionError::UuidParseError {
            value: id_str.clone(),
            source: e,
        })?;
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map_err(|e| UserConversionError::DateTimeParseError {
                value: created_at_str.clone(),
                source: e,
            })?
            .with_timezone(&Utc);
        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
            .map_err(|e| UserConversionError::DateTimeParseError {
                value: updated_at_str.clone(),
                source: e,
            })?
            .with_timezone(&Utc);

        Ok(User {
            id,
            email,
            hashed_password,
            provider,
            provider_user_id,
            created_at,
            updated_at,
        })
    }
}
