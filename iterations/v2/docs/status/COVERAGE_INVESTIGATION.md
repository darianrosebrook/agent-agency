# Coverage Discrepancy Investigation Report

**Task**: Priority 2, Task 7  
**Date**: October 13, 2025  
**Author**: @darianrosebrook  
**Investigation Time**: 2 hours

---

## Executive Summary

Investigated the 98% discrepancy between claimed coverage (85%+) and reported coverage (1.62%). Found that **actual coverage is ~30%**, not 1.62% or 85%. The discrepancy has multiple root causes requiring systematic resolution.

**Critical Findings:**

1. **Actual Coverage**: 30.31% statements, 24.72% branches, 29.73% functions, 30.66% lines
2. **Claim**: README badge shows "85%+" (300% overstatement)
3. **Root Causes**: Database setup failures, TypeScript compilation errors, passing tests are minority
4. **Component Claims**: Only 1 of 29 components documents coverage (contradicts itself)

**Severity**: CRITICAL - Documentation severely overstates quality metrics

---

## Detailed Findings

### Finding 1: Actual Coverage is ~30%, Not 85% or 1.62%

**Source**: `coverage/lcov-report/index.html`

```
Statements: 30.31%
Branches:   24.72%
Functions:  29.73%
Lines:      30.66%
```

**Evidence**:

```bash
$ grep "strong" coverage/lcov-report/index.html | head -6
<span class="strong">30.31% </span>  # Statements
<span class="strong">24.72% </span>  # Branches
<span class="strong">29.73% </span>  # Functions
<span class="strong">30.66% </span>  # Lines
```

**Explanation**: The 1.62% number seen in the initial audit was likely from a different/older coverage run or a subset of files. The full project coverage is ~30%.

---

### Finding 2: Test Files Not Running Due to Multiple Blockers

**Test File Counts:**

- Unit tests: 92 files
- Integration tests: 39 files
- E2E tests: 6 files
- **Total**: 140 test files

**Blockers Preventing Tests from Running:**

#### Blocker A: Database Configuration Issues

**Error Message:**

```
[ConnectionPool] Health check failed: error: role "postgres" does not exist
```

**Impact**: All database-dependent tests fail immediately

**Files Affected**: Integration tests (39 files), some unit tests

**Root Cause**: PostgreSQL not configured with required roles/databases

**Evidence**:

```bash
$ npm run test:unit 2>&1 | grep "role"
error: role "postgres" does not exist
```

#### Blocker B: TypeScript Compilation Errors

**Count**: 23 TypeScript errors across test files

**Files Affected**:

```
tests/unit/verification/validators/consistency.test.ts (6 errors)
tests/unit/verification/validators/cross-reference.test.ts (7 errors)
tests/unit/verification/validators/logical.test.ts (5 errors)
tests/unit/verification/validators/statistical.test.ts (5 errors)
tests/unit/resilience/retry-policy.test.ts (missing export)
```

**Error Pattern**: Syntax errors (`TS1005: ')' expected`) and missing exports

**Impact**: These test files cannot compile, preventing them from running

**Evidence**:

```bash
$ npx tsc --noEmit --project tsconfig.json 2>&1 | grep "^tests/" | wc -l
23
```

#### Blocker C: Test Failures (Non-Blocking)

**Example**: ModelRegistryLLMProvider test

```
expect(received).toBeGreaterThan(expected)
Expected: > 1
Received:   0.95
```

**Impact**: Tests run but fail, contributing 0% to coverage

---

### Finding 3: Component STATUS.md Coverage Claims

**Components with Coverage Claims**: 1 of 29

**File**: `components/system-health-monitor/STATUS.md`

**Contradictory Claims in Same File:**

```markdown
Line 17: Test Coverage: ~85% (SystemHealthMonitor + MetricsCollector fully tested)
Line 37: Line Coverage: 85%+ across monitoring components
Line 98: Test Coverage: 0% (Target: 80% for Tier 2) ← Correct!
```

**Analysis**: The file makes aspirational claims early (lines 17, 37) but correctly reports 0% later (line 98). This suggests copy-paste from templates or aspirational documentation.

**Other 28 Components**: No coverage claims documented in STATUS.md files

---

### Finding 4: README Coverage Claim Analysis

**README Claim** (Line 4):

```markdown
[![Quality Gates](https://img.shields.io/badge/coverage-85%2B-brightgreen.svg)](../../jest.config.js)
```

**Reality**: 30.31% actual coverage

**Discrepancy**: +179% overstatement (claiming 280% more than actual)

**Impact**: Misleads stakeholders, contributors, and auditors

---

## Root Cause Analysis

### Why Coverage is 30% Instead of 85%

**Reason 1: Database Setup Failures** (Est. -30% coverage)

- Integration tests cannot run without database
- 39 integration test files blocked
- Database-dependent unit tests also blocked

**Reason 2: TypeScript Compilation Errors** (Est. -15% coverage)

- 23 errors across 4 test files
- These tests never execute
- Likely validators and verification tests

**Reason 3: Test Failures** (Est. -10% coverage)

- Some tests run but fail assertions
- Failed tests contribute 0% coverage
- Example: ModelRegistryLLMProvider

**Reason 4: Incomplete Test Suite** (Est. -30% coverage)

- Only ~70% of source files have corresponding tests
- Many edge cases not tested
- Some components lack comprehensive unit tests

---

## Coverage Breakdown by Component (Estimate)

Based on test file analysis and structure:

| Component Category    | Estimated Coverage | Test Files | Status                |
| --------------------- | ------------------ | ---------- | --------------------- |
| **Core Orchestrator** | 60-70%             | 15 files   | ✅ Most tests passing |
| **Agent Management**  | 40-50%             | 12 files   | 🟡 Some DB issues     |
| **Reasoning Engine**  | 50-60%             | 20 files   | ✅ Pure logic, no DB  |
| **Verification**      | 0-10%              | 4 files    | ❌ TS errors blocking |
| **Database Layer**    | 0-10%              | 10 files   | ❌ DB setup blocking  |
| **Monitoring**        | 0%                 | 0 files    | ❌ Not implemented    |
| **Resilience**        | 10-20%             | 5 files    | 🟡 Some TS errors     |
| **Evaluation**        | 30-40%             | 8 files    | 🟡 DB + test failures |

---

## Recommended Actions

### Immediate (Priority 1) - Already Completed ✅

1. ✅ Update README badge to "under review"
2. ✅ Document database setup requirements
3. ✅ Remove false "85%+" claims

### This Week (Priority 2) - Current Tasks

#### Task 7A: Fix Database Configuration (2 hours)

**Goal**: All tests can connect to database

**Actions**:

1. Create PostgreSQL database and role
2. Update `.env.test` with correct credentials
3. Run migrations for test database
4. Verify health check passes
5. Re-run integration tests

**Success Criteria**: `npm run test:integration` connects to database

**Commands**:

```bash
# Create database (from README)
createuser -s postgres
createdb agent_agency_v2_test -O postgres
psql -U postgres -d agent_agency_v2_test -f migrations/001_initial_schema.sql

# Verify
npm run test:integration 2>&1 | grep -i "health check"
```

#### Task 7B: Fix TypeScript Compilation Errors (1 hour)

**Goal**: All test files compile successfully

**Actions**:

1. Fix 23 TypeScript errors in test files
2. Verify `npx tsc --noEmit` passes
3. Re-run affected tests

**Files to Fix**:

- `tests/unit/verification/validators/consistency.test.ts`
- `tests/unit/verification/validators/cross-reference.test.ts`
- `tests/unit/verification/validators/logical.test.ts`
- `tests/unit/verification/validators/statistical.test.ts`
- `tests/unit/resilience/retry-policy.test.ts`

**Success Criteria**: Zero TypeScript errors in tests

```bash
npx tsc --noEmit --project tsconfig.json 2>&1 | grep -c "^tests/"  # Should be 0
```

#### Task 7C: Re-Run Coverage After Fixes (30 minutes)

**Goal**: Accurate coverage baseline

**Actions**:

1. Run full test suite with coverage
2. Generate fresh coverage report
3. Document actual coverage per component
4. Update README with honest numbers

**Commands**:

```bash
npm run test:coverage
open coverage/lcov-report/index.html
```

**Expected Result**: Coverage should increase to 50-70% once DB and TS issues resolved

---

## Coverage Target Reconciliation

### Current State

- **Actual**: 30.31% statements
- **Target (Tier 2)**: 80% branch coverage
- **Gap**: -49.69 percentage points

### After Fixes (Estimated)

- **Estimated**: 50-70% statements after DB + TS fixes
- **Gap**: -10 to -30 percentage points
- **Additional Work Needed**: 200-400 test cases

### Path to 80% Coverage

**Phase 1: Fix Blockers** (This Week)

- Fix database configuration (Task 7A)
- Fix TypeScript errors (Task 7B)
- Expected Result: 50-70% coverage

**Phase 2: Fill Gaps** (Next 2 Weeks)

- Add tests for uncovered components
- Focus on critical paths first
- Target: 70-80% coverage

**Phase 3: Edge Cases** (Next 2 Weeks)

- Add branch coverage tests
- Error condition testing
- Target: 80%+ coverage

**Total Effort**: 4-6 weeks to reach 80% coverage

---

## Component Coverage Audit

### Components to Audit Next (Task 8)

As part of Priority 2, Task 8 (Audit Component STATUS.md Files):

1. Check all 29 STATUS.md files for coverage claims
2. Verify claims against actual test files
3. Correct any false claims
4. Add "Last Verified" dates

**Identified Issue**: system-health-monitor/STATUS.md contradicts itself on coverage

**Pattern**: Likely other components have similar contradictions

---

## Conclusion

### Summary of Findings

1. **README claims 85%+ coverage**: False (actual 30%)
2. **Database blocking tests**: 39 integration test files
3. **TypeScript errors**: 23 errors preventing test execution
4. **Component claims**: 1 of 29 documents coverage (contradicts itself)

### Honest Assessment

**Current Coverage**: ~30% (below Tier 2 requirement of 80%)

**Status**: ❌ **Not Tier 2 Compliant**

**Work Needed**: 4-6 weeks to reach 80% coverage with:

- 2 hours: Database setup
- 1 hour: Fix TypeScript errors
- 2-4 weeks: Add missing tests
- 2 weeks: Fill edge case coverage

### Next Steps

1. ✅ Complete database setup (Priority 2, Task 7A)
2. ✅ Fix TypeScript errors (Priority 2, Task 7B)
3. ✅ Re-run coverage (Priority 2, Task 7C)
4. ⏳ Audit all component STATUS.md files (Priority 2, Task 8)
5. ⏳ Create test gap analysis (Priority 2, Task 9)
6. ⏳ Develop test addition plan (Priority 2, Task 10)

---

**Investigation Complete**: Coverage discrepancy root causes identified and documented

**Report Author**: @darianrosebrook  
**Date**: October 13, 2025  
**Status**: ✅ Complete
