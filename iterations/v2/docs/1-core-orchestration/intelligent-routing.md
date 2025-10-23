# Intelligent Routing: Multi-Armed Bandit & Capability Matching

**Author**: @darianrosebrook

---

## Executive Summary

Intelligent routing is the core decision-making component of the arbiter orchestrator. It uses multi-armed bandit algorithms, capability matching, and performance history to route tasks to the optimal agent while continuously learning which routing strategies work best.

**Key Innovation**: Every routing decision is both an immediate optimization (pick the best agent now) and a learning opportunity (gather data for future improvements).

---

## POC Validation: Multi-Turn Feedback & Model Performance

Our POC validated critical insights that inform V2's routing intelligence:

### Multi-Turn Iteration Success

In our POC, we found that multi-turn feedback loops achieved 100% success rates for text transformation tasks. When implementing iterative learning, we discovered that:

- **Optimal iteration count**: Agents averaged 2 of 3 maximum iterations before reaching quality thresholds
- **Iteration success predictors**: Tasks with specific, actionable feedback (e.g., "Remove banned phrases like 'hey team'") improved faster than vague feedback
- **Context preservation**: Maintaining full conversation history across iterations was critical for coherent improvements
- **Early stopping value**: Quality-based early stopping saved ~33% of potential computation time

**Routing Implication**: V2 should route iterative tasks to agents with proven multi-turn success rates, tracked separately from single-turn success metrics.

### Model Performance Characteristics

Benchmark testing validated that model selection significantly impacts routing decisions. Our POC tested multiple Gemma variants:

| Model           | Tokens/Sec | Response Time | Quality | Optimal Use Case                      |
| --------------- | ---------- | ------------- | ------- | ------------------------------------- |
| **gemma3n:e2b** | 36.02      | 9.4s          | 8.5/10  | **Balanced workflows** (selected)     |
| gemma3:1b       | 72.18      | 2.2s          | 6.2/10  | High-speed, low-complexity tasks      |
| gemma3n:e4b     | 23.83      | 5.3s          | 9.1/10  | Quality-critical, time-flexible tasks |

**Key Finding**: The POC demonstrated that **gemma3n:e2b provides the optimal balance** for autonomous agent workflows, delivering 8.5/10 quality at 36 tokens/sec. While gemma3:1b was 3x faster, its quality (6.2/10) proved insufficient for iterative learning. The gemma3n:e4b delivered higher quality but the speed tradeoff (23.83 vs 36 tokens/sec) didn't justify the improvement for most tasks.

**Routing Implication**: V2's multi-armed bandit should track per-model performance for different task types. Simple transformations might benefit from faster models, while complex generation tasks may justify quality-optimized models.

### Task-Specific Performance Patterns

POC telemetry revealed distinct performance profiles by task type:

- **Text transformation**: 2.1s average, 100% success rate, 2/3 iterations
- **Code generation**: 25s average, 80% success rate (4/5 tests), 1/1 iterations
- **Design token application**: 52s average, timeout issues, needs optimization

**Routing Implication**: V2 should maintain separate performance histories for each task type-agent combination, as performance is not transferable across domains.

### Iteration Strategy Learnings

When implementing multi-turn feedback, we discovered:

1. **Feedback specificity matters**: Mock error injection with precise feedback ("Remove phrases: 'hey team', 'really casual'") achieved 100% improvement vs generic feedback ("improve tone") at ~60%
2. **Iteration limits prevent runaway**: 3-iteration limit proved optimal, preventing diminishing returns
3. **Quality thresholds work**: Early success detection (>85% quality score) saved computation in 33% of cases
4. **Context overhead**: Full history preservation added ~200ms per iteration but was essential for coherence

**Routing Implication**: Route iteration-heavy tasks to agents with proven feedback processing capabilities, tracked as a separate performance dimension.

---

## Routing Strategies

### 1. Multi-Armed Bandit (Primary Strategy)

**Purpose**: Balance exploration (try different agents) vs exploitation (use proven agents)

**Algorithm**: Epsilon-Greedy with Upper Confidence Bound (UCB)

