// web-template/server/src/services/auth_service.rs

//! Authentication service for JWT token management
//!
//! This service handles JWT generation, validation, and token claims management.

use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::env;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};

/// JWT Claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // Subject (user ID)
    pub email: String,
    pub exp: i64, // Expiration time (as UTC timestamp)
    pub iat: i64, // Issued at (as UTC timestamp)
}

/// Authentication service for JWT operations
pub struct AuthService {
    encoding_key: EncodingKey,
    #[allow(dead_code)] // Will be used for JWT validation in protected endpoints
    decoding_key: DecodingKey,
}

impl AuthService {
    /// Creates a new `AuthService` instance
    ///
    /// # Errors
    /// Returns `AppError::ConfigurationError` if `JWT_SECRET` environment variable is missing or invalid
    pub fn new() -> AppResult<Self> {
        let jwt_secret = env::var("JWT_SECRET").map_err(|_| {
            AppError::ConfigurationError("JWT_SECRET environment variable is required".to_string())
        })?;

        if jwt_secret.len() < 32 {
            return Err(AppError::ConfigurationError(
                "JWT_SECRET must be at least 32 characters long".to_string(),
            ));
        }

        let encoding_key = EncodingKey::from_secret(jwt_secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(jwt_secret.as_bytes());

        Ok(Self {
            encoding_key,
            decoding_key,
        })
    }

    /// Generates a JWT token for the given user
    ///
    /// # Arguments
    /// * `user_id` - The UUID of the user
    /// * `email` - The email address of the user
    ///
    /// # Returns
    /// A JWT token string
    ///
    /// # Errors
    /// Returns `AppError::InternalServerError` if token generation fails
    pub fn generate_token(&self, user_id: Uuid, email: &str) -> AppResult<String> {
        let now = Utc::now();
        let expiration = now + Duration::hours(24); // Token expires in 24 hours

        let claims = Claims {
            sub: user_id.to_string(),
            email: email.to_string(),
            exp: expiration.timestamp(),
            iat: now.timestamp(),
        };

        encode(&Header::default(), &claims, &self.encoding_key).map_err(|e| {
            tracing::error!("Failed to generate JWT token: {}", e);
            AppError::InternalServerError("Failed to generate authentication token".to_string())
        })
    }

    /// Validates and decodes a JWT token
    ///
    /// # Arguments
    /// * `token` - The JWT token string to validate
    ///
    /// # Returns
    /// The decoded claims if the token is valid
    ///
    /// # Errors
    /// Returns `AppError::Unauthorized` if the token is invalid, expired, or malformed
    #[allow(dead_code)] // Will be used for JWT validation in protected endpoints
    pub fn validate_token(&self, token: &str) -> AppResult<Claims> {
        let validation = Validation::new(Algorithm::HS256);

        decode::<Claims>(token, &self.decoding_key, &validation)
            .map(|token_data| token_data.claims)
            .map_err(|e| {
                tracing::warn!("JWT validation failed: {}", e);
                AppError::Unauthorized("Invalid or expired authentication token".to_string())
            })
    }

    /// Extracts the user ID from a JWT token
    ///
    /// # Arguments
    /// * `token` - The JWT token string
    ///
    /// # Returns
    /// The user UUID if the token is valid
    ///
    /// # Errors
    /// Returns `AppError::Unauthorized` if the token is invalid or the user ID cannot be parsed
    #[allow(dead_code)] // Will be used for JWT validation in protected endpoints
    pub fn get_user_id_from_token(&self, token: &str) -> AppResult<Uuid> {
        let claims = self.validate_token(token)?;

        Uuid::parse_str(&claims.sub).map_err(|e| {
            tracing::error!("Failed to parse user ID from token: {}", e);
            AppError::Unauthorized("Invalid user ID in token".to_string())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn setup_test_env() {
        unsafe {
            env::set_var(
                "JWT_SECRET",
                "test_secret_key_that_is_long_enough_for_testing",
            );
        }
    }

    #[test]
    fn test_auth_service_creation_success() {
        setup_test_env();
        let auth_service = AuthService::new();
        assert!(auth_service.is_ok());
    }

    #[test]
    fn test_auth_service_creation_missing_secret() {
        unsafe {
            env::remove_var("JWT_SECRET");
        }
        let auth_service = AuthService::new();
        assert!(auth_service.is_err());
    }

    #[test]
    fn test_auth_service_creation_short_secret() {
        unsafe {
            env::set_var("JWT_SECRET", "short");
        }
        let auth_service = AuthService::new();
        assert!(auth_service.is_err());
    }

    #[test]
    fn test_token_generation_and_validation() {
        setup_test_env();
        let auth_service = AuthService::new().expect("Failed to create auth service");

        let user_id = Uuid::new_v4();
        let email = "test@example.com";

        let token = auth_service
            .generate_token(user_id, email)
            .expect("Failed to generate token");

        let claims = auth_service
            .validate_token(&token)
            .expect("Failed to validate token");

        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.email, email);
    }

    #[test]
    fn test_get_user_id_from_token() {
        setup_test_env();
        let auth_service = AuthService::new().expect("Failed to create auth service");

        let user_id = Uuid::new_v4();
        let email = "test@example.com";

        let token = auth_service
            .generate_token(user_id, email)
            .expect("Failed to generate token");

        let extracted_user_id = auth_service
            .get_user_id_from_token(&token)
            .expect("Failed to extract user ID");

        assert_eq!(extracted_user_id, user_id);
    }

    #[test]
    fn test_invalid_token_validation() {
        setup_test_env();
        let auth_service = AuthService::new().expect("Failed to create auth service");

        let result = auth_service.validate_token("invalid_token");
        assert!(result.is_err());
    }
}
