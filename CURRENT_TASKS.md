# Current Tasks

This document outlines the tasks to be completed based on `INSTRUCTIONS.md` and `PRD.md`.

## Phase 1: Project Setup and Documentation

1.  **[X] Update `README.md`**
    *   **Action:** Align with project goals from `PRD.md`.
    *   **Action:** Add an \"Architecture\" section.
    *   **Files:** `web-template/README.md`
    *   **Quality Checks:** Manual review.

2.  **[X] Update `CLAUDE.md`**
    *   **Action:** Align with project goals from `PRD.md`.
    *   **Action:** Ensure it reflects instructions from `INSTRUCTIONS.md` regarding project management and quality.
    *   **Files:** `web-template/CLAUDE.md`
    *   **Quality Checks:** Manual review.

3.  **[X] Create `documentation/ARCHITECTURE.md`**
    *   **Action:** Create the file and add initial content based on `PRD.md` and project structure (`svelte-kit` client, `rust` server).
    *   **Details:** Describe the client (SvelteKit), server (Rust/Axum), database (SQLite initially), and their interactions. Mention `dbmate` for migrations and `just` for commands. Cover high-level components for auth, payments, and AI integration as per `PRD.md`.
    *   **Files:** `web-template/documentation/ARCHITECTURE.md`
    *   **Quality Checks:** Manual review.

4.  **[X] Review and Update `justfile`**
    *   **Action:** Ensure `justfile` includes commands for project lifecycle.
    *   **Files:** `web-template/justfile`
    *   **Quality Checks:** `just --list` and execution of several key commands.

5.  **[X] Configure Pre-commit Hooks**
    *   **Action:** Create/update `.pre-commit-config.yaml`.
    *   **Files:** `web-template/.pre-commit-config.yaml`
    *   **Quality Checks:** User will verify with `git commit --dry-run`.

6.  **[X] Update `README.md` and `CLAUDE.md` with Project Management Details**
    *   **Action:** Add detailed instructions on `just` commands and quality (pre-commit hooks).
    *   **Files:** `web-template/README.md`, `web-template/CLAUDE.md`
    *   **Quality Checks:** Manual review.

## Phase 2: User Registration and Login Feature (Local Provider)

This phase focuses on implementing the core authentication flow: user registration, login, and viewing credentials using a local email/password provider. Rigorous security and best practices are paramount.

### Task 2.1: Design User Authentication Flow
*   **Status:** **[X] DONE**
*   **Action:** Define the detailed steps for user registration, login, session management (JWT), and profile viewing.
*   **Output:** Updated `documentation/ARCHITECTURE.md` with an "Authentication Flow" section.
*   **Quality Checks:** Manual review for security considerations and completeness.

### Task 2.2: Server - User Model and Database Migration
*   **Status:** **[X] DONE**
*   **Action:** Define the Rust struct for the `User` model (and `UserFromDb` for mapping).
*   **Action:** Create a `dbmate` migration for the `users` table.
*   **Files:**
    *   `web-template/server/src/models/user.rs`
    *   `web-template/server/src/models/mod.rs`
    *   `web-template/db/migrations/YYYYMMDDHHMMSS_create_users_table.sql`
*   **Quality Checks:**
    *   `just check-server` - PASSED (after fixes)
    *   `just check-client` - PASSED (after fixes)
    *   `dbmate up` (via `just db-migrate`) - PASSED
    *   `dbmate down` (via `just db-rollback`) - PASSED
    *   `dbmate up` again - PASSED

### Task 2.3: Server - Registration Endpoint
*   **Status:** **[ ] IN PROGRESS**
*   **Action:** Implement the `POST /api/auth/register` endpoint.
*   **Details:**
    *   Request: `email`, `password`.
    *   Validate email format and uniqueness.
    *   Validate password strength.
    *   Hash password securely.
    *   Store new user in the database.
    *   Return appropriate success (e.g., 201 Created with user ID/email) or error responses (400, 409, 500).
