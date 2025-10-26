#!/bin/bash
# Fix lib.rs imports after types.rs renaming

# Define the mapping of crate names to new module names
declare -A CRATE_MAPPING=(
    ["agent-data-processing"]="data_processing_types"
    ["agent-mcp"]="mcp_types"
    ["agent-memory"]="memory_types"
    ["agent-workers"]="worker_types"
    ["apple-silicon"]="compatibility_types"  # for compat/types.rs
    ["caching"]="cache_types"
    ["claim-extraction"]="extraction_types"
    ["common-patterns"]="pattern_types"
    ["context-preservation-engine"]="engine_types"
    ["council"]="council_types"
    ["embedding-service"]="embedding_types"
    ["enrichers"]="enricher_types"
    ["indexers"]="indexer_types"
    ["ingestors"]="ingestor_types"
    ["knowledge-ingestor"]="knowledge_types"
    ["minimal-diff-evaluator"]="evaluator_types"
    ["model-benchmarking"]="benchmark_types"
    ["parallel-workers"]="parallel_types"
    ["provenance"]="provenance_types"
    ["recovery"]="recovery_types"
    ["reflexive-learning"]="reflexive_types"
    ["research"]="research_types"
    ["security-policy-enforcer"]="policy_types"
    ["self-prompting-agent"]="prompting_types"
    ["self-prompting-agent.backup"]="backup_prompting_types"
    ["source-integrity"]="integrity_types"
    ["system-health-monitor"]="health_types"
    ["workers"]="worker_types"
    ["workspace-state-manager"]="state_types"
)

for crate_path in iterations/v3/*/; do
    if [[ -d "$crate_path" ]]; then
        crate_name=$(basename "$crate_path")
        lib_file="${crate_path}src/lib.rs"
        
        if [[ -f "$lib_file" ]] && [[ ${CRATE_MAPPING[$crate_name]} ]]; then
            new_module_name=${CRATE_MAPPING[$crate_name]}
            echo "Updating $crate_name: types -> $new_module_name"
            
            # Update pub mod types; to pub mod new_name;
            sed -i '' "s/pub mod types;/pub mod ${new_module_name};/g" "$lib_file"
            
            # Update pub use types::*; to pub use new_name::*;
            sed -i '' "s/pub use types::\*/pub use ${new_module_name}::*/g" "$lib_file"
            
            # Update any other types:: references
            sed -i '' "s/types::/${new_module_name}::/g" "$lib_file"
        fi
    fi
done

echo "All lib.rs files updated!"
