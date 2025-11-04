# Missing Files Priority Analysis

**Date**: Analysis based on compilation errors and system architecture  
**Total Missing Files**: 340 files  
**Total Compilation Errors**: 178 errors across 3 crates

## Priority Tiers

### 🔴 **TIER 1: CRITICAL - Blocking Compilation** (Must Fix First)

These files are essential to resolve the 178 compilation errors and get the system building.

#### **agent-research** (131 errors - 74% of all errors)

**Immediate Priority - Missing Serialization Support:**
- `iterations/v3/agent-research/Cargo.toml` - ✅ Already exists (has schemars)
- **ROOT CAUSE**: Missing `serde` derive features in dependency usage
- **FILES NEEDED**: Files using `#[derive(Serialize, Deserialize)]` need proper serde import

**Critical Missing Files (Directly Impacting Compilation):**
- `iterations/v3/agent-research/src/extraction_types.rs` - Contains types needing serialization
- `iterations/v3/agent-research/src/persistence.rs` - Database persistence layer
- `iterations/v3/agent-research/src/learning_service.rs` - Core learning functionality
- `iterations/v3/agent-research/src/qualification.rs` - Content qualification logic
- `iterations/v3/agent-research/src/processor.rs` - Main processing pipeline
- `iterations/v3/agent-research/src/orchestrator.rs` - Coordination logic
- `iterations/v3/agent-research/src/performance_tracker.rs` - Metrics tracking
- `iterations/v3/agent-research/src/benchmark_runner.rs` - Benchmark execution

**Sub-modules (Check dependencies):**
- `iterations/v3/agent-research/src/coordinator/orchestrator.rs`
- `iterations/v3/agent-research/src/coordinator/state.rs`
- `iterations/v3/agent-research/src/decomposition/core.rs`
- `iterations/v3/agent-research/src/decomposition/extractor.rs`
- `iterations/v3/agent-research/src/disambiguation/entities.rs`
- `iterations/v3/agent-research/src/disambiguation/stage.rs`
- `iterations/v3/agent-research/src/evidence/collector.rs`
- `iterations/v3/agent-research/src/evidence/evidence_analysis.rs`
- `iterations/v3/agent-research/src/evidence/test_execution.rs`
- `iterations/v3/agent-research/src/verification/code_extractor.rs`
- `iterations/v3/agent-research/src/verification/keyword_matcher.rs`
- `iterations/v3/agent-research/src/verification/spec_analysis.rs`
- `iterations/v3/agent-research/src/verification/verification_types.rs`
- `iterations/v3/agent-research/src/verification/verifier.rs`

**Action**: Fix serde derive macro issues by ensuring all files have proper imports and serde features enabled.

#### **agent-orchestration** (32 errors - 18% of all errors)

**Immediate Priority - Private Type Visibility Issues:**
- `iterations/v3/agent-orchestration/src/quality_gates.rs` - Contains `QualityGateResult` (8 errors)
- `iterations/v3/agent-orchestration/src/decision_making.rs` - Contains `AggregatedChanges`, `RefinementDirective`, `RefinementChange` (6 errors)
- `iterations/v3/agent-orchestration/src/planning/plan_executor.rs` - Contains `BatchExecutionResult` (2 errors)
- `iterations/v3/agent-orchestration/src/planning/council_review.rs` - Uses private types (3 errors)
- `iterations/v3/agent-orchestration/src/verdict_aggregation.rs` - Contains `DissentingOpinion`, `WeightedContribution` (4 errors)
- `iterations/v3/agent-orchestration/src/planning/evidence.rs` - Contains `PlanningTaskResult` (3 errors)
- `iterations/v3/agent-orchestration/src/multimodal_orchestration.rs` - Contains `EnrichedEvidence`, `MultimodalContext`, `EnrichmentStats` (6 errors)

**Action**: Make these types `pub` or provide public accessor methods.

#### **agent-workers** (12 errors - 7% of all errors)

**Immediate Priority - Missing Schemars Dependency:**
- `iterations/v3/agent-workers/Cargo.toml` - ✅ Already exists (needs schemars added)
- Files using `#[derive(JsonSchema)]` or `#[schemars(...)]` attributes

**Action**: Add `schemars = "0.8"` to `agent-workers/Cargo.toml`

---

### 🟠 **TIER 2: HIGH PRIORITY - Core Infrastructure** (Critical for System Functionality)