*   **Files Created/Modified:**
    *   `web-template/server/src/handlers/auth_handler.rs`
    *   `web-template/server/src/handlers/mod.rs`
    *   `web-template/server/src/services/user_service.rs`
    *   `web-template/server/src/services/mod.rs`
    *   `web-template/server/src/core/password_utils.rs`
    *   `web-template/server/src/core/mod.rs`
    *   `web-template/server/src/errors.rs`
    *   `web-template/server/src/routes.rs`
    *   `web-template/server/src/main.rs`
    *   `web-template/server/Cargo.toml` (dependencies added)
*   **Current Blocker:** `just check-server` (via `direnv exec .`) failing with E0277 errors in `user_service.rs` due to `sqlx::query_as!` type mapping issues. This needs to be resolved to get to a commit-able baseline.
*   **Quality Checks (Pending after current error resolution):**
    *   `just server-check`
    *   Unit tests for validation logic, password hashing, and user creation service.
    *   Integration test for the `/api/auth/register` endpoint.
    *   `just server-build`

### Task 2.4: Server - Login Endpoint and JWT Issuance
*   **Status:** **[ ] TODO**
*   **Action:** Implement the `POST /api/auth/login` endpoint.
*   **Action:** Implement JWT generation and signing.
*   **Details:**
    *   Request: `email`, `password`.
    *   Find user by email.
    *   Verify password against stored hash.
    *   If valid, generate a JWT containing user ID and other relevant claims.
    *   JWT secret should be from `JWT_SECRET` environment variable.
    *   Return JWT in response (e.g., in an HttpOnly cookie or JSON body).
*   **Files:**
    *   `web-template/server/src/handlers/auth_handler.rs`
    *   `web-template/server/src/services/auth_service.rs` (for JWT logic)
    *   `web-template/server/src/routes.rs`
    *   Update `web-template/server/src/main.rs` or config for JWT secret.
*   **Quality Checks:**
    *   `just server-check`
    *   Unit tests for password verification and JWT generation/validation.
    *   Integration test for `/api/auth/login` endpoint. Test success, invalid credentials, non-existent user.
    *   `just server-build`

### Task 2.5: Server - Protected Endpoint (View Credentials)
*   **Status:** **[ ] TODO**
*   **Action:** Create a protected endpoint (e.g., `GET /api/users/me`) that requires a valid JWT.
*   **Action:** Implement JWT validation middleware/extractor.
*   **Details:**
    *   Middleware should extract JWT from `Authorization` header (Bearer token) or cookie.
    *   Validate JWT signature and expiry.
    *   If valid, extract user ID and fetch user details (excluding password).
    *   Return user credentials (e.g., email, user ID).
*   **Files:**
    *   `web-template/server/src/middleware/auth_middleware.rs` (or Axum extractor)
    *   `web-template/server/src/handlers/user_handler.rs`
    *   `web-template/server/src/routes.rs`
*   **Quality Checks:**
    *   `just server-check`
    *   Unit tests for JWT validation logic.
    *   Integration tests for `/api/users/me`:
        *   With valid JWT.
        *   Without JWT (expect 401).
        *   With invalid/expired JWT (expect 401).
    *   `just server-build`

### Task 2.6: Client - State Management for Auth
*   **Status:** **[ ] TODO**
*   **Action:** Set up Svelte stores for managing authentication state (e.g., current user, JWT, loading status, errors).
*   **Files:** `web-template/client/src/lib/stores/authStore.ts` (or similar)
*   **Quality Checks:**
    *   `just client-check`
    *   Unit tests for store logic if complex.
    *   `just client-build`

