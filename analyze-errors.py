#!/usr/bin/env python3
"""
Analyze Rust compilation errors and categorize them for parallel fixing.
"""

import json
import sys
from collections import defaultdict, Counter
from typing import Dict, List, Tuple

def load_errors_from_file(filepath: str) -> List[Dict]:
    """Load errors from cargo check JSON output."""
    errors = []
    try:
        with open(filepath, 'r') as f:
            for line in f:
                line = line.strip()
                if line:
                    try:
                        data = json.loads(line)
                        if data.get('reason') == 'compiler-message':
                            message = data.get('message', {})
                            if message.get('level') == 'error':
                                errors.append(data)
                    except json.JSONDecodeError:
                        continue
    except FileNotFoundError:
        print(f"Error: File {filepath} not found")
        sys.exit(1)

    return errors

def categorize_errors(errors: List[Dict]) -> Dict[str, List[Dict]]:
    """Categorize errors by type and crate."""
    categories = defaultdict(list)
    error_codes = Counter()
    crates = defaultdict(list)

    for error in errors:
        package_id = error.get('package_id', '')
        if 'agent-workers' in package_id:
            crate = 'agent-workers'
        elif 'agent-orchestration' in package_id:
            crate = 'agent-orchestration'
        elif 'data-infrastructure' in package_id:
            crate = 'data-infrastructure'
        else:
            crate = package_id.split('#')[0] if '#' in package_id else 'unknown'

        crates[crate].append(error)

        # Categorize by error code
        message = error.get('message', {})
        code = message.get('code', {})
        error_code = code.get('code', 'unknown') if isinstance(code, dict) else str(code)

        # Categorize by error type
        rendered = message.get('rendered', '')
        if 'cannot find' in rendered or 'not found in' in rendered:
            categories['missing_imports'].append(error)
        elif 'mismatched types' in rendered:
            categories['type_mismatches'].append(error)
        elif 'cannot return reference to temporary value' in rendered:
            categories['lifetime_issues'].append(error)
        elif 'expected' in rendered and 'found' in rendered:
            categories['type_mismatches'].append(error)
        elif 'field does not exist' in rendered:
            categories['struct_field_missing'].append(error)
        elif 'no method named' in rendered:
            categories['missing_methods'].append(error)
        elif 'cannot borrow' in rendered:
            categories['borrowing_issues'].append(error)
        elif 'trait' in rendered and 'not implemented' in rendered:
            categories['trait_implementation'].append(error)
        elif 'unused' in rendered:
            categories['unused_items'].append(error)
        else:
            categories['other'].append(error)

        error_codes[error_code] += 1

    return {
        'by_category': dict(categories),
        'by_crate': dict(crates),
        'error_codes': dict(error_codes)
    }

def analyze_error_patterns(errors: List[Dict]) -> Dict[str, int]:
    """Analyze error patterns for prioritization."""
    patterns = Counter()

    for error in errors:
        rendered = error.get('message', {}).get('rendered', '')

        # Common patterns
        if 'cannot find' in rendered and 'in the crate root' in rendered:
            patterns['missing_module_declarations'] += 1
        elif 'expected struct' in rendered and 'found' in rendered:
            patterns['struct_construction_errors'] += 1
        elif 'trait bound' in rendered:
            patterns['trait_bounds'] += 1
        elif 'lifetime' in rendered:
            patterns['lifetime_parameters'] += 1
        elif 'associated type' in rendered:
            patterns['associated_types'] += 1
        elif 'const generics' in rendered:
            patterns['const_generics'] += 1
        elif 'async fn' in rendered:
            patterns['async_issues'] += 1
        elif 'macro' in rendered:
            patterns['macro_expansion'] += 1

    return dict(patterns)

