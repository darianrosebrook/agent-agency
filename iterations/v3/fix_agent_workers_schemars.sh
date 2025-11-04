#!/bin/bash
echo "Fixing duplicate schemars attributes in agent-workers..."

# Find all Rust files in agent-workers
find agent-workers/src -name "*.rs" | while read -r file; do
    echo "Processing $file..."
    
    # Create a temporary file
    temp_file="${file}.tmp"
    
    # Use awk to remove duplicate #[schemars(with = "String")] lines
    awk '
    BEGIN { in_schemars = 0 }
    /^    #\[schemars\(with = "String"\)\]$/ {
        if (in_schemars == 0) {
            in_schemars = 1
            print
        } else {
            in_schemars = 0
            next
        }
    }
    /^    #\[schemars\(with = "String"\)\]$/ == 0 {
        in_schemars = 0
        print
    }
    ' "$file" > "$temp_file"
    
    # Replace original file if changes were made
    if ! diff -q "$file" "$temp_file" > /dev/null; then
        echo "  Fixed duplicates in $file"
        mv "$temp_file" "$file"
    else
        rm "$temp_file"
        echo "  No duplicates found in $file"
    fi
done

echo "Done fixing schemars duplicates in agent-workers."
