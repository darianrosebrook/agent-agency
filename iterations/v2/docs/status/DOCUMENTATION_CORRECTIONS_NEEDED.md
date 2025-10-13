# Documentation Corrections Checklist

**Date Created**: October 13, 2025  
**Author**: @darianrosebrook  
**Source**: DOCUMENTATION_ACCURACY_AUDIT.md  
**Status**: Pending Corrections

---

## Priority 1: Critical (Immediate - Today)

### 🔴 1. Fix Component Count Badge and Claims

**Files**: `README.md`

**Current** (line 4):

```markdown
[![Components](https://img.shields.io/badge/components-17/17-brightgreen.svg)](./COMPONENT_STATUS_INDEX.md)
```

**Corrected**:

```markdown
[![Components](https://img.shields.io/badge/components-29%20total-blue.svg)](./COMPONENT_STATUS_INDEX.md)
```

**Additional Changes**:

- Line 19: Change "17/17 Components Implemented" → "29 Components (9 Production-Ready, 13 Functional)"
- Line 59: Update table and text to reflect 29 total components
- Line 60: Change "17 core components total" → "29 total components"

**Estimated Time**: 5 minutes  
**Status**: ⏳ Pending

---

### 🔴 2. Fix Production-Ready Component Count

**Files**: `README.md`

**Current** (line 19):

```markdown
- ✅ **5 Production-Ready Components** - Including Workspace State Manager and System Health Monitor
```

**Corrected**:

```markdown
- ✅ **9 Production-Ready Components** - ARBITER-001, 002, 005, 010, 011, 015, 016, 017, INFRA-005
```

**Additional Changes**:

- Line 60: Change "5 production-ready" → "9 production-ready"
- Line 195: Update performance table

**Estimated Time**: 10 minutes  
**Status**: ⏳ Pending

---

### 🔴 3. Fix E2E Test Badge and Status Claims

**Files**: `README.md`

**Current** (line 6):

```markdown
[![E2E Tests](https://img.shields.io/badge/e2e-24/24-brightgreen.svg)](./tests/e2e/)
```

**Corrected**:

```markdown
[![E2E Tests](<https://img.shields.io/badge/e2e-33%20test%20cases%20(database%20setup%20required)-yellow.svg>)](./tests/e2e/)
```

**Additional Changes**:

- Line 511: Change "✅ All Passing | 24/24 tests" → "🟡 33 test cases in 5 test files (database setup required)"
- Line 519: Change "E2E Tests: ✅ All Passing | 24/24 tests" → "E2E Tests: 🟡 33 test cases across 5 files"
- Remove all "24/24 passing" claims throughout document

**Estimated Time**: 10 minutes  
**Status**: ⏳ Pending

---

### 🔴 4. Fix Coverage Badge and Metrics

**Files**: `README.md`

**Current** (line 5):

```markdown
[![Quality Gates](https://img.shields.io/badge/coverage-85%2B-brightgreen.svg)](../../jest.config.js)
```

**Corrected**:

```markdown
[![Quality Gates](https://img.shields.io/badge/coverage-under%20review-lightgrey.svg)](../../jest.config.js)
```

**Additional Changes**:

- Line 194: Add note: "Test Coverage: ~1.6% overall (individual components vary, under investigation)"
- Line 506-510: Replace quality gates section with honest assessment:

  ```markdown
  ### Quality Gates Status

  **Current Status**: Under Review

  - **Overall Coverage**: 1.62% (investigation in progress)
  - **Component-Level Tests**: Individual components have higher coverage (see component STATUS.md files)
  - **Database Configuration**: Required for full test suite execution
  - **E2E Tests**: 33 test cases implemented, database setup required
  ```

**Estimated Time**: 15 minutes  
**Status**: ⏳ Pending

---

### 🔴 5. Add Database Setup Disclaimer

**Files**: `README.md`

**Location**: After line 332 (Quick Start section)

**Add This Section**:

````markdown
### Database Setup Required

**Important**: The test suite requires a configured PostgreSQL database to run successfully.

**Prerequisites**:

- PostgreSQL 14+ installed
- `postgres` role created with appropriate permissions
- Database named `agent_agency_v2_test` created

**Setup Instructions**:

```bash
# Create PostgreSQL role
createuser -s postgres

# Create test database
createdb agent_agency_v2_test

# Run migrations
npm run migrate

# Verify database connection
npm run db:health-check
```
````

