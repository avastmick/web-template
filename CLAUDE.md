# WEB-TEMPLATE CODEBASE GUIDE

## IMPORTANT: THIS IS A NEW PROJECT - NO LEGACY CODE

**This is a brand new project build. There is NO need for backwards compatibility. Keep the codebase clean:**
- Delete ALL unused code immediately
- Do NOT maintain legacy interfaces or compatibility layers
- Remove old implementations when creating new ones
- Keep only the current, active implementation of any feature
- No "backward compatibility" comments or re-exports
- If refactoring, DELETE the old code, don't keep it around

## IMPORTANT: SESSION START REQUIREMENTS

**ALWAYS BEGIN EACH SESSION BY READING THE FOLLOWING DOCUMENTATION:**
1. `documentation/PRD.md` - Product Requirements Document (defines project goals and features)
2. `documentation/ARCHITECTURE.md` - System architecture and design decisions
3. `documentation/UI_GUIDELINES.md` - The current UI guidelines for the client web application
4. `documentation/SECURITY.md` - The security of the system
5. `README.md` - Project overview and setup instructions

**DO NOT SKIP THIS STEP.** These documents contain critical context about the project's goals, architecture, current state, and development standards. Reading them ensures you understand the project requirements and can make informed decisions aligned with the project's vision.

**WHEN ADDING TESTS:**
If you are asked to add integration or endpoint tests, you MUST also read:
- `server/tests/README.md` - Overview of the three-tier testing strategy
- `server/tests/integration/README.md` - For integration tests (service-level, no HTTP)
- `server/tests/endpoint/README.md` - For endpoint tests (HTTP requests, MUST verify database state)
ENSURE you align to the current means of creating to tests FULLY aligning to the existing practice.

**IMPORTANT** Remember that 'Kanbain' is a working title; take care not to name things in the application using the working title as it is likely to change.


## CRITICAL: Server-First MVP Development

**DO NOT IMPLEMENT ANY CLIENT FUNCTIONALITY UNTIL SERVER MVP v1.0.0 IS COMPLETE**

**ALWAYS** Run `just check-server` frequently during the server development. Go NO MORE than FIVE changes to source files before re-running.

The development approach for this project follows a strict server-first methodology:

1. **Server MVP (v1.0.0) Requirements:**
   - ALL API endpoints fully implemented and tested
   - 80%+ code coverage with comprehensive unit tests
   - 100% of API endpoints have integration tests (NO mocks, real server with clean database)
   - ALL server checks pass (`just check-server`)
   - ALL server tests pass (`just test-server`)
   - Full AI integration (issue creation assistant, duplicate detection, sizing)
   - Complete authentication system (local + OAuth)
   - All database migrations finalized
   - Security measures fully implemented
   - Performance targets met (<100ms response times)

2. **No Client Work Until Server MVP Complete:**
   - Do NOT work on web client (`client/`) until server v1.0.0 is released
   - Do NOT work on CLI until server v1.0.0 is released
   - Focus exclusively on server functionality, testing, and quality

3. **Server MVP Verification:**
   - Server can be tested entirely via API calls (curl, httpie, Postman)
   - All Kanban board functionality works via API
   - AI features are fully functional via API
   - Authentication flows work via API

This approach ensures the server is rock-solid, fully tested, and meets all requirements before any client implementation begins.

## Overview

This project aims to create a high-performance, secure, and high-quality web application. There are two main components:

-   `client/`: A Svelte 5 SPA (Single Page Application) with client-side routing only (NO SSR). Built with SvelteKit configured for static adapter with SPA fallback.
-   `server/`: A Rust/Axum application providing REST API endpoints and interacting with a database (`sqlx`).

**IMPORTANT**: The client is a pure SPA with CSR (Client-Side Rendering) only. There is NO Server-Side Rendering (SSR) in this application. In production, the client is served as static files from the server.

The database is SQLite for local development, with `sqlx-cli` for migrations. `just` is used for ALL command running, and `direnv` manages environment variables through `.envrc`.

## Development

**IMPORTANT RULES - ALWAYS STRICTLY FOLLOW**

