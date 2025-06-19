set dotenv-load := true
set positional-arguments := true

# List available recipes
default:
    @just --list

# --- Environment & Config ---
# Check that required ENV variables are present
check-env:
    @if [ -z "${DATABASE_URL}" ]; then echo "Error: DATABASE_URL is not set. Ensure .envrc is sourced or .env exists."; exit 1; fi
    @if [ -z "${JWT_SECRET}" ]; then echo "Error: JWT_SECRET is not set. Ensure .envrc is sourced or .env exists."; exit 1; fi
    @echo "✅ Environment variables checked."

# --- Database (dbmate) ---
# db-setup: Alias for db-migrate. Sets up DB by applying all migrations.
# Usage: just db-setup
db-setup: db-migrate

# db-migrate: Apply all pending database migrations.
# Usage: just db-migrate
db-migrate: check-env
    @echo "Ensuring database directory exists..."
    @mkdir -p server/db
    @echo "Applying pending database migrations (dbmate up)..."
    cd server && dbmate up
    @echo "✅ Database migrations applied."

# db-rollback: Rollback the last database migration.
# Usage: just db-rollback
db-rollback: check-env
    @echo "Rolling back last database migration (dbmate down)..."
    cd server && dbmate down
    @echo "✅ Last migration rolled back."

# db-new-migration <name>: Create a new database migration file.
# Usage: just db-new-migration my_new_migration
db-new-migration name:
    @echo "Creating new migration file: {{name}}..."
    cd server && dbmate new {{name}}
    @echo "✅ New migration file created: server/db/migrations/*_{{name}}.sql"

# --- Development Servers (User Managed) ---
# These commands are intended for the user to run directly.
# Claude should not run `just dev` or related long-running server commands.

# client-dev-server: Starts the SvelteKit client development server.
# Usage: just client-dev-server
client-dev: check-env
    @echo "Starting client development server (SvelteKit)..."
    echo "Client will run on port ${CLIENT_PORT:-8080} and connect to server at ${SERVER_URL:-http://localhost:8081}"
    cd client && bun run dev

# server-dev-server: Starts the Rust/Axum server development server.
# Usage: just server-dev-server
#        just server-dev-server --hotreload  (enables cargo watch for hot-reloading)
server-dev +hotreload: check-env
    #!/usr/bin/env bash
    echo "Starting server development server (Rust/Axum)..."
    if [[ "{{hotreload}}" == "true" ]]; then
        echo "Hot reloading ENABLED. Using 'cargo watch'."
        cd server && SQLX_OFFLINE=true RUST_LOG=${RUST_LOG:-info} cargo watch -q -c -w src -x run
    else
        echo "Hot reloading DISABLED. Using 'cargo run'."
        cd server && SQLX_OFFLINE=true RUST_LOG=${RUST_LOG:-info} cargo run
    fi

# dev: Starts all development servers using Overmind (requires Procfile.dev).
# Usage: just dev
dev: check-env
    @echo "Starting all development servers via Overmind..."
    @echo "Hot reloading should be configured within the Procfile commands (e.g., using 'cargo watch' for server)."
    overmind s

# refresh: Completely refreshes the workspace - cleans everything, sets up fresh, and starts dev servers
# Usage: just refresh
refresh: clean setup dev
    @echo "✅ Workspace refreshed and development servers started."

# --- Building for Production ---
# build-client: Builds the client application for production.
# Usage: just build-client
build-client:
    @echo "Building client for production (cd client && bun run build)..."
    cd client && bun run build
    @echo "✅ Client production build complete."

# build-server: Builds the server application for production.
# Usage: just build-server
build-server:
    @echo "Building server for production (cd server && cargo build --release)..."
    cd server && SQLX_OFFLINE=true cargo build --release
    @echo "✅ Server production build complete."

# build: Builds both client and server for production.
# Usage: just build
build: build-client build-server
    @echo "✅ All production builds complete."

# docker-build: Builds the Docker image for production.
# Usage: just docker-build
docker-build:
    @echo "Building Docker image for production..."
    docker build -t web-template .
    @echo "✅ Docker image built successfully."

