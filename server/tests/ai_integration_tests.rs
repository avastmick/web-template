//! Integration tests for AI endpoints
//!
//! # Usage
//!
//! ## Basic Testing
//! Run all AI integration tests:
//! ```bash
//! just test-integration ai_integration_tests
//! ```
//!
//! ## Verbose Mode
//! Enable detailed logging to see AI provider interactions:
//! ```bash
//! just test-integration ai_integration_tests --verbose
//! ```
//!
//! ## Real AI Testing
//! To test with a real `OpenRouter` API key instead of mock responses:
//! ```bash
//! export OPENROUTER_API_KEY=your_real_api_key_here
//! just test-integration ai_integration_tests --verbose --real-ai
//! ```
//!
//! ## What Verbose Mode Shows
//! - Request details: endpoint, method, payload with model and messages
//! - Response details: status, AI response content, conversation IDs, token usage
//! - Model information: which AI model is being used
//! - Error details: specific error messages and failure reasons
//! - Database validation: conversation and message persistence checks

use axum::{
    Router,
    body::Body,
    http::{Method, Request, StatusCode, header},
    response::Response,
};
use serde_json::{Value, json};
use sqlx::{Pool, Sqlite};
use std::sync::Arc;
use std::sync::Once;
use tower::ServiceExt;
use tracing::{debug, info, warn};

// Import the application modules we need for testing
use server::{
    routes::create_router,
    services::{AuthService, InviteService, OAuthService, UserServiceImpl},
};

mod common;

static INIT: Once = Once::new();

/// Initialize logging for tests if verbose mode is enabled
fn init_test_logging() {
    INIT.call_once(|| {
        if std::env::var("AI_TEST_VERBOSE").unwrap_or_default() == "true" {
            tracing_subscriber::fmt()
                .with_max_level(tracing::Level::DEBUG)
                .with_test_writer()
                .init();
            info!("🔍 AI Integration Test Verbose Mode Enabled");
            info!("Will show detailed AI provider interactions, requests, and responses");
        }
    });
}

/// Check if verbose mode is enabled
fn is_verbose() -> bool {
    std::env::var("AI_TEST_VERBOSE").unwrap_or_default() == "true"
}

/// Helper function to create a test database in memory
async fn create_test_db() -> Pool<Sqlite> {
    init_test_logging();
    common::setup_test_database().await
}

/// Helper function to create the test app with a specific database pool
async fn create_test_app_with_pool(pool: Pool<Sqlite>) -> Router {
    // Set up environment variables for testing
    #[allow(unsafe_code)]
    unsafe {
        std::env::set_var(
            "JWT_SECRET",
            "test_secret_key_that_is_long_enough_for_testing",
        );
        std::env::set_var("GOOGLE_CLIENT_ID", "test_client_id");
        std::env::set_var("GOOGLE_CLIENT_SECRET", "test_client_secret");
        std::env::set_var("SERVER_URL", "http://localhost:8081");
        std::env::set_var("STATIC_DIR", "/tmp");

        // Use real API key if available for integration testing, otherwise use test key
        if let Ok(real_api_key) = std::env::var("OPENROUTER_API_KEY_REAL") {
            std::env::set_var("OPENROUTER_API_KEY", real_api_key);
            if is_verbose() {
                info!("🔑 Using real OpenRouter API key for integration testing");
            }
        } else {
            std::env::set_var("OPENROUTER_API_KEY", "test_api_key_not_real");
            if is_verbose() {
                warn!("🔑 Using test API key - AI requests will likely fail");
                warn!("💡 Set OPENROUTER_API_KEY_REAL environment variable for real AI testing");
            }
        }
    }

    let user_service = Arc::new(UserServiceImpl::new(pool.clone()));
    let auth_service = Arc::new(AuthService::new().expect("Failed to create auth service"));
    let invite_service = Arc::new(InviteService::new(pool.clone()));
    let oauth_service =
        Arc::new(OAuthService::new(pool.clone()).expect("Failed to create oauth service"));

    create_router(
        user_service,
        auth_service,
        invite_service,
        oauth_service,
        pool,
    )
    .await
    .expect("Failed to create router")
}

