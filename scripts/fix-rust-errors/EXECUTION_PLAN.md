# Rust Error Fix Automation Plan

## Executive Summary

**Total Errors**: 969  
**Automatable**: ~328 errors (34%) - Can be fixed programmatically  
**Manual Required**: ~641 errors (66%) - Need human decision-making

**Time Estimate**:
- **Fully Automated**: 328 errors × 5 seconds = **27 minutes**
- **Manual Review**: 641 errors × 1 minute = **641 minutes**
- **Total**: ~11 hours (vs 16+ hours manual)

## Automation Breakdown

### ✅ Quick Wins (86 errors - 30 minutes)

#### 1. Type Conversions (32 errors)
- `String` ↔ `Uuid`: 15 errors
- `f32` ↔ `f64`: 8 errors  
- `usize` ↔ `u32`: 9 errors

**Script**: `scripts/fix-rust-errors/01-fix-type-conversions.sh`

#### 2. Trait Derives (11 errors)
- Missing `Display` trait: 7 errors
- Missing `JsonSchema`: 4 errors

**Action**: Add `#[derive(Display)]` or implement manually

#### 3. Struct Initialization (43 errors)
- Missing required fields in struct literals
- Can add defaults: `field: Default::default()` or `field: None`

**Action**: Find struct definitions, add missing fields with defaults

### ⚠️ High-Impact Patterns (240+ errors - 2-3 hours)

#### 4. Missing Struct Fields (480 errors total)

**Top Patterns** (Top 10 = 55 errors):
1. `TaskScope`: missing `files`, `directories`, `patterns` (24 errors)
2. `HistoricalClaim`: missing `content`, `source`, `confidence` (15 errors)
3. `SearchResultFeature`: missing `score`, `metadata` (8 errors)
4. `Ambiguity`: missing `position`, `original_text`, `possible_resolutions` (12 errors)
5. `Entity`: missing `text`, `position`, `metadata` (12 errors)

**Strategy**: 
1. Find struct definitions using `rust-analyzer` or `grep`
2. Check if fields exist but were renamed/moved
3. Either:
   - Add fields to struct definitions (if they should exist)
   - Update all access sites to use new field names (if renamed)

**Automation Potential**: High for top 10 patterns (55 errors), Medium for rest

### ⚠️ Method Signature Changes (151 errors - 1-2 hours)

**Patterns**:
- `PgRow::get()` method signature changed
- `Uuid::parse()` vs `Uuid::from_str()`
- Enum methods like `.as_str()` missing

**Strategy**: Update method calls to match new API

## Recommended Execution Plan

### Phase 1: Automated Fixes (30 min)
```bash
# 1. Fix type conversions
bash scripts/fix-rust-errors/01-fix-type-conversions.sh

# 2. Add trait derives (manual - but script can identify)
# Find files needing Display/JsonSchema and add derives

# 3. Fix struct initialization
# For each struct init error, add missing fields with defaults
```

**Expected Result**: ~86 errors fixed

### Phase 2: Structural Fixes (2-3 hours)

#### 2a. Find Struct Definitions
```bash
# Find where TaskScope is defined
find iterations/v3 -name "*.rs" -exec grep -l "struct TaskScope" {} \;

# Find where HistoricalClaim is defined  
find iterations/v3 -name "*.rs" -exec grep -l "struct HistoricalClaim" {} \;
```

#### 2b. Analyze Struct Definitions
- Check if fields exist but with different names
- Check if fields were moved to different structs
- Check if fields were removed intentionally

#### 2c. Batch Fixes
- If fields should exist: Add to struct definitions
- If fields renamed: Update all access sites (use find/replace)
- If fields removed: Comment out or remove accesses

**Expected Result**: ~200-300 errors fixed

### Phase 3: Method Updates (1-2 hours)
- Update `PgRow::get()` calls to new signature
- Fix `Uuid` parsing methods
- Add missing enum methods

**Expected Result**: ~150 errors fixed

### Phase 4: Manual Review (4-6 hours)
- Complex borrow checker errors
- Trait bound issues requiring design decisions
- Integration points
- Edge cases

## Tools & Commands

### Find Struct Definitions
```bash
# Find all struct definitions
find iterations/v3 -name "*.rs" -exec grep -H "^pub struct\|^struct" {} \; | grep -i "struct_name"

# Find all usages of a struct
rg "struct_name" iterations/v3 --type rust
```

### Find Missing Field Errors
```bash
# Count errors by struct
grep "has no field named" cargo-check-current-errors.log | \
  sed 's/.*struct `\([^`]*\)` has no field named `\([^`]*\)`.*/\1::\2/' | \
  sort | uniq -c | sort -nr
```

### Batch Find/Replace
```bash
# Example: Fix TaskScope field access
# First check what fields actually exist
grep -A 20 "struct TaskScope" iterations/v3/**/*.rs

# Then update accesses (be careful!)
find iterations/v3 -name "*.rs" -exec sed -i '' 's/scope\.files/scope.paths/' {} \;
```

## Risk Assessment

**Low Risk** (Safe to automate):
- Type conversions
- Trait derives
- Struct initialization with defaults

**Medium Risk** (Review after automation):
- Adding fields to structs (need to verify field types)
- Updating method calls (need to verify new signatures)

**High Risk** (Manual review required):
- Removing field accesses (need to understand impact)
- Complex borrow checker errors
- Design decisions about struct changes

## Success Metrics

- **Phase 1 Complete**: < 883 errors remaining
- **Phase 2 Complete**: < 600 errors remaining  
- **Phase 3 Complete**: < 450 errors remaining
- **Phase 4 Complete**: 0 errors

## Next Steps

1. ✅ Analyze error patterns (DONE)
2. 🔄 Create automation scripts for Phase 1
3. ⏳ Run Phase 1 fixes
4. ⏳ Analyze struct definitions for Phase 2
5. ⏳ Execute Phase 2-4 fixes

