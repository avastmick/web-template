# Architecture Document

This document outlines the architecture for the `web-template` project.

## 1. Overview

The system is designed as a modern web application with a decoupled frontend and backend.

-   **Client (Frontend):** A SvelteKit single-page application (SPA) responsible for user interface, user experience, and client-side interactions.
-   **Server (Backend):** A Rust-based RESTful API server built with Axum, responsible for business logic, data persistence, and third-party integrations.
-   **Database:** SQLite will be used for local development. The design will allow for easy swapping to other SQL databases (e.g., PostgreSQL) for production. Database migrations are managed by `dbmate`.
-   **Task Runner:** `just` is used as the command runner for managing common development and build tasks across the project.
-   **Environment Management:** `direnv` is used to manage environment variables locally via an `.envrc` file.

## 2. Components

### 2.1. Client (SvelteKit) - `web-template/client/`

-   **Framework:** SvelteKit
-   **Language:** TypeScript (with strict type checking)
-   **Package Manager:** Bun
-   **Key Responsibilities:**
    -   Rendering UI components.
    -   Handling user input and interactions.
    -   Communicating with the backend API via HTTP requests.
    -   Managing client-side state (e.g., user authentication status, UI state) using Svelte stores.
    -   Routing.
-   **Structure (High-Level):**
    -   `src/routes/`: Defines application pages and API routes (for SvelteKit endpoints, if any).
    -   `src/lib/components/`: Reusable Svelte components.
    -   `src/lib/stores/`: Svelte stores for global state management.
    -   `src/lib/services/`: Modules for interacting with the backend API.
    -   `src/lib/utils/`: Common utility functions.
    -   `static/`: Static assets.
    -   `tests/`: Playwright (E2E) and Vitest (unit/integration) tests.

### 2.2. Server (Rust/Axum) - `web-template/server/`

-   **Framework:** Axum (built on Tokio)
-   **Language:** Rust
-   **Package Manager:** Cargo
-   **Key Responsibilities:**
    -   Providing RESTful API endpoints for the client.
    -   User authentication and authorization.
    -   Business logic processing.
    -   Interacting with the database (CRUD operations) via `sqlx`.
    -   Integrating with external services (e.g., payment gateways like Stripe, AI providers).
-   **Structure (High-Level):**
    -   `src/main.rs`: Application entry point, server setup.
    -   `src/routes.rs`: Defines API routes and maps them to handlers.
    -   `src/handlers/`: Request handlers for different API resources.
    -   `src/models/` (or `src/domain/`): Data structures representing business entities.
    -   `src/services/`: Modules containing business logic.
    -   `src/db/` (or `src/repository/`): Database interaction logic, queries using `sqlx`.
    -   `src/middleware/`: Custom Axum middleware (e.g., for authentication, logging).
    -   `src/config.rs`: Application configuration loading.
    -   `src/errors.rs`: Custom error types and error handling.
    -   `tests/`: Integration tests for API endpoints and services.

### 2.3. Database - `web-template/db/`

-   **Migration Tool:** `dbmate`
    -   `migrations/`: Contains SQL migration files.
    -   `schema.sql`: Auto-generated file representing the current database schema.
-   **Local Database:** SQLite (file defined by `DATABASE_URL` in `.envrc`).
-   **Database Access (Server):** `sqlx` crate for asynchronous, compile-time checked SQL queries.

## 3. Key Architectural Goals & Principles

-   **Decoupling:** Frontend and backend are separate applications, communicating via a well-defined API. This allows for independent development, scaling, and technology choices.
-   **Modularity:** Both client and server are structured into modules with clear responsibilities.
-   **Performance:** Technologies (Svelte, Rust, Axum) are chosen for their performance characteristics.
-   **Security:** Emphasis on secure coding practices, secure authentication (JWT-based initially), and data protection.
-   **Scalability:** While starting simple, the architecture should be able to scale (e.g., containerization, stateless server design where possible).
-   **Testability:** Design components and services to be easily testable at unit, integration, and end-to-end levels.
-   **Maintainability:** Clear code structure, good documentation, and consistent coding standards.

## 4. Data Flow (Example: User Registration)

1.  **Client:** User submits registration form (email, password).
2.  **Client:** `AuthService` sends a `POST` request to `/api/auth/register` with user data.
3.  **Server (Axum Router):** Routes request to `auth_handler::register`.
4.  **Server (Handler):** Validates input.
5.  **Server (UserService):** Checks if email exists. Hashes password. Creates new user record in the database via `UserRepository`.
6.  **Server (Handler):** Returns success (e.g., 201 Created) or error response.
7.  **Client:** `AuthService` receives response, updates UI accordingly (e.g., redirect to login, show error message).

