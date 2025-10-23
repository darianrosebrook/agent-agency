# V3 Testing Strategy & Implementation Plan

**Date**: 2025-10-17  
**Status**: In Development  
**Priority**: Critical for Production Readiness

## **Testing Objectives**

1. **Comprehensive Coverage**: Achieve 80%+ line coverage and 90%+ branch coverage across all V3 components
2. **Integration Validation**: Ensure all cross-component communication works correctly
3. **Performance Assurance**: Validate performance budgets and SLAs are met
4. **Contract Compliance**: Verify all API contracts and schemas are properly validated
5. **Production Readiness**: Ensure system is ready for enterprise deployment

## **Current Test Status**

### **Working Components**

- **Claim Extraction Pipeline**: 9/11 tests passing (comprehensive unit tests)
- **Council System**: Contract roundtrip tests, schema conformance tests
- **Orchestration**: Adapter tests, persistence integration tests
- **Embedding Service**: Basic unit tests with dummy provider

### **Compilation Issues (Blocking Tests)**

- **Provenance**: 17 compilation errors (JWT signing, git integration thread safety)
- **Workers**: 58 compilation errors (missing imports, type mismatches)
- **Research**: 5 compilation errors (missing Hash trait, field mismatches)
- **Workspace State Manager**: 44 compilation errors (dependency conflicts)

### **CAWS Tools**

- **Status**: 62/84 tests passing (74% pass rate)
- **Issues**: CLI workflow integration tests failing due to stdio handling
- **Schema Contracts**: AJV warnings about union types

## 🏗️ **Testing Architecture**

### **Test Pyramid Structure**

```
                    E2E Tests (5%)
                   ┌─────────────────┐
                  │  Full System    │
                  │  Integration    │
                 └─────────────────┘
                Integration Tests (15%)
               ┌─────────────────────────┐
              │  Cross-Component        │
              │  Communication          │
             └─────────────────────────┘
            Unit Tests (80%)
           ┌─────────────────────────────────┐
          │  Individual Components          │
          │  Business Logic                 │
          │  Edge Cases                     │
         └─────────────────────────────────┘
```

### **Test Categories**

1. **Unit Tests** (80% of test suite)

   - Component-specific logic
   - Business rule validation
   - Edge case handling
   - Error condition testing

2. **Integration Tests** (15% of test suite)

   - Cross-component communication
   - Database integration
   - External API integration
   - Contract validation

3. **End-to-End Tests** (5% of test suite)
   - Full system workflows
   - User journey validation
   - Performance benchmarks
   - Production-like scenarios

## **Implementation Plan**

### **Phase 1: Fix Compilation Issues (Week 1)**

#### **Priority 1: Critical Compilation Fixes**

- [ ] **Provenance Crate**

  - Fix JWT signing method calls (`from_pem` → `from_rsa_pem`)
  - Resolve git integration thread safety (wrap Repository in Mutex)
  - Fix borrow checker issues in service methods
  - Add missing trait derives (Hash, Eq for ViolationSeverity)

- [ ] **Workers Crate**

  - Fix import paths (council types, TaskSpec)
  - Resolve type mismatches (ChangeComplexity, numeric types)
  - Fix missing field errors (constitutional_ref, busy_workers)
  - Add missing Debug derives for trait objects

- [ ] **Research Crate**

  - Add Hash trait to QueryType enum
  - Fix field name mismatches (relevance_score → score)
  - Resolve type visibility issues

- [ ] **Workspace State Manager**
  - Resolve dependency conflicts (libgit2-sys, libsqlite3-sys)
  - Fix missing trait implementations

#### **Priority 2: Test Infrastructure Setup**

- [ ] **Coverage Reporting**

  - Configure grcov for Rust coverage
  - Set up lcov report generation
  - Integrate with CI/CD pipeline

- [ ] **Test Data Management**
  - Create test data factories
  - Set up database test fixtures
  - Implement test isolation strategies

### **Phase 2: Unit Test Implementation (Week 2-3)**

#### **Component-Specific Unit Tests**

**Council System**

- [ ] **Judge Verdict Tests**

  - Test all judge types (Constitutional, Technical, Quality, Integration)
  - Validate verdict reasoning and confidence scoring
  - Test evidence enrichment integration

- [ ] **Consensus Coordinator Tests**

  - Test weighted voting algorithms
  - Validate consensus score calculations
  - Test debate protocol triggers

- [ ] **Evidence Enrichment Tests**
  - Test claim extraction integration
  - Validate evidence confidence calculations
  - Test evidence source diversity scoring

**Claim Extraction Pipeline**

- [ ] **Stage-Specific Tests**

  - Disambiguation: pronoun resolution, technical term detection
  - Qualification: verifiability assessment, content rewriting
  - Decomposition: atomic claim extraction, context brackets
  - Verification: evidence collection, council integration

- [ ] **Pipeline Integration Tests**
  - End-to-end pipeline processing
  - Error handling and recovery
  - Metadata tracking validation

**Research Agent**

- [ ] **Knowledge Seeking Tests**

  - Vector search functionality
  - Context synthesis algorithms
  - Cross-reference detection

- [ ] **Web Scraping Tests**
  - Content extraction accuracy
  - Rate limiting compliance
  - Error handling for failed requests

**Orchestration System**

- [ ] **Task Routing Tests**

  - Worker selection algorithms
  - Load balancing strategies
  - CAWS compliance checking

- [ ] **Persistence Tests**
  - Database operations
  - Transaction handling
  - Rollback scenarios

### **Phase 3: Integration Tests (Week 4)**

#### **Cross-Component Integration**

