<!-- f4b5942e-1f70-445e-8fb4-071e754a61e6 b64639ef-5c91-462e-abfc-0c3ba570a4c4 -->
# Arbiter Edge Case Implementation Plan

## Context

Implement missing edge case handling capabilities identified in the edge case test suite. All work adheres to CAWS Tier 2 gates (80%+ branch coverage, 50%+ mutation score) with Tier 1 rigor for arbitration governance.

## Phase 0: Integration Groundwork

### TaskIntakeProcessor Integration Point

**Implementation Approach:**

- Modify `TaskOrchestrator.submitTask()` (line 516) to call TaskIntakeProcessor before routing
- Add `intakeProcessor` as constructor dependency with default instantiation
- Wrap existing task validation with intake processing

**Files to modify:**

- `src/orchestrator/TaskOrchestrator.ts` - add intake processing step
- `src/orchestrator/TaskOrchestrator.test.ts` - add intake rejection scenarios

**Acceptance:**

- TaskIntakeProcessor called before routing for all task submissions
- Rejection errors propagated with structured TaskIntakeIssue codes
- Existing task submission tests still pass

### PostgreSQL Schema Setup

**Implementation Approach:**

- Add migration for worker capabilities, task snapshots, credit ledger, and task memory tables
- Use existing PostgreSQL connection from project infrastructure
- Create repository interfaces for each persistence concern

**New files:**

- `migrations/008_worker_resilience.sql` - capability registry, task snapshots tables
- `migrations/009_credit_ledger.sql` - credit tracking and adaptive policy tables
- `src/orchestrator/repositories/WorkerCapabilityRepository.ts`
- `src/orchestrator/repositories/TaskSnapshotRepository.ts`
- `src/orchestrator/repositories/CreditLedgerRepository.ts`

**Schema design:**

```sql
-- Worker capabilities (real-time registration)
CREATE TABLE worker_capabilities (
  worker_id TEXT PRIMARY KEY,
  capabilities JSONB NOT NULL,
  health_status TEXT NOT NULL,
  saturation_ratio REAL NOT NULL,
  last_heartbeat TIMESTAMP NOT NULL,
  registered_at TIMESTAMP NOT NULL
);

-- Task snapshots (resumable state)
CREATE TABLE task_snapshots (
  task_id TEXT PRIMARY KEY,
  snapshot_data JSONB NOT NULL,
  snapshot_version INTEGER NOT NULL,
  created_at TIMESTAMP NOT NULL,
  updated_at TIMESTAMP NOT NULL
);

-- Credit ledger (worker performance)
CREATE TABLE credit_ledger (
  id SERIAL PRIMARY KEY,
  agent_id TEXT NOT NULL,
  credits REAL NOT NULL DEFAULT 0,
  debits REAL NOT NULL DEFAULT 0,
  reason TEXT NOT NULL,
  metadata JSONB,
  created_at TIMESTAMP NOT NULL
);

-- Task memory (long-horizon context)
CREATE TABLE task_memory (
  task_id TEXT NOT NULL,
  memory_key TEXT NOT NULL,
  memory_value JSONB NOT NULL,
  updated_at TIMESTAMP NOT NULL,
  PRIMARY KEY (task_id, memory_key)
);
```

## Phase 1: Task Intake Hardening

### Streaming Validation Enhancement

**Current State:** TaskIntakeProcessor handles complete payloads synchronously

**Enhancement:** Add incremental JSON parsing for large payloads (>5KB)

**Implementation:**

- Add `StreamingJSONParser` helper class that processes chunks
- Modify `process()` to detect oversized payloads and delegate to streaming parser
- Maintain synchronous interface (streaming is internal optimization)

**Files to modify:**

- `src/orchestrator/intake/TaskIntakeProcessor.ts` - add streaming logic
- `src/orchestrator/intake/StreamingJSONParser.ts` - new utility class

**Tests:**

- Property-based tests with large (10KB-50KB) JSON payloads
- Unicode edge cases (emoji, multi-byte characters)
- Binary detection with null bytes
- Malformed JSON at various chunk boundaries

### Priority Router Integration

**Implementation:**

