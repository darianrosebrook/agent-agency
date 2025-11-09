# WorkingSpec Contract Documentation

**Version**: 3.0.0  
**Author**: @darianrosebrook

---

## Overview

The `WorkingSpec` is the canonical contract type that defines comprehensive working specifications for autonomous task execution in Agent Agency V3. It serves as the single source of truth for task constraints, acceptance criteria, quality gates, and execution plans.

**Location**: `agent-agency-contracts::working_spec::WorkingSpec`

---

## Core Structure

### Field Reference

```rust
pub struct WorkingSpec {
    /// Contract version for compatibility (e.g., "3.0.0")
    pub version: String,

    /// Working spec identifier (e.g., FEAT-001, FIX-042)
    pub id: String,

    /// Human-readable title
    pub title: String,

    /// Detailed task description
    pub description: String,

    /// High-level objectives to achieve
    pub goals: Vec<String>,

    /// Risk tier: 1=critical, 2=standard, 3=low
    pub risk_tier: u32,

    /// Execution constraints and safety limits
    pub constraints: WorkingSpecConstraints,

    /// Acceptance criteria in Given-When-Then format
    /// NOTE: Field name is `acceptance_criteria`, but accepts `acceptance` as alias for backward compatibility
    #[serde(alias = "acceptance")]
    pub acceptance_criteria: Vec<AcceptanceCriterion>,

    /// Comprehensive test plan
    pub test_plan: TestPlan,

    /// Rollback and recovery procedures
    pub rollback_plan: RollbackPlan,

    /// Workspace context and dependencies
    pub context: WorkingSpecContext,

    /// Non-functional requirements (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub non_functional_requirements: Option<NonFunctionalRequirements>,

    /// CAWS validation results (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_results: Option<ValidationResults>,

    /// Quality gates that must be satisfied (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality_gates: Option<crate::planning_io::QualityGates>,

    /// Scope boundaries for the working spec
    /// NOTE: This is a Vec<ScopeRestrictions>, not a single scope object
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub scope: Vec<ScopeRestrictions>,

    /// Metadata and versioning information (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<WorkingSpecMetadata>,

    /// Execution milestones defining the implementation plan
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub milestones: Vec<Milestone>,

    /// Change budget defining resource limits
    pub change_budget: ChangeBudget,

    /// File changes that will be made
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub file_changes: Vec<FileChange>,

    /// Test coverage targets (optional)
    /// NOTE: This is Option<CoverageTargets>, not a direct struct
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coverage_targets: Option<CoverageTargets>,

    /// High-level overview of the working spec
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub overview: String,

    /// When the working spec was created
    #[schemars(with = "String")]
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// When the working spec was last updated
    #[schemars(with = "String")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
```

---

## Critical Field Access Patterns

### Acceptance Criteria

**Correct**:
```rust
// Field name is acceptance_criteria
for criterion in &working_spec.acceptance_criteria {
    // Process criterion
}
```

**Backward Compatible**:
```rust
// JSON deserialization accepts "acceptance" as alias
// But code should use acceptance_criteria
#[serde(alias = "acceptance")]
pub acceptance_criteria: Vec<AcceptanceCriterion>
```

### Budget Limits

**Correct**:
```rust
// Budget limits are nested in constraints.budget_limits
let max_files = working_spec.constraints
    .budget_limits
    .as_ref()
    .and_then(|b| b.max_files);

let max_loc = working_spec.constraints
    .budget_limits
    .as_ref()
    .and_then(|b| b.max_loc);
```

**Helper Method**:
```rust
// Use convenience methods when available
let max_files = working_spec.max_files();
let max_loc = working_spec.max_loc();
```

### Scope Restrictions

**Correct**:
```rust
// Scope is Vec<ScopeRestrictions>, not a single object
for restriction in &working_spec.scope {
    let allowed = &restriction.allowed_paths;
    let blocked = &restriction.blocked_paths;
}
```

**Incorrect** (old pattern):
```rust
// DO NOT use - this pattern doesn't exist
let in_paths = working_spec.scope.in_paths;  // ❌
let out_paths = working_spec.scope.out_paths;  // ❌
```

### Coverage Targets

**Correct**:
```rust
// Coverage targets are Option<CoverageTargets>
if let Some(ref ct) = working_spec.coverage_targets {
    if let Some(ref line_coverage) = ct.line_coverage {
        // Use line_coverage
    }
    if let Some(ref branch_coverage) = ct.branch_coverage {
        // Use branch_coverage
    }
    if let Some(ref mutation_score) = ct.mutation_score {
        // Use mutation_score
    }
}
```

**Incorrect** (old pattern):
```rust
// DO NOT use - coverage_targets is Option, not direct struct
let line_cov = working_spec.coverage_targets.line_coverage;  // ❌
```

### Non-Functional Requirements

