# V3 System Architecture

The council-based architecture coordinates specialized judges, a research agent, and a worker pool via a Rust orchestration core optimized for Apple Silicon.

High-Level Diagram (Mermaid):

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
    Ra[Research Agent]
  end

  subgraph Workers
    Wg[Generalist Workers]
    Ws[Specialist Workers]
  end

  subgraph Council
    Jc[Constitutional Judge]
    Jt[Technical Auditor]
    Jq[Quality Evaluator]
    Ji[Integration Validator]
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
```

Apple Silicon Placement (Mermaid):

```mermaid
flowchart TB
  subgraph Device[M3 Pro/Max]
    ANE[ANE: Const. Judge]
    GPU[GPU: Tech Auditor]
    CPU[CPU: Integr/Quality]
    MEM[Unified Memory Manager]
  end
  ANE <---> MEM
  GPU <---> MEM
  CPU <---> MEM
```

See also:
- components/*.md
- interaction-contracts.md
- open-questions-and-research.md
