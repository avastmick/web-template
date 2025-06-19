use oauth2::{
    AuthUrl, Client, ClientId, ClientSecret, CsrfToken, EndpointSet, RedirectUrl, Scope,
    StandardRevocableToken, TokenUrl,
    basic::{
        BasicClient, BasicErrorResponse, BasicRevocationErrorResponse,
        BasicTokenIntrospectionResponse, BasicTokenResponse,
    },
};
use std::env;

use crate::errors::AppError;

/// OAuth provider configuration
#[derive(Debug, Clone)]
#[allow(dead_code)] // Will be used in OAuth endpoints
pub struct OAuthConfig {
    pub google_client_id: String,
    pub google_client_secret: String,
    pub redirect_url: String,
    pub client_url: String,
}

#[allow(dead_code)] // Methods will be used in OAuth endpoints
impl OAuthConfig {
    /// Creates a new OAuth configuration
    ///
    /// # Errors
    ///
    /// Returns an error if required environment variables are missing
    /// or if OAuth client configuration fails
    pub fn new() -> Result<Self, AppError> {
        let google_client_id = env::var("GOOGLE_CLIENT_ID").map_err(|_| {
            AppError::ConfigError("GOOGLE_CLIENT_ID environment variable is required".to_string())
        })?;

        let google_client_secret = env::var("GOOGLE_CLIENT_SECRET").map_err(|_| {
            AppError::ConfigError(
                "GOOGLE_CLIENT_SECRET environment variable is required".to_string(),
            )
        })?;

        let server_url =
            env::var("SERVER_URL").unwrap_or_else(|_| "http://localhost:8081".to_string());
        let client_url =
            env::var("CLIENT_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());

        let redirect_url = format!("{server_url}/api/auth/oauth/google/callback");

        Ok(Self {
            google_client_id,
            google_client_secret,
            redirect_url,
            client_url,
        })
    }

    /// Get the Google OAuth authorization URL with required scopes
    #[must_use]
    pub fn get_google_auth_url(&self, state: &str) -> String {
        let google_client = self.create_google_client();
        let (auth_url, _) = google_client
            .authorize_url(|| CsrfToken::new(state.to_string()))
            .add_scope(Scope::new("openid".to_string()))
            .add_scope(Scope::new("email".to_string()))
            .add_scope(Scope::new("profile".to_string()))
            .url();

        auth_url.to_string()
    }

    /// Create a Google OAuth client
    ///
    /// # Panics
    ///
    /// Panics if the Google OAuth URLs are invalid (which should never happen with hardcoded URLs)
    #[must_use]
    pub fn create_google_client(
        &self,
    ) -> Client<
        BasicErrorResponse,
        BasicTokenResponse,
        BasicTokenIntrospectionResponse,
        StandardRevocableToken,
        BasicRevocationErrorResponse,
        EndpointSet,
        oauth2::EndpointNotSet,
        oauth2::EndpointNotSet,
        oauth2::EndpointNotSet,
        EndpointSet,
    > {
        BasicClient::new(ClientId::new(self.google_client_id.clone()))
            .set_client_secret(ClientSecret::new(self.google_client_secret.clone()))
            .set_auth_uri(
                AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
                    .expect("Valid Google auth URL"),
            )
            .set_token_uri(
                TokenUrl::new("https://www.googleapis.com/oauth2/v3/token".to_string())
                    .expect("Valid Google token URL"),
            )
            .set_redirect_uri(
                RedirectUrl::new(self.redirect_url.clone()).expect("Valid redirect URL"),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_oauth_config_creation_success() {
        // Set up test environment variables
        unsafe {
            env::set_var("GOOGLE_CLIENT_ID", "test_client_id");
            env::set_var("GOOGLE_CLIENT_SECRET", "test_client_secret");
            env::set_var("SERVER_URL", "http://localhost:8081");
        }

        let config = OAuthConfig::new();
        assert!(config.is_ok());

        let config = config.unwrap();
        assert_eq!(
            config.redirect_url,
            "http://localhost:8081/api/auth/oauth/google/callback"
        );
    }

    #[test]
    fn test_oauth_config_missing_client_id() {
        unsafe {
            env::remove_var("GOOGLE_CLIENT_ID");
            env::set_var("GOOGLE_CLIENT_SECRET", "test_client_secret");
        }

        let config = OAuthConfig::new();
        assert!(config.is_err());
    }

    #[test]
    fn test_oauth_config_missing_client_secret() {
        unsafe {
            env::set_var("GOOGLE_CLIENT_ID", "test_client_id");
            env::remove_var("GOOGLE_CLIENT_SECRET");
        }

        let config = OAuthConfig::new();
        assert!(config.is_err());
    }

    #[test]
    fn test_get_google_auth_url() {
        unsafe {
            env::set_var("GOOGLE_CLIENT_ID", "test_client_id");
            env::set_var("GOOGLE_CLIENT_SECRET", "test_client_secret");
        }

        let config = OAuthConfig::new().unwrap();
        let auth_url = config.get_google_auth_url("test_state");

        assert!(auth_url.contains("accounts.google.com"));
        assert!(auth_url.contains("openid"));
        assert!(auth_url.contains("email"));
        assert!(auth_url.contains("profile"));
        assert!(auth_url.contains("test_state"));
    }
}