**Correct**:
```rust
// Non-functional requirements are Option<NonFunctionalRequirements>
if let Some(ref nfr) = working_spec.non_functional_requirements {
    let security_reqs = &nfr.security;
    let performance_reqs = nfr.performance.as_ref();
    let accessibility_reqs = &nfr.accessibility;
}
```

**Incorrect** (old pattern):
```rust
// DO NOT use - non_functional_requirements is Option
let security = working_spec.non_functional_requirements.security_requirements;  // ❌
```

### Risk Tier

**Correct**:
```rust
// Risk tier is u32, but some APIs expect u8
let risk_tier = working_spec.risk_tier as u8;  // Convert if needed
```

**Type Mismatches**:
- `WorkingSpec.risk_tier`: `u32`
- `Milestone.risk_tier`: `u8`
- Convert with `as u8` when assigning to milestone

---

## Migration Guide

### From Legacy Types

If migrating from older code that used different field names:

1. **acceptance → acceptance_criteria**
   ```rust
   // Old
   for criterion in &spec.acceptance { }
   
   // New
   for criterion in &spec.acceptance_criteria { }
   ```

2. **scope.in/out → scope[].allowed_paths/blocked_paths**
   ```rust
   // Old
   let in_paths = spec.scope.in;
   
   // New
   let in_paths: Vec<String> = spec.scope
       .iter()
       .flat_map(|r| r.allowed_paths.clone())
       .collect();
   ```

3. **coverage_targets (direct) → Option<CoverageTargets>**
   ```rust
   // Old
   let line_cov = spec.coverage_targets.line_coverage;
   
   // New
   let line_cov = spec.coverage_targets
       .as_ref()
       .and_then(|ct| ct.line_coverage);
   ```

4. **non_functional_requirements (direct) → Option<NonFunctionalRequirements>**
   ```rust
   // Old
   let security = spec.non_functional_requirements.security_requirements;
   
   // New
   let security = spec.non_functional_requirements
       .as_ref()
       .map(|nfr| nfr.security.clone())
       .unwrap_or_default();
   ```

5. **constraints.max_files/max_loc → constraints.budget_limits**
   ```rust
   // Old
   let max_files = spec.constraints.max_files;
   
   // New
   let max_files = spec.constraints
       .budget_limits
       .as_ref()
       .and_then(|b| b.max_files);
   ```

---

## Common Patterns

### Creating a WorkingSpec

```rust
use agent_agency_contracts::working_spec::WorkingSpec;
use agent_agency_contracts::AcceptanceCriterion;

let spec = WorkingSpec {
    version: "3.0.0".to_string(),
    id: "FEAT-001".to_string(),
    title: "Add user authentication".to_string(),
    description: "Implement JWT-based authentication".to_string(),
    goals: vec!["Secure user access".to_string()],
    risk_tier: 1,  // Critical
    constraints: WorkingSpecConstraints {
        budget_limits: Some(BudgetLimits {
            max_files: Some(25),
            max_loc: Some(1000),
            max_migrations: Some(5),
        }),
        max_duration_minutes: Some(60),
        // ... other constraints
    },
    acceptance_criteria: vec![
        AcceptanceCriterion {
            id: "A1".to_string(),
            given: "User is logged out".to_string(),
            when: "User submits valid credentials".to_string(),
            then: "User is logged in".to_string(),
        },
    ],
    scope: vec![
        ScopeRestrictions {
            allowed_paths: vec!["src/auth/".to_string()],
            blocked_paths: vec!["src/billing/".to_string()],
        },
    ],
    coverage_targets: Some(CoverageTargets {
        line_coverage: Some(0.90),
        branch_coverage: Some(0.95),
        mutation_score: Some(0.70),
    }),
    non_functional_requirements: Some(NonFunctionalRequirements {
        security: vec!["input-validation".to_string(), "csrf-protection".to_string()],
        performance: None,
        accessibility: vec![],
    }),
    // ... other required fields
    test_plan: TestPlan::default(),
    rollback_plan: RollbackPlan::default(),
    context: WorkingSpecContext::default(),
    change_budget: ChangeBudget::default(),
    file_changes: vec![],
    milestones: vec![],
    overview: String::new(),
    created_at: chrono::Utc::now(),
    updated_at: chrono::Utc::now(),
};
```

### Accessing Budget Limits

```rust
// Use helper methods for convenience
let max_files = spec.max_files().unwrap_or(25);
let max_loc = spec.max_loc().unwrap_or(1000);

// Or access directly
if let Some(ref budget) = spec.constraints.budget_limits {
    if let Some(max_files) = budget.max_files {
        // Use max_files
    }
}
```

### Iterating Over Scope Restrictions

```rust
for restriction in &spec.scope {
    println!("Allowed paths: {:?}", restriction.allowed_paths);
    println!("Blocked paths: {:?}", restriction.blocked_paths);
    
    // Check if a file is in scope
    let file_path = "src/auth/user.rs";
    let is_allowed = restriction.allowed_paths.iter()
        .any(|allowed| file_path.starts_with(allowed));
    let is_blocked = restriction.blocked_paths.iter()
        .any(|blocked| file_path.starts_with(blocked));
    
    if is_allowed && !is_blocked {
        // File is in scope
    }
}
```