**Council ↔ Claim Extraction**

- [ ] **Evidence Flow Tests**
  - Test claim extraction → evidence enrichment → judge verdicts
  - Validate evidence confidence propagation
  - Test error handling in evidence chain

**Orchestration ↔ Council**

- [ ] **Task Evaluation Tests**
  - Test task spec → council evaluation → final verdict
  - Validate provenance emission
  - Test consensus building

**Research ↔ Knowledge Base**

- [ ] **Knowledge Integration Tests**
  - Test research agent → knowledge storage → retrieval
  - Validate context synthesis
  - Test cross-reference detection

**Workers ↔ CAWS Compliance**

- [ ] **Compliance Validation Tests**
  - Test worker output → CAWS checking → compliance scoring
  - Validate violation detection
  - Test remediation suggestions

### **Phase 4: Contract & Performance Tests (Week 5)**

#### **Contract Tests**

- [ ] **API Schema Validation**

  - Test all JSON schemas against examples
  - Validate request/response contracts
  - Test schema evolution compatibility

- [ ] **Data Structure Contracts**
  - Test serialization/deserialization
  - Validate type safety
  - Test backward compatibility

#### **Performance Tests**

- [ ] **Benchmark Tests**

  - API response time benchmarks
  - Database query performance
  - Memory usage profiling

- [ ] **Load Tests**
  - Concurrent user simulation
  - Stress testing scenarios
  - Resource exhaustion testing

### **Phase 5: End-to-End Tests (Week 6)**

#### **Full System Workflows**

- [ ] **Complete Task Execution**

  - Test full task lifecycle: spec → routing → execution → evaluation
  - Validate end-to-end data flow
  - Test error recovery scenarios

- [ ] **Multi-Tenant Scenarios**

  - Test concurrent task execution
  - Validate tenant isolation
  - Test resource sharing

- [ ] **Production-Like Scenarios**
  - Test with realistic data volumes
  - Validate performance under load
  - Test failure recovery

## 🛠️ **Test Infrastructure**

### **Testing Tools & Frameworks**

**Rust Testing**

- **Test Runner**: Built-in `cargo test`
- **Assertion Library**: Built-in assertions + `assert_matches!`
- **Mocking**: `mockall` for trait mocking
- **Coverage**: `grcov` + `lcov` for coverage reporting
- **Property Testing**: `quickcheck` for property-based tests

**Integration Testing**

- **Database**: Test containers with PostgreSQL
- **HTTP**: `reqwest` + `mockito` for API testing
- **Async**: `tokio::test` for async test support

**Performance Testing**

- **Benchmarking**: `criterion` for micro-benchmarks
- **Load Testing**: `wrk` or `hey` for HTTP load testing
- **Profiling**: `perf` + `flamegraph` for performance analysis

### **Test Data Management**

**Test Factories**

```rust
// Example test factory pattern
pub struct TestDataFactory;

impl TestDataFactory {
    pub fn create_task_spec() -> TaskSpec { /* ... */ }
    pub fn create_worker() -> Worker { /* ... */ }
    pub fn create_claim() -> AtomicClaim { /* ... */ }
}
```

**Database Fixtures**

- Separate test database instance
- Transaction-wrapped tests for isolation
- Seed data for consistent test scenarios

**Mock Services**

- Mock external API dependencies
- Deterministic responses for testing
- Error injection capabilities

### **CI/CD Integration**

**GitHub Actions Workflow**

```yaml
name: V3 Testing Pipeline
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
      - name: Run Tests
        run: cargo test --workspace --all-features
      - name: Generate Coverage
        run: cargo test --workspace --all-features -- -C instrument-coverage
      - name: Upload Coverage
        uses: codecov/codecov-action@v3
```

**Quality Gates**

- Minimum 80% line coverage
- Minimum 90% branch coverage
- All tests must pass
- No compilation warnings
- Performance benchmarks within SLA

## **Success Metrics**

### **Coverage Targets**

- **Line Coverage**: 80%+ across all components
- **Branch Coverage**: 90%+ for critical paths
- **Function Coverage**: 95%+ for public APIs

### **Test Quality Metrics**

- **Test Execution Time**: < 5 minutes for full suite
- **Flaky Test Rate**: < 1% (zero tolerance for production)
- **Test Maintenance Cost**: < 10% of development time

### **Performance Targets**

- **API Response Time**: P95 < 250ms
- **Database Query Time**: P95 < 100ms
- **Memory Usage**: < 512MB per component
- **CPU Usage**: < 50% under normal load

## **Continuous Improvement**

### **Test Maintenance**

- Regular test review and refactoring
- Automated test quality analysis
- Performance regression detection
- Coverage trend monitoring

### **Test Evolution**

- Add tests for new features
- Update tests for API changes
- Enhance test data factories
- Improve test isolation

### **Quality Assurance**

- Code review for test changes
- Test strategy review sessions
- Performance benchmark updates
- Security test integration

## **Next Steps**

1. **Immediate (This Week)**

   - Fix remaining compilation errors
   - Set up basic test infrastructure
   - Implement unit tests for working components

2. **Short Term (Next 2 Weeks)**

   - Complete unit test coverage
   - Implement integration tests
   - Set up CI/CD pipeline

3. **Medium Term (Next Month)**

   - Add performance benchmarks
   - Implement E2E tests
   - Achieve production readiness

4. **Long Term (Ongoing)**
   - Maintain test quality
   - Continuous performance monitoring
   - Test strategy evolution

---

**This testing strategy ensures V3 achieves production-ready quality with comprehensive test coverage, reliable integration, and performance assurance.**
