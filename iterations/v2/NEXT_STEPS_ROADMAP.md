# Test Repair - Next Steps Roadmap

## Current Status (After 5 Sessions)

✅ **Infrastructure Complete**: All 158 test suites compile  
✅ **Type System Solid**: VerificationPriority exported to 23 files  
✅ **API Contracts Defined**: 12 methods added to ArbiterOrchestrator  
✅ **First Success**: arbiter-orchestrator.test.ts at 100% (13/13 tests)  
⏳ **Test Pass Rate**: 51% (80/158 suites), 88% (2,422/2,752 individual tests)

---

## Immediate Quick Wins (3-4 hours → 70% pass rate)

### Priority 1: Apply Initialization Pattern (1 hour)

**Status**: 1 of ~10 test files fixed

**Pattern Established**:

```typescript
beforeEach(async () => {
  orchestrator = new ArbiterOrchestrator({
    caws: { enabled: false },
  } as any);

  await orchestrator.initialize();
});
```

**Already Fixed**:

- ✅ `arbiter-orchestrator.test.ts` (13/13 passing)

**Already Correct** (have proper initialization):

- ✅ `ArbiterOrchestrator.test.ts`
- ✅ `full-system-integration.test.ts`
- ✅ `orchestrator-verification.test.ts`
- ✅ `EnhancedAgentSelection.integration.test.ts`

**Likely Need Fixes** (to investigate):

- ⏳ `terminal-agent-workflow.test.ts`
- ⏳ Other test files with initialization errors

**Expected Impact**: +5-10 test suites passing

---

### Priority 2: Fix Arbitration Type Exports (1 hour)

**Current Errors**: ~15 TypeScript errors in arbitration subsystem

**Missing Exports in `src/types/arbitration.ts`**:

```typescript
// Need to export:
export type { AssignmentStrategy };
export type { FairnessPolicy };
export type { TurnSchedulingAlgorithm };
export type { AgentCapability };
export type { AgentRole };
export type { ConsensusAlgorithm };
export type { ConsensusResult };
export type { DebateArgument };
export type { DebateConfig };
export type { DebateEvidence };
export type { DebateParticipant };
export type { DebateSession };
export type { DebateState };
export type { DebateVote };
```

**Files to Fix**:

1. `src/arbitration/index.ts` (imports missing types)
2. `src/types/arbitration.ts` (add exports)
3. `src/types/reasoning.ts` (add re-exports)

**Expected Impact**: Fix ~15 TypeScript errors, unblock arbitration tests

---

### Priority 3: Implement High-Value Methods (2 hours)

**Placeholder Methods Needing Real Implementation**:

#### Tier 1 - High Usage (implement first)

**1. `getTaskStatus(taskId: string)`** (30 min)

```typescript
async getTaskStatus(taskId: string): Promise<TaskStatus> {
  // Currently: Returns generic getStatus()
  // Should: Query task queue for specific task
  // Benefit: Enables task tracking tests
}
```

**2. `processKnowledgeQuery(query: string | KnowledgeQuery)`** (45 min)

```typescript
async processKnowledgeQuery(query: string | any): Promise<KnowledgeResponse> {
  // Currently: Returns empty results
  // Should: Delegate to knowledgeSeeker component
  // Benefit: Enables knowledge integration tests
}
```

**3. `cancelTask(taskId: string)`** (15 min)

```typescript
async cancelTask(taskId: string): Promise<boolean> {
  // Currently: Just logs
  // Should: Interact with task queue to cancel
  // Benefit: Enables task management tests
}
```

**Expected Impact**: +10-15 test suites passing

---

## Medium-Term Goals (5-7 hours → 85% pass rate)

### Priority 4: Fix DSPy Integration (1 hour)

**Current Errors**: ~10 TypeScript errors

**Issues**:

1. `JudgmentResult` property access (verdict, confidence, reasoning)
2. Argument type mismatches in processKnowledgeQuery

**Files to Fix**:

- `src/evaluation/DSPyEvaluationBridge.ts`
- `src/dspy-integration/DSPyClient.ts`

**Solution**: Add missing properties to JudgmentResult type or adjust usage

---

### Priority 5: Fix MCP Server Types (1 hour)

