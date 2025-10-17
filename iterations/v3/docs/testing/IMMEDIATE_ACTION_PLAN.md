# Immediate Testing Action Plan

**Date**: 2025-10-17  
**Priority**: Critical - Blocking all testing progress

## 🚨 **Critical Blockers**

### **Compilation Errors (Must Fix First)**

#### **1. Provenance Crate (17 errors)**

```bash
# Current errors:
- JWT signing method calls (from_pem → from_rsa_pem) ✅ FIXED
- Git integration thread safety (Repository not Sync) ✅ PARTIALLY FIXED
- Borrow checker issues in service methods ✅ FIXED
- Missing trait derives (Hash, Eq for ViolationSeverity) ✅ FIXED
```

**Status**: 80% fixed, remaining issues with git integration async trait

#### **2. Workers Crate (58 errors)**

```bash
# Current errors:
- Import paths (council types, TaskSpec) ✅ FIXED
- Type mismatches (ChangeComplexity, numeric types) ❌ PENDING
- Missing field errors (constitutional_ref, busy_workers) ❌ PENDING
- Missing Debug derives for trait objects ❌ PENDING
```

**Status**: 30% fixed, major type system issues remain

#### **3. Research Crate (5 errors)**

```bash
# Current errors:
- Missing Hash trait on QueryType enum ❌ PENDING
- Field name mismatches (relevance_score → score) ❌ PENDING
- Type visibility issues ❌ PENDING
```

**Status**: 0% fixed

#### **4. Workspace State Manager (44 errors)**

```bash
# Current errors:
- Dependency conflicts (libgit2-sys, libsqlite3-sys) ❌ PENDING
- Missing trait implementations ❌ PENDING
```

**Status**: 0% fixed

## 🎯 **Immediate Actions (Next 2 Hours)**

### **Step 1: Fix Research Crate (Easiest)**

```rust
// File: research/src/types.rs
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QueryType {
    Semantic,
    Keyword,
    Hybrid,
}

// File: research/src/enhanced_knowledge_seeker.rs
// Fix field name: relevance_score → score
let score = result.score * self.config.hybrid_search.vector_weight;
```

### **Step 2: Fix Workers Crate Type Issues**

```rust
// File: workers/src/types.rs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeComplexity {
    Low,
    Medium,
    High,
}

// Fix numeric type mismatches
let busy_factor = if recent_tasks > 0.0 {
    worker.performance_metrics.total_tasks as f32 / recent_tasks
} else {
    1.0
};
```

### **Step 3: Fix Workspace State Manager Dependencies**

```toml
# File: workspace-state-manager/Cargo.toml
[dependencies]
git2 = "0.19"  # Align with other crates
# Remove gitoxide dependency to avoid conflicts
```

### **Step 4: Complete Provenance Git Integration**

```rust
// File: provenance/src/git_integration.rs
// Implement proper thread-safe git operations
// or temporarily disable async trait implementation
```

## 🧪 **Testing Implementation (After Compilation Fixes)**

### **Phase 1: Unit Tests for Working Components**

#### **Claim Extraction Pipeline (Already 80% Complete)**

```bash
# Current status: 9/11 tests passing
# Remaining: Fix 2 failing tests due to stub implementations
```

#### **Council System (Basic Tests Exist)**

```bash
# Add comprehensive unit tests for:
- Judge verdict reasoning
- Evidence enrichment
- Consensus coordination
```

#### **Embedding Service (Basic Tests Exist)**

```bash
# Add comprehensive unit tests for:
- Vector similarity calculations
- Cache management
- Context generation
```

### **Phase 2: Integration Tests**

#### **Council ↔ Claim Extraction Integration**

```rust
#[tokio::test]
async fn test_evidence_enrichment_flow() {
    // Test: TaskSpec → Claim Extraction → Evidence → Judge Verdict
}
```

#### **Orchestration ↔ Council Integration**

```rust
#[tokio::test]
async fn test_task_evaluation_flow() {
    // Test: TaskSpec → Council Evaluation → Final Verdict
}
```

## 📊 **Success Criteria**

### **Compilation Success**

- [ ] All crates compile without errors
- [ ] All crates compile without warnings
- [ ] `cargo test --workspace --all-features` runs successfully

### **Test Coverage Targets**

- [ ] 80%+ line coverage on working components
- [ ] 90%+ branch coverage on critical paths
- [ ] All integration tests pass

### **Performance Benchmarks**

- [ ] Unit tests complete in < 30 seconds
- [ ] Integration tests complete in < 2 minutes
- [ ] No memory leaks in test suite

## 🚀 **Execution Plan**

### **Hour 1: Fix Compilation Errors**

1. Fix Research crate (5 errors) - 15 minutes
2. Fix Workers crate type issues (20 errors) - 30 minutes
3. Fix Workspace State Manager dependencies (15 minutes)

### **Hour 2: Verify Compilation & Start Testing**

1. Run full compilation check - 10 minutes
2. Implement unit tests for working components - 40 minutes
3. Set up test infrastructure - 10 minutes

### **Next Session: Integration Testing**

1. Implement cross-component integration tests
2. Add performance benchmarks
3. Set up CI/CD pipeline

## 🔧 **Tools & Commands**

### **Compilation Verification**

```bash
# Check all crates compile
cargo check --workspace --all-features

# Run tests (after compilation fixes)
cargo test --workspace --all-features

# Generate coverage report
RUSTFLAGS="-C instrument-coverage" cargo test --workspace --all-features
grcov . -s . -t lcov --llvm --branch --ignore-not-existing -o target/coverage/lcov.info
```

### **Test Execution**

```bash
# Run specific crate tests
cargo test -p claim-extraction
cargo test -p agent-agency-council

# Run with output
cargo test --workspace --all-features -- --nocapture

# Run integration tests only
cargo test --workspace --all-features --test '*integration*'
```

## 📋 **Checklist**

### **Compilation Fixes**

- [ ] Research crate compiles
- [ ] Workers crate compiles
- [ ] Workspace State Manager compiles
- [ ] Provenance crate compiles
- [ ] All crates compile without warnings

### **Test Implementation**

- [ ] Unit tests for claim extraction (complete existing)
- [ ] Unit tests for council system
- [ ] Unit tests for embedding service
- [ ] Integration tests for council ↔ claim extraction
- [ ] Integration tests for orchestration ↔ council

### **Infrastructure**

- [ ] Test data factories
- [ ] Coverage reporting setup
- [ ] CI/CD pipeline configuration
- [ ] Performance benchmark setup

---

**This plan focuses on the immediate blockers and provides a clear path to get V3 testing operational within the next 2 hours.**