**Without database setup**: Tests will fail with `error: role "postgres" does not exist`

See [Database Setup Guide](./docs/database/SETUP.md) for detailed instructions.

````

**Estimated Time**: 10 minutes
**Status**: ⏳ Pending

---

### 🔴 6. Remove "All Tests Passing" Claims

**Files**: `README.md`

**Find and Replace**:
- "✅ All Passing" → "🟡 Database setup required"
- "All tests passing" → "Test infrastructure complete"
- Any reference to "passing" without verification → Qualified claim

**Affected Lines**:
- Line 511: Unit Tests
- Line 519: E2E Tests
- Line 521: Integration Tests

**Estimated Time**: 5 minutes
**Status**: ⏳ Pending

---

## Priority 1 Summary

**Total Estimated Time**: 55 minutes
**Files to Modify**: 1 (README.md)
**Lines to Change**: ~20
**Impact**: Resolves all critical misrepresentations
**Status**: ⏳ All Pending

---

## Priority 2: High (This Week)

### 🟡 7. Investigate Coverage Discrepancy

**Task**: Determine why coverage report shows 1.62% when components claim 80-95%

**Investigation Steps**:
1. Run `npm run test:coverage` and inspect full output
2. Check if coverage is per-component vs project-wide
3. Verify jest.config.js coverage settings
4. Check if tests are actually running or being skipped
5. Review coverage/lcov.info for detailed breakdown

**Action Items**:
- [ ] Run full test suite with coverage
- [ ] Document findings in `COVERAGE_INVESTIGATION.md`
- [ ] If tests not running: Fix test configuration
- [ ] If coverage accurate: Update all claims to match
- [ ] If coverage tool misconfigured: Fix and re-run

**Estimated Time**: 2 hours
**Status**: ⏳ Pending

---

### 🟡 8. Audit All Component STATUS.md Files

**Task**: Verify claims in all 28 component STATUS.md files

**Files**: All `components/*/STATUS.md` files

**Verification Checklist** (per component):
- [ ] Test count claims match actual test files
- [ ] Coverage percentages verified with coverage report
- [ ] Implementation files actually exist
- [ ] "Production-Ready" claim meets CAWS Tier requirements
- [ ] Test execution status (passing/failing) verified

**Sample Components to Audit**:
1. ARBITER-001: Agent Registry Manager (claimed 47/47 tests, 95.8% coverage)
2. ARBITER-002: Task Routing Manager (claimed 58/58 tests, 94.2% coverage)
3. ARBITER-015: CAWS Arbitration Protocol (claimed 184/184 tests, 96.7% coverage)
4. ARBITER-016: Arbiter Reasoning Engine (claimed 266/266 tests, 95.15% coverage)

**Action Items**:
- [ ] Create STATUS.md audit spreadsheet
- [ ] Verify test counts for top 10 components
- [ ] Spot-check coverage claims
- [ ] Update any inaccurate STATUS.md files

**Estimated Time**: 4 hours
**Status**: ⏳ Pending

---

### 🟡 9. Document Vision Realization Calculation

**Task**: Clarify how "82% Vision Realized" was calculated

**Files**:
- `README.md`
- `docs/status/VISION_REALITY_ASSESSMENT.md`

**Investigation**:
1. Review VISION_REALITY_ASSESSMENT.md calculation sections
2. Check if 82% includes:
   - E2E test infrastructure?
   - Weighted by component importance?
   - Functional + Production-Ready components?
3. Compare with COMPONENT_STATUS_INDEX.md metrics:
   - By count: 22/29 functional+ = 76%
   - By production-ready: 9/29 = 31%
   - By implementation stage: varies

**Action Items**:
- [ ] Add "Calculation Methodology" section to VISION_REALITY_ASSESSMENT.md
- [ ] Document formula: `(Production-Ready × 1.0 + Functional × 0.7 + Alpha × 0.4) / Total`
- [ ] Show work: Calculation steps with component values
- [ ] Update README if percentage changes after clarification

**Estimated Time**: 1 hour
**Status**: ⏳ Pending

---

### 🟡 10. Create Database Setup Guide

**Task**: Write comprehensive database setup documentation

**File**: Create `docs/database/SETUP.md`

**Content Outline**:
```markdown
# Database Setup Guide

## Prerequisites
- PostgreSQL 14+
- pgvector extension
- Role and permissions

## Step-by-Step Setup

### 1. Install PostgreSQL
[Platform-specific instructions]

### 2. Create Database Role
```bash
createuser -s postgres
# Or with specific permissions:
createuser --createdb --createrole --login postgres
````

