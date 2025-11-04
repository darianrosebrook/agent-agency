#!/usr/bin/env python3
"""
Analyze missing struct/enum field errors and generate fix suggestions.
This analyzes the error patterns to understand what fields are missing where.
"""

import re
import json
from collections import defaultdict
from pathlib import Path

def parse_error_log(log_path):
    """Parse cargo check error log and extract missing field errors."""
    errors = []
    
    with open(log_path, 'r') as f:
        for line in f:
            if 'error[' not in line:
                continue
            
            # Extract error info
            match = re.search(r'iterations/v3/([^/]+)/(.+?):(\d+):\d+: error\[E(\d+)\]:\s*(.+)', line)
            if not match:
                continue
            
            crate, file_path, line_num, error_code, error_msg = match.groups()
            
            if error_code in ('0560', '0609'):
                # Missing field error
                field_match = re.search(r'has no field named `([^`]+)`', error_msg)
                struct_match = re.search(r'on type `([^`]+)`', error_msg)
                
                if field_match and struct_match:
                    errors.append({
                        'crate': crate,
                        'file': file_path,
                        'line': int(line_num),
                        'error_code': error_code,
                        'missing_field': field_match.group(1),
                        'struct_type': struct_match.group(1),
                        'full_message': error_msg
                    })
    
    return errors

def group_by_pattern(errors):
    """Group errors by missing field pattern."""
    patterns = defaultdict(list)
    
    for err in errors:
        key = f"{err['struct_type']}::missing::{err['missing_field']}"
        patterns[key].append(err)
    
    return patterns

def main():
    log_path = Path('cargo-check-current-errors.log')
    if not log_path.exists():
        print(f"Error: {log_path} not found")
        return
    
    print("Analyzing missing field errors...")
    errors = parse_error_log(log_path)
    
    print(f"\nFound {len(errors)} missing field errors")
    
    # Group by pattern
    patterns = group_by_pattern(errors)
    
    # Show top patterns
    print("\nTop missing field patterns:")
    print("=" * 80)
    
    sorted_patterns = sorted(patterns.items(), key=lambda x: len(x[1]), reverse=True)
    
    for pattern_key, pattern_errors in sorted_patterns[:20]:
        struct_type = pattern_errors[0]['struct_type']
        missing_field = pattern_errors[0]['missing_field']
        count = len(pattern_errors)
        
        print(f"\n{count:3d}x Missing field '{missing_field}' in {struct_type}")
        
        # Show sample locations
        sample_files = defaultdict(int)
        for err in pattern_errors[:10]:
            sample_files[f"{err['crate']}/{err['file']}"] += 1
        
        for file_path, file_count in list(sample_files.items())[:3]:
            print(f"      → {file_path} ({file_count} occurrences)")
    
    # Generate fix suggestions
    print("\n" + "=" * 80)
    print("FIX STRATEGY:")
    print("=" * 80)
    
    # Count by crate
    by_crate = defaultdict(int)
    for err in errors:
        by_crate[err['crate']] += 1
    
    print("\nMissing field errors by crate:")
    for crate, count in sorted(by_crate.items(), key=lambda x: x[1], reverse=True):
        print(f"  {crate:30s}: {count:4d} errors")
    
    # Save analysis
    analysis = {
        'total_errors': len(errors),
        'unique_patterns': len(patterns),
        'by_crate': dict(by_crate),
        'top_patterns': [
            {
                'struct_type': errs[0]['struct_type'],
                'missing_field': errs[0]['missing_field'],
                'count': len(errs),
                'sample_locations': [f"{e['crate']}/{e['file']}:{e['line']}" for e in errs[:5]]
            }
            for _, errs in sorted_patterns[:30]
        ]
    }
    
    with open('missing-fields-analysis.json', 'w') as f:
        json.dump(analysis, f, indent=2)
    
    print(f"\n✅ Analysis saved to missing-fields-analysis.json")

if __name__ == '__main__':
    main()

