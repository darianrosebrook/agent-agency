# V2 Glossary: Key Terms & Concepts

**Author**: @darianrosebrook

---

## CAWS (Coding-Agent Working Standard) Terms

### Budget

**Definition**: Enforceable limits on code changes for a task.

**Fields**:

- `max_files`: Maximum files that can be modified
- `max_loc`: Maximum lines of code that can be changed

**Purpose**: Prevents scope creep and encourages minimal, targeted changes.

**Example**: `change_budget: { max_files: 25, max_loc: 1000 }`

---

### Gate

**Definition**: Quality check that must pass before work is considered complete.

**Types**:

- Mandatory: Must pass (e.g., `tests-pass`, `lint-clean`)
- Optional: Should pass but can be waived

**Purpose**: Ensures code quality and CAWS compliance.

**Example**: `qualityGates: ["tests-pass", "lint-clean", "coverage-80"]`

---

### Waiver

**Definition**: Documented exception to CAWS requirements for exceptional circumstances.

**Structure**:

```yaml
id: WV-BUDGET-001
title: Emergency hotfix exceeds budget
reason: emergency_hotfix
gates: ["budget-files"]
expires_at: 2025-11-01T00:00:00Z
approved_by: @lead-architect
status: active
```

**Purpose**: Allow necessary exceptions while maintaining audit trail.

---

### Verdict

**Definition**: CAWS validation result for a task execution.

**Output**: `verdict.yaml` file with compliance status, gate results, and provenance.

**Purpose**: Immutable record of CAWS evaluation for audit and training.

---

### Arbiter

**Definition**: The orchestration component that routes tasks and enforces CAWS constitutional authority.

**Responsibilities**:

- Task routing via multi-armed bandit
- CAWS policy enforcement
- Performance tracking
- Provenance recording

**Location**: `src/orchestrator/ArbiterOrchestrator.ts`

---

### Provenance

**Definition**: Immutable audit trail of all decisions and outcomes.

**Fields**:

- Who (agent, human, or hybrid)
- What (task, changes, outcomes)
- When (timestamps)
- Why (rationale, CAWS compliance)
- How (tools used, process followed)

**Purpose**: Complete traceability for compliance, debugging, and training.

---

## Orchestration Terms

### Multi-Armed Bandit

**Definition**: Algorithm that balances exploration (trying new agents) vs exploitation (using proven agents).

**Formula**: UCB (Upper Confidence Bound) = mean success rate + confidence interval

**Purpose**: Optimal agent selection that learns which agents perform best for which tasks.

**Implementation**: `src/orchestrator/MultiArmedBandit.ts`

---

### Agent Profile

**Definition**: Complete record of an agent's capabilities, performance history, and current status.

**Fields**:

- Capabilities: Task types, languages, specializations
- Performance: Success rate, average quality, latency
- Load: Active tasks, queued tasks, utilization

**Purpose**: Enable intelligent routing decisions based on agent strengths.

---

### Routing Decision

**Definition**: Record of why a specific agent was chosen for a task.

**Fields**:

- Selected agent and rationale
- Strategy used (multi-armed-bandit, capability-match, etc.)
- Alternatives considered
- Confidence level

**Purpose**: Audit trail for routing + training data for routing improvements.

---

### Capability Matching

**Definition**: Scoring algorithm that matches task requirements to agent capabilities.

**Factors**:

- Task type match (40% weight)
- Language match (30% weight)
- Specialization match (30% weight)

**Purpose**: Ensure agents are routed tasks they're qualified for.

---

## Benchmark Data Terms

### Benchmark Data Point

**Definition**: Complete record of task execution including routing, execution, evaluation, and RL labels.

**Size**: ~10-20KB per point

**Purpose**: Primary training data for RL agent improvement.

**Schema**: See `2-benchmark-data/data-schema.md`

---

### Quality Gate (Data)

**Definition**: Validation check that data must pass before entering RL training pipeline.

**Categories**:

- Completeness: All required fields present
- Type Safety: Values match schema
- Privacy: No PII or secrets
- Consistency: Cross-field validation

**Purpose**: Ensure only high-quality data trains agents.

---

### Anonymization

**Definition**: Process of removing PII and tenant identifiers from benchmark data.

**Methods**:

- Hash tenant IDs
- Remove file/variable names
- Generalize context
- Add differential privacy noise (optional)

**Purpose**: Enable cross-tenant learning while preserving privacy.

---

### RL-Ready Data

**Definition**: Benchmark data that has passed all quality gates and is formatted for RL consumption.

**Threshold**: ≥95% of collected data should be RL-ready.

**Flag**: `rlReady: boolean` in database

---

## RL Training Terms

### Turn-Level RL

**Definition**: Reinforcement learning on multi-turn conversations with intermediate rewards per action.

**vs Token-Level**: Rewards each dialogue turn (tool call), not each token generated.

**Purpose**: Better credit assignment for multi-turn tasks.

---

### GRPO (Group Relative Policy Optimization)

**Definition**: RL algorithm that groups trajectories by outcome similarity and computes relative advantages.

