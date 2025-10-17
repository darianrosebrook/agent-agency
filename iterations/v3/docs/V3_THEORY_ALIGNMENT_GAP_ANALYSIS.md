# V3 Theory Alignment Gap Analysis

Purpose: Identify gaps between V3 implementation, the theory requirements, and V2 learnings. Drives roadmap and auditing.

## Critical Missing Theory Requirements
- Claim Extraction & Verification Pipeline (4-stage: Disambiguation → Qualification → Decomposition → Verification)
- Reflexive Learning Loop (progress tracking, credit assignment, adaptive allocation)
- Model Performance Benchmarking (micro/macro; multi-dimensional scoring)
- Runtime Optimization Engine (fast-path classification; device/precision profiles)
- MCP Tooling Integration (discovery/invocation; deterministic interfaces)
- Provenance Service (immutable ledger with git trailer integration; JWS signing)

## V2 Components Worth Porting (with improvements)
- Minimal Diff Evaluator
  - Improve: AST/language-aware, integrate into CAWS validator, emit judge-ready evidence.
- Thinking Budget Manager
  - Improve: Tier-based budgets; adaptive from reflexive learning; council-visible.
- Context Preservation Engine
  - Improve: Multi-tenant caches; dedup; research pre-warm; freshness heuristics.
- Security Policy Enforcer
  - Improve: Constitutional judge-aligned rules; unified policy and violation mapping.
- Workspace State Manager
  - Improve: Deterministic snapshots; dry-run patch validation for judges.
- Adaptive Resource Manager
  - Improve: Telemetry-driven Apple Silicon placement; tier-aware throttling.

## V3 In-Flight Components Needing Enhancement
- MCP Server Integration: dynamic tool discovery; runtime hooks.
- Apple Silicon Optimization: quantization pipeline; thermal manager.
- Research Agent: context synthesis; cross-reference detection; metrics.
- CAWS Provenance Ledger: service + signer + git trailer integration.

## Integration Points with Council
- Validator snapshot + research evidence bundled into evaluation context.
- Claim coverage metrics reported in FinalVerdict.
- Remediation references map to constitutional sections and evidence citations.

## Prioritized Gaps (P1–P3)
- P1: Claim pipeline, Reflexive learning, Provenance service
- P2: Benchmarking, Apple Silicon quantization/thermal
- P3: MCP integration, Security enforcement distribution

