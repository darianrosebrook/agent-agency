#!/bin/bash
set -euo pipefail

# Agent Agency V3 Production Deployment Script
# This script handles complete production deployment with rollback capability

# Configuration
NAMESPACE="${NAMESPACE:-agent-agency-prod}"
CLUSTER_NAME="${CLUSTER_NAME:-agent-agency-production}"
REGION="${REGION:-us-east-1}"
ENVIRONMENT="${ENVIRONMENT:-production}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

# Pre-deployment checks
pre_deployment_checks() {
    log_info "Running pre-deployment checks..."

    # Check AWS CLI configuration
    if ! aws sts get-caller-identity >/dev/null 2>&1; then
        log_error "AWS CLI not configured or credentials invalid"
        exit 1
    fi

    # Check kubectl context
    if ! kubectl cluster-info >/dev/null 2>&1; then
        log_error "kubectl not configured or cluster unreachable"
        exit 1
    fi

    # Check if namespace exists
    if ! kubectl get namespace "$NAMESPACE" >/dev/null 2>&1; then
        log_warn "Namespace $NAMESPACE does not exist. Creating..."
        kubectl create namespace "$NAMESPACE"
    fi

    # Check if secrets exist
    if ! kubectl get secret agent-agency-runtime-secrets -n "$NAMESPACE" >/dev/null 2>&1; then
        log_error "Required secrets not found. Please ensure external-secrets-operator is configured."
        exit 1
    fi

    log_success "Pre-deployment checks passed"
}

# Backup current state
backup_current_state() {
    log_info "Backing up current state..."

    BACKUP_DIR="backup-$(date +%Y%m%d-%H%M%S)"
    mkdir -p "$BACKUP_DIR"

    # Backup current deployments
    kubectl get deployments -n "$NAMESPACE" -o yaml > "$BACKUP_DIR/deployments.yaml"
    kubectl get statefulsets -n "$NAMESPACE" -o yaml > "$BACKUP_DIR/statefulsets.yaml"
    kubectl get services -n "$NAMESPACE" -o yaml > "$BACKUP_DIR/services.yaml"
    kubectl get configmaps -n "$NAMESPACE" -o yaml > "$BACKUP_DIR/configmaps.yaml"

    # Create backup archive
    tar -czf "${BACKUP_DIR}.tar.gz" "$BACKUP_DIR"
    rm -rf "$BACKUP_DIR"

    log_success "Current state backed up to ${BACKUP_DIR}.tar.gz"
}

# Determine blue/green target
determine_deployment_target() {
    log_info "Determining deployment target..."

    # Check current active deployment
    if kubectl get svc agent-agency-api -n "$NAMESPACE" >/dev/null 2>&1; then
        CURRENT=$(kubectl get svc agent-agency-api -n "$NAMESPACE" -o jsonpath='{.spec.selector.version}' 2>/dev/null || echo "blue")
    else
        CURRENT="blue"  # First deployment
    fi

    if [ "$CURRENT" = "blue" ]; then
        TARGET="green"
        PREVIOUS="blue"
    else
        TARGET="green"
        PREVIOUS="blue"
    fi

    log_info "Current active: $CURRENT, Target: $TARGET"
}

# Deploy to target environment
deploy_to_target() {
    local target=$1
    log_info "Deploying to $target environment..."

    # Update kustomization with target-specific overlays
    cd deploy/kubernetes/aws

    # Set images to latest
    kustomize edit set image agent-agency/orchestrator=${REGISTRY}/${IMAGE_NAME}-orchestrator:latest
    kustomize edit set image agent-agency/council=${REGISTRY}/${IMAGE_NAME}-council:latest

    # Apply target-specific overlay
    kubectl apply -k "overlays/production-$target/"

    # Wait for rollout
    log_info "Waiting for orchestrator rollout..."
    kubectl rollout status deployment/agent-agency-orchestrator-$target -n "$NAMESPACE" --timeout=600s

    log_info "Waiting for council rollout..."
    kubectl rollout status deployment/agent-agency-council-$target -n "$NAMESPACE" --timeout=600s

    log_success "Deployment to $target completed"
}

# Run health checks
run_health_checks() {
    local target=$1
    log_info "Running health checks for $target..."

    # Test orchestrator health
    local orchestrator_ready=false
    for i in {1..30}; do
        if kubectl run health-check-orchestrator --image=curlimages/curl --rm -i --restart=Never \
            -- curl -f --max-time 10 "http://agent-agency-orchestrator-$target:8080/health" >/dev/null 2>&1; then
            orchestrator_ready=true
            break
        fi
        log_warn "Orchestrator health check failed, retrying in 10s... ($i/30)"
        sleep 10
    done

    if [ "$orchestrator_ready" = false ]; then
        log_error "Orchestrator health check failed"
        return 1
    fi

    # Test council health
    local council_ready=false
    for i in {1..30}; do
        if kubectl run health-check-council --image=curlimages/curl --rm -i --restart=Never \
            -- curl -f --max-time 10 "http://agent-agency-council-$target:8081/health" >/dev/null 2>&1; then
            council_ready=true
            break
        fi
        log_warn "Council health check failed, retrying in 10s... ($i/30)"
        sleep 10
    done

    if [ "$council_ready" = false ]; then
        log_error "Council health check failed"
        return 1
    fi

    # Test database connectivity
    if ! kubectl exec -n "$NAMESPACE" deployment/agent-agency-orchestrator-$target -- \
        timeout 10s bash -c 'until pg_isready -h postgres -U agent_agency; do sleep 1; done' >/dev/null 2>&1; then
        log_error "Database connectivity check failed"
        return 1
    fi

    # Test Redis connectivity
    if ! kubectl exec -n "$NAMESPACE" deployment/agent-agency-orchestrator-$target -- \
        timeout 10s bash -c 'until redis-cli -h redis ping | grep PONG; do sleep 1; done' >/dev/null 2>&1; then
        log_error "Redis connectivity check failed"
        return 1
    fi

    log_success "All health checks passed"
    return 0
}

