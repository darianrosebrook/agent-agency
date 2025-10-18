#!/bin/bash

# CAWS Git Hooks Installation Script
# @author @darianrosebrook

echo "🔧 Installing CAWS Git Hooks..."

# Check if we're in a git repository
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo "❌ Not in a git repository"
    exit 1
fi

# Set the git hooks path
HOOKS_DIR=".githooks"

if [ ! -d "$HOOKS_DIR" ]; then
    echo "❌ Hooks directory not found: $HOOKS_DIR"
    exit 1
fi

# Configure git to use our hooks directory
git config core.hooksPath "$HOOKS_DIR"

echo "✅ Git hooks installed successfully!"
echo ""
echo "📋 Installed hooks:"
echo "   • pre-commit: Warns about hidden TODOs in staged files"
echo ""
echo "💡 Hook behavior:"
echo "   • Pre-commit: Warns (doesn't block) about hidden TODOs"
echo "   • Push: Blocks if hidden TODOs found (unless [skip-todo-check] in message)"
echo "   • Bypass: Use --no-verify flag for emergencies"
echo ""
echo "🔍 To test the hooks:"
echo "   1. Stage some files with TODO comments"
echo "   2. Try to commit: git commit -m 'test'"
echo "   3. The hook should warn about any hidden TODOs"