- Extract priority from TaskIntakeEnvelope metadata
- Map to WorkerPriority enum in TaskOrchestrator
- Pass through to WorkerPoolSupervisor capacity evaluation

**Files to modify:**

- `src/orchestrator/TaskOrchestrator.ts` - use intake priority hint

## Phase 2: Worker Pool Resilience

### WorkerCapabilityRegistry

**Implementation Approach:**

- Create `WorkerCapabilityRegistry` class with PostgreSQL backing
- Track: worker ID, capabilities list, health status, saturation ratio, last heartbeat
- Expose: `register(worker)`, `deregister(workerId)`, `query(requiredCapabilities)`, `updateHealth(workerId, health)`

**New files:**

- `src/orchestrator/resources/WorkerCapabilityRegistry.ts`
- `tests/unit/orchestrator/resources/WorkerCapabilityRegistry.test.ts`

**Integration:**

- `TaskOrchestrator` constructor accepts optional registry instance
- `WorkerPoolSupervisor.evaluateCapacity()` queries registry for capability matching
- Workers send heartbeat messages with capability updates

### TaskSnapshotStore

**Implementation Approach:**

- Create `TaskSnapshotStore` with save/restore/delete operations
- Store serialized task execution state at configurable checkpoints
- Enable retry handler to resume from snapshot instead of restarting

**New files:**

- `src/orchestrator/state/TaskSnapshotStore.ts`
- `tests/unit/orchestrator/state/TaskSnapshotStore.test.ts`

**Integration:**

- `TaskOrchestrator.executeTask()` calls `snapshotStore.save()` before worker execution
- `TaskOrchestrator.handleTaskFailure()` calls `snapshotStore.restore()` for retry
- Cleanup snapshots after successful completion

### Chaos Testing Harness

**Implementation Approach:**

- Build deterministic chaos simulator with seeded PRNG
- Inject failures via worker method overrides
- Support scenarios: crash, timeout, saturation, capability mismatch

**New files:**

- `tests/unit/orchestrator/worker-resilience/ChaosSimulator.ts`
- `tests/unit/orchestrator/worker-resilience/WorkerPoolResilience.test.ts`

**Test scenarios:**

- Worker exhaustion (all workers busy, queue backpressure)
- Mid-task worker crash (snapshot resume)
- Timeout during execution (exponential backoff retry)
- Capability mismatch (graceful rejection with alternatives)

## Phase 3: Verification Extension

### New VerificationType Values

**Add to `src/types/verification.ts`:**

```typescript
export enum VerificationType {
  // Existing types...
  MATH_VERIFICATION = "math_verification",
  CODE_VERIFICATION = "code_verification",
  CONTEXT_VERIFICATION = "context_verification",
}
```

### Verification Adapters

**Math Verifier:**

- Use math.js for symbolic and numeric evaluation
- Verify equations, computations, statistical claims
- Return evidence with calculation steps

**Code Verifier:**

- Use isolated VM2 sandbox for code execution
- Verify code behavior claims (input/output, performance, correctness)
- Timeout protection and resource limits

**Context Verifier:**

- Query stored claims and citations for factual consistency
- Check temporal consistency (timeline validation)
- Cross-reference with conversation context

**New files:**

- `src/verification/adapters/MathVerifier.ts`
- `src/verification/adapters/CodeVerifier.ts`
- `src/verification/adapters/ContextVerifier.ts`
- `tests/unit/verification/adapters/MathVerifier.test.ts`
- `tests/unit/verification/adapters/CodeVerifier.test.ts`
- `tests/unit/verification/adapters/ContextVerifier.test.ts`

**Integration:**

- Register adapters in `VerificationEngineImpl` constructor
- Add to method selection in `executeVerificationMethod()`
- Configure timeouts and resource limits per adapter

### MCP Tool Adapter Interface

**Implementation:**

- Define `VerificationToolAdapter` interface for remote verifiers
- Support MCP protocol for agent-driven verification
- Enable agents to call verification tools during task execution

**New files:**

- `src/verification/adapters/MCPVerificationAdapter.ts`
- Configuration for pluggable MCP tool endpoints

