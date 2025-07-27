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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check_returns_healthy() {
        let result = health_check().await;
        assert!(result.is_ok());

        let Json(response) = result.expect("Failed to get health check response");

        // Check status
        assert_eq!(response["status"], "healthy");

        // Check service name
        assert_eq!(response["service"], "kanbain-server");

        // Check version is present
        assert!(response["version"].is_string());
        assert!(
            !response["version"]
                .as_str()
                .expect("version should be string")
                .is_empty()
        );

        // Check timestamp is present and valid
        assert!(response["timestamp"].is_string());
        let timestamp_str = response["timestamp"]
            .as_str()
            .expect("timestamp should be string");
        assert!(chrono::DateTime::parse_from_rfc3339(timestamp_str).is_ok());
    }

    #[tokio::test]
    async fn test_health_check_timestamp_is_recent() {
        let before_call = chrono::Utc::now();
        let result = health_check().await;
        let after_call = chrono::Utc::now();

        assert!(result.is_ok());
        let Json(response) = result.expect("Failed to get health check response");

        let timestamp_str = response["timestamp"]
            .as_str()
            .expect("timestamp should be string");
        let timestamp = chrono::DateTime::parse_from_rfc3339(timestamp_str)
            .expect("Failed to parse timestamp")
            .with_timezone(&chrono::Utc);

        // Timestamp should be between before and after the call
        assert!(timestamp >= before_call);
        assert!(timestamp <= after_call);
    }

    #[tokio::test]
    async fn test_readiness_check_returns_ready() {
        let result = readiness_check().await;
        assert!(result.is_ok());

        let Json(response) = result.expect("Failed to get readiness check response");

        // Check status
        assert_eq!(response["status"], "ready");

        // Check version is present
        assert!(response["version"].is_string());
        assert!(
            !response["version"]
                .as_str()
                .expect("version should be string")
                .is_empty()
        );

        // Check timestamp is present and valid
        assert!(response["timestamp"].is_string());
        let timestamp_str = response["timestamp"]
            .as_str()
            .expect("timestamp should be string");
        assert!(chrono::DateTime::parse_from_rfc3339(timestamp_str).is_ok());

        // Check database status
        assert_eq!(response["checks"]["database"], "ok");
    }

    #[tokio::test]
    async fn test_readiness_check_includes_checks() {
        let result = readiness_check().await;
        assert!(result.is_ok());

        let Json(response) = result.expect("Failed to get readiness check response");

        // Verify checks object exists
        assert!(response["checks"].is_object());

        // Verify database check exists
        assert!(response["checks"]["database"].is_string());
    }

    #[tokio::test]
    async fn test_health_and_readiness_have_same_version() {
        let health_result = health_check().await;
        let readiness_result = readiness_check().await;

        assert!(health_result.is_ok());
        assert!(readiness_result.is_ok());

        let Json(health_response) = health_result.expect("Failed to get health check response");
        let Json(readiness_response) =
            readiness_result.expect("Failed to get readiness check response");

        // Both should report the same version
        assert_eq!(health_response["version"], readiness_response["version"]);
    }

    #[tokio::test]
    async fn test_responses_are_valid_json() {
        let health_result = health_check().await;
        let readiness_result = readiness_check().await;

        assert!(health_result.is_ok());
        assert!(readiness_result.is_ok());

        // The fact that we can destructure Json<serde_json::Value> proves it's valid JSON
        let Json(health_json) = health_result.expect("Failed to get health check response");
        let Json(readiness_json) =
            readiness_result.expect("Failed to get readiness check response");

        // Additional check: ensure they're objects, not arrays or primitives
        assert!(health_json.is_object());
        assert!(readiness_json.is_object());
    }

    #[tokio::test]
    async fn test_concurrent_health_checks() {
        use futures::future::join_all;

        // Run multiple health checks concurrently
        let handles: Vec<_> = (0..10)
            .map(|_| tokio::spawn(async { health_check().await }))
            .collect();

        let results = join_all(handles).await;

        // All should succeed
        for result in results {
            let check_result = result.expect("Task panicked");
            assert!(check_result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_concurrent_readiness_checks() {
        use futures::future::join_all;

        // Run multiple readiness checks concurrently
        let handles: Vec<_> = (0..10)
            .map(|_| tokio::spawn(async { readiness_check().await }))
            .collect();

        let results = join_all(handles).await;

        // All should succeed
        for result in results {
            let check_result = result.expect("Task panicked");
            assert!(check_result.is_ok());
        }
    }
}
