#!/bin/bash
# Run cargo-mutants with required environment variables
# See .cargo/mutants.toml for configuration

set -e

# Required environment variables for SQLx offline mode and tests
export SQLX_OFFLINE=true
export DATABASE_URL="sqlite:./db/test.sqlite3?mode=rwc"
export DATABASE_PROVIDER=sqlite
export JWT_SECRET=test-secret-key-for-mutation-testing

# Regenerate SQLx offline metadata if needed
if [ "$1" = "--prepare" ]; then
    echo "Regenerating .sqlx metadata..."
    cargo sqlx prepare -- --tests
    shift
fi

# Run cargo-mutants with provided arguments
# Default: --in-place --check (compilation check only)
# Add --baseline=run to run actual tests (requires passing baseline)
echo "Running cargo-mutants..."
cargo mutants --in-place "${@:---check --no-shuffle}"