```typescript
interface BanditState {
  // Per agent-task pair
  agentTaskPairs: Map<
    string,
    {
      attempts: number;
      successes: number;
      avgQuality: number;
      avgLatency: number;
    }
  >;

  // Global state
  totalAttempts: number;
  explorationRate: number;
  decayFactor: number;
}

class MultiArmedBanditRouter {
  async selectAgent(
    candidates: AgentProfile[],
    task: Task,
    state: BanditState
  ): Promise<RoutingDecision> {
    // Compute exploration probability (decreases over time)
    const epsilon =
      state.explorationRate * Math.pow(state.decayFactor, state.totalAttempts);

    if (Math.random() < epsilon) {
      // EXPLORE: Try underutilized agent
      return this.explore(candidates, task, state);
    } else {
      // EXPLOIT: Use best performing agent
      return this.exploit(candidates, task, state);
    }
  }

  private exploit(
    candidates: AgentProfile[],
    task: Task,
    state: BanditState
  ): RoutingDecision {
    // Upper Confidence Bound scoring
    const scored = candidates.map((agent) => {
      const pairKey = `${agent.id}:${task.type}`;
      const history = state.agentTaskPairs.get(pairKey) ?? {
        attempts: 0,
        successes: 0,
        avgQuality: 0.5,
        avgLatency: 0,
      };

      // UCB formula: mean + confidence interval
      const successRate =
        history.attempts > 0 ? history.successes / history.attempts : 0.5;

      const confidenceBonus = Math.sqrt(
        (2 * Math.log(state.totalAttempts + 1)) / (history.attempts + 1)
      );

      const ucbScore = successRate + confidenceBonus;

      return {
        agent,
        score: ucbScore,
        successRate,
        confidenceBonus,
        reason: `UCB: ${ucbScore.toFixed(3)} (success: ${(
          successRate * 100
        ).toFixed(1)}%, bonus: ${confidenceBonus.toFixed(3)})`,
      };
    });

    // Sort by score and select best
    scored.sort((a, b) => b.score - a.score);
    const selected = scored[0];

    return {
      selectedAgent: selected.agent.id,
      strategy: "multi-armed-bandit-exploit",
      confidence: selected.successRate,
      alternativesConsidered: scored.slice(1, 4),
      rationale: selected.reason,
    };
  }

  private explore(
    candidates: AgentProfile[],
    task: Task,
    state: BanditState
  ): RoutingDecision {
    // Select least-tried agent for this task type
    const sorted = candidates
      .map((agent) => {
        const pairKey = `${agent.id}:${task.type}`;
        const history = state.agentTaskPairs.get(pairKey);
        return {
          agent,
          attempts: history?.attempts ?? 0,
        };
      })
      .sort((a, b) => a.attempts - b.attempts);

    const selected = sorted[0];

    return {
      selectedAgent: selected.agent.id,
      strategy: "multi-armed-bandit-explore",
      confidence: 0.5, // Unknown, exploring
      alternativesConsidered: sorted.slice(1, 4).map((s) => ({
        agentId: s.agent.id,
        score: s.attempts,
        reason: `Attempted ${s.attempts} times`,
      })),
      rationale: `Exploration: Agent attempted only ${selected.attempts} times for ${task.type} tasks`,
    };
  }
}
```

### 2. Capability-Based Routing (Fallback)

**Purpose**: Direct matching when agent capabilities are explicit

```typescript
class CapabilityMatcher {
  async matchAgentToTask(
    task: Task,
    agents: AgentProfile[]
  ): Promise<AgentProfile[]> {
    return agents
      .filter((agent) => this.hasRequiredCapabilities(agent, task))
      .map((agent) => ({
        agent,
        matchScore: this.computeMatchScore(agent, task),
      }))
      .sort((a, b) => b.matchScore - a.matchScore)
      .map((m) => m.agent);
  }

  private computeMatchScore(agent: AgentProfile, task: Task): number {
    let score = 0;

    // Task type match
    if (agent.capabilities.taskTypes.includes(task.type)) {
      score += 0.4;
    }

    // Language match
    const languageMatches = task.requirements.filter((req) =>
      agent.capabilities.languages.includes(req)
    ).length;
    score += (languageMatches / task.requirements.length) * 0.3;

    // Specialization match
    const specializationMatches = task.requirements.filter((req) =>
      agent.capabilities.specializations.includes(req)
    ).length;
    score += (specializationMatches / task.requirements.length) * 0.3;

    return score;
  }
}
```

### 3. Load-Based Routing (Performance Optimization)

**Purpose**: Distribute workload evenly

