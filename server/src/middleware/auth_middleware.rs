// kanbain/server/src/middleware/auth_middleware.rs

//! JWT authentication middleware and extractors for Axum
//!
//! This module provides JWT token extraction and validation for protected endpoints.

use axum::{
    Json,
    extract::FromRequestParts,
    http::{StatusCode, request},
    response::{IntoResponse, Response},
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::{core::AppState, errors::AppError};

/// Represents the authenticated user extracted from JWT
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub email: String,
}

/// JWT token extractor that validates and extracts user information from the Authorization header
pub struct JwtAuth {
    pub user: AuthenticatedUser,
}

impl FromRequestParts<Arc<AppState>> for JwtAuth {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut request::Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        // We already have the app state
        let app_state = state;

        // Extract the Authorization header
        let authorization_header =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, &())
                .await
                .map_err(|_| {
                    tracing::debug!("Missing or invalid Authorization header");
                    (
                        StatusCode::UNAUTHORIZED,
                        Json(json!({"error": "Missing or invalid authorization header"})),
                    )
                        .into_response()
                })?;

        let token = authorization_header.token();

        // Validate the JWT token
        let claims = app_state.auth.validate_token(token).map_err(|e| {
            tracing::warn!("JWT validation failed: {:?}", e);
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Invalid or expired token"})),
            )
                .into_response()
        })?;

        // Parse user ID from claims
        let user_id = Uuid::parse_str(&claims.sub).map_err(|e| {
            tracing::error!("Failed to parse user ID from JWT claims: {}", e);
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Invalid token format"})),
            )
                .into_response()
        })?;

        // Verify user still exists in database
        let _user = app_state
            .user
            .find_by_email(&claims.email)
            .await
            .map_err(|e| {
                if let AppError::UserNotFound = e {
                    tracing::warn!(
                        "JWT contains reference to non-existent user: {}",
                        claims.email
                    );
                    (
                        StatusCode::UNAUTHORIZED,
                        Json(json!({"error": "User no longer exists"})),
                    )
                        .into_response()
                } else {
                    tracing::error!("Database error during user verification: {:?}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({"error": "Internal server error"})),
                    )
                        .into_response()
                }
            })?;

        tracing::debug!("Successfully authenticated user: {}", claims.email);

        Ok(JwtAuth {
            user: AuthenticatedUser {
                user_id,
                email: claims.email,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authenticated_user_creation() {
        let user = AuthenticatedUser {
            user_id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
        };
        assert_eq!(user.email, "test@example.com");
    }
}
