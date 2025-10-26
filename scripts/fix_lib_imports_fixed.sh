#!/bin/bash
# Fix lib.rs imports after types.rs renaming

echo "Starting lib.rs import fixes..."

# Handle each crate individually with correct mappings
declare -a CRATES=(
    "agent-data-processing:data_processing_types"
    "agent-mcp:mcp_types"
    "agent-memory:memory_types"
    "agent-workers:worker_types"
    "caching:cache_types"
    "claim-extraction:extraction_types"
    "common-patterns:pattern_types"
    "context-preservation-engine:engine_types"
    "council:council_types"
    "embedding-service:embedding_types"
    "enrichers:enricher_types"
    "indexers:indexer_types"
    "ingestors:ingestor_types"
    "knowledge-ingestor:knowledge_types"
    "minimal-diff-evaluator:evaluator_types"
    "model-benchmarking:benchmark_types"
    "parallel-workers:parallel_types"
    "provenance:provenance_types"
    "recovery:recovery_types"
    "reflexive-learning:reflexive_types"
    "research:research_types"
    "security-policy-enforcer:policy_types"
    "self-prompting-agent:prompting_types"
    "source-integrity:integrity_types"
    "system-health-monitor:health_types"
    "workers:worker_types"
    "workspace-state-manager:state_types"
)

for crate_info in "${CRATES[@]}"; do
    IFS=':' read -r crate_name new_module_name <<< "$crate_info"
    lib_file="iterations/v3/${crate_name}/src/lib.rs"
    
    if [[ -f "$lib_file" ]]; then
        echo "Updating $crate_name: types -> $new_module_name"
        
        # Update pub mod types; to pub mod new_name;
        sed -i '' "s/pub mod types;/pub mod ${new_module_name};/g" "$lib_file"
        
        # Update pub use types::*; to pub use new_name::*;
        sed -i '' "s/pub use types::\*/pub use ${new_module_name}::*/g" "$lib_file"
        
        # Update any other types:: references
        sed -i '' "s/types::/${new_module_name}::/g" "$lib_file"
    else
        echo "Skipping $crate_name - lib.rs not found at $lib_file"
    fi
done

# Handle special cases
echo "Handling special cases..."

# apple-silicon compat types
if [[ -f "iterations/v3/apple-silicon/src/lib.rs" ]]; then
    echo "Updating apple-silicon"
    sed -i '' "s/pub mod compat::types;/pub mod compat::compatibility_types;/g" "iterations/v3/apple-silicon/src/lib.rs"
fi

# council submodules
for council_file in iterations/v3/council/src/*/lib.rs; do
    if [[ -f "$council_file" ]]; then
        echo "Updating council submodule: $(basename $(dirname $council_file))"
        sed -i '' "s/pub mod types;/pub mod council_types;/g" "$council_file"
        sed -i '' "s/pub use types::\*/pub use council_types::*/g" "$council_file"
        sed -i '' "s/types::/council_types::/g" "$council_file"
    fi
done

echo "All lib.rs files updated!"
