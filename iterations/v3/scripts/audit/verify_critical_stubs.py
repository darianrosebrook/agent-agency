#!/usr/bin/env python3
"""
Verify critical stubs identified in STUBS_INVENTORY.md
"""

import json
import re
from pathlib import Path
from typing import Dict, List, Any
from datetime import datetime, timezone

V3_ROOT = Path(__file__).parent.parent.parent
OUTPUT_FILE = V3_ROOT / "docs-status" / "audit-reports" / "critical-stubs-verification.json"

def verify_stub_database_operations() -> Dict[str, Any]:
    """Verify StubDatabaseOperations usage."""
    file_path = V3_ROOT / "agent-orchestration" / "src" / "orchestration" / "unified_orchestrator_factory.rs"
    
    if not file_path.exists():
        return {"status": "error", "message": "File not found"}
    
    content = file_path.read_text(encoding='utf-8')
    
    # Check if stub is marked as dead_code
    is_dead_code = "#[allow(dead_code)]" in content or "allow(dead_code)" in content
    
    # Check if stub is actually used
    # Look for StubDatabaseOperations::new or similar instantiation
    stub_used_patterns = [
        r"StubDatabaseOperations::new",
        r"StubDatabaseOperations\s*\(",
        r"Arc::new\(StubDatabaseOperations",
        r"Box::new\(StubDatabaseOperations",
    ]
    
    stub_used = False
    for pattern in stub_used_patterns:
        if re.search(pattern, content):
            stub_used = True
            break
    
    # Check if real adapter is used instead
    real_adapter_used = "DatabaseOperationsAdapter::new" in content
    
    # Find line numbers
    stub_def_line = None
    for i, line in enumerate(content.split('\n'), 1):
        if "struct StubDatabaseOperations" in line:
            stub_def_line = i
            break
    
    return {
        "file": str(file_path.relative_to(V3_ROOT)),
        "stub_def_line": stub_def_line,
        "is_dead_code": is_dead_code,
        "stub_used_in_production": stub_used,
        "real_adapter_used": real_adapter_used,
        "status": "acceptable" if is_dead_code and not stub_used and real_adapter_used else "critical" if stub_used else "acceptable_reference"
    }

def verify_clip_embedding_stub() -> Dict[str, Any]:
    """Verify CLIP embedding stub usage."""
    file_path = V3_ROOT / "data-infrastructure" / "src" / "embedding" / "provider.rs"
    
    if not file_path.exists():
        return {"status": "error", "message": "File not found"}
    
    content = file_path.read_text(encoding='utf-8')
    
    # Check if stub is called in production code
    stub_called = "generate_embeddings_stub" in content
    
    # Check if it's called from the main generate_embeddings method
    generate_embeddings_calls_stub = False
    lines = content.split('\n')
    for i, line in enumerate(lines):
        if "async fn generate_embeddings" in line and "stub" not in line.lower():
            # Check next 15 lines for stub call
            for j in range(i+1, min(i+16, len(lines))):
                if "generate_embeddings_stub" in lines[j]:
                    generate_embeddings_calls_stub = True
                    break
            if generate_embeddings_calls_stub:
                break
    
    # Find line numbers
    stub_def_line = None
    stub_call_line = None
    for i, line in enumerate(lines, 1):
        if "async fn generate_embeddings_stub" in line:
            stub_def_line = i
        if generate_embeddings_calls_stub and "generate_embeddings_stub" in line and "self." in line and stub_def_line and i > stub_def_line:
            stub_call_line = i
            break
    
    return {
        "file": str(file_path.relative_to(V3_ROOT)),
        "stub_def_line": stub_def_line,
        "stub_call_line": stub_call_line,
        "stub_called_in_production": generate_embeddings_calls_stub,
        "status": "critical" if generate_embeddings_calls_stub else "acceptable"
    }