These files are needed for core system operations but don't block compilation.

#### **agent-agency-contracts** (Core Contracts)
- `iterations/v3/agent-agency-contracts/src/engine.rs` - Task execution engine
- `iterations/v3/agent-agency-contracts/src/execution_artifacts.rs` - Artifact definitions
- `iterations/v3/agent-agency-contracts/src/task_executor_provider.rs` - Executor provider
- `iterations/v3/agent-agency-contracts/src/task_executor.rs` - Core executor implementation
- **Priority**: HIGH - Used by agent-orchestration and agent-workers

#### **agent-memory** (Memory System - Used by Research)
- `iterations/v3/agent-memory/src/lib.rs` - ✅ Already exists
- `iterations/v3/agent-memory/src/memory_types.rs` - Core type definitions
- `iterations/v3/agent-memory/src/context_management.rs` - Context handling
- `iterations/v3/agent-memory/src/workspace_registry.rs` - Workspace management
- `iterations/v3/agent-memory/src/vector_search/search_engine.rs` - Vector search (critical for research)
- `iterations/v3/agent-memory/src/vector_search/reranking.rs` - Result reranking
- **Priority**: HIGH - Required by agent-research for knowledge retrieval

#### **agent-data-processing** (Data Pipeline)
- `iterations/v3/agent-data-processing/Cargo.toml` - ✅ Already exists
- `iterations/v3/agent-data-processing/src/data_processing_types.rs` - Type definitions
- `iterations/v3/agent-data-processing/src/pipeline.rs` - Main pipeline
- `iterations/v3/agent-data-processing/src/ingestion.rs` - Data ingestion
- `iterations/v3/agent-data-processing/src/enrichment.rs` - Data enrichment
- `iterations/v3/agent-data-processing/src/indexing.rs` - Indexing logic
- **Priority**: HIGH - Used by multiple systems for data processing

#### **agent-mcp** (MCP Integration - Used by Workers)
- `iterations/v3/agent-mcp/Cargo.toml` - ✅ Already exists
- `iterations/v3/agent-mcp/src/lib.rs` - ✅ Already exists
- `iterations/v3/agent-mcp/src/server.rs` - MCP server implementation
- `iterations/v3/agent-mcp/src/tool_registry.rs` - Tool registration
- `iterations/v3/agent-mcp/src/tool_discovery/core.rs` - Tool discovery
- `iterations/v3/agent-mcp/src/mcp_caws_integration.rs` - CAWS integration
- **Priority**: HIGH - Required by agent-workers for MCP tool execution

#### **agent-model-management** (Model Management)
- `iterations/v3/agent-model-management/src/lib.rs` - ✅ Already exists
- `iterations/v3/agent-model-management/src/model_orchestration_service.rs` - Core service
- `iterations/v3/agent-model-management/src/inference/backends.rs` - Inference backends
- `iterations/v3/agent-model-management/src/deployment/orchestrator.rs` - Deployment logic
- `iterations/v3/agent-model-management/src/deployment/load_balancer.rs` - Load balancing
- **Priority**: HIGH - Required for model inference and management

---

### 🟡 **TIER 3: MEDIUM PRIORITY** (Important Features)

#### **agent-memory** (Advanced Features)
- `iterations/v3/agent-memory/src/consolidation/consolidation_engine.rs`
- `iterations/v3/agent-memory/src/consolidation/deduplication.rs`
- `iterations/v3/agent-memory/src/consolidation/semantic_clustering.rs`
- `iterations/v3/agent-memory/src/consolidation/summarization.rs`
- `iterations/v3/agent-memory/src/long_term_management/archival.rs`
- `iterations/v3/agent-memory/src/long_term_management/lifecycle.rs`
- `iterations/v3/agent-memory/src/long_term_management/retrieval.rs`
- `iterations/v3/agent-memory/src/decay.rs`
- `iterations/v3/agent-memory/src/provenance.rs`
- **Priority**: MEDIUM - Advanced memory features, can be implemented incrementally

