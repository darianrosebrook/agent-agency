#!/usr/bin/env python3
"""Parse cargo warnings and organize them for cleanup."""

import re
import subprocess
from collections import defaultdict
from pathlib import Path

def parse_cargo_warnings():
    """Run cargo check and parse warnings."""
    result = subprocess.run(
        ["cargo", "check", "--workspace"],
        capture_output=True,
        text=True,
        cwd=Path(__file__).parent.parent
    )
    
    warnings = []
    current_warning = None
    current_file = None
    current_crate = None
    
    for line in result.stderr.split('\n'):
        # Match crate name: warning: `crate-name` (lib) generated N warnings
        crate_match = re.search(r'warning: `([^`]+)`', line)
        if crate_match:
            current_crate = crate_match.group(1)
        
        # Match warning message
        if line.strip().startswith('warning:'):
            if current_warning:
                warnings.append({
                    'crate': current_crate or 'unknown',
                    'warning': current_warning,
                    'file': current_file
                })
            current_warning = line.strip()
            current_file = None
        
        # Match file location: --> file.rs:line:col
        file_match = re.search(r'--> (.+?):(\d+):(\d+)', line)
        if file_match:
            current_file = f"{file_match.group(1)}:{file_match.group(2)}"
    
    # Add last warning
    if current_warning:
        warnings.append({
            'crate': current_crate or 'unknown',
            'warning': current_warning,
            'file': current_file
        })
    
    return warnings

def categorize_warnings(warnings):
    """Categorize warnings by type."""
    categories = defaultdict(list)
    
    for w in warnings:
        warning_text = w['warning']
        
        if 'unused import' in warning_text:
            categories['unused_imports'].append(w)
        elif 'unused variable' in warning_text:
            categories['unused_variables'].append(w)
        elif 'never used' in warning_text or 'never constructed' in warning_text:
            categories['unused_items'].append(w)
        elif 'never read' in warning_text:
            categories['unused_assignments'].append(w)
        elif 'does not need to be mutable' in warning_text:
            categories['unnecessary_mut'].append(w)
        elif 'deprecated' in warning_text.lower():
            categories['deprecated'].append(w)
        elif 'ambiguous glob' in warning_text:
            categories['ambiguous_glob'].append(w)
        elif 'unexpected `cfg`' in warning_text:
            categories['cfg_issues'].append(w)
        else:
            categories['other'].append(w)
    
    return categories

def group_by_crate(warnings):
    """Group warnings by crate."""
    by_crate = defaultdict(list)
    for w in warnings:
        by_crate[w['crate']].append(w)
    return by_crate

def main():
    print("Parsing cargo warnings...")
    warnings = parse_cargo_warnings()
    
    print(f"\nTotal warnings found: {len(warnings)}")
    
    categories = categorize_warnings(warnings)
    by_crate = group_by_crate(warnings)
    
    print("\n=== Warnings by Category ===")
    for cat, items in sorted(categories.items(), key=lambda x: -len(x[1])):
        print(f"\n{cat}: {len(items)} warnings")
    
    print("\n=== Warnings by Crate ===")
    for crate, items in sorted(by_crate.items(), key=lambda x: -len(x[1])):
        print(f"\n{crate}: {len(items)} warnings")
    
    # Create work assignments
    print("\n=== Work Assignment for 3 Workers ===")
    
    # Sort crates by warning count
    sorted_crates = sorted(by_crate.items(), key=lambda x: -len(x[1]))
    
    # Split into 3 groups
    worker1 = []
    worker2 = []
    worker3 = []
    
    for i, (crate, items) in enumerate(sorted_crates):
        if i % 3 == 0:
            worker1.append((crate, items))
        elif i % 3 == 1:
            worker2.append((crate, items))
        else:
            worker3.append((crate, items))
    
    for worker_num, worker_tasks in enumerate([worker1, worker2, worker3], 1):
        total = sum(len(items) for _, items in worker_tasks)
        print(f"\n--- Worker {worker_num} ({total} warnings) ---")
        for crate, items in worker_tasks:
            print(f"  {crate}: {len(items)} warnings")
            # Show first few examples
            for w in items[:3]:
                file_part = f" ({w['file']})" if w['file'] else ""
                print(f"    - {w['warning'][:80]}{file_part}")
            if len(items) > 3:
                print(f"    ... and {len(items) - 3} more")
    
    # Write detailed report
    report_file = Path(__file__).parent.parent / "WARNINGS_CLEANUP.md"
    with open(report_file, 'w') as f:
        f.write("# Cargo Warnings Cleanup Plan\n\n")
        f.write(f"Total warnings: {len(warnings)}\n\n")
        
        f.write("## By Category\n\n")
        for cat, items in sorted(categories.items(), key=lambda x: -len(x[1])):
            f.write(f"### {cat.replace('_', ' ').title()} ({len(items)} warnings)\n\n")
            for w in items[:10]:  # Show first 10
                file_part = f" - `{w['file']}`" if w['file'] else ""
                f.write(f"- {w['warning']}{file_part}\n")
            if len(items) > 10:
                f.write(f"- ... and {len(items) - 10} more\n")
            f.write("\n")
        
        f.write("\n## Work Assignment\n\n")
        for worker_num, worker_tasks in enumerate([worker1, worker2, worker3], 1):
            total = sum(len(items) for _, items in worker_tasks)
            f.write(f"### Worker {worker_num} ({total} warnings)\n\n")
            for crate, items in worker_tasks:
                f.write(f"#### {crate} ({len(items)} warnings)\n\n")
                for w in items:
                    file_part = f" - `{w['file']}`" if w['file'] else ""
                    f.write(f"- {w['warning']}{file_part}\n")
                f.write("\n")
    
    print(f"\nDetailed report written to: {report_file}")

if __name__ == "__main__":
    main()

