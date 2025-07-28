// kanbain/server/src/routes.rs

use axum::http::{HeaderValue, Method, StatusCode};
use axum::{
    Router,
    routing::get_service,
    routing::{get, post},
};
use std::{env, sync::Arc};
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};

use crate::core::AppState;
use crate::handlers::{
    ai_handler::{
        ai_info_handler, archive_conversation_handler, chat_handler, chat_stream_handler,
        code_analysis_handler, contextual_chat_handler, create_invite_handler,
        delete_conversation_handler, delete_invite_handler, demo_message_handler,
        error_demo_handler, get_conversation_handler, get_conversations_handler,
        get_invite_handler, get_usage_stats_handler, health_check_handler, list_invites_handler,
        moderate_content_handler, upload_file_handler, verify_token_handler,
    },
    auth_handler::{login_user_handler, register_user_handler},
    health_handler::{health_check, readiness_check},
    oauth_handler::{
        OAuthAppState, github_login_init, github_oauth_callback, google_login_init,
        google_oauth_callback,
    },
    payment_handler::{
        create_payment_intent_handler, get_payment_status_handler, stripe_webhook_handler,
    },
    user_handler::get_current_user_handler,
};
use crate::services::{
    AiDataService, AiService, AuthService, InviteService, OAuthService, PaymentService,
    UserServiceImpl,
};

/// Create AI routes
fn ai_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/ai/chat", post(chat_handler))
        .route("/api/ai/chat/stream", get(chat_stream_handler))
        .route("/api/ai/chat/contextual", post(contextual_chat_handler))
        .route("/api/ai/analyze/code", post(code_analysis_handler))
        .route("/api/ai/upload", post(upload_file_handler))
        .route("/api/ai/conversations", get(get_conversations_handler))
        .route("/api/ai/conversations/{id}", get(get_conversation_handler))
        .route(
            "/api/ai/conversations/{id}",
            axum::routing::delete(delete_conversation_handler),
        )
        .route(
            "/api/ai/conversations/{id}/archive",
            post(archive_conversation_handler),
        )
        .route("/api/ai/usage", get(get_usage_stats_handler))
        .route("/api/ai/health", get(health_check_handler))
        .route("/api/ai/moderate", post(moderate_content_handler))
        .route("/api/ai/info", get(ai_info_handler))
}

/// Create CORS layer
fn create_cors_layer() -> Result<CorsLayer, Box<dyn std::error::Error>> {
    let allowed_origins = env::var("ALLOWED_ORIGINS").unwrap_or_else(|_| {
        let client_port = env::var("CLIENT_PORT").unwrap_or_else(|_| "8080".to_string());
        format!("http://localhost:{client_port}")
    });

    let first_origin = allowed_origins
        .split(',')
        .next()
        .unwrap_or("http://localhost:8080")
        .trim();

    Ok(CorsLayer::new()
        .allow_origin(
            first_origin
                .parse::<HeaderValue>()
                .map_err(|e| format!("Invalid CORS origin header value '{first_origin}': {e}"))?,
        )
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::DELETE,
            Method::PUT,
            Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
        ]))
}

/// Creates and returns the main application router.
/// It takes the shared application state (`UserServiceImpl`, `AuthService`, `InviteService`, `AiService`, and `OAuthService`) as an argument.
///
/// # Environment Variables
///
/// - `ALLOWED_ORIGINS`: Comma-separated list of allowed CORS origins (first origin is used, e.g., "http://localhost:8080,https://example.com")
/// - `CLIENT_PORT`: Used as fallback to construct default origin if `ALLOWED_ORIGINS` is not set
///
/// # Panics
///
/// # Errors
///
/// Returns an error if services cannot be initialized or routes cannot be configured.
///
/// # Panics
///
/// Panics if the `CLIENT_PORT` environment variable contains an invalid port number
/// that cannot be formatted into a valid HTTP origin URL.
pub fn create_router(
    user_service: Arc<UserServiceImpl>,
    auth_service: Arc<AuthService>,
    invite_service: Arc<InviteService>,
    oauth_service: Arc<OAuthService>,
    db_pool: &sqlx::SqlitePool,
) -> Result<Router, Box<dyn std::error::Error>> {
    // Initialize AI services
    let ai_service = AiService::new()?;
    let ai_data_service = AiDataService::new(db_pool.clone());

    // Initialize Payment service
    let payment_service = PaymentService::new(db_pool.clone())?;

    let app_state = Arc::new(AppState {
        user: user_service,
        auth: auth_service,
        invite: invite_service,
        ai: Arc::new(tokio::sync::RwLock::new(ai_service)),
        ai_data: Arc::new(ai_data_service),
        payment: Arc::new(payment_service),
    });

    let oauth_app_state = OAuthAppState {
        app_state: app_state.clone(),
        oauth_service,
    };

    // Create OAuth routes with their own state
    let oauth_router = Router::new()
        .route("/api/auth/oauth/google", get(google_login_init))
        .route(
            "/api/auth/oauth/google/callback",
            get(google_oauth_callback),
        )
        .route("/api/auth/oauth/github", get(github_login_init))
        .route(
            "/api/auth/oauth/github/callback",
            get(github_oauth_callback),
        )
        .with_state(oauth_app_state);

    let static_dir = env::var("STATIC_DIR").unwrap_or_else(|_| "./static".to_string());

    // Main router with standard auth routes
    let router = Router::new()
        // Health check endpoints (no authentication needed)
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
        // Authentication routes
        .route("/api/auth/register", post(register_user_handler))
        .route("/api/auth/login", post(login_user_handler))
        .route("/api/auth/verify", get(verify_token_handler))
        // Protected user routes
        .route("/api/users/me", get(get_current_user_handler))
        // Payment routes
        .route("/api/payment/status", get(get_payment_status_handler))
        .route(
            "/api/payment/create-intent",
            post(create_payment_intent_handler),
        )
        .route("/api/webhooks/stripe", post(stripe_webhook_handler))
        // Admin routes (invite management)
        .route("/api/admin/invites", get(list_invites_handler))
        .route("/api/admin/invites", post(create_invite_handler))
        .route(
            "/api/admin/invites/{id}",
            axum::routing::delete(delete_invite_handler),
        )
        .route("/api/invites/{email}", get(get_invite_handler))
        // Debug/development routes
        .route("/api/debug/error/{error_type}", get(error_demo_handler))
        .route("/api/debug/message", get(demo_message_handler))
        .merge(ai_routes())
        .with_state(app_state)
        // Merge OAuth routes
        .merge(oauth_router)
        // Serve static files with SPA fallback - this should be last to catch all unmatched routes
        .fallback(
            get_service(
                ServeDir::new(&static_dir)
                    .fallback(ServeFile::new(format!("{static_dir}/index.html"))),
            )
            .handle_error(|_| async move {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to serve front-end application.",
                )
            }),
        )
        .layer(create_cors_layer()?);

    Ok(router)
}

// Example of how you might structure nested routes if needed later:
// fn user_routes() -> Router<Arc<UserServiceImpl>> {
//     Router::new()
//         .route("/me", get(get_current_user_handler))
// }

// Ensure UserServiceImpl is cloneable if it's part of a larger AppState
// that itself isn't cloneable directly for Axum state.
// Arc<UserServiceImpl> is already cloneable.
