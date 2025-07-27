# Endpoint Tests

This module contains tests that verify the REST API endpoints via HTTP requests.

## Purpose

Endpoint tests ensure the complete API stack works correctly:
- HTTP request/response handling
- Middleware execution (auth, CORS, etc.)
- Request validation and error responses
- API contract compliance
- **Database state verification**

## Characteristics

- **Uses HTTP requests** (GET, POST, PUT, DELETE)
- Tests against running server instance
- Verifies both API responses AND database state
- Tests authentication and authorization
- Validates error handling and status codes

## Important: Database Verification

**Every endpoint test MUST verify both:**
1. The API response (status code, body, headers)
2. The database state matches the API response

## Example

```rust
#[tokio::test]
async fn test_create_issue_endpoint() {
    let (app, ctx) = create_test_app().await;
    let user = ctx.create_test_user("test@example.com").await;
    let token = generate_test_token(&user);
    let board = ctx.create_test_board(&user, None).await;

    // Make HTTP request
    let response = send_json_request(
        app,
        Method::POST,
        "/api/issues",
        json!({
            "board_id": board.id,
            "title": "New Issue",
            "type": "feature",
            "priority": "high"
        }),
        Some(&token)
    ).await;

    // Verify API response
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = extract_json_response(response).await;
    assert_eq!(body["title"], "New Issue");

    // CRITICAL: Verify database state
    let issue_id = body["id"].as_str().unwrap();
    let db_issue = ctx.issue_service
        .get_issue(&user.id.to_string(), issue_id)
        .await
        .expect("Issue should exist in database");

    assert_eq!(db_issue.title, "New Issue");
    assert_eq!(db_issue.priority, IssuePriority::High);
}
```

## What Belongs Here

✅ REST API endpoint tests
✅ HTTP request/response verification
✅ Authentication/authorization tests
✅ API error handling tests
✅ Database state verification
✅ Response time measurements

## What Does NOT Belong Here

❌ Direct service calls (use `integration/`)
❌ Unit tests (use source file tests)
❌ Tests without HTTP requests
❌ Tests that don't verify database state

## Test Helpers

Common endpoint test helpers should include:
- `create_test_app()` - Sets up router with test context
- `send_json_request()` - Sends JSON HTTP requests
- `send_authenticated_request()` - Sends requests with auth
- `extract_json_response()` - Extracts JSON from responses
- Database verification helpers

## CRITICAL: Test Implementation Standards

**ALWAYS use the provided helper functions for making HTTP requests:**

✅ **CORRECT - Use helper functions:**
```rust
// For unauthenticated requests
let response = send_json_request(
    app,
    Method::POST,
    "/api/endpoint",
    json!({"key": "value"}),
).await;

// For authenticated requests
let response = send_authenticated_request(
    app,
    Method::GET,
    "/api/protected",
    &token,
).await;
```

❌ **INCORRECT - Do NOT use Request::builder() directly:**
```rust
// DO NOT DO THIS - it breaks route coverage detection
let request = Request::builder()
    .method(Method::GET)
    .uri("/api/endpoint")
    .body(Body::empty())
    .unwrap();
let response = app.oneshot(request).await.unwrap();
```

Using the standard helper functions ensures:
1. Consistent test implementation across all endpoints
2. Proper route coverage detection by test analysis tools
3. Easier maintenance and refactoring
4. Clear, readable test code
