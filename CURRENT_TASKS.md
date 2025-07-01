# Current Tasks

This document outlines the tasks to be completed based on `INSTRUCTIONS.md` and `PRD.md`.

## Status Summary

### Task 1.1: Server & Client - Stripe Payment Integration
*   **Status:** **[x] DONE
*   **Action:** Integrate Stripe for subscription and one-time payment processing for users that do not have an 'invite'. Once registered, non-invited users will be presented with a Stripe payment request. All payment will be handled by Stripe, only the current status of the user's payment is needed to be held - i.e. `has_paid`, `is_current`, or similar.
*   **Current Payment Flow Issues to Fix:**
    *   OAuth registration for non-invited users shows error instead of redirecting to payment page
    *   Payment page shows Stripe initialization error (needs authentication)
    *   OAuth callback redirect should go to `/` instead of old `/profile` route
*   **Details:**
    *   Document how to integrate Stripe and which API keys, etc are required and how to get it setup
    *   Set up Stripe webhook handling for payment events
    *   Add one-time payment processing
*   **Files to Create/Modify:**
    *   `server/src/services/payment_service.rs` (Stripe integration)
    *   `server/src/handlers/payment_handler.rs` (payment endpoints)
    *   `server/src/models/payment.rs` (payment data models)
    *   `server/src/middleware/stripe_webhook.rs` (webhook verification)
    *   `client/src/lib/components/payments/` (payment UI components)
    *   `client/src/lib/services/payments.ts` (payment API client)
    *   `client/src/routes/payment/` (payment routes)
*   **Implementation Notes:**
    *   Use Stripe's official Rust SDK
    *   Implement idempotency for payment operations
    *   Add comprehensive webhook event handling
    *   Ensure PCI compliance best practices - Stripe will handle all payment, card, etc. nothing stored in application
*   **Quality Checks:**
    *   Test with Stripe test mode and webhook testing
    *   Security audit for payment data handling
    *   Integration tests for payment flows

### Task 1.2: Registration, Authn, Payment/Invite flow refactor
*   **Status:** **[x] DONE - Server and Client Complete (2024-06-30)
*   **Action:** The whole registration/authentication/payment/authorization flow needs to be heavily refactored.
Review the current approach to registration and refactor to enable watertight registration/authn/payment/invite handling
*   **Details:**
    *   You need to refactor to align with industry best practice and NOT implement tactical point solutions that may vary from client route to route. The new solution MUST be watertight and provably robust. There MUST be e2e tests to prove the flow and the expected outcomes.
    *   full flow:
        Registration:
        1. User registers using either oauth or email/password
        2. The server checks the user email for duplicate - fails registration if existing email found
        3. The server checks if the user email exists in the 'user_invites' table - if it does there is no need for payment
        4. The server registers the user - the server returns a set of data: `auth_user` and `auth_token` and `payment_user`. The first two holds the user's auth state and JWT token for API use. The latter holds their payment status and its expiry date (both user_invites AND payments MUST have expiry dates - the database migration script needs to be updated, as will the server code)

        Login:
        1. They can use email/password, or oauth. The existing server flow for each is used, but with updated handling for payment/invite. If authn is successful:
        2. The server provides `auth_user` and `auth_token` and `payment_user` information.
        3. The client stores the `auth_user` and `auth_token` in `localStorage` and `payment_user` details in `sessionStorage`
        4. If the `payment_user` has `payment_or_invite_exists` == false OR `payment_or_invite_status_expires` is <= `today()` then the user is directed to the `/payment` route to pay, else they are directed to `/` home
        5. If the user starts a new session, or the `payment_user` is stale, the client MUST fetch a new `payment_user` object and store it appropriately. If the payment has expired go to '4'.
        6. A successfully authn'd user with a valid payment/invite will be redirected to `/chat` (should be configurable in ONE place).

### Task 1.3: Client - UI overhaul, optimisation and refinement
*   **Status:** **[x] DONE - All phases complete including documentation consolidation and test updates (2025-07-01)
*   **Action:** Review the current approach to the UI and theming. Update UI on client to optimise the user experience and make all components and theming consistent. General colour theme is using tailwindcss `indigo` for background, at either end of scale; use `amber` for highlighting (focus border, hover over links and buttons, etc); contrast colours for ease of viewing.
*   **Details:**

### Task 1.4: Enable workspace 'features'
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
*   **Files to Create:**
    *   TODO: list changes here
*   **Implementation Notes:**
    *   Document options clearly in `README.md` and architecture documentation
*   **Quality Checks:**
    *   Include e2e tests to assess whether the correct code is generated on the frontend
    *   Verify all options
    *   Document troubleshooting common issues

### Task 1.5: Deployment Guides and tools
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

### Task 1.6: Developer Experience - Template Scaffolding
*   **Status:** **[ ] TODO**
*   **Action:** Create scaffolding tools for new projects using this template
*   **Details:**
    *   Create CLI tool for project initialization
    *   Implement interactive project setup wizard
    *   Add template customization options (features to include/exclude)
    *   Create project renaming and rebranding automation
    *   Add development environment setup automation
    *   Implement feature flag system for optional components
    *   Create update mechanism for template improvements
*   **Files to Create:**
    *   `scripts/create-project.rs` (project scaffolding script)
    *   `scripts/setup-wizard.rs` (interactive setup)
    *   `scripts/update-template.rs` (template update utility)
    *   `template.config.json` (template configuration)
    *   `documentation/template-usage.md` (usage guide)
*   **Implementation Notes:**
    *   Use Cargo for cross-platform compatibility
    *   Implement file templating with variable substitution
    *   Add Git repository initialization and cleanup
    *   Support different database options during setup
*   **Quality Checks:**
    *   Test project creation on different operating systems
    *   Verify generated projects build and run correctly
    *   Test with different configuration combinations

## **Note on Testing:**
*   **Server:** Unit tests for individual functions/modules. Integration tests for API endpoints (testing request/response, database interaction). Use `cargo test`.
*   **Client:** Unit tests for Svelte components, stores, and services (using Vitest or Jest). E2E tests for user flows (using Playwright). Use `bun test` with `just test` wrappers.
*   All tests should be runnable via `just` commands (e.g., `just server-test`, `just client-test`, `just test-e2e`).
