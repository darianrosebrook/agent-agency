#!/usr/bin/env python3
"""
Fix remaining JsonSchema import issues in newly exposed crates
Then proceed with Phase 2 struct field analysis
"""

import re
from pathlib import Path
from collections import defaultdict

PROJECT_ROOT = Path("/Users/darianrosebrook/Desktop/Projects/agent-agency")

def fix_json_schema_imports():
    """Add missing JsonSchema imports to crates that need them."""
    
    crates_to_fix = [
        "iterations/v3/agent-model-management/src",
        "iterations/v3/system-observability/src",
        "iterations/v3/agent-memory/src",
    ]
    
    fixed_files = 0
    
    for crate_path in crates_to_fix:
        crate_root = PROJECT_ROOT / crate_path
        
        if not crate_root.exists():
            continue
        
        print(f"\nFixing {crate_path}...")
        
        for rs_file in crate_root.rglob("*.rs"):
            if 'test' in str(rs_file):
                continue
            
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
            
            except Exception:
                pass
        
        print(f"   ✅ Fixed imports in {fixed_files} files")
    
    return fixed_files

def analyze_phase2_errors():
    """After fixing JsonSchema, analyze actual struct field errors."""
    print("\n" + "="*80)
    print("PHASE 2: Analyzing Struct Field Errors")
    print("="*80)
    
    import subprocess
    
    # Run cargo check and capture output
    result = subprocess.run(
        ['cargo', 'check', '--workspace'],
        capture_output=True,
        text=True,
        cwd=PROJECT_ROOT
    )
    
    # Parse errors
    field_errors = defaultdict(lambda: {'struct': '', 'field': '', 'locations': []})
    init_errors = []
    
    for line in result.stderr.split('\n'):
        # Missing field errors: E0560, E0609
        match = re.search(r'struct `([^`]+)` has no field named `([^`]+)`', line)
        if match:
            struct_type = match.group(1)
            field_name = match.group(2)
            
            # Extract location
            loc_match = re.search(r'iterations/v3/([^/]+)/(.+?):(\d+):', line)
            if loc_match:
                crate, file_path, line_num = loc_match.groups()
                key = f"{struct_type}::{field_name}"
                field_errors[key]['struct'] = struct_type
                field_errors[key]['field'] = field_name
                field_errors[key]['locations'].append({
                    'crate': crate,
                    'file': file_path,
                    'line': int(line_num)
                })
        
        # Missing fields in initialization: E0063
        if 'missing fields' in line and 'in initializer' in line:
            init_match = re.search(r'missing fields? (.+?) in initializer', line)
            if init_match:
                loc_match = re.search(r'iterations/v3/([^/]+)/(.+?):(\d+):', line)
                if loc_match:
                    init_errors.append({
                        'missing_fields': init_match.group(1),
                        'location': f"{loc_match.group(1)}/{loc_match.group(2)}:{loc_match.group(3)}"
                    })
    
    print(f"\nFound:")
    print(f"  Missing field accesses: {len(field_errors)} patterns")
    print(f"  Missing field initializations: {len(init_errors)} errors")
    
    # Group by struct
    by_struct = defaultdict(lambda: {'fields': [], 'count': 0})
    for key, info in field_errors.items():
        struct = info['struct']
        field = info['field']
        by_struct[struct]['fields'].append(field)
        by_struct[struct]['count'] += len(info['locations'])
    
    # Show top patterns
    sorted_structs = sorted(by_struct.items(), key=lambda x: x[1]['count'], reverse=True)
    
    print(f"\nTop missing field patterns:")
    print("-" * 80)
    
    for struct_type, info in sorted_structs[:15]:
        print(f"\n{struct_type}")
        print(f"  Missing fields: {', '.join(info['fields'])}")
        print(f"  Total errors: {info['count']}")
    
    return field_errors, init_errors

if __name__ == '__main__':
    print("="*80)
    print("FIXING JSONSCHEMA IMPORTS FOR PHASE 2 ANALYSIS")
    print("="*80)
    
    fixed = fix_json_schema_imports()
    print(f"\n✅ Fixed {fixed} files")
    
    # Now analyze Phase 2 errors
    field_errors, init_errors = analyze_phase2_errors()
    
    print(f"\n✅ Phase 2 analysis complete")
    print(f"   Ready to fix {sum(len(info['locations']) for info in field_errors.values())} field access errors")
    print(f"   Ready to fix {len(init_errors)} initialization errors")