**Benefit**: More stable training than vanilla policy gradient.

**Implementation**: `src/rl/GRPOTrainer.ts`

---

### Minimal-Diff

**Definition**: AST-based analysis that measures how minimal a code change is.

**Metrics**:

- AST similarity (tree-edit distance)
- File touch count
- Line change ratio
- Scaffolding penalty

**Purpose**: Prevent reward hacking where agents add unnecessary code to pass tests.

---

### Reward Hacking

**Definition**: When an agent learns to game evaluation metrics rather than genuinely improving.

**Examples**:

- Adding unnecessary scaffolding to appear thorough
- "Spray editing" many files to seem productive
- Defensive over-engineering

**Mitigation**: Minimal-diff analysis, model judges, rubric engineering.

---

### Rubric Engineering

**Definition**: Systematic reward design with explicit weights for different evaluation dimensions.

**Dimensions**:

- Format (JSON compliance, structure)
- Tool (appropriate usage, information gain)
- Task (correctness, completion)
- Minimal (code minimality)
- Cost (token/time efficiency)
- Safety (security, permissions)

**Purpose**: Clear, maintainable, debuggable reward functions.

---

### Model-Based Judge

**Definition**: LLM used to evaluate subjective criteria that are hard to rule-check.

**Judge Types**:

- Relevance: Is information useful?
- Faithfulness: No hallucination?
- Minimality: Simplest correct solution?
- Safety: No harmful patterns?

**Implementation**: `src/evaluation/ModelBasedJudge.ts`

---

### SFT Warmup

**Definition**: Supervised Fine-Tuning on correct examples before RL training.

**Purpose**: Teach basic competencies (tool JSON formatting, error handling) before optimizing with RL.

**Benefit**: Prevents RL from having to learn both format AND strategy simultaneously.

---

### Thinking Budget

**Definition**: Token allocation for agent deliberation, allocated based on task complexity.

**Allocations**:

- Trivial: ≤500 tokens
- Standard: ≤2000 tokens
- Complex: ≤8000 tokens

**Purpose**: Optimize resource usage, prevent infinite thinking loops.

---

## Integration Terms

### Feedback Loop

**Definition**: Circular process where arbiter performance data trains agents, which improves arbiter decisions.

**Cycle**: Arbiter → Benchmark Data → RL Training → Improved Agents → Arbiter

**Duration**: Initial loop ~30-45 days, then continuous.

---

### Self-Improving System

**Definition**: System that uses its own operational data to continuously improve its capabilities.

**Characteristics**:

- Data collection built into normal operation
- Training pipeline automated
- Improvements deploy automatically (with validation)
- Compounding returns over time

**V2 Innovation**: All three pillars work together to create self-improvement.

---

### Constitutional Framework

**Definition**: CAWS as the foundational governance layer that binds all system components.

**Properties**:

- Universal: Applies to all agents and arbiter itself
- Enforceable: Violations prevented, not just detected
- Auditable: Complete provenance trail
- Evolvable: Can be refined through waivers and amendments

---

### Reflexivity

**Definition**: The principle that the arbiter must audit itself using the same CAWS standards it enforces on others.

**Mechanisms**:

- Self-audit: `caws audit self`
- Self-waivers: Documented exceptions
- Reflexive training: RL optimizes arbiter compliance

**Purpose**: Self-consistency and philosophical completeness.

---

## Metric Abbreviations

| Abbreviation | Full Term                           | Typical Value     |
| ------------ | ----------------------------------- | ----------------- |
| **P95**      | 95th percentile                     | Latency: <500ms   |
| **LOC**      | Lines of Code                       | Budget: 1000-3500 |
| **UCB**      | Upper Confidence Bound              | Routing score     |
| **AST**      | Abstract Syntax Tree                | For minimal-diff  |
| **SFT**      | Supervised Fine-Tuning              | Phase before RL   |
| **GRPO**     | Group Relative Policy Optimization  | RL algorithm      |
| **PII**      | Personally Identifiable Information | Must be removed   |

---

## File Naming Conventions

| Pattern               | Example                    | Purpose                |
| --------------------- | -------------------------- | ---------------------- |
| `verdict.yaml`        | `VERDICT-TASK-001.yaml`    | CAWS validation result |
| `WV-*.yaml`           | `WV-BUDGET-001.yaml`       | Waiver document        |
| `SELF-VERDICT-*.yaml` | `SELF-VERDICT-001.yaml`    | Arbiter self-audit     |
| `*.api.yaml`          | `arbiter-routing.api.yaml` | API specification      |

---

## Cross-References

For detailed information:

- **CAWS Standards**: See project root `.cursor/rules/` directory
- **Arbiter Architecture**: `1-core-orchestration/arbiter-architecture.md`
- **Data Schema**: `2-benchmark-data/data-schema.md`
- **RL Training**: `3-agent-rl-training/technical-architecture.md`
- **Integration**: `integration-strategy.md`
- **Reflexivity**: `1-core-orchestration/caws-reflexivity.md`

---

_This glossary provides quick definitions for V2 terminology—use it as a reference while reading the detailed documentation._
