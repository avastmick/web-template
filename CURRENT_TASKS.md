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
*   **Status:** **[ ] TODO
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

*   **Current Issues Identified:**
    1. **OAuth Registration Blocks Non-Invited Users:** OAuth users without invites cannot register at all - they should register and be redirected to payment
    2. **Payment Status Not Cached:** Client fetches payment status on every route navigation instead of using sessionStorage
    3. **Missing Expiry Validation:** Database has expiry fields but they're not consistently checked
    4. **UI Navigation Bug:** Svelte `goto()` causes UI stacking - need proper navigation handling
    5. **Inconsistent Error Handling:** Different auth methods show errors differently

*   **Implementation Steps (in order):**

    **Step 1: Database Schema Updates**
    - Update `user_invites` table: make `expires_at` NOT NULL with default expiry
    - Create new migration to ensure all existing invites have expiry dates
    - Add indexes for performance on email lookups

    **Step 2: Server-Side Payment User Model**
    - Create unified `PaymentUser` response model with fields:
      - `payment_or_invite_exists: bool`
      - `payment_or_invite_status_expires: Option<DateTime>`
      - `payment_type: enum { Invite, Subscription, OneTime }`
      - `requires_payment: bool`
    - Create service method to build PaymentUser from user data

    **Step 3: Fix OAuth Registration Flow**
    - Modify `oauth_handler.rs` to allow registration without invite
    - Set `payment_required = true` for non-invited OAuth users
    - Return same response structure as email/password registration
    - Ensure OAuth callback handles payment redirect properly

    **Step 4: Centralize Auth Response Handler (Server)**
    - Create `AuthResponseBuilder` service that:
      - Checks invite status
      - Checks payment status with expiry validation
      - Builds consistent response for all auth methods
      - Validates expiry dates against current time
    - Use in both registration and login handlers

    **Step 5: Client-Side Storage Service**
    - Create `StorageService` in `client/src/lib/services/storageService.ts`:
      - Manages localStorage for auth data
      - Manages sessionStorage for payment data
      - Provides methods for checking staleness
      - Handles storage errors gracefully

    **Step 6: Client-Side Auth Flow Manager**
    - Create `AuthFlowManager` in `client/src/lib/services/authFlowManager.ts`:
      - Single source of truth for auth/payment state
      - Handles all redirects (configurable success route)
      - Checks payment expiry on route changes
      - Refreshes payment status when stale
      - Uses proper navigation (fix goto() issue)

    **Step 7: Update Auth Guard**
    - Modify `authGuard.ts` to use AuthFlowManager
    - Remove direct API calls
    - Use cached payment status from sessionStorage
    - Only refresh when data is stale or missing

    **Step 8: Fix Payment Page**
    - Add proper authentication headers to Stripe API calls
    - Handle payment success/failure redirects
    - Update payment status in sessionStorage on success
    - Show clear error messages for common issues

    **Step 9: Update All Auth Components**
    - Update login/register pages to use AuthFlowManager
    - Ensure consistent error handling
    - Remove duplicate logic
    - Use centralized navigation

    **Step 10: Documentation Updates**
    - Update `CLIENT_STORAGE.md` with new storage pattern
    - Document navigation workaround in `ARCHITECTURE.md`
    - Add auth flow diagram to documentation

    **Step 11: E2E Tests**
    - Test email/password registration with/without invite
    - Test OAuth registration with/without invite
    - Test payment flow completion
    - Test expiry date handling
    - Test session storage persistence
    - Test navigation between protected routes

