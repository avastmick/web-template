# Server Test Structure

This directory contains all tests for the server, organized into distinct modules based on testing approach and purpose. The testing strategy follows a three-tier approach to ensure comprehensive coverage while maintaining CI compatibility.

## Three-Tier Testing Strategy

### 1. `integration/` - Service Integration Tests
**Direct component testing without HTTP layer:**
- Tests that use server modules, services, and functions programmatically
- NEVER mock or simulate connections or usage
- Direct access to business logic without going through HTTP
- Uses in-memory SQLite database for testing
- Tests the integration between different server components
- Verifies that services work correctly when combined
- Example: Testing that `BoardService` and `IssueService` work together correctly

**Key characteristics:**
- NO HTTP requests or REST API calls
- Direct function/method calls to server code
- Tests business logic and service integration
- Uses `TestContext` to access services directly
- **CI-friendly**: Self-contained, no external dependencies

### 2. `endpoint/` - API Contract Tests (Current)
**Router-level testing with simulated HTTP requests:**
- Tests that use `app.oneshot()` to invoke the router directly
- NEVER mock or simulate connections or usage
- Verifies complete request/response handling within the application
- Tests API contract, status codes, and response formats
- **MUST verify both API response AND database state**
- Uses the actual Axum router with all middleware configured
- Example: POST to `/api/issues` and verify both the response AND that the issue exists in the database

**Key characteristics:**
- Uses simulated HTTP requests through `app.oneshot()`
- Tests the router, handlers, and middleware
- Verifies authentication, authorization, and validation
- Checks database state matches API responses
- **CI-friendly**: Self-contained, no external server needed
- **Limitation**: Doesn't test actual HTTP serialization or network behavior

### 3. `e2e/` - End-to-End Server Tests (To Be Implemented)
**Full HTTP stack testing with running server:**
- Tests that use `reqwest` or similar to make real HTTP requests
- Verifies complete server behavior as it runs in production
- Tests actual HTTP serialization, headers, compression, etc.
- **REQUIRES a running server** (`just dev` or `just server-dev-server`)
- Example: Make actual HTTP requests to `http://localhost:8081/api/issues`

**Key characteristics:**
- Real HTTP requests over the network
- Tests the complete server stack as deployed
- Verifies CORS, compression, rate limiting, WebSockets
- Catches issues that only appear with real HTTP
- **NOT CI-friendly**: Requires external server process
- **Run locally before commits**: Ensures production readiness

## Common Test Utilities

### `common/`
Shared test utilities and helpers used across all test types:
- Test database setup and teardown
- Test data factories
- Shared test context and fixtures
- Helper functions for creating test users, boards, issues, etc.

## Test Organization Rules

1. **Unit tests** belong in the source files (`#[cfg(test)]` modules)
2. **Integration tests** (direct code access) go in `integration/`
3. **API endpoint tests** (HTTP requests) go in `endpoint/`
4. **Shared test utilities** go in `common/`

## Running Tests

```bash
# Run all server tests
just test-server

# Run specific test module
cargo test --test integration_tests
cargo test --test endpoint_tests

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_issue_lifecycle
```

## Test Coverage

All tests contribute to overall code coverage goals:
- Target: >80% overall coverage
- 100% of API endpoints must have endpoint tests
- Critical business logic must have integration tests
- All edge cases should be covered

## Important Testing Standards

### Endpoint Tests MUST Use Standard Helpers

**All endpoint tests MUST use the provided helper functions** (`send_json_request`, `send_authenticated_request`, etc.) for making HTTP requests. DO NOT use `Request::builder()` directly as it:
- Breaks automated route coverage detection
- Creates inconsistent test patterns
- Makes tests harder to maintain

See `tests/endpoint/README.md` for detailed examples of correct vs incorrect patterns.