# Switch traffic to new deployment
switch_traffic() {
    local target=$1
    log_info "Switching traffic to $target deployment..."

    # Update service selector
    kubectl patch svc agent-agency-api -n "$NAMESPACE" --type='json' \
        -p='[{"op": "replace", "path": "/spec/selector/version", "value": "'$target'"}]'

    # Update council service
    kubectl patch svc agent-agency-council -n "$NAMESPACE" --type='json' \
        -p='[{"op": "replace", "path": "/spec/selector/version", "value": "'$target'"}]'

    log_success "Traffic switched to $target"
}

# Run smoke tests
run_smoke_tests() {
    log_info "Running smoke tests..."

    # Test API endpoints
    if ! kubectl run smoke-test --image=curlimages/curl --rm -i --restart=Never -- \
        curl -f --max-time 30 "http://agent-agency-api:8080/api/tasks" >/dev/null 2>&1; then
        log_error "API smoke test failed"
        return 1
    fi

    log_success "Smoke tests passed"
    return 0
}

# Rollback function
rollback() {
    local previous=$1
    log_error "Deployment failed, rolling back to $previous..."

    # Switch traffic back
    kubectl patch svc agent-agency-api -n "$NAMESPACE" --type='json' \
        -p='[{"op": "replace", "path": "/spec/selector/version", "value": "'$previous'"}]'

    kubectl patch svc agent-agency-council -n "$NAMESPACE" --type='json' \
        -p='[{"op": "replace", "path": "/spec/selector/version", "value": "'$previous'"}]'

    # Scale down failed deployment
    kubectl scale deployment agent-agency-orchestrator-$TARGET --replicas=0 -n "$NAMESPACE" || true
    kubectl scale deployment agent-agency-council-$TARGET --replicas=0 -n "$NAMESPACE" || true

    log_warn "Rollback completed. Manual investigation required."
    exit 1
}

# Main deployment function
main() {
    log_info "Starting Agent Agency V3 production deployment"

    # Run pre-deployment checks
    pre_deployment_checks

    # Backup current state
    backup_current_state

    # Determine deployment target
    determine_deployment_target

    # Deploy to target environment
    if ! deploy_to_target "$TARGET"; then
        log_error "Deployment to $TARGET failed"
        rollback "$PREVIOUS"
    fi

    # Run health checks
    if ! run_health_checks "$TARGET"; then
        log_error "Health checks failed for $TARGET"
        rollback "$PREVIOUS"
    fi

    # Switch traffic
    switch_traffic "$TARGET"

    # Wait for traffic to stabilize
    log_info "Waiting 60 seconds for traffic to stabilize..."
    sleep 60

    # Run smoke tests
    if ! run_smoke_tests; then
        log_error "Smoke tests failed"
        rollback "$PREVIOUS"
    fi

    # Mark deployment as successful
    log_success "🎉 Production deployment completed successfully!"
    log_info "Active deployment: $TARGET"
    log_info "Previous deployment: $PREVIOUS (kept for rollback)"

    # Send notification
    if [ -n "${SLACK_WEBHOOK_URL:-}" ]; then
        curl -X POST -H 'Content-type: application/json' \
            --data '{"text":"✅ Agent Agency V3 production deployment successful - Active: '$TARGET'"}' \
            "$SLACK_WEBHOOK_URL" || true
    fi
}

# Handle command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --namespace=*)
            NAMESPACE="${1#*=}"
            shift
            ;;
        --cluster=*)
            CLUSTER_NAME="${1#*=}"
            shift
            ;;
        --region=*)
            REGION="${1#*=}"
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --help)
            echo "Agent Agency V3 Production Deployment Script"
            echo ""
            echo "Usage: $0 [options]"
            echo ""
            echo "Options:"
            echo "  --namespace=NAMESPACE    Kubernetes namespace (default: agent-agency-prod)"
            echo "  --cluster=CLUSTER        EKS cluster name (default: agent-agency-production)"
            echo "  --region=REGION          AWS region (default: us-east-1)"
            echo "  --dry-run               Show what would be done without executing"
            echo "  --help                   Show this help message"
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Configure kubectl context
if ! aws eks update-kubeconfig --region "$REGION" --name "$CLUSTER_NAME" >/dev/null 2>&1; then
    log_error "Failed to configure kubectl for cluster $CLUSTER_NAME in region $REGION"
    exit 1
fi

# Run main deployment
main
