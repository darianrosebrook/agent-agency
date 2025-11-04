#!/usr/bin/env python3
import os
import re

def fix_duplicate_schemars(file_path):
    """Fix duplicate schemars attributes in a file"""
    with open(file_path, 'r') as f:
        content = f.read()
    
    # Pattern to match duplicate consecutive schemars attributes
    pattern = r'(\s*#\s*\[schemars[^\]]*\]\s*\n)\s*#\s*\[schemars[^\]]*\]'
    
    # Replace with single occurrence
    fixed_content = re.sub(pattern, r'\1', content, flags=re.MULTILINE)
    
    if fixed_content != content:
        with open(file_path, 'w') as f:
            f.write(fixed_content)
        print(f"Fixed duplicates in {file_path}")
        return True
    return False

# Process all files in system-federated-ml/src
fixed_count = 0
for root, dirs, files in os.walk('system-federated-ml/src'):
    for file in files:
        if file.endswith('.rs'):
            file_path = os.path.join(root, file)
            if fix_duplicate_schemars(file_path):
                fixed_count += 1

print(f"Fixed duplicates in {fixed_count} files")
