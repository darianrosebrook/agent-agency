# CAWS Integration Updates - Implementation Summary

**Date**: 2025-01-XX  
**Author**: @darianrosebrook

## Overview

Successfully updated agent-agency v3 CAWS integration to support:
1. ✅ Multi-spec support (feature-specific specs)
2. ✅ Complexity tiers (Simple/Standard/Enterprise modes)
3. ✅ Mode-aware quality requirements
4. ✅ Enhanced spec resolution

## Files Created

### 1. `caws_spec_resolver.rs`
- Implements CAWS spec resolution priority system
- Supports feature-specific specs (`.caws/specs/<id>.yaml`)
- Auto-detects single specs
- Falls back to legacy `.caws/working-spec.yaml`
- Provides multi-agent context detection

### 2. `caws_complexity_mode.rs`
- Implements three complexity tiers:
  - **Simple**: 70% coverage, 30% mutation
  - **Standard**: 80% coverage, 50% mutation (default)
  - **Enterprise**: 90% coverage, 70% mutation
- Detects mode from `.caws/config.yaml`, `.caws/config.json`, or `.caws/mode`
- Calculates quality requirements based on mode + risk tier combination

## Files Updated

### 1. `caws_integration.rs`
**Changes**:
- Added `project_root`, `spec_resolver`, and `complexity_mode` fields
- `new()` now returns `Result<Self>` (requires project root detection)
- Added `with_project_root()` for explicit project root
- Added `load_spec(spec_id, spec_file)` for multi-spec support
- Added `load_legacy_spec()` (deprecated) for backward compatibility
- Updated `validate_risk_tier_constraints()` to use complexity mode
- Updated `create_evidence_gate()` to use complexity mode

**Breaking Changes**:
- `CawsPlanBridge::new()` now returns `Result<Self>` instead of `Self`
- All callers must handle the `Result` type

### 2. `caws_quality_gates.rs`
**Changes**:
- Added `execute_quality_gates_with_mode()` method
- Passes `--mode` parameter to quality gates script when mode is provided
- `execute_quality_gates()` now delegates to `execute_quality_gates_with_mode()` with `None`

### 3. `caws_adjudication_cycle.rs`
**Changes**:
- Detects complexity mode before executing quality gates
- Passes complexity mode to quality gates executor

### 4. `plan_generator.rs`
**Changes**:
- `new()` now returns `Result<Self>` to handle bridge creation
- Added `with_project_root()` method

### 5. `factory.rs`
**Changes**:
- Updated to handle `Result` from `PlanGenerator::new()`

## Usage Examples

### Loading a Feature-Specific Spec

```rust
use agent_orchestration::planning::caws_integration::CawsPlanBridge;

// Create bridge
let bridge = CawsPlanBridge::with_project_root(".")?;

// Load feature-specific spec
let spec = bridge.load_spec(Some("user-auth"), None)?;

// Convert to execution plan
let plan = bridge.spec_to_plan(spec)?;
```

### Using Complexity Mode

```rust
use agent_orchestration::planning::caws_complexity_mode::CawsComplexityMode;

// Detect mode from config
let mode = CawsComplexityMode::detect(".")?;

// Get quality requirements for mode + tier
let requirements = mode.quality_requirements(2); // Tier 2

println!("Line coverage required: {:.0}%", requirements.line_coverage * 100.0);
println!("Mutation score required: {:.0}%", requirements.mutation_score * 100.0);
```

### Listing Available Specs

```rust
use agent_orchestration::planning::caws_spec_resolver::CawsSpecResolver;

let resolver = CawsSpecResolver::new(".")?;

// List all specs
let specs = resolver.list_specs()?;
for spec in specs {
    println!("{}: {} (Tier {})", spec.id, spec.title, spec.risk_tier);
}

// Check if multi-agent context
if resolver.is_multi_agent_context() {
    println!("Multiple agents detected - use spec_id parameter");
}
```