*   **Files to Create/Modify:**
    **Database:**
    - `db/migrations/XXXXXX_add_invite_expiry_not_null.sql` (new migration)

    **Server - New Files:**
    - `server/src/models/payment_user.rs` (PaymentUser model)
    - `server/src/services/auth_response_builder.rs` (centralized auth response)

    **Server - Modify:**
    - `server/src/handlers/oauth_handler.rs` (fix OAuth registration)
    - `server/src/handlers/auth_handler.rs` (use AuthResponseBuilder)
    - `server/src/services/invite_service.rs` (add expiry validation)
    - `server/src/services/payment/payment_service.rs` (add expiry checks)

    **Client - New Files:**
    - `client/src/lib/services/storageService.ts` (storage management)
    - `client/src/lib/services/authFlowManager.ts` (auth flow control)
    - `client/src/lib/types/storage.ts` (storage types)

    **Client - Modify:**
    - `client/src/lib/guards/authGuard.ts` (use AuthFlowManager)
    - `client/src/routes/auth/oauth/callback/+page.svelte` (fix redirects)
    - `client/src/routes/login/+page.svelte` (use AuthFlowManager)
    - `client/src/routes/register/+page.svelte` (use AuthFlowManager)
    - `client/src/routes/payment/+page.svelte` (fix Stripe auth)
    - `client/src/lib/stores/authStore.ts` (integrate with StorageService)

    **Documentation:**
    - `documentation/CLIENT_STORAGE.md` (update storage patterns)
    - `documentation/ARCHITECTURE.md` (document navigation issue)
    - `documentation/AUTH_FLOW.md` (new - auth flow diagram)

    **Tests:**
    - `client/e2e/auth-flow.test.ts` (new - comprehensive auth tests)
    - `server/tests/auth_flow_test.rs` (new - server auth tests)

*   **Implementation Notes:**
    - the client is CSR only (there is no SSR aspect at all), ensure this is clearly documented.
    - the `documentation/CLIENT_STORAGE.md` needs to be updated as it is outdated. `payment_user` is new and not documented.
    - there is an issue with using the Svelte `goto()` whereby the previous UI element is not unloaded and instead the new UI is appended below the old UI - for a user it appears the page has not changed (expect the URL in the browser). As a workaround `window.location.href` has been used successfully. Please ensure that this is clearly documented in the `ARCHITECTURE.md` as an issue to resolve / manage.
    - ALWAYS use the simplest solution and ALWAYs use the DRY principle, with a single handler for registration/invite/payment/auth state management and flow on the client.
*   **Quality Checks:**
    *   Use `playwright` MCP to test the registration / authn flow to ensure consistency and correctness
    *   Create e2e tests that ensure the registration/authn and invite/payment flow works correctly in future

### Task 1.3: Client - UI overhaul, optimisation and refinement
*   **Status:** **[ ] TODO
*   **Action:** Review the current approach to the UI and theming. Update UI on client to optimise the user experience and make all components and theming consistent
*   **Details:**
    *   Review the current UI documentation, `UI-UX_SPECIFICATION.md` and `UI_UX_THEME.md`, and understand the shortcomings of the current approach to theming
    *   Update documentation to provide a simpler, `tailwindcss`-based (i.e. with little to no custom colours) approach that will ensure consistent UI/UX theming and simple "in one place" configuration to enable easier changes
    *   Update the CSS to reflect the new approach
    *   Ensure all components align to new approach and there is a simple, non-repeating, set of components that allow for easier changes and consistent outcomes.
*   **Files to Create/Modify:**
    * TODO - Update this!
*   **Implementation Notes:**
    *   Use `tailwindcss` as means of styling. Remove all custom CSS that does not directly use `tailwindcss`.
    *   Ensure navigation bar menu is correctly reactive and scales across all screen sizes.
    *   Ensure all navigation bar components are consistent and share look and feel
    *   Ensure that all components have shared styling and theming and fully reusable
    *   Ensure that the theme is consistent across all pages; create a mechanism that allows the simple addition of pages, ensuring the new pages align with application styling, theming, etc.
    *   Ensure that the dark/light theming is logical and the base colours are opposites - e.g., if the theme background for 'dark' is `indigo-950`, then by logic the 'light' is `indigo-50`, etc.
*   **Quality Checks:**
    *   Use `playwright` MCP to test the UI and its consistency of look and feel
    *   Create e2e tests that ensure the UI works as expected
    *   Create a test that shows that styling and theme can be updated in one place and affect all of the client

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
