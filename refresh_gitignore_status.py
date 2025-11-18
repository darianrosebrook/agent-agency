#!/usr/bin/env python3
"""
Refresh git ignore status by untracking files that match prohibited patterns.
Files already tracked in git won't be ignored by .gitignore until they're untracked.
"""

import subprocess
import re
import sys
from pathlib import Path

# Patterns that should be ignored (matching our .gitignore patterns)
PROHIBITED_PATTERNS = [
    r'.*[-_]summary\.md$',
    r'.*[-_]status\.md$',
    r'.*[-_]progress\.md$',
    r'.*[-_]assessment\.md$',
    r'.*[-_]analysis\.md$',
    r'.*[-_]investigation\.md$',
    r'.*[-_]evaluation\.md$',
    r'.*[-_]report\.md$',
    r'.*[-_]audit\.md$',
    r'.*[-_]roadmap\.md$',
    r'.*[-_]checklist\.md$',
    r'.*[-_]plan\.md$',
    r'.*[-_]complete\.md$',
    r'.*[-_]completion\.md$',
    r'.*[-_]fix.*\.md$',
    r'.*[-_]implementation.*\.md$',
    r'.*[-_]todo.*\.md$',
    r'.*[-_]session.*\.md$',
    r'.*[-_]reassessment\.md$',
    r'.*[-_]update\.md$',
    r'.*[-_]final.*\.md$',
    r'.*[-_]comparison\.md$',
    r'.*[-_]verification.*\.md$',
    r'.*[-_]migration.*\.md$',
    r'.*[-_]conversion.*\.md$',
    r'.*[-_]compilation.*\.md$',
    r'^session[-_].*\.md$',
    r'^implementation[-_].*\.md$',
    r'^debug[-_].*\.md$',
    r'^todo\.md$',
    r'^stubby\.md$',
]

def matches_prohibited_pattern(filename):
    """Check if filename matches any prohibited pattern (case-insensitive)."""
    filename_lower = filename.lower()
    for pattern in PROHIBITED_PATTERNS:
        if re.match(pattern, filename_lower, re.IGNORECASE):
            return True
    return False

def get_tracked_files():
    """Get list of currently tracked markdown files."""
    try:
        result = subprocess.run(
            ['git', 'ls-files', '*.md'],
            capture_output=True,
            text=True,
            check=True
        )
        return [line.strip() for line in result.stdout.splitlines() if line.strip()]
    except subprocess.CalledProcessError:
        print("Error: Not a git repository or git command failed")
        sys.exit(1)

def get_file_status(filepath):
    """Check git status of a file."""
    try:
        result = subprocess.run(
            ['git', 'status', '--porcelain', '--', filepath],
            capture_output=True,
            text=True,
            check=True
        )
        return result.stdout.strip()
    except subprocess.CalledProcessError:
        return ''

def untrack_file(filepath, dry_run=True):
    """Untrack a file from git (remove from index but keep in filesystem)."""
    if dry_run:
        print(f"  [DRY RUN] Would untrack: {filepath}")
        return True
    
    try:
        subprocess.run(
            ['git', 'rm', '--cached', filepath],
            capture_output=True,
            check=True
        )
        print(f"  ✓ Untracked: {filepath}")
        return True
    except subprocess.CalledProcessError as e:
        print(f"  ✗ Error untracking {filepath}: {e}")
        return False

def main():
    import argparse
    
    parser = argparse.ArgumentParser(
        description='Refresh git ignore status by untracking files matching prohibited patterns'
    )
    parser.add_argument(
        '--dry-run',
        action='store_true',
        help='Show what would be done without actually doing it'
    )
    parser.add_argument(
        '--inventory-file',
        default='markdown_files_inventory.txt',
        help='Path to markdown files inventory file'
    )
    args = parser.parse_args()
    
    print("=" * 80)
    print("GIT IGNORE STATUS REFRESH")
    print("=" * 80)
    
    # Get tracked files
    print("\n1. Checking tracked markdown files...")
    tracked_files = get_tracked_files()
    print(f"   Found {len(tracked_files)} tracked markdown files")
    
    # Load inventory file
    print(f"\n2. Loading inventory from {args.inventory_file}...")
    try:
        with open(args.inventory_file, 'r') as f:
            inventory_files = [line.strip() for line in f if line.strip()]
        print(f"   Found {len(inventory_files)} files in inventory")
    except FileNotFoundError:
        print(f"   ⚠ Warning: {args.inventory_file} not found, using tracked files only")
        inventory_files = tracked_files
    
    # Find files that match prohibited patterns
    print("\n3. Identifying files matching prohibited patterns...")
    prohibited_files = []
    
    for filepath in tracked_files:
        filename = Path(filepath).name
        if matches_prohibited_pattern(filename):
            # Check if file is in docs-status/ (which is already ignored)
            if 'docs-status/' not in filepath:
                prohibited_files.append(filepath)
    
    print(f"   Found {len(prohibited_files)} tracked files matching prohibited patterns")
    
    if not prohibited_files:
        print("\n✓ No files need to be untracked!")
        return
    
    # Group files by status
    print("\n4. Analyzing file statuses...")
    staged = []
    modified = []
    unmodified = []
    
    for filepath in prohibited_files:
        status = get_file_status(filepath)
        if status.startswith('A ') or status.startswith('M '):
            staged.append(filepath)
        elif status.startswith(' M'):
            modified.append(filepath)
        elif status.startswith('??'):
            # Untracked but exists
            continue
        else:
            unmodified.append(filepath)
    
    print(f"   Staged: {len(staged)}")
    print(f"   Modified: {len(modified)}")
    print(f"   Unmodified: {len(unmodified)}")
    
    # Show what will be untracked
    print("\n5. Files to untrack:")
    print("-" * 80)
    
    if staged:
        print(f"\n  Staged files ({len(staged)}):")
        for f in sorted(staged)[:10]:
            print(f"    {f}")
        if len(staged) > 10:
            print(f"    ... and {len(staged) - 10} more")
    
    if modified:
        print(f"\n  Modified files ({len(modified)}):")
        for f in sorted(modified)[:10]:
            print(f"    {f}")
        if len(modified) > 10:
            print(f"    ... and {len(modified) - 10} more")
    
    if unmodified:
        print(f"\n  Unmodified files ({len(unmodified)}):")
        for f in sorted(unmodified)[:10]:
            print(f"    {f}")
        if len(unmodified) > 10:
            print(f"    ... and {len(unmodified) - 10} more")
    
    # Execute untracking
    if args.dry_run:
        print("\n" + "=" * 80)
        print("DRY RUN - No changes made")
        print("=" * 80)
        print("\nRun without --dry-run to actually untrack these files")
    else:
        print("\n6. Untracking files...")
        print("-" * 80)
        success_count = 0
        for filepath in prohibited_files:
            if untrack_file(filepath, dry_run=False):
                success_count += 1
        
        print("\n" + "=" * 80)
        print(f"COMPLETE: Untracked {success_count}/{len(prohibited_files)} files")
        print("=" * 80)
        print("\nNext steps:")
        print("  1. Review: git status")
        print("  2. Files matching .gitignore patterns are now ignored")
        print("  3. Legitimate files can be re-added with: git add <file>")
        print("  4. Commit the changes: git commit -m 'Untrack temporal documentation files'")

if __name__ == '__main__':
    main()





