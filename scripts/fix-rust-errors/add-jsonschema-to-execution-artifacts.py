#!/usr/bin/env python3
"""
Add JsonSchema to all structs in execution_artifacts.rs that are missing it
"""

import re
from pathlib import Path

PROJECT_ROOT = Path("/Users/darianrosebrook/Desktop/Projects/agent-agency")
file_path = PROJECT_ROOT / "iterations/v3/agent-agency-contracts/src/execution_artifacts.rs"

with open(file_path, 'r') as f:
    content = f.read()

# List of structs that need JsonSchema
structs_needing_schema = [
    'Provenance',
    'ArtifactMetadata', 
    'DiffArtifact',
    'NewFileArtifact',
    'CodeChangeStats',
    'TestSuiteResults',
    'E2eTestResults',
]

fixed_count = 0

for struct_name in structs_needing_schema:
    # Pattern: #[derive(...)] pub struct StructName
    pattern = rf'#\[derive\(([^)]+)\)\]\s*\n\s*(?:#\[serde[^\]]+\]\s*\n\s*)?pub struct {struct_name}'
    
    def add_schema(match):
        global fixed_count
        derives = match.group(1)
        if 'JsonSchema' not in derives:
            # Add JsonSchema to derives
            new_derives = f"{derives}, JsonSchema"
            fixed_count += 1
            return f"#[derive({new_derives})]"
        return match.group(0)
    
    content = re.sub(pattern, add_schema, content)

# Also fix enum ChangeType
content = re.sub(
    r'#\[derive\(([^)]+)\)\]\s*\n\s*#\[serde[^\]]+\]\s*\n\s*pub enum ChangeType',
    lambda m: f"#[derive({m.group(1)}, JsonSchema)]\n    #[serde(rename_all = \"snake_case\")]" if 'JsonSchema' not in m.group(1) else m.group(0),
    content
)

# Fix DiffHunk and other structs
structs = ['DiffHunk']
for struct_name in structs:
    pattern = rf'#\[derive\(([^)]+)\)\]\s*\n\s*(?:#\[serde[^\]]+\]\s*\n\s*)?pub struct {struct_name}'
    content = re.sub(
        pattern,
        lambda m: f"#[derive({m.group(1)}, JsonSchema)]" if 'JsonSchema' not in m.group(1) else m.group(0),
        content
    )
    if pattern in content:
        fixed_count += 1

with open(file_path, 'w') as f:
    f.write(content)

print(f"✅ Added JsonSchema to {fixed_count} structs in execution_artifacts.rs")

