#!/usr/bin/env python3
"""
Fix all JsonSchema issues in blocking crates:
- Add #[schemars(with = "String")] to Uuid and DateTime<Utc> fields
- Handle enum variants with Uuid/DateTime fields
"""

import re
from pathlib import Path

PROJECT_ROOT = Path("/Users/darianrosebrook/Desktop/Projects/agent-agency")

def fix_all_jsonschema_issues():
    """Fix all JsonSchema trait bound errors."""
    
    fixes = {
        'uuid_fields': 0,
        'datetime_fields': 0,
        'error_enums': 0,
    }
    
    # Fix worker_types.rs enum variants
    file_path = PROJECT_ROOT / "iterations/v3/agent-agency-contracts/src/worker_types.rs"
    if file_path.exists():
        with open(file_path, 'r') as f:
            content = f.read()
        
        original = content
        
        # Fix enum variant fields with Uuid
        # Pattern: WorkerRegistered { worker_id: Uuid,
        content = re.sub(
            r'(\s+)(\w+):\s*Uuid,',
            lambda m: f"{m.group(1)}#[schemars(with = \"String\")]\n{m.group(1)}{m.group(2)}: Uuid," if 'WorkerPoolEvent' in content[max(0, content.rfind(m.group(0), 0, content.find(m.group(0)))-200):content.find(m.group(0))] else m.group(0),
            content
        )
        
        # Fix enum variant fields with DateTime
        content = re.sub(
            r'(\s+)(\w+):\s*DateTime<Utc>,',
            lambda m: f"{m.group(1)}#[schemars(with = \"String\")]\n{m.group(1)}{m.group(2)}: DateTime<Utc>," if 'WorkerPoolEvent' in content[max(0, content.rfind(m.group(0), 0, content.find(m.group(0)))-200):content.find(m.group(0))] else m.group(0),
            content
        )
        
        if content != original:
            with open(file_path, 'w') as f:
                f.write(content)
            print(f"✅ Fixed worker_types.rs enum variants")
    
    # Fix all remaining Uuid fields in structs
    for file_path in [
        PROJECT_ROOT / "iterations/v3/agent-agency-contracts/src/types/planning.rs",
        PROJECT_ROOT / "iterations/v3/agent-agency-contracts/src/task_executor.rs",
    ]:
        if not file_path.exists():
            continue
        
        with open(file_path, 'r') as f:
            lines = f.readlines()
        
        modified = False
        for i, line in enumerate(lines):
            # Check if line has Uuid field without schemars
            if ': Uuid,' in line and 'pub ' in line:
                # Check if previous line doesn't have schemars
                if i > 0 and 'schemars' not in lines[i-1]:
                    # Add schemars annotation
                    indent = len(line) - len(line.lstrip())
                    lines.insert(i, ' ' * indent + '#[schemars(with = "String")]\n')
                    modified = True
                    fixes['uuid_fields'] += 1
            
            # Check if line has DateTime field without schemars
            if ': DateTime<Utc>,' in line and 'pub ' in line:
                if i > 0 and 'schemars' not in lines[i-1]:
                    indent = len(line) - len(line.lstrip())
                    lines.insert(i, ' ' * indent + '#[schemars(with = "String")]\n')
                    modified = True
                    fixes['datetime_fields'] += 1
        
        if modified:
            with open(file_path, 'w') as f:
                f.writelines(lines)
            print(f"✅ Fixed {file_path.name}")
    
    print(f"\nApplied fixes:")
    print(f"  Uuid fields: {fixes['uuid_fields']}")
    print(f"  DateTime fields: {fixes['datetime_fields']}")
    
    return sum(fixes.values())

if __name__ == '__main__':
    fixes = fix_all_jsonschema_issues()
    print(f"\n✅ Fixed {fixes} issues")

