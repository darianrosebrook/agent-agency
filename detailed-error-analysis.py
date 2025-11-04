#!/usr/bin/env python3
"""
Detailed error analysis for parallel fixing.
"""

import json
import sys
from collections import defaultdict, Counter
from typing import Dict, List, Tuple

def load_and_parse_errors(filepath: str) -> Tuple[List[Dict], List[Dict]]:
    """Load and separate errors from warnings."""
    errors = []
    warnings = []

    try:
        with open(filepath, 'r') as f:
            for line in f:
                line = line.strip()
                if line:
                    try:
                        data = json.loads(line)
                        if data.get('reason') == 'compiler-message':
                            message = data.get('message', {})
                            level = message.get('level')
                            if level == 'error':
                                errors.append(data)
                            elif level == 'warning':
                                warnings.append(data)
                    except json.JSONDecodeError:
                        continue
    except FileNotFoundError:
        print(f"Error: File {filepath} not found")
        sys.exit(1)

    return errors, warnings

def analyze_crate_errors(errors: List[Dict]) -> Dict[str, Dict]:
    """Analyze errors by crate with detailed breakdown."""
    crate_analysis = defaultdict(lambda: {
        'total_errors': 0,
        'files': defaultdict(int),
        'error_types': Counter(),
        'critical_errors': [],
        'top_files': []
    })

    for error in errors:
        package_id = error.get('package_id', '')
        if 'agent-workers' in package_id:
            crate = 'agent-workers'
        elif 'agent-orchestration' in package_id:
            crate = 'agent-orchestration'
        elif 'agent-research' in package_id:
            crate = 'agent-research'
        elif 'data-infrastructure' in package_id:
            crate = 'data-infrastructure'
        elif 'system-' in package_id:
            crate = package_id.split('#')[0].split('/')[-1] if '/' in package_id else 'system-crates'
        else:
            crate = package_id.split('#')[0] if '#' in package_id else 'unknown'

        crate_analysis[crate]['total_errors'] += 1

        # File analysis
        spans = error.get('message', {}).get('spans', [])
        if spans:
            file_name = spans[0].get('file_name', 'unknown')
            crate_analysis[crate]['files'][file_name] += 1

        # Error type analysis
        rendered = error.get('message', {}).get('rendered', '')
        if 'cannot find' in rendered and ('in the crate root' in rendered or 'in `prelude`' in rendered):
            crate_analysis[crate]['error_types']['missing_module_declarations'] += 1
            crate_analysis[crate]['critical_errors'].append(error)
        elif 'cannot find' in rendered:
            crate_analysis[crate]['error_types']['missing_imports'] += 1
        elif 'mismatched types' in rendered or ('expected' in rendered and 'found' in rendered):
            crate_analysis[crate]['error_types']['type_mismatches'] += 1
        elif 'cannot return reference to temporary value' in rendered:
            crate_analysis[crate]['error_types']['lifetime_issues'] += 1
            crate_analysis[crate]['critical_errors'].append(error)
        elif 'field does not exist' in rendered:
            crate_analysis[crate]['error_types']['struct_field_missing'] += 1
        elif 'no method named' in rendered:
            crate_analysis[crate]['error_types']['missing_methods'] += 1
        elif 'trait' in rendered and ('not implemented' in rendered or 'not satisfied' in rendered):
            crate_analysis[crate]['error_types']['trait_implementation'] += 1
        else:
            crate_analysis[crate]['error_types']['other'] += 1

    # Calculate top files for each crate
    for crate, data in crate_analysis.items():
        top_files = sorted(data['files'].items(), key=lambda x: x[1], reverse=True)[:5]
        data['top_files'] = top_files

    return dict(crate_analysis)

def generate_parallel_work_plan(crate_analysis: Dict[str, Dict]) -> List[Dict]:
    """Generate a detailed work plan for three workers."""

    # Sort crates by error count
    sorted_crates = sorted(crate_analysis.items(), key=lambda x: x[1]['total_errors'], reverse=True)

    work_plan = []

    # Worker 1: Focus on agent-workers (most errors, structural issues)
    worker1_crate = 'agent-workers'
    if worker1_crate in crate_analysis:
        data = crate_analysis[worker1_crate]
        work_plan.append({
            'worker_id': 1,
            'worker_name': 'Worker 1 - Structural Fixes',
            'primary_crate': worker1_crate,
            'total_errors': data['total_errors'],
            'priority_files': [f"{file} ({count} errors)" for file, count in data['top_files'][:3]],
            'critical_issues': len(data['critical_errors']),
            'strategy': [
                'Fix missing module declarations first (blocking imports)',
                'Resolve struct field issues that break compilation',
                'Address lifetime issues preventing compilation',
                'Fix import chains and dependencies'
            ],
            'estimated_time': '2-3 hours',
            'blocking': True
        })

    # Worker 2: Focus on agent-research (second most errors)
    worker2_crate = 'agent-research'
    if worker2_crate in crate_analysis:
        data = crate_analysis[worker2_crate]
        work_plan.append({
            'worker_id': 2,
            'worker_name': 'Worker 2 - Research Module',
            'primary_crate': worker2_crate,
            'total_errors': data['total_errors'],
            'priority_files': [f"{file} ({count} errors)" for file, count in data['top_files'][:3]],
            'critical_issues': len(data['critical_errors']),
            'strategy': [
                'Fix missing imports and module declarations',
                'Resolve type mismatches in research algorithms',
                'Address macro expansion issues',
                'Fix async trait implementations'
            ],
            'estimated_time': '2-4 hours',
            'blocking': False
        })

    # Worker 3: Focus on agent-orchestration and remaining issues
    worker3_crates = ['agent-orchestration']
    total_errors = 0
    all_files = []
    for crate in worker3_crates:
        if crate in crate_analysis:
            data = crate_analysis[crate]
            total_errors += data['total_errors']
            all_files.extend([f"{file} ({count} errors)" for file, count in data['top_files'][:2]])

    work_plan.append({
        'worker_id': 3,
        'worker_name': 'Worker 3 - Orchestration & Cleanup',
        'primary_crate': 'agent-orchestration + remaining',
        'total_errors': total_errors,
        'priority_files': all_files[:3],
        'critical_issues': 0,  # orchestration has fewer critical issues
        'strategy': [
            'Fix type mismatches in orchestration logic',
            'Resolve trait implementation issues',
            'Clean up unused imports and variables',
            'Address remaining compilation warnings'
        ],
        'estimated_time': '1-2 hours',
        'blocking': False
    })

    return work_plan

