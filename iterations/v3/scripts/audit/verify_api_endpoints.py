#!/usr/bin/env python3
"""
Verify API endpoints documented vs implemented.
"""

import json
import re
from pathlib import Path
from typing import Dict, List, Any
from datetime import datetime, timezone

V3_ROOT = Path(__file__).parent.parent.parent
DOCS_FILE = V3_ROOT / "docs-status" / "audit-reports" / "documented-features.json"
OUTPUT_FILE = V3_ROOT / "docs-status" / "audit-reports" / "api-endpoint-verification.json"

def find_api_handlers() -> List[Dict[str, Any]]:
    """Find all API handler functions in the codebase."""
    handlers = []
    
    # Search in data-interfaces and data-interfaces-adapters
    search_dirs = [
        V3_ROOT / "data-interfaces" / "src",
        V3_ROOT / "data-interfaces-adapters" / "src",
    ]
    
    for search_dir in search_dirs:
        if not search_dir.exists():
            continue
        
        for rust_file in search_dir.rglob("*.rs"):
            try:
                content = rust_file.read_text(encoding='utf-8')
                
                # Look for handler functions
                # Pattern: pub async fn handle_* or pub fn handle_*
                handler_pattern = r'pub\s+(async\s+)?fn\s+(handle_|api_|get_|post_|put_|delete_)(\w+)'
                
                for match in re.finditer(handler_pattern, content):
                    handler_name = match.group(0)
                    line_num = content[:match.start()].count('\n') + 1
                    
                    handlers.append({
                        "file": str(rust_file.relative_to(V3_ROOT)),
                        "line": line_num,
                        "handler": handler_name,
                        "path": str(rust_file.relative_to(V3_ROOT))
                    })
            except Exception as e:
                print(f"Error reading {rust_file}: {e}")
    
    return handlers

def check_endpoint_implementation(endpoint: Dict[str, Any], handlers: List[Dict[str, Any]]) -> Dict[str, Any]:
    """Check if an endpoint has an implementation."""
    method = endpoint.get("method", "").upper()
    path = endpoint.get("path", "")
    
    # Extract path segments
    path_segments = [s for s in path.split('/') if s]
    
    # Look for handlers that might match
    matching_handlers = []
    
    for handler in handlers:
        handler_file = handler["file"]
        handler_name = handler["handler"].lower()
        
        # Check if handler file/name suggests it handles this endpoint
        # Simple heuristic: check if path segments appear in handler name or file
        matches = False
        for segment in path_segments:
            if segment and (segment.lower() in handler_name or segment.lower() in handler_file.lower()):
                matches = True
                break
        
        if matches:
            matching_handlers.append(handler)
    
    # Check for stub/placeholder in handler file
    is_stub = False
    has_placeholder = False
    
    if matching_handlers:
        handler_file = V3_ROOT / matching_handlers[0]["file"]
        if handler_file.exists():
            try:
                content = handler_file.read_text(encoding='utf-8')
                is_stub = "stub" in content.lower() or "Stub" in content
                has_placeholder = "placeholder" in content.lower() or "PLACEHOLDER" in content
            except:
                pass
    
    return {
        "endpoint": endpoint,
        "implemented": len(matching_handlers) > 0,
        "handler_count": len(matching_handlers),
        "handlers": matching_handlers,
        "is_stub": is_stub,
        "has_placeholder": has_placeholder,
        "status": "implemented" if len(matching_handlers) > 0 and not is_stub and not has_placeholder else "stub" if is_stub else "missing" if len(matching_handlers) == 0 else "placeholder"
    }

def main():
    """Main verification function."""
    if not DOCS_FILE.exists():
        print(f"Documented features file not found: {DOCS_FILE}")
        return
    
    with open(DOCS_FILE, 'r', encoding='utf-8') as f:
        docs_data = json.load(f)
    
    documented_endpoints = docs_data.get("api_endpoints", [])
    handlers = find_api_handlers()
    
    verifications = []
    
    for endpoint in documented_endpoints:
        verification = check_endpoint_implementation(endpoint, handlers)
        verifications.append(verification)
    
    # Summary
    implemented_count = sum(1 for v in verifications if v["status"] == "implemented")
    stub_count = sum(1 for v in verifications if v["status"] == "stub")
    placeholder_count = sum(1 for v in verifications if v["status"] == "placeholder")
    missing_count = sum(1 for v in verifications if v["status"] == "missing")
    
    output_data = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "summary": {
            "total_documented": len(documented_endpoints),
            "implemented": implemented_count,
            "stub": stub_count,
            "placeholder": placeholder_count,
            "missing": missing_count,
            "compliance_rate": (implemented_count / len(documented_endpoints) * 100) if documented_endpoints else 0
        },
        "verifications": verifications
    }
    
    OUTPUT_FILE.parent.mkdir(parents=True, exist_ok=True)
    with open(OUTPUT_FILE, 'w', encoding='utf-8') as f:
        json.dump(output_data, f, indent=2, ensure_ascii=False)
    
    print(f"\nAPI Endpoint Verification Complete!")
    print(f"  Documented: {len(documented_endpoints)}")
    print(f"  Implemented: {implemented_count}")
    print(f"  Stub: {stub_count}")
    print(f"  Placeholder: {placeholder_count}")
    print(f"  Missing: {missing_count}")
    print(f"  Compliance: {output_data['summary']['compliance_rate']:.1f}%")

if __name__ == "__main__":
    main()

