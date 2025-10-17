# Lessons From V2 → Design Inputs for V3

This summarizes concrete learnings from V2 artifacts and how they inform V3.

Key Learnings:
- Single-orchestrator bottleneck: Too many responsibilities concentrated in one component caused coupling, slower decisions, and complex failure modes. V3 adopts a specialized council with a coordinator.
- Runtime-only CAWS enforcement: Repetitive, slower, and easy to bypass. V3 bakes CAWS into models (training/fine-tunes) with a runtime validator as backstop.
- Integration complexity from 29 components: High friction to reason about the system. V3 targets ~15–20 components with crisp boundaries and contracts.
- Apple Silicon underutilization: V2 referenced local hardware benefits but did not deeply optimize. V3 uses Core ML/ANE/GPU routing, quantization, and unified memory.
- Research mixed with execution: Workers spent cycles on gathering context. V3 separates a Research Agent to assemble high-quality context.
- Timeout/perf issues on complex tasks: V2 noted 52s timeouts for certain tasks. V3 adds routing, batching, and thermal-aware scheduling.
- Compliance/provenance variance: V2 had strong guidance; enforcement and provenance need to be first-class and automated in V3 verdict flow.

Design Decisions Derived:
- Council of Judges with weighted consensus and debate protocol.
- Contract-first interfaces between workers, judges, and coordinator.
- Deterministic execution by default; inject time/uuid/random.
- Tiered gates align to CAWS risk tiers; unanimous or supermajority for T1.
- Observer Bridge retained and enhanced for transparency.

