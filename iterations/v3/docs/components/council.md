# Council of Judges

Purpose: Independent, specialized evaluation of worker outputs against CAWS and technical quality.

Subcomponents:
- Constitutional Judge: CAWS compliance, budgets, waivers, provenance.
- Technical Auditor: Code quality, security, contracts, migrations.
- Quality Evaluator: Correctness, completeness, maintainability vs acceptance.
- Integration Validator: Cross-file/api/db coherence and breaking change check.
- Consensus Coordinator: Weighted voting, debate protocol, verdict assembly.

Inputs:
- Task spec, scope, risk tier, acceptance criteria.
- Worker structured output + rationale + self-assessment.
- Context bundle from Research Agent.

Outputs:
- Individual judge verdicts {pass|fail|uncertain, reasons, evidence}.
- Final verdict document with weights, votes, and citations.

Key Interactions:
- Receives worker output from Orchestration Core.
- May request additional evidence from Research Agent during debate.
- Emits verdict to Orchestration Core and Provenance store.

Non-Functionals:
- Latency targets per judge, thermal-aware execution.
- Deterministic scoring; no hidden state between evaluations.

