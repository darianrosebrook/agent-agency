# Integration Test Requirements for Critical Blocker Resolutions

This document outlines the integration test requirements for the critical blockers resolved in the v3 theory audit.

## Overview

The following blockers have been resolved with implementation code:
1. **CAWS Runtime Validator Integration** - Integrated real validator in MCP server
2. **Judge Database Operations** - Implemented real database queries for judges
3. **CAWS Compliance Checker** - Implemented real compliance checking in workers
4. **Provenance Evidence Collection** - Integrated with ProvenanceService

## Test Requirements

### 1. Judge Database Operations Tests

**Location**: `iterations/v3/data-interfaces-adapters/tests/judge_operations.rs`

**Requirements**:
- [ ] Test database setup and teardown
- [ ] Test judge creation with valid data
- [ ] Test judge retrieval
- [ ] Test judge evaluation creation
- [ ] Test judge evaluation retrieval by task_id
- [ ] Test type mapping between agent-orchestration and data-infrastructure types
- [ ] Test error handling for invalid data
- [ ] Test concurrent access scenarios

**Dependencies**:
- PostgreSQL database running
- Database migrations applied
- Test database configured

**Execution**:
```bash
cd iterations/v3/data-interfaces-adapters
DATABASE_URL=postgresql://localhost:5432/agent_agency_test cargo test --test judge_operations
```

### 2. CAWS Runtime Validator Integration Tests

**Location**: `iterations/v3/agent-mcp/tests/caws_integration.rs`

**Requirements**:
- [ ] Test tool manifest validation with valid manifests
- [ ] Test tool manifest validation with invalid manifests
- [ ] Test validation error reporting
- [ ] Test compliance score calculation
- [ ] Test violation detection and reporting
- [ ] Test integration with development-tools validator

**Dependencies**:
- development-tools crate available
- CAWS rulebook accessible

**Execution**:
```bash
cd iterations/v3/agent-mcp
cargo test --test caws_integration
```

### 3. CAWS Compliance Checker Tests

**Location**: `iterations/v3/agent-workers/tests/caws_checker.rs`

**Requirements**:
- [ ] Test task compliance checking with valid tasks
- [ ] Test task compliance checking with invalid tasks
- [ ] Test JSON and YAML parsing
- [ ] Test violation detection
- [ ] Test recommendation generation
- [ ] Test integration with CAWS runtime validator

**Dependencies**:
- development-tools crate available
- CAWS rulebook accessible

**Execution**:
```bash
cd iterations/v3/agent-workers
cargo test --test caws_checker
```

### 4. Provenance Evidence Collection Tests

**Location**: `iterations/v3/agent-research/tests/provenance_evidence.rs` (to be created)

**Requirements**:
- [ ] Test provenance evidence collection with valid claims
- [ ] Test task_id extraction from claim scope
- [ ] Test provenance chain querying
- [ ] Test evidence conversion from provenance records
- [ ] Test relevance and confidence calculation
- [ ] Test handling of missing provenance service
- [ ] Test handling of missing task_id

**Dependencies**:
- ProvenanceService available
- Test database with provenance entries
- system-quality-security crate available

**Execution**:
```bash
cd iterations/v3/agent-research
cargo test --test provenance_evidence
```

## End-to-End Integration Tests

### Full CAWS Compliance Flow Test

**Requirements**:
- [ ] Test complete flow: task creation → CAWS validation → execution → provenance tracking → evidence collection
- [ ] Test integration between all components
- [ ] Test error propagation and handling
- [ ] Test performance under load

**Location**: `iterations/v3/testing-validation/src/scenarios/caws_integration_e2e.rs` (to be created)

## Test Data Requirements

### Database Fixtures

- Test judges with various configurations
- Test judge evaluations for various tasks
- Test provenance records for various tasks
- Test CAWS compliance records

### Mock Data

- Valid and invalid tool manifests
- Valid and invalid task specifications
- Test claims with various scopes
- Test provenance chains

## Performance Requirements

- Database operations: < 100ms per operation
- CAWS validation: < 50ms per validation
- Evidence collection: < 200ms per claim
- End-to-end flow: < 2s per task

## Security Requirements

- Test input validation and sanitization
- Test SQL injection prevention
- Test access control and authorization
- Test sensitive data handling

## Continuous Integration

All integration tests should:
- Run in CI/CD pipeline
- Be marked with `#[ignore]` for manual execution
- Have clear documentation on setup requirements
- Provide meaningful error messages on failure

## Next Steps

1. Set up test database infrastructure
2. Create test fixtures and mock data
3. Implement missing integration tests
4. Add performance benchmarks
5. Integrate into CI/CD pipeline