def verify_context_manager_stub() -> Dict[str, Any]:
    """Verify StubContextManager usage."""
    file_path = V3_ROOT / "agent-memory" / "src" / "context_management.rs"
    
    if not file_path.exists():
        return {"status": "error", "message": "File not found"}
    
    content = file_path.read_text(encoding='utf-8')
    
    # Check if stub is used as fallback
    stub_used_as_fallback = "StubContextManager" in content and ("use stub" in content.lower() or "fallback" in content.lower() or "no database" in content.lower())
    
    # Check if it's instantiated
    stub_instantiated = "StubContextManager {" in content or "StubContextManager::new" in content
    
    # Find line numbers
    stub_def_line = None
    stub_instantiation_line = None
    lines = content.split('\n')
    for i, line in enumerate(lines, 1):
        if "struct StubContextManager" in line:
            stub_def_line = i
        if stub_instantiated and ("StubContextManager {" in line or "StubContextManager::new" in line):
            stub_instantiation_line = i
            break
    
    return {
        "file": str(file_path.relative_to(V3_ROOT)),
        "stub_def_line": stub_def_line,
        "stub_instantiation_line": stub_instantiation_line,
        "stub_used_as_fallback": stub_used_as_fallback,
        "stub_instantiated": stub_instantiated,
        "status": "acceptable_fallback" if stub_used_as_fallback else "critical" if stub_instantiated else "acceptable"
    }

def verify_milestone_operations() -> Dict[str, Any]:
    """Verify milestone operations implementation."""
    file_path = V3_ROOT / "data-infrastructure" / "src" / "client" / "orchestrator.rs"
    
    if not file_path.exists():
        return {"status": "error", "message": "File not found"}
    
    content = file_path.read_text(encoding='utf-8')
    
    # Check if operations are implemented
    create_milestone_impl = "async fn create_milestone" in content and "INSERT INTO milestones" in content
    get_milestone_impl = "async fn get_milestone" in content and "SELECT" in content
    update_milestone_impl = "async fn update_milestone" in content and "UPDATE milestones" in content
    delete_milestone_impl = "async fn delete_milestone" in content and "DELETE FROM milestones" in content
    
    # Check for "not implemented" errors
    not_implemented = "not.*implemented" in content.lower() or "NotImplemented" in content
    
    return {
        "file": str(file_path.relative_to(V3_ROOT)),
        "create_milestone_implemented": create_milestone_impl,
        "get_milestone_implemented": get_milestone_impl,
        "update_milestone_implemented": update_milestone_impl,
        "delete_milestone_implemented": delete_milestone_impl,
        "has_not_implemented_errors": not_implemented,
        "status": "implemented" if all([create_milestone_impl, get_milestone_impl, update_milestone_impl, delete_milestone_impl]) else "partial"
    }

def main():
    """Main verification function."""
    results = {
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "critical_stubs": {}
    }
    
    print("Verifying critical stubs...")
    
    results["critical_stubs"]["stub_database_operations"] = verify_stub_database_operations()
    results["critical_stubs"]["clip_embedding_stub"] = verify_clip_embedding_stub()
    results["critical_stubs"]["context_manager_stub"] = verify_context_manager_stub()
    results["critical_stubs"]["milestone_operations"] = verify_milestone_operations()
    
    # Summary
    critical_count = sum(1 for v in results["critical_stubs"].values() if v.get("status") == "critical")
    acceptable_count = sum(1 for v in results["critical_stubs"].values() if "acceptable" in v.get("status", ""))
    
    results["summary"] = {
        "critical": critical_count,
        "acceptable": acceptable_count,
        "total_verified": len(results["critical_stubs"])
    }
    
    OUTPUT_FILE.parent.mkdir(parents=True, exist_ok=True)
    with open(OUTPUT_FILE, 'w', encoding='utf-8') as f:
        json.dump(results, f, indent=2, ensure_ascii=False)
    
    print(f"\nCritical stubs verification complete!")
    print(f"  Critical: {critical_count}")
    print(f"  Acceptable: {acceptable_count}")
    print(f"  Results saved to {OUTPUT_FILE}")

if __name__ == "__main__":
    main()

