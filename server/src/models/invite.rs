use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserInvite {
    pub id: String,
    pub email: String,
    pub invited_by: Option<String>,
    pub invited_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl UserInvite {
    #[must_use]
    pub fn new(email: &str, invited_by: Option<String>, expires_at: Option<DateTime<Utc>>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            email: email.to_lowercase(),
            invited_by,
            invited_at: now,
            used_at: None,
            expires_at,
            created_at: now,
            updated_at: now,
        }
    }

    #[must_use]
    pub fn is_valid(&self) -> bool {
        // Invite is valid if not used and not expired
        self.used_at.is_none() && !self.is_expired()
    }

    #[must_use]
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    pub fn mark_used(&mut self) {
        self.used_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }
}
