#![allow(clippy::missing_errors_doc)]

use oauth2::{AuthorizationCode, TokenResponse, basic::BasicTokenResponse, reqwest};
use reqwest::Client;

use crate::{
    config::OAuthConfig,
    errors::AppError,
    models::oauth::{GitHubEmail, GitHubUserInfo, GoogleUserInfo, OAuthUserInfo},
};

pub struct OAuthService {
    config: OAuthConfig,
    http_client: Client,
}

impl OAuthService {
    /// Creates a new OAuth service
    ///
    /// # Errors
    ///
    /// Returns an error if OAuth configuration fails
    pub fn new() -> Result<Self, AppError> {
        let config = OAuthConfig::new()?;
        let http_client = Client::new();

        Ok(Self {
            config,
            http_client,
        })
    }

    /// Get Google OAuth authorization URL
    #[must_use]
    pub fn get_google_auth_url(&self, state: &str) -> String {
        self.config.get_google_auth_url(state)
    }

    /// Get client URL for redirects
    #[must_use]
    pub fn get_client_url(&self) -> &str {
        &self.config.client_url
    }

    /// Exchange authorization code for access token and get user info
    pub async fn handle_google_callback(&self, code: &str) -> Result<OAuthUserInfo, AppError> {
        // Exchange authorization code for access token
        let http_client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| {
                tracing::error!("Failed to build HTTP client: {:?}", e);
                AppError::InternalServerError("HTTP client configuration error".to_string())
            })?;

        let google_client = self.config.create_google_client();
        let token = google_client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .request_async(&http_client)
            .await
            .map_err(|e| {
                tracing::error!("Failed to exchange OAuth code: {:?}", e);
                AppError::Unauthorized("Invalid authorization code".to_string())
            })?;

        // Get user information from Google
        let user_info = self.get_google_user_info(&token).await?;

        // Verify email is verified
        if user_info.email_verified == Some(false) {
            return Err(AppError::Forbidden(
                "Email address must be verified".to_string(),
            ));
        }

