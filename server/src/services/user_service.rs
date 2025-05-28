// web-template/server/src/services/user_service.rs

use crate::handlers::auth_handler::RegisterUserPayload;
use crate::{
    core::password_utils,
    errors::{AppError, AppResult},
    models::{User, UserConversionError, UserFromDb},
};
use sqlx::SqlitePool;
use tracing::instrument;
use uuid::Uuid;

#[derive(Clone)]
pub struct UserServiceImpl {
    db_pool: SqlitePool,
}

impl UserServiceImpl {
    pub fn new(db_pool: SqlitePool) -> Self {
        Self { db_pool }
    }

    #[instrument(skip(self, payload), fields(user_email = %payload.email), err(Debug))]
    pub async fn create_user(&self, payload: &RegisterUserPayload) -> AppResult<User> {
        let existing_user_check = sqlx::query("SELECT id FROM users WHERE email = $1")
            .bind(&payload.email)
            .fetch_optional(&self.db_pool)
            .await
            .map_err(|e| {
                tracing::error!(
                    "Database error checking for existing user {}: {}",
                    payload.email,
                    e
                );
                AppError::SqlxError(e)
            })?;

        if existing_user_check.is_some() {
            tracing::warn!("Attempt to register with existing email: {}", payload.email);
            return Err(AppError::UserAlreadyExists {
                email: payload.email.clone(),
            });
        }

        let hashed_password_string: String = password_utils::hash_password(&payload.password)
            .map_err(|e| {
                tracing::error!("Password hashing failed for {}: {}", payload.email, e);
                AppError::PasswordUtilError(e)
            })?;
        let hashed_password_slice: &str = hashed_password_string.as_str();

        let new_user_id = Uuid::new_v4();
        let current_time = chrono::Utc::now();
        let new_user_id_str = new_user_id.to_string();

        sqlx::query!(
            r#"
            INSERT INTO users (id, email, hashed_password, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5);
            "#,
            new_user_id_str,
            payload.email,
            hashed_password_slice,
            current_time,
            current_time
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to insert new user {}: {}", payload.email, e);
            AppError::SqlxError(e)
        })?;

        let db_user = sqlx::query_as!(
            UserFromDb,
            "SELECT id, email, hashed_password, created_at, updated_at FROM users WHERE id = $1",
            new_user_id_str
        )
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| {
            tracing::error!(
                "Failed to fetch newly created user {}: {}",
                payload.email,
                e
            );
            AppError::SqlxError(e)
        })?;

        let created_user = User::try_from(db_user).map_err(|conv_err: UserConversionError| {
            tracing::error!(
                "Failed to convert DB user to domain model {}: {}",
                payload.email,
                conv_err
            );
            AppError::InternalServerError(format!("User data conversion error: {conv_err}"))
        })?;

        tracing::info!("User created successfully: {}", created_user.email);
        Ok(created_user)
    }
}
