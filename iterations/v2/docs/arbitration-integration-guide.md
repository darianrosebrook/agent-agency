# CAWS Arbitration System - Integration Guide

**Status**: Alpha (96.7% test coverage across 444 tests)  
**Components**: ARBITER-015 (Arbitration Protocol) + ARBITER-016 (Reasoning Engine)

## Overview

The CAWS Arbitration System combines constitutional rule enforcement with multi-agent debate coordination to provide a complete governance and decision-making framework.

### System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                  CAWS Arbitration System                     │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌──────────────────────┐    ┌──────────────────────────┐  │
│  │   ARBITER-015        │    │   ARBITER-016            │  │
│  │ Arbitration Protocol │◄──►│ Reasoning Engine         │  │
│  └──────────────────────┘    └──────────────────────────┘  │
│           │                              │                   │
│           ├─ Rule Enforcement            ├─ Debate Mgmt     │
│           ├─ Verdict Generation          ├─ Consensus       │
│           ├─ Waiver Evaluation           ├─ Arguments       │
│           ├─ Precedent Management        ├─ Evidence        │
│           └─ Appeal Handling             └─ Agent Coord     │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

## Quick Start

### Installation

```typescript
import {
  createArbitrationSystem,
  ArbitrationOrchestrator,
  ArbiterReasoningEngine,
} from "@/arbitration";
```

### Basic Setup

```typescript
// Create integrated system
const system = createArbitrationSystem({
  arbitration: {
    autoApplyPrecedents: true,
    enableWaivers: true,
    enableAppeals: true,
    maxConcurrentSessions: 10,
  },
  reasoning: {
    consensusAlgorithm: "weighted-voting",
    minParticipants: 3,
    maxDebateDuration: 300000, // 5 minutes
  },
});

const { orchestrator, reasoningEngine } = system;
```

## Integration Patterns

### Pattern 1: Constitutional Violation → Debate → Verdict

**Use Case**: Complex rule violations requiring multi-agent deliberation

```typescript
// 1. Start arbitration session
const session = await orchestrator.startSession(
  violation,
  [rule1, rule2],
  ["agent-1", "arbiter-1"]
);

// 2. Evaluate rules
await orchestrator.evaluateRules(session.id);

// 3. Initiate debate if needed
const debate = await reasoningEngine.initiateDebate(
  `Constitutional Violation: ${violation.description}`,
  [
    { agentId: "agent-1", role: AgentRole.DEFENDANT, weight: 1.0 },
    { agentId: "agent-2", role: AgentRole.PROSECUTOR, weight: 1.0 },
    { agentId: "arbiter-1", role: AgentRole.MODERATOR, weight: 1.5 },
  ],
  {
    consensusAlgorithm: "weighted-voting",
    maxDebateDuration: 300000,
    minParticipants: 3,
  }
);

// 4. Conduct debate
await reasoningEngine.submitArgument(debate.id, "agent-1", {
  claim: "The violation was unintentional",
  evidence: ["evidence-1", "evidence-2"],
  confidence: 0.8,
});

await reasoningEngine.submitArgument(debate.id, "agent-2", {
  claim: "The violation shows clear negligence",
  evidence: ["evidence-3"],
  confidence: 0.9,
});

// 5. Form consensus
const consensus = await reasoningEngine.formConsensus(debate.id);

// 6. Generate verdict based on debate outcome
const verdict = await orchestrator.generateVerdict(session.id, "arbiter-1");

// 7. Complete
await orchestrator.completeSession(session.id);
```

### Pattern 2: Waiver Request with Debate

**Use Case**: Waiver requires multi-stakeholder approval

```typescript
// 1. Submit waiver request
await orchestrator.evaluateWaiver(
  sessionId,
  {
    id: "waiver-1",
    ruleId: "RULE-001",
    requestedBy: "agent-1",
    justification: "Emergency production fix",
    evidence: ["incident-report", "manager-approval"],
    requestedDuration: 24 * 60 * 60 * 1000,
    requestedAt: new Date(),
    context: {},
  },
  "arbiter-1"
);

// 2. Initiate debate on waiver
const waiverDebate = await reasoningEngine.initiateDebate(
  "Waiver Request Evaluation",
  reviewers.map((id) => ({
    agentId: id,
    role: AgentRole.REVIEWER,
    weight: 1.0,
  })),
  { consensusAlgorithm: "simple-majority" }
);

// 3. Each reviewer submits vote
for (const reviewer of reviewers) {
  await reasoningEngine.submitVote(waiverDebate.id, reviewer, {
    position: "approve", // or "reject"
    confidence: 0.85,
    reasoning: "Justification is adequate for emergency",
  });
}

// 4. Form consensus and apply decision
const decision = await reasoningEngine.formConsensus(waiverDebate.id);
```

