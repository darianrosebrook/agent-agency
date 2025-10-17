# Apple Silicon Optimization Layer

Purpose: Maximize throughput/latency via Core ML, ANE/GPU routing, quantization, and unified memory.

Key Elements:
- Core ML executors (ANE for constitutional judge; GPU for auditor).
- Quantization strategy (INT4/INT8 for judges, INT8/FP16 for workers).
- Unified memory manager with pre-warming and LRU eviction.
- Thermal monitoring and adaptive batching.

Interfaces:
- Model registry exposes available precisions and placement hints.
- Execution planner picks device per model and current thermals.

Non-Functionals:
- Upper bounds for peak memory and sustained thermals.

