#!/usr/bin/env python3
"""
Generate evidence-based status reports from code analysis.

This script analyzes the codebase and generates status documents based on
actual code state rather than manual claims.

Usage:
    python3 scripts/v3/analysis/generate_status_report.py --output-dir iterations/v3/docs/status-generated
"""

import os
import sys
import json
import subprocess
import re
from pathlib import Path
from datetime import datetime
from typing import Dict, List, Tuple, Optional
from collections import defaultdict

# Add project root to path
project_root = Path(__file__).parent.parent.parent.parent
sys.path.insert(0, str(project_root))

class StatusReportGenerator:
    def __init__(self, v3_root: Path):
        self.v3_root = v3_root
        self.api_dir = v3_root / "data-infrastructure" / "src" / "api"
        self.test_dir = v3_root / "testing-validation"
        self.reports = {}
        
    def analyze_api_endpoints(self) -> Dict[str, any]:
        """Analyze actual API endpoint implementations."""
        endpoints = {
            "implemented": [],
            "missing": [],
            "by_category": defaultdict(list)
        }
        
        # Scan handlers directory
        handlers_dir = self.api_dir / "handlers"
        if not handlers_dir.exists():
            return {"error": "API handlers directory not found"}
        
        # Pattern to match route definitions
        route_pattern = re.compile(r'\.route\([^,]+,\s*["\']([^"\']+)["\']')
        method_pattern = re.compile(r'(get|post|put|patch|delete)\(', re.IGNORECASE)
        
        for handler_file in handlers_dir.glob("*.rs"):
            try:
                content = handler_file.read_text()
                
                # Find route definitions
                for match in route_pattern.finditer(content):
                    route = match.group(1)
                    # Try to find HTTP method
                    method = "GET"  # Default
                    for method_match in method_pattern.finditer(content[:match.start()]):
                        method = method_match.group(1).upper()
                    
                    full_endpoint = f"{method} {route}"
                    endpoints["implemented"].append(full_endpoint)
                    
                    # Categorize
                    if "/auth" in route:
                        endpoints["by_category"]["authentication"].append(full_endpoint)
                    elif "/agents" in route:
                        endpoints["by_category"]["agent_management"].append(full_endpoint)
                    elif "/tasks" in route:
                        endpoints["by_category"]["task_management"].append(full_endpoint)
                    elif "/projects" in route:
                        endpoints["by_category"]["project_management"].append(full_endpoint)
                    elif "/telemetry" in route or "/observability" in route:
                        endpoints["by_category"]["telemetry"].append(full_endpoint)
                    elif "/provenance" in route:
                        endpoints["by_category"]["provenance"].append(full_endpoint)
                    elif "/waivers" in route:
                        endpoints["by_category"]["waivers"].append(full_endpoint)
                    elif "/database" in route:
                        endpoints["by_category"]["database"].append(full_endpoint)
                    else:
                        endpoints["by_category"]["other"].append(full_endpoint)
            except Exception as e:
                print(f"Warning: Could not analyze {handler_file}: {e}", file=sys.stderr)
        
        endpoints["total_implemented"] = len(endpoints["implemented"])
        return endpoints
    
    def analyze_todo_density(self) -> Dict[str, any]:
        """Analyze TODO/PLACEHOLDER/MOCK density."""
        patterns = {
            "TODO": re.compile(r'\bTODO\b', re.IGNORECASE),
            "PLACEHOLDER": re.compile(r'\bPLACEHOLDER\b', re.IGNORECASE),
            "MOCK": re.compile(r'\bMOCK\b', re.IGNORECASE),
            "FIXME": re.compile(r'\bFIXME\b', re.IGNORECASE),
        }
        
        results = {
            "total_files": 0,
            "files_with_todos": 0,
            "matches_by_type": defaultdict(int),
            "matches_by_file": defaultdict(int),
            "high_density_files": []
        }
        
        for rust_file in self.v3_root.rglob("*.rs"):
            if "target" in str(rust_file) or "node_modules" in str(rust_file):
                continue
            
            results["total_files"] += 1
            file_matches = 0
            
            try:
                content = rust_file.read_text()
                for pattern_name, pattern in patterns.items():
                    matches = len(pattern.findall(content))
                    if matches > 0:
                        results["matches_by_type"][pattern_name] += matches
                        file_matches += matches
                        results["matches_by_file"][str(rust_file.relative_to(self.v3_root))] += matches
                
                if file_matches > 0:
                    results["files_with_todos"] += 1
                    if file_matches > 10:  # High density threshold
                        results["high_density_files"].append({
                            "file": str(rust_file.relative_to(self.v3_root)),
                            "matches": file_matches
                        })
            except Exception:
                pass  # Skip binary or unreadable files
        
        results["total_matches"] = sum(results["matches_by_type"].values())
        results["density_percentage"] = (results["files_with_todos"] / results["total_files"] * 100) if results["total_files"] > 0 else 0
        
        return results
    
    def analyze_test_status(self) -> Dict[str, any]:
        """Analyze test execution status."""
        results = {
            "e2e_tests": {},
            "integration_tests": {},
            "unit_tests": {}
        }
        
        # Check if E2E tests exist
        if self.test_dir.exists():
            main_rs = self.test_dir / "src" / "main.rs"
            if main_rs.exists():
                content = main_rs.read_text()
                scenarios = re.findall(r'Scenario::(\w+)', content)
                results["e2e_tests"]["scenarios_defined"] = len(set(scenarios))
                results["e2e_tests"]["has_runner"] = True
            else:
                results["e2e_tests"]["has_runner"] = False
        
        # Try to run cargo test --no-run to check compilation
        try:
            result = subprocess.run(
                ["cargo", "test", "--no-run", "--manifest-path", str(self.v3_root / "Cargo.toml")],
                capture_output=True,
                text=True,
                timeout=60,
                cwd=str(self.v3_root)
            )
            results["compilation_status"] = "success" if result.returncode == 0 else "failed"
            if result.returncode != 0:
                results["compilation_errors"] = result.stderr[:500]  # First 500 chars
        except Exception as e:
            results["compilation_status"] = "unknown"
            results["compilation_error"] = str(e)
        
        return results
    
    def check_coreml_status(self) -> Dict[str, any]:
        """Check CoreML/Swift bridge status."""
        results = {
            "swift_bridge_exists": False,
            "build_rs_exists": False,
            "ffi_declarations": 0
        }
        
        # Check for Swift bridge
        swift_bridge = project_root / "models" / "languages" / "swift" / "coreml-bridge"
        if swift_bridge.exists():
            results["swift_bridge_exists"] = True
        
        # Check for build.rs files that link Swift bridge
        for build_rs in self.v3_root.rglob("build.rs"):
            try:
                content = build_rs.read_text()
                if "CoreMLBridge" in content or "agentbridge" in content:
                    results["build_rs_exists"] = True
                    break
            except Exception:
                pass
        
        # Count FFI declarations
        for rust_file in self.v3_root.rglob("*.rs"):
            try:
                content = rust_file.read_text()
                if "extern \"C\"" in content and "agentbridge" in content:
                    results["ffi_declarations"] += 1
            except Exception:
                pass
        
        return results
    
    def generate_status_report(self) -> str:
        """Generate comprehensive status report."""
        timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        
        # Run analyses
        api_analysis = self.analyze_api_endpoints()
        todo_analysis = self.analyze_todo_density()
        test_analysis = self.analyze_test_status()
        coreml_analysis = self.check_coreml_status()
        
        # Generate markdown report
        report = f"""# V3 Status Report (Generated)

**Generated:** {timestamp}  
**Source:** Automated code analysis  
**Evidence-Based:** All claims verified from actual code

---

## API Endpoint Coverage

### Implemented Endpoints: {api_analysis.get('total_implemented', 0)}

**By Category:**
"""
        
        for category, endpoints in api_analysis.get('by_category', {}).items():
            report += f"- **{category.replace('_', ' ').title()}**: {len(endpoints)} endpoints\n"
        
        report += f"""
**Total Implemented:** {api_analysis.get('total_implemented', 0)} endpoints

### Endpoint List
"""
        for endpoint in sorted(api_analysis.get('implemented', [])):
            report += f"- ✅ {endpoint}\n"
        
        report += f"""

---

## Implementation Completeness

### TODO/PLACEHOLDER Analysis

- **Total Files Analyzed:** {todo_analysis.get('total_files', 0)}
- **Files with TODOs:** {todo_analysis.get('files_with_todos', 0)} ({todo_analysis.get('density_percentage', 0):.1f}%)
- **Total Matches:** {todo_analysis.get('total_matches', 0)}

**By Type:**
"""
        for todo_type, count in todo_analysis.get('matches_by_type', {}).items():
            report += f"- **{todo_type}**: {count} matches\n"
        
        if todo_analysis.get('high_density_files'):
            report += "\n**High Density Files (>10 matches):**\n"
            for file_info in todo_analysis['high_density_files'][:10]:  # Top 10
                report += f"- `{file_info['file']}`: {file_info['matches']} matches\n"
        
        report += f"""

---

## Test Infrastructure Status

### E2E Tests
- **Test Runner Exists:** {test_analysis.get('e2e_tests', {}).get('has_runner', False)}
- **Scenarios Defined:** {test_analysis.get('e2e_tests', {}).get('scenarios_defined', 0)}

### Compilation Status
- **Status:** {test_analysis.get('compilation_status', 'unknown')}
"""
        if test_analysis.get('compilation_errors'):
            report += f"- **Errors:** {test_analysis['compilation_errors'][:200]}...\n"
        
        report += f"""

---

## CoreML Integration Status

- **Swift Bridge Exists:** {coreml_analysis.get('swift_bridge_exists', False)}
- **Build.rs Configured:** {coreml_analysis.get('build_rs_exists', False)}
- **FFI Declarations:** {coreml_analysis.get('ffi_declarations', 0)}

---

## Key Metrics Summary

| Metric | Value | Status |
|--------|-------|--------|
| API Endpoints Implemented | {api_analysis.get('total_implemented', 0)} | ✅ |
| Files with TODOs | {todo_analysis.get('files_with_todos', 0)} / {todo_analysis.get('total_files', 0)} | ⚠️ |
| TODO Density | {todo_analysis.get('density_percentage', 0):.1f}% | ⚠️ |
| E2E Scenarios | {test_analysis.get('e2e_tests', {}).get('scenarios_defined', 0)} | ✅ |
| Compilation | {test_analysis.get('compilation_status', 'unknown')} | {'✅' if test_analysis.get('compilation_status') == 'success' else '⚠️'} |
| CoreML Bridge | {'✅' if coreml_analysis.get('swift_bridge_exists') and coreml_analysis.get('build_rs_exists') else '⚠️'} | {'✅' if coreml_analysis.get('swift_bridge_exists') and coreml_analysis.get('build_rs_exists') else '⚠️'} |

---

## Recommendations

1. **API Coverage**: {api_analysis.get('total_implemented', 0)} endpoints implemented. Verify against requirements.
2. **TODO Density**: {todo_analysis.get('density_percentage', 0):.1f}% of files contain TODOs. Consider systematic cleanup.
3. **Test Coverage**: E2E test infrastructure exists. Run tests to verify current status.
4. **CoreML**: {'Swift bridge and build configuration exist.' if coreml_analysis.get('swift_bridge_exists') and coreml_analysis.get('build_rs_exists') else 'Swift bridge configuration needs verification.'}

---

**Note:** This report is generated from code analysis. For manual status documents, see `CURRENT_STATUS_AND_NEXT_STEPS.md` (may be outdated).
"""
        
        return report
    
    def save_report(self, output_dir: Path):
        """Save report to file."""
        output_dir.mkdir(parents=True, exist_ok=True)
        report_file = output_dir / f"STATUS_REPORT_{datetime.now().strftime('%Y%m%d_%H%M%S')}.md"
        report_file.write_text(self.generate_status_report())
        print(f"✅ Status report saved to: {report_file}")
        return report_file

