#!/usr/bin/env python3
"""
CAWS Refactoring Script

This script programmatically refactors the monolithic caws_checker.rs file
by moving code to the appropriate module files based on REFACTOR comments.
"""

import re
import os
from pathlib import Path
from typing import Dict, List, Tuple, Optional

class CAWSRefactorer:
    def __init__(self, base_path: str):
        self.base_path = Path(base_path)
        self.workers_path = self.base_path / "iterations/v3/workers/src"
        self.caws_path = self.workers_path / "caws"
        self.source_file = self.workers_path / "caws_checker.rs"
        
        # Track what we've moved
        self.moved_items: Dict[str, List[str]] = {}
        
    def parse_refactor_comments(self) -> List[Tuple[str, str, str]]:
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
    
    def extract_code_block(self, start_line: int, lines: List[str]) -> Tuple[str, int]:
        """Extract a complete code block (struct, enum, impl, etc.) starting from start_line."""
        # Find the start of the actual code (skip comments)
        code_start = start_line + 1
        while code_start < len(lines) and lines[code_start].strip().startswith('//'):
            code_start += 1
            
        # Find the matching brace
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
                
        # Extract the code block
        code_lines = lines[code_start:end_line + 1]
        return ''.join(code_lines), end_line + 1
    
    def extract_impl_block(self, start_line: int, lines: List[str]) -> Tuple[str, int]:
        """Extract an impl block, handling trait implementations."""
        # Find the start of the actual code
        code_start = start_line + 1
        while code_start < len(lines) and lines[code_start].strip().startswith('//'):
            code_start += 1
            
        # For impl blocks, we need to find the end of the impl
        brace_count = 0
        in_code = False
        end_line = code_start
        
        for i in range(code_start, len(lines)):
            line = lines[i]
            
            # Count braces
            for char in line:
                if char == '{':
                    brace_count += 1
                    in_code = True
                elif char == '}':
                    brace_count -= 1
                    
            if in_code and brace_count == 0:
                end_line = i
                break
                
        # Extract the impl block
        code_lines = lines[code_start:end_line + 1]
        return ''.join(code_lines), end_line + 1
    
    def move_item_to_file(self, item: str, destination: str, code: str):
        """Move a code item to its destination file."""
        dest_path = self.caws_path / destination
        
        # Ensure the destination file exists
        if not dest_path.exists():
            print(f"Warning: Destination file {dest_path} does not exist")
            return
            
        # Read the current content
        with open(dest_path, 'r') as f:
            content = f.read()
            
        # Add the code to the file
        # For now, we'll append it (in a real implementation, we'd be smarter about placement)
        with open(dest_path, 'a') as f:
            f.write(f"\n// Moved from caws_checker.rs\n{code}\n")
            
        print(f"Moved {item} to {destination}")
        
    def refactor(self):
        """Main refactoring process."""
        print("Starting CAWS refactoring...")
        
        # Parse refactor comments
        refactor_items = self.parse_refactor_comments()
        print(f"Found {len(refactor_items)} items to refactor")
        
        # Read the source file
        with open(self.source_file, 'r') as f:
            lines = f.readlines()
            
        # Process each refactor item
        for item, destination, line_num in refactor_items:
            print(f"Processing: {item} -> {destination}")
            
            # Extract the code block
            if "impl" in item.lower():
                code, end_line = self.extract_impl_block(line_num, lines)
            else:
                code, end_line = self.extract_code_block(line_num, lines)
                
            # Move to destination file
            self.move_item_to_file(item, destination, code)
            
        print("Refactoring complete!")
        
    def generate_summary(self):
        """Generate a summary of what was moved."""
        print("\n=== REFACTORING SUMMARY ===")
        for destination, items in self.moved_items.items():
            print(f"\n{destination}:")
            for item in items:
                print(f"  - {item}")

def main():
    refactorer = CAWSRefactorer("/Users/darianrosebrook/Desktop/Projects/agent-agency")
    refactorer.refactor()
    refactorer.generate_summary()

if __name__ == "__main__":
    main()
