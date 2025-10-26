#!/bin/bash
# Fix a specific crate's imports precisely

CRATE_NAME=$1
NEW_MODULE_NAME=$2

if [[ -z "$CRATE_NAME" || -z "$NEW_MODULE_NAME" ]]; then
    echo "Usage: $0 <crate_name> <new_module_name>"
    exit 1
fi

LIB_FILE="iterations/v3/${CRATE_NAME}/src/lib.rs"

if [[ ! -f "$LIB_FILE" ]]; then
    echo "Lib file not found: $LIB_FILE"
    exit 1
fi

echo "Fixing $CRATE_NAME: types -> $NEW_MODULE_NAME"

# Fix the module declaration - only the exact line
sed -i '' "s/^pub mod types;$/pub mod ${NEW_MODULE_NAME};/g" "$LIB_FILE"

# Fix the re-export - only the exact pattern
sed -i '' "s/^pub use types::\*;$/pub use ${NEW_MODULE_NAME}::*;/g" "$LIB_FILE"

# Fix any other exact types:: references in the lib file
sed -i '' "s/types::/${NEW_MODULE_NAME}::/g" "$LIB_FILE"

echo "Fixed $CRATE_NAME lib.rs"
