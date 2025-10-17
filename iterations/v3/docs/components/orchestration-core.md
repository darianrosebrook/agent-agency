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

