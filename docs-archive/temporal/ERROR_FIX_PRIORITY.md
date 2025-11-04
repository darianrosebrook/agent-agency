# Error Fix Priority - Three Worker Parallel Plan

## Overview
**Total Errors**: 312 compilation errors across workspace
**Impact**: Blocking compilation of `agent-workers` and `agent-orchestration` crates
**Goal**: Parallel fixing across 3 workers to resolve all errors within 4-6 hours

---

## Error Type Analysis

### Top 10 Error Categories:
1. **E0412**: cannot find type (WorkerId, SubTaskId, TaskId, TaskAnalysis, TaskScope, Priority) - **31 errors**
2. **E0412**: cannot find type (Priority) - **14 errors**
3. **E0433**: failed to resolve: use of undeclared type (SubTaskId) - **14 errors**
4. **E0308**: mismatched types - **27 errors**
5. **E0433**: failed to resolve: use of undeclared type (TaskPattern) - **12 errors**
6. **E0425**: cannot find function, tuple struct or tuple variant (SubTaskId) - **12 errors**
7. **E0433**: failed to resolve: use of undeclared type (Priority) - **10 errors**
8. **E0425**: cannot find function, tuple struct or tuple variant (WorkerId) - **10 errors**
9. **E0422**: cannot find struct, variant or union type (TaskScope) - **10 errors**
10. **E0560**: struct has no field (avg_execution_time_ms, file_patterns) - **3 errors**

### Error Distribution by Severity:
- **Critical (blocks compilation)**: 312 errors
- **Warnings (degrades code quality)**: 452 warnings in agent-orchestration
- **Dead code**: 100+ unused functions/structs across crates

---

## Three Worker Parallel Plan

### Worker 1: Type Resolution Specialist
**Focus**: `agent-workers` crate (highest error count - 282 errors)
**Primary Issues**: Missing type definitions and imports
**Estimated Time**: 2-3 hours

#### Task Breakdown:
1. **Add missing type imports** (E0412, E0433 errors)
   - `WorkerId`, `SubTaskId`, `TaskId` from `worker_types.rs`
   - `TaskAnalysis`, `TaskPattern` from appropriate modules
   - `Priority` from `parallel_types.rs` or `council_types.rs`

2. **Fix struct field definitions** (E0560 errors)
   - Add missing `avg_execution_time_ms` field to `WorkerPerformanceProfile`
   - Add missing `file_patterns` field to `ValidationRule`

3. **Resolve borrow checker issues** (E0382 errors)
   - Fix moved value issues in `learning_persistence.rs`
   - Extract `Copy` values before dropping references

4. **Fix numeric type mismatches** (E0308 errors)
   - Convert `f32` to `f64` in execution stats

#### Files to Focus:
- `src/parallel_types.rs` (E0559, E0560 errors)
- `src/learning_system.rs` (E0560, E0063 errors)
- `src/execution_stats.rs` (E0308 error)
- `src/lib.rs` (E0061 error)
- `src/coordinator.rs` (E0515 error)
- `src/learning/learning_persistence.rs` (E0382 errors)

---

### Worker 2: Struct & Type Mismatch Specialist
**Focus**: `agent-orchestration` crate (88 documented errors)
**Primary Issues**: Struct field mismatches and type conversions
**Estimated Time**: 2-3 hours

#### Task Breakdown:
1. **Fix missing struct fields** (E0063 errors)
   - Add missing fields to `OptimizationEvent`, `SubTask`, `TaskSpec`, `ErrorGroup`
   - Add missing fields to `WorkerPerformanceProfile`

2. **Resolve type mismatches** (E0308 errors)
   - Fix `scope_out` expecting `Option<TaskScope>` but receiving `TaskScope`
   - Fix `risk_tier` expecting `Option<RiskTier>` but receiving integer
   - Fix `acceptance` expecting `Option<String>` but receiving `Vec<String>`

3. **Fix enum variant usage**
   - Replace `RiskTier::Low/Medium/High` with `RiskTier::Tier1/Tier2/Tier3`
   - Replace `TaskPriority::Medium` with `TaskPriority::Normal`

4. **Fix function signatures**
   - Update `Council::new()` calls (remove `.await`, fix parameters)
   - Update `AutonomousExecutor::new()` calls (10+ parameters)

#### Files to Focus:
- `src/lib.rs` (function call issues)
- `src/types.rs` (missing type definitions)
- `src/council.rs` (function signature issues)
- `src/autonomous_executor.rs` (type redefinition issues)
- `src/multimodal_orchestration.rs` (duplicate definitions)

---

### Worker 3: System Integration Specialist
**Focus**: `system-quality-security` and remaining system crates
**Primary Issues**: Dependency issues and system-level fixes
**Estimated Time**: 1-2 hours

#### Task Breakdown:
1. **Fix dependency issues** (E0433 errors)
   - Replace `rand_distr::Laplace` with manual implementation
   - Copy implementation from `system-federated-ml/src/differential_privacy.rs:95-116`

