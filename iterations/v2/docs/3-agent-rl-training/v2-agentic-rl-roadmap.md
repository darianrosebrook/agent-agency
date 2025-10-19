# Agent Agency V2: Agentic RL & Extended Thinking

> **Document Type**: Roadmap & Planning Document  
> **Status**: Describes planned enhancements and aspirational capabilities  
> **Implementation Status**: See [COMPONENT_STATUS_INDEX.md](../../COMPONENT_STATUS_INDEX.md) for actual completion  
> **Current Reality**: 68% complete - Some enhancements described here are not yet implemented

## Risk Tier 2 Enhancement Plan

**Version**: 2.0.0
**Risk Tier**: 🟡 T2 (Features, APIs, data writes)
**Quality Gates**: 80% coverage, 50% mutation, contracts required

---

## Executive Summary

Building on the successful POC (V1) foundation, V2 incorporates advanced agentic reinforcement learning, extended thinking budgets, and practical reliability measures. This enhancement addresses the industry's shift from pure reasoning benchmarks to reliable multi-turn agents that can act safely across tool interactions.

**Key Insights Applied**: Anthropic's "extended thinking" positioning, reward hacking prevention, multi-turn tool learning, and academic-grade evaluation practices.

---

## 1. Extended Thinking as a Budgeted Resource

### Current State (V1)

- Basic iteration loops with satisficing logic
- Fixed evaluation timeouts
- No token utilization tracking

### V2 Enhancement: Thinking Budget Manager

**Core Concept**: Treat thinking as an optimizable resource, not a binary toggle.

```typescript
interface ThinkingBudget {
  taskComplexity: "trivial" | "standard" | "complex";
  allocatedTokens: number;
  consumedTokens: number;
  efficiency: number;
  escalationTriggers: BudgetTrigger[];
}

interface BudgetTrigger {
  condition: "low-confidence" | "partial-success" | "verifier-rejection";
  additionalTokens: number;
  maxTotalBudget: number;
}
```

**Implementation Strategy**:

1. **Adaptive Budget Allocation**:

   - Trivial tasks: 500 tokens max
   - Standard tasks: 2,000 tokens max
   - Complex tasks: 8,000 tokens max

2. **Runtime Budget Management**:

   - Track token consumption per evaluation iteration
   - Escalate budgets based on confidence thresholds
   - Prevent infinite thinking loops with hard ceilings

3. **Observability & Optimization**:
   - Log thinking efficiency metrics
   - A/B test budget allocation curves
   - Optimize for cost/latency tradeoffs

**Risk Mitigation**: Feature flags allow instant rollback to V1 behavior if thinking budgets cause performance issues.

---

## 2. Reward Hacking Prevention & Minimal-Diff Enforcement

### Current State (V1)

- Basic quality gates (tests, lint, types)
- No AST analysis or diff measurement
- Reward signals based only on binary pass/fail

### V2 Enhancement: Minimal-Diff Evaluator

**Core Problem**: Models "cover their bases" with unnecessary scaffolding, extra files, or over-engineering to pass tests.

**Solution**: AST-based minimal-diff analysis with reward penalties.

```typescript
interface MinimalDiffCriteria {
  astSimilarity: number; // Tree-edit distance
  fileTouchCount: number; // Files modified
  lineChangeRatio: number; // Changed vs total lines
  scaffoldingPenalty: number; // Unnecessary abstractions
  rewardMultiplier: number; // 0.1 - 1.0 based on minimality
}
```

**Implementation Components**:

1. **AST Analysis Engine**:

   - Compare before/after code trees
   - Measure functional equivalence vs structural changes
   - Penalize cosmetic vs functional modifications

2. **Diff Quality Metrics**:

   - File touch budgets per task type
   - Line change efficiency ratios
   - Scaffolding detection (extra helpers, interfaces, types)

