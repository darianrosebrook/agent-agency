#!/usr/bin/env python3
"""
Prioritize and distribute Rust compilation errors across 3 workers.
Consolidates errors from existing documentation and creates work assignments.
"""

import json
import re
from pathlib import Path
from typing import Dict, List, Tuple
from collections import defaultdict

def load_compilation_errors() -> Dict[str, Dict]:
    """Load compilation errors from existing documentation."""
    v3_path = Path(__file__).parent.parent
    
    errors = {}
    
    # Load agent-orchestration errors
    orchestration_file = v3_path / "agent-orchestration" / "COMPILATION_ERRORS.md"
    if orchestration_file.exists():
        with open(orchestration_file, "r") as f:
            content = f.read()
            error_count_match = re.search(r"(\d+)\s+compilation errors?", content, re.IGNORECASE)
            error_count = int(error_count_match.group(1)) if error_count_match else 0
            
            # Extract error categories
            categories = []
            for match in re.finditer(r"## Error Category (\d+):\s*(.+)", content):
                cat_num = match.group(1)
                cat_name = match.group(2).strip()
                categories.append((int(cat_num), cat_name))
            
            errors["agent-orchestration"] = {
                "total_errors": error_count,
                "categories": categories,
                "file": str(orchestration_file),
                "priority": "HIGH",  # 88 errors - blocking
                "complexity": "HIGH",
                "estimated_time": "8-12 hours",
            }
    
    # Load system-quality-security errors
    quality_security_file = v3_path / "system-quality-security" / "COMPILATION_ERRORS.md"
    if quality_security_file.exists():
        with open(quality_security_file, "r") as f:
            content = f.read()
            error_count_match = re.search(r"(\d+)\s+compilation errors?", content, re.IGNORECASE)
            error_count = int(error_count_match.group(1)) if error_count_match else 0
            
            errors["system-quality-security"] = {
                "total_errors": error_count,
                "categories": [],
                "file": str(quality_security_file),
                "priority": "MEDIUM",  # 5 errors - manageable
                "complexity": "MEDIUM",
                "estimated_time": "2-3 hours",
            }
    
    # Check for workspace dependency errors
    workspace_error = v3_path.parent.parent / "cargo-check-errors.json"
    if workspace_error.exists():
        with open(workspace_error, "r") as f:
            content = f.read()
            if "candle-core" in content:
                errors["workspace-dependencies"] = {
                    "total_errors": 1,
                    "categories": [("dependency", "candle-core workspace dependency")],
                    "file": str(workspace_error),
                    "priority": "CRITICAL",  # Blocks entire workspace
                    "complexity": "LOW",
                    "estimated_time": "30 minutes",
                    "description": "candle-core dependency missing from workspace.dependencies"
                }
    
    return errors

def categorize_and_prioritize(errors: Dict[str, Dict]) -> List[Tuple[str, Dict, int]]:
    """Categorize errors and assign priority scores."""
    prioritized = []
    
    for crate, error_data in errors.items():
        # Priority scoring: 10 = critical, 1 = low
        priority_map = {
            "CRITICAL": 10,
            "HIGH": 8,
            "MEDIUM": 5,
            "LOW": 2,
        }
        
        priority_score = priority_map.get(error_data["priority"], 5)
        
        # Complexity multiplier
        complexity_map = {
            "HIGH": 1.5,
            "MEDIUM": 1.0,
            "LOW": 0.5,
        }
        
        complexity_mult = complexity_map.get(error_data.get("complexity", "MEDIUM"), 1.0)
        
        # Final score = priority * complexity * error_count (normalized)
        error_count = error_data["total_errors"]
        final_score = priority_score * complexity_mult * (1 + error_count / 10)
        
        prioritized.append((crate, error_data, final_score))
    
    # Sort by score (descending)
    prioritized.sort(key=lambda x: x[2], reverse=True)
    
    return prioritized

