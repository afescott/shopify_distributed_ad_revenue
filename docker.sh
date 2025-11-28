#!/bin/bash
# Docker Compose wrapper script for monorepo
# Usage: ./docker.sh [command] [service...]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMPOSE_FILE="$SCRIPT_DIR/docker-compose.yml"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

show_help() {
    cat << EOF
Docker Compose Wrapper for Shopify Margin Cost Dashboard

Usage:
    ./docker.sh [command] [options] [services...]

Commands:
    up              Start all services (default)
    down            Stop all services
    build           Build all services
    rebuild         Rebuild all services (no cache)
    restart         Restart services
    logs            Show logs (use -f for follow)
    ps              List running containers
    exec <service>  Execute command in service container
    clean           Stop and remove containers, volumes
    help            Show this help message

Options:
    -s, --service <name>    Build/run specific service(s)
                            e.g., -s auth-api -s shopify-consumer
    --no-lib                Skip building lib-shopify (services build it automatically)
    --lib-only              Build only lib-shopify library
    -f, --follow            Follow logs (for logs command)

Services:
    auth-api               Authentication & API service
    shopify-consumer       Shopify sync service
    profit-engine          Profit calculation service (when ready)
    postgres               PostgreSQL database
    kafka                  Kafka message broker
    zookeeper              Zookeeper (required for Kafka)

Examples:
    ./docker.sh                    # Start all services
    ./docker.sh up                 # Start all services
    ./docker.sh build -s auth-api  # Build only auth-api
    ./docker.sh build -s auth-api -s shopify-consumer
    ./docker.sh logs auth-api -f   # Follow auth-api logs
    ./docker.sh restart auth-api   # Restart auth-api
    ./docker.sh down               # Stop all services
    ./docker.sh clean              # Clean everything

Note: lib-shopify is built automatically as a dependency when building services.
      Use --lib-only to build it separately for development.
EOF
}

# Build lib-shopify library
build_lib() {
    echo -e "${GREEN}📦 Building lib-shopify...${NC}"
    cd "$SCRIPT_DIR/libs/lib-shopify"
    cargo build --release
    echo -e "${GREEN}✅ lib-shopify built successfully${NC}"
    cd "$SCRIPT_DIR"
}

# Build specific service(s)
build_service() {
    local service=$1
    case $service in
        auth-api)
            echo -e "${GREEN}🔨 Building auth-api...${NC}"
            cd "$SCRIPT_DIR/services/auth-api"
            cargo build --release
            echo -e "${GREEN}✅ auth-api built successfully${NC}"
            ;;
        shopify-consumer)
            echo -e "${GREEN}🔨 Building shopify-consumer...${NC}"
            cd "$SCRIPT_DIR/services/shopify-consumer"
            cargo build --release
            echo -e "${GREEN}✅ shopify-consumer built successfully${NC}"
            ;;
        profit-engine)
            echo -e "${GREEN}🔨 Building profit-engine...${NC}"
            cd "$SCRIPT_DIR/services/profit-engine"
            cargo build --release
            echo -e "${GREEN}✅ profit-engine built successfully${NC}"
            ;;
        *)
            echo -e "${RED}❌ Unknown service: $service${NC}"
            echo "Available services: auth-api, shopify-consumer, profit-engine"
            exit 1
            ;;
    esac
    cd "$SCRIPT_DIR"
}

# Parse arguments
COMMAND="up"
SERVICES=()
BUILD_LIB=true
LIB_ONLY=false
FOLLOW=false

while [[ $# -gt 0 ]]; do
    case $1 in
        up|down|build|rebuild|restart|logs|ps|exec|clean|help)
            COMMAND=$1
            shift
            ;;
        -s|--service)
            SERVICES+=("$2")
            shift 2
            ;;
        --no-lib)
            BUILD_LIB=false
            shift
            ;;
        --lib-only)
            LIB_ONLY=true
            COMMAND="build-lib"
            shift
            ;;
        -f|--follow)
            FOLLOW=true
            shift
            ;;
        -*)
            echo -e "${RED}Unknown option: $1${NC}"
            show_help
            exit 1
            ;;
        *)
            # Treat as service name if no command specified
            if [[ "$COMMAND" == "up" ]]; then
                SERVICES+=("$1")
            else
                SERVICES+=("$1")
            fi
            shift
            ;;
    esac
