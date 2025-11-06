#!/usr/bin/env python3
"""
Analyze Rust build errors from cargo check and prioritize them for distribution.
"""

import subprocess
import re
import json
from pathlib import Path
from collections import defaultdict
from typing import Dict, List, Tuple

def run_cargo_check() -> str:
    """Run cargo check and return output."""
    v3_path = Path(__file__).parent.parent
    result = subprocess.run(
        ["cargo", "check", "--message-format=short"],
        cwd=v3_path,
        capture_output=True,
        text=True
    )
    return result.stderr + result.stdout

def parse_errors(output: str) -> Dict[str, List[Dict]]:
    """Parse cargo check output and extract errors by crate and binary."""
    errors_by_target = defaultdict(list)
    current_target = None
    current_error = None
    
    lines = output.split('\n')
    
    for i, line in enumerate(lines):
        # Match crate compilation errors with binary name
        crate_match = re.match(r'error: could not compile `([^`]+)` \(bin "([^"]+)"\)', line)
        if crate_match:
            crate_name = crate_match.group(1)
            bin_name = crate_match.group(2)
            current_target = f"{crate_name}::{bin_name}"
            
            # Try to extract error count from the message
            error_count_match = re.search(r'(\d+)\s+previous\s+errors?', line)
            error_count = int(error_count_match.group(1)) if error_count_match else 0
            
            errors_by_target[current_target].append({
                "type": "compilation_failed",
                "count": error_count,
                "message": line.strip(),
                "crate": crate_name,
                "binary": bin_name,
            })
            continue
        
        # Match crate compilation errors without binary
        crate_match = re.match(r'error: could not compile `([^`]+)`', line)
        if crate_match:
            crate_name = crate_match.group(1)
            current_target = crate_name
            
            # Try to extract error count from the message
            error_count_match = re.search(r'(\d+)\s+previous\s+errors?', line)
            error_count = int(error_count_match.group(1)) if error_count_match else 0
            
            errors_by_target[current_target].append({
                "type": "compilation_failed",
                "count": error_count,
                "message": line.strip(),
                "crate": crate_name,
            })
            continue
        
        # Match individual error messages with error codes
        error_match = re.match(r'error\[([^\]]+)\]: (.+)', line)
        if error_match:
            error_code = error_match.group(1)
            error_msg = error_match.group(2).strip()
            
            if current_target:
                errors_by_target[current_target].append({
                    "type": "error",
                    "code": error_code,
                    "message": error_msg,
                    "full_line": line.strip(),
                })
            continue
        
        # Match individual error messages without codes
        error_match = re.match(r'error: (.+)', line)
        if error_match:
            error_msg = error_match.group(1).strip()
            
            if current_target:
                errors_by_target[current_target].append({
                    "type": "error",
                    "message": error_msg,
                    "full_line": line.strip(),
                })
            continue
        
        # Match error locations (file:line:column)
        location_match = re.match(r'\s+-->\s+([^:]+):(\d+):(\d+)', line)
        if location_match and current_target:
            file_path = location_match.group(1)
            line_num = location_match.group(2)
            col_num = location_match.group(3)
            
            # Add location to the last error
            if errors_by_target[current_target]:
                errors_by_target[current_target][-1]["file"] = file_path
                errors_by_target[current_target][-1]["line"] = line_num
                errors_by_target[current_target][-1]["column"] = col_num
    
    return dict(errors_by_target)

def categorize_error(error_msg: str) -> Tuple[str, int]:
    """Categorize error and return (category, priority_score)."""
    error_lower = error_msg.lower()
    
    # Priority 10 (Critical - blocks everything)
    if "could not compile" in error_lower:
        return ("compilation_failure", 10)
    
    # Priority 9 (High - missing dependencies)
    if any(x in error_lower for x in ["cannot find", "no method named", "not found in"]):
        return ("missing_item", 9)
    
    # Priority 8 (High - type mismatches)
    if any(x in error_lower for x in ["mismatched types", "expected", "found", "type mismatch"]):
        return ("type_mismatch", 8)
    
    # Priority 7 (High - trait bounds)
    if any(x in error_lower for x in ["trait bound", "doesn't implement", "the trait"]):
        return ("trait_bound", 7)
    
    # Priority 6 (Medium - borrow checker)
    if any(x in error_lower for x in ["cannot borrow", "borrow checker", "moved value"]):
        return ("borrow_checker", 6)
    
    # Priority 5 (Medium - lifetime issues)
    if any(x in error_lower for x in ["lifetime", "outlives", "does not live long enough"]):
        return ("lifetime", 5)
    
    # Priority 4 (Medium - missing fields)
    if any(x in error_lower for x in ["missing field", "no field", "struct missing"]):
        return ("missing_field", 4)
    
    # Priority 3 (Low - clippy/style)
    if any(x in error_lower for x in ["unused", "dead code", "unused variable"]):
        return ("unused_code", 3)
    
    # Default
    return ("other", 5)