def distribute_work(prioritized: List[Tuple[str, Dict, int]], num_workers: int = 3) -> Dict[int, List[Tuple[str, Dict]]]:
    """Distribute work evenly across workers using priority weighting."""
    workers = defaultdict(list)
    worker_loads = [0.0] * num_workers
    
    # For agent-orchestration, split categories across workers
    for crate, error_data, score in prioritized:
        if crate == "agent-orchestration" and error_data.get("categories"):
            # Split agent-orchestration categories across workers
            categories = error_data["categories"]
            category_errors = error_data["total_errors"] // len(categories) if categories else 0
            
            for i, (cat_num, cat_name) in enumerate(categories):
                worker_idx = i % num_workers
                # Create a sub-task for this category
                category_data = {
                    "total_errors": category_errors,
                    "categories": [(cat_num, cat_name)],
                    "file": error_data["file"],
                    "priority": error_data["priority"],
                    "complexity": error_data["complexity"],
                    "estimated_time": f"{error_data['total_errors'] // len(categories) * 10 // 60} minutes",
                    "category_name": cat_name,
                }
                workers[worker_idx].append((f"{crate} - {cat_name}", category_data))
                worker_loads[worker_idx] += score / len(categories)
        else:
            # Assign to worker with lowest current load
            worker_idx = min(range(num_workers), key=lambda i: worker_loads[i])
            workers[worker_idx].append((crate, error_data))
            worker_loads[worker_idx] += score
    
    return dict(workers)

def create_work_report(errors: Dict[str, Dict], distribution: Dict[int, List[Tuple[str, Dict]]]) -> str:
    """Create a comprehensive work report."""
    report = []
    report.append("=" * 80)
    report.append("V3 BUILD ERROR ANALYSIS & WORK DISTRIBUTION")
    report.append("=" * 80)
    report.append("")
    
    # Summary
    total_errors = sum(e["total_errors"] for e in errors.values())
    report.append(f"SUMMARY")
    report.append(f"  Total Crates with Errors: {len(errors)}")
    report.append(f"  Total Errors: {total_errors}")
    report.append(f"  Workers: 3")
    report.append("")
    
    # Full error list by priority
    prioritized = categorize_and_prioritize(errors)
    report.append("=" * 80)
    report.append("ERRORS BY PRIORITY")
    report.append("=" * 80)
    report.append("")
    
    for crate, error_data, score in prioritized:
        report.append(f"📦 {crate.upper()}")
        report.append(f"   Priority: {error_data['priority']} (Score: {score:.1f})")
        report.append(f"   Total Errors: {error_data['total_errors']}")
        report.append(f"   Complexity: {error_data.get('complexity', 'MEDIUM')}")
        report.append(f"   Estimated Time: {error_data.get('estimated_time', 'Unknown')}")
        if error_data.get('categories'):
            report.append(f"   Categories: {len(error_data['categories'])}")
        report.append(f"   Documentation: {error_data['file']}")
        report.append("")
    
    # Worker assignments
    report.append("=" * 80)
    report.append("WORKER ASSIGNMENTS")
    report.append("=" * 80)
    report.append("")
    
    for worker_id, assignments in sorted(distribution.items()):
        report.append(f"WORKER {worker_id + 1}")
        report.append("-" * 40)
        
        total_errors = sum(e["total_errors"] for _, e in assignments)
        total_time = []
        
        for crate, error_data in assignments:
            report.append(f"  • {crate}")
            report.append(f"    - {error_data['total_errors']} errors")
            report.append(f"    - Priority: {error_data['priority']}")
            report.append(f"    - Time: {error_data.get('estimated_time', 'Unknown')}")
            if error_data.get('categories'):
                for cat_num, cat_name in error_data['categories'][:3]:  # Show first 3
                    report.append(f"    - Category {cat_num}: {cat_name}")
            report.append("")
            
            # Parse time estimate
            time_str = error_data.get('estimated_time', '0')
            if 'hours' in time_str:
                match = re.search(r'(\d+)-(\d+)', time_str)
                if match:
                    total_time.append((int(match.group(1)), int(match.group(2))))
        
        # Estimate total time
        if total_time:
            min_time = sum(t[0] for t in total_time)
            max_time = sum(t[1] for t in total_time)
            report.append(f"  Total: {total_errors} errors, ~{min_time}-{max_time} hours")
        report.append("")
    
    # Detailed breakdown
    report.append("=" * 80)
    report.append("DETAILED BREAKDOWN")
    report.append("=" * 80)
    report.append("")
    
    # agent-orchestration details
    if "agent-orchestration" in errors:
        orch = errors["agent-orchestration"]
        report.append("agent-orchestration (88 errors)")
        report.append("")
        report.append("Key Categories:")
        for cat_num, cat_name in orch.get('categories', [])[:10]:
            report.append(f"  {cat_num}. {cat_name}")
        report.append("")
        report.append("Recommended Fix Order:")
        report.append("  1. Syntax error (orphaned doc comment) - 5 min")
        report.append("  2. Duplicate type definitions - 15 min")
        report.append("  3. Duplicate imports - 10 min")
        report.append("  4. Missing dependency (agent_data_processing) - 30 min")
        report.append("  5. Missing types (TaskType, ExecutionMode) - 30 min")
        report.append("  6. Type mismatches - 2 hours")
        report.append("  7. Wrong function signatures - 2 hours")
        report.append("  8. Missing struct fields - 1 hour")
        report.append("  9. Missing enum variants - 1 hour")
        report.append("  10. Trait implementation issues - 1 hour")
        report.append("")
    
    # system-quality-security details
    if "system-quality-security" in errors:
        qs = errors["system-quality-security"]
        report.append("system-quality-security (5 errors)")
        report.append("")
        report.append("Fix Order:")
        report.append("  1. Borrow checker issue - 30 min")
        report.append("  2. Missing rand_distr::Laplace - 1 hour")
        report.append("  3. Type annotations for BoundKey - 30 min")
        report.append("")
    
    # workspace dependencies
    if "workspace-dependencies" in errors:
        ws = errors["workspace-dependencies"]
        report.append("workspace-dependencies (1 error)")
        report.append("")
        report.append("Fix:")
        report.append("  Add candle-core to workspace.dependencies in Cargo.toml")
        report.append("")
    
    return "\n".join(report)

