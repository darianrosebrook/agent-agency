# CAWS Integration Review - Agent-Agency Updates Required

**Date**: 2025-01-XX  
**Author**: @darianrosebrook

## Executive Summary

Agent-agency's CAWS integration needs updates to align with recent CAWS framework improvements. The main gaps are:

1. **Multi-Spec Support**: CAWS now uses feature-specific specs (`.caws/specs/*.yaml`) but agent-agency still references legacy `.caws/working-spec.yaml`
2. **Complexity Tiers**: CAWS introduced three complexity modes (simple/standard/enterprise) with tiered quality requirements
3. **Spec Resolution**: CAWS has intelligent spec resolution that agent-agency should leverage
4. **Enhanced Validation**: CAWS validation now includes mode-aware quality gates and waiver recognition

## Current State Analysis

### Files Reviewed

1. **`caws_integration.rs`** - Main bridge between CAWS working specs and execution plans
2. **`caws_quality_gates.rs`** - Quality gates executor with waiver support ✅ (mostly current)
3. **`caws_adjudication_cycle.rs`** - Five-stage adjudication cycle ✅ (current)
4. **`caws_debate_scorer.rs`** - Debate scoring algorithm ✅ (current)
5. **`caws_checker.rs`** - Worker-level compliance checking
6. **`caws_policy.rs`** - Recovery-specific policy (domain-separated, OK)

### Current Implementation Strengths

✅ **Quality Gates Integration**: `caws_quality_gates.rs` correctly implements waiver-aware quality gate execution  
✅ **Adjudication Cycle**: Five-stage cycle properly implemented with claim extraction  
✅ **Debate Scoring**: CAWS debate formula correctly implemented  
✅ **Worker Compliance**: Basic CAWS checking at worker level

### Gaps Identified

❌ **Multi-Spec Support**: Hardcoded references to `.caws/working-spec.yaml`  
❌ **Complexity Tiers**: No awareness of simple/standard/enterprise modes  
❌ **Spec Resolution**: No use of CAWS spec resolution system  
❌ **Mode-Aware Validation**: Quality requirements don't adapt to complexity tier

## Required Updates

### 1. Multi-Spec Support (HIGH PRIORITY)

**Current Issue**:  
```rust
// caws_integration.rs - Hardcoded spec path
pub fn spec_to_plan(&self, working_spec: WorkingSpec) -> Result<ContractExecutionPlan>
```

**Required Changes**:

1. **Add Spec Resolution Module**:
   ```rust
   // New: src/planning/caws_spec_resolver.rs
   pub struct CawsSpecResolver {
       project_root: PathBuf,
   }
   
   impl CawsSpecResolver {
       /// Resolve spec using CAWS priority system:
       /// 1. Feature-specific spec (via spec_id): .caws/specs/<id>.yaml
       /// 2. Explicit path (via spec_file)
       /// 3. Auto-detect: If only 1 spec exists, use it
       /// 4. Legacy fallback: .caws/working-spec.yaml
       pub fn resolve_spec(
           &self,
           spec_id: Option<&str>,
           spec_file: Option<&Path>,
       ) -> Result<PathBuf>;
       
       /// List all available specs
       pub fn list_specs(&self) -> Result<Vec<SpecInfo>>;
       
       /// Detect if multi-agent context (multiple specs exist)
       pub fn is_multi_agent_context(&self) -> bool;
   }
   ```

2. **Update `caws_integration.rs`**:
   ```rust
   impl CawsPlanBridge {
       /// Load spec using resolver (preferred)
       pub fn load_spec(
           &self,
           spec_id: Option<&str>,
           spec_file: Option<&Path>,
       ) -> Result<WorkingSpec> {
           let resolver = CawsSpecResolver::new(&self.project_root)?;
           let spec_path = resolver.resolve_spec(spec_id, spec_file)?;
           self.load_spec_from_path(&spec_path)
       }
       
       /// Legacy method (deprecated)
       #[deprecated(note = "Use load_spec with spec_id instead")]
       pub fn load_legacy_spec(&self) -> Result<WorkingSpec> {
           // Load from .caws/working-spec.yaml
       }
   }
   ```

3. **Update Orchestrator Calls**:
   ```rust
   // Update all calls to use spec_id parameter
   bridge.load_spec(Some("user-auth"), None)?;
   ```

### 2. Complexity Tier Support (HIGH PRIORITY)

**Current Issue**:  
Quality requirements are hardcoded based on risk tier only:
```rust
// caws_integration.rs:343
let (min_coverage, min_mutation) = match risk_tier {
    1 => (0.90, 0.70),
    2 => (0.80, 0.50),
    3 => (0.70, 0.30),
```

**Required Changes**:

1. **Add Complexity Mode Detection**:
   ```rust
   // New: src/planning/caws_complexity_mode.rs
   #[derive(Debug, Clone, Copy, PartialEq, Eq)]
   pub enum CawsComplexityMode {
       Simple,    // 70% coverage, 30% mutation
       Standard, // 80% coverage, 50% mutation
       Enterprise, // 90% coverage, 70% mutation
   }
   
   impl CawsComplexityMode {
       /// Detect mode from .caws/config.yaml or .caws/mode file
       pub fn detect(project_root: &Path) -> Result<Self>;
       
       /// Get quality requirements for mode + risk tier combination
       pub fn quality_requirements(
           &self,
           risk_tier: u8,
       ) -> QualityRequirements;
   }
   ```

