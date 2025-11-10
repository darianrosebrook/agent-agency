# Legacy Files Cleanup Guide

**Generated:** 2025-11-10  
**Author:** @darianrosebrook

## Overview

During the Tailwind to SCSS conversion, components were reorganized into feature-specific directories. Legacy files remain in the root `components/` directory but are **not actively used** by the application.

## Legacy Files Identified

The following files contain Tailwind classes but are **not imported** anywhere in the active codebase:

### Root Components Directory
- `src/components/Dashboard.tsx` - Legacy version
  - **Active version:** `src/components/dashboard/Dashboard.tsx` ✅
  - **Status:** Not imported, safe to remove

- `src/components/Chat.tsx` - Legacy version
  - **Active version:** `src/components/chat/Chat.tsx` ✅
  - **Status:** Not imported, safe to remove

- `src/components/Projects.tsx` - Legacy version
  - **Active version:** `src/components/projects/Projects.tsx` ✅
  - **Status:** Not imported, safe to remove

- `src/components/ProjectView.tsx` - Legacy version
  - **Active version:** `src/components/projects/ProjectView.tsx` ✅
  - **Status:** Not imported, safe to remove

## Verification

### Active Imports Verified
All active pages use the organized SCSS versions:

```typescript
// src/app/page.tsx
import("@/components/dashboard/Dashboard") ✅

// src/app/chat/page.tsx
import("@/components/chat/Chat") ✅

// src/app/projects/page.tsx
import("@/components/projects/Projects") ✅
```

### No Legacy Imports Found
Searched entire codebase for imports from root components:
- ✅ No imports from `@/components/Dashboard`
- ✅ No imports from `@/components/Chat`
- ✅ No imports from `@/components/Projects`
- ✅ No imports from `@/components/ProjectView`

## Cleanup Options

### Option 1: Archive Legacy Files (Recommended)
Move legacy files to an archive directory for reference:

```bash
mkdir -p src/components/_legacy
mv src/components/Dashboard.tsx src/components/_legacy/
mv src/components/Chat.tsx src/components/_legacy/
mv src/components/Projects.tsx src/components/_legacy/
mv src/components/ProjectView.tsx src/components/_legacy/
```

### Option 2: Delete Legacy Files
If you're confident they're not needed:

```bash
rm src/components/Dashboard.tsx
rm src/components/Chat.tsx
rm src/components/Projects.tsx
rm src/components/ProjectView.tsx
```

### Option 3: Keep for Reference
Keep files but add a comment header indicating they're legacy:

```typescript
/**
 * LEGACY FILE - NOT USED
 * 
 * This file is kept for reference only.
 * Active version: src/components/dashboard/Dashboard.tsx
 * 
 * @deprecated Use @/components/dashboard/Dashboard instead
 */
```

## Additional Legacy Files

These files may also be legacy but need individual verification:

- `src/components/WorkspaceTab.tsx` - Check if used
- `src/components/OverviewTab.tsx` - Check if used
- `src/components/TasksTab.tsx` - Check if used
- `src/components/TimelineTab.tsx` - Check if used
- `src/components/ManageTab.tsx` - Check if used
- `src/components/PhaseManager.tsx` - Check if used

**Note:** These may be used by the `composers/` or `projects/` directories. Verify before removing.

## Cleanup Script

Create a script to verify and clean up legacy files:

```bash
#!/bin/bash
# scripts/cleanup-legacy-files.sh

echo "Checking for legacy component files..."

LEGACY_FILES=(
  "src/components/Dashboard.tsx"
  "src/components/Chat.tsx"
  "src/components/Projects.tsx"
  "src/components/ProjectView.tsx"
)

for file in "${LEGACY_FILES[@]}"; do
  if [ -f "$file" ]; then
    # Check if file is imported anywhere
    if grep -r "from.*['\"]\.\.\/components\/$(basename $file .tsx)" src/ --exclude-dir=node_modules > /dev/null 2>&1; then
      echo "⚠️  $file is still imported - DO NOT REMOVE"
    else
      echo "✅ $file is not imported - safe to remove"
    fi
  fi
done
```

## Recommendation

**Recommended Action:** Archive legacy files rather than delete them immediately. This provides:
1. Reference for comparison
2. Safety net if something breaks
3. Historical record of migration

After a period of stability (e.g., 1-2 weeks), legacy files can be safely deleted.

## Impact Assessment

### Risk Level: Low
- ✅ No active imports found
- ✅ All pages use organized SCSS versions
- ✅ Legacy files are exact duplicates (just with Tailwind)

### Benefits of Cleanup
- ✅ Cleaner codebase
- ✅ Reduced confusion
- ✅ Smaller repository size
- ✅ Clearer component organization

## Conclusion

Legacy files in the root `components/` directory are safe to archive or remove. The application exclusively uses the organized SCSS versions from feature-specific directories.

**Status:** ✅ **Safe to Clean Up**

