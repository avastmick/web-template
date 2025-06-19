// web-template/server/src/models/mod.rs

pub mod invite;
pub mod oauth;
pub mod user;

pub use invite::UserInvite;
pub use oauth::{GoogleUserInfo, OAuthUserInfo};
pub use user::{User, UserConversionError, UserFromDb};
