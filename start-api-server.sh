#!/bin/bash
# Wrapper script to start API server with proper Swift runtime library paths

export DATABASE_URL="${DATABASE_URL:-postgresql://agent_agency:agent_agency_dev@127.0.0.1:5432/agent_agency?sslmode=disable}"
export RUST_LOG="${RUST_LOG:-info}"
export PATH="/opt/homebrew/bin:/usr/local/bin:$PATH"
export DYLD_LIBRARY_PATH="/usr/lib/swift:/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift/macosx:$DYLD_LIBRARY_PATH"
export DYLD_FALLBACK_LIBRARY_PATH="/usr/lib/swift:$DYLD_FALLBACK_LIBRARY_PATH"

# Function to check if Postgres is ready
check_postgres() {
    pg_isready -h 127.0.0.1 -p 5432 > /dev/null 2>&1
}

# Check if Postgres is running
if ! check_postgres; then
    echo "⚠️  PostgreSQL is not running."
    echo "🔄 Attempting to start PostgreSQL via Homebrew..."
    
    if brew services start postgresql@17; then
        echo "⏳ Waiting for PostgreSQL to become ready..."
        for i in {1..30}; do
            if check_postgres; then
                echo "✅ PostgreSQL is ready!"
                break
            fi
            sleep 1
            echo -n "."
        done
        
        if ! check_postgres; then
            echo ""
            echo "❌ Failed to start PostgreSQL. Please check logs."
            exit 1
        fi
        echo ""
    else
        echo "❌ Failed to run 'brew services start'. Is Homebrew installed?"
        exit 1
    fi
else
    echo "✅ PostgreSQL is already running."
fi

cd "$(dirname "$0")"
exec iterations/v3/target/aarch64-apple-darwin/debug/agent-agency-api-server --host 127.0.0.1 --port 8080 --enable-cors "$@"