#### **agent-research** (Advanced Research Features)
- `iterations/v3/agent-research/src/knowledge_seeker/database.rs`
- `iterations/v3/agent-research/src/knowledge_seeker/index.rs`
- `iterations/v3/agent-research/src/knowledge_seeker/scraping.rs`
- `iterations/v3/agent-research/src/knowledge_seeker/search.rs`
- `iterations/v3/agent-research/src/learning_algorithms/orchestrator.rs`
- `iterations/v3/agent-research/src/learning_algorithms/unsupervised.rs`
- `iterations/v3/agent-research/src/multimodal_retriever/core.rs`
- `iterations/v3/agent-research/src/multimodal_retriever/text_search.rs`
- `iterations/v3/agent-research/src/multimodal_retriever/visual_search.rs`
- `iterations/v3/agent-research/src/planning_agent/planner.rs`
- `iterations/v3/agent-research/src/planning_agent/validation_pipeline.rs`
- `iterations/v3/agent-research/src/planning_agent/spec_generation/working_spec_generator.rs`
- `iterations/v3/agent-research/src/self_prompting_agent/agent_caws_integration.rs`
- `iterations/v3/agent-research/src/self_prompting_agent/evaluation.rs`
- `iterations/v3/agent-research/src/self_prompting_agent/integration.rs`
- `iterations/v3/agent-research/src/self_prompting_agent/models.rs`
- `iterations/v3/agent-research/src/self_prompting_agent/profiling.rs`
- `iterations/v3/agent-research/src/self_prompting_agent/prompting_types.rs`
- `iterations/v3/agent-research/src/self_prompting_agent/prompting.rs`
- `iterations/v3/agent-research/src/self_prompting_agent/sandbox.rs`
- `iterations/v3/agent-research/src/self_prompting_agent/stubs.rs`
- `iterations/v3/agent-research/src/vector_search/embedding.rs`
- `iterations/v3/agent-research/src/vector_search/search.rs`
- `iterations/v3/agent-research/src/vector_search/vector_search_cache.rs`
- `iterations/v3/agent-research/src/multimodal_context_provider.rs`
- `iterations/v3/agent-research/src/unsupervised.rs`
- **Priority**: MEDIUM - Advanced research capabilities, can be built incrementally

#### **agent-orchestration** (Supporting Modules)
- `iterations/v3/agent-orchestration/src/planning/dependency_resolver.rs`
- `iterations/v3/agent-orchestration/src/planning/plan_generator.rs`
- `iterations/v3/agent-orchestration/src/planning/plan_types.rs`
- `iterations/v3/agent-orchestration/src/planning/planning_engine_impl.rs`
- `iterations/v3/agent-orchestration/src/planning/storage.rs`
- `iterations/v3/agent-orchestration/src/planning/worker_assignment.rs`
- `iterations/v3/agent-orchestration/src/planning/scope_guard.rs`
- `iterations/v3/agent-orchestration/src/planning/parallel_coordinator.rs` - ✅ Already exists (has errors)
- `iterations/v3/agent-orchestration/src/planning/waiver_integration.rs`
- `iterations/v3/agent-orchestration/src/planning/caws_integration.rs`
- `iterations/v3/agent-orchestration/src/planning/council_adapter.rs`
- `iterations/v3/agent-orchestration/src/planning/council_monitor.rs`
- `iterations/v3/agent-orchestration/src/planning/data_processing_adapter.rs`
- `iterations/v3/agent-orchestration/src/planning/memory_adapter.rs`
- `iterations/v3/agent-orchestration/src/planning/orchestrator_integration.rs`
- `iterations/v3/agent-orchestration/src/planning/research_adapter.rs`
- `iterations/v3/agent-orchestration/src/planning/tool_chain_adapter.rs`
- `iterations/v3/agent-orchestration/src/planning/tool_chain_bridge.rs`
- `iterations/v3/agent-orchestration/src/planning/tool_chain_types.rs`
- `iterations/v3/agent-orchestration/src/planning/legacy_plan_adapter.rs`
- `iterations/v3/agent-orchestration/src/planning/factory.rs`
- **Priority**: MEDIUM - Supporting planning infrastructure

#### **data-infrastructure** (Supporting Infrastructure)
- Already has most files - missing files are enhancements
- **Priority**: MEDIUM - Infrastructure improvements

---

### 🟢 **TIER 4: LOW PRIORITY** (Nice-to-Have, Tests, Documentation)

#### **Test Files**
- All `tests/` directories
- `iterations/v3/agent-agency-contracts/tests/schema_snapshot.rs`
- `iterations/v3/agent-constitutional-council/tests/basic_functionality.rs`
- `iterations/v3/agent-mcp/tests/tool_execution.rs`
- `iterations/v3/agent-memory/src/tests.rs`
- `iterations/v3/agent-orchestration/tests/integration_autonomous_executor.rs`
- **Priority**: LOW - Can be implemented after core functionality works

