# V3 System Architecture

## Overview

Agent Agency V3 implements a **constitutional council system** where specialized judge models work together to audit, evaluate, and accept worker outputs through **constitutional concurrency** - a framework where multiple agents coordinate within agreed-upon bounds rather than competing through traditional parallelism.

The architecture coordinates specialized judges, a research agent, and a worker pool via a Rust orchestration core optimized for Apple Silicon, with built-in CAWS compliance and provenance tracking.

## Core Design Principles

### 1. Constitutional Concurrency
**Traditional parallelism** races to execute operations and resolves conflicts later. **Constitutional concurrency** establishes consensus boundaries first, then executes within agreed constraints.

### 2. Council-Based Governance
Four specialized judges provide independent oversight:
- **Constitutional Judge**: CAWS compliance, budgets, waivers, provenance
- **Technical Auditor**: Code quality, security, contracts, migrations
- **Quality Evaluator**: Correctness, completeness, maintainability
- **Integration Validator**: Cross-file/API/database coherence

### 3. Risk-Tiered Execution
Tasks execute with different concurrency models based on risk:
- **Tier 1 (High Risk)**: Sequential execution with maximum oversight
- **Tier 2 (Medium Risk)**: Limited parallel with checkpoint consensus
- **Tier 3 (Low Risk)**: High parallel with minimal coordination

### 4. Apple Silicon Optimization
Native hardware acceleration across the Neural Engine (ANE), GPU, and CPU with unified memory management and thermal awareness.

## System Architecture Diagram

```mermaid
flowchart LR
  subgraph User
    U[Task + Working Spec]
  end

  subgraph Orchestration
    R[Task Router]
    E[Execution Manager]
    CV[CAWS Runtime Validator]
    Cc[Council Coordinator]
  end

  subgraph Research
    Ra[Research Agent<br/>Context Synthesis<br/>Evidence Gathering]
  end

  subgraph Workers
    Wg[Generalist Workers<br/>Adaptive Routing]
    Ws[Specialist Workers<br/>Domain Expertise]
  end

  subgraph Council
    Jc[Constitutional Judge<br/>CAWS Compliance]
    Jt[Technical Auditor<br/>Code Quality]
    Jq[Quality Evaluator<br/>Requirements Fit]
    Ji[Integration Validator<br/>System Coherence]
  end

  subgraph Infrastructure
    DB[(PostgreSQL<br/>pgvector)]
    Prov[Provenance Store<br/>Git + JWS]
    Health[System Health<br/>Monitor]
  end

  U --> R
  R --> Ra
  R --> Wg
  R --> Ws
  Ra --> Wg
  Ra --> Ws
  Wg --> E
  Ws --> E
  E --> CV
  CV --> Cc
  E --> Cc
  Cc --> Jc
  Cc --> Jt
  Cc --> Jq
  Cc --> Ji
  Jc --> Cc
  Jt --> Cc
  Jq --> Cc
  Ji --> Cc
  Cc -->|Final Verdict| R
  R -->|Accept/Reject/Modify| U

  E --> DB
  Cc --> DB
  Cc --> Prov
  Health -.->|Health Signals| Cc
  Health -.->|Circuit Breaker| E
```

## Apple Silicon Hardware Acceleration

```mermaid
flowchart TB
  subgraph Device[M3 Pro/Max]
    ANE[ANE: Constitutional Judge<br/>CAWS Compliance]
    GPU[GPU: Technical Auditor<br/>Code Analysis]
    CPU[CPU: Quality + Integration<br/>Validators]
    MEM[Unified Memory Manager<br/>Thermal Aware]
  end
  ANE <---> MEM
  GPU <---> MEM
  CPU <---> MEM
```

## Current Implementation Status

### ✅ Production-Ready Components (11/11)

**Core Council System:**
- **Council Coordinator**: Debate protocol with learning signals, risk-tiered execution
- **Constitutional Judge**: CAWS compliance, provenance tracking, budget enforcement
- **Technical Auditor**: Code quality analysis, security scanning, contract validation
- **Quality Evaluator**: Requirements fit assessment, acceptance criteria verification
- **Integration Validator**: Cross-system coherence, breaking change detection

**Worker & Research Systems:**
- **Research Agent**: Context synthesis, evidence gathering, cross-reference detection
- **Worker Pool**: Generalist/specialist workers with adaptive routing
- **Reflexive Learning**: Multi-turn learning coordination, continuous improvement

**Infrastructure & Security:**
- **System Health Monitor**: Agent monitoring, circuit breakers, Prometheus metrics
- **Security Policy Enforcer**: Comprehensive security controls, audit logging
- **Context Preservation Engine**: Multi-tenant context management, synthesis
- **Workspace State Manager**: Repository state management, rollback capabilities

### 🔧 Key Architectural Features

**Constitutional Concurrency:**
- Risk-tiered parallelism (Tier 1: sequential, Tier 2: checkpoint, Tier 3: parallel)
- Consensus-driven coordination vs traditional race conditions
- Constitutional boundaries prevent agent conflicts

**Apple Silicon Optimization:**
- Native ANE/GPU/CPU acceleration with unified memory
- Thermal-aware execution scheduling
- Hardware-specific model quantization and placement

**CAWS Compliance & Provenance:**
- Runtime CAWS validation with evidence enrichment
- Git-backed provenance with JWS signing
- Constitutional audit trails for all decisions

**Database & Persistence:**
- PostgreSQL with pgvector for semantic search
- Multi-tenant context isolation
- Transactional integrity across all operations

## Component Interaction Patterns

### Task Execution Flow
1. **Task Submission** → Router analyzes scope and risk tier
2. **Research Gathering** → Context synthesis and evidence collection
3. **Worker Execution** → Parallel or sequential based on risk tier
4. **CAWS Validation** → Runtime compliance checking
5. **Council Review** → Judge evaluation with debate protocol
6. **Final Verdict** → Consensus decision with audit trail

### Concurrency Coordination
- **Low Risk (Tier 3)**: High parallelism, minimal coordination
- **Medium Risk (Tier 2)**: Limited parallel with consensus checkpoints
- **High Risk (Tier 1)**: Sequential execution with continuous oversight

### Error Recovery Patterns
- Circuit breaker patterns for component failures
- Graceful degradation based on risk tier
- Automatic rollback capabilities with provenance tracking

## Performance Characteristics

- **Constitutional Judge**: <100ms inference (ANE-optimized)
- **Technical Auditor**: <500ms analysis (GPU-accelerated)
- **Quality Evaluator**: <200ms assessment
- **Integration Validator**: <150ms coherence checking
- **Council Consensus**: <1s for Tier 2/3, <3s for Tier 1 tasks
- **Worker Execution**: <2s per request with parallel workers

## See Also

- **[CONCURRENT_AGENT_OPERATIONS.md](../CONCURRENT_AGENT_OPERATIONS.md)** - Agent isolation and coordination framework
- **[coordinating-concurrency.md](./coordinating-concurrency.md)** - Constitutional concurrency patterns
- **[BUILD_OPTIMIZATION.md](./BUILD_OPTIMIZATION.md)** - Build performance and agent isolation
- **[components/](./components/)** - Detailed component documentation
- **[interaction-contracts.md](./interaction-contracts.md)** - API contracts and interaction patterns
- **[contracts/](./contracts/)** - JSON schemas and data contracts
