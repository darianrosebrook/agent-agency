#!/bin/bash

# Agent Agency Production Deployment Script
set -e

echo "🚀 Agent Agency Production Deployment"
echo "====================================="

# Configuration
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEPLOY_ENV="${DEPLOY_ENV:-production}"
DOCKER_COMPOSE_FILE="docker-compose.production.yml"

echo "📁 Project Root: $PROJECT_ROOT"
echo "🌍 Environment: $DEPLOY_ENV"
echo

# Function to check requirements
check_requirements() {
    echo "🔍 Checking deployment requirements..."

    if ! command -v docker &> /dev/null; then
        echo "❌ Docker is required but not installed"
        exit 1
    fi

    if ! command -v docker-compose &> /dev/null; then
        echo "❌ Docker Compose is required but not installed"
        exit 1
    fi

    echo "✅ Requirements met"
}

# Function to build and deploy
deploy() {
    echo "🏗️  Building production image..."
    docker-compose -f "$DOCKER_COMPOSE_FILE" build --no-cache

    echo "🐳 Starting services..."
    docker-compose -f "$DOCKER_COMPOSE_FILE" up -d

    echo "⏳ Waiting for services to be healthy..."
    sleep 30

    echo "🏥 Checking health..."
    if curl -f http://localhost:8080/health &>/dev/null; then
        echo "✅ Deployment successful!"
        echo "🌐 Service available at: http://localhost:8080"
    else
        echo "❌ Health check failed"
        echo "📜 Checking logs..."
        docker-compose -f "$DOCKER_COMPOSE_FILE" logs agent-orchestration
        exit 1
    fi
}

# Function to rollback
rollback() {
    echo "🔄 Rolling back deployment..."
    docker-compose -f "$DOCKER_COMPOSE_FILE" down
    # Add rollback logic here (e.g., deploy previous version)
    echo "✅ Rollback completed"
}

# Main deployment logic
case "${1:-deploy}" in
    "deploy")
        check_requirements
        deploy
        ;;
    "rollback")
        rollback
        ;;
    "logs")
        docker-compose -f "$DOCKER_COMPOSE_FILE" logs -f agent-orchestration
        ;;
    "stop")
        docker-compose -f "$DOCKER_COMPOSE_FILE" down
        ;;
    *)
        echo "Usage: $0 [deploy|rollback|logs|stop]"
        exit 1
        ;;
esac
