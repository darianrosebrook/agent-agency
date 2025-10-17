# Worker Pool

Purpose: Execute tasks with CAWS-aware workers (generalist + specialists).

Capabilities:
- Structured outputs with rationale and self-assessment.
- Deterministic mode via injected time/uuid/random.
- Concurrency control and cancellation.

Inputs:
- Task from Task Router including risk tier and scope.
- Context package from Research Agent.

Outputs:
- Structured result: {artifacts, diffs, commands, rationale, self-check}.

Contracts:
- Must not exceed scope.in or change budget.
- Emit provenance hooks and deterministic seeds.

Interactions:
- Pulls context from Research Agent.
- Sends output to Council for evaluation.

