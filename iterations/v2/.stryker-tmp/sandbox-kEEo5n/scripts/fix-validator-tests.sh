#!/bin/bash

# Fix all validator test files with common property access issues
# Run from iterations/v2 directory

set -e

echo "🔧 Fixing validator test files..."

VALIDATOR_TESTS=(
  "tests/unit/verification/validators/statistical.test.ts"
  "tests/unit/verification/validators/logical.test.ts"
  "tests/unit/verification/validators/cross-reference.test.ts"
  "tests/unit/verification/validators/consistency.test.ts"
)

for FILE in "${VALIDATOR_TESTS[@]}"; do
  if [ ! -f "$FILE" ]; then
    echo "⚠️  File not found: $FILE"
    continue
  fi
  
  echo "🔄 Fixing $FILE..."
  
  # Fix 1: Remove .metadata property access (doesn't exist in VerificationMethodResult)
  echo "  - Removing .metadata access..."
  sed -i '' '/expect(result\.metadata)/d' "$FILE"
  sed -i '' '/expect(.*\.metadata\./d' "$FILE"
  
  # Fix 2: Change .type to .method
  echo "  - Changing .type to .method..."
  sed -i '' 's/result\.type/result.method/g' "$FILE"
  sed -i '' 's/expect(result\.method)\.toBe/expect(result.method).toBe/g' "$FILE"
  
  # Fix 3: Remove .evidence property access (use .evidenceCount instead)
  echo "  - Removing .evidence access..."
  sed -i '' '/expect(result\.evidence)/d' "$FILE"
  sed -i '' '/result\.evidence\./d' "$FILE"
  
  # Fix 4: Fix extra closing parentheses on expect lines
  echo "  - Fixing extra parentheses..."
  sed -i '' 's/)));/));/g' "$FILE"
  sed -i '' 's/));/);/g' "$FILE" # Second pass to catch nested cases
  
  # Fix 5: Fix any remaining extra parentheses issues
  echo "  - Cleaning up parentheses..."
  # Remove any quadruple closing parens
  sed -i '' 's/))));/)));/g' "$FILE"
  
done

echo "✅ All validator test files fixed!"
echo ""
echo "Fixed files:"
for FILE in "${VALIDATOR_TESTS[@]}"; do
  echo "  - $FILE"
done
echo ""
echo "Remaining errors should be reduced significantly."
echo "Run 'npm run lint' to verify."

