use serde::{Deserialize, Serialize};

/// OAuth provider enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OAuthProvider {
    Google,
}

impl std::fmt::Display for OAuthProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Google => write!(f, "google"),
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
