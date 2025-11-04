#!/usr/bin/env python3
"""
Analyze Rust compilation errors by crate and create a prioritized list.
Extracts error counts, error types, and identifies fixable patterns.
"""

import re
import json
import subprocess
import sys
from pathlib import Path
from collections import defaultdict
from typing import Dict, List, Tuple

def run_cargo_check(workspace_path: Path) -> Tuple[str, int]:
    """Run cargo check and return output with exit code."""
    result = subprocess.run(
        ["cargo", "check", "--message-format=json"],
        cwd=workspace_path,
        capture_output=True,
        text=True,
    )
    return result.stdout, result.returncode

def parse_cargo_json_output(output: str) -> Dict[str, List[Dict]]:
    """Parse cargo JSON output and group by crate."""
    errors_by_crate: Dict[str, List[Dict]] = defaultdict(list)
    
    for line in output.splitlines():
        if not line.strip():
            continue
        try:
            data = json.loads(line)
            if data.get("reason") == "compiler-message":
                message = data.get("message", {})
                level = message.get("level", "")
                
                if level in ("error", "error-note"):
                    spans = message.get("spans", [])
                    if spans:
                        file_path = spans[0].get("file_name", "")
                        # Extract crate name from path
                        crate_match = re.search(r"([^/]+)/src/", file_path)
                        if crate_match:
                            crate = crate_match.group(1)
                            # Skip dependency crates (only analyze our workspace crates)
                            if crate in ("registry", "alloc", "core", "std", "proc_macro", "test"):
                                continue
                        else:
                            # Try to get from target name
                            crate = message.get("target", {}).get("name", "unknown")
                            # Skip dependency crates
                            if crate in ("registry", "alloc", "core", "std", "proc_macro", "test"):
                                continue
                        
                        code_obj = message.get("code")
                        code_str = code_obj.get("code", "") if code_obj else ""
                        
                        errors_by_crate[crate].append({
                            "level": level,
                            "message": message.get("rendered", ""),
                            "code": code_str,
                            "file": file_path,
                            "line": spans[0].get("line_start", 0),
                            "col": spans[0].get("column_start", 0),
                        })
        except json.JSONDecodeError:
            continue
    
    return dict(errors_by_crate)

def categorize_error(error: Dict) -> str:
    """Categorize error for fixability assessment."""
    code = error.get("code", "")
    message = error.get("message", "").lower()
    
    # Pattern-based categorization
    if "unresolved" in message or "cannot find" in message:
        return "missing_import"
    if "expected" in message and "found" in message:
        return "type_mismatch"
    if "unused" in message:
        return "unused_code"
    if "field" in message and "does not exist" in message:
        return "struct_field"
    if "method" in message and "not found" in message:
        return "method_not_found"
    if "trait bound" in message or "trait" in message and "not satisfied" in message:
        return "trait_bound"
    if "lifetime" in message:
        return "lifetime"
    if "cannot move" in message or "borrow" in message:
        return "ownership"
    if "dead code" in message:
        return "dead_code"
    
    return "other"

def is_fixable_pattern(error: Dict) -> bool:
    """Determine if error can be fixed programmatically."""
    category = categorize_error(error)
    message = error.get("message", "").lower()
    
    # Fixable patterns
    fixable_categories = {
        "unused_code",
        "dead_code",
        "missing_import",  # Can auto-add imports sometimes
    }
    
    # Check for specific fixable patterns
    if "unused import" in message:
        return True
    if "unused variable" in message:
        return True
    if "dead code" in message:
        return True
    
    return category in fixable_categories

def create_priority_list(errors_by_crate: Dict[str, List[Dict]]) -> List[Tuple[str, Dict]]:
    """Create prioritized list of crates to fix."""
    crate_stats = {}
    
    for crate, errors in errors_by_crate.items():
        total_errors = len([e for e in errors if e["level"] == "error"])
        fixable_count = sum(1 for e in errors if is_fixable_pattern(e))
        categories = defaultdict(int)
        
        for error in errors:
            if error["level"] == "error":
                cat = categorize_error(error)
                categories[cat] += 1
        
        crate_stats[crate] = {
            "total_errors": total_errors,
            "fixable": fixable_count,
            "categories": dict(categories),
            "errors": errors,
        }
    
    # Sort by: 1) total errors (desc), 2) fixable ratio (desc), 3) crate name
    def priority_key(item):
        crate, stats = item
        fixable_ratio = stats["fixable"] / max(stats["total_errors"], 1)
        return (-stats["total_errors"], -fixable_ratio, crate)
    
    return sorted(crate_stats.items(), key=priority_key)

def main():
    workspace_path = Path(__file__).parent.parent
    print(f"Analyzing Rust compilation errors in: {workspace_path}")
    print("Running cargo check...\n")
    
    output, exit_code = run_cargo_check(workspace_path)
    errors_by_crate = parse_cargo_json_output(output)
    
    if not errors_by_crate:
        print("✅ No compilation errors found!")
        return 0
    
    priority_list = create_priority_list(errors_by_crate)
    
    print(f"Found errors in {len(errors_by_crate)} crates:\n")
    print("=" * 80)
    
    total_errors = 0
    total_fixable = 0
    
    for crate, stats in priority_list:
        total_errors += stats["total_errors"]
        total_fixable += stats["fixable"]
        
        print(f"\n📦 {crate}")
        print(f"   Total Errors: {stats['total_errors']}")
        print(f"   Fixable: {stats['fixable']} ({stats['fixable']/max(stats['total_errors'],1)*100:.1f}%)")
        
        if stats["categories"]:
            print(f"   Error Categories:")
            for cat, count in sorted(stats["categories"].items(), key=lambda x: -x[1]):
                print(f"      - {cat}: {count}")
    
    print("\n" + "=" * 80)
    print(f"\nSummary:")
    print(f"  Total crates with errors: {len(errors_by_crate)}")
    print(f"  Total errors: {total_errors}")
    print(f"  Potentially fixable: {total_fixable} ({total_fixable/max(total_errors,1)*100:.1f}%)")
    
    # Export detailed JSON
    output_file = workspace_path / "error_analysis.json"
    with open(output_file, "w") as f:
        json.dump({
            "summary": {
                "total_crates": len(errors_by_crate),
                "total_errors": total_errors,
                "total_fixable": total_fixable,
            },
            "crates": {
                crate: {
                    "total_errors": stats["total_errors"],
                    "fixable": stats["fixable"],
                    "categories": stats["categories"],
                    "errors": [
                        {
                            "level": e["level"],
                            "code": e["code"],
                            "message": e["message"][:200],  # Truncate long messages
                            "file": e["file"],
                            "line": e["line"],
                            "category": categorize_error(e),
                            "fixable": is_fixable_pattern(e),
                        }
                        for e in stats["errors"]
                    ],
                }
                for crate, stats in priority_list
            }
        }, f, indent=2)
    
    print(f"\nDetailed analysis saved to: {output_file}")
    
    return exit_code

if __name__ == "__main__":
    sys.exit(main())

