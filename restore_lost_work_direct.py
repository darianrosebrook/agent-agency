#!/usr/bin/env python3
"""
Direct Cursor History Restoration

Scans Cursor's history directories directly and restores files to their oldest state
from the lost work period (October 19, 2025 onwards).
"""

import json
import os
import shutil
from pathlib import Path
from datetime import datetime
from typing import Dict, List, Optional, Tuple

def parse_timestamp_range() -> Tuple[int, int]:
    """Get the timestamp range for the lost work period."""
    # Start from October 19, 2025 (when the last remote commit was made)
    # Go until current time
    start_timestamp = int(datetime(2025, 10, 19, 22, 44).timestamp() * 1000)  # Oct 19, 10:44 PM
    end_timestamp = int(datetime.now().timestamp() * 1000)  # Current time
    return start_timestamp, end_timestamp

def scan_cursor_history() -> List[Tuple[str, Dict]]:
    """Scan all Cursor history directories and return entries.json data."""
    history_root = Path.home() / "Library" / "Application Support" / "Cursor" / "User" / "History"
    entries_data = []

    if not history_root.exists():
        print(f"❌ History directory not found: {history_root}")
        return []

    print(f"🔍 Scanning {len(list(history_root.iterdir()))} history directories...")

    for history_dir in history_root.iterdir():
        if not history_dir.is_dir():
            continue

        entries_file = history_dir / "entries.json"
        if not entries_file.exists():
            continue

        try:
            with open(entries_file, 'r') as f:
                data = json.load(f)

            # Check if this is for our agent-agency project
            if "resource" in data and "agent-agency" in data["resource"]:
                entries_data.append((str(history_dir), data))

        except (json.JSONDecodeError, FileNotFoundError):
            continue

    return entries_data

def find_lost_work_entries(entries_data: List[Tuple[str, Dict]]) -> Dict[str, Tuple[str, Dict, int]]:
    """
    Find files that have entries from the lost work period.

    Returns: {file_path: (history_dir, entry_data, oldest_timestamp)}
    """
    start_ts, end_ts = parse_timestamp_range()
    lost_work = {}

    print(f"🎯 Looking for work between {datetime.fromtimestamp(start_ts/1000)} and {datetime.fromtimestamp(end_ts/1000)}")

    for history_dir, data in entries_data:
        resource = data["resource"]

        # Extract the relative path within our project
        if "agent-agency/" in resource:
            relative_path = resource.split("agent-agency/")[-1]
        else:
            continue

        # Find entries in our lost work period
        lost_entries = []
        for entry in data.get("entries", []):
            timestamp = entry.get("timestamp", 0)
            if start_ts <= timestamp <= end_ts:
                lost_entries.append(entry)

        if lost_entries:
            # Find the OLDEST entry from the lost period (as user requested)
            oldest_entry = min(lost_entries, key=lambda x: x["timestamp"])
            oldest_timestamp = oldest_entry["timestamp"]

            lost_work[relative_path] = (history_dir, oldest_entry, oldest_timestamp)
            print(f"  📄 Found lost work: {relative_path} (oldest: {datetime.fromtimestamp(oldest_timestamp/1000)})")

    return lost_work

def restore_file_from_history(relative_path: str, history_dir: str, entry: Dict, timestamp: int, dry_run: bool = False) -> bool:
    """Restore a file from its oldest state in the lost work period."""
    entry_id = entry["id"]

    # The content file should be in the same directory as entries.json
    content_file = Path(history_dir) / entry_id

    if not content_file.exists():
        print(f"❌ Content file not found: {content_file}")
        return False

    # Target path in our project
    target_path = Path(relative_path)
    target_path.parent.mkdir(parents=True, exist_ok=True)

    timestamp_dt = datetime.fromtimestamp(timestamp / 1000)

    if dry_run:
        print(f"📋 Would restore: {relative_path}")
        print(f"   From: {entry_id} ({timestamp_dt.strftime('%Y-%m-%d %H:%M:%S')})")
        print(f"   Source: {entry.get('source', 'unknown')}")
        return True

    try:
        # Copy the historical content to our project
        shutil.copy2(content_file, target_path)
        print(f"✅ Restored: {relative_path}")
        print(f"   From: {entry_id} ({timestamp_dt.strftime('%Y-%m-%d %H:%M:%S')})")
        print(f"   Source: {entry.get('source', 'unknown')}")
        return True
    except Exception as e:
        print(f"❌ Failed to restore {relative_path}: {e}")
        return False

def main():
    """Main restoration function."""
    import sys

    print("🔄 Scanning Cursor history for direct restoration...")

    # Scan all history directories
    entries_data = scan_cursor_history()
    print(f"📊 Found {len(entries_data)} history directories with agent-agency files")

    # Find lost work
    lost_work = find_lost_work_entries(entries_data)
    print(f"🎯 Found {len(lost_work)} files with lost work")

    if not lost_work:
        print("❌ No lost work found in the specified period.")
        return

    print()

    # Sort by timestamp (oldest first, as per user request)
    sorted_work = sorted(lost_work.items(), key=lambda x: x[1][2])  # Sort by timestamp

    # Check command line arguments
    dry_run = '--dry-run' in sys.argv
    num_files = min(20, len(sorted_work))  # Default to 20 or fewer

    # Parse command line arguments
    for arg in sys.argv[1:]:
        if arg.isdigit():
            num_files = min(int(arg), len(sorted_work))
        elif arg == '--dry-run':
            dry_run = True
        elif arg == '--all':
            num_files = len(sorted_work)

    files_to_restore = sorted_work[:num_files]

    print(f"📋 Will process {len(files_to_restore)} oldest files from lost period")
    print()

    if dry_run:
        print("🔍 DRY RUN - Previewing restoration to oldest state:")
        print("=" * 50)
    else:
        print("🔄 Starting restoration to oldest state from lost period:")
        print("=" * 50)

    restored_count = 0
    for relative_path, (history_dir, entry, timestamp) in files_to_restore:
        if restore_file_from_history(relative_path, history_dir, entry, timestamp, dry_run):
            restored_count += 1

    print()
    print("=" * 50)
    if dry_run:
        print(f"📋 Preview complete: {restored_count} files would be restored")
        print()
        print("💡 To actually restore files, run:")
        print(f"   python3 restore_lost_work_direct.py {len(files_to_restore)}")
        print("   python3 restore_lost_work_direct.py --all  # for all files")
    else:
        print(f"✅ Restoration complete: {restored_count}/{len(files_to_restore)} files restored")

        if restored_count > 0:
            print()
            print("📝 Next steps:")
            print("1. Review the restored files")
            print("2. Test that they work correctly")
            print("3. Commit the restored work: git add . && git commit -m 'feat: restore lost development work'")
            print("4. Continue with remaining files if needed")
            print()
            print("⚠️  If anything goes wrong, you can always: git reset --hard origin/main")

if __name__ == "__main__":
    main()
