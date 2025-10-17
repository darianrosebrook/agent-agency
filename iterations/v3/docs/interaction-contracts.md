# Interaction Contracts

Worker Output Schema (summary):
- metadata: task id, risk tier, seeds
- artifacts: file edits, patches, commands (dry-runable)
- rationale: stepwise reasoning
- self_assessment: CAWS checklist results
- waivers: requested exceptions with id/reason/scope

See: contracts/worker-output.schema.json

Judge Verdict Schema:
- judge_id, version
- verdict: pass | fail | uncertain
- reasons: list
- evidence: references to research sources or static checks

See: contracts/judge-verdict.schema.json

Final Verdict Schema:
- votes: per judge with weights
- decision: accept | reject | modify
- dissent: notes and required changes
- remediation: council-required changes to pass gates
- constitutional_refs: CAWS sections cited in decision

See: contracts/final-verdict.schema.json

Router Decision Factors:
- capability tags, historical performance, current load, risk tier

See: contracts/router-decision.schema.json
