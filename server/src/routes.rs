// web-template/server/src/routes.rs

use axum::http::{HeaderValue, Method};
use axum::{
    Router,
    routing::{get, post},
};
use std::{env, sync::Arc};
use tower_http::cors::CorsLayer;

use crate::handlers::{
    auth_handler::{AppState, login_user_handler, register_user_handler},
    user_handler::get_current_user_handler,
};
use crate::services::{AuthService, InviteService, UserServiceImpl};

/// Creates and returns the main application router.
/// It takes the shared application state (`UserServiceImpl`, `AuthService`, and `InviteService`) as an argument.
///
/// # Environment Variables
///
/// - `ALLOWED_ORIGINS`: Comma-separated list of allowed CORS origins (first origin is used, e.g., "http://localhost:8080,https://example.com")
/// - `CLIENT_PORT`: Used as fallback to construct default origin if `ALLOWED_ORIGINS` is not set
///
/// # Panics
///
/// Panics if the `CLIENT_PORT` environment variable contains an invalid port number
/// that cannot be formatted into a valid HTTP origin URL.
pub fn create_router(
    user_service: Arc<UserServiceImpl>,
    auth_service: Arc<AuthService>,
    invite_service: Arc<InviteService>,
) -> Router {
    let app_state = Arc::new(AppState {
        user_service,
        auth_service,
        invite_service,
    });

    Router::new()
        // Authentication routes
        .route("/api/auth/register", post(register_user_handler))
        .route("/api/auth/login", post(login_user_handler))
        // Protected user routes
        .route("/api/users/me", get(get_current_user_handler))
        // Add other routes here (e.g., for other resources)
        .with_state(app_state)
        .layer(
            CorsLayer::new()
                .allow_origin({
                    let allowed_origins = env::var("ALLOWED_ORIGINS").unwrap_or_else(|_| {
                        let client_port =
                            env::var("CLIENT_PORT").unwrap_or_else(|_| "8080".to_string());
                        format!("http://localhost:{client_port}")
                    });

                    // For now, use the first origin (tower-http 0.6.4 doesn't support multiple origins easily)
                    // To support multiple origins, we'd need to upgrade or use a custom middleware
                    let first_origin = allowed_origins
                        .split(',')
                        .next()
                        .unwrap_or("http://localhost:8080")
                        .trim();

                    first_origin.parse::<HeaderValue>().unwrap()
                })
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                .allow_headers([
                    axum::http::header::CONTENT_TYPE,
                    axum::http::header::AUTHORIZATION,
                ]),
        )
}

// Example of how you might structure nested routes if needed later:
// fn user_routes() -> Router<Arc<UserServiceImpl>> {
//     Router::new()
//         .route("/me", get(get_current_user_handler))
// }

// Ensure UserServiceImpl is cloneable if it's part of a larger AppState
// that itself isn't cloneable directly for Axum state.
// Arc<UserServiceImpl> is already cloneable.
