# Arbiter Architecture: CAWS Constitutional Authority

**Author**: @darianrosebrook

---

## Executive Summary

The Arbiter is the central orchestration component of Agent Agency V2, serving as the CAWS constitutional authority that coordinates multiple AI models and tools while ensuring all work complies with budgets, waivers, quality gates, and provenance requirements.

**Core Principle**: CAWS becomes the executable contract that governs all AI contributions. The arbiter interprets CAWS clauses as system calls, verifies compliance before merge, and records immutable provenance.

---

## Arbiter Responsibilities

### 1. Constitutional Authority

The arbiter acts as the runtime enforcer of CAWS policies that no worker model can bypass:

- **Budget Enforcement**: Validates all work against `max_files` and `max_loc` limits
- **Quality Gate Validation**: Ensures tests, lints, coverage thresholds are met
- **Waiver Interpretation**: Evaluates exception requests against policy rules
- **Provenance Recording**: Maintains immutable audit trail of all decisions

### CAWS CLI Interface Contract

The arbiter integrates with CAWS tooling via local CLI commands:

| Command                  | Triggered By             | Purpose                       | Output              | Spec Reference                  |
| ------------------------ | ------------------------ | ----------------------------- | ------------------- | ------------------------------- |
| `caws verify`            | Arbiter before verdict   | Runs gates/tests on task      | `verdict.yaml`      | `api/caws-integration.api.yaml` |
| `caws waiver create`     | Arbiter on budget breach | Creates waiver for approval   | `WV-*.yaml`         | `api/caws-integration.api.yaml` |
| `caws audit self`        | CI bootstrap or manual   | Validates Arbiter code        | `SELF-VERDICT.yaml` | `api/caws-integration.api.yaml` |
| `caws provenance record` | After task completion    | Records immutable audit trail | Provenance hash     | `api/caws-integration.api.yaml` |

**See Also**: `api/caws-integration.api.yaml` for complete CLI contract specification.

### 2. Intelligent Agent Orchestration

Routes tasks to optimal agents using performance-based selection:

- **Multi-Armed Bandit Routing**: Balance exploration (try new agents) vs exploitation (use proven agents)
- **Capability Matching**: Route tasks based on agent specialization
- **Load Distribution**: Balance workload across available agents
- **Conflict Resolution**: Handle disagreements through debate or voting

### 3. Performance Tracking & Learning

Generates training data for RL pipeline:

- **Success/Failure Logging**: Capture every task outcome
- **Quality Scoring**: Track evaluation metrics per agent
- **Efficiency Metrics**: Monitor latency, token usage, tool adoption
- **Trend Analysis**: Identify improving/degrading agent performance

---

## Architecture Components

### Agent Registry Manager

> **Implementation Status**: ✅ **COMPLETE AND TESTED**  
> **Specification**: `components/agent-registry-manager/.caws/working-spec.yaml` (ARBITER-001)  
> **Code**: `src/orchestrator/AgentRegistryManager.ts`  
> **Types**: `src/types/agent-registry.ts`  
> **Tests**: `tests/unit/orchestrator/agent-registry-manager.test.ts` (20/20 passing)

**Purpose**: Maintain agent catalog with capability profiles

**Implemented Interface** (see `src/types/agent-registry.ts`):

```typescript
interface AgentProfile {
  id: string;
  name: string;
  modelFamily: string;
  capabilities: {
    taskTypes: string[]; // ["code-editing", "research", etc.]
    languages: string[]; // ["TypeScript", "Python", etc.]
    specializations: string[]; // ["AST analysis", "API design", etc.]
  };
  performanceHistory: {
    successRate: number; // 0-1
    averageQuality: number; // 0-1
    averageLatency: number; // milliseconds
    taskCount: number;
  };
  currentLoad: {
    activeTasks: number;
    queuedTasks: number;
    utilizationPercent: number;
  };
  registeredAt: Date;
  lastActiveAt: Date;
}
```

**Implemented Class** (see `src/orchestrator/AgentRegistryManager.ts`):

