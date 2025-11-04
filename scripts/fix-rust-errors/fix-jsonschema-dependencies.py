#!/usr/bin/env python3
"""
Fix JsonSchema issues in blocking crates:
1. Remove JsonSchema from crates that don't have schemars dependency
2. Verify WorkerType has JsonSchema properly
"""

import re
from pathlib import Path

PROJECT_ROOT = Path("/Users/darianrosebrook/Desktop/Projects/agent-agency")

def remove_json_schema_from_crate(crate_path):
    """Remove JsonSchema derives from a crate that doesn't have schemars."""
    crate_root = PROJECT_ROOT / crate_path
    
    if not crate_root.exists():
        return 0
    
    fixed_files = 0
    
    # Find all .rs files in the crate
    for rs_file in crate_root.rglob("*.rs"):
        if 'test' in str(rs_file):
            continue
        
        try:
            with open(rs_file, 'r') as f:
                content = f.read()
            
            original = content
            
            # Remove JsonSchema from derive macros
            content = re.sub(
                r',\s*JsonSchema',
                '',
                content
            )
            content = re.sub(
                r'JsonSchema\s*,',
                '',
                content
            )
            content = re.sub(
                r'JsonSchema\s+\)',
                ')',
                content
            )
            content = re.sub(
                r'#\[derive\(([^)]*)\s+JsonSchema\s*\)\]',
                r'#[derive(\1)]',
                content
            )
            
            if content != original:
                with open(rs_file, 'w') as f:
                    f.write(content)
                fixed_files += 1
                print(f"   ✅ Removed JsonSchema from {rs_file.relative_to(PROJECT_ROOT)}")
        
        except Exception as e:
            print(f"   ⚠️  Error processing {rs_file}: {e}")
    
    return fixed_files

def main():
    print("="*80)
    print("FIXING JSONSCHEMA IN BLOCKING CRATES")
    print("="*80)
    
    # Remove JsonSchema from crates without schemars dependency
    crates_to_fix = [
        "iterations/v3/system-configuration/src",
        "iterations/v3/system-common-interfaces/src",
    ]
    
    total_fixed = 0
    for crate_path in crates_to_fix:
        print(f"\nFixing {crate_path}...")
        fixed = remove_json_schema_from_crate(crate_path)
        total_fixed += fixed
        print(f"   Fixed {fixed} files")
    
    print("\n" + "="*80)
    print(f"✅ Removed JsonSchema from {total_fixed} files")
    print("="*80)
    
    # Check WorkerType issue
    print("\nChecking WorkerType JsonSchema...")
    router_file = PROJECT_ROOT / "iterations/v3/agent-agency-contracts/src/router_decision.rs"
    if router_file.exists():
        with open(router_file, 'r') as f:
            content = f.read()
        if 'JsonSchema' in content and 'pub enum WorkerType' in content:
            print("   ✅ WorkerType has JsonSchema derive")
        else:
            print("   ⚠️  WorkerType may need JsonSchema")

if __name__ == '__main__':
    main()

