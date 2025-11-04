#!/usr/bin/env python3
"""
Add serde derives to types that need Serialize/Deserialize traits
"""

import re
import subprocess
from pathlib import Path

PROJECT_ROOT = Path("/Users/darianrosebrook/Desktop/Projects/agent-agency")

def add_serde_derives():
    """Add serde Serialize/Deserialize derives to types that need them."""
    print("🔧 Adding serde derives to types needing Serialize/Deserialize...")

    fixed = 0

    # Types that commonly need serde derives
    types_to_fix = [
        'HealthCheckResult', 'ResponseMetadata', 'WorkerStatus', 'TaskStatus',
        'ExecutionStatus', 'PlanState', 'MilestoneState', 'ValidationResult'
    ]

    for file_path in PROJECT_ROOT.rglob("**/*.rs"):
        if 'test' in str(file_path) or 'target' in str(file_path):
            continue

        try:
            with open(file_path, 'r') as f:
                content = f.read()

            original_content = content

            for type_name in types_to_fix:
                # Add Serialize/Deserialize to derive macros
                pattern = rf'#\[derive\(([^)]*)\)\]\s*\n\s*(pub\s+)?(enum|struct)\s+{re.escape(type_name)}\s'
                content = re.sub(
                    pattern,
                    lambda m: f"#[derive({m.group(1)}, Serialize, Deserialize)]\n{m.group(2) or ''}{m.group(3)} {type_name} ",
                    content
                )

            if content != original_content:
                with open(file_path, 'w') as f:
                    f.write(content)
                fixed += 1
                print(f"    ✅ Added serde derives to {file_path.name}")

        except Exception as e:
            pass

    print(f"✅ Added serde derives to {fixed} files")
    return fixed

if __name__ == '__main__':
    add_serde_derives()