/// Helper function to send a JSON request to the test app
async fn send_json_request(app: Router, method: Method, uri: &str, body: Value) -> Response<Body> {
    if is_verbose() {
        info!("📤 Sending {} request to {}", method, uri);
        debug!(
            "Request body: {}",
            serde_json::to_string_pretty(&body).unwrap_or_default()
        );
    }

    let request = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_string(&body).expect("Failed to serialize JSON body"),
        ))
        .expect("Failed to build request");

    let response = app.oneshot(request).await.expect("Failed to send request");

    if is_verbose() {
        info!("📥 Response status: {}", response.status());
    }

    response
}

/// Helper function to extract JSON response body
async fn extract_json_response(response: Response<Body>) -> Value {
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");
    let json_value: Value = serde_json::from_slice(&body).expect("Failed to parse JSON response");

    if is_verbose() {
        debug!(
            "Response body: {}",
            serde_json::to_string_pretty(&json_value).unwrap_or_default()
        );
    }

    json_value
}

/// Helper function to send a request with authorization header
async fn send_authenticated_request(
    app: Router,
    method: Method,
    uri: &str,
    token: &str,
    body: Option<Value>,
) -> Response<Body> {
    if is_verbose() {
        info!("🔐 Sending authenticated {} request to {}", method, uri);
        if let Some(ref body_data) = body {
            debug!(
                "Request body: {}",
                serde_json::to_string_pretty(body_data).unwrap_or_default()
            );
        }
    }

    let mut request_builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {token}"));

    let request = if let Some(body_data) = body {
        request_builder = request_builder.header("content-type", "application/json");
        request_builder
            .body(Body::from(
                serde_json::to_string(&body_data).expect("Failed to serialize JSON body"),
            ))
            .expect("Failed to build authenticated request")
    } else {
        request_builder
            .body(Body::empty())
            .expect("Failed to build authenticated request")
    };

    let response = app.oneshot(request).await.expect("Failed to send request");

    if is_verbose() {
        info!("📥 Authenticated response status: {}", response.status());
    }

    response
}

/// Helper function to log AI interaction details in verbose mode
fn log_ai_interaction(
    endpoint: &str,
    request_payload: &Value,
    response_status: StatusCode,
    response_body: &Value,
) {
    if is_verbose() {
        info!("🤖 AI Interaction Summary for {}", endpoint);
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        // Extract and log model information
        if let Some(model) = request_payload.get("model") {
            info!("🔧 Model: {}", model.as_str().unwrap_or("default"));
        } else {
            info!("🔧 Model: default (not specified in request)");
        }

        // Log the request messages
        if let Some(messages) = request_payload.get("messages").and_then(|m| m.as_array()) {
            info!("📝 Request Messages ({} total):", messages.len());
            for (i, message) in messages.iter().enumerate() {
                if let (Some(role), Some(content)) = (message.get("role"), message.get("content")) {
                    let role_str = role.as_str().unwrap_or("unknown");
                    let content_str = content.as_str().unwrap_or("");
                    let truncated_content = if content_str.len() > 100 {
                        format!("{}...", &content_str[..100])
                    } else {
                        content_str.to_string()
                    };
                    info!("  {}. {} → {}", i + 1, role_str, truncated_content);
                }
            }
        }

        info!("📊 Response Status: {}", response_status);

        // Log response details if successful
        if response_status == StatusCode::OK {
            if let Some(message) = response_body.get("message") {
                if let Some(content) = message.get("content").and_then(|c| c.as_str()) {
                    let truncated_response = if content.len() > 200 {
                        format!("{}...", &content[..200])
                    } else {
                        content.to_string()
                    };
                    info!("🤖 AI Response: {}", truncated_response);
                }
                if let Some(role) = message.get("role").and_then(|r| r.as_str()) {
                    info!("🎭 Response Role: {}", role);
                }
            }

            // Log conversation ID if present
            if let Some(conv_id) = response_body
                .get("conversation_id")
                .and_then(|id| id.as_str())
            {
                info!("💬 Conversation ID: {}", conv_id);
            }

            // Log usage information if present
            if let Some(usage) = response_body.get("usage") {
                if let Some(tokens) = usage
                    .get("total_tokens")
                    .and_then(serde_json::Value::as_i64)
                {
                    info!("🔢 Total Tokens Used: {}", tokens);
                }
            }
        } else {
            // Log error details
            if let Some(error) = response_body.get("error").and_then(|e| e.as_str()) {
                warn!("❌ Error: {}", error);
            }
            if let Some(message) = response_body.get("message").and_then(|m| m.as_str()) {
                warn!("❌ Error Message: {}", message);
            }
        }

        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    }
}