### 3. Create Databases

```bash
# Development
createdb agent_agency_v2_dev

# Test
createdb agent_agency_v2_test

# Production
createdb agent_agency_v2_prod
```

### 4. Install Extensions

```sql
CREATE EXTENSION IF NOT EXISTS pgvector;
```

### 5. Run Migrations

```bash
npm run migrate
```

### 6. Verify Setup

```bash
npm run db:health-check
```

## Troubleshooting

- "role postgres does not exist" → Create role with createuser
- "database does not exist" → Create database with createdb
- "permission denied" → Grant appropriate permissions

## Environment Configuration

```bash
# .env
DB_HOST=localhost
DB_PORT=5432
DB_NAME=agent_agency_v2_test
DB_USER=postgres
DB_PASSWORD=
```

````

**Estimated Time**: 2 hours
**Status**: ⏳ Pending

---

### 🟡 11. Fix Test Database Configuration

**Task**: Resolve database connection failures in tests

**Files**:
- `tests/setup.ts`
- `.env.test` or `.env.example`
- `jest.config.js`

**Issues to Fix**:
1. "role postgres does not exist" error
2. Database health check failures
3. Test suite cannot connect to database

**Action Items**:
- [ ] Review tests/setup.ts database initialization
- [ ] Check if .env.test is configured properly
- [ ] Verify jest.config.js database settings
- [ ] Add fallback to in-memory DB if PostgreSQL unavailable
- [ ] Document required environment variables
- [ ] Update test README with setup instructions

**Estimated Time**: 2 hours
**Status**: ⏳ Pending

---

### 🟡 12. Verify Test Count Claims

**Task**: Count and verify all test-related claims

**Files**: `README.md`, component STATUS.md files

**Claims to Verify**:
1. "13,000+ lines of tests" (README line 511)
2. ARBITER-001: "47/47 tests"
3. ARBITER-002: "58/58 tests"
4. ARBITER-015: "184/184 tests"
5. ARBITER-016: "266/266 tests"
6. ARBITER-017: "21/21 tests"

**Verification Script**:
```bash
# Count test lines
find tests -name "*.test.ts" -exec wc -l {} + | tail -1

# Count test cases by component
for dir in tests/unit/*/; do
  echo "$dir: $(grep -rh '^\s\+it(' "$dir" | wc -l) test cases"
done
````

**Action Items**:

- [ ] Run verification script
- [ ] Compare results with documented claims
- [ ] Update any incorrect counts
- [ ] Add "Last Verified" dates to all test counts

**Estimated Time**: 1 hour  
**Status**: ⏳ Pending

---

## Priority 2 Summary

**Total Estimated Time**: 12 hours  
**Files to Create**: 2 (SETUP.md, COVERAGE_INVESTIGATION.md)  
**Files to Modify**: Multiple (STATUS.md files, README.md)  
**Impact**: Resolves data integrity and infrastructure issues  
**Status**: ⏳ All Pending

---

## Priority 3: Medium (This Month)

### 🟢 13. Standardize Status Terminology

**Task**: Define and apply consistent status terminology across all documentation

**Files**: All STATUS.md files, README.md, INDEX files

**Create Status Glossary**:

```markdown
# Status Terminology Glossary

## Production-Ready

**Definition**: Component meets ALL of the following:

- ✅ 80%+ test coverage (or 90%+ for Tier 1)
- ✅ All tests passing
- ✅ Zero linting errors
- ✅ Complete documentation
- ✅ Security audit passed (for Tier 1)
- ✅ Performance benchmarks met
- ✅ Can be deployed to production without changes

## Functional

**Definition**: Component works for primary use cases but missing:

- Some edge case handling
- Complete test coverage (60-80%)
- Some documentation
- Performance optimization
- Not production-ready without hardening

## Alpha

**Definition**: Core implementation exists but:

- Limited functionality
- Incomplete tests (<60% coverage)
- Known bugs or issues
- Significant work needed
- Not suitable for production use

## Spec Only

**Definition**: Component specified but:

- No implementation code
- May have design documents
- Waiting for implementation

## Not Started

**Definition**: Component identified but:

- No specification
- No implementation
- No design documents
- Planned for future

## In Development

**Definition**: Active implementation:

