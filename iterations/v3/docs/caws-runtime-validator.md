# CAWS Runtime Validator (V3)

Purpose: Provide lightweight, real-time enforcement of CAWS constraints during task execution, feeding structured compliance data into the Council.

Responsibilities:
- Track change budgets (files/LOC) and scope.in adherence.
- Record tool usage, time, and determinism signals.
- Validate referenced waivers and attach to WorkerOutput.
- Emit compliance status to Council Coordinator before judge evaluation.

Inputs:
- Working spec (.caws/working-spec.yaml)
- Router task descriptor (risk tier, scope)
- Waivers (.caws/waivers/*)

Outputs:
- Compliance snapshot: { within_scope, within_budget, tests_added, deterministic }
- Violations: list with codes and remediation suggestions
- Attached waiver metadata for judge review

Notes:
- This validator does not replace judges; it short-circuits obvious failures and provides ground truth for debate.
- Determinism: enforce injected seeds/time/uuid; flag direct Date.now()/random usage if detectable.

See implementation scaffold:
- v3/orchestration/src/caws_runtime.rs
