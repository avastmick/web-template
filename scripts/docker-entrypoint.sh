#!/bin/sh
# Docker entrypoint script for web-template
# Handles database migrations and application startup

set -e

# Default to server command if no command provided
if [ $# -eq 0 ]; then
    set -- "/server"
fi

# Check if this is a health check
if [ "$1" = "--health-check" ]; then
    exec /server --health-check
fi

# Check if this is a migration command
if [ "$1" = "migrate" ]; then
    echo "Running database migrations..."
    cd /app/db
    export DATABASE_URL="${DATABASE_URL:-sqlite:/data/production.sqlite3?mode=rwc}"
    /usr/local/bin/dbmate up
    echo "Database migrations completed"
    exit 0
fi

# Check if this is the server startup
if [ "$1" = "/server" ] || [ "$1" = "server" ]; then
    # Ensure data directory exists
    mkdir -p /data

    # Auto-run migrations on server startup if AUTO_MIGRATE is set
    if [ "${AUTO_MIGRATE:-true}" = "true" ]; then
        echo "Auto-running database migrations..."
        cd /app/db
        export DATABASE_URL="${DATABASE_URL:-sqlite:/data/production.sqlite3?mode=rwc}"

        # Run migrations with error handling
        if /usr/local/bin/dbmate up; then
            echo "Database migrations completed successfully"
        else
            echo "Warning: Database migrations failed, continuing with server startup"
        fi
    fi

    echo "Starting web-template server..."
    exec /server
fi

# For any other command, execute as-is
exec "$@"
