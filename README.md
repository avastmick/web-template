# Web Template

## Overview

The web-template is a project that will enable the quick-start for a high-performance web application. The project goals are:

-   **Performance:** the project must meet or exceed the very highest performance expectations in all cases
-   **Secure:** the project must be provably secure, using the latest cryptographic techniques and best practices in all cases
-   **High quality:** the code must be provably high-quality, with all static analysis and code quality check turned to 11!
-   **Lightweight:** we need to ensure that the application uses the smallest possible memory and storage footprint, both on the server and in the browser
-   **Easy to use:** usage of web application, the developer experience, and operation of the application should be first class
-   **Beautiful:** the web application must be first class in look and feel, engaging users, causing them to ooh and aah in use.
-   **Functional:** the web application in particular must be able to leverage modern web application tools, extensions and widgets easily, to make development fast and easy.

This foundation is built using a SvelteKit frontend and a Rust backend, further detailed in the Architecture section.

## Specific Goals (from PRD.md)

To achieve the above, the project will deliver on the following specific requirements:

-   Beautiful web application with easy CSS-only changes for themes (dark/light modes, configurable color-schemes).
-   Fast, modular Rust server supporting:
    -   Database access (SQLite initially, using `sqlx`).
    -   Easy integration of server-side components.
    -   Generative AI integration (configurable for various providers like OpenAI, Gemini, Mistral).
-   Highly secure user registration, authentication, and profile management.
    -   Initial auth providers: Local (email/password) and Google OAuth.
-   Payment integration using Stripe.
-   Deployment targets: GCP Cloud Run, Vercel, Supabase.

## Architecture

The project is structured into two main components:

-   **`client/`**: A SvelteKit application responsible for the user interface and client-side logic. It is written in TypeScript and uses Bun for package management.
-   **`server/`**: A Rust application using the Axum framework for the backend REST API. It handles business logic, database interaction (via `sqlx`), and integrations with external services. Cargo is used for package management.

### Key Technologies:

-   **Frontend (Client):**
    -   Svelte / SvelteKit
    -   TypeScript
    -   Bun (Package Manager, Bundler, Test Runner)
    -   Vite (Build Tool)
    -   Playwright (E2E Testing)
    -   Prettier (Formatting)
    -   ESLint (Linting)
-   **Backend (Server):**
    -   Rust
    -   Axum (Web Framework)
    -   Tokio (Async Runtime)
    -   SQLx (Database Interaction)
    -   Cargo (Package Manager & Build Tool)
    -   `clippy` (Linting)
-   **Database:**
    -   SQLite (for local development, configurable for production)
    -   `dbmate` (for database migrations)
-   **Tooling & Orchestration:**
    -   `just` (Command runner for managing project tasks)
    -   `overmind` (or similar, for running multiple processes locally, e.g., client and server dev servers)
    -   `direnv` (for managing environment variables via `.envrc` - which is gitignored)
    -   Git pre-commit hooks (for automated quality checks)

A more detailed description of the architecture, including data flow, authentication mechanisms, and component interactions, can be found in `documentation/ARCHITECTURE.md`.

## Getting Started

