// web-template/server/src/main.rs

use axum::serve;
use sqlx::sqlite::SqlitePoolOptions;
use std::{env, net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt}; // Corrected import

// Declare modules
mod core;
mod errors;
mod handlers;
mod middleware;
mod models;
mod routes;
mod services;
// mod config; // Future placeholder

use services::{AuthService, UserServiceImpl};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Attempt to load .env file. This is a fallback if direnv is not used or .envrc is not sourced.
    // For direnv users, .envrc should already have populated the environment.
    if let Ok(path) = dotenvy::dotenv() {
        info!(".env file loaded from path: {}", path.display());
    } else {
        info!("No .env file found or failed to load, relying on existing environment variables.");
    }

    // Initialize tracing (logging)
    let rust_log_env =
        env::var("RUST_LOG").unwrap_or_else(|_| "info,server=debug,sqlx=warn".to_string());
    let env_filter = EnvFilter::try_new(&rust_log_env).unwrap_or_else(|_| EnvFilter::new("info")); // Fallback if RUST_LOG is invalid

    tracing_subscriber::registry()
        .with(fmt::layer().with_ansi(true)) // Correctly use fmt::layer()
        .with(env_filter)
        .init();

    info!("Tracing initialized. Server starting...");

    // Database setup
    let database_url = env::var("DATABASE_URL").map_err(|e| {
        tracing::error!("DATABASE_URL must be set: {}", e);
        errors::AppError::ConfigError("DATABASE_URL must be set".to_string())
    })?;

    let db_pool_max_connections_str =
        env::var("DB_POOL_MAX_CONNECTIONS").unwrap_or_else(|_| "5".to_string());
    let db_pool_max_connections: u32 = db_pool_max_connections_str.parse().map_err(|e| {
        tracing::error!(
            "Invalid DB_POOL_MAX_CONNECTIONS value '{}': {}",
            db_pool_max_connections_str,
            e
        );
        errors::AppError::ConfigError(format!(
            "Invalid DB_POOL_MAX_CONNECTIONS: {db_pool_max_connections_str}"
        ))
    })?;

    let db_pool = SqlitePoolOptions::new()
        .max_connections(db_pool_max_connections)
        .connect(&database_url)
        .await
        .map_err(|e| {
            tracing::error!(
                "Failed to create SQLite connection pool for URL '{}': {}",
                database_url,
                e
            );
            errors::AppError::SqlxError(e) // Assuming AppError::SqlxError can take sqlx::Error
        })?;

    info!(
        "Database connection pool initialized for {} (max connections: {})",
        database_url, db_pool_max_connections
    );

    // Initialize services and application state
    let user_service = Arc::new(UserServiceImpl::new(db_pool.clone()));
    let auth_service = Arc::new(AuthService::new().map_err(|e| {
        tracing::error!("Failed to initialize AuthService: {:?}", e);
        Box::new(e) as Box<dyn std::error::Error>
    })?);

    // Create the main application router
    let app = routes::create_router(user_service, auth_service);

    // Server address
    let host_str = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port_str = env::var("PORT").unwrap_or_else(|_| "3000".to_string());

    let port: u16 = port_str.parse().map_err(|e| {
        tracing::error!("Invalid PORT value '{}': {}", port_str, e);
        errors::AppError::ConfigError(format!("Invalid PORT: {port_str}"))
    })?;

    let host = host_str.parse().map_err(|e| {
        tracing::error!("Invalid HOST value '{}': {}", host_str, e);
        errors::AppError::ConfigError(format!("Invalid HOST address: {host_str}"))
    })?;

    let addr = SocketAddr::new(host, port);

    info!("Server configured to listen on http://{}", addr);

    let listener = TcpListener::bind(addr).await.map_err(|e| {
        tracing::error!("Failed to bind TCP listener to {}: {}", addr, e);
        // Convert std::io::Error to a Box<dyn std::error::Error>
        Box::new(e) as Box<dyn std::error::Error>
    })?;

    info!("Server listening on http://{}", listener.local_addr()?);

    serve(listener, app.into_make_service())
        .await
        .map_err(|e| {
            tracing::error!("Server failed: {}", e);
            Box::new(e) as Box<dyn std::error::Error>
        })?;

    Ok(())
}