- ALWAYS work in small increments on a minimum set of files, in one area at a time - i.e. either `server` OR `client` NEVER work on both in the same changeset.
- ALWAYS run `just check-server` or `just check-client` after each increment; resolve ALL issues before proceeding.
- DO NOT make many changes without running the checks as it will waste time.
- NEVER add linter exclusions (such as `#[allow(clippy::SOME_EXCLUSION)]` or `eslint-disable`) to any code without explicit reason. Add agreement comment if overridden and date of agreement.
- DO NOT add `#[allow(dead_code)]` or equivalent exclusions; all code MUST be used.
- ALWAYS check local logs; If a `.overmind.sock` is present then the server is running. Check the logs in the `logs` dir to check issues; there are `logs/client_latest.log` and `logs/server_latest.log` for client and server
- ALWAYS use `playwright` mcp to review browser console log
- ALWAYS explicitly follow code organisation conventions listed in `documentation/ARCHITECTURE.md`. Update `ARCHITECTURE.md` if any additional code directories are added.

## CRITICAL: Architecture and Usage Patterns MUST Be Followed

**ALWAYS respect and follow the existing architecture patterns:**

### Adding New Services
When adding a new service (e.g., `AiSessionService`):
1. Create the service in `server/src/services/` following existing patterns
2. Add the service to `AppState` in `server/src/core/state.rs`
3. Initialize the service in `create_router()` in `server/src/routes.rs`
4. Access the service through `state.service_name` in handlers

**Example Pattern (from existing code):**
```rust
// In server/src/core/state.rs
pub struct AppState {
    pub board: Arc<BoardService>,  // Existing service
    pub ai_session: Arc<AiSessionService>,  // New service
}

// In server/src/routes.rs
let board_service = BoardService::new(db_pool.clone());  // Existing
let ai_session_service = AiSessionService::new(db_pool.clone());  // New

// In handlers
let board = state.board.get_user_board(&user_id).await?;  // Usage
```

### Adding New Routes
When adding new API endpoints:
1. Create handlers in `server/src/handlers/` following module organization
2. Import handlers in `server/src/routes.rs`
3. Add routes to appropriate route group function (e.g., `ai_routes()`, `board_issue_routes()`)
4. Use existing authentication patterns (e.g., `AuthUser`, `JwtAuth`)

**NEVER modify core infrastructure without understanding the patterns first!**

## Key Project Goals (from PRD.md and README.md)

-   **MINIMAL CODE:** The least code to achieve the outcome.
-   **Performance:** Exceed highest performance expectations.
-   **Security:** Provably secure with modern cryptography and best practices.
-   **High Quality:** Rigorous static analysis, linting, formatting, and type-checking.
-   **Lightweight:** Minimal memory/storage footprint.
-   **Ease of Use:** First-class UX, DX, and operational experience.
-   **Beautiful & Functional UI:** Engaging, easily extensible UI with dark/light modes and themes.
-   **Modular Backend:** Fast, extensible Rust server for database, AI, payments, etc.

## Code Quality Standards

**IMPORTANT** ALWAYS adhere to the following in ALL cases:

-   Adhere to general quality standards: Accuracy, Correctness, Efficiency, Maintainability, Readability, Security.
-   **DRY Principle:** Extract common code into shared utilities/services. Avoid duplicating logic.
-   **Simplicity:** Avoid deep nesting, complex recursion. Refactor for clarity.
-   **Testing:** Include tests for algorithms, complex calculations, business logic, API endpoints.
    -   **Code Coverage:** Maintain 80%+ code coverage for all server code. Use `cargo tarpaulin` to measure coverage.
    -   **Integration Testing:** All API endpoints MUST have comprehensive integration tests that run against a real server instance (NO MOCKS).
-   **Single Responsibility:** Functions should be concise, handle a single task. Files should group related functionality.
-   **File Size:** Files MUST be under 600 lines. The `just check-server` and `just check-client` commands enforce this limit. Refactor larger files into smaller, logically grouped modules.
-   **Commenting:** Add clear, concise comments for complex logic. Rust functions, structs, enums, and modules should have doc comments (`///` or `//!`). Files should have a header comment summarizing contents and usage if not obvious from structure.
-   **TypeScript - No Non-Null Assertions (`!`)**:
    -   Handle potential absence of data explicitly using checks, optional chaining (`?.`), or type guards.
-   **Error Handling:** Use `Result<T, E>` comprehensively in Rust. Handle errors gracefully and provide meaningful context. Leverage `thiserror` and `anyhow` crates as appropriate.