def prioritize_crates(errors_by_target: Dict[str, List[Dict]]) -> List[Tuple[str, Dict, float]]:
    """Prioritize targets (crates/binaries) based on error count and types."""
    prioritized = []
    
    # Group by crate
    errors_by_crate = defaultdict(lambda: defaultdict(list))
    for target, errors in errors_by_target.items():
        if "::" in target:
            crate, binary = target.split("::", 1)
        else:
            crate = target
            binary = None
        
        errors_by_crate[crate][binary] = errors
    
    for crate, binaries in errors_by_crate.items():
        # Aggregate errors across all binaries
        all_errors = []
        for binary_errors in binaries.values():
            all_errors.extend(binary_errors)
        
        # Count errors by type
        error_counts = defaultdict(int)
        error_codes = defaultdict(int)
        total_errors = 0
        max_priority = 0
        
        for error in all_errors:
            if error.get("count"):
                total_errors += error["count"]
            else:
                total_errors += 1
            
            error_msg = error.get("message", "")
            error_code = error.get("code", "no_code")
            error_codes[error_code] += 1
            
            category, priority = categorize_error(error_msg)
            error_counts[category] += 1
            max_priority = max(max_priority, priority)
        
        # Calculate priority score
        # Base score from highest priority error
        # Weighted by total error count
        priority_score = max_priority * (1 + total_errors / 10)
        
        crate_data = {
            "total_errors": total_errors,
            "error_counts": dict(error_counts),
            "error_codes": dict(error_codes),
            "max_priority": max_priority,
            "binaries": dict(binaries),
            "errors": all_errors,
        }
        
        prioritized.append((crate, crate_data, priority_score))
    
    # Sort by priority score (descending)
    prioritized.sort(key=lambda x: x[2], reverse=True)
    
    return prioritized

def distribute_work(prioritized: List[Tuple[str, Dict, float]], num_workers: int = 3) -> Dict[int, List[Tuple[str, Dict]]]:
    """Distribute work evenly across workers."""
    workers = defaultdict(list)
    worker_loads = [0.0] * num_workers
    
    for crate, crate_data, score in prioritized:
        # Assign to worker with lowest current load
        worker_idx = min(range(num_workers), key=lambda i: worker_loads[i])
        workers[worker_idx].append((crate, crate_data))
        worker_loads[worker_idx] += score
    
    return dict(workers)

