<!-- 37567ed4-6c5a-430a-af85-b410f4d001b4 54e2a456-e1a7-4f04-aeb8-eab2c232f35e -->
# Tool Ecosystem Compilation Fix Plan

## Error Distribution Analysis

**Total Errors: 231 across 6 main files**

### Files Requiring Attention (by error count):
1. `evidence_collection_tools.rs` - 62 errors (27%)
2. `multi_modal_verification.rs` - 38 errors (16%)
3. `tool_chain_planner.rs` - 34 errors (15%)
4. `lib.rs` - 28 errors (12%)
5. `tool_registry.rs` - 22 errors (10%)
6. `conflict_resolution_tools.rs` - 10 errors (4%)

## Error Category Breakdown

### Category 1: Struct Field Mismatches (High Priority - 80 errors)

**AtomicClaim struct issues (40 errors)**
- Missing fields: `claim_text`, `source_location`, `metadata`, `extracted_at`, `evidence_requirements`, `dependencies`, `claim_type`
- Current definition at `evidence_collection_tools.rs:1460-1466` only has: `id`, `statement`, `confidence`, `source_context`, `verification_requirements`
- Used extensively in `evidence_collection_tools.rs`, `multi_modal_verification.rs`

**EvidenceItem struct issues (11 errors)**
- Missing field: `confidence`
- Current definition at `evidence_collection_tools.rs:1565-1572` has: `id`, `content`, `source`, `tags`, `timestamp`, `evidence_type`
- Needs `Deserialize`, `PartialEq`, `Eq`, `Hash` derives

**RegisteredTool struct issues (7 errors)**
- Code accesses `tool.name` and `tool.category` but these are nested in `tool.metadata.name` and `tool.metadata.category`
- Current definition at `tool_registry.rs:26-37`
- Errors in `tool_chain_planner.rs:254, 255, 340, 408, 426, 442, 464`

**ToolMetadata struct issues (3 errors)**
- Missing field: `id`
- Current definition at `tool_registry.rs:40-66`
- Used in `multi_modal_verification.rs:653`

### Category 2: Enum/Type Issues (High Priority - 45 errors)

**EvidenceType enum (32 errors)**
- Missing trait implementations: `Eq`, `Hash`, `Copy`
- Current definition at `evidence_collection_tools.rs:1553-1563`
- Causes HashMap operations to fail (16 errors for binary `==` operations)
- Affects: `multi_modal_verification.rs`, `evidence_collection_tools.rs`

**ToolCategory enum (3 errors)**
- Missing variants: `Analysis`, `Validation`
- Current definition at `tool_registry.rs:69-85`
- Used in `multi_modal_verification.rs:656`

**Missing types (10 errors)**
- `ClaimType` - undeclared type used in AtomicClaim
- `WorkflowTool`, `ReasoningTool`, `QualityGateTool`, `GovernanceTool` - undeclared types

### Category 3: Trait Implementation Issues (Medium Priority - 35 errors)

**Tool trait implementation (21 errors)**
- 7 structs don't implement `Tool` trait: `ClaimExtractor`, `FactVerifier`, `SourceValidator`, `EvidenceSynthesizer`, `DebateOrchestrator`, `ConsensusBuilder`
- Method signature mismatches:
  - `execute` has 2 parameters but trait expects 3 (3 errors)
  - `metadata` has incompatible type (3 errors)

**Debug trait for trait objects (3 errors)**
- `dyn Tool + Send + Sync` doesn't implement Debug
- `dyn Converter` doesn't implement Debug
- `dyn DiscoverySource` doesn't implement Debug
- Affects: `tool_registry.rs:30`, `tool_chain_planner.rs:616`, `tool_discovery.rs:21`

**Iterator Debug formatting (2 errors)**
- `dyn Iterator<Item = ValidationError<'_>> + Send + Sync` doesn't implement Debug
- Used in error formatting at `tool_registry.rs:143` and `lib.rs:375`

### Category 4: Serde Serialization (Medium Priority - 24 errors)

**Graph/NodeIndex serialization (16 errors)**
- `Graph<ToolNode, ToolEdge>` and `NodeIndex` from `petgraph` need Serialize/Deserialize
- Used in `ToolChain` struct at `tool_chain_planner.rs:68-73`
- Used in `ToolChainExecution` struct at `tool_chain_planner.rs:100-105`

**EvidenceItem deserialization (8 errors)**
- Missing `Deserialize` derive
- Used in `multi_modal_verification.rs:641, 667`

### Category 5: Borrowing/Ownership Issues (Low Priority - 20 errors)

**Temporary value lifetime (4 errors)**
- String temporaries dropped while borrowed in HashSet operations
- `evidence_collection_tools.rs:1206, 1210`
- `multi_modal_verification.rs:164, 167`

**Move semantics (9 errors)**
- Cannot move out of shared references for `EvidenceType`, `TaskComplexity`, `RiskLevel`
- `multi_modal_verification.rs:189` (2 errors)
- `tool_chain_planner.rs:605, 607` (2 errors)
- Various borrow of moved value errors (5 errors)

### Category 6: Method/Function Issues (Low Priority - 27 errors)