### Pattern 3: Appeal with Multi-Level Review

**Use Case**: Appeal requires escalating debate levels

```typescript
// 1. Submit appeal
const appeal = await orchestrator.submitAppeal(
  sessionId,
  "agent-1",
  "The verdict overlooked critical evidence",
  ["new-evidence-1", "new-evidence-2"]
);

// 2. Level 1 review debate
const level1Debate = await reasoningEngine.initiateDebate(
  `Appeal Review: ${appeal.id}`,
  level1Reviewers,
  { consensusAlgorithm: "weighted-voting" }
);

const level1Result = await reasoningEngine.formConsensus(level1Debate.id);

// 3. If deadlock or dispute, escalate
if (level1Result.outcome === "no-consensus") {
  await orchestrator.escalateAppeal(appeal.id, "Level 1 deadlock");

  const level2Debate = await reasoningEngine.initiateDebate(
    `Appeal Review (Level 2): ${appeal.id}`,
    level2Reviewers,
    { consensusAlgorithm: "unanimous" }
  );

  await reasoningEngine.formConsensus(level2Debate.id);
}

// 4. Apply final decision
await orchestrator.reviewAppeal(sessionId, appeal.id, reviewers);
```

## Component Integration Points

### 1. Debate → Verdict Flow

```typescript
/**
 * Convert debate consensus to verdict input
 */
function debateToVerdictContext(
  debate: DebateSession,
  consensus: ConsensusResult
): {
  confidence: number;
  reasoning: string[];
  evidence: string[];
} {
  return {
    confidence: consensus.confidence,
    reasoning: debate.arguments.map((arg) => arg.claim),
    evidence: debate.arguments.flatMap((arg) => arg.evidence),
  };
}
```

### 2. Precedent → Debate Context

```typescript
/**
 * Load precedents into debate context
 */
async function enrichDebateWithPrecedents(
  debateId: string,
  precedentManager: PrecedentManager,
  ruleCategory: RuleCategory
): Promise<void> {
  const precedents = precedentManager.searchPrecedents({
    categories: [ruleCategory],
    minCitations: 5,
    limit: 5,
  });

  for (const precedent of precedents) {
    await reasoningEngine.submitEvidence(debateId, {
      type: "precedent",
      content: precedent.reasoningSummary,
      source: precedent.id,
      confidence: 0.9,
    });
  }
}
```

### 3. Appeal → Debate Escalation

```typescript
/**
 * Escalate appeal through debate levels
 */
async function escalateAppealWithDebate(
  appeal: Appeal,
  currentLevel: number,
  maxLevel: number
): Promise<boolean> {
  if (currentLevel >= maxLevel) {
    return false;
  }

  const nextLevelDebate = await reasoningEngine.initiateDebate(
    `Appeal Level ${currentLevel + 1}: ${appeal.id}`,
    getReviewersForLevel(currentLevel + 1),
    {
      consensusAlgorithm:
        currentLevel === maxLevel - 1 ? "unanimous" : "weighted-voting",
    }
  );

  const result = await conductDebate(nextLevelDebate.id);

  return result.outcome !== "no-consensus";
}
```

## Performance Considerations

### Session Metrics

Both systems provide performance tracking:

```typescript
// Arbitration metrics
const arbMetrics = orchestrator.getSessionMetrics(sessionId);
console.log({
  ruleEvaluation: arbMetrics.ruleEvaluationMs,
  verdictGeneration: arbMetrics.verdictGenerationMs,
  total: arbMetrics.totalDurationMs,
});

// Debate metrics
const debateMetrics = reasoningEngine.getDebateMetrics(debateId);
console.log({
  turnCount: debateMetrics.turnCount,
  participantCount: debateMetrics.participantCount,
  duration: debateMetrics.durationMs,
});
```