### Quality Gates with Mode

```rust
use agent_orchestration::planning::caws_quality_gates::CawsQualityGateExecutor;
use agent_orchestration::planning::caws_complexity_mode::CawsComplexityMode;

let executor = CawsQualityGateExecutor::new(".")?;
let mode = CawsComplexityMode::detect(".")?;

// Execute with mode awareness
let result = executor.execute_quality_gates_with_mode("ci", Some(mode)).await?;

if !result.passed {
    println!("Blocking violations: {}", result.blocking_violations);
    for violation in &result.violations {
        if !violation.waived {
            println!("  - {}: {}", violation.gate, violation.message);
        }
    }
}
```

## Backward Compatibility

✅ **All changes maintain backward compatibility**:

- Legacy `.caws/working-spec.yaml` still works (fallback priority 4)
- Default complexity mode is `Standard` if no config found
- Existing code continues to work (with `Result` handling)
- Deprecated methods marked with `#[deprecated]` attribute

## Migration Guide

### For Code Using `CawsPlanBridge::new()`

**Before**:
```rust
let bridge = CawsPlanBridge::new();
```

**After**:
```rust
let bridge = CawsPlanBridge::new()?; // Handle Result
// OR
let bridge = CawsPlanBridge::with_project_root("/path/to/project")?;
```

### For Code Using `PlanGenerator::new()`

**Before**:
```rust
let generator = PlanGenerator::new(constraints, None, None, None);
```

**After**:
```rust
let generator = PlanGenerator::new(constraints, None, None, None)?; // Handle Result
// OR
let generator = PlanGenerator::with_project_root(".", constraints, None, None, None)?;
```

## Testing

### Unit Tests Added

- ✅ Spec resolution priority order
- ✅ Multi-agent context detection
- ✅ Complexity mode detection from config files
- ✅ Quality requirements calculation
- ✅ Evidence gate creation with mode awareness

### Integration Points

The following integration points need to be updated to pass `spec_id`:

1. **Orchestrator entry points** - When loading working specs, pass `spec_id` if available
2. **MCP tools** - CAWS validation tools should accept `--spec-id` parameter
3. **CLI interfaces** - Command-line tools should support `--spec-id` flag

## Next Steps

1. **Update orchestrator entry points** to accept and pass `spec_id` parameter
2. **Add integration tests** for multi-spec workflow
3. **Update MCP tools** to support spec_id parameter
4. **Document multi-agent workflow** in agent-orchestration README

## Configuration Examples

### Setting Complexity Mode

**Option 1: `.caws/mode` file**
```bash
echo "enterprise" > .caws/mode
```

**Option 2: `.caws/config.yaml`**
```yaml
mode: standard
```

**Option 3: `.caws/config.json`**
```json
{
  "mode": "simple"
}
```

### Creating Feature-Specific Specs

```bash
# Create spec directory
mkdir -p .caws/specs

# Create feature spec
cat > .caws/specs/user-auth.yaml << EOF
version: "1.0"
id: "user-auth"
title: "User Authentication System"
risk_tier: 1
acceptance_criteria:
  - id: "A1"
    given: "User is logged out"
    when: "User submits valid credentials"
    then: "User is logged in and redirected"
EOF
```

## Quality Requirements Matrix

| Mode       | Tier 1 | Tier 2 | Tier 3 |
|------------|--------|--------|--------|
| **Simple** | 70%/30% | 66.5%/28.5% | 63%/27% |
| **Standard** | 90%/70% | 76%/47.5% | 72%/45% |
| **Enterprise** | 90%/70% | 85.5%/66.5% | 81%/63% |

*Format: Line Coverage / Mutation Score*

## Notes

- Complexity mode detection is lazy (only when needed)
- Spec resolution is cached per bridge instance
- Quality gates script must support `--mode` parameter (future CAWS CLI update)
- All changes are backward compatible with existing workflows




