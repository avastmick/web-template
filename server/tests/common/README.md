# Common Test Utilities

This module provides shared utilities and helpers for all server tests.

## Contents

- `mod.rs` - Module declarations and common imports
- `test_context.rs` - Test context with initialized services for integration testing

## TestContext

The `TestContext` struct provides:
- Pre-initialized services (UserService, BoardService, IssueService, etc.)
- Test database setup with migrations
- Helper methods for creating test data
- Consistent test environment setup

## Usage

```rust
use crate::common::TestContext;

#[tokio::test]
async fn test_example() {
    let ctx = TestContext::new().await;

    // Create test data
    let user = ctx.create_test_user("test@example.com").await;
    let board = ctx.create_test_board(&user, Some("My Board")).await;
    let issue = ctx.create_test_issue(&user, &board, "Test Issue").await;

    // Use services directly for integration tests
    let result = ctx.issue_service.update_issue(...).await;
}
```

## Guidelines

1. Keep utilities generic and reusable
2. Avoid test-specific logic in common utilities
3. Ensure proper cleanup (database is in-memory, auto-cleaned)
4. Use descriptive names for helper methods
5. Document any non-obvious behavior