# docker-run: Runs the Docker container using local environment variables.
# Usage: just docker-run
docker-run: check-env
    @echo "Running Docker container with local environment variables..."
    ./scripts/run-docker.sh

# --- Testing ---
# server-test [pattern]: Runs Rust server tests.
# Usage: just server-test
#        just server-test specific_module_or_test_name
server-test pattern="":
    #!/usr/bin/env bash
    echo "Running Rust server tests..."
    cd server
    if [ -z "{{pattern}}" ]; then
        echo "Running all server tests (cargo test)..."
        RUST_LOG=${RUST_LOG:-info} SQLX_OFFLINE=true cargo test -- --nocapture
    else
        echo "Running specific server tests matching '{{pattern}}' (cargo test {{pattern}})..."
        RUST_LOG=${RUST_LOG:-info} SQLX_OFFLINE=true cargo test "{{pattern}}" -- --nocapture
    fi
    echo "✅ Server tests complete."

# test-client [pattern]: Runs client-side unit and integration tests (e.g., Vitest).
# Usage: just test-client
#        just test-client specific_component_or_test_name
test-client pattern="": # Corresponds to `just test-client` in CLAUDE.md
    @echo "Running client unit/integration tests (e.g., Vitest via 'cd client && bun run test')..."
    cd client && bun run test {{pattern}} # Assumes `bun run test` executes Vitest/Jest & forwards pattern
    @echo "✅ Client unit/integration tests complete."

# test-e2e [pattern]: Runs end-to-end tests (Playwright).
# Usage: just test-e2e
#        just test-e2e specific_flow_or_test_name
test-e2e pattern="": # Corresponds to `just test-e2e` in CLAUDE.md
    #!/usr/bin/env bash
    @echo "Running client E2E tests (Playwright via 'cd client && bun playwright test')..."
    cd client
    if [ -z "{{pattern}}" ]; then
        echo "Running all Playwright tests..."
        bun playwright test
    else
        echo "Running specific Playwright tests matching '{{pattern}}'..."
        bun playwright test --grep "{{pattern}}"
    fi
    @echo "✅ Client E2E tests complete."

# test [server_pattern] [client_pattern] [e2e_pattern]: Runs all tests.
# Patterns are optional. If a pattern is not provided, all tests for that category run.
# Usage: just test
#        just test auth_tests # runs auth_tests server, auth_tests client, all e2e
#        just test "" "" login_flow # runs all server, all client, login_flow e2e
test server_pattern="" client_pattern="" e2e_pattern="":
    @echo "Running all tests: server, client (unit/integration), and E2E..."
    just server-test "{{server_pattern}}"
    just test-client "{{client_pattern}}"
    just test-e2e "{{e2e_pattern}}"
    @echo "✅ All tests completed."

# --- Formatting ---
# format-server: Formats server code using 'cargo fmt'.
# Usage: just format-server
format-server:
    @echo "Formatting server code (cd server && cargo fmt --all)..."
    cd server && cargo fmt --all
    @echo "✅ Server code formatted."

# format-client: Formats client code (e.g., Prettier via 'bun run format').
# Usage: just format-client
format-client:
    @echo "Formatting client code (cd client && bun run format)..."
    cd client && bun run format # Assumes 'bun run format' is 'prettier --write .\'
    @echo "✅ Client code formatted."

# format: Formats code for both client and server.
# Usage: just format
format: format-server format-client
    @echo "✅ All code formatting complete."

# --- Quality Checks (Linters, Format Checkers, Type Checkers) ---
# check-server: Runs server-side checks (fmt --check, clippy, cargo check).
# As per CLAUDE.md: `cd server && cargo fmt --check && cargo clippy -- -D warnings -D clippy::pedantic`
# We add `cargo check` for completeness.
check-server:
    @echo "Checking server code: cargo fmt --check, cargo clippy, cargo check..."
    cd server && cargo fmt --check
    cd server && SQLX_OFFLINE=true cargo clippy --all-targets --all-features -- -D warnings -D clippy::pedantic
    cd server && SQLX_OFFLINE=true cargo check # Final compilation check
    @echo "✅ All server checks complete."

