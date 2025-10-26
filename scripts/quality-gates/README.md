# Quality Gates - Functional Duplication Prevention System

**Focus on preventing functional duplication and maintaining code quality.** These gates prioritize business logic consolidation over organizational naming conventions.

## 🎯 Priority Focus

The Agent Agency V3 codebase has critical functional duplication issues:
- **692+ duplicate struct names** (CRITICAL - causes compilation conflicts and maintenance issues)
- **200+ duplicate function names** (NEW FOCUS - indicates business logic duplication)
- **100+ duplicate trait names** (NEW FOCUS - indicates interface duplication)
- **11 severe god objects** (>3,000 LOC each - architectural debt)

Quality gates **block commits** that increase functional duplication while allowing Rust naming conventions.

## 🔧 What Quality Gates Check

### 1. Functional Duplication Prevention (`check-duplication.js`)
**Blocks:** Increases in functional duplication beyond thresholds
- ❌ More than 692 duplicate struct names (CRITICAL - compilation conflicts)
- ❌ More than 200 duplicate function names (NEW - business logic duplication)
- ❌ More than 100 duplicate trait names (NEW - interface duplication)
- ❌ More than 20 problematic duplicate filenames (excluding Rust conventions)
- ✅ Rust convention files (lib.rs, mod.rs) are expected and allowed

### 2. Naming Conventions (`check-naming.js`)
**Blocks:** Files/structs with banned modifiers indicating functional duplication
- ❌ `enhanced-*`, `unified-*`, `new-*`, `final-*`, `copy-*`, `revamp-*`, `improved-*`
- ✅ Purpose-first canonical names
- ✅ Rust convention files (lib.rs, mod.rs) are ignored

### 3. God Object Prevention (`check-god-objects.js`)
**Blocks:** Files exceeding size thresholds
- 🚫 **3,000+ LOC**: Severe god objects (immediate intervention required)
- 🚫 **2,000+ LOC**: Critical god objects (CI/CD block)
- ⚠️ **1,500+ LOC**: Warning (allows but flags for decomposition)
- ✅ **<1,500 LOC**: Target for long-term maintainability

## 🚦 How It Works

### Pre-commit Hook (Local Development)
```bash
# Automatic - runs before every commit
✅ Quality gates passed - proceeding with commit
# OR
❌ Quality gates failed - commit blocked
💡 Fix the violations above before committing
```

### CI/CD Pipeline (`.github/workflows/v3-ci.yml`)
- **Job**: `Quality Gates (Crisis Response)`
- **Runs**: Before tests, after linting
- **Blocks**: PR merges if quality violations detected
- **Reports**: Detailed violation breakdown in CI logs

## 📊 Current Functional Duplication Baselines

| Metric | Current | Threshold | Status | Priority |
|--------|---------|-----------|--------|----------|
| Duplicate struct names | 692+ | ≤692 | 🚨 CRITICAL | HIGH |
| Duplicate function names | ~200 | ≤200 | 🚨 CRITICAL | HIGH |
| Duplicate trait names | ~100 | ≤100 | 🚨 CRITICAL | HIGH |
| Problematic filename duplicates | ~20 | ≤20 | ⚠️ MODERATE | MEDIUM |
| Rust convention duplicates (lib.rs, mod.rs) | ~128 | N/A | ✅ EXPECTED | NONE |
| God objects >3K LOC | 11 | 0 | 🚨 CRITICAL | HIGH |
| God objects >2K LOC | Multiple | 0 | 🚨 CRITICAL | HIGH |

**Focus: Functional duplication must decrease, Rust conventions are expected.**

## 🛠️ Usage

### Run All Gates Locally
```bash
# Interactive mode (development)
node scripts/quality-gates/run-quality-gates.js

# CI mode (strict, for automation)
node scripts/quality-gates/run-quality-gates.js --ci

# With auto-fix attempts (experimental)
node scripts/quality-gates/run-quality-gates.js --fix
```