```typescript
class LoadBalancer {
  async selectLeastLoaded(candidates: AgentProfile[]): Promise<AgentProfile> {
    return candidates
      .map((agent) => ({
        agent,
        load:
          agent.currentLoad.activeTasks + agent.currentLoad.queuedTasks * 0.5,
      }))
      .sort((a, b) => a.load - b.load)[0].agent;
  }

  async shouldRejectTask(agent: AgentProfile): Promise<boolean> {
    const maxConcurrent = 10; // Configuration
    return agent.currentLoad.activeTasks >= maxConcurrent;
  }
}
```

---

## Routing Decision Framework

### Decision Process

```mermaid
graph TB
    START[Task Arrives] --> COMPLEX{Task Complexity<br/>Assessment}

    COMPLEX --> |Trivial| FAST[Fast Agent Pool]
    COMPLEX --> |Standard| CAPABLE[Capable Agent Pool]
    COMPLEX --> |Complex| EXPERT[Expert Agent Pool]

    FAST --> FILTER[Filter by Capabilities]
    CAPABLE --> FILTER
    EXPERT --> FILTER

    FILTER --> CHECK{Sufficient<br/>Candidates?}

    CHECK --> |Yes| BANDIT[Multi-Armed Bandit<br/>Selection]
    CHECK --> |No| FALLBACK[Fallback to<br/>General Pool]

    FALLBACK --> BANDIT

    BANDIT --> LOAD{Agent<br/>Available?}

    LOAD --> |Yes| SELECT[Select Agent]
    LOAD --> |No| NEXT[Try Next Candidate]

    NEXT --> BANDIT

    SELECT --> LOG[Log Decision]
    LOG --> ASSIGN[Assign Task]
```

### Implementation

```typescript
class IntelligentRouter {
  async route(task: Task): Promise<RoutingDecision> {
    // Step 1: Assess complexity
    const complexity = await this.complexityAssessor.assess(task);

    // Step 2: Get candidate pool
    const pool = await this.getAgentPool(complexity, task.type);

    // Step 3: Filter by capabilities
    const capable = await this.capabilityMatcher.match(task, pool);

    if (capable.length === 0) {
      // Fallback to general pool
      capable.push(...(await this.getGeneralPool()));
    }

    // Step 4: Multi-armed bandit selection
    const selected = await this.multiArmedBandit.select(capable, task);

    // Step 5: Check availability
    if (await this.loadBalancer.shouldRejectTask(selected.agent)) {
      // Try next best candidate
      return this.route(task); // Recursive with updated state
    }

    // Step 6: Log decision
    await this.performanceTracker.logRoutingDecision({
      taskId: task.id,
      selectedAgent: selected.agent.id,
      strategy: selected.strategy,
      confidence: selected.confidence,
      alternativesConsidered: selected.alternatives,
      rationale: selected.rationale,
    });

    return selected;
  }
}
```

---

## Continuous Learning from Routing

### Feedback Loop

After each task completes:

```typescript
class RoutingLearner {
  async processTaskOutcome(
    task: Task,
    routing: RoutingDecision,
    outcome: TaskResult
  ): Promise<void> {
    // Update bandit state
    await this.updateBanditStatistics(routing.selectedAgent, task.type, {
      success: outcome.success,
      quality: outcome.qualityScore,
      latency: outcome.latencyMs,
    });

    // If routing failed, learn why
    if (!outcome.success) {
      await this.analyzeRoutingFailure(routing, outcome);
    }

    // If routing succeeded exceptionally, reinforce
    if (outcome.qualityScore > 0.9) {
      await this.reinforceSuccessPattern(routing, outcome);
    }

    // Update agent capabilities if new strengths discovered
    if (this.discoveredNewCapability(routing, outcome)) {
      await this.updateAgentCapabilities(routing.selectedAgent, outcome);
    }
  }
}
```

---

## Success Criteria

**Routing Accuracy**:

- Task-agent match accuracy: ≥85%
- First-choice success rate: ≥80%
- Fallback rate: ≤10%

**Learning Effectiveness**:

- Routing quality improves over time
- Exploration rate decreases appropriately
- Agent capability profiles stay current

**Performance**:

- Routing decision latency: <100ms
- Tracking overhead: <50ms
- Memory footprint: <500MB

---

**Intelligent routing ensures every task goes to the right agent while generating the data needed to make even better routing decisions in the future.**
