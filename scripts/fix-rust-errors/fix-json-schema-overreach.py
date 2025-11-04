#!/usr/bin/env python3
"""
Fix JsonSchema derives - only add where schemars is imported
"""

import re
from pathlib import Path

PROJECT_ROOT = Path("/Users/darianrosebrook/Desktop/Projects/agent-agency")

def fix_json_schema_derives():
    """Remove JsonSchema derives from files that don't import schemars."""
    fixed = 0
    
    print("Fixing JsonSchema derives...")
    
    for file_path in PROJECT_ROOT.rglob("**/*.rs"):
        if 'test' in str(file_path) or 'target' in str(file_path):
            continue
        
        try:
            with open(file_path, 'r') as f:
                content = f.read()
            
            original_content = content
            
            # Check if schemars is imported
            has_schemars = 'use schemars' in content or 'use schemars::' in content or 'use.*schemars::JsonSchema' in content
            
            if not has_schemars:
                # Remove JsonSchema from derive macros
                content = re.sub(
                    r'#\[derive\(([^,]+),\s*JsonSchema([^)]*)\)\]',
                    r'#[derive(\1\2)]',
                    content
                )
                content = re.sub(
                    r'#\[derive\(JsonSchema(?:,\s*)?([^)]*)\)\]',
                    r'#[derive(\1)]',
                    content
                )
                
                if content != original_content:
                    with open(file_path, 'w') as f:
                        f.write(content)
                    fixed += 1
        
        except Exception as e:
            pass
    
    print(f"✅ Removed JsonSchema from {fixed} files without schemars import")
    return fixed

if __name__ == '__main__':
    fix_json_schema_derives()

