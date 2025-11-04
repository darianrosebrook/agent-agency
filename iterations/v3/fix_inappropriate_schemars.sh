#!/bin/bash
echo "Fixing inappropriate schemars attributes on function parameters and other invalid locations..."

# Find all Rust files and remove schemars attributes from invalid locations
find data-infrastructure/src -name "*.rs" | while read -r file; do
    echo "Processing $file..."
    
    # Create a temporary file
    temp_file="${file}.tmp"
    
    # Use sed to remove schemars attributes from function parameters and other invalid locations
    # Remove lines that have #[schemars(...)] followed by function parameters or other invalid contexts
    sed '/^[[:space:]]*#[schemars(with = "String")][[:space:]]*$/{
        N
        /async fn.*#[schemars(with = "String")]/{
            s/#[schemars(with = "String")]\n[[:space:]]*/ /
            t
        }
        /fn.*#[schemars(with = "String")]/{
            s/#[schemars(with = "String")]\n[[:space:]]*/ /
            t
        }
        /#[schemars(with = "String")]\n[[:space:]]*start_time:/{
            s/#[schemars(with = "String")]\n[[:space:]]*//
            t
        }
        /#[schemars(with = "String")]\n[[:space:]]*result:/{
            s/#[schemars(with = "String")]\n[[:space:]]*//
            t
        }
        /#[schemars(with = "String")]\n[[:space:]]*metadata_row:/{
            s/#[schemars(with = "String")]\n[[:space:]]*//
            t
        }
        P;D
    }' "$file" > "$temp_file"
    
    # Also remove any remaining inappropriate schemars attributes on function parameters
    sed -i 's/#[schemars(with = "String")][[:space:]]*start_time:/start_time:/g' "$temp_file"
    sed -i 's/#[schemars(with = "String")][[:space:]]*result:/result:/g' "$temp_file"
    sed -i 's/#[schemars(with = "String")][[:space:]]*metadata_row:/metadata_row:/g' "$temp_file"
    
    # Replace original file if changes were made
    if ! diff -q "$file" "$temp_file" > /dev/null; then
        echo "  Fixed inappropriate schemars attributes in $file"
        mv "$temp_file" "$file"
    else
        rm "$temp_file"
    fi
done

echo "Done fixing inappropriate schemars attributes."