### Task 2.7: Client - API Service for Auth
*   **Status:** **[ ] TODO**
*   **Action:** Create a service/module to handle API calls to the server's auth endpoints.
*   **Details:**
    *   Functions for `register(email, password)`, `login(email, password)`, `fetchCurrentUser()`.
    *   Should handle setting/clearing JWT (e.g., in localStorage or a secure cookie if client can access).
    *   Integrate with `authStore` to update state.
*   **Files:** `web-template/client/src/lib/services/apiAuth.ts`
*   **Quality Checks:**
    *   `just client-check`
    *   Unit tests for API service functions (can mock `fetch`).
    *   `just client-build`

### Task 2.8: Client - Registration Page/Component
*   **Status:** **[ ] TODO**
*   **Action:** Create a Svelte component/page for user registration.
*   **Details:**
    *   Form with email and password fields.
    *   Client-side validation (mirroring server-side, but server is authoritative).
    *   Call `apiAuth.register()` on submit.
    *   Display success/error messages.
    *   Redirect on successful registration (e.g., to login page or dashboard).
*   **Files:** `web-template/client/src/routes/register/+page.svelte`
*   **Quality Checks:**
    *   `just client-check`
    *   Manual UI testing of registration flow (valid inputs, invalid inputs, existing user).
    *   `just client-build`
    *   Playwright e2e test for registration.

### Task 2.9: Client - Login Page/Component
*   **Status:** **[ ] TODO**
*   **Action:** Create a Svelte component/page for user login.
*   **Details:**
    *   Form with email and password fields.
    *   Call `apiAuth.login()` on submit.
    *   Store JWT and update `authStore` on successful login.
    *   Display success/error messages.
    *   Redirect to a protected page (e.g., profile/dashboard).
*   **Files:** `web-template/client/src/routes/login/+page.svelte`
*   **Quality Checks:**
    *   `just client-check`
    *   Manual UI testing of login flow (valid credentials, invalid credentials).
    *   `just client-build`
    *   Playwright e2e test for login.

### Task 2.10: Client - Profile Page (View Credentials)
*   **Status:** **[ ] TODO**
*   **Action:** Create a Svelte component/page to display user credentials.
*   **Details:**
    *   This page should be protected (require authentication). Use SvelteKit layouts or hooks for route protection.
    *   On load, if user data is not in `authStore`, call `apiAuth.fetchCurrentUser()`.
    *   Display user email and other non-sensitive info.
    *   Include a logout button that clears JWT and `authStore`, then redirects to login/home.
*   **Files:** `web-template/client/src/routes/profile/+page.svelte`, `web-template/client/src/routes/profile/+layout.svelte` (for auth guard)
*   **Quality Checks:**
    *   `just client-check`
    *   Manual UI testing:
        *   Accessing profile when logged in.
        *   Attempting to access profile when logged out (should redirect to login).
        *   Logout functionality.
    *   `just client-build`
    *   Playwright e2e test for accessing profile page and logout.

## Phase 3: Enhancements and Other Requirements from PRD

(Tasks for Google OAuth, Stripe, Dark/Light modes, configurable color schemes, GenAI integration, deployment targets will be detailed here once Phase 1 and 2 are stable.)

*   **[ ] Task 3.1: Client - Dark/Light Mode & Color Schemes**
*   **[ ] Task 3.2: Server & Client - Google OAuth Integration**
*   **[ ] Task 3.3: Server & Client - Stripe Payment Integration**
*   **[ ] Task 3.4: Server - Generative AI Integration Framework**
*   **[ ] Task 3.5: Documentation - Deployment Guides (GCP Cloud Run, Vercel, Supabase)**

**Note on Testing:**
*   **Server:** Unit tests for individual functions/modules. Integration tests for API endpoints (testing request/response, database interaction). Use `cargo test`.
*   **Client:** Unit tests for Svelte components, stores, and services (using Vitest or Jest). E2E tests for user flows (using Playwright). Use `bun test`.
*   All tests should be runnable via `just` commands (e.g., `just server-test`, `just client-test`, `just test-e2e`).
