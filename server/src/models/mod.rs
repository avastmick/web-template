// web-template/server/src/models/mod.rs

pub mod invite;
pub mod user;

pub use invite::UserInvite;
pub use user::{User, UserConversionError, UserFromDb};
