set dotenv-load := true
set positional-arguments := true

# List available recipes
default:
    @just --list

# --- Environment & Config ---
# Check that required ENV variables are present
check-env:
    #!/usr/bin/env bash
    if [ -z "${DATABASE_URL:-}" ]; then
        echo "Error: DATABASE_URL is not set. Ensure .envrc is sourced or .env exists.";
        exit 1;
    fi
    if [ -z "${JWT_SECRET:-}" ]; then
        echo "Error: JWT_SECRET is not set. Ensure .envrc is sourced or .env exists.";
        exit 1;
    fi
    if [ -z "${DATABASE_PROVIDER:-}" ]; then
        echo "Warning: DATABASE_PROVIDER not set, defaulting to sqlite";
        export DATABASE_PROVIDER="sqlite";
    fi
    echo "✅ Environment variables checked. Using ${DATABASE_PROVIDER:-sqlite} database."

# --- Database (dbmate) ---
# db-clean: Completely cleans all database assets, cache, and state
# Usage: just db-clean
db-clean:
    @echo "🧹 Cleaning all database assets and cache..."
    @echo "Removing SQLite database files..."
    @rm -f server/db/*.sqlite3* || true
    @echo "Removing SQLX query cache..."
    @rm -rf server/.sqlx || true
    @rm -rf .sqlx || true
    @echo "Removing schema.sql..."
    @rm -f server/db/schema.sql || true
    @echo "✅ Database cleaning complete."

# db-setup: Sets up DB by applying all migrations and preparing SQLX cache.
# Usage: just db-setup
db-setup: check-env
    @echo "🏗️  Setting up database from scratch..."
    @echo "Database provider: ${DATABASE_PROVIDER:-sqlite}"
    @echo "Database URL: ${DATABASE_URL}"
    @echo "Ensuring database directory exists..."
    @mkdir -p server/db
    @echo "Applying database migrations using dbmate..."
    cd server && dbmate up
    @echo "Generating schema.sql from current database state..."
    cd server && dbmate dump
    @echo "Preparing SQLx query cache with fresh database (including tests)..."
    cd server && SQLX_OFFLINE=false cargo sqlx prepare --workspace -- --all-targets
    @echo "✅ Database setup complete with dbmate migrations applied and cache prepared."

# db-reset: Cleans everything and rebuilds from scratch
# Usage: just db-reset
db-reset: db-clean db-setup
    @echo "✅ Database completely reset and rebuilt."

# db-migrate: Apply all pending database migrations using dbmate.
# Usage: just db-migrate
db-migrate: check-env
    @echo "Ensuring database directory exists..."
    @mkdir -p server/db
    @echo "Applying pending database migrations using dbmate..."
    cd server && dbmate up
    @echo "Updating schema.sql using dbmate..."
    cd server && dbmate dump
    @echo "✅ Database migrations applied using dbmate."

# db-rollback: Rollback the last database migration using dbmate.
# Usage: just db-rollback
db-rollback: check-env
    @echo "Rolling back last database migration using dbmate..."
    cd server && dbmate down
    @echo "Updating schema.sql after rollback using dbmate..."
    cd server && dbmate dump
    @echo "✅ Last migration rolled back using dbmate."

# db-prepare-cache: Regenerate SQLX query cache only
# Usage: just db-prepare-cache
db-prepare-cache: check-env
    @echo "Regenerating SQLX query cache..."
    @echo "Database provider: ${DATABASE_PROVIDER:-sqlite}"
    @echo "Removing existing SQLX cache..."
    @rm -rf server/.sqlx || true
    @rm -rf .sqlx || true
    @echo "Ensuring database is up to date using dbmate..."
    cd server && dbmate up
    @echo "Preparing new SQLX query cache (including tests)..."
    cd server && SQLX_OFFLINE=false cargo sqlx prepare --workspace -- --all-targets
    @echo "✅ SQLX query cache regenerated."

# db-new-migration <name>: Create a new database migration file using dbmate.
# Usage: just db-new-migration my_new_migration
db-new-migration name:
    @echo "Creating new migration file using dbmate: {{name}}..."
    cd server && dbmate new {{name}}
    @echo "✅ New migration file created using dbmate: server/db/migrations/*_{{name}}.sql"

# db-status: Show database migration status using dbmate
# Usage: just db-status
db-status: check-env
    @echo "📊 Database migration status (using dbmate):"
    cd server && dbmate status
    @echo "📊 SQLX cache status:"
    @if [ -d "server/.sqlx" ]; then echo "✅ SQLX cache exists ($(find server/.sqlx -name '*.json' | wc -l) cached queries)"; else echo "❌ SQLX cache missing"; fi

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
# test-server [pattern]: Runs Rust server tests.
# Usage: just test-server
#        just test-server specific_module_or_test_name
test-server pattern="":
    @echo "Running Rust server tests..."
    @if [ -z "{{pattern}}" ]; then echo "Running all server tests (cargo test)..."; else echo "Running specific server tests matching '{{pattern}}' (cargo test {{pattern}})..."; fi
    @if [ -z "{{pattern}}" ]; then cd server && RUST_LOG=${RUST_LOG:-info} SQLX_OFFLINE=true cargo test -- --nocapture --test-threads=1; else cd server && RUST_LOG=${RUST_LOG:-info} SQLX_OFFLINE=true cargo test "{{pattern}}" -- --nocapture --test-threads=1; fi
    @echo "✅ Server tests complete."

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
    @echo "Running client E2E tests (Playwright via 'cd client && bun playwright test')..."
    @if [ -z "{{pattern}}" ]; then echo "Running all Playwright tests..."; else echo "Running specific Playwright tests matching '{{pattern}}'..."; fi
    @if [ -z "{{pattern}}" ]; then cd client && bun playwright test; else cd client && bun playwright test --grep "{{pattern}}"; fi
    @echo "✅ Client E2E tests complete."

# test-integration [pattern]: Runs integration tests (server tests that require database).
# Usage: just test-integration
#        just test-integration oauth_integration_tests
test-integration pattern="":
    @echo "Running integration tests..."
    @if [ -z "{{pattern}}" ]; then echo "Running all integration tests..."; else echo "Running specific integration tests matching '{{pattern}}'..."; fi
    @if [ -z "{{pattern}}" ]; then cd server && RUST_LOG=${RUST_LOG:-info} SQLX_OFFLINE=true cargo test --test "*integration*" -- --nocapture --test-threads=1; else cd server && RUST_LOG=${RUST_LOG:-info} SQLX_OFFLINE=true cargo test --test "{{pattern}}" -- --nocapture --test-threads=1; fi
    @echo "✅ Integration tests complete."

# test [server_pattern] [client_pattern] [e2e_pattern]: Runs all tests.
# Patterns are optional. If a pattern is not provided, all tests for that category run.
# Usage: just test
#        just test auth_tests # runs auth_tests server, auth_tests client, all e2e
#        just test "" "" login_flow # runs all server, all client, login_flow e2e
test server_pattern="" client_pattern="" e2e_pattern="":
    @echo "Running all tests: server, client (unit/integration), and E2E..."
    just test-server "{{server_pattern}}"
    just test-client "{{client_pattern}}"
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
check-server: format-server
    @echo "Checking server code: cargo fmt --check, cargo clippy, cargo check..."
    cd server && cargo fmt --check
    cd server && SQLX_OFFLINE=true cargo clippy --all-targets --all-features -- -D warnings -D clippy::pedantic
    cd server && SQLX_OFFLINE=true cargo check # Final compilation check
    @echo "✅ All server checks complete."

# check-client: Runs client-side checks (format, lint, strict type check).
# As per CLAUDE.md: `cd client && bun run lint && bun run check:strict`
# `bun run lint` often includes Prettier format check.
check-client: format-client
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
clean: clean-server clean-client db-clean
    @echo "Cleaning project-level temporary files (logs, .DS_Store)..."
    rm -rf logs
    find . -name ".DS_Store" -delete -print
    @echo "✅ All project artifacts, dependencies, temp files, and database cleaned."

# --- Initial Project Setup ---
# setup-client: Installs client dependencies AFTER cleaning.
# Usage: just setup-client
setup-client: clean-client
    @echo "Setting up client: installing dependencies (cd client && bun install)..."
    cd client && bun install && bunx playwright install
    @echo "✅ Client dependencies installed."

# setup-server-deps: Fetches server dependencies only
# Usage: just setup-server-deps
setup-server-deps: clean-server
    @echo "Setting up server: fetching dependencies..."
    cd server && cargo fetch
    @echo "✅ Server dependencies fetched."

# setup-server: Builds server (assumes dependencies and database are ready)
# Usage: just setup-server
setup-server:
    @echo "Building server with SQLX cache..."
    cd server && SQLX_OFFLINE=true cargo build
    @echo "✅ Server built successfully."

# setup: Installs all client and server dependencies after cleaning. Sets up database.
# This is the main setup command a user should run first for a fresh environment.
# Usage: just setup
setup: check-env setup-client setup-server-deps db-reset setup-server
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