## Package Management

This project uses Bun (client) and Cargo (server) *exclusively*.

**IMPORTANT:** Always use the appropriate package manager commands (`cargo add`, `bun add`) NEVER manually edit `Cargo.toml` or `package.json`. Manual edits result in outdated versions and can cause dependency conflicts.

### Client - Bun (`web-template/client/`)

-   Install package: `bun add <package-name>` (User runs this command)
-   Install dev dependency: `bun add -d <package-name>` (User runs this command)
-   Remove package: `bun remove <package-name>`
-   Update dependencies: `bun update`
-   List dependencies: `bun pm ls`
-   Check for unused dependencies: `bun pm ls --prod=false` (More accurately: `bun run depcheck` or similar if configured)
-   Clean unused packages: `bun pm prune` (Be cautious with this, verify before running)
-   Build: `just build-client`
-   Check: `just check-client` includes lints, file size and other checks
-   Test: `just test-client`
-   Format: `just format-client`

### Server - Cargo (`web-template/server/`)

-   Add dependency: `cargo add <crate-name>`
-   Add dependency (with features): `cargo add <crate-name> --features <feature-name>`
-   Add dev dependency: `cargo add <crate-name> --dev`
-   Remove dependency: `cargo remove <crate-name>` (User runs this, check `Cargo.toml` afterwards)
-   Update dependencies: `cargo update`
-   Build: `just build-server`
-   Format: `just format-server`
-   Check: `just check-server` includes lints, filesize and other checks
-   Test: `just test-server`

## Project Commands (using `just` from `web-template/`)

The `justfile` in the project root (`web-template/`) provides a unified interface for common tasks. Always use `just <command>` where possible. Run `just` to see all available commands.

### Common `just` Commands (Examples - will be defined in `justfile`):

*   **Setup & Installation:**
    *   `just setup`: Installs all client and server dependencies after cleaning. Sets up `.envrc` if needed.
    *   `just setup-client`: Cleans client, then installs client dependencies (`cd client && bun install`).
    *   `just setup-server`: Cleans server, then prepares server (e.g., `cd server && cargo build` to fetch dependencies).
*   **Development:**
    *   `just dev`: Starts client and server development servers (e.g., using `overmind` and `Procfile.dev`). (Claude should not run this; for user execution).
    *   `just client-dev-server`: Starts only the client development server.
    *   `just server-dev-server [--hotreload]`: Starts only the server development server, optionally with hot-reloading.
*   **Building:**
    *   `just build`: Builds both client and server for production.
    *   `just build-client`: Builds the client application (`cd client && bun run build`).
    *   `just build-server`: Builds the server application (`cd server && cargo build --release`).
*   **Quality Checks & Formatting:**
    *   `just check`: Runs all linters, formatters (check mode), and type checkers for both client and server.
    *   `just check-client`: Runs client-side checks (e.g., `cd client && bun run lint && bun run check:strict`).
    *   `just check-server`: Runs server-side checks (`cd server && cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings -D clippy::pedantic && cargo check`).
    *   `just format`: Formats code for both client and server.
    *   `just format-client`: Formats client code (`cd client && bun run format`).
    *   `just format-server`: Formats server code (`cd server && cargo fmt`).
*   **Testing:**
    *   `just test [server_pattern] [client_pattern] [e2e_pattern]`: Runs all tests. Patterns are optional.
    *   `just test-client [pattern]`: Runs client-side unit/integration tests (`cd client && bun run test`).
    *   `just test-server [pattern]`: Runs server-side tests (`cd server && cargo test`).
    *   `just test-e2e [pattern]`: Runs end-to-end tests (`cd client && bun playwright test`).
    *   `just test-coverage`: Runs server tests with coverage reporting using cargo-tarpaulin.
    *   `just test-coverage-html`: Generates HTML coverage report and opens in browser.
    *   `just test-integration`: Runs only integration tests against real server.
*   **Database (using `sqlx`):**
    *   `just db-setup`: Sets up the database by applying all pending migrations (`sqlx migrate run`).
    *   `just db-migrate`: Applies pending database migrations (`sqlx migrate run`).
    *   `just db-rollback`: Rolls back the last database migration (`sqlx migrate revert`).
    *   `just db-new-migration <name>`: Creates new migration files (`sqlx migrate add -r <name>`).
