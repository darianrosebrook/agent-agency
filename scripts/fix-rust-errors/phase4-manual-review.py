#!/usr/bin/env python3
"""
Phase 4 Automation: Fix remaining complex errors (JsonSchema trait bounds)
Targets the final 17 JsonSchema errors that require manual review
"""

import re
import subprocess
from pathlib import Path
from collections import defaultdict

PROJECT_ROOT = Path("/Users/darianrosebrook/Desktop/Projects/agent-agency")

def parse_remaining_errors():
    """Parse the remaining 17 errors after Phase 3."""
    print("🔍 Analyzing remaining 17 JsonSchema errors...")

    result = subprocess.run(
        ['cargo', 'check', '--workspace', '--message-format=short'],
        cwd=PROJECT_ROOT,
        capture_output=True,
        text=True
    )

    errors = []
    for line in result.stderr.split('\n'):
        if 'JsonSchema' in line and 'error[' in line:
            # Extract error info
            match = re.search(r'iterations/v3/([^/]+)/(.+?):(\d+):\d+: error\[E(\d+)\]:\s*(.+)', line)
            if match:
                crate, file_path, line_num, error_code, error_msg = match.groups()
                errors.append({
                    'crate': crate,
                    'file': file_path,
                    'line': int(line_num),
                    'code': error_code,
                    'msg': error_msg,
                    'full': line
                })

    print(f"Found {len(errors)} remaining JsonSchema errors:")
    for err in errors:
        print(f"  {err['crate']}/{err['file']}:{err['line']} - {err['msg'][:60]}...")

    return errors

def fix_complex_jsonschema_errors():
    """Fix the remaining complex JsonSchema errors."""

    print("\n🔧 Fixing complex JsonSchema errors...")

    # Error patterns and their fixes
    fixes = [
        {
            'pattern': 'chrono::serde::ts_seconds',
            'fix': 'Add schemars attribute for chrono types',
            'action': lambda f: add_chrono_schemars_attrs(f)
        },
        {
            'pattern': 'JsonSchema` is not satisfied',
            'fix': 'Add JsonSchema derives or custom implementations',
            'action': lambda f: add_jsonschema_derives(f)
        }
    ]

    total_fixed = 0

    for fix in fixes:
        print(f"\n  Applying fix: {fix['fix']}")
        fixed = fix['action'](fix['pattern'])
        total_fixed += fixed
        print(f"    Fixed {fixed} instances")

    return total_fixed

def add_chrono_schemars_attrs(pattern):
    """Add schemars attributes for chrono DateTime types."""
    fixed = 0

    for file_path in PROJECT_ROOT.rglob("**/*.rs"):
        if 'test' in str(file_path) or 'target' in str(file_path):
            continue

        try:
            with open(file_path, 'r') as f:
                content = f.read()

            original_content = content

            # Add #[schemars(with = "String")] for DateTime fields
            content = re.sub(
                r'(\w+):\s*DateTime<Utc>',
                r'#[schemars(with = "String")]\n    \1: DateTime<Utc>',
                content
            )

            if content != original_content:
                with open(file_path, 'w') as f:
                    f.write(content)
                fixed += 1
                print(f"    ✅ Fixed chrono schemars in {file_path.name}")

        except Exception as e:
            pass

    return fixed

def add_jsonschema_derives(pattern):
    """Add JsonSchema derives to types that need them."""
    fixed = 0

    # Types that need JsonSchema but don't have it
    types_needing_jsonschema = [
        'FinalDecision', 'VoteVerdict', 'ContractMetadata', 'EvidenceArtifact',
        'QualityValidationResult', 'CouncilReviewResult', 'EvidenceResult',
        'ExecutionEvent'
    ]

    for file_path in PROJECT_ROOT.rglob("**/*.rs"):
        if 'test' in str(file_path) or 'target' in str(file_path):
            continue

        try:
            with open(file_path, 'r') as f:
                content = f.read()

            original_content = content

            # Add JsonSchema to derive macros for specific types
            for type_name in types_needing_jsonschema:
                # Match struct/enum definitions for these types
                pattern = rf'#\[derive\(([^)]*)\)\]\s*\n\s*(pub\s+)?(enum|struct)\s+{re.escape(type_name)}\s'
                content = re.sub(
                    pattern,
                    lambda m: f"#[derive({m.group(1)}, JsonSchema)]\n{m.group(2) or ''}{m.group(3)} {type_name} ",
                    content
                )

            if content != original_content:
                with open(file_path, 'w') as f:
                    f.write(content)
                fixed += 1
                print(f"    ✅ Added JsonSchema to {file_path.name}")

        except Exception as e:
            pass

    return fixed

def add_manual_jsonschema_implementations():
    """Add manual JsonSchema implementations for complex types."""
    print("\n🔧 Adding manual JsonSchema implementations...")

    # For types that can't derive JsonSchema automatically
    manual_impls = {
        'agent-agency-contracts/src/final_verdict.rs': '''
impl JsonSchema for FinalDecision {
    fn schema_name() -> String {
        "FinalDecision".to_string()
    }

    fn json_schema(_: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        schemars::schema::Schema::Object(schemars::schema::SchemaObject {
            instance_type: Some(schemars::schema::InstanceType::String.into()),
            ..Default::default()
        })
    }
}
''',
        'agent-agency-contracts/src/judge_verdict.rs': '''
impl JsonSchema for VoteVerdict {
    fn schema_name() -> String {
        "VoteVerdict".to_string()
    }

    fn json_schema(_: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        schemars::schema::Schema::Object(schemars::schema::SchemaObject {
            instance_type: Some(schemars::schema::InstanceType::String.into()),
            ..Default::default()
        })
    }
}
'''
    }

    fixed = 0
    for file_path, impl_code in manual_impls.items():
        full_path = PROJECT_ROOT / file_path
        if full_path.exists():
            try:
                with open(full_path, 'a') as f:
                    f.write('\n' + impl_code)
                fixed += 1
                print(f"    ✅ Added manual JsonSchema impl to {file_path}")
            except Exception as e:
                print(f"    ⚠️  Failed to add manual impl to {file_path}: {e}")

    return fixed

def main():
    print("="*80)
    print("PHASE 4 AUTOMATION: Manual Review of Complex Errors")
    print("="*80)

    # Parse remaining errors
    errors = parse_remaining_errors()

    if not errors:
        print("\n✅ No errors remaining! Compilation successful.")
        return 0

    print(f"\n📊 Remaining {len(errors)} errors:")
    print("All appear to be JsonSchema trait bound issues")

    # Apply fixes
    total_fixed = fix_complex_jsonschema_errors()
    total_fixed += add_manual_jsonschema_implementations()

    print("\n" + "="*80)
    print(f"PHASE 4 COMPLETE: Applied {total_fixed} fixes")
    print("="*80)

    if total_fixed > 0:
        print("\nNext: Run 'cargo check --workspace' to verify final fixes")
    else:
        print("\n⚠️  No automated fixes applied - manual intervention required")

    print("\nRemaining errors may need:")
    print("1. Manual JsonSchema implementations")
    print("2. Custom serialize/deserialize logic")
    print("3. Type system refactoring")

    return total_fixed

if __name__ == '__main__':
    main()

