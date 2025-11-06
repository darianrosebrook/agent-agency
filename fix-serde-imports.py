#!/usr/bin/env python3
"""
Automatically add missing serde imports to agent-research files.
"""

import os
import re
from pathlib import Path

def needs_serde_imports(file_path):
    """Check if a file needs serde imports by looking for derive usage without imports."""
    with open(file_path, 'r') as f:
        content = f.read()

    # Check if it uses serde derives but doesn't import serde
    has_derive = re.search(r'#\[derive\([^)]*Serialize[^)]*\)]', content) or re.search(r'#\[derive\([^)]*Deserialize[^)]*\)]', content)
    has_serde_import = 'use serde::' in content

    return has_derive and not has_serde_import

def add_serde_imports(file_path):
    """Add serde imports to a file."""
    with open(file_path, 'r') as f:
        lines = f.readlines()

    # Find where to insert imports (after existing imports)
    insert_index = 0
    for i, line in enumerate(lines):
        if line.strip().startswith('use ') or line.strip() == '':
            insert_index = i + 1
        elif not line.strip().startswith('//') and not line.strip().startswith('//!') and line.strip():
            break

    # Add the imports
    lines.insert(insert_index, 'use serde::{Deserialize, Serialize};\n')
    lines.insert(insert_index + 1, 'use schemars::JsonSchema;\n')

    # Remove duplicates if they exist
    seen_lines = set()
    deduped_lines = []
    for line in lines:
        if line not in seen_lines or not line.strip().startswith('use serde::') and not line.strip().startswith('use schemars::'):
            deduped_lines.append(line)
            if line.strip():
                seen_lines.add(line)

    with open(file_path, 'w') as f:
        f.writelines(deduped_lines)

    print(f"Added serde imports to {file_path}")

def main():
    agent_research_dir = Path('iterations/v3/agent-research/src')

    if not agent_research_dir.exists():
        print(f"Directory {agent_research_dir} not found")
        return

    fixed_count = 0
    for rs_file in agent_research_dir.rglob('*.rs'):
        if needs_serde_imports(rs_file):
            add_serde_imports(rs_file)
            fixed_count += 1

    print(f"Fixed {fixed_count} files with missing serde imports")

if __name__ == "__main__":
    main()



