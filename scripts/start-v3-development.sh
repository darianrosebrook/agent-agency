#!/bin/bash

# Agent Agency V3 Development Startup Script
# Starts all necessary services for V3 development

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
DASHBOARD_PORT=3002
BACKEND_PORT=8080
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
V3_DIR="$PROJECT_ROOT/iterations/v3"

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if port is in use
check_port() {
    local port=$1
    if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; then
        return 0  # Port is in use
    else
        return 1  # Port is free
    fi
}

# Function to wait for service to be ready
wait_for_service() {
    local url=$1
    local service_name=$2
    local max_attempts=30
    local attempt=1

    print_status "Waiting for $service_name to be ready at $url..."

    while [ $attempt -le $max_attempts ]; do
        if curl -s --max-time 2 "$url" > /dev/null 2>&1; then
            print_success "$service_name is ready!"
            return 0
        fi

        echo -n "."
        sleep 1
        attempt=$((attempt + 1))
    done

    print_error "$service_name failed to start within ${max_attempts}s"
    return 1
}

# Function to cleanup on exit
cleanup() {
    print_status "Shutting down services..."

    # Stop dashboard if running
    if [ -f "$V3_DIR/apps/web-dashboard/dashboard.pid" ]; then
        print_status "Stopping dashboard..."
        cd "$V3_DIR/apps/web-dashboard"
        npm run stop 2>/dev/null || true
    fi

    # Kill any background processes started by this script
    if [ ! -z "$DASHBOARD_PID" ]; then
        kill $DASHBOARD_PID 2>/dev/null || true
    fi

    print_success "Cleanup complete"
    exit 0
}

# Set up signal handlers for graceful shutdown
trap cleanup SIGINT SIGTERM

# Main script
main() {
    print_status "Starting Agent Agency V3 Development Environment"
    print_status "Dashboard will run on: http://localhost:$DASHBOARD_PORT"
    print_status "Backend API expected on: http://localhost:$BACKEND_PORT"

    # Check if required tools are installed
    if ! command -v node &> /dev/null; then
        print_error "Node.js is not installed. Please install Node.js first."
        exit 1
    fi

    if ! command -v npm &> /dev/null; then
        print_error "npm is not installed. Please install npm first."
        exit 1
    fi

    # Check if ports are available
    if check_port $DASHBOARD_PORT; then
        print_warning "Port $DASHBOARD_PORT is already in use. Dashboard may fail to start."
        print_warning "To find what's using the port: lsof -i :$DASHBOARD_PORT"
        print_warning "To kill the process: kill -9 \$(lsof -ti :$DASHBOARD_PORT)"
    fi

    # Navigate to dashboard directory
    cd "$V3_DIR/apps/web-dashboard"

    # Check if node_modules exists, install if needed
    if [ ! -d "node_modules" ]; then
        print_status "Installing dashboard dependencies..."
        npm install
    fi

    # Start dashboard in background
    print_status "Starting web dashboard..."
    npm run dev:bg

    # Wait a moment for dashboard to start
    sleep 3

    # Check if dashboard started successfully
    if [ -f "dashboard.pid" ] && kill -0 $(cat dashboard.pid) 2>/dev/null; then
        print_success "Dashboard started successfully (PID: $(cat dashboard.pid))"
        print_success "Dashboard available at: http://localhost:$DASHBOARD_PORT"

        # Try to wait for dashboard to be ready
        if wait_for_service "http://localhost:$DASHBOARD_PORT" "Dashboard"; then
            print_success "All services started successfully!"
            print_status ""
            print_status "Useful commands:"
            print_status "  Check status:     npm run status"
            print_status "  Stop services:    npm run stop"
            print_status "  Restart:          npm run restart"
            print_status "  View logs:        tail -f dashboard.log"
            print_status ""
            print_status "Press Ctrl+C to stop all services"

            # Keep script running to maintain services
            while true; do
                sleep 1
            done
        fi
    else
        print_error "Dashboard failed to start"
        if [ -f "dashboard.log" ]; then
            print_error "Check dashboard.log for details:"
            tail -20 dashboard.log
        fi
        exit 1
    fi
}

# Function to show usage
usage() {
    echo "Agent Agency V3 Development Startup Script"
    echo ""
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  -h, --help          Show this help message"
    echo "  --dashboard-only    Start only the dashboard (no backend)"
    echo "  --backend-only      Start only the backend services"
    echo "  --no-wait           Don't wait for services to be ready"
    echo ""
    echo "Examples:"
    echo "  $0                    Start everything"
    echo "  $0 --dashboard-only  Start only dashboard on port $DASHBOARD_PORT"
    echo "  $0 --help            Show this help"
}

# Parse command line arguments
case "${1:-}" in
    -h|--help)
        usage
        exit 0
        ;;
    --dashboard-only)
        print_status "Starting dashboard only..."
        cd "$V3_DIR/apps/web-dashboard"
        npm run dev
        exit $?
        ;;
    --backend-only)
        print_warning "Backend-only startup not implemented yet"
        print_status "Use individual cargo commands to start backend services"
        exit 1
        ;;
    *)
        main
        ;;
esac
