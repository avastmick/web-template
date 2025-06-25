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
    pub db_pool: SqlitePool,
}

impl UserServiceImpl {
    #[must_use]
    pub fn new(db_pool: SqlitePool) -> Self {
        Self { db_pool }
    }

    /// Create a new user
    ///
    /// # Errors
    ///
    /// Returns an error if user already exists, password hashing fails, or database operation fails
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
            "SELECT id, email, hashed_password, provider, provider_user_id, created_at, updated_at FROM users WHERE id = $1",
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

    /// Finds a user by their email address
    ///
    /// # Arguments
    /// * `email` - The email address to search for
    ///
    /// # Returns
    /// Returns the user if found
    ///
    /// # Errors
    /// Returns `AppError::UserNotFound` if no user with the given email exists
    /// Returns `AppError::SqlxError` for database errors
    #[instrument(skip(self), fields(email = %email), err(Debug))]
    pub async fn find_by_email(&self, email: &str) -> AppResult<User> {
        tracing::debug!("Searching for user with email: {}", email);

        let db_user = sqlx::query_as!(
            UserFromDb,
            "SELECT id, email, hashed_password, provider, provider_user_id, created_at, updated_at FROM users WHERE email = $1",
            email
        )
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error while searching for user {}: {}", email, e);
            AppError::SqlxError(e)
        })?;

        if let Some(user) = db_user {
            let domain_user = User::try_from(user).map_err(|conv_err: UserConversionError| {
                tracing::error!(
                    "Failed to convert DB user to domain model {}: {}",
                    email,
                    conv_err
                );
                AppError::InternalServerError(format!("User data conversion error: {conv_err}"))
            })?;

            tracing::debug!("User found: {}", email);
            Ok(domain_user)
        } else {
            tracing::debug!("User not found: {}", email);
            Err(AppError::UserNotFound)
        }
    }

    /// Check if user requires payment (no invite and no active payment)
    ///
    /// # Arguments
    /// * `user_id` - The user's UUID
    /// * `email` - The user's email address
    /// * `invite_service` - Reference to invite service for checking invites
    /// * `payment_service` - Reference to payment service for checking payment status
    ///
    /// # Returns
    /// Returns true if user requires payment, false otherwise
    ///
    /// # Errors
    /// Returns database errors if unable to check status
    #[instrument(skip(self, invite_service, payment_service), err(Debug))]
    #[allow(dead_code)]
    pub async fn requires_payment(
        &self,
        user_id: Uuid,
        email: &str,
        invite_service: &crate::services::InviteService,
        payment_service: &crate::services::PaymentService,
    ) -> AppResult<bool> {
        // Check if user has a valid invite
        let has_invite = invite_service.check_invite_exists(email).await?;

        // Check user's payment status
        let payment_status = payment_service.get_user_payment_status(user_id).await?;

        // User requires payment if they have neither invite nor active payment
        Ok(!has_invite && !payment_status.has_active_payment)
    }

    /// Get user's complete registration status including payment requirements
    ///
    /// # Arguments
    /// * `user_id` - The user's UUID
    /// * `email` - The user's email address
    /// * `invite_service` - Reference to invite service
    /// * `payment_service` - Reference to payment service
    ///
    /// # Returns
    /// Returns a tuple of (has_invite, has_active_payment, payment_required)
    ///
    /// # Errors
    /// Returns database errors if unable to check status
    #[instrument(skip(self, invite_service, payment_service), err(Debug))]
    #[allow(dead_code)]
    pub async fn get_registration_status(
        &self,
        user_id: Uuid,
        email: &str,
        invite_service: &crate::services::InviteService,
        payment_service: &crate::services::PaymentService,
    ) -> AppResult<(bool, bool, bool)> {
        let has_invite = invite_service.check_invite_exists(email).await?;
        let payment_status = payment_service.get_user_payment_status(user_id).await?;
        let has_active_payment = payment_status.has_active_payment;
        let payment_required = !has_invite && !has_active_payment;

        Ok((has_invite, has_active_payment, payment_required))
    }
}
