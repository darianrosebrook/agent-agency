#!/bin/bash

# Fix syntax errors created by sed replacements
# Run from iterations/v2 directory

set -e

echo "🔧 Fixing sed-induced syntax errors..."

TEST_DIR="tests"

# Fix the VERIFIED_TRUE regexp issue - it caught some that already had parens
echo "🔄 Fixing VERIFIED_TRUE syntax errors..."
find "$TEST_DIR" -name "*.test.ts" -type f -exec sed -i '' \
  's/VerificationVerdict\.VERIFIED_TRUE)/VerificationVerdict.VERIFIED_TRUE/g' {} \;

# Remove duplicate/wrong VerificationPriority imports from wrong modules
echo "🔄 Removing incorrect VerificationPriority imports..."

# Remove from web module imports
find "$TEST_DIR" -name "*.test.ts" -type f -exec sed -i '' \
  '/Module.*web.*has no exported member.*VerificationPriority/d' {} \;

# Fix files that import from wrong modules - just remove the bad line
find "$TEST_DIR/unit/web" -name "*.test.ts" -type f 2>/dev/null -exec sed -i '' \
  '/^  VerificationPriority,$/d' {} \; || true

find "$TEST_DIR/integration/learning" -name "*.test.ts" -type f 2>/dev/null -exec sed -i '' \
  '/^  VerificationPriority,$/d' {} \; || true

find "$TEST_DIR/unit/learning" -name "*.test.ts" -type f 2>/dev/null -exec sed -i '' \
  '/^  VerificationPriority,$/d' {} \; || true

# Fix orchestrator test - remove VerificationPriority from wrong import
echo "🔄 Fixing orchestrator imports..."
sed -i '' '/^  VerificationPriority,$/d' "$TEST_DIR/integration/orchestrator/orchestrator-verification.test.ts" 2>/dev/null || true

# Add proper VerificationPriority import to orchestrator test
sed -i '' 's/^import {$/import {\n  VerificationPriority,/' "$TEST_DIR/integration/orchestrator/orchestrator-verification.test.ts" 2>/dev/null || true

# Fix knowledge seeker test - remove VerificationPriority from wrong import
sed -i '' '/Module.*knowledge.*has no exported member.*VerificationPriority/d' "$TEST_DIR/integration/knowledge/knowledge-seeker-verification.test.ts" 2>/dev/null || true

echo "✅ Syntax error fixes complete!"
echo ""
echo "Remaining errors are structural issues that need manual fixing."