## 5. Authentication and Authorization

-   **Initial Strategy:** Local authentication using email and password.
-   **Mechanism:** JSON Web Tokens (JWTs).
    1.  User logs in with credentials.
    2.  Server validates credentials.
    3.  Server issues a signed JWT (containing user ID, roles, expiration) to the client. Secret key for signing is stored in `JWT_SECRET` env variable.
    4.  Client stores JWT (e.g., in an HttpOnly cookie or localStorage for SPA, to be decided based on security trade-offs) and sends it in the `Authorization` header for subsequent requests to protected endpoints.
    5.  Server middleware validates the JWT on incoming requests to protected routes.
-   **Future Enhancements:** Google OAuth integration.

## 6. Future Considerations / Integrations (as per PRD)

-   **Payment Integration (Stripe):** Will involve client-side components for collecting payment information (Stripe Elements) and server-side handlers for processing payments and managing subscriptions.
-   **Generative AI Integration:** Server-side services to interact with AI provider APIs (OpenAI, Gemini, Mistral). API keys managed via environment variables.
-   **Theming (Dark/Light Modes):** Client-side implementation using CSS variables and Svelte stores to manage theme state.
-   **Deployment:**
    -   GCP Cloud Run: Containerize server and client (or serve client from server).
    -   Vercel: Optimal for SvelteKit client. Server might be deployed as serverless functions or a separate service.
    -   Supabase: Could be an alternative for database and auth if project pivots.

This document will be updated as the project evolves.

## 7. Authentication Flow (Local Provider - Email/Password)

This section details the flow for user registration, login, session management using JWTs, and accessing protected resources for the local email/password authentication provider.

### 7.1. Core Principles & Security Considerations

-   **Password Hashing:** Passwords will never be stored in plaintext. A strong, salted, and adaptive hashing algorithm like Argon2 (preferred) or bcrypt will be used on the server-side.
-   **Input Validation:** All user inputs (email, password) will be validated on both client and server-side.
    -   Email: Valid email format. Uniqueness enforced by database constraint and checked during registration.
    -   Password: Minimum length (e.g., 12 characters), complexity (e.g., mix of uppercase, lowercase, numbers, symbols - configurable).
-   **HTTPS:** All communication between client and server must be over HTTPS in production.
-   **JWT Security:**
    -   Signed with a strong, secret key (`JWT_SECRET` environment variable) using a secure algorithm (e.g., HS256, HS512, or RS256 if asymmetric keys are introduced later).
    -   Short-lived access tokens (e.g., 15-60 minutes).
    -   Consider implementing refresh tokens for longer sessions if required, stored securely (e.g., HttpOnly cookie). For the initial implementation, we will start with access tokens only.
    -   Include essential, non-sensitive claims (e.g., `sub` for user ID, `exp` for expiration, `iat` for issued at). Avoid storing sensitive user data in the JWT payload.
-   **Rate Limiting:** Implement rate limiting on authentication endpoints (`/register`, `/login`) to protect against brute-force attacks.
-   **Error Handling:** Generic error messages for failed login attempts to avoid disclosing whether an email exists or not. Specific error messages for registration where appropriate (e.g., "Email already taken").

### 7.2. Registration Flow

1.  **Client (UI):** User navigates to the registration page and enters their email and password. Client-side validation provides immediate feedback on format/complexity.
2.  **Client (API Call):** On submit, the client sends a `POST` request to `/api/auth/register` with the payload:
    ```json
    {
      "email": "user@example.com",
      "password": "Str0ngP@sswOrd!"
    }
    ```
3.  **Server (Endpoint `POST /api/auth/register`):**
    a.  **Validation:** Validates email format and password strength according to defined rules. If invalid, returns a `400 Bad Request` with error details.
    b.  **Check Email Uniqueness:** Queries the database to check if the email already exists. If it exists, returns a `409 Conflict` (or a `400` to obscure existence, TBD).
    c.  **Hash Password:** Hashes the provided password using Argon2 (or bcrypt).
    d.  **Create User:** Stores the new user record in the `users` table (e.g., `id`, `email`, `hashed_password`, `created_at`, `updated_at`).
    e.  **Response:**
        *   On success: Returns a `201 Created` response, possibly with the user's ID and email (excluding password).
            ```json
            {
              "id": "uuid-goes-here",
              "email": "user@example.com"
            }
            ```
        *   On failure (e.g., database error): Returns a `500 Internal Server Error`.

### 7.3. Login Flow

