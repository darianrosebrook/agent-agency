# V3 CAWS Compliance Plan

**Date**: December 2024  
**Goal**: Get V3 fully CAWS compliant for Tier 1 requirements  
**Status**: In Progress

---

## Current Status

### ✅ Completed
- **Working Spec**: Valid with 9 acceptance criteria (exceeds Tier 1 requirement of 5+)
- **CAWS Tools**: Available in `apps/tools/caws/` with 74% test pass rate
- **Architecture**: All 9 crates implemented with comprehensive functionality

### 🚧 In Progress
- **Compilation Issues**: Multiple crates have compilation errors that need fixing
- **Test Coverage**: No comprehensive test suite yet implemented
- **Mutation Testing**: Not yet set up for Rust crates

### ❌ Missing
- **Contract Tests**: Required for Tier 1 compliance
- **CI/CD Pipeline**: With CAWS gates integration
- **Performance Benchmarking**: Validation of Apple Silicon targets
- **Provenance Tracking**: Git integration for audit trails

---

## Tier 1 CAWS Requirements

Based on CAWS gate checker, Tier 1 requires:

| Requirement | Target | Current Status |
|-------------|--------|----------------|
| **Branch Coverage** | ≥90% | ❌ No tests implemented |
| **Mutation Score** | ≥70% | ❌ Not set up |
| **Max Files** | 40 | ✅ Within budget (9 crates) |
| **Max LOC** | 1500 | ✅ Within budget |
| **Trust Score** | ≥85 | ❌ Not measured |
| **Contracts** | Required | ❌ Not implemented |
| **Manual Review** | Required | ✅ Available |

---

## Implementation Strategy

### Phase 1: Fix Compilation Issues (Priority 1)

**Goal**: Get all crates compiling successfully

**Tasks**:
1. **Fix Council Crate** (97 compilation errors)
   - Add missing imports (`async_trait`, `futures`, `uuid`, `HashMap`)
   - Create missing `models.rs` module
   - Fix trait object issues with async traits
   - Add missing derive implementations (`PartialEq`, `Default`)
   - Fix type mismatches and dyn compatibility issues

2. **Fix Other Crates** (dependency issues)
   - Resolve cross-crate dependency references
   - Fix non-existent crate dependencies
   - Ensure workspace dependencies are properly configured

3. **Validate Build**
   - `cargo build --workspace` should succeed
   - `cargo test --workspace` should compile (tests may fail)

### Phase 2: Implement Basic Test Suite (Priority 2)

**Goal**: Achieve minimum test coverage for CAWS compliance

**Tasks**:
1. **Unit Tests** (Target: 90% branch coverage)
   - Add tests for each crate's core functionality
   - Test all public APIs and critical paths
   - Use `cargo test --workspace` with coverage reporting

2. **Integration Tests**
   - Test inter-crate communication
   - Test database operations
   - Test CAWS compliance checking

3. **Contract Tests** (Required for Tier 1)
   - Define API contracts for all public interfaces
   - Implement contract validation tests
   - Ensure backward compatibility

### Phase 3: Set Up Mutation Testing (Priority 3)

**Goal**: Achieve 70%+ mutation score

**Tasks**:
1. **Install Mutation Testing Tool**
   - Use `cargo-mutants` or similar for Rust
   - Configure mutation testing in CI/CD

2. **Strengthen Test Suite**
   - Identify surviving mutants
   - Add assertions to catch mutations
   - Improve test quality and coverage

### Phase 4: Implement CAWS Gates Integration (Priority 4)

**Goal**: Automated compliance checking

**Tasks**:
1. **CI/CD Pipeline**
   - Set up GitHub Actions or similar
   - Integrate CAWS gates checking
   - Automated compliance validation

2. **Performance Benchmarking**
   - Validate Apple Silicon optimization targets
   - Measure and track performance metrics
   - Set up performance regression detection

3. **Provenance Tracking**
   - Implement git integration for audit trails
   - Set up immutable decision logging
   - JWS signing for council decisions

---

## Immediate Next Steps

### Step 1: Fix Council Crate Compilation

The council crate has 97 compilation errors that need immediate attention:

