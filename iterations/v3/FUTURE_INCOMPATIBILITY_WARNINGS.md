# Future Incompatibility Warnings - Upgrade Plan

**Status**: Documented for future resolution
**Date**: 2025-01-XX

## Overview

The following dependencies generate "future incompatibility" warnings because they use APIs that will be removed in future Rust versions. These warnings indicate that the crates need to be updated to newer major versions that support current Rust APIs.

## Affected Dependencies

### 1. `pdf v0.8.1`
- **Location**: `agent-data-processing/Cargo.toml`
- **Issue**: Uses deprecated Rust APIs
- **Current Status**: At latest compatible version (0.8.x)
- **Upgrade Path**: Update to `pdf v1.x` when available
- **Impact**: Low - only used for PDF processing in data enrichment
- **Timeline**: Q1 2026 (when v1.x is released and tested)

### 2. `redis v0.24.0`
- **Location**: `data-infrastructure/Cargo.toml`, `system-observability/Cargo.toml`
- **Issue**: Uses deprecated async APIs
- **Current Status**: At latest compatible version (0.24.x)
- **Upgrade Path**: Update to `redis v1.x` when available
- **Impact**: Medium - affects caching and observability
- **Timeline**: Q2 2026 (after v1.x stabilizes)

### 3. `sampling v0.1.1`
- **Location**: `system-resources/Cargo.toml`
- **Issue**: Uses deprecated rand/std APIs
- **Current Status**: At latest compatible version (0.1.x)
- **Upgrade Path**: Update to `sampling v1.x` when available
- **Impact**: Low - only used for statistical sampling
- **Timeline**: Q3 2026 (when v1.x is available)

### 4. `sqlx-postgres v0.7.4`
- **Location**: Workspace dependency in `Cargo.toml`
- **Issue**: Uses deprecated PostgreSQL client APIs
- **Current Status**: At latest compatible version (0.7.x)
- **Upgrade Path**: Update sqlx to v0.8.x when PostgreSQL driver is stable
- **Impact**: High - affects all database operations
- **Timeline**: Q4 2025 (when sqlx v0.8.x is stable)

## Mitigation Strategy

### Current Approach
- All dependencies are at their latest compatible versions
- Warnings are acknowledged and documented
- No immediate action required as warnings don't affect functionality

### Future Resolution Plan

1. **Monitor Releases**: Track new major versions of affected crates
2. **Testing**: When new versions are available, create test branches for compatibility
3. **Gradual Rollout**: Update one dependency at a time with full integration testing
4. **Fallback**: Maintain current versions as fallbacks during transition

### Risk Assessment

- **Low Risk**: Warnings don't affect current functionality
- **Medium Risk**: Future Rust versions may break builds (6+ months away)
- **High Risk**: sqlx-postgres update may require significant testing

## Action Items

- [x] Document all future incompatibility warnings
- [ ] Monitor release announcements for affected crates
- [ ] Create upgrade plan for each dependency
- [ ] Schedule testing windows for major version updates
- [ ] Establish rollback procedures for failed upgrades

## Verification

Run the following to verify current status:
```bash
cargo check 2>&1 | grep "future-incompat"
```

Expected output: No new warnings beyond the documented ones above.