def print_detailed_analysis(crate_analysis: Dict[str, Dict], work_plan: List[Dict]):
    """Print detailed error analysis and work plan."""

    print("🔍 DETAILED RUST COMPILATION ERROR ANALYSIS")
    print("=" * 60)

    total_errors = sum(data['total_errors'] for data in crate_analysis.values())
    print(f"Total compilation errors across workspace: {total_errors}")
    print()

    # Detailed crate breakdown
    print("📦 CRATE-BY-CRATE ERROR BREAKDOWN:")
    print("-" * 40)

    for crate, data in sorted(crate_analysis.items(), key=lambda x: x[1]['total_errors'], reverse=True):
        print(f"\n🔧 {crate.upper()}")
        print(f"   Total errors: {data['total_errors']}")
        print(f"   Critical issues: {len(data['critical_errors'])}")

        if data['top_files']:
            print("   Most affected files:")
            for file, count in data['top_files'][:3]:
                print(f"     • {file}: {count} errors")

        if data['error_types']:
            print("   Error types:")
            for error_type, count in sorted(data['error_types'].items(), key=lambda x: x[1], reverse=True):
                print(f"     • {error_type}: {count}")

    print("\n" + "=" * 60)
    print("👷 PARALLEL WORK PLAN FOR THREE WORKERS")
    print("=" * 60)

    for plan in work_plan:
        print(f"\n🚀 {plan['worker_name']}")
        print("-" * 40)
        print(f"Primary crate: {plan['primary_crate']}")
        print(f"Total errors to fix: {plan['total_errors']}")
        print(f"Critical issues: {plan['critical_issues']}")
        print(f"Estimated time: {plan['estimated_time']}")
        print(f"Blocking: {'Yes' if plan['blocking'] else 'No'}")

        print("\nPriority files:")
        for file in plan['priority_files']:
            print(f"  • {file}")

        print("\nStrategy:")
        for step in plan['strategy']:
            print(f"  • {step}")

    # Coordination guidelines
    print("\n" + "=" * 60)
    print("🤝 COORDINATION GUIDELINES")
    print("=" * 60)
    print("""
1. 🔄 SYNCHRONIZATION:
   • Worker 1 goes first (blocking issues)
   • Workers 2 & 3 can work in parallel after Worker 1 clears critical path
   • Coordinate on shared dependencies (agent-agency-contracts)

2. 🧪 TESTING:
   • Run 'cargo check' after each major fix
   • Share progress updates every 30-45 minutes
   • Rebase frequently to avoid conflicts

3. 🎯 SUCCESS CRITERIA:
   • Worker 1: agent-workers compiles successfully
   • Worker 2: agent-research compiles successfully
   • Worker 3: agent-orchestration compiles + warnings cleaned

4. 🚨 ESCALATION:
   • If blocked >30 min, call for help
   • Complex issues: pair program
   • Architecture questions: escalate immediately
""")

    # Quick reference
    print("\n" + "=" * 60)
    print("📋 QUICK REFERENCE")
    print("=" * 60)

    print("Most common error types:")
    all_error_types = Counter()
    for data in crate_analysis.values():
        all_error_types.update(data['error_types'])

    for error_type, count in all_error_types.most_common(5):
        print(f"  • {error_type}: {count} total")

    print(f"\nTotal crates with errors: {len(crate_analysis)}")
    print(f"Total files affected: {sum(len(data['files']) for data in crate_analysis.values())}")

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python detailed-error-analysis.py <cargo-check-output.json>")
        sys.exit(1)

    filepath = sys.argv[1]
    errors, warnings = load_and_parse_errors(filepath)
    crate_analysis = analyze_crate_errors(errors)
    work_plan = generate_parallel_work_plan(crate_analysis)

    print_detailed_analysis(crate_analysis, work_plan)