2. **Update Evidence Gate Creation**:
   ```rust
   // caws_integration.rs
   fn create_evidence_gate(
       &self,
       risk_tier: u8,
       complexity_mode: CawsComplexityMode, // NEW
   ) -> Result<EvidenceGate> {
       let requirements = complexity_mode.quality_requirements(risk_tier);
       Ok(EvidenceGate {
           min_coverage: requirements.line_coverage,
           min_branch_coverage: requirements.branch_coverage,
           min_mutation_score: requirements.mutation_score,
           // ...
       })
   }
   ```

3. **Update Quality Gates Executor**:
   ```rust
   // caws_quality_gates.rs
   impl CawsQualityGateExecutor {
       pub fn execute_quality_gates(
           &self,
           context: &str,
           complexity_mode: Option<CawsComplexityMode>, // NEW
       ) -> Result<CawsQualityGateResult> {
           // Pass mode to script if supported
           let mut cmd = Command::new("node");
           cmd.arg(&self.quality_gates_script);
           if let Some(mode) = complexity_mode {
               cmd.arg("--mode").arg(format!("{:?}", mode).to_lowercase());
           }
           // ...
       }
   }
   ```

### 3. Spec Resolution Integration (MEDIUM PRIORITY)

**Required Changes**:

1. **Add Spec Info Structure**:
   ```rust
   // src/planning/caws_spec_resolver.rs
   #[derive(Debug, Clone)]
   pub struct SpecInfo {
       pub id: String,
       pub path: PathBuf,
       pub title: String,
       pub risk_tier: u8,
       pub mode: Option<CawsComplexityMode>,
       pub last_modified: SystemTime,
   }
   ```

2. **Update Orchestrator to Use Resolver**:
   ```rust
   // In unified_orchestrator.rs or similar
   let resolver = CawsSpecResolver::new(&project_root)?;
   
   // Warn if using legacy spec in multi-agent context
   if resolver.is_multi_agent_context() && spec_id.is_none() {
       warn!("Multiple specs detected but no spec_id provided. Using legacy spec.");
       warn!("Consider using: caws specs list to see available specs");
   }
   
   let spec = bridge.load_spec(spec_id.as_deref(), None)?;
   ```

### 4. Mode-Aware Validation (MEDIUM PRIORITY)

**Required Changes**:

1. **Update Validation Rules**:
   ```rust
   // caws_integration.rs
   fn validate_risk_tier_constraints(
       &self,
       working_spec: &WorkingSpec,
       complexity_mode: CawsComplexityMode, // NEW
   ) -> Result<()> {
       let requirements = complexity_mode.quality_requirements(working_spec.risk_tier);
       
       // Check coverage requirements based on mode + tier
       if let Some(ref ct) = working_spec.coverage_targets {
           if let Some(ref lc) = ct.line_coverage {
               if *lc < requirements.line_coverage {
                   return Err(anyhow!(
                       "Mode {:?} + Tier {} requires {:.0}% line coverage, spec has {:.0}%",
                       complexity_mode,
                       working_spec.risk_tier,
                       requirements.line_coverage * 100.0,
                       lc * 100.0
                   ));
               }
           }
       }
       // ...
   }
   ```

### 5. Worker-Level Updates (LOW PRIORITY)

**Current**: `caws_checker.rs` uses basic validation

**Required Updates**:

1. **Add Mode Awareness**:
   ```rust
   // caws_checker.rs
   impl CawsChecker {
       pub async fn check_compliance(
           &self,
           task: &str,
           complexity_mode: Option<CawsComplexityMode>, // NEW
       ) -> Result<CawsCheckResult, WorkerError> {
           // Adjust validation thresholds based on mode
           // ...
       }
   }
   ```

## Implementation Plan

### Phase 1: Multi-Spec Support (Week 1)

1. ✅ Create `caws_spec_resolver.rs` module
2. ✅ Update `caws_integration.rs` to use resolver
3. ✅ Update orchestrator calls to pass `spec_id`
4. ✅ Add tests for spec resolution

### Phase 2: Complexity Tiers (Week 1-2)

1. ✅ Create `caws_complexity_mode.rs` module
2. ✅ Update evidence gate creation
3. ✅ Update quality gates executor
4. ✅ Add mode detection from config

### Phase 3: Integration & Testing (Week 2)

1. ✅ Update all CAWS integration points
2. ✅ Add integration tests
3. ✅ Update documentation
4. ✅ Verify backward compatibility

## Backward Compatibility

**Critical**: All changes must maintain backward compatibility:

- Legacy `.caws/working-spec.yaml` still works (fallback)
- If no mode detected, default to `Standard`
- Existing code continues to work without `spec_id`

## Testing Requirements

1. **Unit Tests**:
   - Spec resolution priority order
   - Mode detection from config
   - Quality requirements calculation

2. **Integration Tests**:
   - Multi-spec workflow
   - Mode-aware validation
   - Legacy spec fallback

3. **E2E Tests**:
   - Full adjudication cycle with multi-spec
   - Quality gates with mode awareness

## Documentation Updates

1. Update `agent-orchestration/README.md` with multi-spec usage
2. Add complexity mode documentation
3. Update CAWS integration examples

## Risk Assessment

**Low Risk**:
- Multi-spec support (additive, backward compatible)
- Complexity modes (additive, defaults to Standard)

**Medium Risk**:
- Spec resolution changes (requires testing)
- Mode-aware validation (may affect existing workflows)

**Mitigation**:
- Feature flags for new behavior
- Comprehensive test coverage
- Gradual rollout

## Success Criteria

✅ Agent-agency can resolve feature-specific specs  
✅ Quality requirements adapt to complexity mode  
✅ Backward compatibility maintained  
✅ All tests pass  
✅ Documentation updated




