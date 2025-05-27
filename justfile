#!/usr/bin/env just --justfile

set dotenv-load := true

# List available recipes
default:
    @just --list

# Check that required ENV variables are present
check-env:

# Check database connection
check-db: check-env

# Run database migrations
db-migrate: check-db


# Start client development server with log rotation
client-dev: check-env
    #!/usr/bin/env bash
    echo "Starting client development server with logs..."
    current_date=$(date +"%Y-%m-%d")
    log_file="logs/client_dev_server_${current_date}.log"
    echo "Client logs will be written to ${log_file}"
    echo "Client will run on port ${CLIENT_PORT}, connecting to server at ${SERVER_URL}"
    mkdir -p logs
    # Create a symbolic link to the current log file for easier access
    ln -sf "${log_file}" client_dev_server.log
    # Run the NextJS client application
    (cd  client && bun run dev) | tee -a "${log_file}"

# Start server development server with log rotation
server-dev hotreload: check-env
    #!/usr/bin/env bash
    echo "Starting server development server with logs..."
    current_date=$(date +"%Y-%m-%d")
    log_file="logs/server_dev_${current_date}.log"
    echo "Server logs will be written to ${log_file}"
    mkdir -p logs
    # Create a symbolic link to the current log file for easier access
    ln -sf "${log_file}" server_dev.log
    # Run the Rust server
    if [ "{{hotreload}}" = "true" ]; then
        echo "Running server using cargo watch for hot reloading"
        cd server && RUST_LOG=debug cargo watch -x run -w src -w tests 2>&1 | tee -a "../${log_file}"
    else
        cd server && RUST_LOG=debug cargo run 2>&1 | tee -a "../${log_file}"
    fi

# Start all development servers (client and server)
dev hotreload: check-env
    #!/usr/bin/env bash
    echo "Starting all development servers..."
    echo "Use 'just server-dev' or 'just client-dev' for individual servers"
    echo "Starting with hotreload = {{hotreload}}"

    if [ "{{hotreload}}" = "true" ]; then
        echo "Running with hot reload enabled"
        export OVERMIND_HOT_RELOAD=true
    else
        export OVERMIND_HOT_RELOAD=false
    fi

    # Run all services with overmind using the Procfile
    clear && overmind s


# Run server tests, optionally with specific test pattern
server-test test_pattern="":
    #!/usr/bin/env bash
    echo "Running Rust server tests..."
    cd server
    if [ -z "{{test_pattern}}" ]; then
        echo "Running all server tests..."
        RUST_LOG=debug cargo test -- --nocapture
    else
        echo "Running specific tests matching: {{test_pattern}}"
        RUST_LOG=debug cargo test {{test_pattern}} -- --nocapture
    fi

# Run client tests, optionally with specific test pattern
client-test test_pattern="":
    #!/usr/bin/env bash
    echo "Running client tests..."
    cd client
    if [ -z "{{test_pattern}}" ]; then
        echo "Running all client tests..."
        npx playwright test
    else
        echo "Running specific tests matching: {{test_pattern}}"
        npx playwright test --grep "{{test_pattern}}"
    fi

# Run all tests, optionally with specific test pattern
test test_pattern="":
    #!/usr/bin/env bash
    echo "Running all tests..."
    if [ -z "{{test_pattern}}" ]; then
        just server-test
        just client-test
    else
        echo "Running specific tests matching: {{test_pattern}}"
        just server-test "{{test_pattern}}"
        just client-test "{{test_pattern}}"
    fi
    echo "All tests completed"

# Run server linting and checks
server-check:
    echo "Running Rust server linting and checks..."
    cd server && cargo check && cargo clippy --all-targets --all-features -- -D warnings

# Run client linting and type checks
client-check:
    echo "Running client linting and type checking..."
    cd test_client && bun run lint && bun run check

# Run all linting and type checks
check: server-check client-check
    echo "All linting and type checking completed"

# Format server code
server-format:
    echo "Formatting Rust server code..."
    cd server && cargo fmt --all

# Format client code
client-format:
    echo "Formatting client code..."
    cd client && bun run format

# Format all code
format: server-format client-format
    echo "All code formatting completed"

# Clean server workspace
server-clean:
    echo "Cleaning Rust server workspace..."
    cd server && cargo clean && echo "✅ Server workspace cleaned"

# Clean client workspace
client-clean:
    echo "Cleaning client workspace..."
    cd client && \
    rm -rf node_modules .next .turbo coverage .swc out && \
    echo "✅ Client workspace cleaned"

# Clean all workspaces
clean: server-clean client-clean 
    echo "Cleaning project logs and temporary files..."
    rm -rf logs/*.log
    find . -name ".DS_Store" -delete
    find . -name "*.log" -delete
    echo "✅ All workspaces and project files cleaned"

# Setup fresh server workspace
server-setup: server-clean
    echo "Setting up fresh Rust server workspace..."
    cd server && cargo fetch

# Setup fresh client workspace
client-setup: client-clean
    echo "Setting up fresh client workspace..."
    cd client && bun install

# Setup all workspaces
setup: server-setup client-setup
    echo "All workspaces setup complete"


