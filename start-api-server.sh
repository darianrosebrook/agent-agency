#!/bin/bash
# Wrapper script to start API server with proper Swift runtime library paths

export DATABASE_URL="${DATABASE_URL:-postgresql://postgres:postgres@localhost:5432/agent_agency}"
export RUST_LOG="${RUST_LOG:-info}"
export DYLD_LIBRARY_PATH="/usr/lib/swift:/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift/macosx:$DYLD_LIBRARY_PATH"
export DYLD_FALLBACK_LIBRARY_PATH="/usr/lib/swift:$DYLD_FALLBACK_LIBRARY_PATH"

cd "$(dirname "$0")"
exec target/debug/agent-agency-api-server --host 127.0.0.1 --port 8080 --enable-cors "$@"

