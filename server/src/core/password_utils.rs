// kanbain/server/src/core/password_utils.rs

use argon2::{
    Argon2,
    password_hash::{
        Error as PasswordHashError, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
        rand_core::OsRng,
    },
};
use std::fmt;

// Custom error type for password operations
#[derive(Debug)]
pub enum PasswordError {
    HashingError(PasswordHashError),
    VerificationError(PasswordHashError),
}

impl fmt::Display for PasswordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PasswordError::HashingError(e) => write!(f, "Password hashing failed: {e}"),
            PasswordError::VerificationError(e) => write!(f, "Password verification failed: {e}"),
        }
    }
}

impl std::error::Error for PasswordError {}

/// Hashes a given password string using Argon2.
///
/// # Arguments
/// * `password` - The plaintext password to hash.
///
/// # Returns
/// A `Result` containing the hashed password string or a `PasswordError`.
///
/// # Errors
/// Returns `PasswordError::HashingError` if the Argon2 hashing process fails.
pub fn hash_password(password: &str) -> Result<String, PasswordError> {
    let salt = SaltString::generate(&mut OsRng);
    // Argon2::default() provides sensible defaults for hashing parameters.
    // These can be configured if specific needs arise (e.g., argon2::Params).
    let argon2 = Argon2::default();

    match argon2.hash_password(password.as_bytes(), &salt) {
        Ok(password_hash) => Ok(password_hash.to_string()),
        Err(e) => Err(PasswordError::HashingError(e)),
    }
}

/// Verifies a given password against a stored hashed password.
///
/// # Arguments
/// * `password` - The plaintext password to verify.
/// * `hashed_password_str` - The stored hashed password string (PHC format).
///
/// # Returns
/// A `Result` indicating whether verification was successful (`Ok(())`) or an error.
/// `PasswordError::VerificationError` with `argon2::password_hash::Error::Password` indicates a mismatch.
///
/// # Errors
/// Returns `PasswordError::VerificationError` if the hash string is invalid or password verification fails.
pub fn verify_password(password: &str, hashed_password_str: &str) -> Result<(), PasswordError> {
    let parsed_hash = match PasswordHash::new(hashed_password_str) {
        Ok(hash) => hash,
        Err(e) => return Err(PasswordError::VerificationError(e)), // Error parsing the hash string
    };

    match Argon2::default().verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(()),
        Err(e) => Err(PasswordError::VerificationError(e)), // Error during verification (e.g., mismatch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify_password_success() {
        let password = "mySecurePassword123!";
        let hashed_password = hash_password(password).expect("Failed to hash password");

        assert_ne!(password, hashed_password); // Ensure hash is not the same as password
        verify_password(password, &hashed_password).expect("Password verification failed");
    }

    #[test]
    fn test_verify_password_failure_wrong_password() {
        let password = "mySecurePassword123!";
        let wrong_password = "WrongPassword!";
        let hashed_password = hash_password(password).expect("Failed to hash password");

        let result = verify_password(wrong_password, &hashed_password);
        assert!(matches!(
            result,
            Err(PasswordError::VerificationError(
                PasswordHashError::Password
            ))
        ));
    }

    #[test]
    fn test_verify_password_failure_invalid_hash() {
        let password = "mySecurePassword123!";
        let invalid_hash = "not_a_valid_phc_string";

        let result = verify_password(password, invalid_hash);
        assert!(matches!(result, Err(PasswordError::VerificationError(_))));
        // More specific error check if needed, e.g. Error::PhcStringField
    }

    #[test]
    fn test_hash_password_empty_string() {
        let password = "";
        let result = hash_password(password);
        assert!(result.is_ok());
        let hashed = result.expect("Failed to hash empty password");
        verify_password(password, &hashed).expect("Failed to verify empty password");
    }

    #[test]
    fn test_hash_password_very_long_password() {
        let password = "a".repeat(1000);
        let result = hash_password(&password);
        assert!(result.is_ok());
        let hashed = result.expect("Failed to hash long password");
        verify_password(&password, &hashed).expect("Failed to verify long password");
    }

    #[test]
    fn test_hash_password_special_characters() {
        let password = "🔐🔑😀 Special !@#$%^&*()_+-=[]{}|;':\",./<>?";
        let result = hash_password(password);
        assert!(result.is_ok());
        let hashed = result.expect("Failed to hash special character password");
        verify_password(password, &hashed).expect("Failed to verify special character password");
    }

    #[test]
    fn test_hash_password_produces_different_hashes() {
        let password = "samePassword123!";
        let hash1 = hash_password(password).expect("Failed to hash password");
        let hash2 = hash_password(password).expect("Failed to hash password");

        // Same password should produce different hashes due to random salt
        assert_ne!(hash1, hash2);

        // But both should verify correctly
        verify_password(password, &hash1).expect("Failed to verify hash1");
        verify_password(password, &hash2).expect("Failed to verify hash2");
    }

    #[test]
    fn test_password_error_display() {
        // Test Display implementation for PasswordError
        let hash_error = PasswordError::HashingError(PasswordHashError::Password);
        assert_eq!(
            format!("{hash_error}"),
            "Password hashing failed: invalid password"
        );

        let verify_error = PasswordError::VerificationError(PasswordHashError::Password);
        assert_eq!(
            format!("{verify_error}"),
            "Password verification failed: invalid password"
        );
    }

    #[test]
    fn test_password_error_debug() {
        // Test Debug implementation for PasswordError
        let error = PasswordError::HashingError(PasswordHashError::Password);
        let debug_str = format!("{error:?}");
        assert!(debug_str.contains("HashingError"));
    }

    #[test]
    fn test_verify_password_empty_hash() {
        let password = "test123";
        let result = verify_password(password, "");
        assert!(matches!(result, Err(PasswordError::VerificationError(_))));
    }

    #[test]
    fn test_verify_password_malformed_phc_string() {
        let password = "test123";
        // PHC format requires $ separators
        let malformed_hash = "$argon2id$invalid";
        let result = verify_password(password, malformed_hash);
        assert!(matches!(result, Err(PasswordError::VerificationError(_))));
    }
}
