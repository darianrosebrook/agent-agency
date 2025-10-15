# Agent Agency V2: Quick Reference

**Version**: 2.0.0
**Author**: @darianrosebrook

---

## One-Sentence Summary

V2 is a **self-improving multi-agent system** where an arbiter orchestrator collects performance data from every task execution to continuously train and improve agents through reinforcement learning.

---

## The Three Pillars

### 1. Arbiter Orchestration

**Multi-armed bandit routing + CAWS constitutional authority**

- Routes tasks to optimal agents based on performance history
- Enforces CAWS budgets, quality gates, and provenance
- Tracks every decision for RL training

📁 `1-core-orchestration/`

### 2. Benchmark Data Pools

**Performance tracking → RL training data**

- Collects routing decisions, execution metrics, evaluation outcomes
- Validates quality, ensures privacy, anonymizes data
- Exports RL-ready batches for training

📁 `2-benchmark-data/`

### 3. Agent RL Training

**Continuous improvement through reinforcement learning**

- Trains agents on benchmark data (GRPO, turn-level RL)
- Prevents reward hacking (minimal-diff analysis)
- Optimizes thinking budgets and tool adoption
- Deploys improved agents back to arbiter

📁 `3-agent-rl-training/`

---

## The Feedback Loop

```
Arbiter routes → Tracks performance → Builds dataset →
Trains agents → Deploys improvements → Arbiter updates → Repeat
```

**Result**: System gets smarter with every task executed.

---

## Key Metrics

| Metric              | Target         | Pillar      |
| ------------------- | -------------- | ----------- |
| Routing accuracy    | ≥85%           | Arbiter     |
| Data quality        | ≥95%           | Benchmark   |
| Agent improvement   | +10% per cycle | RL Training |
| Tool adoption       | +300%          | RL Training |
| Thinking efficiency | -40% waste     | RL Training |
| CAWS compliance     | 100%           | Arbiter     |

---

## Document Map

### Start Here

- `README.md` - Complete V2 overview
- `integration-strategy.md` - How pillars integrate
- `.caws/working-spec.yaml` - Acceptance criteria

### By Pillar

- **Orchestration**: `1-core-orchestration/README.md`
- **Data**: `2-benchmark-data/README.md`
- **RL**: `3-agent-rl-training/README.md`

### Deep Dives

- **Arbiter Theory**: `1-core-orchestration/theory.md` (1,188 lines)
- **RL Architecture**: `3-agent-rl-training/technical-architecture.md`
- **Data Schema**: `2-benchmark-data/data-schema.md`

---

## Implementation Timeline

**Phase 1** (Weeks 1-8): Arbiter & data collection
**Phase 2** (Weeks 4-12): Data infrastructure
**Phase 3** (Weeks 8-18): RL training
**Phase 4** (Weeks 14-20): Continuous learning

**Flexible**: No hard deadlines, focus on quality and integration

---

## Key Innovation

**Traditional orchestration**: Route tasks → Agents execute → Done

**V2 orchestration**: Route tasks → Track performance → Train agents → Deploy improvements → Better routing → Repeat

**The difference**: Compounding intelligence that improves with every task.

---

## Contact

**Questions?**

- Orchestration: See `1-core-orchestration/`
- Data: See `2-benchmark-data/`
- RL: See `3-agent-rl-training/`
- Integration: See `integration-strategy.md`

---

_V2 transforms multi-agent coordination into continuous intelligence improvement._