### Checking Coverage Targets

```rust
fn validate_coverage(spec: &WorkingSpec, risk_tier: u32) -> Result<()> {
    let min_line_coverage = match risk_tier {
        1 => 0.90,
        2 => 0.80,
        3 => 0.70,
        _ => return Err(anyhow!("Invalid risk tier")),
    };
    
    if let Some(ref ct) = spec.coverage_targets {
        if let Some(ref line_coverage) = ct.line_coverage {
            if *line_coverage < min_line_coverage {
                return Err(anyhow!(
                    "Line coverage {} below minimum {}",
                    line_coverage * 100.0,
                    min_line_coverage * 100.0
                ));
            }
        }
    }
    
    Ok(())
}
```

---

## Related Types

### WorkingSpecConstraints

```rust
pub struct WorkingSpecConstraints {
    pub budget_limits: Option<BudgetLimits>,
    pub max_duration_minutes: Option<u32>,
    // ... other constraints
}
```

### BudgetLimits

```rust
pub struct BudgetLimits {
    pub max_files: Option<u32>,
    pub max_loc: Option<u32>,
    pub max_migrations: Option<u32>,
}
```

### ScopeRestrictions

```rust
pub struct ScopeRestrictions {
    pub allowed_paths: Vec<String>,
    pub blocked_paths: Vec<String>,
}
```

### CoverageTargets

```rust
pub struct CoverageTargets {
    pub line_coverage: Option<f64>,      // 0.0-1.0
    pub branch_coverage: Option<f64>,    // 0.0-1.0
    pub mutation_score: Option<f64>,      // 0.0-1.0
}
```

### NonFunctionalRequirements

```rust
pub struct NonFunctionalRequirements {
    pub security: Vec<String>,
    pub performance: Option<PerformanceRequirements>,
    pub accessibility: Vec<String>,
}
```

---

## Validation and Quality Gates

WorkingSpec validation ensures:

1. **Field Consistency**: All required fields are present
2. **Type Safety**: All field types match contract definitions
3. **Constraint Validation**: Budget limits, scope boundaries, and risk tiers are valid
4. **Acceptance Criteria**: At least one acceptance criterion is defined
5. **Coverage Targets**: Coverage targets match risk tier requirements

---

## JSON Schema

The canonical JSON schema is available at:
- `iterations/v3/docs/contracts/working-spec.schema.json`

This schema is generated from the Rust type definitions using `schemars` and can be used for:
- API validation
- Client code generation
- Documentation generation
- Schema validation tools

---

## Integration Points

### Planning Engine

The planning engine consumes `WorkingSpec` to generate execution plans:

```rust
use agent_agency_contracts::working_spec::WorkingSpec;
use agent_agency_contracts::planning_io::ExecutionPlan;

fn generate_plan(spec: &WorkingSpec) -> Result<ExecutionPlan> {
    // Convert working spec to execution plan
    // Uses acceptance_criteria, constraints, scope, etc.
}
```

### Council Review

The council review system validates `WorkingSpec` against constitutional invariants:

```rust
fn review_spec(spec: &WorkingSpec) -> Result<CouncilVerdict> {
    // Validates risk_tier, coverage_targets, non_functional_requirements
    // Ensures quality gates are appropriate for risk tier
}
```

### Autonomous Executor

The autonomous executor uses `WorkingSpec` to guide task execution:

```rust
async fn execute_task(spec: &WorkingSpec) -> Result<ExecutionResult> {
    // Uses acceptance_criteria for validation
    // Uses constraints for resource limits
    // Uses scope for file boundary checks
}
```

---

## Troubleshooting

### Common Compilation Errors

**Error**: `no field named acceptance`
- **Fix**: Use `acceptance_criteria` instead of `acceptance`

**Error**: `no field named max_files on WorkingSpecConstraints`
- **Fix**: Use `constraints.budget_limits.as_ref().and_then(|b| b.max_files)`

**Error**: `no field named in_paths on Vec<ScopeRestrictions>`
- **Fix**: Iterate over `scope` and access `allowed_paths`/`blocked_paths` from each `ScopeRestriction`

**Error**: `expected Option<CoverageTargets>, found CoverageTargets`
- **Fix**: Wrap in `Some()`: `coverage_targets: Some(CoverageTargets { ... })`

**Error**: `expected u8, found u32` (risk_tier)
- **Fix**: Convert with `as u8`: `risk_tier: spec.risk_tier as u8`

---

## References

- **Source Code**: `iterations/v3/agent-agency-contracts/src/working_spec.rs`
- **JSON Schema**: `iterations/v3/docs/contracts/working-spec.schema.json`
- **Planning Types**: `iterations/v3/agent-agency-contracts/src/planning_io.rs`
- **CAWS Guide**: `docs/agents/full-guide.md`