## Phase 4: Arbitration Enhancement

### Confidence Scoring

**Implementation:**

- Create `ConfidenceScorer` utility in orchestrator
- Score based on: model reliability, worker agreement, past accuracy, CAWS compliance
- Weight factors: verification success rate (40%), claim evidence quality (30%), worker history (20%), arbitration wins (10%)

**New files:**

- `src/orchestrator/arbitration/ConfidenceScorer.ts`
- `tests/unit/orchestrator/arbitration/ConfidenceScorer.test.ts`

### Board Coordinator

**Implementation:**

- Create `ArbitrationBoardCoordinator` facade
- Aggregate pleading decisions with confidence weights
- Produce final arbitration decision with justification

**Integration:**

- Extend `PleadingWorkflowManager` to call confidence scorer
- Add board coordinator to resolve conflicting approvals
- Link claim-based arbitration from ClaimExtractor

**New files:**

- `src/orchestrator/arbitration/ArbitrationBoardCoordinator.ts`
- `tests/unit/orchestrator/arbitration/ArbitrationBoardCoordinator.test.ts`

## Phase 5: Credit Ledger & Adaptive Policies

### Credit Ledger

**Implementation:**

- Track worker performance metrics in PostgreSQL
- Credits for: successful completions, arbitration wins, CAWS compliance
- Debits for: failures, timeouts, non-compliance
- Query operations: `getBalance(agentId)`, `recordCredit()`, `recordDebit()`, `getTopPerformers()`

**New files:**

- `src/orchestrator/learning/CreditLedger.ts`
- `tests/unit/orchestrator/learning/CreditLedger.test.ts`

### Adaptive Policy Engine

**Implementation:**

- Load policy rules from `config/adaptive-policies.yaml`
- Adjust task assignment weights based on credit scores
- Modify timeout budgets and retry caps dynamically
- Expose: `adjustWeight(agentId)`, `getTimeoutMultiplier(agentId)`, `shouldRetry(agentId, attemptCount)`

**New files:**

- `src/orchestrator/learning/AdaptivePolicyEngine.ts`
- `config/adaptive-policies.yaml`
- `tests/unit/orchestrator/learning/AdaptivePolicyEngine.test.ts`

**Integration:**

- `TaskRoutingManager` queries adaptive engine for weight adjustments
- `TaskRetryHandler` consults engine for retry decisions
- Credit ledger updates after task completion/failure

## Phase 6: CAWS Enforcement Pipeline

### Multi-Stage Policy Checks

**Intake Stage (TaskIntakeProcessor):**

- Schema validation (already present)
- Forbidden field detection (secrets, credentials)
- Prompt injection detection (SQL, command injection patterns)

**Execution Stage (CAWSPolicyEnforcer):**

- Real-time budget tracking (file count, LOC count)
- Tool usage monitoring (allowed tools per task type)
- Reasoning depth guards (prevent infinite loops)

**Post-Task Stage (PolicyAuditManager):**

- Comprehensive CAWS compliance audit
- Waiver evaluation and approval workflow
- Violation reporting and remediation tracking

**New files:**

- `src/orchestrator/compliance/CAWSPolicyEnforcer.ts` - middleware for runtime checks
- `src/orchestrator/compliance/PolicyAuditManager.ts` - post-task auditing
- `src/orchestrator/compliance/PromptInjectionDetector.ts` - intake security
- `tests/unit/orchestrator/compliance/CAWSPolicyEnforcer.test.ts`
- `tests/unit/orchestrator/compliance/PolicyAuditManager.test.ts`

### Adversarial Test Suite

**Implementation:**

- Create adversarial payload corpus
- Test scenarios: SQL injection, command injection, path traversal, budget overflow, infinite recursion
- Assert proper rejection with actionable error codes

**New files:**

- `tests/security/adversarial-payloads.test.ts`
- `tests/security/fixtures/adversarial-corpus.json`

### Incident Response Playbooks

**Implementation:**

- Markdown documentation under `docs/caws/playbooks/`
- Playbooks for: injection detected, budget exceeded, worker compromise, arbitration deadlock
- PolicyAuditManager references playbooks in violation reports