def main():
    """Main entry point."""
    v3_path = Path(__file__).parent.parent
    
    print("Analyzing compilation errors...")
    errors = load_compilation_errors()
    
    if not errors:
        print("No compilation errors found in documentation.")
        return
    
    prioritized = categorize_and_prioritize(errors)
    distribution = distribute_work(prioritized, num_workers=3)
    
    # Generate report
    report = create_work_report(errors, distribution)
    
    # Save report
    report_file = v3_path / "BUILD_ERRORS_PRIORITY_REPORT.md"
    with open(report_file, "w") as f:
        f.write(report)
    
    print(f"\n✅ Report generated: {report_file}\n")
    print(report)
    
    # Save JSON for programmatic access
    json_file = v3_path / "build_errors_priority.json"
    with open(json_file, "w") as f:
        json.dump({
            "summary": {
                "total_crates": len(errors),
                "total_errors": sum(e["total_errors"] for e in errors.values()),
                "workers": 3,
            },
            "errors": {
                crate: {
                    "total_errors": data["total_errors"],
                    "priority": data["priority"],
                    "complexity": data.get("complexity", "MEDIUM"),
                    "estimated_time": data.get("estimated_time", "Unknown"),
                    "categories": [{"id": cat[0], "name": cat[1]} for cat in data.get("categories", [])],
                }
                for crate, data in errors.items()
            },
            "distribution": {
                f"worker_{i+1}": [
                    {
                        "crate": crate,
                        "total_errors": data["total_errors"],
                        "priority": data["priority"],
                        "estimated_time": data.get("estimated_time", "Unknown"),
                    }
                    for crate, data in assignments
                ]
                for i, assignments in sorted(distribution.items())
            },
        }, f, indent=2)
    
    print(f"\n✅ JSON data saved: {json_file}")

if __name__ == "__main__":
    main()

