#!/usr/bin/env python3
"""
Phase 2 Automation: Fix struct field definitions
Analyzes missing field errors and creates fixes
"""

import re
import json
from pathlib import Path
from collections import defaultdict

PROJECT_ROOT = Path("/Users/darianrosebrook/Desktop/Projects/agent-agency")

def parse_missing_field_errors():
    """Parse cargo check output for missing field errors."""
    errors = defaultdict(lambda: {'struct': '', 'field': '', 'locations': []})
    
    # Run cargo check and parse errors
    import subprocess
    result = subprocess.run(
        ['cargo', 'check', '--workspace'],
        capture_output=True,
        text=True,
        cwd=PROJECT_ROOT
    )
    
    for line in result.stderr.split('\n'):
        # Match: struct `TypeName` has no field named `field_name`
        match = re.search(r'struct `([^`]+)` has no field named `([^`]+)`', line)
        if match:
            struct_type = match.group(1)
            field_name = match.group(2)
            
            # Extract location
            loc_match = re.search(r'iterations/v3/([^/]+)/(.+?):(\d+):', line)
            if loc_match:
                crate, file_path, line_num = loc_match.groups()
                key = f"{struct_type}::{field_name}"
                errors[key]['struct'] = struct_type
                errors[key]['field'] = field_name
                errors[key]['locations'].append({
                    'crate': crate,
                    'file': file_path,
                    'line': int(line_num)
                })
    
    return errors

def find_struct_definition(struct_type):
    """Find where a struct is defined."""
    # Try to find the struct definition
    struct_name = struct_type.split('::')[-1]
    
    for rs_file in PROJECT_ROOT.rglob("**/*.rs"):
        if 'test' in str(rs_file) or 'target' in str(rs_file):
            continue
        
        try:
            with open(rs_file, 'r') as f:
                content = f.read()
                
            # Look for struct definition
            pattern = rf'pub struct {struct_name}\s*\{{'
            if re.search(pattern, content):
                # Check if it matches the full type path
                if struct_type in content or struct_name in content:
                    return rs_file, content
        except Exception:
            continue
    
    return None, None

def analyze_field_patterns():
    """Analyze missing field patterns and group by struct."""
    print("="*80)
    print("PHASE 2: STRUCT FIELD FIX ANALYSIS")
    print("="*80)
    
    errors = parse_missing_field_errors()
    
    # Group by struct
    by_struct = defaultdict(lambda: {'fields': [], 'locations': []})
    
    for key, info in errors.items():
        struct = info['struct']
        field = info['field']
        by_struct[struct]['fields'].append(field)
        by_struct[struct]['locations'].extend(info['locations'])
    
    # Sort by number of errors
    sorted_structs = sorted(
        by_struct.items(),
        key=lambda x: len(x[1]['locations']),
        reverse=True
    )
    
    print(f"\nFound {len(errors)} missing field patterns across {len(by_struct)} structs\n")
    
    print("Top missing field patterns:")
    print("-" * 80)
    
    analysis_results = []
    
    for struct_type, info in sorted_structs[:20]:
        field_count = len(info['fields'])
        error_count = len(info['locations'])
        
        print(f"\n{struct_type}")
        print(f"  Missing fields ({field_count}): {', '.join(info['fields'])}")
        print(f"  Total errors: {error_count}")
        
        # Find struct definition
        def_file, def_content = find_struct_definition(struct_type)
        if def_file:
            print(f"  Definition: {def_file.relative_to(PROJECT_ROOT)}")
            analysis_results.append({
                'struct': struct_type,
                'missing_fields': info['fields'],
                'error_count': error_count,
                'definition_file': str(def_file.relative_to(PROJECT_ROOT)),
                'strategy': 'add_fields_to_struct'  # Default strategy
            })
        else:
            print(f"  Definition: NOT FOUND")
            analysis_results.append({
                'struct': struct_type,
                'missing_fields': info['fields'],
                'error_count': error_count,
                'definition_file': None,
                'strategy': 'remove_field_accesses'  # If struct not found, remove accesses
            })
    
    # Save analysis
    with open('phase2-analysis.json', 'w') as f:
        json.dump(analysis_results, f, indent=2)
    
    print(f"\n✅ Analysis saved to phase2-analysis.json")
    print(f"\nTotal errors to fix: {sum(len(info['locations']) for _, info in sorted_structs)}")
    
    return analysis_results

if __name__ == '__main__':
    results = analyze_field_patterns()

