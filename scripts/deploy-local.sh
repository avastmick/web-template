#!/bin/bash
# Local deployment script using Docker Compose
# For testing the production container setup locally

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo -e "${GREEN}🚀 Starting local deployment with Docker Compose${NC}"

# Check if we're in the right directory
if [[ ! -f "$PROJECT_ROOT/docker-compose.yml" ]]; then
    echo -e "${RED}❌ Error: docker-compose.yml not found in $PROJECT_ROOT${NC}"
    exit 1
fi

cd "$PROJECT_ROOT"

# Parse command line arguments
ACTION="${1:-up}"

case "$ACTION" in
    "up"|"start")
        echo -e "${BLUE}🏗️  Building and starting services...${NC}"
        docker-compose up --build -d
        echo -e "${GREEN}✅ Services started successfully${NC}"
        echo ""
        echo -e "${YELLOW}📋 Service Status:${NC}"
        docker-compose ps
        echo ""
        echo -e "${YELLOW}🌐 Application URLs:${NC}"
        echo "  Web Application: http://localhost:8080"
        echo "  Health Check: http://localhost:8080/health"
        echo "  Readiness Check: http://localhost:8080/ready"
        echo ""
        echo -e "${YELLOW}📝 Useful commands:${NC}"
        echo "  View logs: docker-compose logs -f"
        echo "  Stop services: $0 down"
        echo "  Restart: $0 restart"
        ;;

    "down"|"stop")
        echo -e "${BLUE}🛑 Stopping services...${NC}"
        docker-compose down
        echo -e "${GREEN}✅ Services stopped${NC}"
        ;;

    "restart")
        echo -e "${BLUE}🔄 Restarting services...${NC}"
        docker-compose down
        docker-compose up --build -d
        echo -e "${GREEN}✅ Services restarted${NC}"
        ;;

    "logs")
        echo -e "${BLUE}📋 Showing logs...${NC}"
        docker-compose logs -f
        ;;

    "clean")
        echo -e "${BLUE}🧹 Cleaning up...${NC}"
        docker-compose down -v --remove-orphans
        docker system prune -f
        echo -e "${GREEN}✅ Cleanup completed${NC}"
        ;;

    "status")
        echo -e "${BLUE}📊 Service status:${NC}"
        docker-compose ps
        echo ""
        echo -e "${BLUE}🔍 Health check:${NC}"
        curl -s http://localhost:8080/health | jq . || echo "Health check failed or jq not installed"
        ;;

    "shell")
        echo -e "${BLUE}🐚 Opening shell in container...${NC}"
        docker-compose exec web-app sh
        ;;

    *)
        echo -e "${YELLOW}📖 Usage: $0 {up|down|restart|logs|clean|status|shell}${NC}"
        echo ""
        echo -e "${YELLOW}Commands:${NC}"
        echo "  up/start  - Build and start services"
        echo "  down/stop - Stop services"
        echo "  restart   - Restart services"
        echo "  logs      - View logs"
        echo "  clean     - Stop services and clean up volumes"
        echo "  status    - Show service status and health"
        echo "  shell     - Open shell in container"
        exit 1
        ;;
esac
