#!/usr/bin/env python3
"""
Cursor History Recovery Script

Analyzes Cursor's undo history to recover lost development work from the gap period
between the last remote commit (Oct 19, 2025, 10:44 PM) and local commit (Oct 21, 2025, 1:03 AM).
"""

import json
import os
import glob
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Tuple, Optional
from dataclasses import dataclass

@dataclass
class HistoryEntry:
    """Represents a single history entry for a file."""
    file_path: str
    history_id: str
    timestamp: int
    source: Optional[str] = None

    @property
    def datetime(self) -> datetime:
        """Convert timestamp to datetime."""
        return datetime.fromtimestamp(self.timestamp / 1000)

    @property
    def relative_path(self) -> str:
        """Get the relative path within the project."""
        if "agent-agency" in self.file_path:
            return self.file_path.split("agent-agency/")[-1]
        return self.file_path

@dataclass
class LostFile:
    """Represents a file that had lost work."""
    relative_path: str
    last_modified: datetime
    history_entries: List[HistoryEntry]
    latest_content_path: Optional[str] = None

def parse_timestamp_range() -> Tuple[int, int]:
    """Get the timestamp range for the lost work period."""
    # Last remote commit: October 19, 2025, 10:44 PM
    # Local commit: October 21, 2025, 1:03 AM
    start_timestamp = int(datetime(2025, 10, 19, 22, 44).timestamp() * 1000)  # Oct 19, 10:44 PM
    end_timestamp = int(datetime(2025, 10, 21, 1, 3).timestamp() * 1000)     # Oct 21, 1:03 AM
    return start_timestamp, end_timestamp

def scan_history_directories() -> List[str]:
    """Find all Cursor history directories."""
    history_root = Path.home() / "Library" / "Application Support" / "Cursor" / "User" / "History"
    if not history_root.exists():
        print(f"History directory not found: {history_root}")
        return []

    # Find all directories containing entries.json
    entries_files = list(history_root.rglob("entries.json"))
    directories = [str(f.parent) for f in entries_files]
    return sorted(directories)

def parse_history_entry(history_dir: str) -> Optional[HistoryEntry]:
    """Parse a single history directory's entries.json."""
    entries_file = os.path.join(history_dir, "entries.json")

    try:
        with open(entries_file, 'r') as f:
            data = json.load(f)

        if "resource" not in data or "entries" not in data:
            return None

        # Extract file path
        resource = data["resource"]
        if "agent-agency" not in resource:
            return None

        # Get the most recent entry
        entries = data["entries"]
        if not entries:
            return None

        latest_entry = max(entries, key=lambda x: x.get("timestamp", 0))

        return HistoryEntry(
            file_path=resource,
            history_id=latest_entry["id"],
            timestamp=latest_entry["timestamp"],
            source=latest_entry.get("source")
        )

    except (json.JSONDecodeError, KeyError, FileNotFoundError):
        return None

def find_latest_content(history_dir: str, history_id: str) -> Optional[str]:
    """Find the actual content file for a history entry."""
    # Check for files with the history_id followed by extension
    for ext in ['.ts', '.js', '.rs', '.md', '.py', '.json', '.yaml', '.yml', '.toml', '.tsx', '.jsx', '.rs', '.sh', '.sql', '.html', '.scss', '.swift']:
        content_file = os.path.join(history_dir, f"{history_id}{ext}")
        if os.path.exists(content_file):
            return content_file

    # Also check for files that start with the history_id
    history_path = Path(history_dir)
    for file_path in history_path.iterdir():
        if file_path.is_file() and file_path.name.startswith(history_id):
            return str(file_path)

    return None

def analyze_lost_work() -> Dict[str, LostFile]:
    """Analyze all history entries and identify lost work."""
    start_ts, end_ts = parse_timestamp_range()
    history_dirs = scan_history_directories()

    print(f"Scanning {len(history_dirs)} history directories...")
    print(f"Looking for work between {datetime.fromtimestamp(start_ts/1000)} and {datetime.fromtimestamp(end_ts/1000)}")

    lost_files: Dict[str, LostFile] = {}

    for i, history_dir in enumerate(history_dirs):
        if i % 100 == 0:
            print(f"Processed {i}/{len(history_dirs)} directories...")

        entry = parse_history_entry(history_dir)
        if not entry:
            continue

        # Check if this entry is within our lost period
        if not (start_ts <= entry.timestamp <= end_ts):
            continue

        relative_path = entry.relative_path

        # Find the content file
        content_path = find_latest_content(history_dir, entry.history_id)

        if relative_path not in lost_files:
            lost_files[relative_path] = LostFile(
                relative_path=relative_path,
                last_modified=entry.datetime,
                history_entries=[entry],
                latest_content_path=content_path
            )
        else:
            # Update if this is more recent
            if entry.timestamp > lost_files[relative_path].last_modified.timestamp() * 1000:
                lost_files[relative_path].last_modified = entry.datetime
                lost_files[relative_path].latest_content_path = content_path
            lost_files[relative_path].history_entries.append(entry)

    return lost_files

