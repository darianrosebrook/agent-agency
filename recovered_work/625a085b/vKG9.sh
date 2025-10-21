#!/usr/bin/env bash
# tmpfs Setup Script for Build Performance Optimization
#
# Creates memory-backed filesystems for intermediate build artifacts
# to accelerate compilation and reduce disk I/O.
#
# @author @darianrosebrook

set -euo pipefail

# Configuration
TMPFS_BASE="${TMPFS_BASE:-/tmp/agent-build}"
TMPFS_SIZE="${TMPFS_SIZE:-4G}"
FORCE="${FORCE:-false}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

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

# Check if running as root or can use sudo
check_privileges() {
    if [[ $EUID -eq 0 ]]; then
        SUDO=""
        log_info "Running as root"
    elif command -v sudo >/dev/null 2>&1; then
        SUDO="sudo"
        log_info "Will use sudo for privileged operations"
    else
        log_error "This script requires root privileges or sudo"
        exit 1
    fi
}

# Check available memory
check_memory() {
    local total_mem_kb
    total_mem_kb=$(grep MemTotal /proc/meminfo | awk '{print $2}')

    if [[ $total_mem_kb -lt 2097152 ]]; then  # Less than 2GB
        log_warn "System has less than 2GB RAM, tmpfs may not be beneficial"
    fi

    local available_mem_kb
    available_mem_kb=$(grep MemAvailable /proc/meminfo | awk '{print $2}')

    if [[ $available_mem_kb -lt 1048576 ]]; then  # Less than 1GB available
        log_warn "Less than 1GB RAM available, consider smaller tmpfs size"
    fi
}

# Setup tmpfs mount
setup_tmpfs_mount() {
    local mount_point="$1"
    local size="$2"

    log_info "Setting up tmpfs mount: $mount_point (${size})"

    # Create mount point
    $SUDO mkdir -p "$mount_point"

    # Check if already mounted
    if mountpoint -q "$mount_point" 2>/dev/null; then
        if [[ "$FORCE" == "true" ]]; then
            log_warn "Mount point already exists, unmounting and remounting"
            $SUDO umount "$mount_point"
        else
            log_info "Mount point already exists, skipping"
            return 0
        fi
    fi

    # Mount tmpfs
    if $SUDO mount -t tmpfs -o size="$size",mode=0755 tmpfs "$mount_point"; then
        log_success "tmpfs mounted at $mount_point"
    else
        log_error "Failed to mount tmpfs at $mount_point"
        return 1
    fi
}

# Setup Rust target directories on tmpfs
setup_rust_tmpfs() {
    log_info "Setting up Rust tmpfs optimization..."

    local rust_tmpfs="${TMPFS_BASE}/rust"
    setup_tmpfs_mount "$rust_tmpfs" "$TMPFS_SIZE"

    # Create symlink for cargo target directory
    local cargo_home="${CARGO_HOME:-$HOME/.cargo}"
    local target_link="${cargo_home}/tmpfs-target"

    if [[ -L "$target_link" ]]; then
        rm "$target_link"
    fi

    ln -sf "$rust_tmpfs" "$target_link"

    # Update environment
    cat >> ~/.config/tmpfs.env << EOF 2>/dev/null || echo "" > ~/.config/tmpfs.env
# Rust tmpfs configuration
export CARGO_TMPFS_TARGET="$target_link"
EOF

    log_success "Rust tmpfs setup complete"
}

# Setup Node.js cache on tmpfs
setup_nodejs_tmpfs() {
    log_info "Setting up Node.js tmpfs optimization..."

    local node_tmpfs="${TMPFS_BASE}/node"
    setup_tmpfs_mount "$node_tmpfs" "1G"

    # Create symlinks for common cache directories
    local turbo_cache="${node_tmpfs}/turbo"
    local pnpm_cache="${node_tmpfs}/pnpm"

    mkdir -p "$turbo_cache" "$pnpm_cache"

    # Symlink to standard locations
    ln -sf "$turbo_cache" "$HOME/.cache/turbo-tmpfs" 2>/dev/null || true
    ln -sf "$pnpm_cache" "$HOME/.cache/pnpm-tmpfs" 2>/dev/null || true

    # Update environment
    cat >> ~/.config/tmpfs.env << EOF
# Node.js tmpfs configuration
export TURBO_CACHE_DIR_TMPFS="$turbo_cache"
export PNPM_STORE_TMPFS="$pnpm_cache"
EOF

    log_success "Node.js tmpfs setup complete"
}