```typescript
class AgentRegistryManager {
  async registerAgent(agent: Agent): Promise<AgentProfile> {
    // ✅ IMPLEMENTED - See src/orchestrator/AgentRegistryManager.ts:60-94
    // Validates, initializes capability tracking, returns profile
  }

  async getAgentsByCapability(query: AgentQuery): Promise<AgentQueryResult[]> {
    // ✅ IMPLEMENTED - See src/orchestrator/AgentRegistryManager.ts:125-196
    // Filters by capabilities, sorts by success rate, returns with match scores
  }

  async updatePerformance(
    agentId: string,
    metrics: PerformanceMetrics
  ): Promise<AgentProfile> {
    // ✅ IMPLEMENTED - See src/orchestrator/AgentRegistryManager.ts:209-244
    // Uses incremental averaging: newAvg = oldAvg + (newValue - oldAvg) / (count + 1)
  }
}
```

**Production-Ready Features**:

- ✅ All acceptance criteria (A1-A5) implemented and tested
- ✅ Performance: <50ms P95 for queries (measured at ~1ms)
- ✅ Scalability: 1000 agents, 2000 queries/sec
- ✅ Database schema: `migrations/001_create_agent_registry_tables.sql`
- ✅ Complete test suite: 20 unit tests, 100% pass rate

### Task Routing Manager

> **Implementation Status**: 📋 Specification complete, implementation planned  
> **Specification**: `components/task-routing-manager/.caws/working-spec.yaml` (ARBITER-002)  
> **Dependencies**: Requires ARBITER-001 ✅

**Purpose**: Intelligent task-to-agent assignment

```typescript
interface RoutingDecision {
  taskId: string;
  selectedAgent: string;
  routingStrategy: "multi-armed-bandit" | "capability-match" | "load-balance";
  confidence: number;
  alternativesConsidered: Array<{
    agentId: string;
    score: number;
    reason: string;
  }>;
  rationale: string;
}

class TaskRoutingManager {
  async routeTask(task: Task): Promise<RoutingDecision> {
    // Get candidate agents
    const candidates = await this.agentRegistry.getAgentsByCapability(
      task.type,
      task.requirements
    );

    // Apply multi-armed bandit selection
    const selectedAgent = await this.multiArmedBandit.select(candidates, task);

    // Log decision for RL training
    const decision: RoutingDecision = {
      taskId: task.id,
      selectedAgent: selectedAgent.id,
      routingStrategy: "multi-armed-bandit",
      confidence: this.calculateConfidence(selectedAgent, task),
      alternativesConsidered: candidates.map((c) => ({
        agentId: c.id,
        score: this.scoreAgent(c, task),
        reason: this.explainScore(c, task),
      })),
      rationale: `Selected ${selectedAgent.name} with ${(
        selectedAgent.performanceHistory.successRate * 100
      ).toFixed(1)}% success rate for ${task.type} tasks`,
    };

    await this.performanceTracker.logRoutingDecision(decision);

    return decision;
  }
}
```

### Multi-Armed Bandit Implementation

> **Implementation Status**: 📋 Specification complete, implementation planned  
> **Specification**: Part of ARBITER-002  
> **Algorithm**: Epsilon-greedy with UCB scoring

**Purpose**: Balance trying new agents vs using proven ones

```typescript
interface BanditConfig {
  explorationRate: number; // 0-1, higher = more exploration
  decayFactor: number; // Reduce exploration over time
  minSampleSize: number; // Minimum attempts before trusting stats
}

class MultiArmedBandit {
  private config: BanditConfig = {
    explorationRate: 0.2, // 20% exploration
    decayFactor: 0.995,
    minSampleSize: 10,
  };

  async select(agents: AgentProfile[], task: Task): Promise<AgentProfile> {
    // Epsilon-greedy strategy
    const epsilon =
      this.config.explorationRate *
      Math.pow(this.config.decayFactor, this.totalTasks);

    if (Math.random() < epsilon) {
      // Explore: try random agent or underutilized one
      return this.selectForExploration(agents);
    } else {
      // Exploit: use best performing agent
      return this.selectForExploitation(agents, task);
    }
  }

  private selectForExploitation(
    agents: AgentProfile[],
    task: Task
  ): AgentProfile {
    // Upper Confidence Bound (UCB) scoring
    return agents
      .map((agent) => ({
        agent,
        score: this.calculateUCB(agent, task),
      }))
      .sort((a, b) => b.score - a.score)[0].agent;
  }

  private calculateUCB(agent: AgentProfile, task: Task): number {
    const successRate = agent.performanceHistory.successRate;
    const taskCount = agent.performanceHistory.taskCount;

    // UCB formula: mean + exploration bonus
    const explorationBonus = Math.sqrt(
      (2 * Math.log(this.totalTasks)) / (taskCount + 1)
    );

    return successRate + explorationBonus;
  }
}
```

