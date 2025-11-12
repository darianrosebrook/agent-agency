#!/usr/bin/env python3
"""
Extract documented features from v3 documentation files.
Creates a structured JSON file with all documented features for verification.
"""

import json
import re
from pathlib import Path
from typing import Dict, List, Any
from datetime import datetime

V3_ROOT = Path(__file__).parent.parent.parent
DOCS_DIR = V3_ROOT / "docs"
OUTPUT_FILE = V3_ROOT / "docs-status" / "audit-reports" / "documented-features.json"

def extract_api_endpoints(content: str, source_file: str) -> List[Dict[str, Any]]:
    """Extract API endpoints from documentation."""
    endpoints = []
    
    # Pattern for API endpoint documentation
    # Matches: `POST /api/v1/tasks` - Description
    pattern = r'`(GET|POST|PUT|DELETE|PATCH)\s+([^\s`]+)`\s*[-–]\s*(.+?)(?=\n|$)'
    
    for match in re.finditer(pattern, content, re.MULTILINE):
        method = match.group(1)
        path = match.group(2)
        description = match.group(3).strip()
        
        endpoints.append({
            "method": method,
            "path": path,
            "description": description,
            "source_file": source_file,
            "documented": True
        })
    
    return endpoints

def extract_features_from_markdown(content: str, source_file: str) -> List[Dict[str, Any]]:
    """Extract features from markdown documentation."""
    features = []
    
    # Pattern for feature lists (bullet points with descriptions)
    # Matches: - **Feature Name**: Description
    feature_pattern = r'[-*]\s*\*\*([^*]+)\*\*:\s*(.+?)(?=\n[-*]|\n\n|$)'
    
    for match in re.finditer(feature_pattern, content, re.MULTILINE):
        feature_name = match.group(1).strip()
        description = match.group(2).strip()
        
        features.append({
            "name": feature_name,
            "description": description,
            "source_file": source_file,
            "documented": True
        })
    
    # Also extract from headers followed by descriptions
    header_pattern = r'^###+\s+(.+?)$\n(.+?)(?=^###+|$)'
    for match in re.finditer(header_pattern, content, re.MULTILINE | re.DOTALL):
        header = match.group(1).strip()
        description = match.group(2).strip()[:200]  # Limit description length
        
        if len(description) > 20:  # Only include if substantial description
            features.append({
                "name": header,
                "description": description,
                "source_file": source_file,
                "documented": True
            })
    
    return features

def extract_component_features(content: str, source_file: str) -> List[Dict[str, Any]]:
    """Extract component features from component documentation."""
    features = []
    
    # Look for "Key Features:" sections
    key_features_pattern = r'Key Features?:\s*\n((?:[-*]\s*.+?\n)+)'
    match = re.search(key_features_pattern, content, re.MULTILINE)
    
    if match:
        features_text = match.group(1)
        for line in features_text.split('\n'):
            if line.strip().startswith('-') or line.strip().startswith('*'):
                feature = line.strip().lstrip('-*').strip()
                if feature:
                    features.append({
                        "name": feature,
                        "description": "",
                        "source_file": source_file,
                        "documented": True,
                        "type": "component_feature"
                    })
    
    return features

def process_documentation_file(file_path: Path) -> Dict[str, Any]:
    """Process a single documentation file."""
    try:
        content = file_path.read_text(encoding='utf-8')
    except Exception as e:
        print(f"Error reading {file_path}: {e}")
        return {}
    
    relative_path = str(file_path.relative_to(V3_ROOT))
    
    result = {
        "file": relative_path,
        "features": [],
        "api_endpoints": [],
        "components": []
    }
    
    # Extract features
    features = extract_features_from_markdown(content, relative_path)
    result["features"].extend(features)
    
    # Extract component features
    component_features = extract_component_features(content, relative_path)
    result["components"].extend(component_features)
    
    # Extract API endpoints
    endpoints = extract_api_endpoints(content, relative_path)
    result["api_endpoints"].extend(endpoints)
    
    return result

def main():
    """Main extraction function."""
    output_dir = OUTPUT_FILE.parent
    output_dir.mkdir(parents=True, exist_ok=True)
    
    # Key documentation files to process
    key_files = [
        DOCS_DIR / "system-overview.md",
        DOCS_DIR / "architecture.md",
        V3_ROOT / "README.md",
        DOCS_DIR / "WEB_INTERFACE_README.md",
    ]
    
    # Also process component documentation
    components_dir = DOCS_DIR / "components"
    if components_dir.exists():
        key_files.extend(components_dir.glob("*.md"))
    
    all_features = []
    all_endpoints = []
    all_components = []
    
    for file_path in key_files:
        if not file_path.exists():
            continue
        
        print(f"Processing {file_path.name}...")
        result = process_documentation_file(file_path)
        
        all_features.extend(result.get("features", []))
        all_endpoints.extend(result.get("api_endpoints", []))
        all_components.extend(result.get("components", []))
    
    # Create output structure
    output_data = {
        "timestamp": datetime.utcnow().isoformat() + "Z",
        "summary": {
            "total_features": len(all_features),
            "total_endpoints": len(all_endpoints),
            "total_components": len(all_components),
            "files_processed": len(key_files)
        },
        "features": all_features,
        "api_endpoints": all_endpoints,
        "components": all_components
    }
    
    # Write output
    with open(OUTPUT_FILE, 'w', encoding='utf-8') as f:
        json.dump(output_data, f, indent=2, ensure_ascii=False)
    
    print(f"\nExtraction complete!")
    print(f"  Features: {len(all_features)}")
    print(f"  API Endpoints: {len(all_endpoints)}")
    print(f"  Components: {len(all_components)}")
    print(f"  Output: {OUTPUT_FILE}")

if __name__ == "__main__":
    main()

