#!/bin/bash

# Port Check Script for Arbiter
# Checks if required ports are available before starting the arbiter

echo "🔍 Checking arbiter port availability..."

# Check port 3000 (web interface)
if lsof -i :3000 > /dev/null 2>&1; then
    echo "⚠️  Port 3000 is in use. Consider setting WEB_OBSERVER_PORT environment variable."
    echo "   Current process using port 3000:"
    lsof -i :3000
    echo ""
else
    echo "✅ Port 3000 is available for web interface"
fi

# Check port 3001 (MCP server)
if lsof -i :3001 > /dev/null 2>&1; then
    echo "⚠️  Port 3001 is in use (MCP server)"
    lsof -i :3001
    echo ""
else
    echo "✅ Port 3001 is available for MCP server"
fi

# Check port 4387 (observer API)
if lsof -i :4387 > /dev/null 2>&1; then
    echo "⚠️  Port 4387 is in use (observer API)"
    lsof -i :4387
    echo ""
else
    echo "✅ Port 4387 is available for observer API"
fi

echo "🚀 Port check complete. You can now start the arbiter with:"
echo "   npm run dev"
echo ""
echo "💡 To use a different port for web interface:"
echo "   WEB_OBSERVER_PORT=3001 npm run dev"