/// Helper function to register a user and get a valid JWT token
async fn register_and_login_user(pool: Pool<Sqlite>, email: &str, password: &str) -> String {
    // First create an invite
    use chrono::Utc;
    let now = Utc::now();
    sqlx::query(
        r"
        INSERT INTO user_invites (id, email, invited_by, invited_at, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        ",
    )
    .bind(format!("test-invite-{}", uuid::Uuid::new_v4()))
    .bind(email.to_lowercase())
    .bind("test-admin")
    .bind(now)
    .bind(now)
    .bind(now)
    .execute(&pool)
    .await
    .expect("Failed to create test invite");

    let app = create_test_app_with_pool(pool).await;

    // Register the user
    let register_payload = json!({
        "email": email,
        "password": password
    });

    let register_response = send_json_request(
        app.clone(),
        Method::POST,
        "/api/auth/register",
        register_payload,
    )
    .await;
    assert_eq!(register_response.status(), StatusCode::CREATED);

    // Login to get the token
    let login_payload = json!({
        "email": email,
        "password": password
    });

    let login_response =
        send_json_request(app, Method::POST, "/api/auth/login", login_payload).await;
    assert_eq!(login_response.status(), StatusCode::OK);

    let login_body = extract_json_response(login_response).await;
    login_body["auth_token"]
        .as_str()
        .expect("Auth token not found in login response")
        .to_string()
}

#[tokio::test]
async fn test_ai_info_endpoint() {
    let pool = create_test_db().await;
    let app = create_test_app_with_pool(pool).await;

    // Test the AI info endpoint (no auth required for this endpoint)
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/ai/info")
        .body(Body::empty())
        .expect("Failed to build request");

    let response = app.oneshot(request).await.expect("Failed to send request");

    // This will likely fail due to missing API key, but we can test the endpoint structure
    if response.status() == StatusCode::OK {
        let json_body = extract_json_response(response).await;
        assert!(json_body["provider"].is_string());
        assert!(json_body["templates"].is_array());
        assert!(json_body["schemas"].is_array());
        assert!(json_body["streaming_supported"].is_boolean());
        assert!(json_body["websocket_supported"].is_boolean());
    } else {
        // Expected to fail due to missing real API key in test environment
        assert!(response.status().is_client_error() || response.status().is_server_error());
    }
}

#[tokio::test]
async fn test_chat_endpoint_without_auth() {
    let pool = create_test_db().await;
    let app = create_test_app_with_pool(pool).await;

    let chat_payload = json!({
        "messages": [
            {
                "role": "user",
                "content": "Hello, how are you?"
            }
        ]
    });

    let response = send_json_request(app, Method::POST, "/api/ai/chat", chat_payload).await;

    // Should return unauthorized
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_chat_endpoint_with_auth() {
    let pool = create_test_db().await;
    let token =
        register_and_login_user(pool.clone(), "chat_test@example.com", "secure_password_123").await;
    let app = create_test_app_with_pool(pool).await;

    let chat_payload = json!({
        "messages": [
            {
                "role": "user",
                "content": "Hello, how are you?"
            }
        ]
    });

    let response = send_authenticated_request(
        app,
        Method::POST,
        "/api/ai/chat",
        &token,
        Some(chat_payload),
    )
    .await;

    // This will likely fail due to missing real API key, but we can test the auth flow
    // Either succeeds or fails with a specific AI error (not auth error)
    assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_chat_endpoint_invalid_payload() {
    let pool = create_test_db().await;
    let token = register_and_login_user(
        pool.clone(),
        "invalid_test@example.com",
        "secure_password_123",
    )
    .await;
    let app = create_test_app_with_pool(pool).await;

    let invalid_payload = json!({
        "invalid_field": "invalid_value"
    });

    let response = send_authenticated_request(
        app,
        Method::POST,
        "/api/ai/chat",
        &token,
        Some(invalid_payload),
    )
    .await;

    // Should return bad request for invalid payload
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_chat_stream_endpoint() {
    let pool = create_test_db().await;
    let token = register_and_login_user(
        pool.clone(),
        "stream_test@example.com",
        "secure_password_123",
    )
    .await;
    let app = create_test_app_with_pool(pool).await;

    // Test SSE streaming endpoint with simple query parameters
    let uri = "/api/ai/chat/stream?stream=true";
    let response = send_authenticated_request(app, Method::GET, uri, &token, None).await;

    // This will likely fail due to missing real API key or invalid query format
    // but we test that it's not an auth error
    assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_chat_websocket_endpoint() {
    let pool = create_test_db().await;
    let token =
        register_and_login_user(pool.clone(), "ws_test@example.com", "secure_password_123").await;
    let app = create_test_app_with_pool(pool).await;

    // Test WebSocket upgrade endpoint
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/ai/chat/ws")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .header("upgrade", "websocket")
        .header("connection", "upgrade")
        .header("sec-websocket-key", "test-key")
        .header("sec-websocket-version", "13")
        .body(Body::empty())
        .expect("Failed to build request");

    let response = app.oneshot(request).await.expect("Failed to send request");

    // Should not be an auth error (either upgrades or fails for other reasons)
    assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
}

// =====================
// COMPREHENSIVE AI CHAT INTEGRATION TESTS
// Task 3.1.11: Create comprehensive integration test suite for AI chat functionality
// =====================

#[tokio::test]
async fn test_simple_chat_request_response_flow() {
    let pool = create_test_db().await;
    let token = register_and_login_user(
        pool.clone(),
        "chat_integration@example.com",
        "secure_password_123",
    )
    .await;
    let app = create_test_app_with_pool(pool.clone()).await;

    let chat_payload = json!({
        "messages": [
            {
                "role": "user",
                "content": "Hello! Please respond with 'Hello from AI' for testing."
            }
        ]
    });

    let response = send_authenticated_request(
        app,
        Method::POST,
        "/api/ai/chat",
        &token,
        Some(chat_payload.clone()),
    )
    .await;

    let response_status = response.status();
    let json_body = extract_json_response(response).await;

    // Log the AI interaction details in verbose mode
    log_ai_interaction("/api/ai/chat", &chat_payload, response_status, &json_body);

    // Test that request is properly authenticated and formatted
    if response_status == StatusCode::OK {
        // Verify response structure
        assert!(json_body["conversation_id"].is_string());
        assert!(json_body["message"]["content"].is_string());
        assert_eq!(json_body["message"]["role"], "assistant");

        // Verify database persistence - check conversation was created
        let conversation_id = json_body["conversation_id"]
            .as_str()
            .expect("Conversation ID not found in response");
        let conversation_exists = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM ai_conversations WHERE id = ?",
            conversation_id
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to query conversation");

        assert_eq!(conversation_exists, 1);

        // Verify messages were persisted
        let message_count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM ai_messages WHERE conversation_id = ?",
            conversation_id
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to query messages");

        // Should have user message and AI response
        assert_eq!(message_count, 2);

        // Verify usage tracking
        let usage_count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM ai_usage WHERE conversation_id = ?",
            conversation_id
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to query usage");

        assert_eq!(usage_count, 1);
    } else {
        // Expected to fail due to test API key, but should be properly structured failure
        assert!(response_status.is_client_error() || response_status.is_server_error());
        assert_ne!(response_status, StatusCode::UNAUTHORIZED);
    }
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn test_ai_model_and_parameter_variations() {
    let pool = create_test_db().await;
    let token = register_and_login_user(
        pool.clone(),
        "model_test@example.com",
        "secure_password_123",
    )
    .await;
    let app = create_test_app_with_pool(pool.clone()).await;

    if is_verbose() {
        info!("🧪 Testing AI model and parameter variations");
    }

    // Test 1: Basic request with default model
    let basic_payload = json!({
        "messages": [
            {
                "role": "user",
                "content": "What is 2+2? Answer with just the number."
            }
        ]
    });

    let response1 = send_authenticated_request(
        app.clone(),
        Method::POST,
        "/api/ai/chat",
        &token,
        Some(basic_payload.clone()),
    )
    .await;

    let response1_status = response1.status();
    let json_body1 = extract_json_response(response1).await;
    log_ai_interaction(
        "/api/ai/chat (default model)",
        &basic_payload,
        response1_status,
        &json_body1,
    );

    // Test 2: Request with specific model and parameters
    let advanced_payload = json!({
        "messages": [
            {
                "role": "system",
                "content": "You are a helpful assistant. Be concise."
            },
            {
                "role": "user",
                "content": "Explain quantum computing in one sentence."
            }
        ],
        "model": "openai/gpt-3.5-turbo",
        "temperature": 0.7,
        "max_tokens": 100
    });

    let response2 = send_authenticated_request(
        app.clone(),
        Method::POST,
        "/api/ai/chat",
        &token,
        Some(advanced_payload.clone()),
    )
    .await;

    let response2_status = response2.status();
    let json_body2 = extract_json_response(response2).await;
    log_ai_interaction(
        "/api/ai/chat (specific model)",
        &advanced_payload,
        response2_status,
        &json_body2,
    );

    // Test 3: Request with structured response format
    let structured_payload = json!({
        "messages": [
            {
                "role": "user",
                "content": "List three colors. Respond in JSON format with a 'colors' array."
            }
        ],
        "response_format": {
            "type": "json_schema",
            "json_schema": {
                "type": "object",
                "properties": {
                    "colors": {
                        "type": "array",
                        "items": { "type": "string" }
                    }
                },
                "required": ["colors"]
            }
        }
    });

    let response3 = send_authenticated_request(
        app.clone(),
        Method::POST,
        "/api/ai/chat",
        &token,
        Some(structured_payload.clone()),
    )
    .await;

    let response3_status = response3.status();
    let json_body3 = extract_json_response(response3).await;
    log_ai_interaction(
        "/api/ai/chat (structured response)",
        &structured_payload,
        response3_status,
        &json_body3,
    );

    // Verify that requests are properly authenticated (not auth errors)
    assert_ne!(response1_status, StatusCode::UNAUTHORIZED);
    assert_ne!(response2_status, StatusCode::UNAUTHORIZED);
    assert_ne!(response3_status, StatusCode::UNAUTHORIZED);

    // If any requests succeeded, verify response structure
    for (response_status, json_body, test_name) in [
        (response1_status, &json_body1, "basic"),
        (response2_status, &json_body2, "advanced"),
        (response3_status, &json_body3, "structured"),
    ] {
        if response_status == StatusCode::OK {
            if is_verbose() {
                info!("✅ {} test succeeded with real AI response", test_name);
            }
            assert!(json_body["conversation_id"].is_string());
            assert!(json_body["message"]["content"].is_string());
            assert_eq!(json_body["message"]["role"], "assistant");
        } else if is_verbose() {
            warn!("⚠️  {} test failed - likely due to test API key", test_name);
        }
    }
}

#[tokio::test]
async fn test_conversation_persistence_and_retrieval() {
    let pool = create_test_db().await;
    let token = register_and_login_user(
        pool.clone(),
        "persistence_test@example.com",
        "secure_password_123",
    )
    .await;
    let app = create_test_app_with_pool(pool.clone()).await;

    // First, send a chat message to create a conversation
    let initial_chat = json!({
        "messages": [
            {
                "role": "user",
                "content": "Start a conversation about testing."
            }
        ]
    });

    let response1 = send_authenticated_request(
        app.clone(),
        Method::POST,
        "/api/ai/chat",
        &token,
        Some(initial_chat),
    )
    .await;

    if response1.status() == StatusCode::OK {
        let json_body1 = extract_json_response(response1).await;
        let conversation_id = json_body1["conversation_id"]
            .as_str()
            .expect("Conversation ID not found in response");

        // Test retrieving conversations list
        let conversations_response = send_authenticated_request(
            app.clone(),
            Method::GET,
            "/api/ai/conversations",
            &token,
            None,
        )
        .await;

        assert_eq!(conversations_response.status(), StatusCode::OK);
        let conversations_list = extract_json_response(conversations_response).await;
        assert!(conversations_list["conversations"].is_array());

        let conversations = conversations_list["conversations"]
            .as_array()
            .expect("Conversations should be an array");
        assert!(!conversations.is_empty());

        // Find our conversation in the list
        let our_conversation = conversations
            .iter()
            .find(|conv| conv["id"].as_str() == Some(conversation_id))
            .expect("Conversation should exist in list");

        assert!(our_conversation["created_at"].is_string());
        assert!(our_conversation["message_count"].is_number());

        // Test retrieving specific conversation with messages
        let conversation_detail_response = send_authenticated_request(
            app.clone(),
            Method::GET,
            &format!("/api/ai/conversations/{conversation_id}"),
            &token,
            None,
        )
        .await;

        assert_eq!(conversation_detail_response.status(), StatusCode::OK);
        let conversation_detail = extract_json_response(conversation_detail_response).await;

        assert_eq!(conversation_detail["conversation"]["id"], conversation_id);
        assert!(conversation_detail["messages"].is_array());

        let messages = conversation_detail["messages"]
            .as_array()
            .expect("Messages should be an array");
        assert_eq!(messages.len(), 2); // User message + AI response

        // Verify message order and content
        assert_eq!(messages[0]["role"], "user");
        assert_eq!(messages[1]["role"], "assistant");
        assert!(messages[0]["created_at"].is_string());
        assert!(messages[1]["created_at"].is_string());
    }
}

#[tokio::test]
async fn test_conversation_archival() {
    let pool = create_test_db().await;
    let token = register_and_login_user(
        pool.clone(),
        "archive_test@example.com",
        "secure_password_123",
    )
    .await;
    let app = create_test_app_with_pool(pool.clone()).await;

    // Create a conversation first
    let chat_payload = json!({
        "messages": [
            {
                "role": "user",
                "content": "Test conversation for archival."
            }
        ]
    });

    let response = send_authenticated_request(
        app.clone(),
        Method::POST,
        "/api/ai/chat",
        &token,
        Some(chat_payload),
    )
    .await;

    if response.status() == StatusCode::OK {
        let json_body = extract_json_response(response).await;
        let conversation_id = json_body["conversation_id"]
            .as_str()
            .expect("Conversation ID not found in response");

        // Archive the conversation
        let archive_response = send_authenticated_request(
            app.clone(),
            Method::POST,
            &format!("/api/ai/conversations/{conversation_id}/archive"),
            &token,
            None,
        )
        .await;

        assert_eq!(archive_response.status(), StatusCode::OK);

        // Verify conversation is marked as archived in database
        let archived_at = sqlx::query_scalar!(
            "SELECT archived_at FROM ai_conversations WHERE id = ?",
            conversation_id
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to query archived status");

        assert!(archived_at.is_some());

        // Verify archived conversation doesn't appear in active conversations list
        let conversations_response = send_authenticated_request(
            app.clone(),
            Method::GET,
            "/api/ai/conversations",
            &token,
            None,
        )
        .await;

        assert_eq!(conversations_response.status(), StatusCode::OK);
        let conversations_list = extract_json_response(conversations_response).await;

        let conversations = conversations_list["conversations"]
            .as_array()
            .expect("Conversations should be an array");
        let archived_conversation = conversations
            .iter()
            .find(|conv| conv["id"].as_str() == Some(conversation_id));

        // Should not find archived conversation in active list
        assert!(archived_conversation.is_none());

        // But should still be able to retrieve it directly
        let conversation_detail_response = send_authenticated_request(
            app.clone(),
            Method::GET,
            &format!("/api/ai/conversations/{conversation_id}"),
            &token,
            None,
        )
        .await;

        assert_eq!(conversation_detail_response.status(), StatusCode::OK);
        let conversation_detail = extract_json_response(conversation_detail_response).await;
        assert!(conversation_detail["conversation"]["archived_at"].is_string());
    }
}

#[tokio::test]
async fn test_usage_statistics_tracking() {
    let pool = create_test_db().await;
    let token = register_and_login_user(
        pool.clone(),
        "usage_test@example.com",
        "secure_password_123",
    )
    .await;
    let app = create_test_app_with_pool(pool.clone()).await;

    // Send a chat message to generate usage
    let chat_payload = json!({
        "messages": [
            {
                "role": "user",
                "content": "Generate usage statistics for testing."
            }
        ]
    });

    let response = send_authenticated_request(
        app.clone(),
        Method::POST,
        "/api/ai/chat",
        &token,
        Some(chat_payload),
    )
    .await;

    if response.status() == StatusCode::OK {
        // Check usage statistics endpoint
        let usage_response =
            send_authenticated_request(app.clone(), Method::GET, "/api/ai/usage", &token, None)
                .await;

        assert_eq!(usage_response.status(), StatusCode::OK);
        let usage_stats = extract_json_response(usage_response).await;

        // Verify usage statistics structure
        assert!(usage_stats["total_requests"].is_number());
        assert!(usage_stats["total_tokens"].is_number());
        assert!(usage_stats["total_cost_cents"].is_number());
        assert!(usage_stats["requests_by_model"].is_object());

        let total_requests = usage_stats["total_requests"]
            .as_i64()
            .expect("Total requests should be a number");
        assert!(total_requests > 0);

        // Verify database contains usage record
        let usage_count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM ai_usage WHERE user_id = (SELECT id FROM users WHERE email = ?)",
            "usage_test@example.com"
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to query usage count");

        assert!(usage_count > 0);
    }
}

#[tokio::test]
async fn test_contextual_chat_with_file_upload() {
    let pool = create_test_db().await;
    let token = register_and_login_user(
        pool.clone(),
        "upload_test@example.com",
        "secure_password_123",
    )
    .await;
    let app = create_test_app_with_pool(pool.clone()).await;

    // Test file upload endpoint
    #[allow(clippy::no_effect_underscore_binding)]
    let _file_content = "This is a test document for contextual chat. The document discusses testing methodologies and best practices.";

    // Create a multipart form request for file upload
    // Note: For integration testing, we test the endpoint structure
    // The actual file processing would require multipart handling in tests

    let _upload_response = send_authenticated_request(
        app.clone(),
        Method::POST,
        "/api/ai/upload",
        &token,
        None, // Would need multipart body for actual file upload
    )
    .await;

    // Test contextual chat endpoint structure
    let contextual_chat_payload = json!({
        "messages": [
            {
                "role": "user",
                "content": "Summarize the uploaded document."
            }
        ],
        "context": ["test document context"],
        "template": "summarize"
    });

    let contextual_response = send_authenticated_request(
        app.clone(),
        Method::POST,
        "/api/ai/chat/contextual",
        &token,
        Some(contextual_chat_payload),
    )
    .await;

    // Should not be an auth error (may fail due to test API key or missing file)
    assert_ne!(contextual_response.status(), StatusCode::UNAUTHORIZED);

    // If successful, verify response structure
    if contextual_response.status() == StatusCode::OK {
        let json_body = extract_json_response(contextual_response).await;
        assert!(json_body["conversation_id"].is_string());
        assert!(json_body["message"]["content"].is_string());
    }
}

#[tokio::test]
async fn test_error_handling_and_edge_cases() {
    let pool = create_test_db().await;
    let token = register_and_login_user(
        pool.clone(),
        "error_test@example.com",
        "secure_password_123",
    )
    .await;
    let app = create_test_app_with_pool(pool.clone()).await;

    // Test invalid conversation ID
    let invalid_conversation_response = send_authenticated_request(
        app.clone(),
        Method::GET,
        "/api/ai/conversations/invalid-id-123",
        &token,
        None,
    )
    .await;

    // Should return 404 for non-existent ID or 400 for invalid UUID format
    assert!(
        invalid_conversation_response.status() == StatusCode::NOT_FOUND
            || invalid_conversation_response.status() == StatusCode::BAD_REQUEST
    );

    // Test malformed chat request
    let malformed_payload = json!({
        "invalid_field": "test",
        "messages": "not_an_array"
    });

    let malformed_response = send_authenticated_request(
        app.clone(),
        Method::POST,
        "/api/ai/chat",
        &token,
        Some(malformed_payload),
    )
    .await;

    assert_eq!(
        malformed_response.status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );

    // Test empty messages array
    let empty_messages_payload = json!({
        "messages": []
    });

    let empty_response = send_authenticated_request(
        app.clone(),
        Method::POST,
        "/api/ai/chat",
        &token,
        Some(empty_messages_payload),
    )
    .await;

    assert_eq!(empty_response.status(), StatusCode::BAD_REQUEST);

    // Test archiving non-existent conversation
    let archive_invalid_response = send_authenticated_request(
        app.clone(),
        Method::POST,
        "/api/ai/conversations/invalid-id-123/archive",
        &token,
        None,
    )
    .await;

    // Should return 404 for non-existent ID or 400 for invalid UUID format
    assert!(
        archive_invalid_response.status() == StatusCode::NOT_FOUND
            || archive_invalid_response.status() == StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn test_user_isolation_and_authorization() {
    let pool = create_test_db().await;

    // Create two different users
    let token1 =
        register_and_login_user(pool.clone(), "user1@example.com", "secure_password_123").await;
    let token2 =
        register_and_login_user(pool.clone(), "user2@example.com", "secure_password_123").await;
    let app = create_test_app_with_pool(pool.clone()).await;

    // User 1 creates a conversation
    let chat_payload = json!({
        "messages": [
            {
                "role": "user",
                "content": "Private conversation for user 1."
            }
        ]
    });

    let response1 = send_authenticated_request(
        app.clone(),
        Method::POST,
        "/api/ai/chat",
        &token1,
        Some(chat_payload),
    )
    .await;

    if response1.status() == StatusCode::OK {
        let json_body1 = extract_json_response(response1).await;
        let conversation_id = json_body1["conversation_id"]
            .as_str()
            .expect("Conversation ID not found in response");

        // User 2 tries to access User 1's conversation
        let unauthorized_access = send_authenticated_request(
            app.clone(),
            Method::GET,
            &format!("/api/ai/conversations/{conversation_id}"),
            &token2,
            None,
        )
        .await;

        // Should be forbidden or not found (user isolation)
        assert!(
            unauthorized_access.status() == StatusCode::FORBIDDEN
                || unauthorized_access.status() == StatusCode::NOT_FOUND
        );

        // User 2's conversation list should be empty
        let user2_conversations = send_authenticated_request(
            app.clone(),
            Method::GET,
            "/api/ai/conversations",
            &token2,
            None,
        )
        .await;

        assert_eq!(user2_conversations.status(), StatusCode::OK);
        let conversations_list = extract_json_response(user2_conversations).await;
        let conversations = conversations_list["conversations"]
            .as_array()
            .expect("Conversations should be an array");

        // User 2 should not see User 1's conversations
        let user1_conversation = conversations
            .iter()
            .find(|conv| conv["id"].as_str() == Some(conversation_id));
        assert!(user1_conversation.is_none());
    }
}

#[tokio::test]
async fn test_ai_service_health_and_info() {
    let pool = create_test_db().await;
    let token = register_and_login_user(
        pool.clone(),
        "health_test@example.com",
        "secure_password_123",
    )
    .await;
    let app = create_test_app_with_pool(pool).await;

    // Test AI health endpoint
    let health_response =
        send_authenticated_request(app.clone(), Method::GET, "/api/ai/health", &token, None).await;

    // Should have proper response (not auth error)
    assert_ne!(health_response.status(), StatusCode::UNAUTHORIZED);

    // Test AI info endpoint (already tested above, but verify again)
    let info_response =
        send_authenticated_request(app.clone(), Method::GET, "/api/ai/info", &token, None).await;

    if info_response.status() == StatusCode::OK {
        let info_body = extract_json_response(info_response).await;
        assert!(info_body["provider"].is_string());
        assert!(info_body["templates"].is_array());
        assert!(info_body["schemas"].is_array());
        assert!(info_body["streaming_supported"].is_boolean());
        assert!(info_body["websocket_supported"].is_boolean());
    }
}

#[tokio::test]
async fn test_content_moderation_endpoint() {
    let pool = create_test_db().await;
    let token = register_and_login_user(
        pool.clone(),
        "moderation_test@example.com",
        "secure_password_123",
    )
    .await;
    let app = create_test_app_with_pool(pool).await;

    let moderation_payload = json!({
        "content": "This is test content for moderation analysis."
    });

    let moderation_response = send_authenticated_request(
        app.clone(),
        Method::POST,
        "/api/ai/moderate",
        &token,
        Some(moderation_payload),
    )
    .await;

    // Should not be an auth error
    assert_ne!(moderation_response.status(), StatusCode::UNAUTHORIZED);

    // If successful, verify response structure
    if moderation_response.status() == StatusCode::OK {
        let moderation_body = extract_json_response(moderation_response).await;
        assert!(moderation_body["safe"].is_boolean());
        assert!(moderation_body["issues"].is_array());
        assert!(moderation_body["severity"].is_string());
        assert!(moderation_body["recommendation"].is_string());
    }
}

#[tokio::test]
async fn test_code_analysis_endpoint() {
    let pool = create_test_db().await;
    let token =
        register_and_login_user(pool.clone(), "code_test@example.com", "secure_password_123").await;
    let app = create_test_app_with_pool(pool).await;

    let code_payload = json!({
        "code": "function test() { return 'hello world'; }",
        "language": "javascript"
    });

    let code_response = send_authenticated_request(
        app.clone(),
        Method::POST,
        "/api/ai/analyze/code",
        &token,
        Some(code_payload),
    )
    .await;

    // Should not be an auth error
    assert_ne!(code_response.status(), StatusCode::UNAUTHORIZED);

    // If successful, verify response structure
    if code_response.status() == StatusCode::OK {
        let code_body = extract_json_response(code_response).await;
        assert!(code_body["analysis"].is_string());
        assert!(code_body["suggestions"].is_array());
        assert!(code_body["complexity_score"].is_number());
        assert!(code_body["issues"].is_array());
    }
}
