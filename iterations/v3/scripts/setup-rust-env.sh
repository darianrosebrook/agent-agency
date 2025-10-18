#!/usr/bin/env bash
# Rust Environment Setup
# @darianrosebrook

set -euo pipefail

setup_rust() {
    if ! command -v rustc >/dev/null 2>&1; then
        echo "Rust not found, skipping setup"
        return 0
    fi

    # Unique target directory
    export CARGO_TARGET_DIR=".target/${BUILD_NAMESPACE:-unknown}"

    # Compiler cache
    if command -v sccache >/dev/null 2>&1; then
        export RUSTC_WRAPPER="$(which sccache)"
        export SCCACHE_CACHE_SIZE="${SCCACHE_CACHE_SIZE:-50G}"
        export SCCACHE_NAMESPACE="${SCCACHE_NAMESPACE:-agent-agency}"
    fi

    # Fast linker configuration
    case "${AGENT_PLATFORM:-unknown}" in
        *-darwin)
            if command -v ld64.lld >/dev/null 2>&1; then
                export RUSTFLAGS="${RUSTFLAGS:-} -Clink-arg=-fuse-ld=/usr/local/bin/ld64.lld"
            fi
            ;;
        *-linux)
            export RUSTFLAGS="${RUSTFLAGS:-} -Clink-arg=-fuse-ld=lld"
            ;;
    esac

    # Cranelift for development (if nightly and dev agent)
    if [[ "${AGENT_TYPE:-}" == *"dev"* ]] && rustc -vV | grep -q nightly; then
        export RUSTFLAGS="$RUSTFLAGS -Zcodegen-backend=cranelift"
    fi

    # Create target directory
    mkdir -p "$CARGO_TARGET_DIR"
}

# Run setup if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    setup_rust
fi
