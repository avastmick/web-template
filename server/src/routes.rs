// web-template/server/src/routes.rs

use axum::{
    Router,
    routing::{get, post},
};
use std::sync::Arc;

use crate::handlers::{
    auth_handler::{AppState, login_user_handler, register_user_handler},
    user_handler::get_current_user_handler,
};
use crate::services::{AuthService, UserServiceImpl};

// This function will create and return the main application router.
// It takes the shared application state (UserServiceImpl and AuthService) as an argument.
pub fn create_router(user_service: Arc<UserServiceImpl>, auth_service: Arc<AuthService>) -> Router {
    let app_state = Arc::new(AppState {
        user_service,
        auth_service,
    });

    Router::new()
        // Authentication routes
        .route("/api/auth/register", post(register_user_handler))
        .route("/api/auth/login", post(login_user_handler))
        // Protected user routes
        .route("/api/users/me", get(get_current_user_handler))
        // Add other routes here (e.g., for other resources)
        .with_state(app_state)
}

// Example of how you might structure nested routes if needed later:
// fn user_routes() -> Router<Arc<UserServiceImpl>> {
//     Router::new()
//         .route("/me", get(get_current_user_handler))
// }

// Ensure UserServiceImpl is cloneable if it's part of a larger AppState
// that itself isn't cloneable directly for Axum state.
// Arc<UserServiceImpl> is already cloneable.
