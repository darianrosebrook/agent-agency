# Open Questions and Research Plan

Ambiguities to Resolve:
- Judge Models: Final model choices, sizes, quantization levels per device class.
- Debate Protocol: Round limits, message budgets, and escalation conditions.
- Weighting Scheme: Fixed vs adaptive weights per risk tier and task type.
- Research Agent Sources: Allowed domains, rate limits, and offline modes.
- Contract Testing: Minimal set for Tier 2/3 vs comprehensive for Tier 1.
- Provenance Format: Verdict document schema and signing approach.
- Thermal Policies: Thresholds for throttling on M3 Pro/Max variants.
- Memory Sharing: Zero-copy feasibility across Rust/Swift boundary.
- Fallback Paths: Behavior when a judge is unavailable or times out.

Research Tasks and Metrics:
- Core ML vs CPU-only: record p50/p95 latency per judge, ANE/GPU utilization %, and quality deltas (pass/fail rates) at INT4/INT8/FP16.
- Mutation testing vs gate latency: track mutation score vs council evaluation time; target T2 ≥50% with <1s council eval p95.
- Vector DB/embeddings: measure recall@k and latency on Metal; memory footprint per 100k vectors.
- Debate efficacy: resolution rate after round 1/2/3, time per round, and improvement in consensus score; aim ≥80% resolved by round 2.
- Pipeline throughput: tasks/min under 4/8/12 workers, peak memory and thermal; SLA for Tier 2 task end-to-end <90s p95.

Decision Records (to write as ADRs):
- ADR-001: Consensus weighting and thresholds.
- ADR-002: Quantization and device placement policy.
- ADR-003: Verdict schema and provenance signing.
- ADR-004: Research Agent source policy and caching.
