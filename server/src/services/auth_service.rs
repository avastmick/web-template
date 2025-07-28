// kanbain/server/src/services/auth_service.rs

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
    use chrono::{Duration, Utc};
    use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
    use std::env;
    use std::sync::Mutex;

    // Global mutex to ensure tests that modify JWT_SECRET run serially
    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    fn setup_test_env() {
        #[allow(unsafe_code)]
        unsafe {
            env::set_var(
                "JWT_SECRET",
                "test_secret_key_that_is_long_enough_for_testing",
            );
        }
    }

    fn cleanup_test_env() {
        #[allow(unsafe_code)]
        unsafe {
            env::remove_var("JWT_SECRET");
        }
    }

    #[test]
    fn test_auth_service_creation_success() {
        setup_test_env();
        let auth_service = AuthService::new();
        assert!(auth_service.is_ok());
        cleanup_test_env();
    }

    #[test]
    fn test_auth_service_creation_missing_secret() {
        let _guard = ENV_MUTEX.lock().expect("Failed to acquire mutex");

        // Save the original value
        let original = env::var("JWT_SECRET").ok();

        // Remove JWT_SECRET
        cleanup_test_env();

        // Test that creation fails
        let auth_service = AuthService::new();
        assert!(auth_service.is_err());
        match auth_service {
            Err(AppError::ConfigurationError(msg)) => {
                assert!(msg.contains("JWT_SECRET environment variable is required"));
            }
            _ => panic!("Expected ConfigurationError"),
        }

        // Restore original value
        if let Some(value) = original {
            #[allow(unsafe_code)]
            unsafe {
                env::set_var("JWT_SECRET", value);
            }
        }
    }

    #[test]
    fn test_auth_service_creation_short_secret() {
        let _guard = ENV_MUTEX.lock().expect("Failed to acquire mutex");

        // Save the original value
        let original = env::var("JWT_SECRET").ok();

        // Set a short secret
        #[allow(unsafe_code)]
        unsafe {
            env::set_var("JWT_SECRET", "short");
        }

        // Test that creation fails
        let auth_service = AuthService::new();
        assert!(auth_service.is_err());
        match auth_service {
            Err(AppError::ConfigurationError(msg)) => {
                assert!(msg.contains("JWT_SECRET must be at least 32 characters long"));
            }
            _ => panic!("Expected ConfigurationError"),
        }

        // Restore original value
        if let Some(value) = original {
            #[allow(unsafe_code)]
            unsafe {
                env::set_var("JWT_SECRET", value);
            }
        }
    }

    #[test]
    fn test_auth_service_creation_exactly_32_chars() {
        #[allow(unsafe_code)]
        unsafe {
            env::set_var("JWT_SECRET", "a".repeat(32));
        }
        let auth_service = AuthService::new();
        assert!(auth_service.is_ok());
        cleanup_test_env();
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
        assert!(claims.exp > claims.iat);
        assert!(claims.exp > Utc::now().timestamp());
        cleanup_test_env();
    }

    #[test]
    fn test_token_expiration_time() {
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

        // Check that token expires in approximately 24 hours
        let now = Utc::now().timestamp();
        let expected_exp = now + Duration::hours(24).num_seconds();
        assert!((claims.exp - expected_exp).abs() < 60); // Within 1 minute tolerance
        cleanup_test_env();
    }

    #[test]
    fn test_token_with_empty_email() {
        setup_test_env();
        let auth_service = AuthService::new().expect("Failed to create auth service");

        let user_id = Uuid::new_v4();
        let email = "";

        let token = auth_service
            .generate_token(user_id, email)
            .expect("Failed to generate token");

        let claims = auth_service
            .validate_token(&token)
            .expect("Failed to validate token");

        assert_eq!(claims.email, "");
        cleanup_test_env();
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
        cleanup_test_env();
    }

    #[test]
    fn test_get_user_id_from_token_with_invalid_uuid() {
        setup_test_env();
        let auth_service = AuthService::new().expect("Failed to create auth service");

        // Create a token with invalid UUID in the sub claim
        let now = Utc::now();
        let expiration = now + Duration::hours(24);

        let claims = Claims {
            sub: "not-a-valid-uuid".to_string(),
            email: "test@example.com".to_string(),
            exp: expiration.timestamp(),
            iat: now.timestamp(),
        };

        let token = encode(&Header::default(), &claims, &auth_service.encoding_key)
            .expect("Failed to encode token");

        let result = auth_service.get_user_id_from_token(&token);
        assert!(result.is_err());
        match result {
            Err(AppError::Unauthorized(msg)) => {
                assert!(msg.contains("Invalid user ID in token"));
            }
            _ => panic!("Expected Unauthorized error"),
        }
        cleanup_test_env();
    }

    #[test]
    fn test_invalid_token_validation() {
        setup_test_env();
        let auth_service = AuthService::new().expect("Failed to create auth service");

        let result = auth_service.validate_token("invalid_token");
        assert!(result.is_err());
        match result {
            Err(AppError::Unauthorized(msg)) => {
                assert!(msg.contains("Invalid or expired authentication token"));
            }
            _ => panic!("Expected Unauthorized error"),
        }
        cleanup_test_env();
    }

    #[test]
    fn test_expired_token_validation() {
        setup_test_env();
        let auth_service = AuthService::new().expect("Failed to create auth service");

        // Create an expired token
        let now = Utc::now();
        let past = now - Duration::hours(25); // Expired 1 hour ago

        let claims = Claims {
            sub: Uuid::new_v4().to_string(),
            email: "test@example.com".to_string(),
            exp: past.timestamp(),
            iat: (past - Duration::hours(24)).timestamp(),
        };

        let token = encode(&Header::default(), &claims, &auth_service.encoding_key)
            .expect("Failed to encode token");

        let result = auth_service.validate_token(&token);
        assert!(result.is_err());
        match result {
            Err(AppError::Unauthorized(msg)) => {
                assert!(msg.contains("Invalid or expired authentication token"));
            }
            _ => panic!("Expected Unauthorized error"),
        }
        cleanup_test_env();
    }

    #[test]
    fn test_token_with_wrong_algorithm() {
        setup_test_env();
        let auth_service = AuthService::new().expect("Failed to create auth service");

        // Create a token with a different algorithm
        let now = Utc::now();
        let expiration = now + Duration::hours(24);

        let claims = Claims {
            sub: Uuid::new_v4().to_string(),
            email: "test@example.com".to_string(),
            exp: expiration.timestamp(),
            iat: now.timestamp(),
        };

        // Create a token with HS384 instead of HS256
        let header = Header::new(Algorithm::HS384);
        let token =
            encode(&header, &claims, &auth_service.encoding_key).expect("Failed to encode token");

        let result = auth_service.validate_token(&token);
        assert!(result.is_err());
        cleanup_test_env();
    }

    #[test]
    fn test_token_with_different_secret() {
        setup_test_env();
        let auth_service = AuthService::new().expect("Failed to create auth service");

        // Create a token with a different secret
        let different_key = EncodingKey::from_secret(b"different_secret_key_that_is_long_enough");
        let now = Utc::now();
        let expiration = now + Duration::hours(24);

        let claims = Claims {
            sub: Uuid::new_v4().to_string(),
            email: "test@example.com".to_string(),
            exp: expiration.timestamp(),
            iat: now.timestamp(),
        };

        let token =
            encode(&Header::default(), &claims, &different_key).expect("Failed to encode token");

        let result = auth_service.validate_token(&token);
        assert!(result.is_err());
        cleanup_test_env();
    }

    #[test]
    fn test_malformed_token_validation() {
        setup_test_env();
        let auth_service = AuthService::new().expect("Failed to create auth service");

        // Test various malformed tokens
        let malformed_tokens = vec![
            "",
            "not.a.jwt",
            "too.many.dots.here.invalid",
            "missing_dots",
            "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9", // Only header, no payload or signature
            "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ0ZXN0In0", // Missing signature
        ];

        for token in malformed_tokens {
            let result = auth_service.validate_token(token);
            assert!(result.is_err(), "Token '{token}' should be invalid");
        }
        cleanup_test_env();
    }

    #[test]
    fn test_claims_serialization() {
        let user_id = Uuid::new_v4();
        let now = Utc::now();
        let claims = Claims {
            sub: user_id.to_string(),
            email: "test@example.com".to_string(),
            exp: now.timestamp() + 3600,
            iat: now.timestamp(),
        };

        // Test that Claims can be serialized/deserialized
        let serialized = serde_json::to_string(&claims).expect("Failed to serialize claims");
        let deserialized: Claims =
            serde_json::from_str(&serialized).expect("Failed to deserialize claims");

        assert_eq!(deserialized.sub, claims.sub);
        assert_eq!(deserialized.email, claims.email);
        assert_eq!(deserialized.exp, claims.exp);
        assert_eq!(deserialized.iat, claims.iat);
    }
}
