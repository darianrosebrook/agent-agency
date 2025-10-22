# Worker 2 - Ready to Begin

## Handoff from Worker 1: ✅ COMPLETE

Worker 1 has successfully removed all 39 duplicate type definitions from `council/src/learning.rs`.

### Current Status
- **Compilation Errors**: 200 (down from 525)
- **Conflicting Trait Implementations**: 0 ✅
- **File Size**: 2,259 lines (down from 4,676)

## Your Mission: Fix Missing Enum Variants & Define Missing Types

### Priority 1: Missing Enum Variants (20+ errors)

You need to add missing variants to existing enums. Search for these error patterns:

```bash
cargo check --package agent-agency-council 2>&1 | grep "failed to resolve: use of undeclared type"
```

**Task 1a: Add TaskComplexity Variants**

Find where `TaskComplexity` is defined (around line 1400-1450) and add these variants if missing:
```rust
pub enum TaskComplexity {
    Low,
    Medium, 
    High,
    Critical,
}
```

**Task 1b: Add RiskTier Variants**

Find where `RiskTier` is defined and add these variants if missing:
```rust
pub enum RiskTier {
    Tier1,
    Tier2,
    Tier3,
}
```

### Priority 2: Define Missing Types (20+ errors)

These types are referenced but never defined. Add them to the types module:

**Task 2a: Define TaskType**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskType {
    pub name: String,
    pub category: String,
}
```

**Task 2b: Define SpecializationScore**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecializationScore {
    pub domain: String,
    pub score: f32,
}
```

**Task 2c: Define ResourceTrend**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceTrend {
    pub trend_type: TrendType,
    pub slope: f32,
    pub confidence: f32,
    pub time_window: i64,
}
```

**Task 2d: Define TrendType**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendType {
    Increasing,
    Decreasing,
    Stable,
}
```

**Task 2e: Define TrendAnalysis**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    pub trend_type: TrendType,
    pub slope: f32,
    pub confidence: f32,
    pub time_window: i64,
}
```

## Success Criteria

- [ ] No "failed to resolve: use of undeclared type" errors
- [ ] No "cannot find struct, variant or union type" errors for the missing types
- [ ] `cargo check --package agent-agency-council` compiles with <= 150 errors
- [ ] All new types have proper derive attributes (Debug, Clone, Serialize, Deserialize)

## Quick Commands

```bash
# Check current error count
cargo check --package agent-agency-council 2>&1 | grep -c "error\["

# Check for your specific error types
cargo check --package agent-agency-council 2>&1 | grep "failed to resolve: use of undeclared type"
cargo check --package agent-agency-council 2>&1 | grep "cannot find struct, variant or union type"
```

## Files to Modify

- Primary: `council/src/learning.rs`
- Secondary: `council/src/types.rs` (or wherever types are defined)

## Estimated Time
- 1-2 hours to identify all locations and add definitions
- Typical completion: ~50 errors fixed, bringing total to ~150

## Ready?

When you're ready to start, run:
```bash
cd /Users/darianrosebrook/Desktop/Projects/agent-agency/iterations/v3
cargo check --package agent-agency-council 2>&1 | head -20
```

Good luck, Worker 2! 🚀
