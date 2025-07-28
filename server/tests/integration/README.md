# Integration Tests

This module contains integration tests that access server components directly through code.

## Purpose

Integration tests verify that different server components work correctly together:
- Service-to-service interactions
- Business logic flows across multiple services
- Database operations through services
- Complex workflows that span multiple components

## Characteristics

- **NO HTTP requests** - Direct function/method calls only
- **NO REST API testing** - That belongs in `endpoint/` tests
- Uses `TestContext` to access services directly
- Tests business logic integration
- Uses in-memory SQLite database

## Example

```rust
use crate::common::TestContext;

#[tokio::test]
async fn test_board_service_creates_default_board() {
    let ctx = TestContext::new().await;
    let user = ctx.create_test_user("test@example.com").await;

    // Direct service call - NOT an HTTP request
    let board = ctx.board_service
        .get_user_board(&user.id.to_string())
        .await
        .expect("Failed to get user board");

    assert_eq!(board.name, "My Board");
    assert_eq!(board.user_id, user.id);
}
```

## What Belongs Here

✅ Service integration tests
✅ Multi-service workflows
✅ Business logic that spans components
✅ Database transaction tests
✅ Complex data flow tests

## What Does NOT Belong Here

❌ HTTP/REST API tests (use `endpoint/`)
❌ Unit tests (use `#[cfg(test)]` in source files)
❌ Tests that make network requests
❌ Tests that start a server
❌ Tests using `Router` or HTTP methods