def generate_report(lost_files: Dict[str, LostFile]) -> str:
    """Generate a comprehensive report of lost work."""
    report_lines = []
    report_lines.append("# Lost Work Recovery Report")
    report_lines.append("")
    report_lines.append("## Summary")
    report_lines.append(f"- **Lost Period**: October 19, 2025 (10:44 PM) - October 21, 2025 (1:03 AM)")
    report_lines.append(f"- **Files with Lost Work**: {len(lost_files)}")
    report_lines.append("")

    # Group by file type
    file_types = {}
    for file_path, lost_file in lost_files.items():
        ext = os.path.splitext(file_path)[1] or "no-extension"
        file_types[ext] = file_types.get(ext, 0) + 1

    report_lines.append("## File Types Affected")
    for ext, count in sorted(file_types.items()):
        report_lines.append(f"- **{ext}**: {count} files")
    report_lines.append("")

    # Sort files by last modification time
    sorted_files = sorted(lost_files.values(), key=lambda x: x.last_modified, reverse=True)

    report_lines.append("## Lost Files (Most Recent First)")
    report_lines.append("")

    for lost_file in sorted_files:
        report_lines.append(f"### `{lost_file.relative_path}`")
        report_lines.append(f"- **Last Modified**: {lost_file.last_modified.strftime('%Y-%m-%d %H:%M:%S')}")
        report_lines.append(f"- **History Entries**: {len(lost_file.history_entries)}")
        if lost_file.latest_content_path:
            report_lines.append(f"- **Content Available**: ✅ ({os.path.basename(lost_file.latest_content_path)})")
        else:
            report_lines.append("- **Content Available**: ❌ (not found)")
        report_lines.append("")

    # Show some example content for the most recently modified files
    report_lines.append("## Sample Lost Content")
    report_lines.append("")

    for i, lost_file in enumerate(sorted_files[:3]):  # Show top 3 most recent
        if lost_file.latest_content_path:
            report_lines.append(f"### {lost_file.relative_path}")
            report_lines.append("```")
            try:
                with open(lost_file.latest_content_path, 'r') as f:
                    content = f.read()
                    # Show first 20 lines or 1000 chars, whichever is smaller
                    preview = content[:1000].split('\n')[:20]
                    report_lines.append('\n'.join(preview))
                    if len(content) > 1000 or len(content.split('\n')) > 20:
                        report_lines.append("... (truncated)")
            except Exception as e:
                report_lines.append(f"Error reading content: {e}")
            report_lines.append("```")
            report_lines.append("")

    return '\n'.join(report_lines)

def main():
    """Main execution function."""
    print("🔍 Analyzing Cursor history for lost work...")
    print()

    lost_files = analyze_lost_work()

    print(f"\n📊 Found {len(lost_files)} files with lost work!")
    print()

    # Generate and save report
    report = generate_report(lost_files)
    report_path = "/Users/darianrosebrook/Desktop/Projects/agent-agency/lost-work-report.md"

    with open(report_path, 'w') as f:
        f.write(report)

    print(f"📄 Report saved to: {report_path}")
    print()

    # Print summary
    print("🎯 Top 5 Most Recently Modified Lost Files:")
    sorted_files = sorted(lost_files.values(), key=lambda x: x.last_modified, reverse=True)
    for i, lost_file in enumerate(sorted_files[:5]):
        status = "✅" if lost_file.latest_content_path else "❌"
        print(f"  {i+1}. {status} {lost_file.relative_path} ({lost_file.last_modified.strftime('%m/%d %H:%M')})")

    print()
    print("🚀 Next Steps:")
    print("1. Review the detailed report")
    print("2. Restore files you want to keep")
    print("3. Commit the recovered work")

if __name__ == "__main__":
    main()
