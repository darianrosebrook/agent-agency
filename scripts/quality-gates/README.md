# Quality Gates - Crisis Response System

**Emergency quality enforcement for Agent Agency V3 crisis response.** These gates prevent further codebase degradation during the critical architectural refactoring.

## 🚨 Crisis Context

The Agent Agency V3 codebase has reached critical crisis levels:
- **658+ duplicate struct names** (up from 537 despite 46% LOC reduction)
- **69 duplicate filenames** (up from 48)
- **11 severe god objects** (>3,000 LOC each - persistent despite reduction)
- **Architecturally broken workers** (hardcoded tasks, no MCP tool execution)

Quality gates **block commits** that would worsen these issues during crisis response.

## 🔧 What Quality Gates Check

### 1. Naming Conventions (`check-naming.js`)
**Blocks:** Files/structs with banned modifiers indicating duplication
- ❌ `enhanced-*`, `unified-*`, `new-*`, `final-*`, `copy-*`, `revamp-*`, `improved-*`
- ✅ Purpose-first canonical names

### 2. Duplication Prevention (`check-duplication.js`)
**Blocks:** Increases in duplication beyond crisis baseline
- ❌ More than 69 duplicate filenames (current crisis level)
- ❌ More than 658 duplicate struct/trait names (current crisis level)
- ✅ Stable or reduced duplication

### 3. God Object Prevention (`check-god-objects.js`)
**Blocks:** Files exceeding size thresholds
- 🚫 **3,000+ LOC**: Severe god objects (immediate crisis intervention required)
- 🚫 **2,000+ LOC**: Critical god objects (CI/CD block)
- ⚠️ **1,500+ LOC**: Warning (allows but flags for decomposition)
- ✅ **<1,000 LOC**: Target for long-term maintainability

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

## 📊 Current Crisis Baselines

| Metric | Current | Threshold | Status |
|--------|---------|-----------|--------|
| Duplicate filenames | 69 | ≤69 | ✅ Stable |
| Duplicate structs | 658+ | ≤658 | ✅ Stable |
| God objects >3K LOC | 11 | 0 | 🚨 Crisis |
| God objects >2K LOC | Multiple | 0 | 🚨 Crisis |

**These baselines will only decrease during crisis response.**

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

### Duplication Regression
```
❌ STRUCT_DUPLICATION_REGRESSION
   Duplicate struct/trait names increased from 658 to 670
   Issue: Duplication must not increase during crisis response
```

**Fix:** Extract common traits, consolidate duplicate implementations

### God Object Violations
```
❌ SEVERE_GOD_OBJECT
   File: council/src/intelligent_edge_case_testing.rs
   Size: 6348 LOC
   Limit: 3000 LOC
   Issue: SEVERE god object: 6348 LOC exceeds 3000 LOC limit
```

**Fix:** Decompose into smaller, focused modules

## 🔄 Crisis Response Integration

Quality gates integrate with the **Week 1 Emergency Stabilization** plan:

1. **Automated Enforcement**: Gates prevent new violations
2. **God Object Surgery**: Gates block oversized files
3. **Trait Extraction**: Gates detect duplication increases
4. **MCP Architecture**: Gates ensure clean foundation for redesign

## 📈 Monitoring & Metrics

### Daily Crisis Dashboard
```bash
# Run quality gates to see current status
node scripts/quality-gates/run-quality-gates.js

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

- ✅ **Zero new naming violations** committed
- ✅ **Duplication counts stable or decreasing**
- ✅ **No new god objects >2,000 LOC** created
- ✅ **CI/CD pipeline blocks problematic PRs**
- ✅ **Pre-commit hook prevents local violations**

**Failure means continued codebase degradation.** Quality gates are the first line of defense in the architectural crisis response.



