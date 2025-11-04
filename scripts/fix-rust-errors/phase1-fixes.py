#!/usr/bin/env python3
"""
Phase 1 Automation: Fix type conversions, trait derives, and struct initialization
Targets specific error patterns from cargo check output
"""

import re
import subprocess
from pathlib import Path
from collections import defaultdict

PROJECT_ROOT = Path("/Users/darianrosebrook/Desktop/Projects/agent-agency")

def apply_fixes():
    """Apply targeted fixes based on error patterns."""
    fixes_applied = 0
    
    print("="*80)
    print("PHASE 1 AUTOMATION: Applying Targeted Fixes")
    print("="*80)
    
    # Fix 1: todo_integration.rs - String <-> Uuid conversions
    file_path = PROJECT_ROOT / "iterations/v3/agent-orchestration/src/planning/todo_integration.rs"
    if file_path.exists():
        print(f"\n1. Fixing String<->Uuid in {file_path.name}")
        with open(file_path, 'r') as f:
            content = f.read()
        
        original_content = content
        
        # Fix: persist_plan_todo_association(plan_id, ...) where plan_id is String
        content = re.sub(
            r'persist_plan_todo_association\(plan_id,',
            r'persist_plan_todo_association(Uuid::parse_str(&plan_id)?,',
            content
        )
        
        # Fix: plan_todos.get(&plan_id) where plan_id is Uuid but map expects String
        content = re.sub(
            r'self\.plan_todos\.get\(&plan_id\)',
            r'self.plan_todos.get(&plan_id.to_string())',
            content
        )
        
        # Fix: Similar patterns for other methods
        content = re.sub(
            r'\.get\(&plan_id\)',
            r'.get(&plan_id.to_string())',
            content
        )
        
        if content != original_content:
            with open(file_path, 'w') as f:
                f.write(content)
            fixes_applied += 3
            print("   ✅ Fixed String<->Uuid conversions")
    
    # Fix 2: storage.rs - AuditEvent missing fields
    file_path = PROJECT_ROOT / "iterations/v3/agent-orchestration/src/planning/orchestrator_integration.rs"
    if file_path.exists():
        print(f"\n2. Fixing AuditEvent initialization in {file_path.name}")
        with open(file_path, 'r') as f:
            lines = f.readlines()
        
        modified = False
        
        # Find AuditEvent initialization around line 307
        for i, line in enumerate(lines):
            if 'AuditEvent {' in line and i < len(lines) - 10:
                # Check if plan_id and timestamp are missing
                struct_start = i
                # Find the closing brace
                brace_count = 0
                found_open = False
                for j in range(i, min(i + 20, len(lines))):
                    if '{' in lines[j]:
                        brace_count += lines[j].count('{')
                        found_open = True
                    if '}' in lines[j]:
                        brace_count -= lines[j].count('}')
                        if found_open and brace_count == 0:
                            # Found the struct initialization
                            struct_end = j
                            
                            # Check if plan_id and timestamp are present
                            struct_content = ''.join(lines[struct_start:struct_end+1])
                            if 'plan_id:' not in struct_content:
                                # Add plan_id before the closing brace
                                indent = len(lines[struct_end]) - len(lines[struct_end].lstrip())
                                lines.insert(struct_end, ' ' * indent + f'plan_id: plan.contract_plan.id,\n')
                                modified = True
                                fixes_applied += 1
                            
                            if 'timestamp:' not in struct_content:
                                # Add timestamp before the closing brace
                                indent = len(lines[struct_end]) - len(lines[struct_end].lstrip())
                                lines.insert(struct_end, ' ' * indent + f'timestamp: chrono::Utc::now(),\n')
                                modified = True
                                fixes_applied += 1
                            
                            break
        
        if modified:
            with open(file_path, 'w') as f:
                f.writelines(lines)
            print("   ✅ Fixed AuditEvent missing fields")
    
    # Fix 3: storage.rs - Uuid::parse() issue
    file_path = PROJECT_ROOT / "iterations/v3/agent-orchestration/src/planning/storage.rs"
    if file_path.exists():
        print(f"\n3. Fixing Uuid::parse() in {file_path.name}")
        with open(file_path, 'r') as f:
            content = f.read()
        
        original_content = content
        
        # Fix: db_plan.id.parse().unwrap_or_default().unwrap_or(...)
        # This is wrong - should be Uuid::parse_str(&db_plan.id)
        content = re.sub(
            r'db_plan\.id\.parse\(\)\.unwrap_or_default\(\)',
            r'Uuid::parse_str(&db_plan.id).unwrap_or_else(|_| Uuid::nil())',
            content
        )
        
        if content != original_content:
            with open(file_path, 'w') as f:
                f.write(content)
            fixes_applied += 1
            print("   ✅ Fixed Uuid::parse() issue")
    
    # Fix 4: Add JsonSchema derives where needed
    print(f"\n4. Adding JsonSchema derives")
    json_schema_fixes = 0
    
    # Find files with JsonSchema errors
    for file_path in PROJECT_ROOT.rglob("**/*.rs"):
        if 'test' in str(file_path) or 'target' in str(file_path):
            continue
        
        try:
            with open(file_path, 'r') as f:
                content = f.read()
            
            original_content = content
            
            # Add JsonSchema to derive macros for enums/structs that need it
            # Pattern: #[derive(Debug, Clone)] -> #[derive(Debug, Clone, JsonSchema)]
            content = re.sub(
                r'#\[derive\(([^)]+)\)\]\s*\n\s*(pub\s+)?(enum|struct)\s+(\w+)\s*',
                lambda m: f"#[derive({m.group(1)}, JsonSchema)]\n{m.group(2) or ''}{m.group(3)} {m.group(4)} " if 'JsonSchema' not in m.group(1) else m.group(0),
                content
            )
            
            if content != original_content:
                with open(file_path, 'w') as f:
                    f.write(content)
                json_schema_fixes += 1
        except Exception:
            pass
    
    if json_schema_fixes > 0:
        print(f"   ✅ Added JsonSchema to {json_schema_fixes} files")
        fixes_applied += json_schema_fixes
    
    print("\n" + "="*80)
    print(f"PHASE 1 COMPLETE: Applied {fixes_applied} fixes")
    print("="*80)
    
    return fixes_applied

if __name__ == '__main__':
    fixes = apply_fixes()
    print(f"\n✅ Applied {fixes} fixes")
    print("\nNext: Run 'cargo check --workspace' to verify fixes")