### CAWS Validator

> **Implementation Status**: 📋 Specification complete, implementation planned  
> **Specification**: `components/caws-validator/.caws/working-spec.yaml` (ARBITER-003)  
> **Risk Tier**: 1 (Critical) - Requires manual review

**Purpose**: Enforce constitutional rules

```typescript
interface CAWSValidationResult {
  compliant: boolean;
  violations: CAWSViolation[];
  waiverRequired: boolean;
  budgetStatus: BudgetStatus;
  qualityGateResults: QualityGateResult[];
}

class CAWSValidator {
  async validateTaskResult(
    task: Task,
    result: TaskResult,
    workingSpec: WorkingSpec
  ): Promise<CAWSValidationResult> {
    const violations: CAWSViolation[] = [];

    // Budget validation
    if (result.filesChanged > workingSpec.change_budget.max_files) {
      violations.push({
        type: "budget",
        message: `${result.filesChanged} files changed, exceeds budget of ${workingSpec.change_budget.max_files}`,
        severity: "error",
      });
    }

    if (result.linesChanged > workingSpec.change_budget.max_loc) {
      violations.push({
        type: "budget",
        message: `${result.linesChanged} lines changed, exceeds budget of ${workingSpec.change_budget.max_loc}`,
        severity: "error",
      });
    }

    // Quality gate validation
    const qualityGates = await this.runQualityGates(result, workingSpec);
    const failedGates = qualityGates.filter((g) => !g.passed);

    for (const gate of failedGates) {
      violations.push({
        type: "quality-gate",
        message: `Quality gate '${gate.name}' failed: ${gate.reason}`,
        severity: gate.mandatory ? "error" : "warning",
      });
    }

    return {
      compliant: violations.filter((v) => v.severity === "error").length === 0,
      violations,
      waiverRequired: violations.some((v) => v.waiverEligible),
      budgetStatus: this.computeBudgetStatus(result, workingSpec),
      qualityGateResults: qualityGates,
    };
  }
}
```

### Performance Tracker

> **Implementation Status**: 📋 Specification complete, implementation planned  
> **Specification**: `performance-tracker/.caws/working-spec.yaml` (ARBITER-004)  
> **Data Volume**: 10,000 points/day, 100GB capacity

**Purpose**: Collect data for RL training

```typescript
class PerformanceTracker {
  async logRoutingDecision(decision: RoutingDecision): Promise<void> {
    await this.benchmarkCollector.record({
      type: "routing-decision",
      timestamp: new Date(),
      data: decision,
    });
  }

  async logTaskExecution(
    taskId: string,
    agentId: string,
    metrics: ExecutionMetrics
  ): Promise<void> {
    await this.benchmarkCollector.record({
      type: "task-execution",
      timestamp: new Date(),
      data: {
        taskId,
        agentId,
        success: metrics.success,
        latencyMs: metrics.latencyMs,
        tokensUsed: metrics.tokensUsed,
        qualityScore: metrics.qualityScore,
      },
    });
  }

  async logEvaluationOutcome(
    taskId: string,
    evaluation: EvaluationResult
  ): Promise<void> {
    await this.benchmarkCollector.record({
      type: "evaluation-outcome",
      timestamp: new Date(),
      data: {
        taskId,
        passed: evaluation.passed,
        score: evaluation.score,
        rubricScores: evaluation.rubricScores,
        minimalDiffMetrics: evaluation.diffMetrics,
        cawsCompliant: evaluation.cawsCompliant,
      },
    });
  }
}
```

---

## Documents in This Section

- **theory.md** - Comprehensive arbiter stack requirements and research background
- **arbiter-architecture.md** - This document: Concrete implementation architecture
- **intelligent-routing.md** - Task routing algorithms and multi-armed bandit details
- **performance-tracking.md** - Data collection strategy for RL training
- **implementation-roadmap.md** - Development timeline for orchestration layer

---

## Next Steps

1. Read **theory.md** for research background and requirements
2. Review **arbiter-architecture.md** (this doc) for implementation details
3. See **intelligent-routing.md** for routing algorithm specifics
4. Check **performance-tracking.md** for RL data collection strategy
5. Follow **implementation-roadmap.md** for development plan

---

**The arbiter is more than an orchestrator—it's the constitutional authority that ensures quality while generating the data needed for continuous system improvement.**
