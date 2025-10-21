#!/usr/bin/env python3
"""
Advanced CAWS Refactoring Script

This script programmatically refactors the monolithic caws_checker.rs file
by moving code to the appropriate module files based on REFACTOR comments.
"""

import re
import os
from pathlib import Path
from typing import Dict, List, Tuple, Optional, Set
import shutil

class AdvancedCAWSRefactorer:
    def __init__(self, base_path: str):
        self.base_path = Path(base_path)
        self.workers_path = self.base_path / "iterations/v3/workers/src"
        self.caws_path = self.workers_path / "caws"
        self.source_file = self.workers_path / "caws_checker.rs"
        
        # Track what we've moved
        self.moved_items: Dict[str, List[str]] = {}
        self.imports_to_add: Dict[str, Set[str]] = {}
        
    def parse_refactor_comments(self) -> List[Tuple[str, str, int]]:
        """Parse REFACTOR comments from the source file."""
        refactor_items = []
        
        with open(self.source_file, 'r') as f:
            lines = f.readlines()
            
        for i, line in enumerate(lines):
            if "// REFACTOR:" in line:
                # Extract the item and destination
                match = re.search(r'\[send (.+?) to (.+?)\]', line)
                if match:
                    item = match.group(1).strip()
                    destination = match.group(2).strip()
                    refactor_items.append((item, destination, i))
                    
        return refactor_items
    
    def find_code_block_boundaries(self, start_line: int, lines: List[str]) -> Tuple[int, int]:
        """Find the start and end of a code block."""
        # Skip comment lines
        code_start = start_line + 1
        while code_start < len(lines) and lines[code_start].strip().startswith('//'):
            code_start += 1
            
        # Find the actual start of the code (struct, enum, impl, etc.)
        while code_start < len(lines) and lines[code_start].strip() == '':
            code_start += 1
            
        # Find the matching closing brace
        brace_count = 0
        in_code = False
        end_line = code_start
        
        for i in range(code_start, len(lines)):
            line = lines[i]
            
            # Count braces to find the end of the block
            for char in line:
                if char == '{':
                    brace_count += 1
                    in_code = True
                elif char == '}':
                    brace_count -= 1
                    
            if in_code and brace_count == 0:
                end_line = i
                break
                
        return code_start, end_line + 1
    
    def extract_code_with_imports(self, start_line: int, end_line: int, lines: List[str]) -> str:
        """Extract code block and determine needed imports."""
        code_lines = lines[start_line:end_line]
        code = ''.join(code_lines)
        
        # Determine what imports this code needs
        needed_imports = set()
        
        # Check for common types that need imports
        if 'chrono::' in code:
            needed_imports.add('use chrono::{DateTime, Utc};')
        if 'serde::' in code:
            needed_imports.add('use serde::{Deserialize, Serialize};')
        if 'std::collections::HashMap' in code:
            needed_imports.add('use std::collections::HashMap;')
        if 'anyhow::' in code:
            needed_imports.add('use anyhow::Result;')
        if 'tracing::' in code:
            needed_imports.add('use tracing::{info, warn, error};')
        if 'uuid::' in code:
            needed_imports.add('use uuid::Uuid;')
        if 'sqlx::' in code:
            needed_imports.add('use sqlx::{query, Row};')
            
        return code, needed_imports
    
    def update_destination_file(self, destination: str, code: str, imports: Set[str]):
        """Update the destination file with the moved code."""
        dest_path = self.caws_path / destination
        
        if not dest_path.exists():
            print(f"Warning: Destination file {dest_path} does not exist")
            return
            
        # Read current content
        with open(dest_path, 'r') as f:
            content = f.read()
            
        # Add imports if not already present
        for import_line in imports:
            if import_line not in content:
                # Find where to insert imports (after existing use statements)
                lines = content.split('\n')
                insert_pos = 0
                for i, line in enumerate(lines):
                    if line.startswith('use ') and not line.startswith('use crate::'):
                        insert_pos = i + 1
                        
                lines.insert(insert_pos, import_line)
                content = '\n'.join(lines)
        
        # Add the code
        content += f"\n\n// Moved from caws_checker.rs\n{code}\n"
        
        # Write back to file
        with open(dest_path, 'w') as f:
            f.write(content)
            
    def remove_from_source(self, start_line: int, end_line: int):
        """Remove the moved code from the source file."""
        # This is a placeholder - in a real implementation, we'd:
        # 1. Remove the code from the source file
        # 2. Add appropriate imports to the source file
        # 3. Update any references to the moved items
        pass
    
    def refactor(self):
        """Main refactoring process."""
        print("Starting advanced CAWS refactoring...")
        
        # Create backup of source file
        backup_path = self.source_file.with_suffix('.rs.backup')
        shutil.copy2(self.source_file, backup_path)
        print(f"Created backup: {backup_path}")
        
        # Parse refactor comments
        refactor_items = self.parse_refactor_comments()
        print(f"Found {len(refactor_items)} items to refactor")
        
        # Read the source file
        with open(self.source_file, 'r') as f:
            lines = f.readlines()
            
        # Process each refactor item
        for item, destination, line_num in refactor_items:
            print(f"Processing: {item} -> {destination}")
            
            # Find code block boundaries
            start_line, end_line = self.find_code_block_boundaries(line_num, lines)
            
            # Extract code and determine imports
            code, imports = self.extract_code_with_imports(start_line, end_line, lines)
            
            # Update destination file
            self.update_destination_file(destination, code, imports)
            
            # Track what we moved
            if destination not in self.moved_items:
                self.moved_items[destination] = []
            self.moved_items[destination].append(item)
            
        print("Refactoring complete!")
        
    def generate_summary(self):
        """Generate a summary of what was moved."""
        print("\n=== REFACTORING SUMMARY ===")
        for destination, items in self.moved_items.items():
            print(f"\n{destination}:")
            for item in items:
                print(f"  - {item}")
                
    def generate_import_updates(self):
        """Generate the import updates needed for the main file."""
        print("\n=== IMPORT UPDATES NEEDED ===")
        print("Add these imports to caws_checker.rs:")
        print("use crate::caws::*;")
        print("// Or more specifically:")
        for destination in self.moved_items.keys():
            module_name = destination.replace('.rs', '').replace('caws/', '')
            print(f"use crate::caws::{module_name}::*;")

def main():
    refactorer = AdvancedCAWSRefactorer("/Users/darianrosebrook/Desktop/Projects/agent-agency")
    refactorer.refactor()
    refactorer.generate_summary()
    refactorer.generate_import_updates()

if __name__ == "__main__":
    main()
