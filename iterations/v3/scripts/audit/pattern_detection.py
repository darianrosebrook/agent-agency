#!/usr/bin/env python3
"""
Pattern Detection Script for V3 Documentation Reality Audit
Detects stubs, placeholders, mocks, and incomplete implementations
"""

import json
import re
from pathlib import Path
from typing import Dict, List, Any
from datetime import datetime, timezone

V3_ROOT = Path(__file__).parent.parent.parent
OUTPUT_DIR = V3_ROOT / "docs-status" / "audit-reports"
RESULTS_FILE = OUTPUT_DIR / "pattern-detection-results.json"

def find_patterns(pattern: str, search_path: Path, exclude_patterns: List[str] = None) -> List[Dict[str, Any]]:
    """Find patterns in source files."""
    matches = []
    exclude_patterns = exclude_patterns or []
    
    if not search_path.exists():
        return matches
    
    for rust_file in search_path.rglob("*.rs"):
        # Skip excluded paths
        file_str = str(rust_file)
        if any(exclude in file_str for exclude in exclude_patterns):
            continue
        
        try:
            content = rust_file.read_text(encoding='utf-8')
            lines = content.split('\n')
            
            for line_num, line in enumerate(lines, 1):
                if re.search(pattern, line, re.IGNORECASE):
                    matches.append({
                        "file": str(rust_file.relative_to(V3_ROOT)),
                        "line": line_num,
                        "content": line.strip()[:200]  # Limit content length
                    })
        except Exception as e:
            print(f"Error reading {rust_file}: {e}")
    
    return matches

def main():
    """Main pattern detection function."""
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    
    results = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "patterns": {}
    }
    
    # Stub implementations
    print("Detecting stub implementations...")
    stub_pattern = r"struct Stub|class Stub|impl.*Stub|_stub\(|stub_|Stub.*::new"
    stub_matches = find_patterns(stub_pattern, V3_ROOT / "src", exclude_patterns=["tests/", "examples/", "playground/"])
    results["patterns"]["stubs"] = {
        "count": len(stub_matches),
        "matches": stub_matches[:100]  # Limit to first 100
    }
    
    # Placeholder implementations
    print("Detecting placeholder implementations...")
    placeholder_pattern = r"PLACEHOLDER|placeholder.*not.*implemented|return.*placeholder"
    placeholder_matches = find_patterns(placeholder_pattern, V3_ROOT / "src", exclude_patterns=["tests/", "examples/", "playground/"])
    results["patterns"]["placeholders"] = {
        "count": len(placeholder_matches),
        "matches": placeholder_matches[:100]
    }
    
    # Mock data
    print("Detecting mock data...")
    mock_pattern = r"MOCK_DATA|mock.*data|fake.*data|hardcoded|test.*value|dummy.*value"
    mock_matches = find_patterns(mock_pattern, V3_ROOT / "src", exclude_patterns=["tests/", "examples/", "playground/"])
    results["patterns"]["mocks"] = {
        "count": len(mock_matches),
        "matches": mock_matches[:100]
    }
    
    # Incomplete implementations
    print("Detecting incomplete implementations...")
    incomplete_pattern = r"not.*implemented|NotImplemented|unimplemented"
    incomplete_matches = find_patterns(incomplete_pattern, V3_ROOT / "src", exclude_patterns=["tests/", "examples/", "playground/"])
    results["patterns"]["incomplete"] = {
        "count": len(incomplete_matches),
        "matches": incomplete_matches[:100]
    }
    
    # Write results
    with open(RESULTS_FILE, 'w', encoding='utf-8') as f:
        json.dump(results, f, indent=2, ensure_ascii=False)
    
    print(f"\nPattern detection complete!")
    print(f"  Stubs: {len(stub_matches)}")
    print(f"  Placeholders: {len(placeholder_matches)}")
    print(f"  Mocks: {len(mock_matches)}")
    print(f"  Incomplete: {len(incomplete_matches)}")
    print(f"  Results saved to {RESULTS_FILE}")

if __name__ == "__main__":
    main()