2. **Add type annotations** (E0282 errors)
   - Use `BoundKey<SealingKey<AES_256_GCM>>` for encryption
   - Use `BoundKey<OpeningKey<AES_256_GCM>>` for decryption
   - Import from `ring::aead`

3. **Fix borrow checker issues** (E0505, E0507 errors)
   - Extract `Copy` values before dropping locks
   - Follow pattern from `src/keystore.rs:277-314`

4. **Resolve remaining numeric issues** (E0689 errors)
   - Fix ambiguous numeric types with explicit type annotations

#### Files to Focus:
- `system-quality-security/src/privacy_anonymization.rs`
- `system-quality-security/src/data_encryption.rs`
- `system-quality-security/src/keystore.rs`

---

## Parallel Execution Strategy

### Phase 1: Independent Work (0-60 min)
- **Worker 1**: Start with `agent-workers/src/parallel_types.rs` (high impact, independent)
- **Worker 2**: Start with `agent-orchestration/src/types.rs` (type definitions, independent)
- **Worker 3**: Start with `system-quality-security` fixes (solutions documented, independent)

### Phase 2: Interdependent Fixes (60-120 min)
- **Worker 1**: Continue with learning system and persistence fixes
- **Worker 2**: Fix function signatures and enum variants in `lib.rs` and `council.rs`
- **Worker 3**: Complete system-quality-security fixes

### Phase 3: Integration Testing (120-180 min)
- **Worker 1**: Resolve any remaining borrow checker issues
- **Worker 2**: Fix multimodal orchestration duplicates and imports
- **Worker 3**: Verify system integration and run tests

### Phase 4: Verification (180-240 min)
- All workers: Run `cargo check --workspace` to verify fixes
- Address any new errors introduced
- Run `cargo test` on affected crates
- Update documentation

---

## Success Metrics

### Completion Criteria:
- [ ] `cargo check --workspace` passes (0 errors)
- [ ] `cargo test` passes on fixed crates
- [ ] No new warnings introduced
- [ ] All documented solutions applied

### Quality Gates:
- **Worker 1**: `agent-workers` compiles successfully
- **Worker 2**: `agent-orchestration` compiles successfully
- **Worker 3**: `system-quality-security` compiles successfully

---

## Risk Mitigation

### Potential Blockers:
1. **Circular Dependencies**: May need to restructure imports
2. **Type Definition Conflicts**: Multiple definitions of same types
3. **API Contract Changes**: May require coordination between workers

### Communication Protocol:
- **Slack/Teams Channel**: Real-time updates on blockers
- **GitHub Issues**: Track specific error resolutions
- **Daily Standup**: 15-min sync on progress and blockers

### Rollback Plan:
- **Git Branches**: Each worker works on feature branch
- **Atomic Commits**: Small, focused commits for easy rollback
- **Master Backup**: Keep clean master branch for reference

---

## Timeline & Milestones

### Hour 1: Setup & Independent Work
- [ ] Workers assigned and briefed
- [ ] Development environment verified
- [ ] Initial fixes in parallel

### Hour 2: Interdependent Fixes
- [ ] Type definitions stabilized
- [ ] Function signatures updated
- [ ] System integrations complete

### Hour 3: Integration Testing
- [ ] Cross-worker dependencies resolved
- [ ] Compilation verified
- [ ] Tests passing

### Hour 4: Verification & Documentation
- [ ] Full workspace compiles
- [ ] All tests pass
- [ ] Documentation updated
- [ ] Success celebration! 🎉

---

## Files with Highest Error Density

### Critical Priority:
1. `agent-workers/src/parallel_types.rs` - 5+ errors
2. `agent-orchestration/src/lib.rs` - Multiple function call issues
3. `agent-orchestration/src/types.rs` - Missing type definitions
4. `agent-workers/src/learning_system.rs` - Struct field issues

### High Priority:
1. `agent-workers/src/learning/learning_persistence.rs` - Borrow checker issues
2. `agent-orchestration/src/multimodal_orchestration.rs` - Duplicate definitions
3. `agent-orchestration/src/council.rs` - Function signature issues
4. `system-quality-security/src/data_encryption.rs` - Type annotation issues

---

## Monitoring & Progress Tracking

### Real-time Dashboard:
```bash
# Run every 15 minutes during fixing
watch -n 900 'cargo check --workspace 2>&1 | grep "error\[" | wc -l'
```

### Individual Worker Progress:
- **Worker 1**: Track `cargo check -p agent-workers` error count
- **Worker 2**: Track `cargo check -p agent-orchestration` error count
- **Worker 3**: Track `cargo check -p system-quality-security` error count

### Completion Checklist:
- [ ] All 312 errors resolved
- [ ] Workspace compiles successfully
- [ ] No new errors introduced
- [ ] Tests pass on modified crates
- [ ] Documentation updated

---

*This plan leverages existing solution documentation and parallelizes work across error categories for maximum efficiency.*
