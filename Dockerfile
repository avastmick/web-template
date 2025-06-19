# Multi-stage Dockerfile for production deployment
# Builds SvelteKit SPA and serves it from Rust server
# Optimized for serverless container environments (Cloud Run, Fargate)

# Stage 1: Build the client (SvelteKit SPA)
FROM oven/bun:1.1.38-alpine AS client-builder

WORKDIR /app/client

# Copy client package files
COPY client/package.json client/bun.lock ./

# Install client dependencies
RUN bun install --frozen-lockfile

# Copy client source
COPY client/ ./

# Build SvelteKit for production (static SPA)
RUN bun run build

# Stage 2: Build the Rust server and install dbmate
FROM rust:bookworm AS server-builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libsqlite3-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install dbmate for database migrations (detect architecture)
RUN ARCH=$(uname -m) && \
    if [ "$ARCH" = "x86_64" ]; then ARCH="amd64"; elif [ "$ARCH" = "aarch64" ]; then ARCH="arm64"; fi && \
    curl -fsSL -o /usr/local/bin/dbmate https://github.com/amacneil/dbmate/releases/latest/download/dbmate-linux-${ARCH} && \
    chmod +x /usr/local/bin/dbmate

WORKDIR /app/server

# Copy Rust manifest files first (for better caching)
COPY server/Cargo.toml server/Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release && rm -rf src/

# Copy the actual source code
COPY server/src ./src/

# Copy database files
COPY server/db ./db/

# No special environment needed with rustls

# Build the application with rustls (no OpenSSL dependency)
RUN cargo build --release

# Create initial database with migrations
RUN mkdir -p /app/data && \
    cd /app/server/db && \
    DATABASE_URL="sqlite:/app/data/production.sqlite3?mode=rwc" /usr/local/bin/dbmate up

# Stage 3: Final runtime image
FROM scratch

# Copy CA certificates for HTTPS requests (OAuth, etc.)
COPY --from=server-builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# Copy the compiled binary
COPY --from=server-builder /app/server/target/release/server /server

# Copy the built client assets to be served by Rust server
COPY --from=client-builder /app/client/build /app/static

# Copy the pre-built database with migrations applied
COPY --from=server-builder /app/data/production.sqlite3 /app/db-template.sqlite3

# Switch to non-root user (use UID for scratch image)
USER 1000:1000

# Set environment variables for production
ENV RUST_LOG=warn,server=info
ENV DATABASE_URL=sqlite:/data/production.sqlite3?mode=rwc
ENV HOST=0.0.0.0
ENV SERVER_PORT=8080
ENV STATIC_DIR=/app/static

# Expose the port (standard for Cloud Run)
EXPOSE 8080

# Create data directory for SQLite volume mount
VOLUME ["/data"]

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
  CMD ["/server", "--health-check"] || exit 1

# Run the server
ENTRYPOINT ["/server"]