        Ok(user_info.into())
    }

    /// Get user information from Google using access token
    async fn get_google_user_info(
        &self,
        token: &BasicTokenResponse,
    ) -> Result<GoogleUserInfo, AppError> {
        let access_token = token.access_token().secret();

        let response = self
            .http_client
            .get("https://www.googleapis.com/oauth2/v2/userinfo")
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch Google user info: {:?}", e);
                AppError::InternalServerError("Failed to fetch user information".to_string())
            })?;

        if !response.status().is_success() {
            tracing::error!("Google API returned error: {}", response.status());
            return Err(AppError::InternalServerError(
                "Failed to fetch user information".to_string(),
            ));
        }

        // First get the raw response text to debug
        let response_text = response.text().await.map_err(|e| {
            tracing::error!("Failed to read Google user info response: {:?}", e);
            AppError::InternalServerError("Failed to read user information".to_string())
        })?;

        tracing::debug!("Google user info response: {}", response_text);

        let user_info: GoogleUserInfo = serde_json::from_str(&response_text).map_err(|e| {
            tracing::error!("Failed to parse Google user info: {:?}", e);
            tracing::error!("Raw response: {}", response_text);
            AppError::InternalServerError("Invalid user information format".to_string())
        })?;

        Ok(user_info)
    }

    /// Get GitHub OAuth authorization URL
    #[must_use]
    pub fn get_github_auth_url(&self, state: &str) -> String {
        self.config.get_github_auth_url(state)
    }

    /// Exchange GitHub authorization code for access token and get user info
    pub async fn handle_github_callback(&self, code: &str) -> Result<OAuthUserInfo, AppError> {
        // Exchange authorization code for access token
        let http_client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| {
                tracing::error!("Failed to build HTTP client: {:?}", e);
                AppError::InternalServerError("HTTP client configuration error".to_string())
            })?;

        let github_client = self.config.create_github_client();
        let token = github_client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .request_async(&http_client)
            .await
            .map_err(|e| {
                tracing::error!("Failed to exchange GitHub OAuth code: {:?}", e);
                AppError::Unauthorized("Invalid authorization code".to_string())
            })?;

        // Get user information from GitHub
        let user_info = self.get_github_user_info(&token).await?;

        Ok(user_info)
    }

    /// Get user information from GitHub using access token
    async fn get_github_user_info(
        &self,
        token: &BasicTokenResponse,
    ) -> Result<OAuthUserInfo, AppError> {
        let access_token = token.access_token().secret();

        // Get basic user info
        let user_response = self
            .http_client
            .get("https://api.github.com/user")
            .header("Accept", "application/vnd.github.v3+json")
            .header("User-Agent", "web-template-oauth")
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch GitHub user info: {:?}", e);
                AppError::InternalServerError("Failed to fetch user information".to_string())
            })?;

        if !user_response.status().is_success() {
            tracing::error!("GitHub API returned error: {}", user_response.status());
            return Err(AppError::InternalServerError(
                "Failed to fetch user information".to_string(),
            ));
        }

        let response_text = user_response.text().await.map_err(|e| {
            tracing::error!("Failed to read GitHub user info response: {:?}", e);
            AppError::InternalServerError("Failed to read user information".to_string())
        })?;

        tracing::debug!("GitHub user info response: {}", response_text);

        let user_info: GitHubUserInfo = serde_json::from_str(&response_text).map_err(|e| {
            tracing::error!("Failed to parse GitHub user info: {:?}", e);
            tracing::error!("Raw response: {}", response_text);
            AppError::InternalServerError("Invalid user information format".to_string())
        })?;

        // If email is not public, fetch it separately
        let email = if let Some(email) = user_info.email.clone() {
            email
        } else {
            // Fetch emails endpoint
            let emails_response = self
                .http_client
                .get("https://api.github.com/user/emails")
                .header("Accept", "application/vnd.github.v3+json")
                .header("User-Agent", "web-template-oauth")
                .bearer_auth(access_token)
                .send()
                .await
                .map_err(|e| {
                    tracing::error!("Failed to fetch GitHub emails: {:?}", e);
                    AppError::InternalServerError("Failed to fetch email information".to_string())
                })?;

            if !emails_response.status().is_success() {
                tracing::error!(
                    "GitHub emails API returned error: {}",
                    emails_response.status()
                );
                return Err(AppError::InternalServerError(
                    "Failed to fetch email information".to_string(),
                ));
            }

            let emails: Vec<GitHubEmail> = emails_response.json().await.map_err(|e| {
                tracing::error!("Failed to parse GitHub emails: {:?}", e);
                AppError::InternalServerError("Invalid email information format".to_string())
            })?;

            // Find primary verified email
            emails
                .into_iter()
                .find(|e| e.primary && e.verified)
                .map(|e| e.email)
                .ok_or_else(|| AppError::Forbidden("No verified primary email found".to_string()))?
        };

        Ok(user_info.into_oauth_user_info(email))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn setup_test_env() {
        unsafe {
            env::set_var("GOOGLE_CLIENT_ID", "test_client_id");
            env::set_var("GOOGLE_CLIENT_SECRET", "test_client_secret");
            env::set_var("SERVER_URL", "http://localhost:8081");
        }
    }

    #[test]
    fn test_oauth_service_creation() {
        setup_test_env();
        let service = OAuthService::new();
        assert!(service.is_ok());
    }

    #[test]
    fn test_get_google_auth_url() {
        setup_test_env();
        let service = OAuthService::new().unwrap();
        let auth_url = service.get_google_auth_url("test_state");

        assert!(auth_url.contains("accounts.google.com"));
        assert!(auth_url.contains("test_state"));
    }

    #[test]
    fn test_oauth_service_creation_missing_google_config() {
        // Clear all OAuth env vars first
        unsafe {
            env::remove_var("GOOGLE_CLIENT_ID");
            env::remove_var("GOOGLE_CLIENT_SECRET");
            env::remove_var("GITHUB_CLIENT_ID");
            env::remove_var("GITHUB_CLIENT_SECRET");
        }

        let service = OAuthService::new();
        assert!(service.is_err());
    }

    #[test]
    fn test_get_github_auth_url() {
        setup_test_env();
        unsafe {
            env::set_var("GITHUB_CLIENT_ID", "test_github_client_id");
            env::set_var("GITHUB_CLIENT_SECRET", "test_github_client_secret");
        }
        let service = OAuthService::new().unwrap();
        let auth_url = service.get_github_auth_url("test_state");

        assert!(auth_url.contains("github.com"));
        assert!(auth_url.contains("test_state"));
    }
}
