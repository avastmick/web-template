# Current Tasks

This document outlines the tasks to be completed based on `INSTRUCTIONS.md` and `PRD.md`.

## Status Summary

**Task 1.1: GitHub OAuth Integration** - **[✓] COMPLETED**
- All GitHub OAuth implementation completed successfully
- All server tests (42) and client tests (10) passing
- Code formatting and quality checks passing
- Test suite issues resolved (environment variable isolation in server tests)

---

### Task 1.1: Server & Client - GitHub OAuth Integration with Invite System

This task implements GitHub OAuth as an additional authentication provider alongside the existing oAuth provider, Google.

**Key Requirements:**
- Only invited users can register or login (both local and OAuth)
- Invites are stored in a `user_invite` table with email addresses
- Registration/login attempts by non-invited users should be rejected with appropriate on-screen error messages
- Invite validation should occur before user creation in both local and OAuth flows

#### Sub-task 1.1.1: Server - Extend OAuth Models for GitHub Support
*   **Status:** **[✓] COMPLETED**
*   **Action:** Extend existing OAuth models and configuration to support GitHub as a provider
*   **Details:**
    *   Add `GitHub` variant to `OAuthProvider` enum in `server/src/models/oauth.rs`
    *   Create `GitHubUserInfo` struct following the pattern of `GoogleUserInfo`
    *   Update `OAuthConfig` in `server/src/config/oauth.rs` to support multiple providers
    *   Add GitHub OAuth environment variables: `GITHUB_CLIENT_ID`, `GITHUB_CLIENT_SECRET`, `GITHUB_REDIRECT_URI`
    *   Update `.envrc.example` with GitHub OAuth variables
*   **Files to Modify:**
    *   `server/src/models/oauth.rs` (add GitHub to enum and create GitHubUserInfo struct)
    *   `server/src/config/oauth.rs` (extend to support GitHub provider)
    *   `server/src/main.rs` (validate GitHub env variables on startup)
    *   `.envrc.example` (add GitHub OAuth variables)
*   **Implementation Notes:**
    *   GitHub user info endpoint: `https://api.github.com/user`
    *   GitHub OAuth authorize URL: `https://github.com/login/oauth/authorize`
    *   GitHub token URL: `https://github.com/login/oauth/access_token`
*   **Quality Checks:**
    *   `just check-server` (formatting, linting, type checking)
    *   `just build-server` (ensure compilation succeeds)


#### Sub-task 1.1.2: Server - Extend OAuth Service for GitHub
*   **Status:** **[✓] COMPLETED**
*   **Action:** Add GitHub-specific methods to the OAuth service
*   **Details:**
    *   Add `github()` method to `OAuthService` that returns GitHub OAuth client
    *   Implement `get_github_user_info()` method following `get_google_user_info()` pattern
    *   Add proper error handling for GitHub-specific API responses
    *   Ensure the service can handle both Google and GitHub providers dynamically
*   **Files to Modify:**
    *   `server/src/services/oauth_service.rs` (add GitHub methods)
*   **Implementation Notes:**
    *   GitHub requires User-Agent header for API calls
    *   GitHub returns email in a separate API call to `/user/emails` if not public
    *   Use the same `oauth2` and `reqwest` crates as Google implementation
*   **Quality Checks:**
    *   Unit tests for new GitHub methods
    *   `just check-server`
    *   `just test-server oauth_service`

#### Sub-task 1.1.3: Server - GitHub OAuth Handler Implementation
*   **Status:** **[✓] COMPLETED**
*   **Action:** Create GitHub-specific OAuth handlers following the existing Google OAuth pattern
*   **Details:**
    *   Create `github_auth()` handler for `GET /api/auth/oauth/github` - returns GitHub authorization URL
    *   Create `github_callback()` handler for `GET /api/auth/oauth/github/callback` - handles OAuth callback
    *   Follow the exact pattern of `google_auth()` and `google_callback()` handlers
    *   Ensure invite validation is performed before creating new users
    *   Reuse the existing `handle_oauth_callback()` helper function
*   **Files to Modify:**
    *   `server/src/handlers/oauth_handler.rs` (add GitHub handlers)
    *   `server/src/routes.rs` (add GitHub OAuth routes)
*   **Implementation Notes:**
    *   The handlers should be nearly identical to Google handlers, just using GitHub provider
    *   State parameter should be properly validated to prevent CSRF
    *   Error responses should match existing patterns for consistency
*   **Quality Checks:**
    *   `just check-server`
    *   Manual testing with actual GitHub OAuth app
    *   Verify invite system blocks unauthorized users

#### Sub-task 1.1.4: Client - GitHub OAuth UI Components
*   **Status:** **[✓] COMPLETED**
*   **Action:** Create GitHub Sign-In button following the existing Google OAuth pattern
*   **Details:**
    *   Create `GitHubSignInButton.svelte` component similar to existing OAuth button
    *   Add GitHub logo/icon (use SVG for consistency)
    *   Style button to match existing UI design with dark mode support
    *   Add "Sign in with GitHub" button to login and registration pages
    *   Ensure proper loading states and error handling
*   **Files to Create/Modify:**
    *   `client/src/lib/components/GitHubSignInButton.svelte` (new component)
    *   `client/src/routes/(auth)/login/+page.svelte` (add GitHub button)
    *   `client/src/routes/(auth)/register/+page.svelte` (add GitHub button)
*   **Implementation Notes:**
    *   Follow the exact component structure as Google sign-in button
    *   Use consistent spacing and styling with existing OAuth buttons
    *   Ensure accessibility (proper ARIA labels, keyboard navigation)
*   **Quality Checks:**
    *   `just check-client`
    *   Visual testing in both light and dark modes
    *   Test button states (normal, hover, disabled, loading)

#### Sub-task 1.1.5: Client - GitHub OAuth API Integration
*   **Status:** **[✓] COMPLETED**
*   **Action:** Add GitHub OAuth methods to the auth service
*   **Details:**
    *   Add `initiateGitHubOAuth()` method following the `initiateGoogleOAuth()` pattern
    *   Reuse existing `handleOAuthCallback()` method (it's provider-agnostic)
    *   No changes needed to user types (already supports provider field)
*   **Files to Modify:**
    *   `client/src/lib/services/apiAuth.ts` (add initiateGitHubOAuth method)
*   **Implementation Notes:**
    *   The method should simply redirect to `/api/auth/oauth/github`
    *   No additional types needed - existing OAuth infrastructure supports multiple providers
    *   The callback handler at `/auth/callback` already works for any provider
*   **Quality Checks:**
    *   `just check-client`
    *   Test complete OAuth flow with GitHub
    *   Verify user data is properly stored in authStore

#### Sub-task 1.1.6: End-to-End OAuth Testing
*   **Status:** **[✓] COMPLETED**
*   **Action:** Extend E2E tests to cover GitHub OAuth flow
*   **Details:**
    *   Add GitHub OAuth tests to existing test suite
    *   Test complete sign-in flow with GitHub
    *   Test registration of new users via GitHub OAuth
    *   Test invite validation (both valid and invalid invites)
    *   Test error handling for OAuth failures
*   **Files to Modify:**
    *   `client/tests/auth.test.ts` (add GitHub OAuth test cases)
*   **Implementation Notes:**
    *   Mock OAuth provider responses for consistent testing
    *   Test both success and failure scenarios
    *   Ensure tests cover the invite validation flow
*   **Quality Checks:**
    *   `just test-e2e` (run all E2E tests)
    *   All tests should pass
    *   No flaky tests

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
