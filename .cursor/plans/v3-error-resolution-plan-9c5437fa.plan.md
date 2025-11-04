<!-- 9c5437fa-0ad7-41ae-821b-efffdafb3faf bb6339ca-f257-4744-9643-1bd840a76079 -->
# V3 Crate Error Resolution - Four Worker Distribution

## Error Summary

**Total Errors: 1190**

- agent-orchestration: 546 errors (46%)
- agent-workers: 352 errors (30%)
- agent-research: 292 errors (24%)

**Top Error Types:**

- E0603 (private item access): 330 occurrences
- E0277 (trait bounds): 284 occurrences  
- E0308 (type mismatches): 111 occurrences
- E0560 (missing struct fields): 83 occurrences
- E0609 (field access on wrong type): 68 occurrences

## Worker Distribution Strategy

### Worker 1: Module Visibility & Imports (E0603, E0432, E0433)

**Priority: HIGHEST - Blocks other fixes**

**Estimated Errors: ~370**

**Rationale:** Privacy/import errors cascade and block type resolution. Must be fixed first.

**Tasks:**

1. Fix E0603 errors in agent-orchestration (329 occurrences)

   - Files: adapter.rs, autonomous_executor.rs, autonomous_integration.rs, backup_types.rs, council.rs, and 24 more
   - Action: Add `pub` modifiers or re-export through module hierarchy

2. Fix E0432/E0433 import errors (39 occurrences)

   - Files: audit_trail.rs, various integration files
   - Action: Update import paths after visibility fixes

**Success Criteria:** All E0603, E0432, E0433 errors resolved

---

### Worker 2: Trait Bounds & Serialization (E0277)

**Priority: HIGH - Type system foundation**

**Estimated Errors: ~284**

**Rationale:** Trait bound errors affect multiple crates and prevent compilation.

**Tasks:**

1. Fix serde serialization issues in agent-orchestration (125 occurrences)

   - Primary file: audit_trail.rs
   - Issue: `RwLock<GlobalAuditStats>` missing Serialize/Deserialize
   - Action: Add `#[serde(skip)]` or custom serialization

2. Fix trait bound errors in agent-workers (86 occurrences)

   - Files: channels.rs, coordinator.rs, core.rs, executor.rs, failure_taxonomy.rs
   - Action: Add missing trait bounds or derive macros

3. Fix trait bound errors in agent-research (73 occurrences)

   - Files: core.rs, entities.rs, extraction_types.rs, extractor.rs, historical.rs
   - Action: Implement missing traits or adjust bounds

**Success Criteria:** All E0277 errors resolved, types properly constrained

---

### Worker 3: Type Mismatches & Struct Fields (E0308, E0560, E0063)

**Priority: MEDIUM - Structural fixes**

**Estimated Errors: ~230**

**Rationale:** Type mismatches and missing fields are localized and can be fixed in parallel.

**Tasks:**

1. Fix type mismatch errors (E0308) - 111 occurrences

   - agent-workers: 81 in adaptive_selector.rs, coordinator.rs, dependency_graph.rs, execution_stats.rs, executor.rs
   - Action: Align types with function signatures

2. Fix missing struct fields (E0560) - 83 occurrences

   - agent-workers: 35 in coordinator.rs, executor.rs, learning_persistence.rs
   - agent-research: 46 in context.rs, core.rs, data_extractor.rs, detection.rs, entities.rs
   - Action: Add missing fields to struct initializations

3. Fix struct field access (E0063) - 36 occurrences

   - agent-workers: 28 in config_optimizer.rs, coordinator.rs, core.rs, executor.rs
   - Action: Update struct construction with all required fields

**Success Criteria:** All E0308, E0560, E0063 errors resolved

---

### Worker 4: Method/Field Access & Trait Implementations (E0609, E0599, E0119, E0412)

**Priority: MEDIUM-LOW - Depends on other fixes**

**Estimated Errors: ~306**

**Rationale:** These errors often resolve after visibility and trait fixes.

**Tasks:**

1. Fix field access errors (E0609) - 68 occurrences

   - agent-workers: 59 in adaptive_selector.rs, bridges.rs, config_optimizer.rs, coordinator.rs
   - Action: Fix field access patterns after type corrections

2. Fix method not found errors (E0599) - 57 occurrences

   - agent-research: 30 in agent_caws_integration.rs, context.rs, detection.rs, disambiguation.rs
   - Action: Implement missing methods or fix trait bounds

3. Fix trait implementation conflicts (E0119) - 56 occurrences

   - agent-orchestration: 27 in audit_trail.rs, autonomous_executor.rs, council.rs
   - agent-research: 29 in core.rs, extraction_types.rs, integration.rs, planner.rs
   - Action: Remove duplicate impls or move to appropriate modules

4. Fix undefined type errors (E0412) - 49 occurrences

   - agent-orchestration: 14 in audit_trail.rs
   - agent-research: 35 in code_analysis.rs, collector.rs, constitutional.rs, documentation.rs
   - Action: Add missing imports or type definitions

**Success Criteria:** All E0609, E0599, E0119, E0412 errors resolved

---

## Execution Order

**Phase 1 (Worker 1):** Module visibility fixes - MUST complete first

**Phase 2 (Workers 2-4 in parallel):** After Worker 1 completes, remaining workers can proceed in parallel

## Validation

After each worker completes:

```bash
cd iterations/v3
cargo check --package <crate-name> 2>&1 | grep "error\[E"
```

Final validation:

```bash
cd iterations/v3
cargo check --workspace
cargo test --workspace --no-run
```

## Notes

- Worker 1 is the critical path - other workers may need to wait
- Some errors will auto-resolve after visibility fixes
- Expect ~20% error reduction from cascading fixes
- Focus on structural fixes over warnings initially