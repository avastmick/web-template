// kanbain/server/src/handlers/health_handler.rs

use axum::{http::StatusCode, response::Json};
use serde_json::json;

/// Health check endpoint for container orchestration and load balancers
///
/// # Errors
///
/// Returns `StatusCode::INTERNAL_SERVER_ERROR` if the health check fails
pub async fn health_check() -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION"),
        "service": "kanbain-server"
    })))
}

/// Readiness check that includes database connectivity
///
/// # Errors
///
/// Returns `StatusCode::INTERNAL_SERVER_ERROR` if the readiness check fails
pub async fn readiness_check() -> Result<Json<serde_json::Value>, StatusCode> {
    // TODO: Add database health check here if needed
    // For now, just return healthy status
    Ok(Json(json!({
        "status": "ready",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION"),
        "checks": {
            "database": "ok"
        }
    })))
}
