#!/usr/bin/env python3
"""
Fix JsonSchema issues in blocking crates:
- Add #[schemars(with = "String")] to Uuid and DateTime<Utc> fields
- Remove JsonSchema from error enums with non-serializable types
"""

import re
from pathlib import Path

PROJECT_ROOT = Path("/Users/darianrosebrook/Desktop/Projects/agent-agency")

def fix_json_schema_issues():
    """Fix JsonSchema trait bound errors."""
    fixes_applied = 0
    
    print("="*80)
    print("FIXING JSONSCHEMA ISSUES IN BLOCKING CRATES")
    print("="*80)
    
    # Files to fix
    files_to_fix = [
        "iterations/v3/agent-agency-contracts/src/planning.rs",
        "iterations/v3/agent-agency-contracts/src/types/planning.rs",
        "iterations/v3/agent-agency-contracts/src/worker_types.rs",
        "iterations/v3/agent-agency-contracts/src/task_executor.rs",
    ]
    
    for rel_path in files_to_fix:
        file_path = PROJECT_ROOT / rel_path
        if not file_path.exists():
            print(f"⚠️  File not found: {rel_path}")
            continue
        
        print(f"\nFixing {rel_path}...")
        
        with open(file_path, 'r') as f:
            content = f.read()
        
        original_content = content
        
        # Fix 1: Remove JsonSchema from error enums that have #[from] std::io::Error or serde_json::Error
        # Pattern: #[derive(..., JsonSchema)] on enum with #[from] non-serializable types
        lines = content.split('\n')
        modified = False
        in_error_enum = False
        has_io_error = False
        has_json_error = False
        enum_start = -1
        
        for i, line in enumerate(lines):
            # Check if we're starting an error enum
            if '#[derive(' in line and 'JsonSchema' in line and ('enum' in lines[i+1] if i+1 < len(lines) else False):
                # Check next few lines for enum definition
                for j in range(i+1, min(i+5, len(lines))):
                    if 'enum' in lines[j]:
                        in_error_enum = True
                        enum_start = j
                        break
            
            if in_error_enum:
                # Check for #[from] std::io::Error or serde_json::Error
                if '#[from] std::io::Error' in line or 'IoError(#[from] std::io::Error)' in line:
                    has_io_error = True
                if '#[from] serde_json::Error' in line or 'JsonError(#[from] serde_json::Error)' in line:
                    has_json_error = True
                
                # If we hit the closing brace, process the enum
                if '}' in line and enum_start >= 0:
                    if has_io_error or has_json_error:
                        # Remove JsonSchema from the derive
                        derive_line_idx = None
                        for k in range(enum_start - 5, enum_start):
                            if k >= 0 and '#[derive(' in lines[k] and 'JsonSchema' in lines[k]:
                                derive_line_idx = k
                                break
                        
                        if derive_line_idx is not None:
                            lines[derive_line_idx] = lines[derive_line_idx].replace(', JsonSchema', '').replace('JsonSchema,', '').replace('JsonSchema', '')
                            modified = True
                            fixes_applied += 1
                    
                    in_error_enum = False
                    has_io_error = False
                    has_json_error = False
                    enum_start = -1
        
        # Fix 2: Add #[schemars(with = "String")] to Uuid and DateTime<Utc> fields
        # Pattern: pub field_name: Uuid, -> #[schemars(with = "String")] pub field_name: Uuid,
        content = '\n'.join(lines)
        
        # Fix Uuid fields
        uuid_pattern = r'(\s+)(pub\s+(\w+):\s*Uuid,)'
        def add_uuid_schemars(match):
            indent = match.group(1)
            field_def = match.group(2)
            field_name = match.group(3)
            # Don't add if already has schemars
            if 'schemars' not in content[max(0, content.rfind(field_def, 0, content.find(field_def))-50):content.find(field_def)]:
                return f"{indent}#[schemars(with = \"String\")]\n{indent}{field_def}"
            return match.group(0)
        
        new_content = re.sub(uuid_pattern, add_uuid_schemars, content)
        
        # Fix DateTime<Utc> fields
        datetime_pattern = r'(\s+)(pub\s+(\w+):\s*DateTime<Utc>,)'
        def add_datetime_schemars(match):
            indent = match.group(1)
            field_def = match.group(2)
            field_name = match.group(3)
            if 'schemars' not in content[max(0, content.rfind(field_def, 0, content.find(field_def))-50):content.find(field_def)]:
                return f"{indent}#[schemars(with = \"String\")]\n{indent}{field_def}"
            return match.group(0)
        
        new_content = re.sub(datetime_pattern, add_datetime_schemars, new_content)
        
        if new_content != original_content:
            with open(file_path, 'w') as f:
                f.write(new_content)
            fixes_applied += 1
            print(f"   ✅ Fixed JsonSchema issues")
    
    print("\n" + "="*80)
    print(f"Applied {fixes_applied} fixes")
    print("="*80)
    
    return fixes_applied

if __name__ == '__main__':
    fixes = fix_json_schema_issues()
    print(f"\n✅ Fixed {fixes} files")

