# Current Tasks

This document outlines the tasks to be completed based on `INSTRUCTIONS.md` and `PRD.md`.

## Status Summary

### Task 1.1: Achieve 80% Server Code Coverage
*   **Status:** **[-] TODO**
*   **Action:** Implement comprehensive test coverage for all server code
*   **Details:**
    *   Install and configure cargo-tarpaulin for coverage reporting
    *   Write unit tests to achieve >80% coverage for all services
    *   Write unit tests for all models and utilities
    *   Create test helpers and fixtures for common scenarios
    *   Add property-based tests for complex business logic
    *   Configure CI to enforce 90% minimum coverage
    *   Add new just commands for testing:
        - just coverage: Run tests with coverage reporting
        - just coverage-html: Generate HTML coverage report
*   **Files to Create/Modify:**
    *   `server/src/**/*.rs` (add #[cfg(test)] modules)
    *   `server/.tarpaulin.toml` (coverage configuration)
    *   `justfile` (add coverage commands)
*   **Implementation Notes:**
    *   Use cargo-tarpaulin for coverage measurement
    *   Focus on business logic and edge cases
    *   Test error paths thoroughly
    *   Use property testing for issue sizing logic
    *   Mock only external services (AI providers)
*   **Quality Checks:**
    *   Run `cargo tarpaulin --out Html` for coverage report
    *   Verify 90%+ coverage achieved
    *   All tests pass reliably (no flaky tests)
    *   Tests complete in under 30 seconds

### Task 1.2: Create Integration Tests for ALL Endpoints
*   **Status:** **[ ] TODO**
*   **Action:** Write comprehensive integration tests for every API endpoint
*   **Details:**
    *   Create test infrastructure for spawning real server instances
    *   Implement database setup/teardown for each test
    *   Write tests for authentication endpoints
    *   Write tests for AI endpoints
    *   Test error scenarios (401, 403, 404, 422, 500)
    *   Test pagination, filtering, and sorting
    *   Verify response times <100ms
*   **Files to Create/Modify:**
    *   `server/tests/common/mod.rs` (test helpers)
    *   `server/tests/integration/auth_endpoints.rs`
    *   `server/tests/integration/ai_endpoints.rs`
    *   `server/tests/integration/performance.rs`
*   **Implementation Notes:**
    *   NO MOCKS - tests run against real server
    *   Use reqwest for HTTP client
    *   Create test data factories
    *   Each test gets fresh database
    *   Test authorization boundaries
    *   Include timing assertions
*   **Test Structure Example:**
    ```rust
    #[tokio::test]
    async fn test_list_issues_with_filters() {
        // Setup
        let app = TestApp::spawn().await;
        let user = app.create_test_user().await;
        let board = app.create_board(&user).await;
        let issues = app.create_issues(&board, 20).await;

        // Execute
        let response = app.client
            .get(&format!("{}/api/issues?status=doing&page=2", app.address))
            .bearer_auth(&user.token)
            .send()
            .await
            .expect("Failed to execute request");

        // Assert
        assert_eq!(response.status(), 200);
        let body: ListIssuesResponse = response.json().await.unwrap();
        assert!(body.issues.len() <= 10); // page size
        assert!(response.elapsed() < Duration::from_millis(100));
    }
    ```
*   **Quality Checks:**
    *   Every endpoint has success case tests
    *   Every endpoint has auth failure tests
    *   Every endpoint has validation tests
    *   Performance assertions pass
    *   No test pollution between runs


### Task 1.3: Enable workspace 'features'
*   **Status:** **[ ] TODO**
*   **Action:** Create a means of enabling/disabling workspace features, such as local auth, Google auth, PostgreSQL, etc.
*   **Details:**
    *   Create a set of features in the `server` using the conventional `Cargo.toml` feature flags for:
        - local auth as `local_auth`, where email/password is offered as an option to register/login
        - Google oauth as `google_auth`, where Google oauth is offered as an option to register/login
        - GitHub oauth as `github_auth`, where GitHub oauth is offered as an option to register/login
        - PostgreSQL as a `db_pg`, where PostgreSQL is offered as an overriding option for the database backend instead of `SQLite`
        - Stripe as `stripe_payment`, where Stripe is conditionally offered as a payment option provider. Note if this option is not set then non-invited users registering will get an error 'This service is invite only'; if it is set then the non-invited users will be presented the payment page.
        - Chat as `chat`, where chat is offered as the main application option on successful registration and login - the main page. If disabled, the main page is just a holding page.
    *   Create a means of switching off components in the client, if the features are not set - i.e., they are not compiled into JS.
    *   The default is all current features: `local_auth`, `google_auth`, `github_auth`, `db_sqlite`, `stripe_payment`, and `chat`. Only `db_pg` is default disabled.

*   **Implementation Plan (Detailed Steps):**

    **Phase 1: Server-Side Feature Flags (Cargo.toml)**
    1. Update `server/Cargo.toml` to add feature flags:
       - Define features: `local_auth`, `google_auth`, `github_auth`, `db_pg`, `stripe_payment`, `chat`
       - Set default features = all except `db_pg`
       - Make dependencies conditional based on features (e.g., `async-stripe` only with `stripe_payment`)

    2. Refactor database module (`server/src/db/`):
       - Create feature-gated database backends
       - Use `#[cfg(feature = "db_pg")]` for PostgreSQL code
       - Use `#[cfg(not(feature = "db_pg"))]` for SQLite code
       - Update connection pool initialization

    3. Refactor auth handlers (`server/src/handlers/auth_handler.rs`, `oauth_handler.rs`):
       - Gate local auth endpoints with `#[cfg(feature = "local_auth")]`
       - Gate Google OAuth with `#[cfg(feature = "google_auth")]`
       - Gate GitHub OAuth with `#[cfg(feature = "github_auth")]`
       - Update route registration to conditionally include endpoints

    4. Refactor payment handling (`server/src/handlers/payment_handler.rs`, `server/src/services/payment_service.rs`):
       - Gate all Stripe-related code with `#[cfg(feature = "stripe_payment")]`
       - Add alternative flow for non-payment mode (invite-only error)

    5. Create feature configuration endpoint:
       - Add `/api/config/features` endpoint that returns enabled features
       - Client will use this to determine which UI components to show

    **Phase 2: Client-Side Feature Detection**
    1. Create feature configuration service (`client/src/lib/services/featureService.ts`):
       - Fetch enabled features from server on app load
       - Store in a feature store
       - Export typed feature flags

    2. Update auth components:
       - Conditionally render local auth form based on feature flag
       - Conditionally render OAuth buttons based on feature flags
       - Update `client/src/routes/login/+page.svelte` and `register/+page.svelte`

    3. Update payment flow:
       - Check if `stripe_payment` is enabled before redirecting to payment
       - Show "invite-only" message if payment is disabled

    4. Update main navigation:
       - Conditionally show chat route based on `chat` feature
       - Create simple holding page for when chat is disabled

    **Phase 3: Build Configuration**
    1. Update build scripts in `justfile`:
       - Add commands to build with specific features
       - Example: `just build-server-features "local_auth,db_pg"`

    2. Create feature validation:
       - Ensure incompatible features cannot be enabled together
       - Validate required environment variables for enabled features

    **Phase 4: Testing & Documentation**
    1. Create feature-specific tests:
       - Test each feature in isolation
       - Test feature combinations
       - E2E tests for different feature sets

    2. Update documentation:
       - Add feature flag documentation to README.md
       - Create FEATURES.md with detailed feature descriptions
       - Update deployment guides with feature-specific configurations

*   **Files to Create/Modify:**
    *   `server/Cargo.toml` - Add feature definitions
    *   `server/src/config.rs` - Add feature configuration
    *   `server/src/handlers/config_handler.rs` - New: feature config endpoint
    *   `server/src/db/mod.rs` - Feature-gated database backends
    *   `server/src/routes.rs` - Conditional route registration
    *   `client/src/lib/services/featureService.ts` - New: feature detection
    *   `client/src/lib/stores/featureStore.ts` - New: feature state management
    *   `client/src/routes/+layout.ts` - Load features on app init
    *   `documentation/FEATURES.md` - New: feature documentation
    *   Update all auth/payment/chat components for conditional rendering

*   **Implementation Notes:**
    *   Use compile-time feature flags for server to reduce binary size
    *   Client features determined at runtime from server config
    *   Ensure graceful degradation when features are disabled
    *   Document options clearly in `README.md` and architecture documentation

*   **Quality Checks:**
    *   Include e2e tests to assess whether the correct code is generated on the frontend
    *   Verify all feature combinations work correctly
    *   Test that disabled features are completely removed from client bundle
    *   Ensure no runtime errors when features are disabled
    *   Document troubleshooting common issues

### Task 1.4: Deployment Guides and tools
*   **Status:** **[ ] TODO**
*   **Action:** Create comprehensive deployment guides for major cloud platforms
*   **Details:**
    *   Create a PostgreSQL database provider option
    *   Create GCP Cloud Run deployment with Docker
    *   Create fly.io deployment with Docker
    *   Create Vercel deployment
    *   Create CI/CD pipeline examples (GitHub Actions)
    *   Add monitoring and logging setup guides - may need refactor of server tracing to support JSON-style GCP logging
    *   Create environment variable management guides
*   **Files to Create:**
    *   `documentation/deployment/gcp-cloud-run.md`
    *   `documentation/deployment/flyio.md`
    *   `documentation/deployment/vercel.md`
    *   `documentation/deployment/database-setup.md`
    *   `documentation/deployment/ci-cd.md`
    *   `documentation/deployment/monitoring.md`
    *   `.github/workflows/` (CI/CD workflow examples)
*   **Implementation Notes:**
    *   Include Docker configurations for production
    *   Add health check endpoints for deployment platforms
    *   Document scaling considerations and performance tuning
    *   Include cost optimization strategies
*   **Quality Checks:**
    *   Test deployment guides on actual platforms
    *   Verify all environment variables and configurations
    *   Document troubleshooting common issues