*   **Cleaning:**
    *   `just clean`: Removes all build artifacts, dependencies (`node_modules`, `target`), and temporary files.
    *   `just clean-client`: Cleans client artifacts and dependencies (`rm -rf client/node_modules client/.svelte-kit client/build client/bun.lockb client/.bun`).
    *   `just clean-server`: Cleans server artifacts (`rm -rf server/target`).

## Code Style Guidelines

-   **General:**
    -   Indentation: Tabs (not spaces).
    -   Quotes: Single quotes for TypeScript/JavaScript/CSS. Double quotes for Rust string literals.
    -   Line width: 140 characters max.
    -   No trailing commas (except where idiomatic, e.g., Rust multiline struct/enum, Svelte props).
    -   Prefer `async/await` over promise chains in TypeScript.
    -   Use descriptive error messages in `catch` blocks and Rust `Result::Err`.
    -   Import order: External libraries first, then internal modules, grouped logically.
-   **Svelte (Client):**
    -   Component filenames: `PascalCase.svelte`.
    -   Variables/functions: `camelCase`.
    -   **UI/UX:** All components MUST follow theming guidelines. See `documentation/UI_UX_THEME.md` for detailed requirements.
-   **Rust (Server):**
    -   Modules, crates, functions, variables: `snake_case`.
    -   Types (structs, enums, traits): `PascalCase`.
    -   Follow official Rust API Guidelines.

## TypeScript Strictness Settings (Client)

The `client/tsconfig.json` is configured for maximum strictness:
-   `strict: true` (enables all strict type checking options)
-   `noImplicitAny: true`
-   `strictNullChecks: true`
-   `noImplicitReturns: true`
-   `noUnusedLocals: true` & `noUnusedParameters: true`
-   `noUncheckedIndexedAccess: true`
-   `exactOptionalPropertyTypes: true`

Always handle nullability explicitly. Ensure all code paths are covered.

## Rust Strictness Settings (Server)

-   The `just check-server` command enforces `cargo clippy --all-targets --all-features -- -D warnings -D clippy::pedantic`. Address all lints reported.
-   Ensure `Cargo.toml` does not have wildcard dependencies. Pin versions.

## Environment Variables

-   All environment variables, including secrets, are managed in `web-template/.envrc` using `direnv`.
-   `.envrc` is in `.gitignore`. Use `web-template/.envrc.example` as a template.
-   Required ENV variables (examples): `DATABASE_URL`, `JWT_SECRET`.
-   Optional (for specific features): `OPENAI_API_KEY`, `GEMINI_API_KEY`, `MISTRAL_API_KEY`.
-   Use uppercase for all environment variable names (e.g., `DATABASE_URL`).

## Pre-commit Hooks (`.pre-commit-config.yaml`)

The project uses pre-commit hooks defined in `web-template/.pre-commit-config.yaml` to ensure code quality and consistency before anything is committed. These hooks automate:
-   **Secrets Detection:** Using `gitleaks` to prevent committing sensitive information.
-   **Code Formatting:**
    -   Client: Prettier (via local `bun run format` hook) for various client file types.
    -   Server: `cargo fmt` for Rust code.
-   **Linting:**
    -   Client: ESLint (via local `bun run lint` hook) for TypeScript and Svelte files.
    -   Server: `cargo clippy --all-targets --all-features -- -D warnings -D clippy::pedantic` for strict Rust linting.
-   **Type Checking:** `svelte-check --fail-on-warnings` (via local `bun run check:strict` hook) for the client.
-   **Lockfile Consistency:** Checks if `Cargo.lock` and `bun.lockb` are up-to-date with their respective manifest files (`Cargo.toml`, `package.json`).
-   **General Checks:** Trailing whitespace, end-of-file fixing, large file detection (max 1MB), valid JSON/YAML/TOML, merge conflict markers.