3. **Reward Signal Integration**:
   - Multiply evaluation scores by minimality factor
   - Log reward hacking incidents for analysis
   - Progressive penalties for repeated offenses

**Expected Impact**: Reduce "spray edits" by 60-80%, improve code maintainability.

---

## 3. Turn-Level RL for Multi-Turn Tool Use

### Current State (V1)

- Basic tool execution with error handling
- No credit assignment for tool choice quality
- Token-level rewards only

### V2 Enhancement: Agentic RL Trainer

**Core Concept**: Treat each dialogue/tool turn as an action with intermediate rewards.

**Architecture**:

```typescript
interface TurnLevelReward {
  turnNumber: number;
  toolChoice: ToolCall;
  informationGain: number; // Relevance of retrieved data
  formatCorrectness: number; // JSON/schema compliance
  taskProgress: number; // Steps closer to completion
  totalReward: number;
}

class AgenticRLTrainer {
  async trainOnConversation(conversation: Conversation): Promise<ModelUpdate> {
    const turnRewards = await this.computeTurnRewards(conversation);
    const policyUpdate = await this.updatePolicy(turnRewards);
    return policyUpdate;
  }
}
```

**Implementation Strategy**:

1. **Intermediate Reward Computation**:

   - **Tool Relevance**: LLM judge scores information utility
   - **Format Compliance**: Schema validation + error affordance
   - **Progress Measurement**: Task completion percentage

2. **Credit Assignment**:

   - GRPO-style group comparisons for multi-turn trajectories
   - Separate rewards for tool choice vs execution quality
   - Temporal credit propagation for long-horizon tasks

3. **Training Data Collection**:
   - Log all multi-turn conversations with outcome labels
   - Anonymize tenant data for cross-project learning
   - Progressive difficulty curriculum

**Training Approach**: Start with SFT on correct tool usage patterns, then RL fine-tuning with turn-level rewards.

---

## 4. Enhanced Evaluation System with Model-Based Judges

### Current State (V1)

- Rule-based evaluators for code/text/design
- Binary pass/fail criteria
- No model-based assessment

### V2 Enhancement: Intelligent Evaluation Orchestrator

**Core Concept**: Use local AI models as judges for properties that are hard to rule-check.

```typescript
interface ModelBasedJudge {
  judgeType: "relevance" | "faithfulness" | "minimality" | "safety";
  prompt: string;
  responseSchema: JSONSchema;
  confidenceThreshold: number;
}

class IntelligentEvaluator extends BaseEvaluator {
  async evaluateWithModelJudge(artifact: Artifact): Promise<EvaluationReport> {
    const modelJudgments = await this.collectModelJudgments(artifact);
    const ruleBasedChecks = await this.runRuleChecks(artifact);
    return this.combineJudgments(modelJudgments, ruleBasedChecks);
  }
}
```

**Judgment Types**:

1. **Faithfulness**: Does output match ground truth without hallucination?
2. **Relevance**: Does retrieved information actually help the task?
3. **Minimality**: Is the solution the simplest correct one?
4. **Safety**: Does the output avoid harmful or insecure patterns?

**Integration**: Model judges complement rule-based checks, especially for creative or subjective criteria.

---

## 5. Tool Learning Warmup & Intermediate Rewards

### Current State (V1)

- Tools available but no explicit training incentives
- Small models often ignore tools or use them incorrectly

### V2 Enhancement: Tool Adoption Framework

**Problem**: Small/vanilla models won't use tools unless explicitly incentivized.

**Solution**: Two-phase approach:

1. **SFT Warmup Phase**:

   - Supervised fine-tuning on correct tool usage examples
   - Include both successful and failed tool call patterns
   - Teach proper JSON formatting and error handling

2. **RL with Intermediate Rewards**:
   - Reward valid tool call structure (even if result is suboptimal)
   - Separate credit for tool choice vs tool execution quality
   - Penalize dummy calls that don't contribute to task progress

