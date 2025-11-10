#!/bin/bash
# Verify legacy files are not imported

echo "🔍 Checking for legacy component imports..."

LEGACY_FILES=(
  "Dashboard.tsx"
  "Chat.tsx"
  "Projects.tsx"
  "ProjectView.tsx"
)

for file in "${LEGACY_FILES[@]}"; do
  component=$(basename "$file" .tsx)
  echo -n "Checking $component... "
  
  if grep -r "from.*['\"]\.\.\/components\/$component['\"]" src/ --exclude-dir=node_modules --exclude-dir=.next > /dev/null 2>&1; then
    echo "⚠️  STILL IMPORTED"
  else
    echo "✅ Not imported (safe)"
  fi
done

echo ""
echo "✅ Verification complete"
