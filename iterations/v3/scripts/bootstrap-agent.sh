#!/usr/bin/env bash
# Agent Bootstrap Script
# @darianrosebrook
#
# Automatically detects platform/toolchains and sets up agent environment
# for concurrent work across Rust, Python, and Node/TypeScript

set -euo pipefail

# Configuration
export AGENT_ID="${AGENT_ID:-$(uuidgen | cut -c1-8)}"
export AGENT_TYPE="${AGENT_TYPE:-auto}"  # auto, rust-dev, python-test, node-build, etc.
export VERBOSE="${VERBOSE:-false}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() { echo -e "${BLUE}ℹ️  $1${NC}"; }
log_success() { echo -e "${GREEN}✅ $1${NC}"; }
log_warning() { echo -e "${YELLOW}⚠️  $1${NC}"; }
log_error() { echo -e "${RED}❌ $1${NC}"; }

if [[ "$VERBOSE" == "true" ]]; then
    log_info "Starting agent bootstrap for ID: $AGENT_ID (type: $AGENT_TYPE)"
fi

# Detect platform and architecture
detect_platform() {
    local os arch platform

    case "$(uname -s)" in
        Darwin)
            os="darwin"
            case "$(uname -m)" in
                arm64) arch="aarch64" ;;
                x86_64) arch="x86_64" ;;
                *) arch="unknown" ;;
            esac
            ;;
        Linux)
            os="linux"
            case "$(uname -m)" in
                x86_64) arch="x86_64" ;;
                aarch64) arch="aarch64" ;;
                *) arch="unknown" ;;
            esac
            ;;
        *)
            os="unknown"
            arch="unknown"
            ;;
    esac

    platform="${arch}-${os}"
    echo "$platform"
}

# Compute deterministic namespace
compute_namespace() {
    local platform toolchain_hash project_hash

    platform="$(detect_platform)"

    # Toolchain hash (language-specific)
    toolchain_hash=""
    if command -v rustc >/dev/null 2>&1; then
        toolchain_hash="rust-$(rustc -vV | grep '^release:' | sed 's/release: //')"
    fi
    if command -v python3 >/dev/null 2>&1; then
        toolchain_hash="${toolchain_hash}_python-$(python3 -c 'import sys; print(f"{sys.version_info.major}.{sys.version_info.minor}")')"
    fi
    if command -v node >/dev/null 2>&1; then
        toolchain_hash="${toolchain_hash}_node-$(node --version | sed 's/v//')"
    fi

    # Project hash (lockfiles + source)
    if [[ -f "Cargo.lock" ]]; then
        project_hash="cargo-$(sha1sum Cargo.lock 2>/dev/null | cut -c1-8 || echo 'nolock')"
    elif [[ -f "package-lock.json" ]] || [[ -f "yarn.lock" ]] || [[ -f "pnpm-lock.yaml" ]]; then
        project_hash="node-$(find . -maxdepth 1 -name "*lock*" -type f -exec sha1sum {} \; 2>/dev/null | sort | sha1sum | cut -c1-8 || echo 'nolock')"
    elif [[ -f "requirements.txt" ]] || [[ -f "pyproject.toml" ]]; then
        project_hash="python-$(find . -maxdepth 1 -name "requirements*.txt" -o -name "pyproject.toml" | xargs sha1sum 2>/dev/null | sort | sha1sum | cut -c1-8 || echo 'noreq')"
    else
        project_hash="unknown-$(find . -name "*.rs" -o -name "*.py" -o -name "*.js" -o -name "*.ts" | head -10 | xargs sha1sum 2>/dev/null | sort | sha1sum | cut -c1-8 || echo 'nosrc')"
    fi

    # Clean and combine
    toolchain_hash="${toolchain_hash#_}"  # Remove leading underscore
    echo "${platform}_${toolchain_hash}_${project_hash}_${AGENT_ID}"
}