- Code being written
- Tests being added
- Between Alpha and Functional states
```

**Action Items**:

- [ ] Create STATUS_GLOSSARY.md
- [ ] Review all STATUS.md files
- [ ] Reclassify any components using inconsistent terminology
- [ ] Add glossary link to README

**Estimated Time**: 3 hours  
**Status**: ⏳ Pending

---

### 🟢 14. Add "Last Verified" Dates to STATUS.md Files

**Task**: Track when status claims were last verified

**Template Addition** (for all STATUS.md files):

```markdown
## Verification History

| Aspect           | Last Verified | Result        | Verified By    |
| ---------------- | ------------- | ------------- | -------------- |
| Test Count       | 2025-10-13    | 47/47 passing | Automated      |
| Coverage         | 2025-10-13    | 95.8%         | Jest Report    |
| Implementation   | 2025-10-13    | Complete      | Code Review    |
| Documentation    | 2025-10-13    | Complete      | Manual Review  |
| Production-Ready | 2025-10-13    | Meets Tier 2  | CAWS Validator |

**Next Verification Due**: 2025-11-13 (Monthly)
```

**Action Items**:

- [ ] Add verification table template to all STATUS.md files
- [ ] Populate with current verification dates
- [ ] Create monthly verification reminder
- [ ] Add verification script to CI/CD

**Estimated Time**: 2 hours  
**Status**: ⏳ Pending

---

### 🟢 15. Create Documentation Update Checklist

**Task**: Define process for keeping documentation synchronized with code

**File**: Create `docs/DOCUMENTATION_MAINTENANCE.md`

**Content**:

````markdown
# Documentation Maintenance Checklist

## When to Update Documentation

### After Every PR Merge

- [ ] Update component STATUS.md if component changed
- [ ] Update COMPONENT_STATUS_INDEX.md if status changed
- [ ] Update README.md if public API changed
- [ ] Update coverage metrics if tests added/removed

### Monthly Maintenance

- [ ] Run documentation accuracy audit
- [ ] Verify all test counts
- [ ] Update "Last Verified" dates
- [ ] Review and update vision realization percentage

### Before Release

- [ ] Full documentation review
- [ ] Verify all badges accurate
- [ ] Update changelog
- [ ] Verify all links work

## Automated Checks

### Pre-Commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Check if component files changed
if git diff --cached --name-only | grep -q "^src/"; then
  echo "⚠️ Source files changed. Remember to update:"
  echo "  - Component STATUS.md"
  echo "  - COMPONENT_STATUS_INDEX.md if status changed"
  echo "  - README.md if public API changed"
fi
```
````

### CI/CD Check

- Run documentation validation script
- Compare claimed metrics with actual
- Fail build if critical mismatches found

## Documentation Validation Script

Location: `scripts/validate-docs.sh`

Checks:

- Component count in README matches actual
- Test counts in STATUS.md match test files
- Coverage claims within 5% of actual
- No broken links in documentation
- All required sections present in STATUS.md files

````

**Action Items**:
- [ ] Create DOCUMENTATION_MAINTENANCE.md
- [ ] Write documentation validation script
- [ ] Add pre-commit hook
- [ ] Add CI/CD validation step
- [ ] Train team on process

**Estimated Time**: 4 hours
**Status**: ⏳ Pending

---

### 🟢 16. Implement Automated Documentation Validation

**Task**: Create scripts to automatically verify documentation accuracy

**File**: Create `scripts/validate-docs.js`