**Critical Issues**:
- Missing `async_trait` import in multiple files
- Missing `futures` crate for `join_all`
- Missing `uuid` and `HashMap` imports
- Missing `models.rs` module
- Trait object issues with async traits (dyn compatibility)
- Missing derive implementations

**Fix Strategy**:
1. Add missing imports to all files
2. Create `council/src/models.rs` module
3. Fix trait object issues by making traits dyn-compatible
4. Add missing derive implementations
5. Fix type mismatches

### Step 2: Set Up Basic Testing Infrastructure

**Minimum Viable Test Suite**:
1. **Unit Tests** for each crate
2. **Integration Tests** for cross-crate functionality
3. **Contract Tests** for public APIs
4. **Coverage Reporting** with `cargo-tarpaulin`

### Step 3: Implement CAWS Runtime Validation

**Core Requirements**:
1. **CAWS Compliance Checking** in worker pool
2. **Budget Validation** against working spec
3. **Scope Enforcement** for file changes
4. **Provenance Tracking** for all decisions

---

## Success Criteria

### Tier 1 Compliance Checklist

- [ ] **All crates compile successfully** (`cargo build --workspace`)
- [ ] **Branch coverage ≥90%** (measured with `cargo-tarpaulin`)
- [ ] **Mutation score ≥70%** (measured with `cargo-mutants`)
- [ ] **Contract tests pass** (all public API contracts validated)
- [ ] **CAWS gates pass** (`npx tsx gates.ts all 1`)
- [ ] **Trust score ≥85** (calculated by CAWS tools)
- [ ] **Manual review completed** (code review and approval)

### Performance Targets

- [ ] **Council consensus <5s** (acceptance criteria A1)
- [ ] **Apple Silicon optimization** (acceptance criteria A4)
- [ ] **CAWS compliance >95%** (runtime validation)
- [ ] **Audit trail completeness** (provenance tracking)

---

## Risk Mitigation

### Compilation Issues
- **Risk**: Complex trait object issues may require architectural changes
- **Mitigation**: Start with simple fixes, use enum dispatch for trait objects if needed

### Test Coverage
- **Risk**: 90% branch coverage may be challenging for complex async code
- **Mitigation**: Focus on critical paths first, use integration tests for complex flows

### Mutation Testing
- **Risk**: 70% mutation score requires very strong test suite
- **Mitigation**: Use property-based testing, focus on business logic coverage

### Time Constraints
- **Risk**: Full compliance may take significant time
- **Mitigation**: Prioritize compilation fixes first, then incremental compliance improvements

---

## Tools and Commands

### Development Commands
```bash
# Build all crates
cargo build --workspace

# Run all tests
cargo test --workspace

# Check CAWS compliance
cd iterations/v3
npx tsx ../../apps/tools/caws/gates.ts tier 1

# Validate working spec
npx tsx ../../apps/tools/caws/validate.ts spec .caws/working-spec.yaml
```

### Coverage Commands
```bash
# Install coverage tool
cargo install cargo-tarpaulin

# Run coverage analysis
cargo tarpaulin --workspace --out Html
```

### Mutation Testing Commands
```bash
# Install mutation testing tool
cargo install cargo-mutants

# Run mutation testing
cargo mutants --workspace
```

---

## Timeline

### Week 1: Compilation Fixes
- Fix all 97 compilation errors in council crate
- Resolve dependency issues in other crates
- Achieve successful `cargo build --workspace`

### Week 2: Basic Testing
- Implement unit tests for core functionality
- Set up coverage reporting
- Achieve >50% branch coverage

### Week 3: Advanced Testing
- Implement contract tests
- Set up mutation testing
- Achieve >70% mutation score

### Week 4: CAWS Integration
- Set up CI/CD pipeline
- Implement CAWS gates integration
- Achieve Tier 1 compliance

---

## Conclusion

V3 has excellent architecture and comprehensive functionality, but needs compilation fixes and testing infrastructure to achieve CAWS compliance. The focus should be on:

1. **Immediate**: Fix compilation issues to get builds working
2. **Short-term**: Implement comprehensive test suite
3. **Medium-term**: Achieve Tier 1 CAWS compliance
4. **Long-term**: Optimize performance and add advanced features

The foundation is solid - we just need to make it build and test properly to meet CAWS standards.