1.  **Client (UI):** User navigates to the login page and enters their email and password.
2.  **Client (API Call):** On submit, the client sends a `POST` request to `/api/auth/login` with the payload:
    ```json
    {
      "email": "user@example.com",
      "password": "Str0ngP@sswOrd!"
    }
    ```
3.  **Server (Endpoint `POST /api/auth/login`):**
    a.  **Validation:** Validates email format and that password is provided. If invalid, returns a `400 Bad Request`.
    b.  **Fetch User:** Retrieves the user from the database by email. If no user is found, returns a `401 Unauthorized` (generic message).
    c.  **Verify Password:** Compares the provided password with the stored hashed password using the hashing algorithm's verify function. If verification fails, returns a `401 Unauthorized` (generic message).
    d.  **Generate JWT:** If password verification is successful, generates a JWT (Access Token).
        *   **Payload example:** `{ "sub": "user-uuid", "exp": <timestamp>, "iat": <timestamp> }`
        *   Signed with `JWT_SECRET`.
    e.  **Response:** Returns a `200 OK` with the JWT.
        ```json
        {
          "accessToken": "your.jwt.here"
        }
        ```
        (Alternatively, the JWT could be sent in an HttpOnly cookie for better XSS protection if not using `localStorage` on client).

### 7.4. Session Management & Accessing Protected Resources

1.  **Client (Storage):** On successful login, the client stores the `accessToken` securely (e.g., in an Svelte store that persists to `localStorage` or `sessionStorage`).
2.  **Client (API Call to Protected Endpoint):** When accessing a protected resource (e.g., `/api/users/me`), the client includes the `accessToken` in the `Authorization` header with the `Bearer` scheme:
    ```
    Authorization: Bearer your.jwt.here
    ```
3.  **Server (Middleware):**
    a.  An authentication middleware intercepts requests to protected endpoints.
    b.  It extracts the JWT from the `Authorization` header.
    c.  It validates the JWT:
        *   Checks signature using `JWT_SECRET`.
        *   Checks for expiration (`exp` claim).
        *   Checks `iat` (issued at) / `nbf` (not before) claims if used.
    d.  If valid, it may extract the user ID (`sub` claim) and attach user information to the request context for use by the handler.
    e.  If invalid (missing, malformed, expired, invalid signature), the middleware returns a `401 Unauthorized`.
4.  **Server (Protected Endpoint Handler - e.g., `GET /api/users/me`):**
    a.  If the middleware passes, the handler executes.
    b.  It can access the authenticated user's information (e.g., user ID) from the request context.
    c.  Fetches necessary data (e.g., user profile from database, excluding sensitive fields like `hashed_password`).
    d.  **Response:** Returns a `200 OK` with the requested data.
        ```json
        {
          "id": "user-uuid",
          "email": "user@example.com",
          "createdAt": "timestamp"
          // other non-sensitive user details
        }
        ```

### 7.5. Logout

1.  **Client (Action):** User clicks a logout button.
2.  **Client (State Update):**
    a.  Remove the stored `accessToken` from client-side storage (and associated Svelte stores).
    b.  Redirect user to the login page or homepage.
3.  **Server (Consideration - Token Invalidation):**
    *   With JWTs, true server-side invalidation is complex as JWTs are stateless.
    *   **Short-lived tokens** are the primary defense.
    *   For higher security needs (if required later):
        *   Maintain a token blocklist (e.g., in Redis or database). Middleware would check this list. This adds state back.
        *   Use opaque session tokens stored in DB and HttpOnly cookies, moving away from pure JWT statelessness for sessions.
    *   For this initial implementation, client-side removal is sufficient given short token expiry.

### 7.6. API Endpoints Summary

-   `POST /api/auth/register`: User registration.
-   `POST /api/auth/login`: User login, returns JWT.
-   `GET /api/users/me`: (Protected) Get current authenticated user's profile.
-   (Future) `POST /api/auth/refresh`: (If refresh tokens are implemented) Get a new access token.
-   (Future) `POST /api/auth/logout`: (If server-side token invalidation is implemented) Invalidate current token/session.

### 7.7. Client-Side State Management (Svelte Stores)

-   `authStore`:
    -   `user: User | null` (User object, null if not authenticated)
    -   `accessToken: string | null`
    -   `isAuthenticated: boolean` (derived from user/token)
    -   `isLoading: boolean` (for async auth operations)
    -   `error: string | null` (for auth errors)
-   Actions/functions to interact with API service: `login()`, `register()`, `logout()`, `fetchUser()`.
-   Store should persist `accessToken` to `localStorage` (or `sessionStorage`) and rehydrate on app load to maintain session across page refreshes.

This detailed flow should provide a good basis for implementation.