**Missing methods (7 errors)**
- `compute_plan_hash` called as method but is associated function (2 errors at `tool_chain_planner.rs:262, 353`)
- Missing methods on Arc-wrapped structs (5 errors)

**Missing dependencies (2 errors)**
- `rand` crate not in Cargo.toml
- Used at `tool_execution.rs:442`

**Miscellaneous (18 errors)**
- Generic argument count mismatch (13 errors)
- Type mismatches (8 errors)
- Ambiguous numeric types (3 errors)

## Implementation Strategy

### Phase 1: Struct Field Fixes (Priority 1)

**Step 1.1: Fix AtomicClaim struct**
- File: `evidence_collection_tools.rs:1460-1466`
- Add missing fields with appropriate types
- Add `ClaimType` enum definition

**Step 1.2: Fix EvidenceItem struct**
- File: `evidence_collection_tools.rs:1565-1572`
- Add `confidence: f64` field
- Add derives: `Deserialize`, `PartialEq`, `Eq`, `Hash`

**Step 1.3: Fix RegisteredTool field access**
- Files: `tool_chain_planner.rs` (7 locations)
- Change `tool.name` to `tool.metadata.name`
- Change `tool.category` to `tool.metadata.category`

**Step 1.4: Fix ToolMetadata struct**
- File: `tool_registry.rs:40-66`
- Add `id: String` field at top of struct

### Phase 2: Enum/Type Fixes (Priority 1)

**Step 2.1: Fix EvidenceType enum**
- File: `evidence_collection_tools.rs:1553-1563`
- Add derives: `PartialEq`, `Eq`, `Hash`, `Copy`, `Clone`

**Step 2.2: Fix ToolCategory enum**
- File: `tool_registry.rs:69-85`
- Add variants: `Analysis`, `Validation`

**Step 2.3: Add ClaimType enum**
- File: `evidence_collection_tools.rs` (near AtomicClaim)
- Define enum with variants for different claim types

### Phase 3: Trait Implementation Fixes (Priority 2)

**Step 3.1: Implement Tool trait for 7 structs**
- Files: `evidence_collection_tools.rs`, `conflict_resolution_tools.rs`
- Add proper `execute` method with 3 parameters
- Add proper `metadata` method returning `ToolMetadata`

**Step 3.2: Add Debug implementations for trait objects**
- Create wrapper types or use manual Debug implementations
- Files: `tool_registry.rs`, `tool_chain_planner.rs`, `tool_discovery.rs`

**Step 3.3: Fix Iterator Debug formatting**
- Collect iterator into Vec before formatting
- Files: `tool_registry.rs:143`, `lib.rs:375`

### Phase 4: Serde Fixes (Priority 2)

**Step 4.1: Fix Graph/NodeIndex serialization**
- Option A: Remove Serialize/Deserialize derives from ToolChain
- Option B: Create custom serialization for Graph types
- Files: `tool_chain_planner.rs:68, 100`

**Step 4.2: Add EvidenceItem Deserialize**
- Already added in Step 1.2

### Phase 5: Borrowing/Ownership Fixes (Priority 3)

**Step 5.1: Fix temporary value lifetimes**
- Use `let` bindings for intermediate values
- Files: `evidence_collection_tools.rs:1206, 1210`, `multi_modal_verification.rs:164, 167`

**Step 5.2: Fix move semantics**
- Add `.clone()` calls where needed
- Files: `multi_modal_verification.rs:189`, `tool_chain_planner.rs:605, 607`

### Phase 6: Method/Function Fixes (Priority 3)

**Step 6.1: Fix compute_plan_hash calls**
- Change from `self.compute_plan_hash(chain)` to `ToolChainPlanner::compute_plan_hash(chain)`
- Files: `tool_chain_planner.rs:262, 353`

**Step 6.2: Add rand dependency**
- File: `tool-ecosystem/Cargo.toml`
- Add `rand = "0.8"` to dependencies

**Step 6.3: Fix remaining method issues**
- Add missing methods or fix method signatures
- Various files

## Validation Steps

After each phase:
1. Run `cargo check --lib` to verify error count reduction
2. Track progress: expect ~40 errors fixed per phase
3. Address any new errors introduced by fixes

## Expected Outcome

- Phase 1: 80 errors → 151 errors remaining
- Phase 2: 151 errors → 106 errors remaining
- Phase 3: 106 errors → 71 errors remaining
- Phase 4: 71 errors → 47 errors remaining
- Phase 5: 47 errors → 27 errors remaining
- Phase 6: 27 errors → 0 errors remaining

Total estimated fixes: 6 phases to achieve clean compilation.


### To-dos

- [ ] Phase 1: Fix struct field mismatches (AtomicClaim, EvidenceItem, RegisteredTool, ToolMetadata) - 80 errors
- [ ] Phase 2: Fix enum/type issues (EvidenceType, ToolCategory, ClaimType) - 45 errors
- [ ] Phase 3: Implement missing traits (Tool, Debug) - 35 errors
- [ ] Phase 4: Fix serde serialization (Graph, NodeIndex, EvidenceItem) - 24 errors
- [ ] Phase 5: Fix borrowing/ownership issues - 20 errors
- [ ] Phase 6: Fix method/function issues (compute_plan_hash, rand dependency) - 27 errors