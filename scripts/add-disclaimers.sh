#!/bin/bash

# Script to add disclaimer headers to component README files

DISCLAIMER='> **⚠️ NOTICE**: This document describes proposed architecture, not current implementation.  
> **Implementation Status**: See [COMPONENT_STATUS_INDEX.md](../iterations/v2/COMPONENT_STATUS_INDEX.md) for actual status.  
> **Last Verified**: 2025-10-13  
> **Status**: Aspirational/Planning Document

---

'

# Add disclaimer to each README
for readme in "$@"; do
    if [ -f "$readme" ]; then
        # Check if disclaimer already exists
        if ! grep -q "⚠️ NOTICE" "$readme"; then
            # Create temp file with disclaimer + original content
            {
                echo "$DISCLAIMER"
                cat "$readme"
            } > "${readme}.tmp"
            
            # Replace original
            mv "${readme}.tmp" "$readme"
            echo "✅ Added disclaimer to $(basename $(dirname $readme))/README.md"
        else
            echo "⏭️  Skipped $(basename $(dirname $readme))/README.md (already has disclaimer)"
        fi
    fi
done