1.  **Prerequisites:**
    *   Install Rust: [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)
    *   Install Bun: [https://bun.sh/docs/installation](https://bun.sh/docs/installation)
    *   Install `just`: [https://github.com/casey/just#installation](https://github.com/casey/just#installation)
    *   Install `dbmate`: [https://github.com/amacneil/dbmate#installation](https://github.com/amacneil/dbmate#installation)
    *   Install `direnv`: [https://direnv.net/docs/installation.html](https://direnv.net/docs/installation.html) and hook it into your shell.
    *   Install `pre-commit`: [https://pre-commit.com/#install](https://pre-commit.com/#install)

2.  **Clone the repository.**
    ```bash
    git clone <repository-url> # Replace <repository-url> with the actual URL
    cd <repository-name>/web-template # Replace <repository-name>
    ```
3.  **Setup Environment:**
    *   If it doesn't exist, copy `web-template/.envrc.example` to `web-template/.envrc`.
        ```bash
        cp .envrc.example .envrc
        ```
    *   Fill in the required environment variables in `.envrc`:
        *   `DATABASE_URL`: SQLite database connection string (e.g., `sqlite:./db/dev.sqlite3?mode=rwc`)
        *   `JWT_SECRET`: A secure 32+ character secret key for JWT token signing
        *   `GOOGLE_CLIENT_ID`: Your Google OAuth client ID from Google Console
        *   `GOOGLE_CLIENT_SECRET`: Your Google OAuth client secret from Google Console
        *   `SERVER_URL`: Your server URL (default: `http://localhost:8081`)
    *   Run `direnv allow` in the project root (`web-template/`) to load the environment variables.
4.  **Install Pre-commit Hooks:**
    *   Run `pre-commit install` to set up the git hooks defined in `.pre-commit-config.yaml`. This ensures code quality checks are run before each commit.
5.  **Initial Project Setup (Clean Install & Build):**
    *   Run `just setup`. This command cleans previous build artifacts and dependencies, then installs fresh dependencies for both client and server, and performs an initial build.
6.  **Database Setup:**
    *   Run `just db-setup` to apply database migrations using `dbmate`. This will create the necessary tables in your database.
7.  **OAuth Configuration (Required for Authentication):**
    *   Create a Google Cloud Project at [Google Cloud Console](https://console.cloud.google.com/)
    *   Enable the Google OAuth 2.0 API
    *   Create OAuth 2.0 credentials:
        *   Go to "Credentials" → "Create Credentials" → "OAuth client ID"
        *   Choose "Web application"
        *   Add authorized redirect URIs: `http://localhost:8081/api/auth/oauth/google/callback` (development)
        *   Note the Client ID and Client Secret for your `.envrc` file
    *   **Note:** This application uses an invite-only system. Users must be invited before they can register via OAuth or email/password.
8.  **Run the application (Development):**
    *   Run `just dev`. This will start both the client and server development servers using Overmind (ensure `Procfile.dev` is configured in `web-template/`).
    *   Client is typically available at `http://localhost:5173` (or as configured by Vite/SvelteKit).
    *   Server is typically available at `http://localhost:3000` (or as configured in `server/.env` if applicable, or `Rocket.toml` for Rocket).

## Project Management

All common development tasks are managed via the `justfile` located in the `web-template` directory. Run `just` in the terminal from this directory to see a list of available commands.

Key command categories include:
*   `setup`: For initial project setup (cleans, installs all dependencies, and builds). (`just setup`)
*   `dev`: For running development servers using Overmind. (`just dev`). For individual servers: `just client-dev-server` or `just server-dev-server [--hotreload]`.
*   `build`: For building client and server for production. (`just build`, `just build-client`, `just build-server`)
*   `check`: For running linters, type checkers, and formatters (in check mode). (`just check`, `just check-client`, `just check-server`)
*   `format`: For auto-formatting code. (`just format`, `just format-client`, `just format-server`)
*   `test`: For running unit, integration, and e2e tests. (`just test [server_pattern] [client_pattern] [e2e_pattern]`, `just server-test [pattern]`, `just test-client [pattern]`, `just test-e2e [pattern]`)
*   `db-*`: For database migration tasks using `dbmate`. (`just db-setup`, `just db-migrate`, `just db-rollback`, `just db-new-migration <name>`)
*   `clean`: For cleaning build artifacts, dependencies, and temporary files. (`just clean`, `just clean-client`, `just clean-server`)

Refer to `CLAUDE.md` for detailed guidelines on development practices, code style, and contributing to this project.

### Code Quality and Standards

This project enforces high code quality through several mechanisms:

*   **Pre-commit Hooks:** Configured in `web-template/.pre-commit-config.yaml`, these hooks automatically run on every `git commit` attempt. They perform checks like:
    *   Secrets detection (`gitleaks`).
    *   Code formatting (Prettier for client, `cargo fmt` for server).
    *   Linting (ESLint for client, `cargo clippy -D warnings -D clippy::pedantic` for server).
    *   Type checking for Svelte/TypeScript.
    *   Ensuring lockfiles (`Cargo.lock`, `bun.lockb`) are up-to-date and consistent with `Cargo.toml`/`package.json`.
    If any hook fails, the commit will be aborted, allowing you to fix the issues. You must run `pre-commit install` once in the repository (from the `web-template` directory or the repo root if configured project-wide) to enable these hooks.

*   **Manual Quality Checks:** Use the `just check` command to run all linters, formatters (in check mode), and type checkers for both client and server.
    *   `just check-client`: Runs all checks for the SvelteKit client.
    *   `just check-server`: Runs all checks for the Rust server.
    It's good practice to run these before pushing changes, even with pre-commit hooks enabled.

*   **Continuous Integration (CI):** (To be configured) CI pipelines will eventually run all checks (`just check`), builds (`just build`), and tests (`just test`) automatically on pull requests and merges to the main branch to ensure ongoing quality and stability.