# check-client: Runs client-side checks (format, lint, strict type check).
# As per CLAUDE.md: `cd client && bun run lint && bun run check:strict`
# `bun run lint` often includes Prettier format check.
check-client:
    @echo "Checking client code: format (prettier --check), lint (eslint), type-check (svelte-check/tsc)..."
    cd client && bun run lint # Typically includes Prettier check
    cd client && bun run check:strict # Assumes 'check:strict\' is 'svelte-check --fail-on-warnings\'
    @echo "✅ All client checks complete."

# check: Runs all checks for both client and server.
# Usage: just check
check: check-server check-client
    @echo "✅ All project checks completed."

# --- Cleaning ---
# clean-server: Removes server build artifacts ('server/target').
# Usage: just clean-server
clean-server:
    @echo "Cleaning server build artifacts (cd server && cargo clean)..."
    cd server && cargo clean
    @echo "✅ Server artifacts cleaned."

# clean-client: Removes client build artifacts and dependencies.
# Usage: just clean-client
clean-client:
    @echo "Cleaning client artifacts (node_modules, .svelte-kit, build, coverage, bun.lockb)..."
    cd client && rm -rf node_modules .svelte-kit build coverage bun.lockb client/.bun
    @echo "✅ Client artifacts and dependencies cleaned."

# clean: Removes all build artifacts, dependencies, temporary files, and database from project.
# Usage: just clean
clean: clean-server clean-client
    @echo "Cleaning project-level temporary files (logs, .DS_Store)..."
    rm -rf logs
    find . -name ".DS_Store" -delete -print
    @echo "Cleaning database file..."
    @if [ -f "server/db/dev.sqlite3" ]; then rm -f "server/db/dev.sqlite3" && echo "Database file removed."; else echo "No database file found."; fi
    @echo "✅ All project artifacts, dependencies, temp files, and database cleaned."

# --- Initial Project Setup ---
# setup-client: Installs client dependencies AFTER cleaning.
# Usage: just setup-client
setup-client: clean-client
    @echo "Setting up client: installing dependencies (cd client && bun install)..."
    cd client && bun install && bunx playwright install
    @echo "✅ Client dependencies installed."

# setup-server: Prepares server (e.g., fetches dependencies and builds) AFTER cleaning.
# Usage: just setup-server
setup-server: clean-server
    @echo "Setting up server: fetching dependencies and initial build (cd server && cargo build)..."
    cd server && SQLX_OFFLINE=true cargo build
    @echo "✅ Server dependencies fetched and built."

# setup: Installs all client and server dependencies after cleaning. Sets up database.
# This is the main setup command a user should run first for a fresh environment.
# Usage: just setup
setup: check-env setup-client setup-server db-setup
    @echo "✅ Project setup complete. Dependencies installed and database initialized."

# --- Log Tailing Aliases (from original file, if direct log access needed) ---
# These are for convenience if not using Overmind\'s aggregated logs.
client-dev-logs: check-env
    #!/usr/bin/env bash
    echo "Starting client development server with direct log tailing..."
    current_date=$(date +"%Y-%m-%d")
    log_file="logs/client_dev_server_${current_date}.log"
    echo "Client logs will be written to ${log_file}"
    echo "Client will run on port ${CLIENT_PORT:-8080}, connecting to server at ${SERVER_URL:-http://localhost:8081}"
    mkdir -p logs
    ln -sf "${log_file}" client_dev_server.log # Symlink for easy access
    (cd client && bun run dev) 2>&1 | tee -a "${log_file}"

server-dev-logs hotreload: check-env
    #!/usr/bin/env bash
    echo "Starting server development server with direct log tailing..."
    current_date=$(date +"%Y-%m-%d")
    log_file="logs/server_dev_${current_date}.log"
    echo "Server logs will be written to ${log_file}"
    mkdir -p logs
    ln -sf "${log_file}" server_dev.log # Symlink for easy access
    if [ "{{hotreload}}" = "true" ]; then
        echo "Running server using cargo watch for hot reloading"
        (cd server && RUST_LOG=${RUST_LOG:-info} cargo watch -q -c -w src -x run) 2>&1 | tee -a "../${log_file}"
    else
        (cd server && RUST_LOG=${RUST_LOG:-info} cargo run) 2>&1 | tee -a "../${log_file}"
    fi
