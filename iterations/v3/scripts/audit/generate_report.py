#!/usr/bin/env python3
"""
Generate comprehensive audit report from all verification data.
"""

import json
from pathlib import Path
from typing import Dict, List, Any
from datetime import datetime, timezone

V3_ROOT = Path(__file__).parent.parent.parent
REPORTS_DIR = V3_ROOT / "docs-status" / "audit-reports"
OUTPUT_FILE = REPORTS_DIR / "v3-documentation-reality-audit.md"

def load_json_file(file_path: Path) -> Dict[str, Any]:
    """Load JSON file or return empty dict."""
    if file_path.exists():
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                return json.load(f)
        except Exception as e:
            print(f"Error loading {file_path}: {e}")
    return {}

def generate_report():
    """Generate comprehensive audit report."""
    
    # Load all verification data
    documented_features = load_json_file(REPORTS_DIR / "documented-features.json")
    api_verification = load_json_file(REPORTS_DIR / "api-endpoint-verification.json")
    pattern_detection = load_json_file(REPORTS_DIR / "pattern-detection-results.json")
    stubs_inventory = V3_ROOT / "STUBS_INVENTORY.md"
    
    # Read stubs inventory if exists
    stubs_content = ""
    if stubs_inventory.exists():
        stubs_content = stubs_inventory.read_text(encoding='utf-8')
    
    # Start building report
    report_lines = []
    report_lines.append("# V3 Documentation Reality Audit Report")
    report_lines.append("")
    report_lines.append(f"**Generated:** {datetime.now(timezone.utc).strftime('%Y-%m-%d %H:%M:%S UTC')}")
    report_lines.append(f"**Author:** @darianrosebrook")
    report_lines.append("")
    report_lines.append("---")
    report_lines.append("")
    
    # Executive Summary
    report_lines.append("## Executive Summary")
    report_lines.append("")
    
    total_features = documented_features.get("summary", {}).get("total_features", 0)
    total_endpoints = documented_features.get("summary", {}).get("total_endpoints", 0)
    
    api_summary = api_verification.get("summary", {})
    implemented_endpoints = api_summary.get("implemented", 0)
    stub_endpoints = api_summary.get("stub", 0)
    missing_endpoints = api_summary.get("missing", 0)
    
    pattern_summary = pattern_detection.get("patterns", {})
    stub_count = pattern_summary.get("stubs", {}).get("count", 0)
    placeholder_count = pattern_summary.get("placeholders", {}).get("count", 0)
    mock_count = pattern_summary.get("mocks", {}).get("count", 0)
    incomplete_count = pattern_summary.get("incomplete", {}).get("count", 0)
    
    report_lines.append(f"- **Total Features Documented:** {total_features}")
    report_lines.append(f"- **Total API Endpoints Documented:** {total_endpoints}")
    report_lines.append(f"- **API Endpoints Implemented:** {implemented_endpoints}")
    report_lines.append(f"- **API Endpoints Missing:** {missing_endpoints}")
    report_lines.append(f"- **API Endpoints with Stubs:** {stub_endpoints}")
    report_lines.append(f"- **Stub Patterns Found:** {stub_count}")
    report_lines.append(f"- **Placeholder Patterns Found:** {placeholder_count}")
    report_lines.append(f"- **Mock Data Patterns Found:** {mock_count}")
    report_lines.append(f"- **Incomplete Implementation Patterns:** {incomplete_count}")
    report_lines.append("")
    
    # Calculate compliance score
    if total_endpoints > 0:
        compliance_rate = api_summary.get("compliance_rate", 0)
        report_lines.append(f"### Overall Compliance Score")
        report_lines.append("")
        report_lines.append(f"- **API Endpoint Compliance:** {compliance_rate:.1f}%")
        report_lines.append("")
    
    # Critical Issues
    report_lines.append("## Critical Issues")
    report_lines.append("")
    
    critical_issues = []
    
    # Check for critical stubs
    if stub_count > 0:
        critical_issues.append({
            "severity": "HIGH",
            "category": "Stub Implementations",
            "description": f"Found {stub_count} stub implementation patterns in production code",
            "count": stub_count
        })
    
    if placeholder_count > 0:
        critical_issues.append({
            "severity": "HIGH",
            "category": "Placeholder Implementations",
            "description": f"Found {placeholder_count} placeholder implementation patterns",
            "count": placeholder_count
        })
    
    if mock_count > 0:
        critical_issues.append({
            "severity": "MEDIUM",
            "category": "Mock Data",
            "description": f"Found {mock_count} mock data patterns in production code",
            "count": mock_count
        })
    
    if missing_endpoints > 0:
        critical_issues.append({
            "severity": "HIGH",
            "category": "Missing API Endpoints",
            "description": f"{missing_endpoints} documented API endpoints have no implementation",
            "count": missing_endpoints
        })
    
    if stub_endpoints > 0:
        critical_issues.append({
            "severity": "HIGH",
            "category": "Stub API Endpoints",
            "description": f"{stub_endpoints} API endpoints use stub implementations",
            "count": stub_endpoints
        })
    
    if critical_issues:
        for issue in critical_issues:
            report_lines.append(f"### {issue['severity']}: {issue['category']}")
            report_lines.append("")
            report_lines.append(f"{issue['description']}")
            report_lines.append("")
    else:
        report_lines.append("No critical issues found.")
        report_lines.append("")
    
    # API Endpoint Verification
    report_lines.append("## API Endpoint Verification")
    report_lines.append("")
    
    if api_verification.get("verifications"):
        report_lines.append("### Endpoint Status Summary")
        report_lines.append("")
        report_lines.append("| Status | Count |")
        report_lines.append("|--------|-------|")
        report_lines.append(f"| Implemented | {implemented_endpoints} |")
        report_lines.append(f"| Stub | {stub_endpoints} |")
        report_lines.append(f"| Placeholder | {api_summary.get('placeholder', 0)} |")
        report_lines.append(f"| Missing | {missing_endpoints} |")
        report_lines.append("")
        
        # List missing endpoints
        if missing_endpoints > 0:
            report_lines.append("### Missing Endpoints")
            report_lines.append("")
            for verification in api_verification.get("verifications", []):
                if verification.get("status") == "missing":
                    endpoint = verification.get("endpoint", {})
                    report_lines.append(f"- `{endpoint.get('method', '')} {endpoint.get('path', '')}` - {endpoint.get('description', '')}")
            report_lines.append("")
    
    # Pattern Detection Results
    report_lines.append("## Pattern Detection Results")
    report_lines.append("")
    
    for pattern_type in ["stubs", "placeholders", "mocks", "incomplete"]:
        pattern_data = pattern_summary.get(pattern_type, {})
        count = pattern_data.get("count", 0)
        matches = pattern_data.get("matches", [])
        
        if count > 0:
            report_lines.append(f"### {pattern_type.title()}")
            report_lines.append("")
            report_lines.append(f"**Total Found:** {count}")
            report_lines.append("")
            
            if matches:
                report_lines.append("**Sample Matches:**")
                report_lines.append("")
                for match in matches[:10]:  # Show first 10
                    file_path = match.get("file", "")
                    line_num = match.get("line", 0)
                    content = match.get("content", "")[:100]
                    report_lines.append(f"- `{file_path}:{line_num}` - {content}")
                report_lines.append("")
    
    # Recommendations
    report_lines.append("## Recommendations")
    report_lines.append("")
    
    recommendations = []
    
    if missing_endpoints > 0:
        recommendations.append({
            "priority": "HIGH",
            "action": "Implement missing API endpoints",
            "count": missing_endpoints
        })
    
    if stub_count > 0:
        recommendations.append({
            "priority": "HIGH",
            "action": "Replace stub implementations with real implementations",
            "count": stub_count
        })
    
    if placeholder_count > 0:
        recommendations.append({
            "priority": "MEDIUM",
            "action": "Complete placeholder implementations",
            "count": placeholder_count
        })
    
    if mock_count > 0:
        recommendations.append({
            "priority": "MEDIUM",
            "action": "Remove mock data from production code",
            "count": mock_count
        })
    
    if recommendations:
        for rec in recommendations:
            report_lines.append(f"### {rec['priority']} Priority")
            report_lines.append("")
            report_lines.append(f"- **{rec['action']}** ({rec['count']} instances)")
            report_lines.append("")
    else:
        report_lines.append("No recommendations at this time.")
        report_lines.append("")
    
    # Known Stubs from Inventory
    if stubs_content:
        report_lines.append("## Known Stubs Inventory")
        report_lines.append("")
        report_lines.append("The following stubs were identified in `STUBS_INVENTORY.md`:")
        report_lines.append("")
        # Extract key sections from stubs inventory
        lines = stubs_content.split('\n')
        in_critical = False
        for line in lines[:100]:  # First 100 lines
            if "## Critical Stubs" in line:
                in_critical = True
            if in_critical and line.startswith('#'):
                report_lines.append(line)
            elif in_critical and line.strip():
                report_lines.append(line)
        report_lines.append("")
    
    # Write report
    OUTPUT_FILE.parent.mkdir(parents=True, exist_ok=True)
    with open(OUTPUT_FILE, 'w', encoding='utf-8') as f:
        f.write('\n'.join(report_lines))
    
    print(f"Report generated: {OUTPUT_FILE}")
    print(f"  Critical Issues: {len(critical_issues)}")
    print(f"  Recommendations: {len(recommendations)}")

if __name__ == "__main__":
    generate_report()

