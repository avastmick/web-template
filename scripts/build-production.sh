#!/bin/bash
# Production build script for web-template application
# Builds a production-ready Docker image

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo -e "${GREEN}🏗️  Building production Docker image for web-template${NC}"

# Check if we're in the right directory
if [[ ! -f "$PROJECT_ROOT/Dockerfile" ]]; then
    echo -e "${RED}❌ Error: Dockerfile not found in $PROJECT_ROOT${NC}"
    echo "Please run this script from the web-template directory or ensure Dockerfile exists"
    exit 1
fi

# Check if required files exist
if [[ ! -f "$PROJECT_ROOT/client/package.json" ]]; then
    echo -e "${RED}❌ Error: client/package.json not found${NC}"
    exit 1
fi

if [[ ! -f "$PROJECT_ROOT/server/Cargo.toml" ]]; then
    echo -e "${RED}❌ Error: server/Cargo.toml not found${NC}"
    exit 1
fi

# Set default image name and tag
IMAGE_NAME="${IMAGE_NAME:-web-template}"
IMAGE_TAG="${IMAGE_TAG:-latest}"
FULL_IMAGE_NAME="$IMAGE_NAME:$IMAGE_TAG"

echo -e "${YELLOW}📋 Build Configuration:${NC}"
echo "  Image: $FULL_IMAGE_NAME"
echo "  Project Root: $PROJECT_ROOT"
echo ""

# Build the Docker image
echo -e "${GREEN}🔨 Building Docker image...${NC}"
cd "$PROJECT_ROOT"

if docker build -t "$FULL_IMAGE_NAME" .; then
    echo -e "${GREEN}✅ Docker image built successfully: $FULL_IMAGE_NAME${NC}"
else
    echo -e "${RED}❌ Failed to build Docker image${NC}"
    exit 1
fi

# Get image size
IMAGE_SIZE=$(docker images "$FULL_IMAGE_NAME" --format "table {{.Size}}" | tail -n 1)
echo -e "${GREEN}📦 Image size: $IMAGE_SIZE${NC}"

# Verify the image works
echo -e "${GREEN}🧪 Testing image health check...${NC}"
if timeout 30 docker run --rm "$FULL_IMAGE_NAME" --health-check; then
    echo -e "${GREEN}✅ Health check passed${NC}"
else
    echo -e "${RED}❌ Health check failed${NC}"
    exit 1
fi

# Test database initialization
echo -e "${GREEN}🧪 Testing database initialization...${NC}"
if timeout 30 docker run --rm -v temp_test_db:/data "$FULL_IMAGE_NAME" --init-db; then
    echo -e "${GREEN}✅ Database initialization test passed${NC}"
    # Clean up test volume
    docker volume rm temp_test_db >/dev/null 2>&1 || true
else
    echo -e "${RED}❌ Database initialization test failed${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}🎉 Production build completed successfully!${NC}"
echo ""
echo -e "${YELLOW}📝 Next steps:${NC}"
echo "  1. Test locally: docker run -p 8080:8080 -v db_data:/data $FULL_IMAGE_NAME"
echo "  2. Push to registry: docker push $FULL_IMAGE_NAME"
echo "  3. Deploy to production environment"
echo ""
echo -e "${YELLOW}💡 Environment variables needed for production:${NC}"
echo "  - JWT_SECRET (required)"
echo "  - GOOGLE_CLIENT_ID (required)"
echo "  - GOOGLE_CLIENT_SECRET (required)"
echo "  - SERVER_URL (required)"
echo "  - ALLOWED_ORIGINS (required)"
