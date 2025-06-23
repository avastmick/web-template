// web-template/server/src/core/password_utils.rs

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
}
