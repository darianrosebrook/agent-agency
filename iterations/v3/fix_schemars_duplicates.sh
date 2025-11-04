#!/bin/bash
echo "Fixing duplicate schemars attributes..."

# Find all files with schemars attributes
find system-federated-ml/src -name "*.rs" -exec grep -l "#\[schemars" {} \; | while read file; do
    echo "Processing $file..."
    
    # Use sed to remove duplicate consecutive schemars attributes
    sed -i '/^\s*#\s*\[schemars/N;s/\n\s*#\s*\[schemars.*\]//;P;D' "$file"
    
    # Alternative approach: remove duplicate lines
    awk '!seen[$0]++ || !/#\[schemars/' "$file" > "${file}.tmp" && mv "${file}.tmp" "$file"
    
    echo "Fixed $file"
done

echo "Done fixing duplicate schemars attributes"