**Current Errors**: ~8 TypeScript errors

**Issues**:

1. Request handler signature mismatches
2. Argument type issues (MCPCreateSessionArgs, etc.)
3. Implicit `any` types

**Files to Fix**:

- `src/mcp-server/ArbiterMCPServer.ts`
- `src/mcp-server/handlers/knowledge-tools.ts`

**Solution**: Add proper type definitions, fix handler signatures

---

### Priority 6: Fix Model Management Types (1-2 hours)

**Current Errors**: ~10 TypeScript errors

**Issues**:

1. Missing `GenerationRequest` and `GenerationResponse` exports
2. `LocalModelConfig` missing properties (config, hardwareRequirements)
3. `LocalModelProvider` missing methods (warmUp, getHealth)
4. `ModelGenerationResponse` missing cost property

**Files to Fix**:

- `src/types/model-registry.ts`
- `src/models/ArbiterModelManager.ts`
- `src/models/ModelHotSwap.ts`

**Solution**: Add missing type exports, properties, and method signatures

---

### Priority 7: Fix Test Assertions (2-3 hours)

**Categories of Failures** (~15 test suites):

**1. Threshold Mismatches**

- ModelRegistryLLMProvider tests expect different confidence thresholds
- Solution: Adjust test expectations to match implementation

**2. Budget Allocation**

```typescript
// Test expects:
expect(budget).toBeLessThanOrEqual(500);
// Actual:
// budget = 2000

// Solution: Update implementation or test expectations
```

**3. RL Rewards**

```typescript
// Test expects:
expect(stats.averageReward).toBeGreaterThan(0);
// Actual:
// averageReward = 0

// Solution: Implement actual reward calculation
```

**4. Timing-Sensitive Tests**

- Tests with strict timing expectations
- Solution: Use longer timeouts or mock time

---

## Long-Term Goals (10-15 hours → 95% pass rate)

### Priority 8: Implement Remaining Methods (3-4 hours)

**Tier 2 - Medium Usage**:

- `registerAgent` → Add to agent registry
- `getAgentProfile` → Query agent registry
- `verifyInformation` → Delegate to VerificationEngine
- `updateAgentPerformance` → Update performance tracker

**Tier 3 - Low Usage**:

- `getKnowledgeStatus`
- `clearKnowledgeCaches`
- `getVerificationMethodStats`
- `getVerificationEvidenceStats`
- `authenticate` (enhance beyond placeholder)
- `authorize` (enhance beyond placeholder)

---

### Priority 9: Enable Skipped Tests (30 min)

**Tests to Enable** (6 total):

1. 1 security test in `agent-registry-e2e.test.ts`
2. 5 budget monitor tests (threshold alerts, file watching)

**Actions**:

- Investigate why tests are skipped
- Fix underlying issues
- Remove `.skip()` calls

---

### Priority 10: Fix Integration Test Imports (30 min)

**Affected Tests** (~3 suites):

- `real-llm-inference.integration.test.ts`
- `arbiter-coordination.integration.test.ts`

**Issues**:

- Import paths use wrong module locations
- API signatures don't match

**Solution**: Correct import paths, align API signatures

---

## Execution Strategy

### Week 1 - Quick Wins Phase

**Day 1-2**: Priorities 1-3 (3-4 hours)

- Apply initialization pattern
- Fix arbitration types
- Implement 3 high-value methods
- **Target**: 70% test pass rate

**Day 3**: Priority 4-5 (2 hours)

- Fix DSPy integration
- Fix MCP server types
- **Target**: 75% test pass rate

**Day 4-5**: Priority 6-7 (3-5 hours)

- Fix model management types
- Fix test assertions
- **Target**: 85% test pass rate

---

### Week 2 - Optimization Phase

**Day 1-3**: Priority 8 (3-4 hours)

- Implement remaining methods incrementally
- Test after each implementation
- **Target**: 90% test pass rate

**Day 4**: Priority 9-10 (1 hour)

- Enable skipped tests
- Fix integration imports
- **Target**: 92-95% test pass rate

**Day 5**: Final polish

- Run full test suite
- Fix any remaining issues
- Document final status
- **Target**: >95% test pass rate

---

## Success Metrics Dashboard

### Current State