### Run Individual Gates
```bash
# Check naming only
node scripts/quality-gates/check-naming.js

# Check duplication only
node scripts/quality-gates/check-duplication.js

# Check god objects only
node scripts/quality-gates/check-god-objects.js
```

### Setup Pre-commit Hook
```bash
# Install quality gates as pre-commit hook
./scripts/setup-pre-commit-hook.sh

# Verify hook is active
cat .git/hooks/pre-commit
```

## 🚫 When Gates Block Commits

### Naming Violations
```
❌ FILENAME_BANNED_MODIFIER: iterations/v3/src/enhanced_parser.rs
   Filename contains banned modifier: enhanced
   Rule: No duplicate "enhanced/unified/new/final" modules
```

**Fix:** Rename to purpose-first canonical name (e.g., `parser.rs`)

### Functional Duplication Regression
```
❌ STRUCT_DUPLICATION_REGRESSION
   Duplicate struct names increased from 692 to 700
   Issue: Functional duplication must not increase

❌ FUNCTION_DUPLICATION_REGRESSION  
   Duplicate function names increased from 200 to 210
   Issue: Business logic duplication detected

❌ TRAIT_DUPLICATION_REGRESSION
   Duplicate trait names increased from 100 to 105
   Issue: Interface duplication detected
```

**Fix:** Extract common traits, consolidate duplicate business logic, unify interfaces

### God Object Violations
```
❌ SEVERE_GOD_OBJECT
   File: council/src/intelligent_edge_case_testing.rs
   Size: 6348 LOC
   Limit: 3000 LOC
   Issue: SEVERE god object: 6348 LOC exceeds 3000 LOC limit
```

**Fix:** Decompose into smaller, focused modules

## 🔄 Functional Duplication Response Integration

Quality gates integrate with the **Functional Duplication Prevention** plan:

1. **Automated Enforcement**: Gates prevent new functional duplication
2. **Business Logic Consolidation**: Gates detect duplicate functions and traits
3. **Interface Unification**: Gates prevent duplicate trait definitions
4. **Structural Cleanup**: Gates allow Rust conventions while blocking problematic patterns

## 📈 Monitoring & Metrics

### Daily Functional Duplication Dashboard
```bash
# Run quality gates to see current functional duplication status
node scripts/quality-gates/run-quality-gates.js

# Check functional duplication specifically
node scripts/quality-gates/check-duplication.js

# Check god object sizes specifically
node scripts/quality-gates/check-god-objects.js
```

### CI/CD Integration
- **Quality Gates job** runs on every PR
- **Failure blocks merge** until violations fixed
- **Reports link to** `docs/refactoring.md` crisis plan

## 🔧 Emergency Overrides

### Temporary Bypass (Not Recommended)
```bash
# Skip pre-commit hook (only for emergencies)
git commit --no-verify

# Skip CI quality gates (requires maintainer approval)
# Edit workflow to temporarily disable quality_gates job
```

**⚠️ Bypasses should only be used for critical hotfixes during crisis response.**

## 📚 Related Documentation

- **Crisis Response Plan**: `docs/refactoring.md`
- **Naming Violations**: `docs/audits/v3-codebase-audit-2025-10/06-naming-violations.md`
- **Duplication Report**: `docs/audits/v3-codebase-audit-2025-10/02-duplication-report.md`
- **God Objects Analysis**: `docs/audits/v3-codebase-audit-2025-10/03-god-objects-analysis.md`

---

## 🎯 Success Criteria

Quality gates are successful when:

- ✅ **Zero new functional duplication** committed (structs, functions, traits)
- ✅ **Functional duplication counts stable or decreasing**
- ✅ **Rust conventions (lib.rs, mod.rs) are allowed and expected**
- ✅ **No new god objects >2,000 LOC** created
- ✅ **CI/CD pipeline blocks functional duplication increases**
- ✅ **Pre-commit hook prevents local functional duplication**

**Focus: Functional duplication is the real enemy, not organizational naming conventions.** Quality gates prioritize business logic consolidation over file naming patterns.



