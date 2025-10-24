#!/bin/bash

# Agent Agency V3 Full System Demo
# Starts the complete orchestration platform with web interface

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WEB_PORT=3000
API_PORT=8080
DB_NAME="agent_agency"
DB_USER="${DB_USER:-postgres}"
DB_PASSWORD="${DB_PASSWORD:-password}"
DB_HOST="${DB_HOST:-localhost}"
DB_PORT="${DB_PORT:-5432}"

# Database URL
DATABASE_URL="postgresql://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}"

echo -e "${BLUE}🚀 Agent Agency V3 Full System Demo${NC}"
echo -e "${BLUE}=====================================${NC}"
echo

# Function to check if a port is in use
check_port() {
    local port=$1
    if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; then
        echo -e "${RED}❌ Port $port is already in use${NC}"
        return 1
    fi
    return 0
}

# Function to wait for a service to be ready
wait_for_service() {
    local url=$1
    local service_name=$2
    local max_attempts=30
    local attempt=1

    echo -e "${YELLOW}⏳ Waiting for $service_name to be ready...${NC}"

    while [ $attempt -le $max_attempts ]; do
        if curl -s "$url" >/dev/null 2>&1; then
            echo -e "${GREEN}✅ $service_name is ready!${NC}"
            return 0
        fi

        echo -e "${YELLOW}   Attempt $attempt/$max_attempts...${NC}"
        sleep 2
        ((attempt++))
    done

    echo -e "${RED}❌ $service_name failed to start within $(($max_attempts * 2)) seconds${NC}"
    return 1
}

# Function to start PostgreSQL
start_postgres() {
    echo -e "${BLUE}🐘 Starting PostgreSQL...${NC}"

    # Check if PostgreSQL is already running
    if pg_isready -h $DB_HOST -p $DB_PORT >/dev/null 2>&1; then
        echo -e "${GREEN}✅ PostgreSQL is already running${NC}"
        return 0
    fi

    # Try to start PostgreSQL (adjust for your system)
    if command -v brew >/dev/null 2>&1; then
        # macOS with Homebrew
        brew services start postgresql
    elif command -v systemctl >/dev/null 2>&1; then
        # Linux with systemd
        sudo systemctl start postgresql
    else
        echo -e "${RED}❌ Could not determine how to start PostgreSQL${NC}"
        echo -e "${YELLOW}💡 Please start PostgreSQL manually and run this script again${NC}"
        return 1
    fi

    # Wait for PostgreSQL to be ready
    wait_for_service "postgresql://$DB_USER:$DB_PASSWORD@$DB_HOST:$DB_PORT/$DB_NAME" "PostgreSQL"
}

# Function to setup database
setup_database() {
    echo -e "${BLUE}🗄️  Setting up database...${NC}"

    # Create database if it doesn't exist
    psql -h $DB_HOST -p $DB_PORT -U $DB_USER -c "CREATE DATABASE $DB_NAME;" 2>/dev/null || true

    # Run database setup script
    cd "$PROJECT_ROOT"
    if [ -f "scripts/setup/setup-database-v3.cjs" ]; then
        node scripts/setup/setup-database-v3.cjs
    else
        echo -e "${RED}❌ Database setup script not found${NC}"
        return 1
    fi

    echo -e "${GREEN}✅ Database setup complete${NC}"
}

# Function to start API server
start_api_server() {
    echo -e "${BLUE}🔧 Starting API Server...${NC}"

    # Check if port is available
    if ! check_port $API_PORT; then
        echo -e "${YELLOW}💡 Port $API_PORT is in use. Please stop the service using it.${NC}"
        return 1
    fi

    cd "$PROJECT_ROOT"

    # Start API server in background
    export DATABASE_URL
    cargo run --features api-server -- serve --port $API_PORT --database-url "$DATABASE_URL" &
    API_PID=$!

    echo -e "${YELLOW}📝 API Server PID: $API_PID${NC}"

    # Wait for API server to be ready
    wait_for_service "http://localhost:$API_PORT/api/health" "API Server"

    # Store PID for cleanup
    echo $API_PID > .api_server.pid
}