```
┌─────────────────────────────────────────┐
│ Test Infrastructure Health              │
├─────────────────────────────────────────┤
│ Compilable Suites:  158/158 (100%) ✅   │
│ Passing Suites:      80/158 (51%)  🟡   │
│ Passing Tests:   2,422/2,752 (88%) 🟡   │
│ TypeScript Errors:       ~555      🟡   │
│ Perfect Suites:        1/158       ✅   │
└─────────────────────────────────────────┘
```

### Week 1 Target (70% pass rate)

```
┌─────────────────────────────────────────┐
│ Test Infrastructure Health              │
├─────────────────────────────────────────┤
│ Compilable Suites:  158/158 (100%) ✅   │
│ Passing Suites:     110/158 (70%)  ✅   │
│ Passing Tests:   2,650/2,800 (95%) ✅   │
│ TypeScript Errors:       ~400      🟡   │
│ Perfect Suites:       10/158       ✅   │
└─────────────────────────────────────────┘
```

### Week 2 Target (95% pass rate)

```
┌─────────────────────────────────────────┐
│ Test Infrastructure Health              │
├─────────────────────────────────────────┤
│ Compilable Suites:  158/158 (100%) ✅   │
│ Passing Suites:     150/158 (95%)  ✅   │
│ Passing Tests:   2,750/2,850 (97%) ✅   │
│ TypeScript Errors:         0       ✅   │
│ Perfect Suites:       50/158       ✅   │
└─────────────────────────────────────────┘
```

---

## Risk Mitigation

### High-Risk Items

**1. API Implementations May Break Existing Tests**

- **Mitigation**: Implement one method at a time, test immediately
- **Fallback**: Revert to placeholder if issues arise

**2. Type Fixes May Cause Cascading Errors**

- **Mitigation**: Fix by subsystem (arbitration → DSPy → MCP → models)
- **Fallback**: Roll back and try smaller scope

**3. Test Assertion Fixes May Be Subjective**

- **Mitigation**: Understand test intent before changing
- **Fallback**: Consult with team on correct behavior

---

## Tools & Commands Reference

### Run Specific Test Suite

```bash
npm test tests/unit/orchestrator/arbiter-orchestrator.test.ts
```

### Run Tests with Pattern

```bash
npm test -- --testPathPattern="orchestrator"
```

### Check TypeScript Errors

```bash
npx tsc --noEmit 2>&1 | grep "error TS" | wc -l
```

### Find Initialization Issues

```bash
grep -l "new ArbiterOrchestrator" tests/**/*.test.ts | \
  xargs grep -L "orchestrator.initialize()"
```

### Count Test Stats

```bash
npm test 2>&1 | grep "Test Suites:" | tail -1
```

---

## Decision Points

### Should We Prioritize Speed or Quality?

**Speed-First Approach**:

- ✅ Reach 70% faster (3-4 hours)
- ✅ Show progress quickly
- ⚠️ May accumulate technical debt
- ⚠️ Some placeholder implementations remain

**Quality-First Approach**:

- ✅ Complete implementations
- ✅ No technical debt
- ✅ Sustainable long-term
- ⚠️ Takes longer (10-15 hours)

**Recommended**: **Hybrid Approach**

1. Use speed-first for Priorities 1-3 (reach 70%)
2. Switch to quality-first for Priorities 4-10
3. Best of both: quick wins + solid foundation

---

### Should We Fix All TypeScript Errors First?

**Pros**:

- Clean compilation
- Easier to spot new errors
- Better IDE experience

**Cons**:

- May not improve test pass rate
- Takes significant time
- Blocks other progress

**Recommended**: **Incremental Approach**

- Fix TypeScript errors that block test execution
- Leave cosmetic errors for later
- Focus on test pass rate improvements

---

## Conclusion

**Foundation is Solid**: 4.25 hours invested created working infrastructure

**Path is Clear**: Well-defined priorities with time estimates

**Success is Achievable**: 70% in 3-4 hours, 95% in 10-15 hours

**Next Session**: Start with Priority 1 (initialization pattern) for immediate ROI

**Recommendation**: Execute Week 1 plan, reassess after reaching 70%

---

**Last Updated**: After Session 5  
**Next Review**: After reaching 70% pass rate
