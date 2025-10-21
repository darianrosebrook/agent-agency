#!/usr/bin/env bash
# Distributed Cache Setup Script
#
# This script configures distributed caching for build optimizations
# across Rust, Node.js, and Python environments.
#
# @author @darianrosebrook

set -euo pipefail

# Configuration
CACHE_DIR="${CACHE_DIR:-$HOME/.cache/agent-agency}"
TURBO_CACHE_DIR="${TURBO_CACHE_DIR:-$HOME/.cache/turbo}"
SCCACHE_DIR="${SCCACHE_DIR:-$HOME/.cache/sccache}"
UV_CACHE_DIR="${UV_CACHE_DIR:-$HOME/.cache/uv}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Create cache directories
setup_directories() {
    log_info "Setting up cache directories..."

    mkdir -p "$CACHE_DIR"
    mkdir -p "$TURBO_CACHE_DIR"
    mkdir -p "$SCCACHE_DIR"
    mkdir -p "$UV_CACHE_DIR"

    # Create tmpfs mount point if it doesn't exist
    if [[ ! -d "/tmp/agent-cache" ]]; then
        sudo mkdir -p /tmp/agent-cache 2>/dev/null || true
    fi

    log_success "Cache directories created"
}

# Setup tmpfs for intermediate artifacts (if available)
setup_tmpfs() {
    log_info "Setting up tmpfs for intermediate artifacts..."

    # Check if tmpfs is available and has enough space
    if [[ -d "/tmp" ]] && df -t tmpfs /tmp >/dev/null 2>&1; then
        TMPFS_SIZE="${TMPFS_SIZE:-2G}"

        # Create tmpfs mount if not already mounted
        if ! mountpoint -q /tmp/agent-cache 2>/dev/null; then
            log_info "Creating tmpfs mount at /tmp/agent-cache (${TMPFS_SIZE})"
            sudo mount -t tmpfs -o size=${TMPFS_SIZE} tmpfs /tmp/agent-cache 2>/dev/null || {
                log_warn "Failed to create tmpfs mount, using regular directory"
                mkdir -p /tmp/agent-cache
            }
        fi

        # Link tmpfs to cache directory for faster intermediate storage
        ln -sf /tmp/agent-cache "$CACHE_DIR/tmpfs"
        log_success "tmpfs setup complete"
    else
        log_warn "tmpfs not available, skipping tmpfs setup"
    fi
}

# Configure Rust caching (sccache)
setup_rust_cache() {
    log_info "Configuring Rust distributed caching..."

    # Export environment variables for sccache
    cat > ~/.config/sccache.env << EOF
export SCCACHE_DIR="$SCCACHE_DIR"
export SCCACHE_CACHE_SIZE="50G"
export SCCACHE_IDLE_TIMEOUT="0"
export RUSTC_WRAPPER="$(which sccache 2>/dev/null || echo '')"
EOF

    # Check if sccache is installed
    if command -v sccache >/dev/null 2>&1; then
        # Configure sccache for distributed caching if available
        if [[ -n "${SCCACHE_BUCKET:-}" ]]; then
            log_info "Configuring sccache for distributed caching"
            sccache --start-server 2>/dev/null || true
        fi
        log_success "Rust caching configured"
    else
        log_warn "sccache not installed, install with: cargo install sccache"
    fi
}

# Configure Turborepo remote caching
setup_turbo_cache() {
    log_info "Configuring Turborepo remote caching..."

    # Create turbo config
    cat > turbo.json << EOF 2>/dev/null || true
{
  "globalDependencies": [
    "**/.env.*local"
  ],
  "remoteCache": {
    "enabled": true
  }
}
EOF

    # Export turbo environment variables
    cat >> ~/.config/turbo.env << EOF 2>/dev/null || echo "" > ~/.config/turbo.env
export TURBO_CACHE_DIR="$TURBO_CACHE_DIR"
export TURBO_REMOTE_CACHE_DISABLED="${TURBO_REMOTE_CACHE_DISABLED:-false}"
export TURBO_TOKEN="${TURBO_TOKEN:-}"
export TURBO_TEAM="${TURBO_TEAM:-}"
EOF

    log_success "Turborepo caching configured"
}