#### **Development Tools**
- `iterations/v3/development-tools/src/integration.rs`
- `iterations/v3/development-tools/src/analyzers/javascript.rs`
- `iterations/v3/development-tools/src/analyzers/typescript.rs`
- `iterations/v3/development-tools/src/analyzers/test.rs`
- `iterations/v3/development-tools/src/codemod/mod.rs`
- `iterations/v3/development-tools/src/templates/mod.rs`
- **Priority**: LOW - Developer tooling

#### **Documentation & Examples**
- `iterations/v3/docs/generate_diagram_example.rs`
- **Priority**: LOW - Documentation helpers

#### **Advanced Features**
- `iterations/v3/agent-constitutional-council/src/metrics.rs`
- `iterations/v3/agent-constitutional-council/src/judges/technical_auditor.rs`
- `iterations/v3/agent-orchestration/src/restored_examples.rs` - Example code
- `iterations/v3/agent-orchestration/src/coreml/demo.rs` - Demo code
- Various advanced features in system-* crates
- **Priority**: LOW - Can be implemented incrementally

---

## Recommended Parallel Work Strategy

### **Worker 1: Fix Compilation Errors (agent-research)**
**Focus**: Tier 1 - agent-research crate
**Estimated Impact**: Resolves 74% of compilation errors (131 errors)
**Tasks**:
1. Ensure all files using `#[derive(Serialize, Deserialize)]` have proper serde imports
2. Verify `serde.workspace = true` includes derive features
3. Add missing type definitions in extraction_types.rs, persistence.rs
4. Implement core processing files (processor.rs, orchestrator.rs, etc.)
5. Run `cargo check` after each major file completion

### **Worker 2: Fix Visibility Issues (agent-orchestration)**
**Focus**: Tier 1 - agent-orchestration crate
**Estimated Impact**: Resolves 18% of compilation errors (32 errors)
**Tasks**:
1. Make private types public in quality_gates.rs (QualityGateResult)
2. Make private types public in decision_making.rs (AggregatedChanges, etc.)
3. Make private types public in verdict_aggregation.rs
4. Make private types public in planning/evidence.rs (PlanningTaskResult)
5. Make private types public in multimodal_orchestration.rs (EnrichedEvidence, etc.)
6. Add missing serde derives where needed
7. Run `cargo check` after each module fix

### **Worker 3: Fix Dependencies & Core Infrastructure**
**Focus**: Tier 1 - agent-workers + Tier 2 - Core Infrastructure
**Estimated Impact**: Resolves 7% of compilation errors (12 errors) + enables system functionality
**Tasks**:
1. Add `schemars = "0.8"` to agent-workers/Cargo.toml
2. Fix serde attribute issues in agent-workers
3. Implement critical agent-agency-contracts files (engine.rs, task_executor.rs)
4. Implement critical agent-mcp files (server.rs, tool_registry.rs)
5. Implement critical agent-data-processing files (pipeline.rs, ingestion.rs)
6. Run `cargo check` after dependency fixes

---

## Success Criteria

- [ ] All 178 compilation errors resolved
- [ ] `cargo check --workspace` passes with zero errors
- [ ] Core crates (agent-research, agent-orchestration, agent-workers) compile successfully
- [ ] Critical infrastructure files implemented
- [ ] System can run basic end-to-end tasks

## Next Steps After Compilation Fixes

1. Address 552 warnings (unused variables, deprecated code, dead code)
2. Implement Tier 2 (High Priority) infrastructure files
3. Add comprehensive test coverage
4. Implement Tier 3 (Medium Priority) advanced features
5. Complete Tier 4 (Low Priority) polish and tooling

---

## File Count Summary

| Tier | Files | Priority | Blocks Compilation |
|------|-------|----------|-------------------|
| Tier 1 | ~50 | CRITICAL | YES |
| Tier 2 | ~40 | HIGH | NO (but needed for functionality) |
| Tier 3 | ~150 | MEDIUM | NO |
| Tier 4 | ~100 | LOW | NO |
| **Total** | **~340** | | |

**Recommendation**: Focus 100% on Tier 1 first, then proceed to Tier 2 once compilation succeeds.

