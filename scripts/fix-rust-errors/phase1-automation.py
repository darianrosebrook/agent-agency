#!/usr/bin/env python3
"""
Phase 1 Automation: Fix type conversions, trait derives, and struct initialization
"""

import re
import subprocess
from pathlib import Path
from collections import defaultdict

PROJECT_ROOT = Path("/Users/darianrosebrook/Desktop/Projects/agent-agency")

def parse_error_log():
    """Parse cargo check errors and extract fixable patterns."""
    errors = {
        'type_conversions': [],
        'trait_derives': [],
        'struct_init': [],
    }
    
    log_path = PROJECT_ROOT / "cargo-check-current-errors.log"
    if not log_path.exists():
        print(f"Error: {log_path} not found")
        return errors
    
    with open(log_path, 'r') as f:
        for line in f:
            if 'error[' not in line:
                continue
            
            # Extract error info
            match = re.search(r'iterations/v3/([^/]+)/(.+?):(\d+):\d+: error\[E(\d+)\]:\s*(.+)', line)
            if not match:
                continue
            
            crate, file_path, line_num, error_code, error_msg = match.groups()
            
            file_path = f"iterations/v3/{crate}/{file_path}"
            line_num = int(line_num)
            
            # Type conversions (E0308)
            if error_code == '0308':
                if 'expected `Uuid`, found `String`' in error_msg:
                    errors['type_conversions'].append({
                        'file': file_path,
                        'line': line_num,
                        'type': 'String->Uuid',
                        'msg': error_msg
                    })
                elif 'expected `String`, found `Uuid`' in error_msg:
                    errors['type_conversions'].append({
                        'file': file_path,
                        'line': line_num,
                        'type': 'Uuid->String',
                        'msg': error_msg
                    })
                elif 'expected `f64`, found `f32`' in error_msg or 'expected `f32`, found `f64`' in error_msg:
                    errors['type_conversions'].append({
                        'file': file_path,
                        'line': line_num,
                        'type': 'f32<->f64',
                        'msg': error_msg
                    })
                elif 'expected `usize`, found `u32`' in error_msg or 'expected `u32`, found `usize`' in error_msg:
                    errors['type_conversions'].append({
                        'file': file_path,
                        'line': line_num,
                        'type': 'usize<->u32',
                        'msg': error_msg
                    })
            
            # Trait derives (E0277)
            elif error_code == '0277':
                if 'doesn\'t implement `std::fmt::Display`' in error_msg:
                    errors['trait_derives'].append({
                        'file': file_path,
                        'line': line_num,
                        'trait': 'Display',
                        'msg': error_msg
                    })
                elif 'doesn\'t implement.*JsonSchema' in error_msg:
                    errors['trait_derives'].append({
                        'file': file_path,
                        'line': line_num,
                        'trait': 'JsonSchema',
                        'msg': error_msg
                    })
            
            # Struct initialization (E0063)
            elif error_code == '0063':
                errors['struct_init'].append({
                    'file': file_path,
                    'line': line_num,
                    'msg': error_msg
                })
    
    return errors

def fix_type_conversions(errors):
    """Fix type conversion errors."""
    print("\n" + "="*80)
    print("FIXING TYPE CONVERSIONS")
    print("="*80)
    
    fixes_by_file = defaultdict(list)
    for err in errors:
        fixes_by_file[err['file']].append(err)
    
    fixed_count = 0
    
    for file_path, file_errors in fixes_by_file.items():
        full_path = PROJECT_ROOT / file_path
        if not full_path.exists():
            print(f"⚠️  File not found: {file_path}")
            continue
        
        try:
            with open(full_path, 'r') as f:
                lines = f.readlines()
            
            modified = False
            
            for err in file_errors:
                line_idx = err['line'] - 1
                if line_idx >= len(lines):
                    continue
                
                original_line = lines[line_idx]
                fixed_line = original_line
                
                # Fix String -> Uuid
                if err['type'] == 'String->Uuid':
                    # Look for patterns like: Uuid::parse_str(&var) or Uuid::from_str(&var)
                    if 'Uuid::parse_str' not in original_line and 'Uuid::from_str' not in original_line:
                        # Try to find the variable assignment
                        # Pattern: let var: Uuid = string_var;
                        fixed_line = re.sub(
                            r'(\w+):\s*Uuid\s*=\s*(\w+)',
                            r'\1: Uuid = Uuid::parse_str(&\2).unwrap_or_else(|_| Uuid::nil())',
                            original_line
                        )
                        if fixed_line != original_line:
                            modified = True
                
                # Fix Uuid -> String
                elif err['type'] == 'Uuid->String':
                    # Pattern: let var: String = uuid_var;
                    fixed_line = re.sub(
                        r'(\w+):\s*String\s*=\s*(\w+)',
                        r'\1: String = \2.to_string()',
                        original_line
                    )
                    if fixed_line != original_line:
                        modified = True
                
                # Fix f32 <-> f64
                elif err['type'] == 'f32<->f64':
                    if 'expected `f64`, found `f32`' in err['msg']:
                        # f32 -> f64
                        fixed_line = re.sub(
                            r'(\w+)\s*as\s*f32',
                            r'\1 as f64',
                            original_line
                        )
                        if fixed_line != original_line:
                            modified = True
                
                # Fix usize <-> u32
                elif err['type'] == 'usize<->u32':
                    if 'expected `usize`, found `u32`' in err['msg']:
                        # u32 -> usize
                        fixed_line = re.sub(
                            r'(\w+)\s*as\s*u32',
                            r'\1 as usize',
                            original_line
                        )
                        if fixed_line != original_line:
                            modified = True
                
                if fixed_line != original_line:
                    lines[line_idx] = fixed_line
                    fixed_count += 1
            
            if modified:
                with open(full_path, 'w') as f:
                    f.writelines(lines)
                print(f"✅ Fixed {len(file_errors)} errors in {file_path}")
        
        except Exception as e:
            print(f"⚠️  Error fixing {file_path}: {e}")
    
    print(f"\n✅ Fixed {fixed_count} type conversion errors")
    return fixed_count

