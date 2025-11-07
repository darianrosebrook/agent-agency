# Waiver Recognition Differences: CAWS vs Agent-Agency

## Summary

The agent-agency quality gates implementation is **missing all waiver recognition functionality** that exists in the CAWS version. This means waivers created via CAWS are not being recognized during commit hooks and quality gate runs.

## Key Missing Components

### 1. Missing Waiver Loading in `run-quality-gates.mjs`

**CAWS Version Has:**
- `activeWaivers` property initialized in constructor (line 207)
- `loadActiveWaivers()` method (lines 516-598)
- Waiver loading called in constructor (lines 264-275)
- `isViolationWaived()` method (lines 611-624)
- Waiver checking in `reportResults()` (lines 1583-1626)

**Agent-Agency Version Missing:**
- No `activeWaivers` property
- No `loadActiveWaivers()` method
- No `isViolationWaived()` method
- No waiver checking in `reportResults()`

### 2. Missing Waiver Support in `shared-exception-framework.mjs`

**CAWS Version Has:**
- `loadCawsWaivers()` function (lines 189-338)
- Waiver loading from `.caws/waivers/active-waivers.yaml`
- Waiver loading from individual waiver YAML files
- Waiver-to-exception conversion logic
- Gate name mapping (`documentation_quality` → `documentation`, `hidden_todo` → `hidden-todo`)
- Date format normalization for waivers
- Merging waivers into exceptions in `loadExceptionConfig()` (lines 401-411)
- Special handling for waiver exceptions in `addHit()` (lines 687-708)

**Agent-Agency Version Missing:**
- No `loadCawsWaivers()` function
- No YAML import (`yaml` from `js-yaml`)
- No waiver directory constants (`WAIVERS_DIR`, `ACTIVE_WAIVERS_PATH`)
- No waiver-to-exception conversion
- No waiver merging in `loadExceptionConfig()`

### 3. Missing Waiver Reporting in `reportResults()`

**CAWS Version Has:**
```javascript
// Check waivers for each violation
for (const violation of this.violations) {
  this.isViolationWaived(violation);
}

// Separate waived and blocking violations
const waivedViolations = this.violations.filter((v) => v.waivedBy);
const blockingViolations = this.violations.filter((v) => !v.waivedBy);

// Report active waivers
if (this.activeWaivers.length > 0 && !QUIET_MODE && !JSON_MODE) {
  console.log(`\n🔖 ACTIVE WAIVERS (${this.activeWaivers.length}):`);
  // ... waiver reporting ...
}

// Report waived violations
if (waivedViolations.length > 0 && !QUIET_MODE && !JSON_MODE) {
  console.log(`\n✅ WAIVED VIOLATIONS (${waivedViolations.length}) - ALLOWED:`);
  // ... waived violation reporting ...
}

// Only block commit if there are non-waived violations
const nonWaivedViolations = this.violations.filter((v) => !v.waivedBy);
process.exit(nonWaivedViolations.length ? 1 : 0);
```

**Agent-Agency Version Has:**
```javascript
// No waiver checking at all
// Just reports all violations as blocking
process.exit(this.violations.length ? 1 : 0);
```

### 4. Missing Waiver Metadata in Report Payload

**CAWS Version Includes:**
```javascript
waivers: {
  active: this.activeWaivers.length,
  applied: waivedViolations.length,
  details: this.activeWaivers.map((w) => ({
    id: w.id,
    title: w.title,
    gates: w.gates,
    expires_at: w.expires_at,
  })),
}
```

**Agent-Agency Version Missing:**
- No `waivers` object in report payload

## Required Changes

To fix waiver recognition in agent-agency, you need to:

1. **Add waiver loading to `run-quality-gates.mjs`:**
   - Add `activeWaivers` property to constructor
   - Add `loadActiveWaivers()` method
   - Add `isViolationWaived()` method
   - Call `loadActiveWaivers()` in constructor
   - Add waiver checking in `reportResults()`

2. **Add waiver support to `shared-exception-framework.mjs`:**
   - Import `yaml` from `js-yaml`
   - Add `WAIVERS_DIR` and `ACTIVE_WAIVERS_PATH` constants
   - Add `loadCawsWaivers()` function
   - Add `normalizeExceptionDates()` function
   - Merge waivers in `loadExceptionConfig()`
   - Update `addHit()` to skip persistence for waiver exceptions

3. **Update `reportResults()` to:**
   - Check waivers for each violation
   - Separate waived vs blocking violations
   - Report active waivers
   - Report waived violations
   - Only block commits for non-waived violations
   - Include waiver metadata in report payload

## Files to Update

1. `/Users/darianrosebrook/Desktop/Projects/agent-agency/scripts/quality-gates/run-quality-gates.mjs`
2. `/Users/darianrosebrook/Desktop/Projects/agent-agency/scripts/quality-gates/shared-exception-framework.mjs`

## Verification

After applying changes, verify:
- Waivers are loaded from `.caws/waivers/active-waivers.yaml`
- Waivers are converted to exceptions correctly
- Violations matching waivers are marked as waived
- Waived violations don't block commits
- Waiver information is reported in console output
- Waiver metadata is included in JSON report