# Function to start web interface
start_web_interface() {
    echo -e "${BLUE}🌐 Starting Web Interface...${NC}"

    # Check if port is available
    if ! check_port $WEB_PORT; then
        echo -e "${YELLOW}💡 Port $WEB_PORT is in use. Please stop the service using it.${NC}"
        return 1
    fi

    cd "$PROJECT_ROOT/web-app"

    # Start web server in background
    python3 server.py $WEB_PORT &
    WEB_PID=$!

    echo -e "${YELLOW}📝 Web Interface PID: $WEB_PID${NC}"

    # Wait for web server to be ready
    wait_for_service "http://localhost:$WEB_PORT" "Web Interface"

    # Store PID for cleanup
    echo $WEB_PID > ../.web_server.pid
}

# Function to show status
show_status() {
    echo
    echo -e "${GREEN}🎉 All services started successfully!${NC}"
    echo
    echo -e "${BLUE}📊 Service Status:${NC}"
    echo -e "  🐘 PostgreSQL:    ${GREEN}Running${NC} (localhost:$DB_PORT)"
    echo -e "  🔧 API Server:    ${GREEN}Running${NC} (http://localhost:$API_PORT)"
    echo -e "  🌐 Web Interface: ${GREEN}Running${NC} (http://localhost:$WEB_PORT)"
    echo
    echo -e "${BLUE}🌟 Access Points:${NC}"
    echo -e "  🖥️  Web Dashboard: ${GREEN}http://localhost:$WEB_PORT${NC}"
    echo -e "  🔌 API Endpoints: ${GREEN}http://localhost:$API_PORT/api/${NC}"
    echo
    echo -e "${BLUE}📋 Available API Endpoints:${NC}"
    echo -e "  GET  /api/health           - System health"
    echo -e "  GET  /api/tasks/active     - Active tasks"
    echo -e "  POST /api/tasks/{id}/execute - Execute task"
    echo -e "  POST /api/workers/register - Register worker"
    echo
    echo -e "${YELLOW}⏹️  Press Ctrl+C to stop all services${NC}"
}

# Function to cleanup on exit
cleanup() {
    echo
    echo -e "${YELLOW}🧹 Cleaning up services...${NC}"

    # Stop API server
    if [ -f ".api_server.pid" ]; then
        API_PID=$(cat .api_server.pid)
        if kill -0 $API_PID 2>/dev/null; then
            echo -e "${YELLOW}Stopping API Server (PID: $API_PID)...${NC}"
            kill $API_PID 2>/dev/null || true
        fi
        rm -f .api_server.pid
    fi

    # Stop web server
    if [ -f ".web_server.pid" ]; then
        WEB_PID=$(cat .web_server.pid)
        if kill -0 $WEB_PID 2>/dev/null; then
            echo -e "${YELLOW}Stopping Web Interface (PID: $WEB_PID)...${NC}"
            kill $WEB_PID 2>/dev/null || true
        fi
        rm -f .web_server.pid
    fi

    echo -e "${GREEN}✅ Cleanup complete${NC}"
    exit 0
}

# Set up signal handlers
trap cleanup SIGINT SIGTERM

# Main execution
main() {
    echo -e "${BLUE}🔍 Pre-flight checks...${NC}"

    # Check if required tools are available
    command -v cargo >/dev/null 2>&1 || { echo -e "${RED}❌ Cargo not found. Please install Rust.${NC}"; exit 1; }
    command -v node >/dev/null 2>&1 || { echo -e "${RED}❌ Node.js not found. Please install Node.js.${NC}"; exit 1; }
    command -v python3 >/dev/null 2>&1 || { echo -e "${RED}❌ Python 3 not found. Please install Python 3.${NC}"; exit 1; }
    command -v psql >/dev/null 2>&1 || { echo -e "${RED}❌ PostgreSQL client not found. Please install PostgreSQL.${NC}"; exit 1; }

    echo -e "${GREEN}✅ All required tools found${NC}"

    # Start services
    start_postgres
    setup_database
    start_api_server
    start_web_interface

    # Show final status
    show_status

    # Wait indefinitely
    echo -e "${BLUE}🔄 Services are running. Press Ctrl+C to stop...${NC}"
    while true; do
        sleep 1
    done
}

# Run main function
main "$@"
