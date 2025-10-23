#!/bin/bash
"""
Setup Documentation Quality Hooks

This script sets up pre-commit hooks to automatically check documentation quality
and prevent problematic content from being committed.
"""

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPT_DIR="$PROJECT_ROOT/scripts"
HOOKS_DIR="$PROJECT_ROOT/.git/hooks"

echo "🔧 Setting up documentation quality hooks..."

# Make the linter executable
chmod +x "$SCRIPT_DIR/doc-quality-linter.py"

# Create pre-commit hook
cat > "$HOOKS_DIR/pre-commit" << 'EOF'
#!/bin/bash
"""
Pre-commit hook for documentation quality checks
"""

set -e

PROJECT_ROOT="$(git rev-parse --show-toplevel)"
SCRIPT_DIR="$PROJECT_ROOT/scripts"

echo "🔍 Running documentation quality checks..."

# Run the documentation quality linter
python3 "$SCRIPT_DIR/doc-quality-linter.py" --path "$PROJECT_ROOT" --format text --exit-code

if [ $? -ne 0 ]; then
    echo ""
    echo "❌ Documentation quality issues found!"
    echo "Please fix the issues above before committing."
    echo ""
    echo "💡 To run the linter manually:"
    echo "   python3 scripts/doc-quality-linter.py"
    echo ""
    exit 1
fi

echo "✅ Documentation quality checks passed!"
EOF

# Make pre-commit hook executable
chmod +x "$HOOKS_DIR/pre-commit"

# Create pre-push hook
cat > "$HOOKS_DIR/pre-push" << 'EOF'
#!/bin/bash
"""
Pre-push hook for comprehensive documentation quality checks
"""

set -e

PROJECT_ROOT="$(git rev-parse --show-toplevel)"
SCRIPT_DIR="$PROJECT_ROOT/scripts"

echo "🔍 Running comprehensive documentation quality checks..."

# Run the documentation quality linter with stricter checks
python3 "$SCRIPT_DIR/doc-quality-linter.py" --path "$PROJECT_ROOT" --format text --exit-code

if [ $? -ne 0 ]; then
    echo ""
    echo "❌ Documentation quality issues found!"
    echo "Please fix the issues above before pushing."
    echo ""
    echo "💡 To run the linter manually:"
    echo "   python3 scripts/doc-quality-linter.py"
    echo ""
    exit 1
fi

echo "✅ Documentation quality checks passed!"
EOF

# Make pre-push hook executable
chmod +x "$HOOKS_DIR/pre-push"

echo "✅ Documentation quality hooks installed successfully!"
echo ""
echo "📋 Hooks installed:"
echo "  - pre-commit: Checks documentation quality before commits"
echo "  - pre-push: Comprehensive quality checks before pushing"
echo ""
echo "💡 To run the linter manually:"
echo "   python3 scripts/doc-quality-linter.py"
echo ""
echo "🔧 To disable hooks temporarily:"
echo "   git commit --no-verify"
echo "   git push --no-verify"
echo ""
echo "⚠️  Note: --no-verify is not recommended for production code"