# Setup Rust environment
setup_rust_env() {
    if [[ "$AGENT_TYPE" == "auto" ]] || [[ "$AGENT_TYPE" == *"rust"* ]]; then
        if command -v rustc >/dev/null 2>&1; then
            log_info "Setting up Rust environment..."

            # Unique target directory
            export CARGO_TARGET_DIR=".target/$BUILD_NAMESPACE"

            # Compiler cache
            if command -v sccache >/dev/null 2>&1; then
                export RUSTC_WRAPPER="$(which sccache)"
                export SCCACHE_CACHE_SIZE="${SCCACHE_CACHE_SIZE:-50G}"
                export SCCACHE_NAMESPACE="${SCCACHE_NAMESPACE:-agent-agency}"
                log_success "Rust: sccache configured"
            else
                log_warning "Rust: sccache not found - install with: cargo install sccache"
            fi

            # Fast linker configuration
            local platform="$(detect_platform)"
            case "$platform" in
                *-darwin)
                    if command -v ld64.lld >/dev/null 2>&1; then
                        export RUSTFLAGS="${RUSTFLAGS:-} -Clink-arg=-fuse-ld=/usr/local/bin/ld64.lld"
                        log_success "Rust: fast linker (ld64.lld) configured"
                    else
                        log_warning "Rust: ld64.lld not found - install with: brew install llvm"
                    fi
                    ;;
                *-linux)
                    export RUSTFLAGS="${RUSTFLAGS:-} -Clink-arg=-fuse-ld=lld"
                    log_success "Rust: fast linker (lld) configured"
                    ;;
            esac

            # Cranelift for development (if nightly)
            if [[ "$AGENT_TYPE" == *"dev"* ]] && rustc -vV | grep -q nightly; then
                export RUSTFLAGS="$RUSTFLAGS -Zcodegen-backend=cranelift"
                log_success "Rust: Cranelift backend enabled for development"
            fi

            log_success "Rust environment ready"
        else
            log_warning "Rust: rustc not found - skipping Rust setup"
        fi
    fi
}

# Setup Python environment
setup_python_env() {
    if [[ "$AGENT_TYPE" == "auto" ]] || [[ "$AGENT_TYPE" == *"python"* ]]; then
        if command -v python3 >/dev/null 2>&1; then
            log_info "Setting up Python environment..."

            # Virtual environment path
            export PYTHON_VENV_PATH=".venv/$BUILD_NAMESPACE"
            export PYTHONPATH="${PYTHON_VENV_PATH}/lib/python$(python3 -c 'import sys; print(f"{sys.version_info.major}.{sys.version_info.minor}")')/site-packages"

            # Wheel cache (shared)
            export PIP_CACHE_DIR="${PIP_CACHE_DIR:-$HOME/.cache/pip}"
            export PIP_WHEEL_CACHE="${PIP_WHEEL_CACHE:-$HOME/.cache/pip/wheels}"

            # pytest cache (per-agent)
            export PYTEST_CACHE_DIR=".pytest_cache/$BUILD_NAMESPACE"
            export MYPY_CACHE_DIR=".mypy_cache/$BUILD_NAMESPACE"

            # Ensure cache directories exist
            mkdir -p "$PIP_CACHE_DIR" "$PIP_WHEEL_CACHE" "$PYTEST_CACHE_DIR" "$MYPY_CACHE_DIR"

            log_success "Python environment configured"
            log_info "Python: venv path = $PYTHON_VENV_PATH"
            log_info "Python: cache dirs created"
        else
            log_warning "Python: python3 not found - skipping Python setup"
        fi
    fi
}

# Setup Node/TypeScript environment
setup_node_env() {
    if [[ "$AGENT_TYPE" == "auto" ]] || [[ "$AGENT_TYPE" == *"node"* ]]; then
        if command -v node >/dev/null 2>&1; then
            log_info "Setting up Node/TypeScript environment..."

            # Cache directories
            export NODE_CACHE_DIR=".cache/$BUILD_NAMESPACE"
            export TURBO_CACHE_DIR="${TURBO_CACHE_DIR:-$HOME/.cache/turbo}"
            export NX_CACHE_DIR="${NX_CACHE_DIR:-$HOME/.cache/nx}"

            # pnpm configuration (if available)
            if command -v pnpm >/dev/null 2>&1; then
                export PNPM_STORE_DIR="${PNPM_STORE_DIR:-$HOME/.cache/pnpm}"
                export PNPM_HOME="${PNPM_HOME:-$HOME/.cache/pnpm}"
                log_success "Node: pnpm configured with shared store"
            fi

            # Turbo configuration (if available)
            if command -v turbo >/dev/null 2>&1; then
                export TURBO_FORCE="false"
                export TURBO_REMOTE_CACHE_DISABLED="false"
                log_success "Node: Turbo configured"
            fi

            # Nx configuration (if available)
            if command -v nx >/dev/null 2>&1; then
                log_success "Node: Nx configured"
            fi

            # Ensure cache directories exist
            mkdir -p "$NODE_CACHE_DIR" "$TURBO_CACHE_DIR" "$NX_CACHE_DIR"

            log_success "Node/TypeScript environment configured"
        else
            log_warning "Node: node not found - skipping Node setup"
        fi
    fi
}

