use axum::serve;
use sqlx::sqlite::SqlitePoolOptions;
use std::{env, net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt}; // Corrected import

// Declare modules
mod ai;
mod config;
mod core;
mod errors;
mod handlers;
mod middleware;
mod models;
mod routes;
mod services;

use services::{AuthService, InviteService, OAuthService, UserServiceImpl};

/// Initialize tracing/logging
fn initialize_tracing() {
    let rust_log_env =
        env::var("RUST_LOG").unwrap_or_else(|_| "info,server=debug,sqlx=warn".to_string());
    let env_filter = EnvFilter::try_new(&rust_log_env).unwrap_or_else(|_| EnvFilter::new("info")); // Fallback if RUST_LOG is invalid

    tracing_subscriber::registry()
        .with(fmt::layer().with_ansi(true)) // Correctly use fmt::layer()
        .with(env_filter)
        .init();

    info!("Tracing initialized. Server starting...");
}

/// Set up the OAuth state cleanup scheduler
async fn setup_oauth_cleanup_scheduler(
    oauth_service: &Arc<OAuthService>,
) -> Result<(), Box<dyn std::error::Error>> {
    let scheduler = JobScheduler::new().await.map_err(|e| {
        tracing::error!("Failed to create job scheduler: {:?}", e);
        Box::new(e) as Box<dyn std::error::Error>
    })?;

    let oauth_service_for_cleanup = oauth_service.clone();
    scheduler
        .add(
            Job::new_async("0 */10 * * * *", move |_uuid, _l| {
                let oauth_service = oauth_service_for_cleanup.clone();
                Box::pin(async move {
                    match oauth_service.cleanup_expired_states().await {
                        Ok(deleted_count) => {
                            if deleted_count > 0 {
                                tracing::info!("Cleaned up {} expired OAuth states", deleted_count);
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to cleanup expired OAuth states: {:?}", e);
                        }
                    }
                })
            })
            .map_err(|e| {
                tracing::error!("Failed to create cleanup job: {:?}", e);
                Box::new(e) as Box<dyn std::error::Error>
            })?,
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to add cleanup job to scheduler: {:?}", e);
            Box::new(e) as Box<dyn std::error::Error>
        })?;

    scheduler.start().await.map_err(|e| {
        tracing::error!("Failed to start job scheduler: {:?}", e);
        Box::new(e) as Box<dyn std::error::Error>
    })?;

    info!("OAuth state cleanup job scheduled to run every 10 minutes");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Handle command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "--health-check" => return perform_health_check().await,
            "--init-db" => return init_database(),
            _ => {}
        }
    }

    // Attempt to load .env file. This is a fallback if direnv is not used or .envrc is not sourced.
    // For direnv users, .envrc should already have populated the environment.
    if let Ok(path) = dotenvy::dotenv() {
        info!(".env file loaded from path: {}", path.display());
    } else {
        info!("No .env file found or failed to load, relying on existing environment variables.");
    }

    // Initialize tracing (logging)
    initialize_tracing();

    // Database setup
    let database_url = env::var("DATABASE_URL").map_err(|e| {
        tracing::error!("DATABASE_URL must be set: {}", e);
        errors::AppError::ConfigError("DATABASE_URL must be set".to_string())
    })?;

    // Initialize database from template if it doesn't exist (for containerized deployments)
    if init_database_if_needed(&database_url).is_err() {
        tracing::warn!("Could not initialize database from template, assuming it already exists");
    }

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
    let invite_service = Arc::new(InviteService::new(db_pool.clone()));
    let oauth_service = Arc::new(OAuthService::new(db_pool.clone()).map_err(|e| {
        tracing::error!("Failed to initialize OAuthService: {:?}", e);
        Box::new(e) as Box<dyn std::error::Error>
    })?);

    // Set up scheduled cleanup task for OAuth states
    setup_oauth_cleanup_scheduler(&oauth_service).await?;

    // Create the main application router
    let app = routes::create_router(
        user_service,
        auth_service,
        invite_service,
        oauth_service,
        db_pool.clone(),
    )
    .await?;

    // Server address
    let host_str = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port_str = env::var("SERVER_PORT")
        .or_else(|_| env::var("PORT"))
        .unwrap_or_else(|_| "8081".to_string());

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

async fn perform_health_check() -> Result<(), Box<dyn std::error::Error>> {
    // Simple health check that verifies the server can start basic components
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:/data/production.sqlite3?mode=rwc".to_string());

    // Test database connection
    let db_pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await?;

    // Simple query to verify database is accessible
    sqlx::query("SELECT 1").fetch_one(&db_pool).await?;

    println!("Health check passed");
    Ok(())
}

fn init_database() -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;
    use std::path::Path;

    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:/data/production.sqlite3?mode=rwc".to_string());

    println!("Initializing database from template...");

    // Extract the file path from the DATABASE_URL
    if let Some(db_path) = extract_sqlite_path(&database_url) {
        // Ensure the directory exists
        if let Some(parent) = Path::new(&db_path).parent() {
            fs::create_dir_all(parent)?;
        }

        // Copy template database
        fs::copy("/app/db-template.sqlite3", &db_path)?;
        println!("Database initialized successfully at: {db_path}");
    } else {
        return Err("Could not extract database path from DATABASE_URL".into());
    }

    Ok(())
}

fn init_database_if_needed(database_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::path::Path;

    // Extract the file path from the DATABASE_URL
    if let Some(db_path) = extract_sqlite_path(database_url) {
        // Check if database file already exists
        if !Path::new(&db_path).exists() {
            // Copy template database
            use std::fs;

            // Ensure the directory exists
            if let Some(parent) = Path::new(&db_path).parent() {
                fs::create_dir_all(parent)?;
            }

            fs::copy("/app/db-template.sqlite3", &db_path)?;
            tracing::info!("Database initialized from template at: {}", db_path);
        }
    }

    Ok(())
}

fn extract_sqlite_path(database_url: &str) -> Option<String> {
    // Parse SQLite URL format: sqlite:path/to/file.db?options
    if let Some(stripped) = database_url.strip_prefix("sqlite:") {
        if let Some(path_part) = stripped.split('?').next() {
            return Some(path_part.to_string());
        }
    }
    None
}
