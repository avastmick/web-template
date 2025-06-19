#!/bin/bash
# Setup script for systemd deployment
# Creates user, directories, and installs systemd service

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
APP_USER="web-template"
APP_GROUP="web-template"
APP_DIR="/opt/web-template"
DATA_DIR="/var/lib/web-template"
LOG_DIR="/var/log/web-template"

echo -e "${GREEN}🔧 Setting up systemd deployment for web-template${NC}"

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   echo -e "${RED}❌ This script must be run as root${NC}"
   exit 1
fi

# Create system user and group
echo -e "${BLUE}👤 Creating system user and group...${NC}"
if ! getent group "$APP_GROUP" >/dev/null 2>&1; then
    groupadd --system "$APP_GROUP"
    echo -e "${GREEN}✅ Created group: $APP_GROUP${NC}"
else
    echo -e "${YELLOW}⚠️  Group already exists: $APP_GROUP${NC}"
fi

if ! getent passwd "$APP_USER" >/dev/null 2>&1; then
    useradd --system --gid "$APP_GROUP" --shell /bin/false \
            --home-dir "$APP_DIR" --create-home \
            --comment "Web Template Application" "$APP_USER"
    echo -e "${GREEN}✅ Created user: $APP_USER${NC}"
else
    echo -e "${YELLOW}⚠️  User already exists: $APP_USER${NC}"
fi

# Create directories
echo -e "${BLUE}📁 Creating directories...${NC}"
mkdir -p "$APP_DIR"
mkdir -p "$DATA_DIR/db"
mkdir -p "$LOG_DIR"

# Set ownership and permissions
chown -R "$APP_USER:$APP_GROUP" "$APP_DIR"
chown -R "$APP_USER:$APP_GROUP" "$DATA_DIR"
chown -R "$APP_USER:$APP_GROUP" "$LOG_DIR"

chmod 755 "$APP_DIR"
chmod 700 "$DATA_DIR"
chmod 755 "$LOG_DIR"

echo -e "${GREEN}✅ Directories created and secured${NC}"

# Copy systemd service file
echo -e "${BLUE}⚙️  Installing systemd service...${NC}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SERVICE_FILE="$SCRIPT_DIR/web-template.service"

if [[ -f "$SERVICE_FILE" ]]; then
    cp "$SERVICE_FILE" /etc/systemd/system/
    chown root:root /etc/systemd/system/web-template.service
    chmod 644 /etc/systemd/system/web-template.service
    echo -e "${GREEN}✅ Systemd service installed${NC}"
else
    echo -e "${RED}❌ Service file not found: $SERVICE_FILE${NC}"
    exit 1
fi

# Reload systemd and enable service
echo -e "${BLUE}🔄 Reloading systemd...${NC}"
systemctl daemon-reload

echo -e "${BLUE}🚀 Enabling web-template service...${NC}"
systemctl enable web-template.service

echo -e "${GREEN}✅ Setup completed successfully!${NC}"
echo ""
echo -e "${YELLOW}📝 Next steps:${NC}"
echo "  1. Copy your application binary to: $APP_DIR/server"
echo "  2. Copy static files to: $APP_DIR/static/"
echo "  3. Create environment file: $APP_DIR/.env"
echo "  4. Run database migrations if needed"
echo "  5. Start the service: systemctl start web-template"
echo ""
echo -e "${YELLOW}📋 Example environment file ($APP_DIR/.env):${NC}"
echo "JWT_SECRET=your-super-secure-64-character-jwt-secret-key"
echo "GOOGLE_CLIENT_ID=your-google-client-id.apps.googleusercontent.com"
echo "GOOGLE_CLIENT_SECRET=your-google-client-secret"
echo "SERVER_URL=https://yourdomain.com"
echo "ALLOWED_ORIGINS=https://yourdomain.com"
echo ""
echo -e "${YELLOW}🔧 Useful systemd commands:${NC}"
echo "  systemctl start web-template     - Start service"
echo "  systemctl stop web-template      - Stop service"
echo "  systemctl restart web-template   - Restart service"
echo "  systemctl status web-template    - Check status"
echo "  journalctl -u web-template -f    - View logs"