def generate_report(
    errors_by_target: Dict[str, List[Dict]],
    prioritized: List[Tuple[str, Dict, float]],
    distribution: Dict[int, List[Tuple[str, Dict]]]
) -> str:
    """Generate a comprehensive report."""
    report = []
    report.append("=" * 80)
    report.append("V3 BUILD ERROR ANALYSIS & WORK DISTRIBUTION")
    report.append("=" * 80)
    report.append("")
    
    # Summary
    total_errors = sum(len(errors) for errors in errors_by_target.values())
    total_targets = len(errors_by_target)
    total_crates = len(prioritized)
    
    # Count unique error instances
    unique_error_count = 0
    for _, crate_data, _ in prioritized:
        unique_error_count += crate_data['total_errors']
    
    report.append("SUMMARY")
    report.append(f"  Total Crates with Errors: {total_crates}")
    report.append(f"  Total Targets (crates/binaries): {total_targets}")
    report.append(f"  Total Unique Errors: {unique_error_count}")
    report.append(f"  Workers: 3")
    report.append("")
    
    # Errors by crate (prioritized)
    report.append("=" * 80)
    report.append("ERRORS BY CRATE (PRIORITIZED)")
    report.append("=" * 80)
    report.append("")
    
    for crate, crate_data, score in prioritized:
        report.append(f"CRATE: {crate}")
        report.append(f"  Priority Score: {score:.1f}")
        report.append(f"  Total Errors: {crate_data['total_errors']}")
        report.append(f"  Max Priority: {crate_data['max_priority']}/10")
        
        # Show binaries
        if crate_data.get('binaries'):
            report.append(f"  Binaries with Errors:")
            for binary, errors in crate_data['binaries'].items():
                error_count = sum(e.get('count', 1) for e in errors)
                report.append(f"    - {binary}: {error_count} errors")
            report.append("")
        
        report.append(f"  Error Categories:")
        for category, count in sorted(crate_data['error_counts'].items(), key=lambda x: -x[1]):
            report.append(f"    - {category}: {count}")
        
        report.append(f"  Error Codes:")
        for code, count in sorted(crate_data.get('error_codes', {}).items(), key=lambda x: -x[1])[:10]:
            report.append(f"    - {code}: {count}")
        
        # Show sample errors
        sample_errors = crate_data['errors'][:5]
        if sample_errors:
            report.append(f"  Sample Errors:")
            for error in sample_errors:
                code = error.get('code', '')
                msg = error.get('message', '')
                if msg:
                    if code:
                        report.append(f"    - [{code}] {msg[:80]}")
                    else:
                        report.append(f"    - {msg[:80]}")
                if error.get('file'):
                    report.append(f"      -> {error['file']}:{error.get('line', '?')}")
        report.append("")
    
    # Worker assignments
    report.append("=" * 80)
    report.append("WORKER ASSIGNMENTS")
    report.append("=" * 80)
    report.append("")
    
    for worker_id, assignments in sorted(distribution.items()):
        report.append(f"WORKER {worker_id + 1}")
        report.append("-" * 40)
        
        total_errors = sum(data['total_errors'] for _, data in assignments)
        total_load = sum(
            data['max_priority'] * (1 + data['total_errors'] / 10)
            for _, data in assignments
        )
        
        report.append(f"  Total Errors: {total_errors}")
        report.append(f"  Total Load Score: {total_load:.1f}")
        report.append("")
        
        for crate, crate_data in assignments:
            report.append(f"  • {crate}")
            report.append(f"    - {crate_data['total_errors']} errors")
            report.append(f"    - Priority: {crate_data['max_priority']}/10")
            report.append(f"    - Categories: {', '.join(crate_data['error_counts'].keys())}")
            report.append("")
    
    return "\n".join(report)

def main():
    """Main entry point."""
    v3_path = Path(__file__).parent.parent
    
    print("Running cargo check...")
    output = run_cargo_check()
    
    print("Parsing errors...")
    errors_by_target = parse_errors(output)
    
    if not errors_by_target:
        print("✅ No compilation errors found!")
        return
    
    print(f"Found errors in {len(errors_by_target)} target(s)")
    
    print("Prioritizing errors...")
    prioritized = prioritize_crates(errors_by_target)
    
    print("Distributing work across 3 workers...")
    distribution = distribute_work(prioritized, num_workers=3)
    
    # Generate report
    report = generate_report(errors_by_target, prioritized, distribution)
    
    # Save report
    report_file = v3_path / "BUILD_ERRORS_ANALYSIS.md"
    with open(report_file, "w") as f:
        f.write(report)
    
    print(f"\n✅ Report generated: {report_file}\n")
    print(report)
    
    # Save JSON for programmatic access
    json_file = v3_path / "build_errors_analysis.json"
    with open(json_file, "w") as f:
        json.dump({
            "summary": {
                "total_crates": len(prioritized),
                "total_targets": len(errors_by_target),
                "total_error_groups": sum(len(errors) for errors in errors_by_target.values()),
                "total_errors": sum(
                    sum(e.get("count", 1) for e in errors)
                    for errors in errors_by_target.values()
                ),
                "workers": 3,
            },
            "crates": {
                crate: {
                    "total_errors": data['total_errors'],
                    "max_priority": data['max_priority'],
                    "error_counts": data['error_counts'],
                    "sample_errors": [
                        {
                            "message": e.get("message", "")[:200],
                            "file": e.get("file"),
                            "line": e.get("line"),
                        }
                        for e in data['errors'][:5]
                    ],
                }
                for crate, data, _ in prioritized
            },
            "distribution": {
                f"worker_{i+1}": [
                    {
                        "crate": crate,
                        "total_errors": data['total_errors'],
                        "max_priority": data['max_priority'],
                        "error_categories": list(data['error_counts'].keys()),
                    }
                    for crate, data in assignments
                ]
                for i, assignments in sorted(distribution.items())
            },
        }, f, indent=2)
    
    print(f"\n✅ JSON data saved: {json_file}")

if __name__ == "__main__":
    main()

