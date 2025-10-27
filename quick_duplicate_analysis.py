#!/usr/bin/env python3
"""
Quick Duplicate Analysis for Agent Agency V3
Focuses on the specific duplicates found
"""

import os
import hashlib
from pathlib import Path

def get_file_hash(filepath):
    """Calculate SHA256 hash of file content"""
    try:
        with open(filepath, 'rb') as f:
            return hashlib.sha256(f.read()).hexdigest()
    except:
        return None

def analyze_specific_duplicates():
    """Analyze the specific duplicate groups found"""
    
    # Known duplicate groups from the audit
    duplicate_groups = [
        [
            "iterations/v3/agent-research/src/reinforcement.rs",
            "iterations/v3/agent-research/src/learning_algorithms/reinforcement.rs"
        ],
        [
            "iterations/v3/agent-research/src/adapters.rs",
            "iterations/v3/agent-research/src/self_prompting_agent/adapters.rs"
        ],
        [
            "iterations/v3/agent-research/src/vector_search/text_processing.rs",
            "iterations/v3/agent-research/src/vector_search/vector_text_processing.rs"
        ],
        [
            "iterations/v3/agent-research/src/vector_search/vector_embedding.rs",
            "iterations/v3/agent-research/src/vector_search/embedding.rs"
        ],
        [
            "iterations/v3/agent-research/src/vector_search/qdrant.rs",
            "iterations/v3/agent-research/src/vector_search/vector_qdrant.rs"
        ],
        [
            "iterations/v3/agent-research/src/vector_search/vector_search_ops.rs",
            "iterations/v3/agent-research/src/vector_search/search.rs"
        ],
        [
            "iterations/v3/system-observability/src/analytics_dashboard/dashboard_metrics.rs",
            "iterations/v3/system-observability/src/analytics/analytics_metrics.rs"
        ],
        [
            "iterations/v3/system-observability/src/analytics_dashboard/updates.rs",
            "iterations/v3/system-observability/src/analytics/updates.rs"
        ],
        [
            "iterations/v3/system-observability/src/analytics_dashboard/data.rs",
            "iterations/v3/system-observability/src/analytics/data.rs"
        ],
        [
            "iterations/v3/system-observability/src/analytics_dashboard/ml.rs",
            "iterations/v3/system-observability/src/analytics/ml.rs"
        ],
        [
            "iterations/v3/agent-cli/src/bin/cli_binary.rs",
            "iterations/v3/data-interfaces/src/bin/advanced-cli.rs"
        ],
        [
            "iterations/v3/agent-cli/src/bin/main.rs",
            "iterations/v3/data-interfaces/examples/demo.rs"
        ],
        [
            "iterations/v3/agent-cli/src/bin/api-server.rs",
            "iterations/v3/data-interfaces/src/bin/api-server.rs"
        ]
    ]
    
    print("🔍 DUPLICATE ANALYSIS REPORT")
    print("=" * 50)
    
    confirmed_duplicates = []
    false_positives = []
    
    for i, group in enumerate(duplicate_groups, 1):
        print(f"\n📋 Group {i}:")
        
        hashes = []
        for file_path in group:
            if os.path.exists(file_path):
                file_hash = get_file_hash(file_path)
                hashes.append(file_hash)
                rel_path = os.path.relpath(file_path, "iterations/v3")
                print(f"  📄 {rel_path}")
            else:
                print(f"  ❌ {file_path} (file not found)")
                hashes.append(None)
        
        # Check if all files have the same hash
        if len(set(h for h in hashes if h is not None)) == 1:
            confirmed_duplicates.append(group)
            print(f"  ✅ CONFIRMED DUPLICATE")
        else:
            false_positives.append(group)
            print(f"  ⚠️  NOT EXACT DUPLICATE")
    
    print(f"\n📊 SUMMARY:")
    print(f"  ✅ Confirmed duplicates: {len(confirmed_duplicates)}")
    print(f"  ⚠️  False positives: {len(false_positives)}")
    
    print(f"\n🎯 RECOMMENDATIONS:")
    print("-" * 20)
    
    if confirmed_duplicates:
        print("❌ CRITICAL: Remove exact duplicates:")
        for i, group in enumerate(confirmed_duplicates, 1):
            print(f"\n  Group {i}:")
            # Suggest keeping the first file, removing others
            keep_file = group[0]
            remove_files = group[1:]
            
            rel_keep = os.path.relpath(keep_file, "iterations/v3")
            print(f"    📌 KEEP: {rel_keep}")
            
            for remove_file in remove_files:
                rel_remove = os.path.relpath(remove_file, "iterations/v3")
                print(f"    🗑️  REMOVE: {rel_remove}")
    
    if false_positives:
        print("\n⚠️  Review similar files:")
        for i, group in enumerate(false_positives, 1):
            print(f"\n  Group {i}:")
            for file_path in group:
                rel_path = os.path.relpath(file_path, "iterations/v3")
                print(f"    📄 {rel_path}")
            print(f"    🔍 Review for consolidation opportunities")

if __name__ == "__main__":
    analyze_specific_duplicates()
