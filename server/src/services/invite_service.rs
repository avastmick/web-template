use crate::errors::AppError;
use crate::models::UserInvite;
use chrono::{DateTime, Utc};
use sqlx::{Pool, Sqlite};

pub struct InviteService {
    db: Pool<Sqlite>,
}

impl InviteService {
    #[must_use]
    pub fn new(db: Pool<Sqlite>) -> Self {
        Self { db }
    }

    /// Check if a valid invite exists for the given email
    ///
    /// # Errors
    ///
    /// Returns `AppError` if database connection fails
    pub async fn check_invite_exists(&self, email: &str) -> Result<bool, AppError> {
        let email_lower = email.to_lowercase();

        let current_time = Utc::now();
        let result = sqlx::query!(
            r#"
            SELECT id FROM user_invites
            WHERE email = ?1
            AND used_at IS NULL
            AND (expires_at IS NULL OR expires_at > ?2)
            "#,
            email_lower,
            current_time
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(result.is_some())
    }

    /// Get a valid invite by email
    ///
    /// # Errors
    ///
    /// Returns `AppError` if database connection fails
    pub async fn get_valid_invite(&self, email: &str) -> Result<Option<UserInvite>, AppError> {
        let email_lower = email.to_lowercase();

        let invite = sqlx::query_as!(
            UserInvite,
            r#"
            SELECT
                id,
                email,
                invited_by,
                invited_at as "invited_at: DateTime<Utc>",
                used_at as "used_at: DateTime<Utc>",
                expires_at as "expires_at: DateTime<Utc>",
                created_at as "created_at: DateTime<Utc>",
                updated_at as "updated_at: DateTime<Utc>"
            FROM user_invites
            WHERE email = ?1
            AND used_at IS NULL
            AND (expires_at IS NULL OR expires_at > datetime('now'))
            "#,
            email_lower
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(invite)
    }

    /// Get any invite for a user (used or unused)
    /// This is used to check if a user has ever had a valid invite
    ///
    /// # Errors
    ///
    /// Returns `AppError` if database connection fails
    pub async fn get_user_invite(&self, email: &str) -> Result<Option<UserInvite>, AppError> {
        let email_lower = email.to_lowercase();

        let invite = sqlx::query_as!(
            UserInvite,
            r#"
            SELECT
                id,
                email,
                invited_by,
                invited_at as "invited_at: DateTime<Utc>",
                used_at as "used_at: DateTime<Utc>",
                expires_at as "expires_at: DateTime<Utc>",
                created_at as "created_at: DateTime<Utc>",
                updated_at as "updated_at: DateTime<Utc>"
            FROM user_invites
            WHERE email = ?1
            ORDER BY created_at DESC
            LIMIT 1
            "#,
            email_lower
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(invite)
    }

    /// Mark an invite as used
    ///
    /// # Errors
    ///
    /// Returns `AppError` if:
    /// - Database connection fails
    /// - Invite not found
    pub async fn mark_invite_used(&self, email: &str) -> Result<(), AppError> {
        let email_lower = email.to_lowercase();
        let now = Utc::now();

        let result = sqlx::query!(
            r#"
            UPDATE user_invites
            SET used_at = ?1, updated_at = ?1
            WHERE email = ?2
            AND used_at IS NULL
            "#,
            now,
            email_lower
        )
        .execute(&self.db)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::InviteNotFound);
        }

        Ok(())
    }

    /// Create a new invite
    ///
    /// # Errors
    ///
    /// Returns `AppError` if:
    /// - Database connection fails
    /// - Failed to generate invite ID
    pub async fn create_invite(
        &self,
        email: &str,
        invited_by: Option<String>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<UserInvite, AppError> {
        let invite = UserInvite::new(email, invited_by, expires_at);

        sqlx::query!(
            r#"
            INSERT INTO user_invites (id, email, invited_by, invited_at, expires_at, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            invite.id,
            invite.email,
            invite.invited_by,
            invite.invited_at,
            invite.expires_at,
            invite.created_at,
            invite.updated_at
        )
        .execute(&self.db)
        .await?;

        Ok(invite)
    }

    /// List all invites (for admin purposes)
    ///
    /// # Errors
    ///
    /// Returns `AppError` if database connection fails
    pub async fn list_invites(&self) -> Result<Vec<UserInvite>, AppError> {
        let invites = sqlx::query_as!(
            UserInvite,
            r#"
            SELECT
                id,
                email,
                invited_by,
                invited_at as "invited_at: DateTime<Utc>",
                used_at as "used_at: DateTime<Utc>",
                expires_at as "expires_at: DateTime<Utc>",
                created_at as "created_at: DateTime<Utc>",
                updated_at as "updated_at: DateTime<Utc>"
            FROM user_invites
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.db)
        .await?;

        Ok(invites)
    }

    /// Delete an invite
    ///
    /// # Errors
    ///
    /// Returns `AppError` if:
    /// - Database connection fails
    /// - Invite not found
    pub async fn delete_invite(&self, id: &str) -> Result<(), AppError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM user_invites WHERE id = ?1
            "#,
            id
        )
        .execute(&self.db)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::InviteNotFound);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_test_db() -> Pool<Sqlite> {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create test database");

        // Run migrations to ensure test database matches production schema
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");

        pool
    }

    #[tokio::test]
    async fn test_create_and_check_invite() {
        let pool = setup_test_db().await;
        let service = InviteService::new(pool);

        // Create an invite
        let invite = service
            .create_invite("test_create@example.com", Some("admin".to_string()), None)
            .await
            .expect("Failed to create invite");

        assert_eq!(invite.email, "test_create@example.com");
        assert_eq!(invite.invited_by, Some("admin".to_string()));
        assert!(invite.used_at.is_none());

        // Check invite exists
        let exists = service
            .check_invite_exists("test@example.com")
            .await
            .expect("Failed to check invite exists");
        assert!(exists);

        // Check with different case
        let exists = service
            .check_invite_exists("TEST@EXAMPLE.COM")
            .await
            .expect("Failed to check invite exists with different case");
        assert!(exists);
    }

    #[tokio::test]
    async fn test_mark_invite_used() {
        let pool = setup_test_db().await;
        let service = InviteService::new(pool);

        // Create an invite
        service
            .create_invite("test_mark_used@example.com", None, None)
            .await
            .expect("Failed to create invite");

        // Mark as used
        service
            .mark_invite_used("test_mark_used@example.com")
            .await
            .expect("Failed to mark invite as used");

        // Check invite no longer exists (as valid)
        let exists = service
            .check_invite_exists("test_mark_used@example.com")
            .await
            .expect("Failed to check invite exists after marking used");
        assert!(!exists);
    }

    #[tokio::test]
    async fn test_expired_invite() {
        let pool = setup_test_db().await;
        let service = InviteService::new(pool);

        // Create an expired invite
        let past_time = Utc::now() - chrono::Duration::hours(1);
        service
            .create_invite("test_expired@example.com", None, Some(past_time))
            .await
            .expect("Failed to create expired invite");

        // Check invite does not exist (expired)
        let exists = service
            .check_invite_exists("test_expired@example.com")
            .await
            .expect("Failed to check expired invite exists");
        assert!(!exists);
    }
}