# Configure uv caching
setup_uv_cache() {
    log_info "Configuring uv caching..."

    # Create uv config
    mkdir -p ~/.config/uv

    cat > ~/.config/uv/uv.toml << EOF
[cache]
dir = "$UV_CACHE_DIR"

[network]
concurrency = 16
timeout = 60000
EOF

    log_success "uv caching configured"
}

# Setup cache cleanup and monitoring
setup_cache_management() {
    log_info "Setting up cache management..."

    # Create cache cleanup script
    cat > "$CACHE_DIR/cleanup.sh" << 'EOF'
#!/bin/bash
# Cache cleanup script

CACHE_DIR="$HOME/.cache/agent-agency"
TURBO_CACHE_DIR="$HOME/.cache/turbo"
SCCACHE_DIR="$HOME/.cache/sccache"
UV_CACHE_DIR="$HOME/.cache/uv"

echo "Cleaning old cache files..."

# Clean turborepo cache (keep last 7 days)
find "$TURBO_CACHE_DIR" -type f -mtime +7 -delete 2>/dev/null || true

# Clean sccache (keep under 50GB)
sccache --cleanup 2>/dev/null || true

# Clean uv cache (keep last 30 days)
find "$UV_CACHE_DIR" -type f -mtime +30 -delete 2>/dev/null || true

echo "Cache cleanup complete"
EOF

    chmod +x "$CACHE_DIR/cleanup.sh"

    # Create cache stats script
    cat > "$CACHE_DIR/stats.sh" << 'EOF'
#!/bin/bash
# Cache statistics script

echo "=== Cache Statistics ==="
echo

echo "Turbo cache:"
du -sh "$HOME/.cache/turbo" 2>/dev/null || echo "Not found"

echo
echo "sccache:"
du -sh "$HOME/.cache/sccache" 2>/dev/null || echo "Not found"
sccache --show-stats 2>/dev/null || echo "sccache stats unavailable"

echo
echo "uv cache:"
du -sh "$HOME/.cache/uv" 2>/dev/null || echo "Not found"

echo
echo "Agent cache:"
du -sh "$HOME/.cache/agent-agency" 2>/dev/null || echo "Not found"

echo
echo "tmpfs (if mounted):"
df -h /tmp/agent-cache 2>/dev/null || echo "tmpfs not mounted"
EOF

    chmod +x "$CACHE_DIR/stats.sh"

    log_success "Cache management scripts created"
}

# Create environment file for easy sourcing
create_env_file() {
    log_info "Creating environment configuration file..."

    cat > ~/.config/agent-cache.env << EOF
# Agent Agency Distributed Cache Configuration
# Source this file in your shell: source ~/.config/agent-cache.env

# Rust caching
export SCCACHE_DIR="$SCCACHE_DIR"
export SCCACHE_CACHE_SIZE="50G"
export RUSTC_WRAPPER="\$(which sccache 2>/dev/null || echo '')"

# Turborepo caching
export TURBO_CACHE_DIR="$TURBO_CACHE_DIR"
export TURBO_REMOTE_CACHE_DISABLED="${TURBO_REMOTE_CACHE_DISABLED:-false}"

# uv caching
export UV_CACHE_DIR="$UV_CACHE_DIR"

# General cache directory
export AGENT_CACHE_DIR="$CACHE_DIR"

# Add to PATH if needed
export PATH="\$PATH:$HOME/.local/bin"
EOF

    log_success "Environment configuration created at ~/.config/agent-cache.env"
}

# Main setup function
main() {
    echo "🚀 Setting up distributed caching for Agent Agency"
    echo

    setup_directories
    setup_tmpfs
    setup_rust_cache
    setup_turbo_cache
    setup_uv_cache
    setup_cache_management
    create_env_file

    echo
    log_success "Distributed caching setup complete!"
    echo
    echo "Next steps:"
    echo "1. Source the environment: source ~/.config/agent-cache.env"
    echo "2. View cache stats: $CACHE_DIR/stats.sh"
    echo "3. Clean caches: $CACHE_DIR/cleanup.sh"
    echo
    echo "For distributed caching, set these environment variables:"
    echo "- SCCACHE_BUCKET (for sccache)"
    echo "- TURBO_TOKEN and TURBO_TEAM (for Turborepo)"
}

# Run main function
main "$@"