# Setup Python cache on tmpfs
setup_python_tmpfs() {
    log_info "Setting up Python tmpfs optimization..."

    local python_tmpfs="${TMPFS_BASE}/python"
    setup_tmpfs_mount "$python_tmpfs" "1G"

    # Create symlink for uv cache
    local uv_cache="${python_tmpfs}/uv"
    mkdir -p "$uv_cache"

    ln -sf "$uv_cache" "$HOME/.cache/uv-tmpfs" 2>/dev/null || true

    # Update environment
    cat >> ~/.config/tmpfs.env << EOF
# Python tmpfs configuration
export UV_CACHE_DIR_TMPFS="$uv_cache"
EOF

    log_success "Python tmpfs setup complete"
}

# Create tmpfs management script
create_management_script() {
    log_info "Creating tmpfs management script..."

    cat > ~/bin/tmpfs-manage.sh << 'EOF'
#!/bin/bash
# tmpfs Management Script

set -euo pipefail

ACTION="${1:-status}"
TMPFS_BASE="${TMPFS_BASE:-/tmp/agent-build}"

case "$ACTION" in
    "status")
        echo "=== tmpfs Status ==="
        mount | grep tmpfs | grep "$TMPFS_BASE" || echo "No tmpfs mounts found"
        echo
        echo "=== Memory Usage ==="
        df -h | grep -E "(tmpfs|$TMPFS_BASE)" || echo "No tmpfs filesystems found"
        ;;

    "cleanup")
        echo "Cleaning tmpfs contents..."
        find "$TMPFS_BASE" -mindepth 1 -delete 2>/dev/null || true
        echo "tmpfs cleanup complete"
        ;;

    "unmount")
        echo "Unmounting tmpfs filesystems..."
        mount | grep tmpfs | grep "$TMPFS_BASE" | awk '{print $3}' | xargs -r sudo umount
        echo "tmpfs unmounting complete"
        ;;

    "remount")
        echo "Remounting tmpfs filesystems..."
        "$0" unmount
        # Re-run setup script
        "$(dirname "$0")/setup-tmpfs.sh"
        ;;

    *)
        echo "Usage: $0 {status|cleanup|unmount|remount}"
        exit 1
        ;;
esac
EOF

    chmod +x ~/bin/tmpfs-manage.sh

    log_success "tmpfs management script created at ~/bin/tmpfs-manage.sh"
}

# Create systemd service for persistent tmpfs (optional)
create_systemd_service() {
    if [[ ! -d "/etc/systemd/system" ]]; then
        log_info "Systemd not available, skipping service creation"
        return 0
    fi

    log_info "Creating systemd service for persistent tmpfs..."

    cat > /tmp/agent-tmpfs.service << EOF
[Unit]
Description=Agent Build tmpfs Setup
After=local-fs.target
Before=network.target

[Service]
Type=oneshot
ExecStart=$(realpath "$0")
RemainAfterExit=yes

[Install]
WantedBy=multi-user.target
EOF

    $SUDO mv /tmp/agent-tmpfs.service /etc/systemd/system/

    log_info "To enable persistent tmpfs on boot:"
    log_info "  sudo systemctl enable agent-tmpfs.service"
}

# Main setup function
main() {
    echo "🚀 Setting up tmpfs for build performance optimization"
    echo

    check_privileges
    check_memory

    # Setup base tmpfs
    setup_tmpfs_mount "$TMPFS_BASE" "512M"

    # Setup language-specific tmpfs
    setup_rust_tmpfs
    setup_nodejs_tmpfs
    setup_python_tmpfs

    # Create management tools
    create_management_script
    create_systemd_service

    # Create environment file
    mkdir -p ~/.config
    echo "# Source this file to use tmpfs optimizations: source ~/.config/tmpfs.env" > ~/.config/tmpfs.env

    echo
    log_success "tmpfs setup complete!"
    echo
    echo "Next steps:"
    echo "1. Source environment: source ~/.config/tmpfs.env"
    echo "2. Check status: ~/bin/tmpfs-manage.sh status"
    echo "3. Clean tmpfs: ~/bin/tmpfs-manage.sh cleanup"
    echo
    echo "Note: tmpfs contents are lost on reboot unless systemd service is enabled"
}

# Run main function
main "$@"