def generate_worker_tasks(categorized: Dict, error_patterns: Dict) -> List[Dict]:
    """Generate prioritized tasks for three workers."""
    tasks = []

    # Worker 1: Critical structural issues (agent-workers has 81 errors)
    worker1_tasks = {
        'worker': 'Worker 1 (Structural Fixes)',
        'focus': 'agent-workers crate - fix fundamental compilation issues',
        'priority_errors': [
            'missing_imports',
            'struct_field_missing',
            'lifetime_issues',
            'missing_methods'
        ],
        'estimated_errors': 81,
        'strategy': 'Fix core structural issues that block other compilation'
    }

    # Worker 2: Type system and trait issues (agent-orchestration has 7 errors)
    worker2_tasks = {
        'worker': 'Worker 2 (Type System)',
        'focus': 'agent-orchestration crate - fix type mismatches and trait implementations',
        'priority_errors': [
            'type_mismatches',
            'trait_implementation',
            'borrowing_issues'
        ],
        'estimated_errors': 7,
        'strategy': 'Fix type system issues and trait implementations'
    }

    # Worker 3: Cleanup and remaining issues
    worker3_tasks = {
        'worker': 'Worker 3 (Cleanup)',
        'focus': 'Remaining crates - fix warnings and remaining errors',
        'priority_errors': [
            'unused_items',
            'other'
        ],
        'estimated_errors': len(categorized.get('by_crate', {}).get('other', [])),
        'strategy': 'Clean up warnings and handle remaining compilation issues'
    }

    return [worker1_tasks, worker2_tasks, worker3_tasks]

def print_analysis(categorized: Dict, error_patterns: Dict, tasks: List[Dict]):
    """Print comprehensive error analysis."""

    print("🚫 RUST COMPILATION ERROR ANALYSIS")
    print("=" * 50)

    # Overall statistics
    total_errors = sum(len(errors) for errors in categorized.get('by_category', {}).values())
    print(f"Total compilation errors: {total_errors}")

    # Errors by crate
    print("\n📦 ERRORS BY CRATE:")
    for crate, errors in categorized.get('by_crate', {}).items():
        print(f"  {crate}: {len(errors)} errors")

    # Error categories
    print("\n📋 ERROR CATEGORIES:")
    for category, errors in categorized.get('by_category', {}).items():
        print(f"  {category}: {len(errors)} errors")

    # Error codes
    print("\n🔢 TOP ERROR CODES:")
    error_codes = categorized.get('error_codes', {})
    for code, count in sorted(error_codes.items(), key=lambda x: x[1], reverse=True)[:10]:
        print(f"  {code}: {count} occurrences")

    # Error patterns
    print("\n🎯 ERROR PATTERNS:")
    for pattern, count in sorted(error_patterns.items(), key=lambda x: x[1], reverse=True):
        print(f"  {pattern}: {count} occurrences")

    # Worker assignments
    print("\n👷 WORKER TASK ASSIGNMENTS:")
    print("Based on error distribution and types:\n")

    for task in tasks:
        print(f"🔧 {task['worker']}")
        print(f"   Focus: {task['focus']}")
        print(f"   Estimated errors: {task['estimated_errors']}")
        print(f"   Strategy: {task['strategy']}")
        print(f"   Priority error types: {', '.join(task['priority_errors'])}")
        print()

    # Mermaid diagram for visualization
    print("📊 ERROR DISTRIBUTION (Mermaid):")
    print("```mermaid")
    print("pie title Error Distribution by Category")
    for category, errors in categorized.get('by_category', {}).items():
        print(f"    \"{category}\" : {len(errors)}")
    print("```")

    # Recommendations
    print("\n💡 RECOMMENDATIONS:")
    print("1. Start with Worker 1 (agent-workers) - it has the most critical structural issues")
    print("2. Worker 2 can work in parallel on type system issues in agent-orchestration")
    print("3. Worker 3 should handle cleanup and remaining issues")
    print("4. Focus on missing imports and struct field issues first - they're blocking")
    print("5. Coordinate on shared dependencies to avoid conflicts")

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python analyze-errors.py <cargo-check-output.json>")
        sys.exit(1)

    filepath = sys.argv[1]
    errors = load_errors_from_file(filepath)
    categorized = categorize_errors(errors)
    error_patterns = analyze_error_patterns(errors)
    tasks = generate_worker_tasks(categorized, error_patterns)

    print_analysis(categorized, error_patterns, tasks)
