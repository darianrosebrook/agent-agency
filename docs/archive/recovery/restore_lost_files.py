#!/usr/bin/env python3
"""
Restore Lost Files from Cursor History

Restores the most critical lost files from the gap period back to the project.
"""

import json
import os
import shutil
from pathlib import Path
from typing import Dict, List, Optional
from dataclasses import dataclass
from datetime import datetime

@dataclass
class LostFile:
    """Represents a file that had lost work."""
    relative_path: str
    last_modified: datetime
    history_entries: List[Dict]
    latest_content_path: Optional[str] = None

def load_lost_work_data() -> Dict[str, LostFile]:
    """Load the lost work data from the report."""
    lost_files = {}

    try:
        with open('lost-work-report.md', 'r') as f:
            content = f.read()

        # Parse the file entries from the markdown
        lines = content.split('\n')
        current_file = None
        current_content_path = None

        for line in lines:
            if line.startswith('### `') and line.endswith('`'):
                # New file entry
                current_file = line[5:-1]  # Remove ### ` and `
                lost_files[current_file] = LostFile(
                    relative_path=current_file,
                    last_modified=datetime.now(),  # Will be updated
                    history_entries=[]
                )
            elif current_file and '**Last Modified**: ' in line:
                # Parse timestamp
                timestamp_str = line.split('**Last Modified**: ')[1]
                try:
                    dt = datetime.strptime(timestamp_str, '%Y-%m-%d %H:%M:%S')
                    lost_files[current_file].last_modified = dt
                except ValueError:
                    pass
            elif current_file and '**Content Available**: ✅ (' in line:
                # Extract content path
                content_part = line.split('**Content Available**: ✅ (')[1]
                content_file = content_part.split(')')[0]
                lost_files[current_file].latest_content_path = content_file

    except FileNotFoundError:
        print("❌ lost-work-report.md not found. Run recover_lost_work.py first.")
        return {}

    return lost_files

def find_history_content_path(file_path: str, content_filename: str) -> Optional[str]:
    """Find the full path to the history content file."""
    history_root = Path.home() / "Library" / "Application Support" / "Cursor" / "User" / "History"

    # Find directories that contain agent-agency files
    for history_dir in history_root.iterdir():
        if not history_dir.is_dir():
            continue

        entries_file = history_dir / "entries.json"
        if not entries_file.exists():
            continue

        try:
            with open(entries_file, 'r') as f:
                data = json.load(f)

            if "resource" not in data:
                continue

            resource = data["resource"]
            if "agent-agency" in resource and file_path in resource:
                # Found the directory, check for the content file
                content_path = history_dir / content_filename
                if content_path.exists():
                    return str(content_path)
        except (json.JSONDecodeError, FileNotFoundError):
            continue

    return None

def restore_file(lost_file: LostFile, dry_run: bool = False) -> bool:
    """Restore a single lost file."""
    if not lost_file.latest_content_path:
        print(f"❌ No content available for {lost_file.relative_path}")
        return False

    # Find the actual content file path
    content_path = find_history_content_path(lost_file.relative_path, lost_file.latest_content_path)
    if not content_path:
        print(f"❌ Could not locate content file for {lost_file.relative_path}")
        return False

    # Target path in the project
    target_path = Path(lost_file.relative_path)

    # Create directories if needed
    target_path.parent.mkdir(parents=True, exist_ok=True)

    if dry_run:
        print(f"📋 Would restore: {lost_file.relative_path}")
        print(f"   From: {content_path}")
        print(f"   Last modified: {lost_file.last_modified.strftime('%Y-%m-%d %H:%M:%S')}")
        return True

    # Copy the file
    try:
        shutil.copy2(content_path, target_path)
        print(f"✅ Restored: {lost_file.relative_path}")
        print(f"   From: {content_path}")
        print(f"   Last modified: {lost_file.last_modified.strftime('%Y-%m-%d %H:%M:%S')}")
        return True
    except Exception as e:
        print(f"❌ Failed to restore {lost_file.relative_path}: {e}")
        return False

def main():
    """Main restoration function."""
    import sys

    print("🔄 Loading lost work data...")

    lost_files = load_lost_work_data()
    if not lost_files:
        print("❌ No lost work data found. Run recover_lost_work.py first.")
        return

    # Sort by last modified (most recent first)
    sorted_files = sorted(lost_files.values(), key=lambda x: x.last_modified, reverse=True)

    print(f"📊 Found {len(sorted_files)} files with recoverable content")
    print()

    # Check command line arguments
    dry_run = '--dry-run' in sys.argv
    num_files = 20  # Default number of files to restore

    # Parse command line arguments
    for arg in sys.argv[1:]:
        if arg.isdigit():
            num_files = int(arg)
        elif arg == '--dry-run':
            dry_run = True
        elif arg == '--all':
            num_files = len(sorted_files)
            dry_run = False

    files_to_restore = sorted_files[:num_files]

    print(f"📋 Will process {len(files_to_restore)} most recent files")
    print()

    if dry_run:
        print("🔍 DRY RUN - Previewing restoration:")
        print("=" * 50)
    else:
        print("🔄 Starting restoration:")
        print("=" * 50)

    restored_count = 0
    for lost_file in files_to_restore:
        if restore_file(lost_file, dry_run):
            restored_count += 1

    print()
    print("=" * 50)
    if dry_run:
        print(f"📋 Preview complete: {restored_count} files would be restored")
        print()
        print("💡 To actually restore files, run:")
        print(f"   python3 restore_lost_files.py {len(files_to_restore)}")
        print("   python3 restore_lost_files.py --all  # for all files")
    else:
        print(f"✅ Restoration complete: {restored_count}/{len(files_to_restore)} files restored")

        if restored_count > 0:
            print()
            print("📝 Next steps:")
            print("1. Review the restored files")
            print("2. Test that they work correctly")
            print("3. Commit the restored work: git add . && git commit -m 'feat: restore lost development work'")
            print("4. Continue with remaining files if needed")

if __name__ == "__main__":
    main()