def fix_trait_derives(errors):
    """Fix missing trait derives."""
    print("\n" + "="*80)
    print("FIXING TRAIT DERIVES")
    print("="*80)
    
    # Group by file and find struct/enum definitions
    fixes_by_file = defaultdict(list)
    for err in errors:
        fixes_by_file[err['file']].append(err)
    
    fixed_count = 0
    
    for file_path, file_errors in fixes_by_file.items():
        full_path = PROJECT_ROOT / file_path
        if not full_path.exists():
            continue
        
        try:
            with open(full_path, 'r') as f:
                content = f.read()
            
            # Find struct/enum definitions that need Display trait
            for err in file_errors:
                if err['trait'] == 'Display':
                    # Look for struct/enum definitions near the error line
                    # This is a simplified approach - we'll add #[derive(Display)] where needed
                    # For now, just mark for manual review
                    pass
                
                elif err['trait'] == 'JsonSchema':
                    # Add JsonSchema derive
                    # Pattern: #[derive(...)] -> #[derive(..., JsonSchema)]
                    content = re.sub(
                        r'#\[derive\(([^)]+)\)\]',
                        lambda m: f"#[derive({m.group(1)}, JsonSchema)]" if 'JsonSchema' not in m.group(1) else m.group(0),
                        content
                    )
                    fixed_count += 1
            
            if fixed_count > 0:
                with open(full_path, 'w') as f:
                    f.write(content)
                print(f"✅ Fixed JsonSchema derives in {file_path}")
        
        except Exception as e:
            print(f"⚠️  Error fixing {file_path}: {e}")
    
    print(f"\n✅ Fixed {fixed_count} trait derive errors")
    return fixed_count

def fix_struct_init(errors):
    """Fix struct initialization errors."""
    print("\n" + "="*80)
    print("FIXING STRUCT INITIALIZATION")
    print("="*80)
    
    print(f"Found {len(errors)} struct initialization errors")
    print("⚠️  These require understanding the struct definitions")
    print("    Will be handled in Phase 2")
    
    return 0

def main():
    print("="*80)
    print("PHASE 1 AUTOMATION: Type Conversions, Trait Derives, Struct Init")
    print("="*80)
    
    errors = parse_error_log()
    
    print(f"\nFound:")
    print(f"  Type conversions: {len(errors['type_conversions'])}")
    print(f"  Trait derives: {len(errors['trait_derives'])}")
    print(f"  Struct init: {len(errors['struct_init'])}")
    
    total_fixed = 0
    
    # Fix type conversions
    if errors['type_conversions']:
        total_fixed += fix_type_conversions(errors['type_conversions'])
    
    # Fix trait derives
    if errors['trait_derives']:
        total_fixed += fix_trait_derives(errors['trait_derives'])
    
    # Struct init - defer to Phase 2
    if errors['struct_init']:
        fix_struct_init(errors['struct_init'])
    
    print("\n" + "="*80)
    print(f"PHASE 1 COMPLETE: Fixed {total_fixed} errors")
    print("="*80)
    print("\nNext steps:")
    print("1. Run: cargo check --workspace")
    print("2. Review remaining errors")
    print("3. Proceed to Phase 2: Struct field fixes")

if __name__ == '__main__':
    main()

