#!/bin/bash

# Script to run the Docker container using local environment variables from direnv
# Usage: ./scripts/run-docker.sh

set -e

# Ensure we're in the project root
cd "$(dirname "$0")/.."

# Check if direnv is available and .envrc is sourced
if ! command -v direnv &> /dev/null; then
    echo "Error: direnv is not installed. Please install direnv first."
    exit 1
fi

if [[ -z "$JWT_SECRET" ]]; then
    echo "Error: Environment variables not loaded. Please ensure direnv is configured:"
    echo "  1. Run 'direnv allow' in the project root"
    echo "  2. Copy .envrc.example to .envrc and configure your secrets"
    exit 1
fi

# Create data directory if it doesn't exist
mkdir -p data

# Build the SERVER_URL and GOOGLE_REDIRECT_URI dynamically based on CLIENT_PORT
# In Docker, the server runs on port 8080 and is exposed to the CLIENT_PORT
SERVER_URL="http://localhost:${CLIENT_PORT:-8080}"
GOOGLE_REDIRECT_URI="${SERVER_URL}/api/auth/oauth/google/callback"

echo "Starting Docker container with local environment variables..."
echo "  Client accessible at: http://localhost:${CLIENT_PORT:-8080}"
echo "  Server URL (for OAuth): ${SERVER_URL}"
echo "  Google Redirect URI: ${GOOGLE_REDIRECT_URI}"
echo "  Using DATABASE_URL: sqlite:/data/production.sqlite3?mode=rwc"
echo ""

# Run the container with environment variables from direnv
docker run --rm -p "${CLIENT_PORT:-8080}:8080" \
  -v "$(pwd)/data:/data" \
  -e "RUST_LOG=${RUST_LOG:-warn,server=info}" \
  -e "JWT_SECRET=$JWT_SECRET" \
  -e "GOOGLE_CLIENT_ID=$GOOGLE_CLIENT_ID" \
  -e "GOOGLE_CLIENT_SECRET=$GOOGLE_CLIENT_SECRET" \
  -e "SERVER_URL=$SERVER_URL" \
  -e "GOOGLE_REDIRECT_URI=$GOOGLE_REDIRECT_URI" \
  -e "CLIENT_URL=http://localhost:${CLIENT_PORT:-8080}" \
  web-template
