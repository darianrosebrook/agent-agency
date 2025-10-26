#!/bin/bash
# Precise fix for lib.rs imports - avoid over-replacement

echo "Starting precise import fixes..."

# Fix the mangled replacements
for lib_file in $(find iterations/v3 -name "lib.rs" -exec grep -l "pattern_prompting\|data_processing_prompting\|mcp_prompting" {} \;); do
    echo "Fixing mangled imports in $lib_file"
    # Fix the most common mangled patterns
    sed -i '' 's/pattern_prompting_pattern_types/pattern_types/g' "$lib_file"
    sed -i '' 's/data_processing_prompting_data_processing_types/data_processing_types/g' "$lib_file"
    sed -i '' 's/mcp_prompting_mcp_types/mcp_types/g' "$lib_file"
    sed -i '' 's/memory_prompting_memory_types/memory_types/g' "$lib_file"
    sed -i '' 's/worker_prompting_worker_types/worker_types/g' "$lib_file"
    sed -i '' 's/cache_prompting_cache_types/cache_types/g' "$lib_file"
    sed -i '' 's/extraction_prompting_extraction_types/extraction_types/g' "$lib_file"
    sed -i '' 's/engine_prompting_engine_types/engine_types/g' "$lib_file"
    sed -i '' 's/council_prompting_council_types/council_types/g' "$lib_file"
    sed -i '' 's/embedding_prompting_embedding_types/embedding_types/g' "$lib_file"
    sed -i '' 's/enricher_prompting_enricher_types/enricher_types/g' "$lib_file"
    sed -i '' 's/indexer_prompting_indexer_types/indexer_types/g' "$lib_file"
    sed -i '' 's/ingestor_prompting_ingestor_types/ingestor_types/g' "$lib_file"
    sed -i '' 's/knowledge_prompting_knowledge_types/knowledge_types/g' "$lib_file"
    sed -i '' 's/evaluator_prompting_evaluator_types/evaluator_types/g' "$lib_file"
    sed -i '' 's/benchmark_prompting_benchmark_types/benchmark_types/g' "$lib_file"
    sed -i '' 's/parallel_prompting_parallel_types/parallel_types/g' "$lib_file"
    sed -i '' 's/provenance_prompting_provenance_types/provenance_types/g' "$lib_file"
    sed -i '' 's/recovery_prompting_recovery_types/recovery_types/g' "$lib_file"
    sed -i '' 's/reflexive_prompting_reflexive_types/reflexive_types/g' "$lib_file"
    sed -i '' 's/research_prompting_research_types/research_types/g' "$lib_file"
    sed -i '' 's/policy_prompting_policy_types/policy_types/g' "$lib_file"
    sed -i '' 's/integrity_prompting_integrity_types/integrity_types/g' "$lib_file"
    sed -i '' 's/health_prompting_health_types/health_types/g' "$lib_file"
    sed -i '' 's/state_prompting_state_types/state_types/g' "$lib_file"
done

# Fix internal references in other files
echo "Fixing internal references..."
for rs_file in $(find iterations/v3 -name "*.rs" -exec grep -l "crate::types::" {} \; | grep -v lib.rs); do
    echo "Fixing crate::types references in $rs_file"
    
    # Extract the crate name from path
    crate_path=$(echo "$rs_file" | sed 's|/src/.*||')
    crate_name=$(basename "$crate_path")
    
    case $crate_name in
        "agent-data-processing") new_module="data_processing_types" ;;
        "agent-mcp") new_module="mcp_types" ;;
        "agent-memory") new_module="memory_types" ;;
        "agent-workers") new_module="worker_types" ;;
        "caching") new_module="cache_types" ;;
        "claim-extraction") new_module="extraction_types" ;;
        "common-patterns") new_module="pattern_types" ;;
        "context-preservation-engine") new_module="engine_types" ;;
        "council") new_module="council_types" ;;
        "embedding-service") new_module="embedding_types" ;;
        "enrichers") new_module="enricher_types" ;;
        "indexers") new_module="indexer_types" ;;
        "ingestors") new_module="ingestor_types" ;;
        "knowledge-ingestor") new_module="knowledge_types" ;;
        "minimal-diff-evaluator") new_module="evaluator_types" ;;
        "model-benchmarking") new_module="benchmark_types" ;;
        "parallel-workers") new_module="parallel_types" ;;
        "provenance") new_module="provenance_types" ;;
        "recovery") new_module="recovery_types" ;;
        "reflexive-learning") new_module="reflexive_types" ;;
        "research") new_module="research_types" ;;
        "security-policy-enforcer") new_module="policy_types" ;;
        "self-prompting-agent") new_module="prompting_types" ;;
        "source-integrity") new_module="integrity_types" ;;
        "system-health-monitor") new_module="health_types" ;;
        "workers") new_module="worker_types" ;;
        "workspace-state-manager") new_module="state_types" ;;
        *) continue ;;
    esac
    
    sed -i '' "s/crate::types::/crate::${new_module}::/g" "$rs_file"
done

echo "Precise fixes completed!"
