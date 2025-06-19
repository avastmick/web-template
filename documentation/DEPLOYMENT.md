# Deployment Guide

This document provides comprehensive instructions for deploying the web-template application to production environments.

## Table of Contents

1. [Production Requirements](#production-requirements)
2. [Environment Configuration](#environment-configuration)
3. [Security Configuration](#security-configuration)
4. [Database Setup](#database-setup)
5. [OAuth Configuration](#oauth-configuration)
6. [Deployment Options](#deployment-options)
7. [Health Checks and Monitoring](#health-checks-and-monitoring)
8. [Backup and Recovery](#backup-and-recovery)
9. [Troubleshooting](#troubleshooting)

## Production Requirements

### System Requirements

- **Operating System**: Linux (Ubuntu 20.04+ recommended) or Container Runtime
- **Memory**: Minimum 512MB RAM, 1GB+ recommended
- **Storage**: Minimum 1GB free space for application and database
- **Network**: HTTPS enabled (TLS 1.2+)

### Software Dependencies

- **Rust**: 1.70+ (for server compilation)
- **Node.js**: 18+ (for client build if needed)
- **Database**: SQLite (included) or PostgreSQL (for scaling)
- **Reverse Proxy**: Nginx, Caddy, or Cloudflare (recommended)

## Environment Configuration

### Required Environment Variables

Create a production `.env` file with the following variables:

```bash
# Database Configuration
DATABASE_URL="sqlite:./db/production.sqlite3?mode=rwc"

# Authentication
JWT_SECRET="your-super-secure-64-character-jwt-secret-key-for-production"

# OAuth Configuration
GOOGLE_CLIENT_ID="your-production-google-client-id.apps.googleusercontent.com"
GOOGLE_CLIENT_SECRET="your-production-google-client-secret"
SERVER_URL="https://yourdomain.com"

# Server Configuration
HOST="0.0.0.0"
SERVER_PORT="8081"
ALLOWED_ORIGINS="https://yourdomain.com"

# Logging
RUST_LOG="warn,server=info"

# Optional: Database Pool Configuration
DB_POOL_MAX_CONNECTIONS="10"
```

### Environment Variable Security

1. **JWT_SECRET**: Generate a cryptographically secure secret:
   ```bash
   openssl rand -hex 32
   ```

2. **Database URL**: Use strong file permissions:
   ```bash
   chmod 600 production.env
   ```

3. **Secrets Management**: Consider using:
   - AWS Secrets Manager
   - HashiCorp Vault
   - Google Secret Manager
   - Environment-specific CI/CD secrets

## Security Configuration

### TLS/HTTPS Setup

**NEVER run in production without HTTPS.** Configure your reverse proxy:

#### Nginx Configuration Example
```nginx
server {
    listen 443 ssl http2;
    server_name yourdomain.com;

    ssl_certificate /path/to/your/certificate.crt;
    ssl_certificate_key /path/to/your/private.key;

    # Security headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Frame-Options DENY always;
    add_header X-Content-Type-Options nosniff always;
    add_header Referrer-Policy strict-origin-when-cross-origin always;

    location / {
        proxy_pass http://127.0.0.1:8081;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}

# Redirect HTTP to HTTPS
server {
    listen 80;
    server_name yourdomain.com;
    return 301 https://$server_name$request_uri;
}
```

### File Permissions

```bash
# Application files
chmod 755 /path/to/your/app
chmod 600 /path/to/your/app/.env

# Database
chmod 600 /path/to/your/app/db/production.sqlite3
chown app-user:app-user /path/to/your/app/db/production.sqlite3

# Logs directory
mkdir -p /var/log/your-app
chown app-user:app-user /var/log/your-app
chmod 755 /var/log/your-app
```

### User Isolation

Run the application as a non-root user:

```bash
# Create dedicated user
sudo useradd -r -s /bin/false your-app-user

# Run application as this user
sudo -u your-app-user ./your-app-binary
```

## Database Setup

### SQLite Production Configuration

1. **Database File Location**:
   ```bash
   mkdir -p /var/lib/your-app/db
   chown your-app-user:your-app-user /var/lib/your-app/db
   chmod 700 /var/lib/your-app/db
   ```

2. **Run Migrations**:
   ```bash
   DATABASE_URL="sqlite:/var/lib/your-app/db/production.sqlite3?mode=rwc" dbmate up
   ```

3. **WAL Mode for Performance**:
   ```sql
   PRAGMA journal_mode=WAL;
   PRAGMA synchronous=NORMAL;
   PRAGMA cache_size=1000;
   PRAGMA foreign_keys=true;
   PRAGMA temp_store=memory;
   ```

### PostgreSQL (Recommended for Scale)

For production scale, consider PostgreSQL:

```bash
# Database URL format
DATABASE_URL="postgresql://username:password@localhost:5432/your_app_db"
```

## OAuth Configuration

### Google OAuth Production Setup

1. **Create Production OAuth Client**:
   - Go to [Google Cloud Console](https://console.cloud.google.com/)
   - Create a new project or use existing
   - Enable Google OAuth 2.0 API
   - Create credentials → OAuth 2.0 Client ID

2. **Configure Authorized Redirect URIs**:
   ```
   https://yourdomain.com/api/auth/oauth/google/callback
   ```

3. **Domain Verification**:
   - Verify your domain in Google Search Console
   - Add your domain to OAuth consent screen

4. **Production Consent Screen**:
   - Complete OAuth consent screen configuration
   - Add privacy policy and terms of service URLs
   - Submit for verification if using sensitive scopes

## Deployment Options

### Option 1: Docker Deployment

1. **Build Docker Image**:
   ```dockerfile
   # Dockerfile
   FROM rust:1.70 as builder
   WORKDIR /app
   COPY server/ .
   RUN cargo build --release

   FROM debian:bullseye-slim
   RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
   WORKDIR /app
   COPY --from=builder /app/target/release/server .
   COPY --from=builder /app/db ./db
   EXPOSE 8081
   CMD ["./server"]
   ```

2. **Build and Run**:
   ```bash
   docker build -t your-app .
   docker run -d -p 8081:8081 --env-file production.env your-app
   ```

### Option 2: Systemd Service

1. **Create Service File** (`/etc/systemd/system/your-app.service`):
   ```ini
   [Unit]
   Description=Your Web Application
   After=network.target

   [Service]
   Type=simple
   User=your-app-user
   WorkingDirectory=/opt/your-app
   Environment=DATABASE_URL=sqlite:/var/lib/your-app/db/production.sqlite3?mode=rwc
   EnvironmentFile=/opt/your-app/.env
   ExecStart=/opt/your-app/server
   Restart=always
   RestartSec=5

   [Install]
   WantedBy=multi-user.target
   ```

2. **Enable and Start**:
   ```bash
   sudo systemctl daemon-reload
   sudo systemctl enable your-app
   sudo systemctl start your-app
   sudo systemctl status your-app
   ```

### Option 3: Cloud Deployment

#### Google Cloud Run
```yaml
# cloudbuild.yaml
steps:
  - name: 'gcr.io/cloud-builders/docker'
    args: ['build', '-t', 'gcr.io/$PROJECT_ID/your-app', '.']
  - name: 'gcr.io/cloud-builders/docker'
    args: ['push', 'gcr.io/$PROJECT_ID/your-app']
  - name: 'gcr.io/cloud-builders/gcloud'
    args:
      - 'run'
      - 'deploy'
      - 'your-app'
      - '--image=gcr.io/$PROJECT_ID/your-app'
      - '--region=us-central1'
      - '--platform=managed'
```

#### Vercel (Frontend + Serverless Backend)
```json
{
  "builds": [
    {
      "src": "client/package.json",
      "use": "@vercel/static-build",
      "config": { "distDir": "build" }
    }
  ],
  "env": {
    "VITE_SERVER_PORT": "8081"
  }
}
```

## Health Checks and Monitoring

### Application Health Check

Add a health endpoint to your server (`server/src/handlers/health.rs`):

```rust
use axum::{response::Json, http::StatusCode};
use serde_json::json;

pub async fn health_check() -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION")
    })))
}
```

### Monitoring Setup

1. **Application Metrics**:
   ```bash
   # CPU and Memory usage
   ps aux | grep your-app

   # Disk usage
   du -sh /var/lib/your-app/

   # Database size
   ls -lh /var/lib/your-app/db/
   ```

2. **Log Monitoring**:
   ```bash
   # Centralized logging
   journalctl -u your-app -f

   # Error tracking
   grep ERROR /var/log/your-app/app.log
   ```

3. **Uptime Monitoring**:
   - Use services like Uptime Robot, Pingdom, or StatusCake
   - Monitor `https://yourdomain.com/health`

## Backup and Recovery

### Database Backup

1. **SQLite Backup Script**:
   ```bash
   #!/bin/bash
   # backup-db.sh

   DB_PATH="/var/lib/your-app/db/production.sqlite3"
   BACKUP_DIR="/var/backups/your-app"
   DATE=$(date +%Y%m%d_%H%M%S)

   # Create backup directory
   mkdir -p $BACKUP_DIR

   # Create backup
   sqlite3 $DB_PATH ".backup $BACKUP_DIR/db_backup_$DATE.sqlite3"

   # Compress backup
   gzip "$BACKUP_DIR/db_backup_$DATE.sqlite3"

   # Keep only last 30 days
   find $BACKUP_DIR -name "*.gz" -mtime +30 -delete

   echo "Backup completed: db_backup_$DATE.sqlite3.gz"
   ```

2. **Automated Backups with Cron**:
   ```bash
   # Add to crontab
   0 2 * * * /opt/your-app/backup-db.sh
   ```

### Recovery Procedures

1. **Database Recovery**:
   ```bash
   # Stop application
   sudo systemctl stop your-app

   # Restore from backup
   gunzip /var/backups/your-app/db_backup_YYYYMMDD_HHMMSS.sqlite3.gz
   cp /var/backups/your-app/db_backup_YYYYMMDD_HHMMSS.sqlite3 /var/lib/your-app/db/production.sqlite3
   chown your-app-user:your-app-user /var/lib/your-app/db/production.sqlite3

   # Start application
   sudo systemctl start your-app
   ```

2. **Application Recovery**:
   ```bash
   # Rolling update
   sudo systemctl stop your-app
   cp /path/to/new/binary /opt/your-app/server
   sudo systemctl start your-app

   # Rollback if needed
   cp /opt/your-app/server.backup /opt/your-app/server
   sudo systemctl restart your-app
   ```

## Troubleshooting

### Common Issues

1. **OAuth Redirect URI Mismatch**:
   ```
   Error: redirect_uri_mismatch
   Solution: Verify OAuth redirect URIs in Google Console match exactly
   ```

2. **Database Permission Errors**:
   ```bash
   # Fix database permissions
   chown your-app-user:your-app-user /var/lib/your-app/db/production.sqlite3
   chmod 600 /var/lib/your-app/db/production.sqlite3
   ```

3. **JWT Token Issues**:
   ```bash
   # Verify JWT secret is set and consistent
   echo $JWT_SECRET | wc -c  # Should be 32+ characters
   ```

4. **CORS Issues**:
   ```bash
   # Verify ALLOWED_ORIGINS matches your domain
   export ALLOWED_ORIGINS="https://yourdomain.com"
   ```

### Log Analysis

```bash
# Check application logs
journalctl -u your-app -n 100

# Check nginx logs
tail -f /var/log/nginx/access.log
tail -f /var/log/nginx/error.log

# Database connection issues
grep "database" /var/log/your-app/app.log
```

### Performance Tuning

1. **Database Optimization**:
   ```sql
   -- Enable WAL mode
   PRAGMA journal_mode=WAL;

   -- Optimize cache
   PRAGMA cache_size=-64000;  -- 64MB cache

   -- Analyze query performance
   EXPLAIN QUERY PLAN SELECT * FROM users WHERE email = ?;
   ```

2. **Connection Pool Tuning**:
   ```bash
   # Adjust based on expected load
   export DB_POOL_MAX_CONNECTIONS="20"
   ```

## Production Checklist

Before going live, ensure:

- [ ] HTTPS is properly configured
- [ ] OAuth credentials are production-ready
- [ ] Database backups are automated
- [ ] Monitoring and alerting are configured
- [ ] Log rotation is set up
- [ ] Security headers are enabled
- [ ] Environment variables are secured
- [ ] Health checks are working
- [ ] Error handling is comprehensive
- [ ] Performance testing is completed

For additional support, refer to `ARCHITECTURE.md` for technical details and `CLAUDE.md` for development guidelines.