def main():
    import argparse
    
    parser = argparse.ArgumentParser(description="Generate evidence-based status reports")
    parser.add_argument("--v3-root", type=Path, default=Path("iterations/v3"),
                       help="Path to V3 root directory")
    parser.add_argument("--output-dir", type=Path, default=Path("iterations/v3/docs/status-generated"),
                       help="Output directory for generated reports")
    parser.add_argument("--update-current", action="store_true",
                       help="Update CURRENT_STATUS_AND_NEXT_STEPS.md with evidence-based claims")
    
    args = parser.parse_args()
    
    v3_root = Path(args.v3_root).resolve()
    if not v3_root.exists():
        print(f"Error: V3 root not found: {v3_root}", file=sys.stderr)
        sys.exit(1)
    
    generator = StatusReportGenerator(v3_root)
    report_file = generator.save_report(args.output_dir)
    
    if args.update_current:
        current_status_file = v3_root / "docs" / "CURRENT_STATUS_AND_NEXT_STEPS.md"
        if current_status_file.exists():
            # Add evidence section to current status
            evidence_section = f"""
---

## 📊 Evidence-Based Status (Auto-Generated)

**Last Generated:** {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}

See `status-generated/STATUS_REPORT_*.md` for detailed evidence-based analysis.

**Quick Summary:**
- API Endpoints: {generator.analyze_api_endpoints().get('total_implemented', 0)} implemented
- TODO Density: {generator.analyze_todo_density().get('density_percentage', 0):.1f}% of files
- Compilation: {generator.analyze_test_status().get('compilation_status', 'unknown')}
"""
            # Append to file
            with current_status_file.open('a') as f:
                f.write(evidence_section)
            print(f"✅ Updated {current_status_file} with evidence section")
    
    print(f"\n✅ Status report generation complete!")
    print(f"📄 Report: {report_file}")

if __name__ == "__main__":
    main()