**New files:**

- `docs/caws/playbooks/injection-response.md`
- `docs/caws/playbooks/budget-exceeded.md`
- `docs/caws/playbooks/worker-compromise.md`
- `docs/caws/playbooks/arbitration-deadlock.md`

## Phase 7: Integration & End-to-End Testing

### Full Pipeline Tests

**Implementation:**

- End-to-end scenarios exercising all components
- Test: intake → routing → execution → verification → arbitration → completion
- Include: happy path, retry with snapshot resume, arbitration with pleading, CAWS violation rejection

**New files:**

- `tests/integration/orchestrator/full-pipeline.test.ts`

### Performance Baseline Tests

**Implementation:**

- Establish baseline metrics for each component
- Assert: intake latency <50ms, routing decision <100ms, verification <5s per claim
- Include load tests with 100 concurrent tasks

**New files:**

- `tests/performance/orchestrator-baseline.test.ts`

## Quality Gates (CAWS Tier 2)

- [ ] Branch coverage ≥80% across all new modules
- [ ] Mutation score ≥50% for business logic
- [ ] All property-based tests pass with 1000 iterations
- [ ] Integration tests pass with real PostgreSQL database
- [ ] Chaos tests deterministically pass with fixed seeds
- [ ] Adversarial test suite rejects all malicious payloads
- [ ] Performance benchmarks within documented SLAs
- [ ] No linting errors or TypeScript compilation issues

## Rollout Strategy

1. Phase 0-1: Intake hardening (low risk, isolated)
2. Phase 2: Worker resilience (medium risk, affects scheduling)
3. Phase 3-4: Verification & arbitration (medium risk, extends existing)
4. Phase 5: Credit ledger (low risk, parallel tracking)
5. Phase 6: CAWS enforcement (high risk, gates all tasks - feature flag)
6. Phase 7: Integration testing & performance validation

Each phase includes:

- Implementation with unit tests
- Integration with existing orchestrator
- Documentation updates
- Manual verification with edge case scenarios from test suite

### To-dos

- [ ] Integrate TaskIntakeProcessor into TaskOrchestrator.submitTask() with rejection handling
- [ ] Create PostgreSQL migrations for worker capabilities, task snapshots, credit ledger, and task memory tables
- [ ] Implement repository interfaces for WorkerCapabilityRepository, TaskSnapshotRepository, and CreditLedgerRepository
- [ ] Add StreamingJSONParser for incremental parsing of large (>5KB) payloads in TaskIntakeProcessor
- [ ] Create property-based test suite for intake validation with malformed/unicode/binary payloads
- [ ] Implement WorkerCapabilityRegistry with PostgreSQL backing and heartbeat mechanism
- [ ] Implement TaskSnapshotStore with save/restore operations for resumable task execution
- [ ] Build deterministic chaos testing harness with seeded PRNG for worker failure simulation
- [ ] Add MATH_VERIFICATION, CODE_VERIFICATION, CONTEXT_VERIFICATION to VerificationType enum
- [ ] Implement MathVerifier, CodeVerifier, and ContextVerifier adapters with sandbox execution
- [ ] Create MCPVerificationAdapter interface for remote agent-driven verification
- [ ] Implement ConfidenceScorer with multi-factor weighting for arbitration decisions
- [ ] Create ArbitrationBoardCoordinator to aggregate pleading decisions with confidence scoring
- [ ] Implement CreditLedger with PostgreSQL persistence for worker performance tracking
- [ ] Create AdaptivePolicyEngine with YAML configuration for dynamic weight/timeout adjustments
- [ ] Implement CAWSPolicyEnforcer middleware for runtime budget and tool usage monitoring
- [ ] Create PolicyAuditManager for post-task CAWS compliance auditing and waiver evaluation
- [ ] Implement PromptInjectionDetector for intake-stage security validation
- [ ] Build adversarial test suite with injection, overflow, and traversal attack scenarios
- [ ] Write incident response playbooks for common CAWS violations and security incidents
- [ ] Create end-to-end pipeline tests covering intake → routing → execution → verification → arbitration
- [ ] Establish performance baselines and load tests with 100 concurrent tasks