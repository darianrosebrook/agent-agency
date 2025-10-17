#!/usr/bin/env python3
"""
Enhanced Hidden TODO Pattern Analyzer

This script includes newly discovered patterns that indicate hidden work
and incomplete implementations.
"""

import os
import re
import json
from pathlib import Path
from collections import defaultdict, Counter
from typing import Dict, List, Set, Tuple


class EnhancedHiddenTodoAnalyzer:
    def __init__(self, root_dir: str):
        self.root_dir = Path(root_dir)
        # Comprehensive list of file patterns to ignore
        self.ignored_file_patterns = [
            # Test files
            r'\btest\b',
            r'\btests\b',
            r'_test\.rs$',
            r'_tests\.rs$',

            # Build artifacts and generated files
            r'\btarget\b',
            r'\bbuild\b',
            r'\bout\b',
            r'generated\.rs$',
            r'bindgen\.rs$',
            r'private\.rs$',
            r'mime_types_generated\.rs$',
            r'named_entities\.rs$',
            r'ascii_case_insensitive_html_attributes\.rs$',

            # Package management
            r'\bnode_modules\b',
            r'package-lock\.json$',
            r'package\.json$',
            r'\bvenv\b',
            r'\bpip\b',

            # Version control and IDE
            r'\.git\b',
            r'\.github\b',
            r'\.vscode\b',
            r'\.idea\b',
            r'\.DS_Store$',
            r'\.DS_Store\?$',
            r'\._',
            r'\.Spotlight-V100$',

            # Documentation and examples
            r'\bdocs\b',
            r'\bexamples\b',
            r'\bdoc\b',

            # Temporary and cache files
            r'\bcache\b',
            r'\btmp\b',
            r'\btemp\b',
            r'\.tmp$',
            r'\.temp$',

            # OS-specific files
            r'Thumbs\.db$',
            r'desktop\.ini$',
            r'\.fseventsd$',
            r'\.Trashes$',

            # Rust-specific build artifacts
            r'\.rlib$',
            r'\.rmeta$',
            r'\.d$',
            r'\.pdb$',
        ]
        # Enhanced patterns including newly discovered ones
        self.enhanced_patterns = {
            # Original patterns
            'temporal': [
                r'\bfor now\b',
                r'\bsimplified\b',
                r'\bbasic\b',
                r'\bsimple\b',
                r'\bminimal\b',
                r'\btemporary\b',
                r'\bpreliminary\b',
                r'\binitial\b',
                r'\bprototype\b',
            ],

            'future': [
                r'// Would be',
                r'// Would contain',
                r'// This would',
                r'// This should',
                r'// This will',
                r'// This might',
                r'// In production',
                r'// In a real implementation',
                r'// Eventually',
                r'// Later',
            ],

            'placeholder': [
                r'\bplaceholder\b',
                r'\bmock\b',
                r'\bstub\b',
                r'\bdummy\b',
                r'\bfake\b',
                r'\bexample\b',
                r'\bdemo\b',
                r'\bsample\b',
                r'\btemplate\b',
            ],

            'simulation': [
                r'\bsimulate\b',
                r'\bsimulating\b',
                r'\bsimulated\b',
                r'\bsimulation\b',
            ],

            # NEWLY DISCOVERED PATTERNS

            # Conditional/Contextual patterns
            'conditional': [
                r'\bif\b.*\bimplemented\b',
                r'\bwhen\b.*\bready\b',
                r'\bonce\b.*\bavailable\b',
                r'\bafter\b.*\bcomplete\b',
                r'\bbefore\b.*\bfinal\b',
                r'\bshould\b.*\bcontain\b',
                r'\bcould\b.*\binclude\b',
            ],

            # Version/Integration patterns
            'version_integration': [
                r'\bv[0-9]+\b.*\bport\b',
                r'\bv[0-9]+\b.*\bintegration\b',
                r'\bv[0-9]+\b.*\bupgrade\b',
                r'\bv[0-9]+\b.*\bmigration\b',
                r'\bv[0-9]+\b.*\bcompatibility\b',
            ],

            # Performance/Quality indicators
            'performance_quality': [
                r'\boptimize\b',
                r'\befficient\b',
                r'\bperformance\b',
                r'\bspeed\b.*\bimprovement\b',
                r'\brough\b.*\bheuristic\b',
                r'\bcrude\b.*\bimplementation\b',
                r'\bnaive\b.*\bapproach\b',
            ],

            # Implementation status patterns
            'implementation_status': [
                r'\bnot yet\b.*\bimplemented\b',
                r'\bmissing\b.*\bimplementation\b',
                r'\bincomplete\b.*\bimplementation\b',
                r'\bpartial\b.*\bimplementation\b',
                r'\bunimplemented\b',
                r'\bnot done\b',
                r'\bpending\b.*\bimplementation\b',
            ],

            # Workaround/Hack patterns
            'workarounds': [
                r'\bworkaround\b',
                r'\bhack\b',
                r'\btemporary fix\b',
                r'\bpatch\b',
                r'\bquick fix\b',
                r'\bbypass\b',
            ],

            # Hardcoded/Configuration patterns
            'hardcoded_config': [
                r'\bhardcoded\b',
                r'\bhard-coded\b',
                r'\bmagic number\b',
                r'\bmagic string\b',
                r'\bconstant\b.*\bvalue\b',
                r'\bdefault\b.*\bvalue\b',
            ],

            # Fallback/Alternative patterns
            'fallback_alternatives': [
                r'\bfallback\b',
                r'\bbackup\b.*\bimplementation\b',
                r'\balternative\b.*\bapproach\b',
                r'\belse\b.*\buse\b',
                r'\botherwise\b.*\bdefault\b',
            ],

            # Stub/Interface patterns
            'stub_interfaces': [
                r'\bstub\b.*\binterface\b',
                r'\bstub\b.*\bimplementation\b',
                r'\btrait\b.*\bimplement\b',
                r'\binterface\b.*\bimplement\b',
            ],

            # Error handling patterns
            'error_handling': [
                r'\berror\b.*\bhandling\b',
                r'\bexception\b.*\bhandling\b',
                r'\bfail\b.*\bgracefully\b',
                r'\bretry\b.*\blogic\b',
                r'\brecover\b.*\bfrom\b',
            ],

            # Database/Storage patterns
            'database_storage': [
                r'\bdatabase\b.*\bimplementation\b',
                r'\bdb\b.*\bclient\b',
                r'\bpostgres\b.*\bintegration\b',
                r'\bstorage\b.*\bbackend\b',
                r'\bpersistence\b.*\blayer\b',
            ],

            # API/Network patterns
            'api_network': [
                r'\bapi\b.*\bendpoint\b',
                r'\bhttp\b.*\bclient\b',
                r'\brest\b.*\binterface\b',
                r'\bnetwork\b.*\bcommunication\b',
                r'\brequest\b.*\bhandling\b',
            ],

            # Security patterns
            'security': [
                r'\bsecurity\b.*\bvalidation\b',
                r'\bauth\b.*\bimplementation\b',
                r'\bpermission\b.*\bcheck\b',
                r'\baccess\b.*\bcontrol\b',
                r'\bencrypt\b.*\bdata\b',
            ],

            # Testing patterns (for non-test files)
            'testing_related': [
                r'\btest\b.*\bimplementation\b',
                r'\btesting\b.*\bframework\b',
                r'\bmock\b.*\bservice\b',
                r'\bfixture\b.*\bdata\b',
            ],

            # Documentation patterns
            'documentation': [
                r'\bdocument\b.*\bimplementation\b',
                r'\bcomment\b.*\bcode\b',
                r'\bexplain\b.*\blogic\b',
                r'\bdescribe\b.*\bprocess\b',
            ],
        }

        self.results = defaultdict(list)
        self.file_stats = defaultdict(int)
        self.pattern_stats = defaultdict(int)

    def should_ignore_file(self, file_path: Path) -> bool:
        """Check if a file should be ignored based on patterns."""
        path_str = str(file_path)

        # Check against ignored patterns
        for pattern in self.ignored_file_patterns:
            if re.search(pattern, path_str, re.IGNORECASE):
                return True

        # Additional specific checks
        return (
            # Test files
            '/tests/' in path_str or
            '/test/' in path_str or
            path_str.endswith('_test.rs') or
            path_str.endswith('_tests.rs') or
            'test_' in path_str or
            'tests_' in path_str or

            # Build artifacts and generated files
            '/target/' in path_str or
            '/build/' in path_str or
            '/out/' in path_str or
            'generated.rs' in path_str or
            'bindgen.rs' in path_str or
            'private.rs' in path_str or
            'mime_types_generated.rs' in path_str or
            'named_entities.rs' in path_str or
            'ascii_case_insensitive_html_attributes.rs' in path_str or

            # Examples and documentation
            '/examples/' in path_str or
            '/docs/' in path_str or

            # IDE and system files
            '/.vscode/' in path_str or
            '/.idea/' in path_str or
            '/.git/' in path_str or
            '.DS_Store' in path_str or
            '._' in path_str or
            '.Spotlight-V100' in path_str
        )

    def extract_comments_from_file(self, file_path: Path) -> List[Tuple[int, str]]:
        """Extract all comments from a Rust file."""
        comments = []
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                lines = f.readlines()

            for line_num, line in enumerate(lines, 1):
                line = line.strip()

                # Skip empty lines
                if not line:
                    continue

                # Extract single-line comments
                if line.startswith('//'):
                    comment = line[2:].strip()
                    if comment:  # Skip empty comments
                        comments.append((line_num, comment))

        except Exception as e:
            print(f"Error reading {file_path}: {e}")

        return comments

    def analyze_comment(self, comment: str, line_num: int, file_path: Path) -> Dict[str, List[str]]:
        """Analyze a single comment for hidden TODO patterns."""
        matches = defaultdict(list)

        for category, patterns in self.enhanced_patterns.items():
            for pattern in patterns:
                if re.search(pattern, comment, re.IGNORECASE):
                    matches[category].append(pattern)
                    self.pattern_stats[pattern] += 1

        return matches

    def analyze_file(self, file_path: Path) -> Dict:
        """Analyze a single Rust file for hidden TODO patterns."""
        if not file_path.suffix == '.rs':
            return {}

        # Skip ignored files for implementation analysis
        if self.should_ignore_file(file_path):
            return {}

        comments = self.extract_comments_from_file(file_path)
        file_analysis = {
            'file_path': str(file_path.relative_to(self.root_dir)),
            'total_comments': len(comments),
            'hidden_todos': defaultdict(list),
            'all_comments': []
        }

        for line_num, comment in comments:
            matches = self.analyze_comment(comment, line_num, file_path)

            if matches:
                file_analysis['hidden_todos'][line_num] = {
                    'comment': comment,
                    'matches': matches
                }

            # Store all comments for analysis
            file_analysis['all_comments'].append({
                'line': line_num,
                'comment': comment
            })

        return file_analysis

    def analyze_directory(self) -> Dict:
        """Analyze all non-test Rust files in the directory."""
        print(
            f"Analyzing non-test Rust files with enhanced patterns in: {self.root_dir}")

        rust_files = list(self.root_dir.rglob('*.rs'))
        non_ignored_files = [
            f for f in rust_files if not self.should_ignore_file(f)]

        print(f"Found {len(rust_files)} total Rust files")
        print(f"Found {len(non_ignored_files)} non-ignored Rust files")

        all_results = {
            'summary': {
                'total_files': len(rust_files),
                'non_ignored_files': len(non_ignored_files),
                'ignored_files': len(rust_files) - len(non_ignored_files),
                'files_with_hidden_todos': 0,
                'total_hidden_todos': 0,
                'pattern_counts': dict(self.pattern_stats),
                'new_patterns_found': []
            },
            'files': {},
            'patterns': defaultdict(list)
        }

        for file_path in non_ignored_files:
            print(f"Analyzing: {file_path.relative_to(self.root_dir)}")
            file_analysis = self.analyze_file(file_path)

            if file_analysis and file_analysis['hidden_todos']:
                all_results['files'][file_analysis['file_path']
                                     ] = file_analysis
                all_results['summary']['files_with_hidden_todos'] += 1
                all_results['summary']['total_hidden_todos'] += len(
                    file_analysis['hidden_todos'])

                # Group by patterns
                for line_num, data in file_analysis['hidden_todos'].items():
                    for category, patterns in data['matches'].items():
                        all_results['patterns'][category].append({
                            'file': file_analysis['file_path'],
                            'line': line_num,
                            'comment': data['comment'],
                            'patterns': patterns
                        })

        return all_results

    def generate_enhanced_report(self, results: Dict) -> str:
        """Generate an enhanced report with new pattern insights."""
        report = []
        report.append("# Enhanced Hidden TODO Analysis Report")
        report.append("=" * 60)
        report.append("")

        # Summary
        summary = results['summary']
        report.append("## Summary")
        report.append(f"- Total Rust files: {summary['total_files']}")
        report.append(
            f"- Non-ignored Rust files: {summary['non_ignored_files']}")
        report.append(f"- Ignored files: {summary['ignored_files']}")
        report.append(
            f"- Files with hidden TODOs: {summary['files_with_hidden_todos']}")
        report.append(
            f"- Total hidden TODOs found: {summary['total_hidden_todos']}")
        report.append("")

        # New patterns discovered
        report.append("## Newly Discovered Pattern Categories")
        new_categories = [
            'conditional', 'version_integration', 'performance_quality',
            'implementation_status', 'workarounds', 'hardcoded_config',
            'fallback_alternatives', 'stub_interfaces', 'error_handling',
            'database_storage', 'api_network', 'security', 'testing_related', 'documentation'
        ]

        for category in new_categories:
            if category in results['patterns'] and results['patterns'][category]:
                count = len(results['patterns'][category])
                report.append(
                    f"- **{category.replace('_', ' ').title()}**: {count} items")
        report.append("")

        # Pattern statistics
        report.append("## Enhanced Pattern Statistics")
        for pattern, count in sorted(summary['pattern_counts'].items(), key=lambda x: x[1], reverse=True):
            if count > 0:
                report.append(f"- `{pattern}`: {count} occurrences")
        report.append("")

        # Files with most hidden TODOs
        report.append("## Files with Most Hidden TODOs")
        file_todo_counts = []
        for file_path, data in results['files'].items():
            file_todo_counts.append((file_path, len(data['hidden_todos'])))

        file_todo_counts.sort(key=lambda x: x[1], reverse=True)
        for file_path, count in file_todo_counts[:15]:
            report.append(f"- `{file_path}`: {count} hidden TODOs")
        report.append("")

        # New pattern examples
        report.append("## Examples of Newly Discovered Patterns")
        for category in new_categories:
            if category in results['patterns'] and results['patterns'][category]:
                items = results['patterns'][category]
                report.append(
                    f"### {category.replace('_', ' ').title()} ({len(items)} items)")
                for item in items[:3]:  # Show first 3 examples
                    report.append(
                        f"- `{item['file']}:{item['line']}`: {item['comment'][:100]}...")
                if len(items) > 3:
                    report.append(f"- ... and {len(items) - 3} more")
                report.append("")

        return "\n".join(report)


