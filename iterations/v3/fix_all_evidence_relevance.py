#!/usr/bin/env python3
"""
Fix all Evidence struct initializations by adding missing relevance field
"""

import re

file_path = "claim-extraction/src/evidence.rs"

with open(file_path, 'r') as f:
    content = f.read()

# Count how many Evidence struct initializations we have
evidence_count = content.count("Evidence {")
print(f"Found {evidence_count} Evidence struct initializations")

# Count how many already have relevance field
relevance_count = content.count("relevance:")
print(f"Found {relevance_count} existing relevance fields")

# Pattern to find Evidence struct initializations that are missing relevance field
# This looks for Evidence { ... } blocks that don't contain "relevance:"
pattern = r'Evidence \{[^}]*\}(?![^}]*relevance:)'

def add_relevance_field(match):
    evidence_block = match.group(0)
    
    # Check if relevance field already exists
    if "relevance:" in evidence_block:
        return evidence_block
    
    # Find the last field before the closing brace
    # Look for patterns like "field: value," followed by "}"
    last_field_pattern = r'(\w+:\s*[^,}]+),?\s*(\})'
    last_field_match = re.search(last_field_pattern, evidence_block)
    
    if last_field_match:
        last_field = last_field_match.group(1)
        closing_brace = last_field_match.group(2)
        
        # Add relevance field before the closing brace
        new_evidence_block = evidence_block.replace(
            f"{last_field},{closing_brace}",
            f"{last_field},\n                relevance: 0.8, // Default relevance score\n            {closing_brace}"
        )
        
        # If there was no comma after the last field, add one
        if not last_field.endswith(','):
            new_evidence_block = new_evidence_block.replace(
                f"{last_field}\n                relevance:",
                f"{last_field},\n                relevance:"
            )
        
        return new_evidence_block
    else:
        # Fallback: just add relevance field before the closing brace
        return evidence_block.replace(
            "}",
            "                relevance: 0.8, // Default relevance score\n            }"
        )

# Apply the fix
new_content = re.sub(pattern, add_relevance_field, content, flags=re.DOTALL)

# Count changes
evidence_count_after = new_content.count("Evidence {")
relevance_count_after = new_content.count("relevance:")

print(f"After fix: {evidence_count_after} Evidence structs, {relevance_count_after} relevance fields")
print(f"Added {relevance_count_after - relevance_count} relevance fields")

if new_content != content:
    with open(file_path, 'w') as f:
        f.write(new_content)
    print(f"Fixed Evidence relevance fields in {file_path}")
else:
    print(f"No changes made to {file_path}")
