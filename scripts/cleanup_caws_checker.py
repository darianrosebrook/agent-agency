#!/usr/bin/env python3
"""
Cleanup script for caws_checker.rs

This script removes the moved code from the original file and adds proper imports.
"""

import re
from pathlib import Path

def cleanup_caws_checker():
    """Clean up the caws_checker.rs file after refactoring."""
    
    source_file = Path("/Users/darianrosebrook/Desktop/Projects/agent-agency/iterations/v3/workers/src/caws_checker.rs")
    
    # Read the current file
    with open(source_file, 'r') as f:
        lines = f.readlines()
    
    # Find all REFACTOR comments and remove the associated code blocks
    lines_to_remove = set()
    
    for i, line in enumerate(lines):
        if "// REFACTOR:" in line:
            # Find the end of this code block
            start_line = i + 1
            # Skip comment lines
            while start_line < len(lines) and lines[start_line].strip().startswith('//'):
                start_line += 1
            
            # Skip empty lines
            while start_line < len(lines) and lines[start_line].strip() == '':
                start_line += 1
                
            # Find the matching closing brace
            brace_count = 0
            in_code = False
            end_line = start_line
            
            for j in range(start_line, len(lines)):
                line_content = lines[j]
                
                for char in line_content:
                    if char == '{':
                        brace_count += 1
                        in_code = True
                    elif char == '}':
                        brace_count -= 1
                        
                if in_code and brace_count == 0:
                    end_line = j + 1
                    break
            
            # Mark lines for removal
            for k in range(i, end_line):
                lines_to_remove.add(k)
    
    # Create new file content without the moved code
    new_lines = []
    for i, line in enumerate(lines):
        if i not in lines_to_remove:
            new_lines.append(line)
    
    # Add imports for the new module structure
    import_lines = [
        "//! CAWS Checker\n",
        "//!\n",
        "//! Provides CAWS compliance checking and validation for worker outputs.\n",
        "//! Enhanced with AST-based diff sizing and violation code mapping.\n",
        "\n",
        "// Import the refactored CAWS modules\n",
        "use crate::caws::*;\n",
        "\n",
        "use crate::types::*;\n",
        "use agent_agency_council::models::{\n",
        "    FileModification as CouncilFileModification, FileOperation as CouncilFileOperation, RiskTier,\n",
        "    TaskSpec, WorkerOutput as CouncilWorkerOutput,\n",
        "};\n",
        "use agent_agency_database::{CawsViolation as DbCawsViolation, DatabaseClient};\n",
        "use anyhow::{Context, Result};\n",
        "use serde_json::json;\n",
        "use sqlx::Row;\n",
        "use std::collections::HashMap;\n",
        "use tracing::info;\n",
        "use uuid::Uuid;\n",
        "\n",
    ]
    
    # Write the cleaned up file
    with open(source_file, 'w') as f:
        f.writelines(import_lines)
        f.writelines(new_lines)
    
    print("✓ Cleaned up caws_checker.rs")
    print("✓ Added imports for refactored modules")
    print("✓ Removed moved code blocks")

if __name__ == "__main__":
    cleanup_caws_checker()