def main():
    analyzer = EnhancedHiddenTodoAnalyzer('.')
    results = analyzer.analyze_directory()

    # Print summary
    summary = results['summary']
    print(f"\n{'='*60}")
    print("ENHANCED HIDDEN TODO ANALYSIS COMPLETE")
    print(f"{'='*60}")
    print(f"Total Rust files: {summary['total_files']}")
    print(f"Non-ignored Rust files: {summary['non_ignored_files']}")
    print(f"Ignored files: {summary['ignored_files']}")
    print(f"Files with hidden TODOs: {summary['files_with_hidden_todos']}")
    print(f"Total hidden TODOs: {summary['total_hidden_todos']}")

    # Show new pattern categories
    new_categories = [
        'conditional', 'version_integration', 'performance_quality',
        'implementation_status', 'workarounds', 'hardcoded_config',
        'fallback_alternatives', 'stub_interfaces', 'error_handling',
        'database_storage', 'api_network', 'security', 'testing_related', 'documentation'
    ]

    print(f"\nNew pattern categories found:")
    for category in new_categories:
        if category in results['patterns'] and results['patterns'][category]:
            count = len(results['patterns'][category])
            print(f"  {category.replace('_', ' ').title()}: {count} items")

    print(f"\nTop patterns found:")
    for pattern, count in sorted(summary['pattern_counts'].items(), key=lambda x: x[1], reverse=True)[:15]:
        if count > 0:
            print(f"  {pattern}: {count}")

    # Save reports
    with open('enhanced_hidden_todos_analysis.json', 'w') as f:
        json.dump(results, f, indent=2)
    print(f"\nDetailed results saved to: enhanced_hidden_todos_analysis.json")

    report = analyzer.generate_enhanced_report(results)
    with open('enhanced_hidden_todos_report.md', 'w') as f:
        f.write(report)
    print(f"Enhanced report saved to: enhanced_hidden_todos_report.md")

    print("\n" + report)


if __name__ == '__main__':
    main()
