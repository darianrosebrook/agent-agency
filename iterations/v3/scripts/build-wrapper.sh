#!/usr/bin/env bash
# Agent-oriented build wrapper for Rust projects
# @darianrosebrook
#
# This script eliminates file lock contention by giving each agent
# its own target directory and optimizes build performance.

set -euo pipefail

# Configuration
AGENT_ID="${AGENT_ID:-$(uuidgen | cut -c1-8)}"
MODE="${MODE:-dev}" # dev|test|release
PLATFORM="$(rustc -vV | sed -n 's/^host: //p')"
RUSTC_VERSION="$(rustc -vV | sed -n 's/^release: //p')"
LOCK_HASH="$(sha1sum Cargo.lock 2>/dev/null | cut -c1-8 || echo nolock)"
FEATURES="${FEATURES:-}"
VERBOSE="${VERBOSE:-false}"

# Setup compiler cache
export RUSTC_WRAPPER="$(command -v sccache 2>/dev/null || echo "")"
if [[ -n "$RUSTC_WRAPPER" ]]; then
    export SCCACHE_DIR="${SCCACHE_DIR:-$HOME/.cache/sccache}"
    export SCCACHE_NAMESPACE="${SCCACHE_NAMESPACE:-agent-agency}"
    export SCCACHE_CACHE_SIZE="${SCCACHE_CACHE_SIZE:-50G}"

    # Distributed caching support
    if [[ -n "${SCCACHE_BUCKET:-}" ]]; then
        export SCCACHE_BUCKET="$SCCACHE_BUCKET"
        export SCCACHE_REGION="${SCCACHE_REGION:-us-east-1}"
        if [[ "$VERBOSE" == "true" ]]; then
            echo "✅ Using distributed sccache with bucket: $SCCACHE_BUCKET"
        fi
    fi

    if [[ "$VERBOSE" == "true" ]]; then
        echo "✅ Using sccache: $RUSTC_WRAPPER"
        echo "   Cache dir: $SCCACHE_DIR"
        echo "   Namespace: $SCCACHE_NAMESPACE"
    fi
else
    echo "⚠️  sccache not found - install with: cargo install sccache"
fi

# Setup fast linker based on platform
case "$PLATFORM" in
    *linux*)
        export RUSTFLAGS="${RUSTFLAGS:-} -Clink-arg=-fuse-ld=lld"
        if [[ "$VERBOSE" == "true" ]]; then
            echo "✅ Using lld linker for Linux"
        fi
        ;;
    *darwin*)
        # Try ld64.lld first, fall back to system linker
        if command -v ld64.lld >/dev/null 2>&1; then
            export RUSTFLAGS="${RUSTFLAGS:-} -Clink-arg=-fuse-ld=/usr/local/bin/ld64.lld"
            if [[ "$VERBOSE" == "true" ]]; then
                echo "✅ Using ld64.lld linker for macOS"
            fi
        else
            if [[ "$VERBOSE" == "true" ]]; then
                echo "⚠️  ld64.lld not found - install with: brew install llvm"
            fi
        fi
        ;;
esac

# Create unique target directory to avoid lock contention
FEATURE_HASH="$(printf '%s' "$FEATURES" | sha1sum | cut -c1-8)"
TARGET_DIR=".target/${RUSTC_VERSION}_${PLATFORM}_${LOCK_HASH}_${FEATURE_HASH}_${AGENT_ID}"
export CARGO_TARGET_DIR="$TARGET_DIR"

# Create target directory
mkdir -p "$TARGET_DIR"

if [[ "$VERBOSE" == "true" ]]; then
    echo "🚀 Agent build configuration:"
    echo "   Agent ID: $AGENT_ID"
    echo "   Mode: $MODE"
    echo "   Platform: $PLATFORM"
    echo "   Rust version: $RUSTC_VERSION"
    echo "   Lock hash: $LOCK_HASH"
    echo "   Target dir: $TARGET_DIR"
    echo "   Features: ${FEATURES:-none}"
fi

# Setup Cranelift for dev builds (faster compilation)
if [[ "$MODE" == "dev" ]]; then
    if rustc -vV | grep -q nightly; then
        export RUSTFLAGS="$RUSTFLAGS -Zcodegen-backend=cranelift"
        if [[ "$VERBOSE" == "true" ]]; then
            echo "✅ Using Cranelift backend for faster dev builds"
        fi
    else
        if [[ "$VERBOSE" == "true" ]]; then
            echo "⚠️  Nightly toolchain required for Cranelift backend"
        fi
    fi
fi

# Execute the appropriate cargo command
case "$MODE" in
    "dev")
        if [[ "$VERBOSE" == "true" ]]; then
            echo "🔨 Running cargo build (dev mode)..."
        fi
        cargo build ${FEATURES:+--features "$FEATURES"} -j "$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)" "$@"
        ;;
    "test")
        if [[ "$VERBOSE" == "true" ]]; then
            echo "🧪 Running cargo test..."
        fi
        # Use nextest if available, otherwise fall back to cargo test
        if command -v cargo-nextest >/dev/null 2>&1; then
            cargo nextest run ${FEATURES:+--features "$FEATURES"} "$@"
        else
            cargo test ${FEATURES:+--features "$FEATURES"} "$@"
        fi
        ;;
    "release")
        if [[ "$VERBOSE" == "true" ]]; then
            echo "🚀 Running cargo build --release..."
        fi
        cargo build --release ${FEATURES:+--features "$FEATURES"} "$@"
        ;;
    "check")
        if [[ "$VERBOSE" == "true" ]]; then
            echo "✅ Running cargo check..."
        fi
        cargo check ${FEATURES:+--features "$FEATURES"} "$@"
        ;;
    *)
        echo "❌ Unknown mode: $MODE. Use dev|test|release|check"
        exit 1
        ;;
esac

if [[ "$VERBOSE" == "true" ]]; then
    echo "✅ Build completed successfully"
    echo "   Target directory: $TARGET_DIR"
fi