**Validation Checks**:
```javascript
/**
 * Documentation Validation Script
 *
 * Checks:
 * 1. Component count in README matches actual directories
 * 2. Test counts in STATUS.md match actual test files
 * 3. Coverage percentages match coverage reports
 * 4. All STATUS.md files have required sections
 * 5. No broken internal links
 * 6. Badge values match actual metrics
 */

async function validateDocs() {
  const results = {
    passed: [],
    failed: [],
    warnings: []
  };

  // 1. Validate component count
  const actualComponents = countComponentDirs();
  const readmeComponentCount = extractComponentCount('README.md');
  if (actualComponents !== readmeComponentCount) {
    results.failed.push({
      check: 'Component Count',
      expected: actualComponents,
      actual: readmeComponentCount,
      location: 'README.md badge'
    });
  }

  // 2. Validate test counts
  for (const component of getComponents()) {
    const claimedTests = extractTestCount(component.statusFile);
    const actualTests = countTestFiles(component.testDir);
    if (claimedTests !== actualTests) {
      results.failed.push({
        check: `${component.name} Test Count`,
        expected: actualTests,
        actual: claimedTests,
        location: component.statusFile
      });
    }
  }

  // 3. Validate coverage
  const coverageReport = parseCoverageReport();
  const readmeCoverage = extractCoverage('README.md');
  if (Math.abs(coverageReport.overall - readmeCoverage) > 5) {
    results.failed.push({
      check: 'Coverage Percentage',
      expected: coverageReport.overall,
      actual: readmeCoverage,
      location: 'README.md'
    });
  }

  // Generate report
  return generateReport(results);
}
````

**Integration**:

```json
// package.json
{
  "scripts": {
    "docs:validate": "node scripts/validate-docs.js",
    "docs:validate:ci": "node scripts/validate-docs.js --strict",
    "precommit": "npm run docs:validate"
  }
}
```

**Action Items**:

- [ ] Write validate-docs.js script
- [ ] Add to package.json scripts
- [ ] Integrate with pre-commit hooks
- [ ] Add to CI/CD pipeline
- [ ] Create documentation validation report format

**Estimated Time**: 8 hours  
**Status**: ⏳ Pending

---

## Priority 3 Summary

**Total Estimated Time**: 17 hours  
**Files to Create**: 3 (STATUS_GLOSSARY.md, DOCUMENTATION_MAINTENANCE.md, validate-docs.js)  
**Files to Modify**: All STATUS.md files  
**Impact**: Long-term documentation maintainability  
**Status**: ⏳ All Pending

---

## Overall Checklist Summary

### By Priority

| Priority                | Tasks        | Est. Time     | Status         |
| ----------------------- | ------------ | ------------- | -------------- |
| Priority 1 (Today)      | 6 tasks      | 55 minutes    | ⏳ Pending     |
| Priority 2 (This Week)  | 6 tasks      | 12 hours      | ⏳ Pending     |
| Priority 3 (This Month) | 4 tasks      | 17 hours      | ⏳ Pending     |
| **Total**               | **16 tasks** | **~30 hours** | **⏳ Pending** |

### By Impact

| Impact   | Tasks | Focus Area                |
| -------- | ----- | ------------------------- |
| Critical | 6     | Correct false claims      |
| High     | 6     | Verify and document truth |
| Medium   | 4     | Long-term sustainability  |

---

## Quick Start: Priority 1 Only (55 Minutes)

For immediate correction of critical issues:

```bash
cd /Users/darianrosebrook/Desktop/Projects/agent-agency/iterations/v2

# 1. Fix badges (5 min)
# Edit README.md lines 3-7

# 2. Update component count (5 min)
# Edit README.md lines 19, 59-60

# 3. Fix production-ready count (10 min)
# Edit README.md lines 19, 60, 195

# 4. Fix E2E test claims (10 min)
# Edit README.md lines 6, 511, 519

# 5. Fix coverage badge (15 min)
# Edit README.md lines 5, 194, 506-510

# 6. Add database disclaimer (10 min)
# Edit README.md after line 332

# Commit changes
git add README.md
git commit -m "docs: correct critical documentation inaccuracies

- Fix component count (17 → 29)
- Update production-ready count (5 → 9)
- Correct E2E test count (24 → 33) and remove passing claim
- Update coverage metrics (85% → under review)
- Add database setup requirements
- Remove unverified 'all tests passing' claims

See DOCUMENTATION_ACCURACY_AUDIT.md for details."
```

---

## Verification After Corrections

After applying corrections, verify with:

```bash
# Check component count
ls -1 components/ | wc -l  # Should be 29

# Check E2E test count
grep -h '^\s\+it(' tests/e2e/*.e2e.test.ts | wc -l  # Should be 33

# Check STATUS.md files
find components -name "STATUS.md" | wc -l  # Should be 28

# Check production-ready components
grep "Production-Ready" COMPONENT_STATUS_INDEX.md | wc -l  # Should show 9 ARBITER/INFRA components
```

---

## Next Steps

1. **Immediate** (Today): Complete Priority 1 tasks (55 minutes)
2. **This Week**: Investigate coverage discrepancy (Priority 2, Task 7)
3. **This Week**: Create database setup guide (Priority 2, Task 10)
4. **This Month**: Implement automated validation (Priority 3, Task 16)

---

**Checklist Created**: October 13, 2025  
**Status**: Ready for Implementation  
**Owner**: @darianrosebrook  
**Next Review**: After Priority 1 completion
