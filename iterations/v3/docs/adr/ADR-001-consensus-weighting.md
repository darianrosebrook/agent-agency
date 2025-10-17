# ADR-001: Consensus Weighting and Thresholds

Status: Proposed

Context:
- Judges: Constitutional (Jc), Technical (Jt), Quality (Jq), Integration (Ji).
- Need deterministic, auditable decision policy per risk tier.

Decision (Proposed):
- Weights: Jc=0.4, Jt=0.2, Jq=0.2, Ji=0.2.
- Thresholds: Tier1 ≥0.8; Tier2/3 ≥0.6.
- Debate: up to 2 additional rounds if below threshold but above 0.5.

Consequences:
- Prioritizes CAWS compliance.
- Clear supermajority for high risk.

Alternatives:
- Equal weights; adaptive per task type.

