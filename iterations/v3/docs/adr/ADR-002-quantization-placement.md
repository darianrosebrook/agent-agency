# ADR-002: Quantization and Device Placement

Status: Proposed

Decision (Proposed):
- Judges: INT4/INT8; Jc on ANE, Jt on GPU, Jq/Ji on CPU.
- Workers: INT8/FP16 depending on model size and memory pressure.
- Placement switches when thermal >85°C or mem pressure >80%.

Rationale:
- Meets latency targets while controlling thermals.

Risks:
- Quality loss from heavy quantization; to be benchmarked.

