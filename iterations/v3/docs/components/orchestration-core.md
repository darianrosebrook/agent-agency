# Orchestration Core

Purpose: Route tasks, coordinate execution and evaluation, enforce CAWS gates.

Subcomponents:
- Task Router: selects worker(s) based on specialty and risk tier.
- Execution Manager: parallelization, retries, backoff.
- Council Coordinator: runs judges, aggregates verdicts, triggers debate.

Inputs:
- User task with working spec and risk tier.

Outputs:
- Accepted artifacts or actionable feedback.
- Provenance updates and audit trail entries.

Policies:
- Tier 1: unanimous or 80%+ supermajority with debate rounds.
- Tier 2/3: simple majority; still record dissent.

Metrics Emitted:
- council_eval_ms: p50/p95 council evaluation time
- debate_rounds: rounds taken to resolve conflicts
- consensus_score: weighted vote total per decision
- dissent_rate: proportion of decisions with dissent notes
- t2_e2e_ms_p95: end-to-end latency for Tier 2 tasks p95
- throughput_tasks_min: tasks completed per minute
- peak_mem_gb, max_thermal_c: resource peaks during runs
- mutation_score, coverage_branch_pct: quality gates
