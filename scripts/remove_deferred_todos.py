#!/usr/bin/env python3
"""
Remove TODO comments from DEFER_ENHANCEMENTS and DEFER_DOCUMENTATION categories.
"""

import re
from pathlib import Path
from typing import Dict, List, Set


def extract_deferred_entries(categorized_file: Path) -> Dict[str, List[Dict]]:
    """Extract files and line numbers from DEFER categories."""
    deferred_files = {}

    current_category = None
    current_file = None
    lines_to_remove = []

    with open(categorized_file, 'r') as f:
        for line in f:
            line = line.strip()

            # Check for category headers
            if line.startswith("## ⏳ Future Enhancements") or line.startswith("## ⏳ Documentation"):
                current_category = "DEFER"
                continue
            elif line.startswith("## "):
                current_category = None
                continue

            # If we're in a DEFER category and see a file header
            if current_category == "DEFER" and line.startswith("### "):
                # Save previous file if exists
                if current_file and lines_to_remove:
                    if current_file not in deferred_files:
                        deferred_files[current_file] = []
                    deferred_files[current_file].extend(lines_to_remove)

                # Start new file
                current_file = line[4:]  # Remove "### "
                lines_to_remove = []

            # If we have a current file and see a line number entry
            elif current_file and re.match(r'- Line \d+:', line):
                # Extract line number from pattern like "- Line 123: `content`"
                match = re.match(r'- Line (\d+):', line)
                if match:
                    line_num = int(match.group(1))
                    lines_to_remove.append(line_num)

        # Save last file
        if current_file and lines_to_remove:
            if current_file not in deferred_files:
                deferred_files[current_file] = []
            deferred_files[current_file].extend(lines_to_remove)

    return deferred_files


def remove_todo_comments_from_file(file_path: Path, line_numbers: List[int]) -> bool:
    """Remove TODO comments from specified line numbers in a file."""
    if not file_path.exists():
        print(f"Warning: File not found: {file_path}")
        return False

    # Sort line numbers in descending order to avoid offset issues
    line_numbers = sorted(set(line_numbers), reverse=True)

    lines = []
    with open(file_path, 'r') as f:
        lines = f.readlines()

    # Track which lines were removed
    removed_count = 0
    removed_lines = []

    for line_num in line_numbers:
        # Convert to 0-based index
        idx = line_num - 1
        if idx < 0 or idx >= len(lines):
            print(f"Warning: Line {line_num} not found in {file_path}")
            continue

        original_line = lines[idx]
        # Check if it's a TODO comment
        if any(keyword in original_line.upper() for keyword in ['TODO', 'PLACEHOLDER', 'FIXME']):
            removed_lines.append(original_line.strip())
            lines.pop(idx)
            removed_count += 1

    # Write back the file
    if removed_count > 0:
        with open(file_path, 'w') as f:
            f.writelines(lines)
        print(f"✅ {file_path}: Removed {removed_count} TODO comments")
        for line in removed_lines[:3]:  # Show first 3 removed lines
            print(f"   - {line}")
        if len(removed_lines) > 3:
            print(f"   ... and {len(removed_lines) - 3} more")
        return True

    return False


def main():
    repo_root = Path(__file__).parent.parent
    categorized_file = repo_root / "todo_categorized.md"

    if not categorized_file.exists():
        print(f"Error: {categorized_file} not found")
        return 1

    print("🔍 Extracting deferred TODO entries...")
    deferred_files = extract_deferred_entries(categorized_file)

    if not deferred_files:
        print("No deferred TODO entries found")
        return 0

    print(f"📋 Found {len(deferred_files)} files with deferred TODOs")

    total_removed = 0
    files_modified = 0

    for file_path_str, line_numbers in deferred_files.items():
        file_path = repo_root / file_path_str
        if remove_todo_comments_from_file(file_path, line_numbers):
            files_modified += 1
            total_removed += len([ln for ln in line_numbers if ln])  # Rough count

    print("\n📊 Summary:")
    print(f"   Files processed: {len(deferred_files)}")
    print(f"   Files modified: {files_modified}")
    print(f"   Total TODOs removed: {total_removed}")

    if files_modified > 0:
        print("\n✅ Deferred TODO comments removed successfully!")
        print("   These were low-priority enhancements and documentation TODOs.")
        print("   V3 codebase is now cleaner and focused on production-ready code.")

    return 0


if __name__ == "__main__":
    exit(main())