```typescript
interface ToolRewardSignal {
  callStructureValid: boolean; // Correct JSON format
  toolChoiceAppropriate: boolean; // Right tool for the job
  informationUtility: number; // How useful was the result
  errorHandlingCorrect: boolean; // Proper fallback behavior
}
```

**Expected Outcome**: 3-5x improvement in tool adoption rates for smaller models.

---

## Implementation Roadmap

### Phase 1: Foundation (Weeks 1-3)

- [ ] Create ThinkingBudgetManager
- [ ] Implement basic AST diff analysis
- [ ] Add turn-level reward logging

### Phase 2: Core RL Features (Weeks 4-7)

- [ ] Build AgenticRLTrainer with GRPO-style updates
- [ ] Implement minimal-diff evaluator
- [ ] Add model-based judges to evaluation system

### Phase 3: Tool Learning (Weeks 8-10)

- [ ] SFT warmup pipeline for tool usage
- [ ] Intermediate reward computation
- [ ] Tool adoption metrics and monitoring

### Phase 4: Production Integration (Weeks 11-12)

- [ ] Performance benchmarking
- [ ] A/B testing framework
- [ ] Rollback procedures and feature flags

---

## Quality Assurance Strategy

### Coverage Requirements (Tier 2)

- **Branch Coverage**: ≥80%
- **Mutation Score**: ≥50%
- **Contract Tests**: Required for RL and evaluation APIs

### Key Test Scenarios

1. **Thinking Budget Tests**: Verify budget allocation and escalation
2. **Minimal-Diff Tests**: Ensure reward penalties work correctly
3. **Turn-Level RL Tests**: Validate credit assignment in multi-turn scenarios
4. **Tool Adoption Tests**: Measure improvement in tool usage rates

### Performance Budgets

- API P95: 500ms (increased for RL inference)
- Thinking Budget Max: 10 seconds
- Tool Call P95: 200ms

---

## Risk Mitigation

### Technical Risks

1. **RL Training Instability**: Feature flags allow instant disable
2. **Thinking Budget Exhaustion**: Hard ceilings prevent infinite loops
3. **Reward Hacking Regression**: Fallback to V1 evaluation criteria

### Operational Risks

1. **Performance Impact**: Comprehensive benchmarking before production
2. **Tenant Data Privacy**: Strict anonymization for RL training data
3. **Backward Compatibility**: V1 workflows remain functional

### Rollback Strategy

1. Feature flags disable all V2 enhancements
2. Configuration reverts to V1 defaults
3. Database migrations are non-destructive

---

## Success Metrics

### Quantitative Targets

- **Tool Adoption Rate**: +300% for small models
- **Thinking Efficiency**: -40% token waste on trivial tasks
- **Reward Hacking Incidents**: -70% reduction
- **Task Completion Rate**: +25% for complex multi-turn tasks

### Qualitative Improvements

- More reliable agent behavior in edge cases
- Reduced unnecessary code changes
- Better tool integration in conversations
- Improved evaluation accuracy for subjective criteria

---

## Dependencies & Prerequisites

### Required V1 Features

- ✅ Multi-tenant memory system
- ✅ MCP server with tool management
- ✅ Evaluation orchestrator foundation
- ✅ Agent orchestrator with task routing

### New Dependencies

- Local RL training infrastructure
- AST parsing libraries for diff analysis
- Model-based judgment pipelines
- Enhanced observability for RL metrics

---

## Future Extensions (V3 Considerations)

Based on V2 learnings, future versions could explore:

- Federated RL across multiple agent deployments
- Hierarchical thinking budgets for complex task decomposition
- Multi-modal evaluation (code + visual feedback)
- Cross-tenant RL with differential privacy guarantees

---

_This V2 enhancement plan transforms the POC foundation into a production-ready agentic RL system, incorporating industry best practices from Anthropic's extended thinking work and practical reliability measures for multi-turn tool use._


