// web-template/server/src/routes.rs

use axum::{Router, routing::post};
use std::sync::Arc;

// Assuming UserServiceImpl is the state we'll pass.
// If using a more general AppState struct, replace UserServiceImpl with AppState.
use crate::handlers::auth_handler::register_user_handler;
use crate::services::UserServiceImpl; // Import the handler

// This function will create and return the main application router.
// It takes the shared application state (e.g., UserServiceImpl) as an argument.
pub fn create_router(user_service: Arc<UserServiceImpl>) -> Router {
    Router::new()
        // Authentication routes
        .route("/api/auth/register", post(register_user_handler))
        // .route("/api/auth/login", post(login_handler)) // Placeholder for login
        // Add other routes here (e.g., for users, other resources)
        // .nest("/api/users", user_routes())
        .with_state(user_service) // Make UserServiceImpl available to all routes
}

// Example of how you might structure nested routes if needed later:
// fn user_routes() -> Router<Arc<UserServiceImpl>> {
//     Router::new()
//         .route("/me", get(get_current_user_handler))
// }

// Ensure UserServiceImpl is cloneable if it's part of a larger AppState
// that itself isn't cloneable directly for Axum state.
// Arc<UserServiceImpl> is already cloneable.
