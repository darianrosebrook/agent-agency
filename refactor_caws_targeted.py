#!/usr/bin/env python3
"""
Targeted CAWS Refactoring Script

This script performs the actual refactoring by moving code blocks
from the monolithic caws_checker.rs to the appropriate module files.
"""

import re
import os
from pathlib import Path
from typing import Dict, List, Tuple, Set
import shutil

class TargetedCAWSRefactorer:
    def __init__(self, base_path: str):
        self.base_path = Path(base_path)
        self.workers_path = self.base_path / "iterations/v3/workers/src"
        self.caws_path = self.workers_path / "caws"
        self.source_file = self.workers_path / "caws_checker.rs"
        
        # Track moved items
        self.moved_items: Dict[str, List[str]] = {}
        
    def find_brace_boundaries(self, lines: List[str], start_idx: int) -> Tuple[int, int]:
        """Find the start and end of a code block by matching braces."""
        # Skip comment lines
        code_start = start_idx + 1
        while code_start < len(lines) and lines[code_start].strip().startswith('//'):
            code_start += 1
            
        # Skip empty lines
        while code_start < len(lines) and lines[code_start].strip() == '':
            code_start += 1
            
        # Find matching braces
        brace_count = 0
        in_code = False
        end_line = code_start
        
        for i in range(code_start, len(lines)):
            line = lines[i]
            
            for char in line:
                if char == '{':
                    brace_count += 1
                    in_code = True
                elif char == '}':
                    brace_count -= 1
                    
            if in_code and brace_count == 0:
                end_line = i + 1
                break
                
        return code_start, end_line
    
    def extract_and_move_item(self, item: str, destination: str, line_num: int, lines: List[str]):
        """Extract a single item and move it to its destination."""
        print(f"Moving {item} to {destination}")
        
        # Find the code boundaries
        start_line, end_line = self.find_brace_boundaries(lines, line_num)
        
        # Extract the code
        code_lines = lines[start_line:end_line]
        code = ''.join(code_lines)
        
        # Determine the destination file
        dest_file = self.workers_path / destination
        
        if not dest_file.exists():
            print(f"Warning: {dest_file} does not exist")
            return
            
        # Read current content
        with open(dest_file, 'r') as f:
            content = f.read()
            
        # Add the code to the destination file
        # We'll append it for now (in a real implementation, we'd be smarter about placement)
        with open(dest_file, 'a') as f:
            f.write(f"\n\n// Moved from caws_checker.rs: {item}\n{code}\n")
            
        # Track what we moved
        if destination not in self.moved_items:
            self.moved_items[destination] = []
        self.moved_items[destination].append(item)
        
        print(f"✓ Moved {item} to {destination}")
    
    def refactor(self):
        """Perform the refactoring."""
        print("Starting targeted CAWS refactoring...")
        
        # Create backup
        backup_path = self.source_file.with_suffix('.rs.backup')
        shutil.copy2(self.source_file, backup_path)
        print(f"Created backup: {backup_path}")
        
        # Read the source file
        with open(self.source_file, 'r') as f:
            lines = f.readlines()
            
        # Find all REFACTOR comments
        refactor_items = []
        for i, line in enumerate(lines):
            if "// REFACTOR:" in line:
                match = re.search(r'\[send (.+?) to (.+?)\]', line)
                if match:
                    item = match.group(1).strip()
                    destination = match.group(2).strip()
                    refactor_items.append((item, destination, i))
                    
        print(f"Found {len(refactor_items)} items to refactor")
        
        # Process each item
        for item, destination, line_num in refactor_items:
            try:
                self.extract_and_move_item(item, destination, line_num, lines)
            except Exception as e:
                print(f"Error moving {item}: {e}")
                continue
                
        print("\nRefactoring complete!")
        
    def generate_summary(self):
        """Generate a summary of the refactoring."""
        print("\n" + "="*50)
        print("REFACTORING SUMMARY")
        print("="*50)
        
        for destination, items in self.moved_items.items():
            print(f"\n📁 {destination}:")
            for item in items:
                print(f"  ✓ {item}")
                
        print(f"\nTotal items moved: {sum(len(items) for items in self.moved_items.values())}")
        
    def generate_next_steps(self):
        """Generate next steps for completing the refactoring."""
        print("\n" + "="*50)
        print("NEXT STEPS")
        print("="*50)
        
        print("\n1. Update imports in caws_checker.rs:")
        print("   Add: use crate::caws::*;")
        
        print("\n2. Remove moved code from caws_checker.rs:")
        print("   - Remove the moved structs, enums, and impls")
        print("   - Keep only the main CawsChecker struct and its core impl")
        
        print("\n3. Update lib.rs:")
        print("   - Add: pub mod caws;")
        print("   - Update re-exports")
        
        print("\n4. Test compilation:")
        print("   - Run: cargo check -p agent-agency-workers")
        print("   - Fix any import issues")
        
        print("\n5. Clean up:")
        print("   - Remove backup file if everything works")
        print("   - Update any remaining references")

def main():
    refactorer = TargetedCAWSRefactorer("/Users/darianrosebrook/Desktop/Projects/agent-agency")
    refactorer.refactor()
    refactorer.generate_summary()
    refactorer.generate_next_steps()

if __name__ == "__main__":
    main()
