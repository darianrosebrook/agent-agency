#!/usr/bin/env python3
"""
Add missing schemars::JsonSchema imports to files that use JsonSchema derive
"""

import re
from pathlib import Path

PROJECT_ROOT = Path("/Users/darianrosebrook/Desktop/Projects/agent-agency")

def add_json_schema_imports():
    """Add schemars import to files that need it."""
    crate_path = PROJECT_ROOT / "iterations/v3/agent-agency-contracts/src"
    
    fixed_files = 0
    
    # Find all .rs files in the crate
    for rs_file in crate_path.rglob("*.rs"):
        try:
            with open(rs_file, 'r') as f:
                content = f.read()
            
            # Check if file uses JsonSchema but doesn't import it
            if 'JsonSchema' in content and 'use schemars' not in content and 'schemars::JsonSchema' not in content:
                # Find the last use statement
                lines = content.split('\n')
                last_use_idx = -1
                
                for i, line in enumerate(lines):
                    if line.strip().startswith('use ') and not line.strip().startswith('use crate::'):
                        last_use_idx = i
                
                # Add schemars import after the last use statement
                if last_use_idx >= 0:
                    indent = len(lines[last_use_idx]) - len(lines[last_use_idx].lstrip())
                    lines.insert(last_use_idx + 1, ' ' * indent + 'use schemars::JsonSchema;')
                    
                    with open(rs_file, 'w') as f:
                        f.write('\n'.join(lines))
                    
                    fixed_files += 1
                    print(f"   ✅ Added schemars import to {rs_file.relative_to(PROJECT_ROOT)}")
        
        except Exception as e:
            print(f"   ⚠️  Error processing {rs_file}: {e}")
    
    return fixed_files

if __name__ == '__main__':
    print("Adding missing JsonSchema imports...")
    fixed = add_json_schema_imports()
    print(f"\n✅ Added imports to {fixed} files")

