#!/bin/bash
# CAWS Git Hooks Installation Script
# Installs git hooks for provenance trailer enforcement

set -e

# Get the root directory of the project
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
HOOKS_DIR="$PROJECT_ROOT/.git/hooks"

echo " Installing CAWS Git Hooks..."
echo "   Project root: $PROJECT_ROOT"
echo "   Hooks directory: $HOOKS_DIR"

# Create hooks directory if it doesn't exist
mkdir -p "$HOOKS_DIR"

# Install pre-commit hook
cat > "$HOOKS_DIR/pre-commit" << 'EOF'
#!/bin/bash
# CAWS Pre-commit Hook
# Ensures AI-assisted changes include provenance trailers

set -e

# Get the project root
PROJECT_ROOT="$(git rev-parse --show-toplevel)"

# Check if this is a CAWS project
if [ ! -d "$PROJECT_ROOT/.caws" ]; then
    echo "⚠️  Not a CAWS project, skipping provenance checks"
    exit 0
fi

echo " Checking for AI-assisted changes..."

# Check if any staged files contain AI-assisted markers
HAS_AI_CHANGES=false

# Check for Cursor AI markers in staged files
if git diff --cached --name-only | xargs -I {} sh -c '
    if git show ":{}" | grep -q "AI-assisted\|Cursor\|GitHub Copilot\|Claude\|GPT\|AI-generated"; then
        echo " Found AI-assisted content in: {}"
        HAS_AI_CHANGES=true
        exit 1
    fi
' _ {}; then
    HAS_AI_CHANGES=true
fi

# Check commit message for AI attribution
COMMIT_MSG_FILE="$1"
if [ -n "$COMMIT_MSG_FILE" ] && [ -f "$COMMIT_MSG_FILE" ]; then
    if grep -q "AI-assisted\|Cursor\|GitHub Copilot\|Claude\|GPT\|AI-generated" "$COMMIT_MSG_FILE"; then
        HAS_AI_CHANGES=true
        echo " Commit message indicates AI assistance"
    fi
fi

if [ "$HAS_AI_CHANGES" = true ]; then
    echo "⚠️  AI-assisted changes detected but no provenance trailer found!"
    echo ""
    echo "To add provenance attribution, run:"
    echo "  node apps/tools/caws/provenance.js generate"
    echo "  git add .caws/provenance.json"
    echo "  git commit --amend"
    echo ""
    echo "Or add trailer manually:"
    echo "  git commit --trailer \"Provenance: <verdict-id>\""
    echo ""
    exit 1
fi

echo " Pre-commit checks passed"
EOF

chmod +x "$HOOKS_DIR/pre-commit"

# Install commit-msg hook
cat > "$HOOKS_DIR/commit-msg" << 'EOF'
#!/bin/bash
# CAWS Commit Message Hook
# Validates and enhances commit messages with provenance trailers

set -e

# Get the project root and commit message file
PROJECT_ROOT="$(git rev-parse --show-toplevel)"
COMMIT_MSG_FILE="$1"

# Check if this is a CAWS project
if [ ! -d "$PROJECT_ROOT/.caws" ]; then
    exit 0
fi

echo " Validating commit message..."

# Read the commit message
COMMIT_MSG="$(cat "$COMMIT_MSG_FILE")"

# Check for AI-assisted markers
HAS_AI_CONTENT=false
if echo "$COMMIT_MSG" | grep -q "AI-assisted\|Cursor\|GitHub Copilot\|Claude\|GPT\|AI-generated"; then
    HAS_AI_CONTENT=true
fi

# Check staged files for AI content
if git diff --cached --name-only | xargs -I {} sh -c '
    if git show ":{}" | grep -q "AI-assisted\|Cursor\|GitHub Copilot\|Claude\|GPT\|AI-generated"; then
        HAS_AI_CONTENT=true
        exit 1
    fi
' _ {}; then
    HAS_AI_CONTENT=true
fi

# If AI content is detected, ensure provenance trailer exists
if [ "$HAS_AI_CONTENT" = true ]; then
    if ! echo "$COMMIT_MSG" | grep -q "^Provenance:"; then
        echo "⚠️  AI-assisted commit detected but missing Provenance trailer!"
        echo ""
        echo "Please add a provenance trailer to your commit message:"
        echo "  git commit --trailer \"Provenance: <verdict-id>\""
        echo ""
        echo "Or generate provenance automatically:"
        echo "  node apps/tools/caws/provenance.js generate"
        echo ""
        exit 1
    fi
fi

# Validate trailer format
if echo "$COMMIT_MSG" | grep -q "^Provenance:"; then
    # Extract trailer value
    TRAILER_VALUE=$(echo "$COMMIT_MSG" | grep "^Provenance:" | head -1 | sed 's/^Provenance: *//')

    # Validate UUID format
    if ! echo "$TRAILER_VALUE" | grep -qE '^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$'; then
        echo "⚠️  Invalid Provenance trailer format. Expected UUID, got: $TRAILER_VALUE"
        exit 1
    fi

    echo " Provenance trailer validated: $TRAILER_VALUE"
fi

echo " Commit message validation passed"
EOF

chmod +x "$HOOKS_DIR/commit-msg"

# Install post-commit hook
cat > "$HOOKS_DIR/post-commit" << 'EOF'
#!/bin/bash
# CAWS Post-commit Hook
# Records provenance information after successful commits

set -e

# Get the project root
PROJECT_ROOT="$(git rev-parse --show-toplevel)"

# Check if this is a CAWS project
if [ ! -d "$PROJECT_ROOT/.caws" ]; then
    exit 0
fi

# Get the current commit hash
COMMIT_HASH=$(git rev-parse HEAD)
COMMIT_MSG=$(git log -1 --pretty=%B)

echo " Recording commit provenance..."

# Check if commit has provenance trailer
if echo "$COMMIT_MSG" | grep -q "^Provenance:"; then
    TRAILER_VALUE=$(echo "$COMMIT_MSG" | grep "^Provenance:" | head -1 | sed 's/^Provenance: *//')

    # Update provenance record with commit hash
    if [ -f "$PROJECT_ROOT/.caws/provenance.json" ]; then
        # Update the JSON file with commit hash
        node -e "
            const fs = require('fs');
            const path = require('path');
            const provenancePath = path.join('$PROJECT_ROOT', '.caws', 'provenance.json');

            try {
                const data = JSON.parse(fs.readFileSync(provenancePath, 'utf8'));
                data.commitHash = '$COMMIT_HASH';
                data.trailer = 'Provenance: $TRAILER_VALUE';
                fs.writeFileSync(provenancePath, JSON.stringify(data, null, 2));
                console.log(' Provenance record updated with commit hash');
            } catch (err) {
                console.error('⚠️  Failed to update provenance record:', err.message);
            }
        "
    fi

    echo " Commit $COMMIT_HASH linked to provenance $TRAILER_VALUE"
else
    echo "ℹ️  No provenance trailer found in commit"
fi
EOF

chmod +x "$HOOKS_DIR/post-commit"

echo " CAWS Git hooks installed successfully!"
echo ""
echo "Installed hooks:"
echo "  - pre-commit: Checks for AI-assisted changes"
echo "  - commit-msg: Validates provenance trailers"
echo "  - post-commit: Records commit-provenance linkage"
echo ""
echo "To uninstall hooks, remove files from .git/hooks/"
echo ""
echo " Next steps:"
echo "  1. Configure your CAWS project: node apps/tools/caws/cli.js init"
echo "  2. Generate provenance: node apps/tools/caws/provenance.js generate"
echo "  3. Make AI-assisted commits with proper attribution"