done

# Execute command
case $COMMAND in
    help)
        show_help
        exit 0
        ;;
    build-lib)
        build_lib
        exit 0
        ;;
    build)
        if [[ ${#SERVICES[@]} -eq 0 ]]; then
            # Build all services
            echo -e "${GREEN}🔨 Building all services...${NC}"
            [[ "$BUILD_LIB" == true ]] && build_lib
            build_service "auth-api"
            build_service "shopify-consumer"
            build_service "profit-engine"
        else
            # Build specific services
            [[ "$BUILD_LIB" == true ]] && build_lib
            for service in "${SERVICES[@]}"; do
                build_service "$service"
            done
        fi
        echo -e "${GREEN}✅ All builds completed${NC}"
        ;;
    rebuild)
        echo -e "${YELLOW}⚠️  Rebuilding (no cache)...${NC}"
        if [[ ${#SERVICES[@]} -eq 0 ]]; then
            docker compose -f "$COMPOSE_FILE" build --no-cache
        else
            docker compose -f "$COMPOSE_FILE" build --no-cache "${SERVICES[@]}"
        fi
        ;;
    up)
        if [[ ${#SERVICES[@]} -eq 0 ]]; then
            echo -e "${GREEN}🚀 Starting all services...${NC}"
            docker compose -f "$COMPOSE_FILE" up -d
            docker compose -f "$COMPOSE_FILE" ps
        else
            echo -e "${GREEN}🚀 Starting services: ${SERVICES[*]}${NC}"
            docker compose -f "$COMPOSE_FILE" up -d "${SERVICES[@]}"
            docker compose -f "$COMPOSE_FILE" ps
        fi
        ;;
    down)
        echo -e "${YELLOW}🛑 Stopping services...${NC}"
        if [[ ${#SERVICES[@]} -eq 0 ]]; then
            docker compose -f "$COMPOSE_FILE" down
        else
            docker compose -f "$COMPOSE_FILE" stop "${SERVICES[@]}"
        fi
        ;;
    restart)
        if [[ ${#SERVICES[@]} -eq 0 ]]; then
            echo -e "${YELLOW}🔄 Restarting all services...${NC}"
            docker compose -f "$COMPOSE_FILE" restart
        else
            echo -e "${YELLOW}🔄 Restarting services: ${SERVICES[*]}${NC}"
            docker compose -f "$COMPOSE_FILE" restart "${SERVICES[@]}"
        fi
        ;;
    logs)
        if [[ ${#SERVICES[@]} -eq 0 ]]; then
            if [[ "$FOLLOW" == true ]]; then
                docker compose -f "$COMPOSE_FILE" logs -f
            else
                docker compose -f "$COMPOSE_FILE" logs
            fi
        else
            if [[ "$FOLLOW" == true ]]; then
                docker compose -f "$COMPOSE_FILE" logs -f "${SERVICES[@]}"
            else
                docker compose -f "$COMPOSE_FILE" logs "${SERVICES[@]}"
            fi
        fi
        ;;
    ps)
        docker compose -f "$COMPOSE_FILE" ps
        ;;
    exec)
        if [[ ${#SERVICES[@]} -lt 1 ]]; then
            echo -e "${RED}❌ exec requires a service name${NC}"
            echo "Usage: ./docker.sh exec <service> <command>"
            exit 1
        fi
        SERVICE="${SERVICES[0]}"
        shift
        docker compose -f "$COMPOSE_FILE" exec "$SERVICE" "$@"
        ;;
    clean)
        echo -e "${RED}🧹 Cleaning all containers, volumes, and images...${NC}"
        read -p "Are you sure? (y/N) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            docker compose -f "$COMPOSE_FILE" down -v --rmi all
            echo -e "${GREEN}✅ Cleanup complete${NC}"
        else
            echo "Cancelled"
        fi
        ;;
    *)
        echo -e "${RED}Unknown command: $COMMAND${NC}"
        show_help
        exit 1
        ;;
esac










