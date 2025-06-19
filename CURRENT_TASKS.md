# Current Tasks

This document outlines the tasks to be completed based on `INSTRUCTIONS.md` and `PRD.md`.

### Task 1.1: Server & Client - GitHub OAuth Integration with Invite System

This task implements GitHub OAuth as an additional authentication provider alongside the existing oAuth provider, Google.

**Key Requirements:**
- Only invited users can register or login (both local and OAuth)
- Invites are stored in a `user_invite` table with email addresses
- Registration/login attempts by non-invited users should be rejected with appropriate on-screen error messages
- Invite validation should occur before user creation in both local and OAuth flows

#### Sub-task 1.1.1: Server - OAuth Configuration and Dependencies to support GitHub
*   **Status:** **[ ] TODO**
*   **Action:** Add OAuth dependencies and configuration to support GitHub
*   **Details:**
    *   Add required crates: `oauth2`, `reqwest` (for token verification)
    *   Create OAuth configuration module for managing provider settings
    *   Add GitHub OAuth environment variables validation to startup
*   **Files to Create/Modify:**
    *   `server/src/config/oauth.rs` (OAuth provider configuration)
    *   `server/src/config/mod.rs` (export oauth module)
    *   `server/src/main.rs` (validate GitHub env variables on startup)
*   **Quality Checks:**
    *   `just check-server` (formatting, linting, type checking)
    *   `just build-server` (ensure compilation succeeds)


#### Sub-task 1.1.4: Server - OAuth Endpoints Implementation
*   **Status:** **[ ] TODO**
*   **Action:** Implement OAuth-specific API endpoints
*   **Details:**
    *   `GET /api/auth/oauth/google` - Initiates OAuth flow, returns authorization URL
    *   `GET /api/auth/oauth/google/callback` - Handles OAuth callback, exchanges code for tokens
    *   Create or update user based on GitHub profile
    *   Issue JWT upon successful authentication
*   **Files to Create/Modify:**
    *   `server/src/handlers/oauth_handler.rs` (OAuth request handlers)
    *   `server/src/handlers/mod.rs` (export oauth_handler)
    *   `server/src/routes.rs` (add OAuth routes)
    *   `server/tests/oauth_integration_tests.rs` (comprehensive OAuth flow tests)
*   **Quality Checks:**
    *   Integration tests for OAuth endpoints
    *   Test error cases (invalid code, expired tokens, unauthorized users)
    *   `just server-test` (all tests should pass)

#### Sub-task 1.1.5: Client - OAuth UI Components
*   **Status:** **[ ] TODO**
*   **Action:** Create GitHub Sign-In button and OAuth flow UI
*   **Details:**
    *   Add "Sign in with GitHub" button to login and registration pages
    *   Handle OAuth redirect flow
    *   Show loading state during OAuth callback processing
    *   Update authStore to handle OAuth tokens
*   **Files to Create/Modify:**
    *   `client/src/lib/components/GitHubSignInButton.svelte` (reusable OAuth button)
    *   `client/src/routes/login/+page.svelte` (add GitHub sign-in option)
    *   `client/src/routes/register/+page.svelte` (add GitHub sign-in option)
    *   `client/src/routes/auth/google/callback/+page.svelte` (OAuth callback handler)
*   **Quality Checks:**
    *   `just check-client` (formatting, linting, type checking)
    *   Visual testing of OAuth button styling
    *   `just build-client` (ensure production build succeeds)

#### Sub-task 1.1.6: Client - OAuth API Integration
*   **Status:** **[ ] TODO**
*   **Action:** Extend auth service to support OAuth flow
*   **Details:**
    *   Add methods for initiating OAuth flow
    *   Handle OAuth callback and token storage
    *   Update user type to include provider information
*   **Files to Create/Modify:**
    *   `client/src/lib/services/apiAuth.ts` (add OAuth methods: initiateGitHubAuth, handleOAuthCallback)
    *   `client/src/lib/types/auth.ts` (add OAuth types: OAuthProvider, extend User type)
    *   `client/src/lib/stores/authStore.ts` (handle OAuth user data)
*   **Quality Checks:**
    *   `just check-client`
    *   Test OAuth flow end-to-end manually
    *   Ensure existing auth flows still work

#### Sub-task 1.1.7: End-to-End OAuth Testing
*   **Status:** **[ ] TODO**
*   **Action:** Create comprehensive E2E tests for OAuth flow for GitHub
*   **Details:**
    *   Test complete OAuth sign-in flow
    *   Test OAuth registration for new users
    *   Test linking OAuth to existing accounts (if implemented)
    *   Test access control (ALLOWED_USERS)
*   **Files to Create/Modify:**
    *   `client/e2e/oauth.test.ts` (Playwright tests for OAuth flows)
*   **Quality Checks:**
    *   `just test-e2e oauth` (run OAuth E2E tests)
    *   Manual testing with real GitHub accounts
    *   Test on different browsers

### Task 2.1: Client - Layout and styling using tailwindcss
*   **Status:** **[ ] TODO**
*   **Action:** Implement UI layout and base styling
*   **Details:** (To be expanded)

### Task 2.2: Client - Dark/Light Mode & Color Schemes
*   **Status:** **[ ] TODO**
*   **Action:** Implement theme system with dark/light modes and customizable color schemes
*   **Details:** (To be expanded)

### Task 2.3: Server - Generative AI Integration Framework
*   **Status:** **[ ] TODO**
*   **Action:** Create flexible framework for integrating various AI providers
*   **Details:** (To be expanded)

### Task 2.4: Server & Client - Stripe Payment Integration
*   **Status:** **[ ] TODO**
*   **Action:** Integrate Stripe for payment processing
*   **Details:** (To be expanded)

### Task 2.5: Documentation - Deployment Guides
*   **Status:** **[ ] TODO**
*   **Action:** Create deployment guides for GCP Cloud Run, Vercel, and Supabase
*   **Details:** (To be expanded)

### Task 2.6: Template usage - create a simple means of using the current template for a new project
*   **Status:** **[ ] TODO**
*   **Action:** Create script that will use a clean template instance for a new project
*   **Details:** (To be expanded)


**Note on Testing:**
*   **Server:** Unit tests for individual functions/modules. Integration tests for API endpoints (testing request/response, database interaction). Use `cargo test`.
*   **Client:** Unit tests for Svelte components, stores, and services (using Vitest or Jest). E2E tests for user flows (using Playwright). Use `bun test`.
*   All tests should be runnable via `just` commands (e.g., `just server-test`, `just client-test`, `just test-e2e`).
