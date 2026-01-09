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

# --- Database (sqlx) ---
# db-clean: Completely cleans all database assets, cache, and state
# Usage: just db-clean
db-clean:
    @echo "🧹 Cleaning all database assets and cache..."
    @echo "Removing SQLite database files..."
    @rm -f server/db/*.sqlite3* || true
    @echo "Removing SQLX query cache..."
    @rm -rf server/.sqlx || true
    @rm -rf .sqlx || true
    @echo "✅ Database cleaning complete."

# db-setup: Sets up DB by applying all migrations and preparing SQLX cache.
# Usage: just db-setup
db-setup: check-env
    @echo "🏗️  Setting up database from scratch..."
    @echo "Database provider: ${DATABASE_PROVIDER:-sqlite}"
    @echo "Database URL: ${DATABASE_URL}"
    @echo "Ensuring database directory exists..."
    @mkdir -p server/db
    @echo "Creating database if it doesn't exist..."
    cd server && sqlx database create || true
    @echo "Applying database migrations using sqlx..."
    cd server && sqlx migrate run
    @echo "Preparing SQLx query cache with fresh database (including tests)..."
    cd server && SQLX_OFFLINE=false cargo sqlx prepare --workspace -- --all-targets
    @echo "✅ Database setup complete with sqlx migrations applied and cache prepared."

# db-reset: Cleans everything and rebuilds from scratch
# Usage: just db-reset
db-reset: db-clean db-setup
    @echo "✅ Database completely reset and rebuilt."

# db-migrate: Apply all pending database migrations using sqlx.
# Usage: just db-migrate
db-migrate: check-env
    @echo "Ensuring database directory exists..."
    @mkdir -p server/db
    @echo "Applying pending database migrations using sqlx..."
    cd server && sqlx migrate run
    @echo "✅ Database migrations applied using sqlx."

# db-rollback: Rollback the last database migration using sqlx.
# Usage: just db-rollback
db-rollback: check-env
    @echo "Rolling back last database migration using sqlx..."
    cd server && sqlx migrate revert
    @echo "✅ Last migration rolled back using sqlx."

# db-prepare-cache: Regenerate SQLX query cache only
# Usage: just db-prepare-cache
db-prepare-cache: check-env
    @echo "Regenerating SQLX query cache..."
    @echo "Database provider: ${DATABASE_PROVIDER:-sqlite}"
    @echo "Removing existing SQLX cache..."
    @rm -rf server/.sqlx || true
    @rm -rf .sqlx || true
    @echo "Ensuring database is up to date using sqlx..."
    cd server && sqlx migrate run
    @echo "Preparing new SQLX query cache (including tests)..."
    cd server && SQLX_OFFLINE=false cargo sqlx prepare --workspace -- --all-targets
    @echo "✅ SQLX query cache regenerated."

# db-new-migration <name>: Create a new database migration file using sqlx.
# Usage: just db-new-migration my_new_migration
db-new-migration name:
    @echo "Creating new migration file using sqlx: {{name}}..."
    cd server && sqlx migrate add -r {{name}}
    @echo "✅ New migration files created using sqlx: server/migrations/*_{{name}}.up.sql and .down.sql"

# db-status: Show database migration status using sqlx
# Usage: just db-status
db-status: check-env
    @echo "📊 Database migration status (using sqlx):"
    cd server && sqlx migrate info
    @echo "📊 SQLX cache status:"
    @if [ -d "server/.sqlx" ]; then echo "✅ SQLX cache exists ($(find server/.sqlx -name '*.json' | wc -l) cached queries)"; else echo "❌ SQLX cache missing"; fi

# db-dump: Show current database schema
# Usage: just db-dump
db-dump: check-env
    @echo "📄 Current database schema:"
    @echo "Note: sqlx does not have a built-in schema dump. Use SQLite CLI for schema inspection."
    cd server && sqlite3 "${DATABASE_URL#sqlite:}" ".schema" || echo "Error: Could not dump schema"

# --- Development Servers (User Managed) ---
# These commands are intended for the user to run directly.
# Claude should not run `just dev` or related long-running server commands.

# client-dev-server: Starts the SvelteKit client development server.
# Usage: just client-dev
#        just client-dev --log  (enables logging to file)
client-dev *args: check-env
    #!/usr/bin/env bash
    echo "Starting client development server (SvelteKit)..."
    echo "Client will run on port ${CLIENT_PORT:-8080} and connect to server at ${SERVER_URL:-http://localhost:8081}"

    # Parse arguments
    log=false
    for arg in {{args}}; do
        case "$arg" in
            "--log") log=true ;;
        esac
    done

    # Setup logging if requested
    if [ "$log" = true ]; then
        mkdir -p logs
        current_date=$(date +"%Y-%m-%d_%H-%M-%S")
        log_file="logs/client_${current_date}.log"
        echo "📝 Client logs will be written to: ${log_file}"
        ln -sf "client_${current_date}.log" logs/client_latest.log
    fi

    # Execute with or without logging
    if [ "$log" = true ]; then
        (cd client && bun run dev) 2>&1 | tee -a "${log_file}"
    else
        cd client && bun run dev
    fi

# server-dev-server: Starts the Rust/Axum server development server.
# Usage: just server-dev
#        just server-dev --hotreload  (enables cargo watch for hot-reloading)
#        just server-dev --log        (enables logging to file)
#        just server-dev --hotreload --log
server-dev *args: check-env
    #!/usr/bin/env bash
    echo "Starting server development server (Rust/Axum)..."

    # Parse arguments
    hotreload=false
    log=false
    for arg in {{args}}; do
        case "$arg" in
            "--hotreload") hotreload=true ;;
            "--log") log=true ;;
        esac
    done

    # Setup logging if requested
    if [ "$log" = true ]; then
        mkdir -p logs
        current_date=$(date +"%Y-%m-%d_%H-%M-%S")
        log_file="logs/server_${current_date}.log"
        echo "📝 Server logs will be written to: ${log_file}"
        ln -sf "server_${current_date}.log" logs/server_latest.log
    fi

    # Check cargo-watch installation if hotreload is requested
    if [ "$hotreload" = true ]; then
        if ! command -v cargo-watch &> /dev/null; then
            echo "⚠️  cargo-watch not found. Installing..."
            cargo install cargo-watch
        fi
    fi

    # Build command
    cmd="cd server && SQLX_OFFLINE=true RUST_LOG=${RUST_LOG:-info}"
    if [ "$hotreload" = true ]; then
        echo "🔥 Hot reloading ENABLED. Using 'cargo watch'."
        cmd="$cmd cargo watch -q -c -w src -x run"
    else
        echo "⚡ Hot reloading DISABLED. Using 'cargo run'."
        cmd="$cmd cargo run"
    fi

    # Execute with or without logging
    if [ "$log" = true ]; then
        eval "$cmd" 2>&1 | tee -a "${log_file}"
    else
        eval "$cmd"
    fi

# dev: Starts all development servers using Overmind (requires Procfile).
# Usage: just dev                          (default: hotreload and logging enabled)
#        OVERMIND_HOTRELOAD=false just dev (disable hot-reloading)
#        OVERMIND_LOGGING=false just dev   (disable logging)
#        OVERMIND_HOTRELOAD=false OVERMIND_LOGGING=false just dev (disable both)
dev: check-env
    #!/usr/bin/env bash
    echo "Starting all development servers via Overmind..."

    # Default to true if not explicitly set to false
    hotreload="${OVERMIND_HOTRELOAD:-true}"
    logging="${OVERMIND_LOGGING:-true}"

    # Check if cargo-watch is installed when hotreload is enabled
    if [ "$hotreload" != "false" ]; then
        echo "🔥 Hot reloading ENABLED for server (cargo watch)"
        if ! command -v cargo-watch &> /dev/null; then
            echo "⚠️  cargo-watch not found. Installing..."
            cargo install cargo-watch
        fi
        export OVERMIND_HOTRELOAD=true
    else
        echo "⚡ Hot reloading DISABLED"
        unset OVERMIND_HOTRELOAD
    fi

    if [ "$logging" != "false" ]; then
        echo "📝 Logging ENABLED (check logs/ directory)"
        mkdir -p logs
        export OVERMIND_LOGGING=true
    else
        echo "📵 Logging DISABLED"
        unset OVERMIND_LOGGING
    fi

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

# logs: View the latest server and client logs
# Usage: just logs          (shows both)
#        just logs server   (shows server logs)
#        just logs client   (shows client logs)
logs type="":
    #!/usr/bin/env bash
    if [ ! -d "logs" ]; then
        echo "No logs directory found. Run servers with --log flag or OVERMIND_LOGGING=true"
        exit 1
    fi

    case "{{type}}" in
        "server")
            if [ -f "logs/server_latest.log" ]; then
                echo "📄 Viewing latest server logs (logs/server_latest.log):"
                tail -f logs/server_latest.log
            else
                echo "No server logs found"
            fi
            ;;
        "client")
            if [ -f "logs/client_latest.log" ]; then
                echo "📄 Viewing latest client logs (logs/client_latest.log):"
                tail -f logs/client_latest.log
            else
                echo "No client logs found"
            fi
            ;;
        *)
            echo "📄 Available log files:"
            ls -la logs/
            echo ""
            echo "Use 'just logs server' or 'just logs client' to tail specific logs"
            ;;
    esac

# --- Testing ---
# test-server [pattern]: Runs Rust server tests.
# Usage: just test-server
#        just test-server specific_module_or_test_name
test-server pattern="":
    @echo "Running Rust server tests..."
    @if [ -z "{{pattern}}" ]; then echo "Running all server tests (cargo test)..."; else echo "Running specific server tests matching '{{pattern}}' (cargo test {{pattern}})..."; fi
    @if [ -z "{{pattern}}" ]; then cd server && SQLX_OFFLINE=true cargo test; else cd server && SQLX_OFFLINE=true cargo test "{{pattern}}"; fi
    @echo "✅ Server tests complete."

test-server-verbose pattern="":
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

# test-integration [pattern] [--verbose] [--real-ai]: Runs integration tests (server tests that require database).
# Usage: just test-integration
#        just test-integration integration_tests
#        just test-integration endpoint_tests
#        just test-integration auth_tests --verbose           (enables AI test verbose mode)
#        just test-integration ai_tests --verbose --real-ai   (uses real OpenRouter API key)
test-integration pattern="" *flags="":
    #!/usr/bin/env bash
    echo "Running integration tests..."

    # Check for flags
    test_env=""
    for flag in {{flags}}; do
        case "$flag" in
            "--verbose")
                test_env="$test_env AI_TEST_VERBOSE=true"
                echo "🔍 Verbose mode enabled for AI integration tests"
                ;;
            "--real-ai")
                if [ -n "${OPENROUTER_API_KEY:-}" ]; then
                    test_env="$test_env OPENROUTER_API_KEY_REAL=${OPENROUTER_API_KEY}"
                    echo "🤖 Using real OpenRouter API key for AI testing"
                else
                    echo "⚠️  --real-ai flag requires OPENROUTER_API_KEY environment variable"
                    echo "💡 Export your OpenRouter API key: export OPENROUTER_API_KEY=your_key_here"
                fi
                ;;
        esac
    done

    if [ -z "{{pattern}}" ]; then
        echo "Running all integration and endpoint tests..."
        cd server && eval "$test_env RUST_LOG=${RUST_LOG:-info} SQLX_OFFLINE=true cargo test --test integration_tests --test endpoint_tests -- --nocapture --test-threads=1"
    else
        echo "Running specific integration tests matching '{{pattern}}'..."
        cd server && eval "$test_env RUST_LOG=${RUST_LOG:-info} SQLX_OFFLINE=true cargo test --test '{{pattern}}' -- --nocapture --test-threads=1"
    fi
    echo "✅ Integration tests complete."

# test-coverage: Run tests with coverage reporting using cargo-tarpaulin
# Usage: just test-coverage
test-coverage:
    @echo "Running tests with coverage reporting..."
    cd server && SQLX_OFFLINE=true cargo tarpaulin --out Lcov --all-features --workspace --timeout 120 --exclude-files "**/migrations/*" --exclude-files "**/tests/*" --ignore-panics --ignore-tests
    @echo "✅ Coverage report generated."

# test-coverage-html: Generate HTML coverage report and open in browser
# Usage: just test-coverage-html
test-coverage-html:
    @echo "Generating HTML coverage report..."
    cd server && SQLX_OFFLINE=true cargo tarpaulin --out Html --all-features --workspace --timeout 120 --exclude-files "**/migrations/*" --exclude-files "**/tests/*" --ignore-panics --ignore-tests
    @echo "Opening coverage report in browser..."
    @open server/tarpaulin-report.html 2>/dev/null || xdg-open server/tarpaulin-report.html 2>/dev/null || echo "Please open server/tarpaulin-report.html manually"
    @echo "✅ HTML coverage report generated."

# test-coverage-validate: Run both server and client coverage with 95% threshold gates
# Usage: just test-coverage-validate
# Environment: Requires OPENROUTER_API_KEY and AI_DEFAULT_MODEL for server AI tests
test-coverage-validate:
    #!/usr/bin/env bash
    set -e
    echo "🎯 Running coverage validation with 95% threshold..."

    # Server coverage with tarpaulin
    echo ""
    echo "📊 Running server coverage..."
    cd server
    SQLX_OFFLINE=true cargo tarpaulin --out Json --output-dir . --all-features --workspace --timeout 300 \
        --exclude-files "**/migrations/*" --exclude-files "**/tests/*" --ignore-panics --ignore-tests 2>&1 || true

    # Parse server coverage
    if [ -f "tarpaulin-report.json" ]; then
        server_coverage=$(jq '.files | map(.covered / .coverable * 100) | add / length' tarpaulin-report.json 2>/dev/null || echo "0")
        echo "Server coverage: ${server_coverage}%"
        if (( $(echo "$server_coverage < 95" | bc -l) )); then
            echo "❌ Server coverage ${server_coverage}% is below 95% threshold"
            server_passed=false
        else
            echo "✅ Server coverage ${server_coverage}% meets 95% threshold"
            server_passed=true
        fi
    else
        echo "⚠️  Could not parse server coverage report"
        server_passed=false
    fi
    cd ..

    # Client coverage with vitest
    echo ""
    echo "📊 Running client coverage..."
    cd client
    bun run test:unit:coverage 2>&1 || true

    # Parse client coverage from vitest json output
    if [ -f "coverage/coverage-summary.json" ]; then
        client_coverage=$(jq '.total.lines.pct' coverage/coverage-summary.json 2>/dev/null || echo "0")
        echo "Client coverage: ${client_coverage}%"
        if (( $(echo "$client_coverage < 95" | bc -l) )); then
            echo "❌ Client coverage ${client_coverage}% is below 95% threshold"
            client_passed=false
        else
            echo "✅ Client coverage ${client_coverage}% meets 95% threshold"
            client_passed=true
        fi
    else
        echo "⚠️  Could not parse client coverage report"
        client_passed=false
    fi
    cd ..

    # Summary
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "📋 Coverage Validation Summary"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    if [ "$server_passed" = true ] && [ "$client_passed" = true ]; then
        echo "✅ All coverage thresholds met!"
        exit 0
    else
        echo "❌ Coverage validation failed"
        [ "$server_passed" = false ] && echo "   - Server: below 95%"
        [ "$client_passed" = false ] && echo "   - Client: below 95%"
        exit 1
    fi

# test-mutations: Run mutation testing with cargo-mutants (server) and Stryker (client)
# Usage: just test-mutations
#        just test-mutations server  (server only)
#        just test-mutations client  (client only)
# Note: Mutation testing is slow - expect 10-30+ minutes depending on codebase size
test-mutations target="all":
    #!/usr/bin/env bash
    set -e
    echo "🧬 Running mutation testing..."

    run_server_mutations() {
        echo ""
        echo "🦀 Running cargo-mutants on server..."
        if ! command -v cargo-mutants &> /dev/null; then
            echo "⚠️  cargo-mutants not found. Installing..."
            cargo install cargo-mutants
        fi
        cd server
        # Run with timeout and limited mutants for CI efficiency
        SQLX_OFFLINE=true cargo mutants --timeout 60 --jobs 4 2>&1 || {
            echo "⚠️  Some mutants survived or mutation testing encountered errors"
            echo "   Check mutants.out/ directory for details"
        }
        if [ -d "mutants.out" ]; then
            echo ""
            echo "📊 Server mutation testing results:"
            cat mutants.out/outcomes.json 2>/dev/null | jq -r '.outcomes | group_by(.outcome) | map({outcome: .[0].outcome, count: length}) | .[]' 2>/dev/null || echo "   See mutants.out/ for detailed results"
        fi
        cd ..
    }

    run_client_mutations() {
        echo ""
        echo "⚡ Running Stryker on client..."
        cd client
        if [ ! -f "stryker.conf.json" ]; then
            echo "❌ Stryker config not found (stryker.conf.json)"
            echo "   Run: bun add -d @stryker-mutator/core @stryker-mutator/typescript-checker @stryker-mutator/vitest-runner"
            exit 1
        fi
        bunx stryker run 2>&1 || {
            echo "⚠️  Some mutants survived or Stryker encountered errors"
            echo "   Check reports/mutation/ directory for details"
        }
        cd ..
    }

    case "{{target}}" in
        "server")
            run_server_mutations
            ;;
        "client")
            run_client_mutations
            ;;
        "all"|*)
            run_server_mutations
            run_client_mutations
            ;;
    esac

    echo ""
    echo "✅ Mutation testing complete."

# quality-check: Full quality validation pipeline (format, lint, test, coverage)
# Usage: just quality-check
#        just quality-check --skip-mutations  (skip slow mutation tests)
# This is the comprehensive quality gate for CI/CD pipelines
quality-check *flags="":
    #!/usr/bin/env bash
    set -e
    echo "🔍 Running full quality validation pipeline..."
    echo ""

    # Parse flags
    skip_mutations=false
    for flag in {{flags}}; do
        case "$flag" in
            "--skip-mutations") skip_mutations=true ;;
        esac
    done

    # Track failures
    failures=""

    # Step 1: Format and lint checks
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "📝 Step 1/4: Code quality checks (format + lint)"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    just check || failures="$failures check"

    # Step 2: Run all tests
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "🧪 Step 2/4: Running all tests"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    just test || failures="$failures tests"

    # Step 3: Coverage validation
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "📊 Step 3/4: Coverage validation (95% threshold)"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    just test-coverage-validate || failures="$failures coverage"

    # Step 4: Mutation testing (optional)
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "🧬 Step 4/4: Mutation testing"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    if [ "$skip_mutations" = true ]; then
        echo "⏭️  Skipping mutation tests (--skip-mutations flag)"
    else
        just test-mutations || failures="$failures mutations"
    fi

    # Summary
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "📋 Quality Check Summary"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    if [ -z "$failures" ]; then
        echo "✅ All quality checks passed!"
        exit 0
    else
        echo "❌ Quality check failed. Failed steps:$failures"
        exit 1
    fi

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
# As per CLAUDE.md: `cd server && cargo fmt --check && cargo clippy -- -D warnings`
# We add `cargo check` for completeness.
check-server: format-server
    @echo "Checking server code: cargo fmt --check, cargo clippy, cargo check..."
    cd server && cargo fmt --check
    cd server && SQLX_OFFLINE=true cargo clippy --all-targets --all-features -- -D warnings
    @echo "Checking for overlong files (>600 lines)..."
    @find server/src -name "*.rs" -exec bash -c 'lines=$(wc -l < "{}"); if [ "$lines" -gt 600 ]; then echo "❌ ERROR: {} has $lines lines (max: 600)"; exit 1; fi' \; || (echo "❌ Some server files exceed 600 lines. Please refactor them into smaller modules." && exit 1)
    @echo "✅ All server checks complete."

# check-client: Runs client-side checks (format, lint, strict type check).
# As per CLAUDE.md: `cd client && bun run lint && bun run check:strict`
# `bun run lint` often includes Prettier format check.
check-client: format-client
    @echo "Checking client code: format (prettier --check), lint (eslint), type-check (svelte-check/tsc)..."
    cd client && bun run lint # Typically includes Prettier check
    cd client && bun run check:strict # Assumes 'check:strict\' is 'svelte-check --fail-on-warnings\'
    @echo "Checking for overlong files (>600 lines)..."
    @find client/src -name "*.ts" -o -name "*.js" -o -name "*.svelte" -o -name "*.vue" -exec bash -c 'lines=$(wc -l < "{}"); if [ "$lines" -gt 600 ]; then echo "❌ ERROR: {} has $lines lines (max: 600)"; exit 1; fi' \; || (echo "❌ Some client files exceed 600 lines. Please refactor them into smaller modules." && exit 1)
    @echo "✅ All client checks complete."

# check-template: Runs template generate checks (fmt --check, clippy, cargo check).
#  `cd scripts/create-web-template && cargo fmt --check && cargo clippy -- -D warnings`
check-template:
    @echo "Checking template generation code: cargo fmt --check, cargo clippy, cargo check..."
    cd scripts/create-web-template && cargo fmt --all
    cd scripts/create-web-template && cargo clippy --all-targets --all-features -- -D warnings
    @echo "Checking for overlong files (>600 lines)..."
    @find scripts/create-web-template/src -name "*.rs" -exec bash -c 'lines=$(wc -l < "{}"); if [ "$lines" -gt 600 ]; then echo "❌ ERROR: {} has $lines lines (max: 600)"; exit 1; fi' \; || (echo "❌ Some server files exceed 600 lines. Please refactor them into smaller modules." && exit 1)
    @echo "✅ All server checks complete."

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

# clean-template: Removes template build artifacts ('template/target').
# Usage: just clean-server
clean-template:
    @echo "Cleaning template build artifacts (cd scripts/create-web-template && cargo clean)..."
    cd scripts/create-web-template && cargo clean
    @echo "✅ Create template artifacts cleaned."

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
