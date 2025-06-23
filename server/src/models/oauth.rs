use serde::{Deserialize, Serialize};

/// OAuth provider enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OAuthProvider {
    Google,
    GitHub,
}

impl std::fmt::Display for OAuthProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Google => write!(f, "google"),
            Self::GitHub => write!(f, "github"),
        }
    }
}

/// User information from OAuth provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthUserInfo {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub picture: Option<String>,
    pub provider: OAuthProvider,
}

/// Google OAuth user information response
#[derive(Debug, Deserialize)]
pub struct GoogleUserInfo {
    #[serde(alias = "id")]
    pub sub: String,
    pub email: String,
    pub name: Option<String>,
    pub picture: Option<String>,
    #[serde(alias = "verified_email")]
    pub email_verified: Option<bool>,
}

impl From<GoogleUserInfo> for OAuthUserInfo {
    fn from(google_info: GoogleUserInfo) -> Self {
        Self {
            id: google_info.sub,
            email: google_info.email,
            name: google_info.name,
            picture: google_info.picture,
            provider: OAuthProvider::Google,
        }
    }
}

/// GitHub OAuth user information response
#[derive(Debug, Deserialize)]
pub struct GitHubUserInfo {
    pub id: i64,
    pub login: String,
    pub email: Option<String>,
    pub name: Option<String>,
    #[serde(alias = "avatar_url")]
    pub avatar_url: Option<String>,
}

/// GitHub email information response
#[derive(Debug, Deserialize)]
pub struct GitHubEmail {
    pub email: String,
    pub primary: bool,
    pub verified: bool,
}

impl GitHubUserInfo {
    /// Convert GitHub user info to OAuth user info
    /// Note: Email might need to be fetched separately if not public
    #[must_use]
    pub fn into_oauth_user_info(self, email: String) -> OAuthUserInfo {
        OAuthUserInfo {
            id: self.id.to_string(),
            email,
            name: self.name.or(Some(self.login)),
            picture: self.avatar_url,
            provider: OAuthProvider::GitHub,
        }
    }
}