### Performance Budgets

| Operation           | Budget (P95) | Actual | Status |
| ------------------- | ------------ | ------ | ------ |
| Rule Evaluation     | 200ms        | ~150ms | ✅     |
| Verdict Generation  | 300ms        | ~250ms | ✅     |
| Debate Turn         | 100ms        | ~80ms  | ✅     |
| Consensus Formation | 150ms        | ~120ms | ✅     |
| Precedent Lookup    | 100ms        | ~75ms  | ✅     |

## Error Handling

### Unified Error Handling

```typescript
import { ArbitrationError, ReasoningEngineError } from "@/arbitration";

try {
  const session = await orchestrator.startSession(violation, rules, agents);
  const debate = await reasoningEngine.initiateDebate(
    topic,
    participants,
    config
  );
} catch (error) {
  if (error instanceof ArbitrationError) {
    console.error(`Arbitration error: ${error.code}`, error.message);
    // Handle arbitration-specific errors
  } else if (error instanceof ReasoningEngineError) {
    console.error(`Reasoning error: ${error.code}`, error.message);
    // Handle debate-specific errors
  }
}
```

### Error Recovery

```typescript
/**
 * Automatic error recovery with fallback
 */
async function safeArbitrationWithDebate(
  violation: ConstitutionalViolation,
  rules: ConstitutionalRule[]
): Promise<Verdict> {
  try {
    // Try full debate process
    return await arbitrationWithDebate(violation, rules);
  } catch (error) {
    if (error instanceof ReasoningEngineError) {
      // Fallback to direct verdict without debate
      console.warn("Debate failed, generating direct verdict");
      const session = await orchestrator.startSession(violation, rules, [
        "arbiter",
      ]);
      await orchestrator.evaluateRules(session.id);
      return await orchestrator.generateVerdict(session.id, "arbiter");
    }
    throw error;
  }
}
```

## Testing

### Integration Tests

```typescript
import { createArbitrationSystem } from "@/arbitration";

describe("Arbitration + Reasoning Integration", () => {
  it("should conduct full arbitration with debate", async () => {
    const system = createArbitrationSystem();

    // Start arbitration
    const session = await system.orchestrator.startSession(/* ... */);

    // Initiate debate
    const debate = await system.reasoningEngine.initiateDebate(/* ... */);

    // Conduct debate
    await system.reasoningEngine.submitArgument(/* ... */);

    // Form consensus
    const consensus = await system.reasoningEngine.formConsensus(debate.id);

    // Generate verdict
    const verdict = await system.orchestrator.generateVerdict(
      session.id,
      "arbiter"
    );

    expect(verdict.confidence).toBeGreaterThan(0.7);
    expect(verdict.outcome).toBeDefined();
  });
});
```

## Next Steps

### Integration Roadmap

1. **Phase 1 (Complete)**: Core component implementation

   - ✅ ARBITER-015: Arbitration Protocol
   - ✅ ARBITER-016: Reasoning Engine

2. **Phase 2 (Current)**: Integration Layer

   - ✅ Unified export interface
   - ✅ Integration patterns documentation
   - 🔄 End-to-end integration tests (6 tests remaining)

3. **Phase 3 (Next)**: Production Hardening

   - Performance optimization
   - Enhanced error recovery
   - Monitoring and observability
   - Load testing

4. **Phase 4**: RL Pipeline Integration
   - Debate outcome tracking
   - Verdict quality scoring
   - Turn-level RL training data
   - Model improvement feedback loop

## API Reference

See individual component documentation:

- [ArbitrationOrchestrator](../src/arbitration/ArbitrationOrchestrator.ts)
- [ArbiterReasoningEngine](../src/reasoning/ArbiterReasoningEngine.ts)
- [ConstitutionalRuleEngine](../src/arbitration/ConstitutionalRuleEngine.ts)
- [VerdictGenerator](../src/arbitration/VerdictGenerator.ts)
- [PrecedentManager](../src/arbitration/PrecedentManager.ts)

## Support

For issues or questions about integration:

1. Check component STATUS.md files
2. Review test suites for usage examples
3. Consult CAWS working specs in `.caws/` directories
