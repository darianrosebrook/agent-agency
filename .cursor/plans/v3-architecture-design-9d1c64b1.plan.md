<!-- 9d1c64b1-34c0-4fd3-b183-5f3537c9f972 00b31381-d4c7-4d0e-8dcb-2b307c97b8b9 -->
# V3 Theory Alignment & Component Integration Plan

## Overview

Document all gaps between V3 current state, theory requirements, and V2 learnings to create a comprehensive implementation roadmap. This ensures V3 achieves full theory alignment while incorporating optimized versions of proven V2 components.

## Phase 1: Gap Analysis Documentation

### Create Master Gap Analysis Document

Create `iterations/v3/docs/V3_THEORY_ALIGNMENT_GAP_ANALYSIS.md` with:

**Section 1: Critical Missing Theory Requirements**

- Claim Extraction & Verification Pipeline (theory lines 113-547)
- 4-stage processing: Disambiguation → Qualification → Decomposition → Verification
- Research-based evaluation metrics from Metropolitansky & Larson 2025
- Integration points with council verdict system
- Reflexive Learning Loop (theory lines 582-715)
- Memory system integration with multi-tenant context
- Turn-level progress tracking and credit assignment
- Adaptive resource allocation based on performance
- Model Performance Benchmarking (theory lines 717-1069)
- Continuous micro-benchmarks (daily) and macro-benchmarks (weekly)
- Multi-dimensional scoring framework per task surface
- New model evaluation pipeline with staged assessment
- Runtime Optimization Engine (theory lines 1071-1333)
- Multi-stage decision pipeline with fast-path classification
- Worker-specific optimization profiles (INT8/FP16 strategies)
- Bayesian parameter optimization for continuous improvement

**Section 2: V2 Components Worth Porting**
For each component, document:

- V2 implementation quality (production-ready/functional/alpha)
- Value proposition for V3
- Required modifications for Rust/Council architecture
- Integration complexity (low/medium/high)
- Priority tier (critical/high/medium/low)

Components to evaluate:

1. Multi-Turn Learning Coordinator (high value, medium complexity)
2. Context Preservation Engine (high value, medium complexity)
3. Adaptive Resource Manager (medium value, low complexity)
4. Security Policy Enforcer (medium value, low complexity)
5. System Health Monitor (medium value, low complexity)
6. Workspace State Manager (medium value, medium complexity)
7. Minimal Diff Evaluator (high value, low complexity)
8. Thinking Budget Manager (high value, low complexity)

**Section 3: V3 In-Flight Components Needing Enhancement**

- MCP Server Integration
- Current: Types and stubs only
- Missing: Dynamic tool discovery, runtime integration
- Theory

### To-dos

- [ ] Create v3 directory structure and initialize Rust workspace with core services
- [ ] Define Modelfile specifications for all judge and worker models with CAWS training datasets
- [ ] Design simplified v3 database schema for council, workers, and verdicts
- [ ] Implement consensus coordinator and debate protocol in Rust
- [ ] Build and fine-tune constitutional judge model on CAWS principles
- [ ] Implement worker pool manager with task routing and CAWS self-check utilities
- [ ] Create Core ML integration layer with ANE/GPU/CPU routing and quantization
- [ ] Implement knowledge seeker with vector search and context synthesis
- [ ] Enhance observer bridge for council deliberation visualization
- [ ] Build comprehensive test suite covering council behavior, CAWS compliance, and performance