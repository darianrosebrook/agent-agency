#!/usr/bin/env python3
"""
Remove JsonSchema from derive macros in agent-research.
"""

import os
import re
from pathlib import Path

def remove_jsonschema_from_file(filepath):
    """Remove JsonSchema from derive macros in a file."""
    with open(filepath, 'r') as f:
        content = f.read()

    # Pattern to match #[derive(...JsonSchema...)] and remove JsonSchema
    pattern = r'#\[derive\(([^)]*?)JsonSchema([^)]*?)\)\]'
    replacement = r'#[derive(\1\2)]'

    # Remove JsonSchema from the derive
    new_content = re.sub(pattern, replacement, content)

    # Also handle cases where JsonSchema is the only thing or at the end
    new_content = re.sub(r'#\[derive\(([^)]*), JsonSchema([^)]*)\)\]', r'#[derive(\1\2)]', new_content)
    new_content = re.sub(r'#\[derive\(([^)]*)JsonSchema, ([^)]*)\)\]', r'#[derive(\1\2)]', new_content)
    new_content = re.sub(r'#\[derive\(([^)]*), JsonSchema\)\]', r'#[derive(\1)]', new_content)
    new_content = re.sub(r'#\[derive\(JsonSchema\)\]', '', new_content)

    if new_content != content:
        with open(filepath, 'w') as f:
            f.write(new_content)
        print(f"Updated {filepath}")
        return True
    return False

def main():
    agent_research_dir = Path('iterations/v3/agent-research/src')

    if not agent_research_dir.exists():
        print(f"Directory {agent_research_dir} not found")
        return

    updated_count = 0
    for rs_file in agent_research_dir.rglob('*.rs'):
        if remove_jsonschema_from_file(rs_file):
            updated_count += 1

    print(f"Updated {updated_count} files by removing JsonSchema from derive macros")

if __name__ == "__main__":
    main()
