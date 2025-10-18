#!/usr/bin/env bash
set -euo pipefail

# Agent-oriented cargo wrapper to eliminate build lock contention
# Usage: ./scripts/cargo-agent-wrapper.sh [dev|test|release] [cargo args...]

AGENT_ID="${AGENT_ID:-$(uuidgen | cut -c1-8)}"
MODE="${1:-dev}"
shift || true

# Detect platform and toolchain info
PLATFORM="$(rustc -vV | sed -n 's/^host: //p')"
RUSTC_VERSION="$(rustc -vV | sed -n 's/^release: //p')"
LOCK_HASH="$(sha1sum Cargo.lock 2>/dev/null | cut -c1-8 || echo nolock)"
FEATURES="${FEATURES:-}"

# Set up compiler cache
export RUSTC_WRAPPER="$(command -v sccache 2>/dev/null || echo)"
export SCCACHE_DIR="${SCCACHE_DIR:-$HOME/.cache/sccache}"
export SCCACHE_NAMESPACE="${SCCACHE_NAMESPACE:-agent-agency-v3}"

# Configure fast linkers per platform
case "$PLATFORM" in
  *linux*) 
    export RUSTFLAGS="${RUSTFLAGS:-} -Clink-arg=-fuse-ld=lld"
    ;;
  *darwin*) 
    # Try ld64.lld first, fallback to system linker
    if command -v ld64.lld >/dev/null 2>&1; then
      export RUSTFLAGS="${RUSTFLAGS:-} -Clink-arg=-fuse-ld=ld64.lld"
    elif command -v zld >/dev/null 2>&1; then
      export RUSTFLAGS="${RUSTFLAGS:-} -Clink-arg=-fuse-ld=zld"
    fi
    ;;
esac

# Create unique target directory to avoid lock contention
TARGET_DIR=".target/${RUSTC_VERSION}_${PLATFORM}_${LOCK_HASH}_${AGENT_ID}_${MODE}"
export CARGO_TARGET_DIR="$TARGET_DIR"

echo "Agent $AGENT_ID building in $TARGET_DIR (mode: $MODE)"

# Configure build mode
case "$MODE" in
  dev)
    # Use Cranelift for faster dev builds if available
    if rustc -vV | grep -q nightly && command -v rustc_codegen_cranelift >/dev/null 2>&1; then
      export RUSTFLAGS="$RUSTFLAGS -Zcodegen-backend=cranelift"
      echo "Using Cranelift backend for faster dev builds"
    fi
    cargo build ${FEATURES:+--features "$FEATURES"} -j "$(nproc 2>/dev/null || sysctl -n hw.ncpu)" "$@"
    ;;
  test)
    # Use nextest if available, otherwise regular test
    if command -v cargo-nextest >/dev/null 2>&1; then
      cargo nextest run ${FEATURES:+--features "$FEATURES"} "$@"
    else
      cargo test ${FEATURES:+--features "$FEATURES"} "$@"
    fi
    ;;
  release)
    cargo build --release ${FEATURES:+--features "$FEATURES"} "$@"
    ;;
  check)
    cargo check ${FEATURES:+--features "$FEATURES"} "$@"
    ;;
  *)
    echo "Unknown mode: $MODE. Use dev|test|release|check"
    exit 1
    ;;
esac