# Set concurrency gates based on system resources
set_concurrency_gates() {
    log_info "Setting concurrency gates..."

    local cpu_count
    cpu_count=$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)

    # Memory-aware gates
    local mem_gb
    mem_gb=$(($(sysctl -n hw.memsize 2>/dev/null || grep MemTotal /proc/meminfo | awk '{print int($2/1024/1024)}' || echo 8)))

    # Compilation: CPU-bound, scale with cores
    export COMPILE_JOBS="$cpu_count"

    # Linking: Memory-bound, conservative
    export LINK_JOBS=$(( mem_gb > 16 ? 4 : 2 ))

    # Testing: I/O-bound, scale with cores but leave headroom
    export TEST_JOBS=$(( cpu_count > 2 ? cpu_count - 1 : cpu_count ))

    # File operations: Sequential to avoid contention
    export FILE_JOBS="1"

    if [[ "$VERBOSE" == "true" ]]; then
        log_info "Concurrency: COMPILE_JOBS=$COMPILE_JOBS, LINK_JOBS=$LINK_JOBS, TEST_JOBS=$TEST_JOBS, FILE_JOBS=$FILE_JOBS"
        log_info "Resources: CPU=$cpu_count, Memory=${mem_gb}GB"
    fi
}

# Export final environment
export_environment() {
    # Core agent identity
    export BUILD_NAMESPACE="$BUILD_NAMESPACE"
    export AGENT_PLATFORM="$(detect_platform)"
    export AGENT_START_TIME="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

    # Language-specific paths
    if [[ -n "${CARGO_TARGET_DIR:-}" ]]; then
        mkdir -p "$CARGO_TARGET_DIR"
    fi
    if [[ -n "${PYTHON_VENV_PATH:-}" ]]; then
        mkdir -p "$PYTHON_VENV_PATH"
    fi
    if [[ -n "${NODE_CACHE_DIR:-}" ]]; then
        mkdir -p "$NODE_CACHE_DIR"
    fi

    log_success "Agent bootstrap complete"
    log_info "Agent ID: $AGENT_ID"
    log_info "Namespace: $BUILD_NAMESPACE"
    log_info "Platform: $AGENT_PLATFORM"

    if [[ "$VERBOSE" == "true" ]]; then
        echo ""
        echo "🔧 Environment variables set:"
        env | grep -E '^(CARGO_|PYTHON_|NODE_|TURBO_|PNPM_|SCCACHE_|COMPILE_|LINK_|TEST_|FILE_|BUILD_|AGENT_)' | sort
    fi
}

# Main execution
main() {
    log_info "🚀 Bootstrapping agent: $AGENT_ID"

    # Compute namespace first
    BUILD_NAMESPACE="$(compute_namespace)"

    # Setup language environments
    setup_rust_env
    setup_python_env
    setup_node_env

    # Set concurrency gates
    set_concurrency_gates

    # Export final environment
    export_environment

    # Verification
    if [[ "$VERBOSE" == "true" ]]; then
        echo ""
        echo "🔍 Verification:"
        echo "  Namespace deterministic: $(compute_namespace | grep -c "$BUILD_NAMESPACE" || echo "false")"
        echo "  Cache dirs exist: $(find . -maxdepth 1 -name ".target" -o -name ".cache" -o -name ".venv" | wc -l)"
    fi

    log_success "Agent $AGENT_ID is ready for concurrent work! 🎉"
}

# Run main function
main "$@"