**To use pre-commit hooks:**
1.  Ensure `pre-commit` is installed (see `README.md`).
2.  Run `pre-commit install` once from the repository root (or `web-template/` if it's the git root) to install the hooks into your `.git/hooks` directory.
After installation, these checks will run automatically when you attempt to `git commit`. If any check fails, the commit will be aborted, and you'll need to fix the reported issues and re-stage the files before trying to commit again.

## Project Structure (High-Level)

**CRITICAL: NO CODE OR SQL FILES IN PROJECT ROOT**
The project root should NEVER contain:
- Migration files (.sql)
- Source code files (.rs, .ts, .js, .svelte)
- Build artifacts

ALL code and SQL files MUST be in their appropriate subdirectories as listed below:

-   `client/`: SvelteKit frontend.
    -   `src/lib/components/`: Reusable Svelte components.
    -   `src/lib/services/`: Client-side services (e.g., API calls).
    -   `src/lib/stores/`: Svelte stores for state management.
    -   `src/lib/utils/`: Utility functions.
    -   `src/routes/`: SvelteKit routes.
    -   `tests/`: Playwright E2E tests and Vitest unit/integration tests.
-   `server/`: Rust/Axum backend.
    -   `src/handlers/`: Axum request handlers.
    -   `src/models/` (or `src/domain/`): Data structures, business objects.
    -   `src/services/`: Business logic services.
    -   `src/db/` (or `src/repository/`): Database interaction logic (`sqlx`).
    -   `src/middleware/`: Axum middleware.
    -   `src/config.rs`: Application configuration.
    -   `src/errors.rs`: Custom error types.
    -   `src/main.rs`: Application entry point.
    -   `src/routes.rs`: API route definitions.
    -   `tests/`: Integration tests for handlers and services.
-   **`server/migrations/`**: Database migrations (managed by `sqlx`).
    -   **CRITICAL**: ALL migration files MUST be created here using `just db-new-migration <name>`
    -   Migration SQL files in format: `<timestamp>_<name>.up.sql` and `.down.sql`
    -   **NEVER create migrations in the project root `/migrations/` directory**
-   `documentation/`: Project documentation (`PRD.md`, `ARCHITECTURE.md`, etc.).
-   `justfile`: Command definitions for the `web-template` root.
-   `.envrc`: Environment variables (gitignored).
-   `.envrc.example`: Template for `.envrc`.
-   `.pre-commit-config.yaml`: Pre-commit hook definitions.


## Important Claude-Specific Workflow Notes

-   **Reading First:** Always try to read relevant files (`README.md`, `CLAUDE.md`, `CURRENT_TASKS.md`, specific source files) before suggesting changes or asking questions.
-   **Small, Incremental Changes:** Make small, focused changes.
-   **Verify After Each Change:** After each modification (especially code):
    1.  State the file(s) you modified.
    2.  Suggest running the appropriate formatting command, e.g., `just format-client` or `just format-server`. (Pre-commit hooks will also do this, but manual run is good practice).
    3.  Suggest running the comprehensive checks: `just check-client` or `just check-server` (or `just check` for both). These include format checks, linting, and type checking.
    4.  Suggest running `just build` (or `just build-client`/`just build-server`).
    5.  Suggest running relevant tests: `just test` or more specific variants like `just server-test [pattern]`, `just test-client [pattern]`, `just test-e2e [pattern]`.
    6.  If diagnostics are reported from these commands, attempt to fix them. If unsure after 1-2 tries, present the diagnostic clearly to the user.
-   **Committing:** Never attempt to commit code directly. You can suggest a `git commit --dry-run -m "Relevant commit message"` to verify what would be committed based on staged changes. The user handles actual commits. Pre-commit hooks will run automatically on actual commit attempts by the user.
-   **Running Servers:** Do not attempt to run `just dev` or any other command that starts a long-running server process. The user will handle this.
-   **Existing Code:** Do not modify existing, working code unless the task explicitly requires it. If you see a potential issue outside your current task, note it for the user to review and prioritize.
-   **Tool Usage:**
    -   When providing paths to tools, the path should always begin with `web-template/` if it's a project file.
    -   Use `grep` for finding symbols or code snippets, `find_path` for locating files by pattern.
    -   All paths provided to tools must be relative to the current working directory, which is assumed to be the root of the repository containing `web-template`.
-   **Asking for Clarification:** If requirements are unclear, or you're unsure how to proceed, ask specific clarifying questions.
-   **Code Block Formatting:** Adhere strictly to the `path/to/Something.blah#L123-456` format for all code blocks. Use `/dev/null/example.ext` for generic examples not tied to project files.
-   **Adherence to `CLAUDE.md`:** You are expected to follow all guidelines in this document.
