#!/bin/bash

# Setup Pre-commit Hook for Quality Gates
# This script installs the quality gates as a pre-commit hook to prevent commits that violate quality standards.

HOOK_DIR=".git/hooks"
HOOK_FILE="$HOOK_DIR/pre-commit"
QUALITY_GATES_SCRIPT="scripts/quality-gates/run-quality-gates.js"

echo "🔧 Setting up quality gates pre-commit hook..."

# Check if we're in a git repository
if [ ! -d ".git" ]; then
    echo "❌ Error: Not in a git repository"
    exit 1
fi

# Create hooks directory if it doesn't exist
mkdir -p "$HOOK_DIR"

# Check if quality gates script exists
if [ ! -f "$QUALITY_GATES_SCRIPT" ]; then
    echo "❌ Error: Quality gates script not found at $QUALITY_GATES_SCRIPT"
    exit 1
fi

# Create or update pre-commit hook
cat > "$HOOK_FILE" << 'EOF'
#!/bin/bash

# CAWS Quality Gates Pre-commit Hook
# Blocks commits that violate quality standards during crisis response

echo "🚦 Running CAWS Quality Gates (Crisis Response Mode)..."

# Run quality gates
if command -v node >/dev/null 2>&1; then
    if node scripts/quality-gates/run-quality-gates.js; then
        echo "✅ Quality gates passed"
    else
        echo "❌ Quality gates failed - commit blocked"
        echo "💡 Fix the violations above before committing"
        echo "📖 See docs/refactoring.md for crisis response plan"
        exit 1
    fi
else
    echo "⚠️  Node.js not found - skipping quality gates"
    echo "💡 Install Node.js to enable automatic quality checking"
    exit 0
fi

# Run hidden TODO analysis
echo "🔍 Checking for hidden TODOs..."
if command -v python3 >/dev/null 2>&1; then
    if python3 scripts/v3/analysis/todo_analyzer.py --v3-only --ci-mode --min-confidence 0.8 >/dev/null 2>&1; then
        echo "✅ No critical hidden TODOs found"
    else
        echo "❌ Critical hidden TODOs detected - commit blocked"
        echo "💡 Fix stub implementations and placeholder code before committing"
        echo "📖 See docs/PLACEHOLDER-DETECTION-GUIDE.md for classification"
        echo ""
        echo "🔍 Running detailed analysis..."
        python3 scripts/v3/analysis/todo_analyzer.py --v3-only --min-confidence 0.8
        exit 1
    fi
else
    echo "⚠️  Python3 not found - skipping hidden TODO analysis"
    echo "💡 Install Python3 to enable automatic TODO checking"
fi

echo "✅ All quality checks passed - proceeding with commit"
exit 0
EOF

# Make hook executable
chmod +x "$HOOK_FILE"

echo "✅ Pre-commit hook installed successfully!"
echo "📍 Location: $HOOK_FILE"
echo ""
echo "🎯 What this does:"
echo "   - Runs quality gates before every commit"
echo "   - Blocks commits with naming violations, duplication, or god objects"
echo "   - Blocks commits with critical hidden TODOs (stub implementations)"
echo "   - Prevents further codebase degradation during crisis response"
echo ""
echo "🔧 To bypass temporarily (not recommended):"
echo "   git commit --no-verify"
echo ""
echo "📊 Current quality status:"
node scripts/quality-gates/run-quality-gates.js --ci


