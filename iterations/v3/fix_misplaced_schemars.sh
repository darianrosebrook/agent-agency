#!/bin/bash
echo "Fixing misplaced schemars attributes in system-federated-ml..."

# Find all Rust files in system-federated-ml and fix misplaced schemars attributes
find system-federated-ml/src -name "*.rs" | while read -r file; do
    echo "Processing $file..."
    
    # Create a temporary file
    temp_file="${file}.tmp"
    
    # Use sed to fix misplaced schemars attributes
    # Pattern: #[schemars(with = "String")] followed by empty line then field
    sed 'N; s/#\[schemars(with = "String")\]\n[[:space:]]*\n[[:space:]]*pub \([a-zA-Z_][a-zA-Z0-9_]*\):/#\[schemars(with = "String")\]\n    pub \1:/g' "$file" > "$temp_file"
    
    # Replace original file if changes were made
    if ! diff -q "$file" "$temp_file" > /dev/null; then
        echo "  Fixed misplaced schemars attributes in $file"
        mv "$temp_file" "$file"
    else
        rm "$temp_file"
    fi
done

echo "Done fixing misplaced schemars attributes."
