# Agent Agency V3: Constitutional AI Agent Governance

## Overview

**Agent Agency V3** is a production-ready Rust framework for governing AI agent ecosystems through **constitutional oversight** - a novel approach where specialized judge models evaluate, constrain, and improve agent behaviors in real-time.

### Core Purpose
V3 addresses the fundamental challenge of **trustworthy AI agent deployment**: how to enable powerful AI agents to collaborate effectively while preventing failures, bias, and unintended consequences.

### Key Innovation: Constitutional Concurrency
Traditional agent systems use rigid hierarchies or market-based coordination. V3 introduces **constitutional concurrency** - agents coordinate through agreed-upon constitutional principles rather than competing for resources or following fixed command structures.

### Technical Foundation
- **23 Rust crates** with comprehensive error handling and observability
- **Apple Silicon optimization** with ANE/GPU/CPU orchestration
- **PostgreSQL + pgvector** for semantic persistence and retrieval
- **CAWS compliance** embedded in runtime validation
- **Git-backed provenance** with cryptographic audit trails

### What V3 Enables
- **Safe multi-agent collaboration** at scale without race conditions
- **Runtime constitutional compliance** preventing agent misbehavior
- **Continuous improvement** through judge learning and adaptation
- **Enterprise-grade reliability** with circuit breakers and resilience patterns
- **Hardware-accelerated performance** on Apple Silicon platforms

## Core Capabilities

### Constitutional Governance System
**Purpose**: Evaluates agent outputs against constitutional principles and CAWS compliance.

- **Four Specialized Judges**: Constitutional (compliance), Technical (code quality), Quality (requirements), Integration (system coherence)
- **Risk-Tiered Evaluation**: Sequential oversight for high-risk tasks, parallel evaluation for low-risk operations
- **Debate Protocol**: Judges can debate conflicting verdicts with evidence-based reasoning
- **Consensus Formation**: Weighted voting produces final verdicts with audit trails
- **Learning Integration**: Judges improve through experience and feedback loops

### Multi-Agent Orchestration Engine
**Purpose**: Coordinates worker agents with constitutional oversight and resource management.

- **Task Routing**: Intelligent distribution of work based on agent capabilities and constitutional constraints
- **Lifecycle Management**: Agent startup, monitoring, failure recovery, and graceful shutdown
- **Resource Arbitration**: Prevents agent conflicts through coordinated resource allocation
- **Execution Tracking**: Complete audit trails of agent actions and decision rationale
- **Circuit Breaker Protection**: Automatic failure isolation and recovery coordination

### Research & Context Intelligence
**Purpose**: Provides contextual intelligence and evidence gathering for agent decision-making.

- **Semantic Search**: Vector-based retrieval from PostgreSQL with pgvector
- **Context Synthesis**: Multi-source information integration and relevance ranking
- **Evidence Enrichment**: Automated gathering of supporting data for constitutional evaluation
- **Knowledge Graph Integration**: Cross-referencing capabilities for comprehensive analysis
- **Adaptive Learning**: Context understanding improves through usage patterns

### Advanced AI Pipelines
**Purpose**: Specialized AI processing for claim analysis, learning optimization, and performance evaluation.

- **Claim Extraction**: Automated identification and validation of factual claims in agent outputs
- **Reflexive Learning**: Multi-turn learning coordination with feedback integration
- **Model Benchmarking**: Comparative performance evaluation across different AI models
- **Minimal Diff Analysis**: AST-based change assessment for precise impact evaluation
- **Performance Optimization**: Automated tuning based on hardware capabilities and task requirements

### Compliance & Audit Infrastructure
**Purpose**: Ensures operational integrity, regulatory compliance, and complete auditability.

- **Runtime CAWS Validation**: Continuous compliance checking during agent execution
- **Cryptographic Provenance**: Git-backed audit trails with JWS digital signatures
- **Waiver Management**: Structured exception handling with approval workflows
- **Security Monitoring**: Real-time threat detection and response coordination
- **Regulatory Reporting**: Automated compliance documentation and evidence collection

### Apple Silicon Acceleration Platform
**Purpose**: Hardware-optimized execution environment leveraging native Apple Silicon capabilities.

- **ANE Integration**: Neural Engine acceleration for constitutional judge inference
- **GPU Orchestration**: Metal-based acceleration for complex analysis tasks
- **Unified Memory Management**: Efficient data sharing across CPU, GPU, and ANE
- **Thermal Awareness**: Performance scaling based on system temperature and power constraints
- **Core ML Optimization**: Model quantization and deployment optimization for Apple hardware
  - ⚠️ **Status**: [Blocked on Objective-C FFI bridge](./docs/CORE_ML_INTEGRATION_RISK.md) – See [Core ML Integration Risk Document](./docs/CORE_ML_INTEGRATION_RISK.md) for architecture, timeline, and alternatives
  - 📋 **Implementation Path**: [De-Risked Core ML Tactics](./docs/CORE_ML_IMPLEMENTATION_PATH.md) – Concrete Swift→C ABI bridge strategy with telemetry gates and decision checkpoints

## Problem Statement

**The Challenge**: AI agent deployment faces critical barriers to trustworthy, scalable operation:

- **Safety & Compliance**: How do we ensure AI agents follow ethical guidelines and legal requirements in real-time?
- **Coordination Complexity**: Traditional agent orchestration leads to race conditions, resource conflicts, and unpredictable behavior
- **Quality Assurance**: Manual review of agent outputs doesn't scale and introduces human error
- **Continuous Improvement**: Agents need to learn from experience without compromising safety
- **Enterprise Reliability**: Production agent systems require auditability, resilience, and regulatory compliance

**V3 Solution**: Constitutional governance provides the missing framework for trustworthy AI agent ecosystems.

## Key Innovations

### Constitutional Concurrency
**Traditional Approach**: Agents compete for resources or follow rigid hierarchies, leading to conflicts and inefficiencies.

**V3 Approach**: Agents coordinate through constitutional principles, establishing agreed bounds before execution rather than resolving conflicts after the fact.

### Council-Based Governance
**Traditional Approach**: Single orchestrator or manual oversight creates bottlenecks and single points of failure.

**V3 Approach**: Four specialized judges provide independent oversight, enabling parallel evaluation while maintaining comprehensive governance.

### Runtime Compliance
**Traditional Approach**: Compliance checked after execution, requiring remediation and rollback.

**V3 Approach**: CAWS principles embedded in runtime validation, preventing non-compliant actions before they occur.

## Use Cases & Applications

### Enterprise Software Development
- **Code Review Automation**: Constitutional judges evaluate code changes for quality, security, and compliance
- **Architecture Governance**: Integration validators ensure system coherence across microservices
- **Deployment Safety**: Risk-tiered evaluation prevents problematic releases

### Research & Analysis
- **Scientific Claim Validation**: Judges verify factual accuracy in research outputs
- **Bias Detection**: Constitutional oversight identifies and prevents biased analysis
- **Peer Review Automation**: Multi-judge consensus replaces manual review processes

### Financial Services
- **Regulatory Compliance**: Runtime validation ensures all actions meet financial regulations
- **Risk Assessment**: Constitutional judges evaluate transaction safety and market impacts
- **Audit Automation**: Cryptographic provenance provides immutable transaction trails

### Healthcare & Medical AI
- **Diagnostic Validation**: Judges verify medical AI outputs against clinical guidelines
- **Patient Safety**: Constitutional constraints prevent harmful recommendations
- **Regulatory Compliance**: Automated compliance checking for medical device certification

## Success Criteria

### Technical Metrics
- **Zero Race Conditions**: Agents coordinate without resource conflicts
- **Sub-3s Council Consensus**: Constitutional evaluation completes within performance budgets
- **99.9% Uptime**: Enterprise-grade reliability with circuit breaker protection
- **100% CAWS Compliance**: Runtime validation prevents non-compliant operations

### Quality Metrics
- **Judge Accuracy >95%**: Constitutional judges provide reliable evaluations
- **False Positive Rate <5%**: Minimal incorrect blocking of valid operations
- **Learning Improvement**: Judge accuracy increases through experience
- **Audit Completeness**: 100% of agent actions have cryptographic provenance

### Adoption Metrics
- **Multi-Agent Coordination**: Support for 10+ concurrent agents without conflicts
- **Enterprise Integration**: Successful deployment in regulated environments
- **Performance Scaling**: Maintains quality under increased load
- **Developer Productivity**: Reduces manual oversight by 80%

## Architecture Advantages

### Vs. Traditional Orchestration
- **Safety**: Constitutional constraints prevent unsafe operations at runtime
- **Scalability**: Parallel evaluation supports more agents than hierarchical approaches
- **Reliability**: Consensus-based decisions are more robust than single-orchestrator systems
- **Auditability**: Complete provenance trails enable regulatory compliance

### Vs. Rule-Based Systems
- **Adaptability**: Judge learning enables continuous improvement beyond static rules
- **Context Awareness**: Evidence-based evaluation considers situational factors
- **Flexibility**: Constitutional principles adapt to new requirements without code changes
- **Intelligence**: AI judges provide nuanced evaluation beyond binary rule matching

### Vs. Market-Based Coordination
- **Predictability**: Constitutional bounds prevent chaotic agent competition
- **Compliance**: Embedded CAWS principles ensure ethical operation
- **Coordination**: Structured consensus replaces unpredictable market dynamics
- **Accountability**: Clear audit trails assign responsibility for decisions

## Current Status & Next Steps

### Implementation Progress
- **Database Layer**: 11/22 async methods implemented (50% complete)
- **Core Council**: Sequential judge evaluation framework operational
- **Build Status**: 41 compilation errors, 30 warnings (active stabilization)
- **Test Coverage**: Basic unit tests implemented, integration tests in progress

### Known Limitations & Blockers

See **[docs/CORE_ML_INTEGRATION_RISK.md](./docs/CORE_ML_INTEGRATION_RISK.md)** for detailed risk analysis of pending work.

| Component | Status | Issue | Impact |
|-----------|--------|-------|--------|
| **Core ML Optimization** | ⚠️ Blocked | Requires Objective-C FFI bridge | ANE acceleration unavailable; falls back to CPU |
| **ANE Device Context** | ✓ Scaffolded | Framework APIs not exposed | Device capability detection works; compilation pending |
| **Model Lifecycle** | ✓ Complete | — | Model usage tracking, LRU eviction fully implemented |
| **Consensus Instrumentation** | ✓ Complete | — | Evaluation latency now properly tracked |
| **Judge Modifications** | ✓ Complete | — | RequiredChange synthesis from verdicts working |

**Recommendation**: Focus on CPU-based optimization and establish performance baseline before investing in Core ML FFI work. See [Core ML Integration Risk Document](./docs/CORE_ML_INTEGRATION_RISK.md) for alternatives and timeline.

### Immediate Priorities (Q1 2025)
1. **Resolve compilation errors** - Stabilize core functionality
2. **Complete database client** - Finish remaining 11 async methods
3. **Implement risk-tiered execution** - Enable parallel judge evaluation
4. **Establish test coverage** - Achieve 85%+ coverage target
5. **Performance optimization** - Meet sub-3s council consensus target

### Medium-term Goals (Q2-Q3 2025)
1. **Enterprise hardening** - Circuit breakers, security, observability
2. **Multi-tenant isolation** - Context separation and access controls
3. **Advanced learning** - Judge improvement through experience
4. **Regulatory compliance** - SOC 2, ISO certifications
5. **Ecosystem expansion** - Third-party judge marketplace

### Long-term Vision (2026+)
1. **Autonomous governance** - Self-improving constitutional systems
2. **Global standards** - Universal AI agent governance frameworks
3. **Predictive capabilities** - Governance failure prevention
4. **Inter-agent societies** - Constitutional coordination at scale

## System Metrics

### Codebase Statistics
- **Source files**: 240+ Rust modules
- **Total lines**: 240,000+ lines of code
- **Workspace crates**: 23 Rust packages
- **Build status**: 83 compilation errors remaining (48% reduction from 161), active development
- **Test coverage**: Implementation in progress (target: 85%+)

### Database Implementation
- **Client methods**: 11/22 async methods implemented
- **Schema**: PostgreSQL with pgvector extension
- **Migrations**: Version-controlled schema evolution

### Performance Targets
- **Constitutional Judge**: <100ms inference (ANE-optimized)
- **Technical Auditor**: <500ms analysis (GPU-accelerated)
- **Quality Evaluator**: <200ms assessment
- **Integration Validator**: <150ms coherence checking
- **Council consensus**: <3s maximum (risk-tier dependent)

## Implementation Status

### Core Architecture (23 Crates)

**Coordination Layer:**
- `council/`: Consensus coordination and debate protocol
- `orchestration/`: Task routing and execution management
- `workers/`: Agent pool and lifecycle management

**Domain Services:**
- `research/`: Context synthesis and evidence gathering
- `claim-extraction/`: V2 pipeline port for claim processing
- `reflexive-learning/`: Multi-turn learning coordination
- `model-benchmarking/`: Performance evaluation and scoring

**Infrastructure:**
- `database/`: PostgreSQL/pgvector persistence layer
- `provenance/`: Git-backed audit trails with JWS signing
- `apple-silicon/`: Hardware-accelerated inference optimization
- `system-health-monitor/`: Agent monitoring and circuit breakers
- `security-policy-enforcer/`: Security controls and audit logging
- `context-preservation-engine/`: Multi-tenant context management
- `workspace-state-manager/`: Repository state tracking
- `embedding-service/`: Vector embedding computation
- `minimal-diff-evaluator/`: AST-based change assessment
- `integration-tests/`: Cross-component validation
- `config/`: Configuration management and validation
- `resilience/`: Circuit breaker and retry patterns
- `observability/`: Metrics collection and monitoring
- `mcp-integration/`: Model Context Protocol support

### Development Status

**Completed Core Systems (9/9):**
- ✅ Council Debate Enhancement - Real session data integration with contextual intelligence
- ✅ Council Learning Store - Complete trait implementation with performance analysis
- ✅ Tool Discovery System - Advanced filtering, validation, and health monitoring
- ✅ Tool Registry Execution - Capability-based routing with security validations
- ✅ Coordinator Debate Rounds - Risk-tiered orchestration with proper stop criteria
- ✅ Coordinator Evaluation Timing - Comprehensive SLA monitoring and metrics
- ✅ Coordinator Metrics Endpoint - Production-ready monitoring and dashboards
- ✅ MCP Server HTTP/WebSocket - Full API implementation with authentication
- ✅ Council Verdicts Storage - Complete CRUD with memory/Postgres backends

**Active Development:**
- Database model synchronization (83 compilation errors remaining - 48% reduction)
- Test suite establishment for completed systems
- Configuration system integration testing

**Integration Testing:**
- Cross-component validation framework
- Performance benchmarking infrastructure
- CAWS compliance verification

**Documentation:**
- Architecture specification completion
- Component implementation guides
- Deployment and operations procedures

## Workspace Layout

```text
iterations/v3/
├── Cargo.toml                    # Workspace configuration
├── council/                      # Council of judges system
│   ├── src/                     # (coordinator, debate, verdicts, learning)
│   ├── models/                  # Judge model configurations
│   └── Cargo.toml
├── workers/                      # Worker pool management
│   ├── src/                     # (manager, router, executor, compliance)
│   └── Cargo.toml
├── research/                     # Research agent system
│   └── src/                     # (knowledge seeker, vector search, context)
├── apple-silicon/                # Apple Silicon optimization
│   └── src/                     # (Core ML, ANE, GPU, thermal, quantization)
├── claim-extraction/             # V2 claim extraction pipeline
├── reflexive-learning/           # Multi-turn learning coordination
├── model-benchmarking/           # Performance evaluation system
├── mcp-integration/              # Model Context Protocol integration
├── provenance/                   # Git-backed audit trails
├── database/                     # PostgreSQL/pgvector persistence
│   ├── schema.sql
│   ├── migrations/
│   └── src/
├── orchestration/                # Core orchestration logic
├── caws/                         # CAWS runtime validation
├── config/                       # Configuration management
├── context-preservation-engine/  # Multi-tenant context management
├── embedding-service/            # Vector embedding service
├── integration-tests/            # Cross-component integration tests
├── minimal-diff-evaluator/       # AST-based change assessment
├── observer/                     # Deliberation visualization
├── resilience/                   # Production resilience patterns
├── scripts/                      # Build and utility scripts
├── security-policy-enforcer/     # Security controls and audit logging
├── system-health-monitor/        # Health assessment and monitoring
├── workspace-state-manager/      # Repository state management
├── docs/                         # Architectural documentation
├── docs-status/                  # Implementation status (git-ignored)
├── tests/                        # Test suites (unit, integration, e2e)
├── training/                     # Model fine-tuning assets
└── apps/                         # Application binaries
```

## Setup

### Prerequisites

- Rust 1.70+ with `cargo`
- PostgreSQL 15+ with the pgvector extension
- Ollama for local model serving
- Apple Silicon hardware (recommended for target performance)

### Initialization

1. Navigate into the V3 workspace:

   ```bash
   cd iterations/v3
   ```

2. **Bootstrap agent environment** (recommended for concurrent work):

   ```bash
   # Auto-detect platform and setup agent isolation
   ./scripts/bootstrap-agent.sh

   # Or manually set agent identity
   export AGENT_ID="dev-setup-$(date +%s)"
   ./scripts/bootstrap-agent.sh
   ```

3. Provision PostgreSQL and pgvector, then create the project database:

   ```bash
   brew install postgresql pgvector
   createdb agent_agency_v3
   psql agent_agency_v3 < database/schema.sql
   ```

4. Pull required baseline models via Ollama (fine-tuning happens later):

   ```bash
   ollama pull gemma3n:e2b
   ollama pull gemma3n:e4b
   ollama pull embeddinggemma
   ollama pull llama3.3:3b
   ollama pull codellama:7b
   ollama pull mistral:3b
   ```

5. Build and test the workspace:

   ```bash
   # Use optimized build wrapper for concurrent safety
   AGENT_ID="setup-build" ./scripts/build-wrapper.sh dev --workspace
   AGENT_ID="setup-test" ./scripts/build-wrapper.sh test --workspace
   ```

### Concurrent Agent Operations

For running multiple agents concurrently without resource conflicts:

- **Read the comprehensive guide**: [`CONCURRENT_AGENT_OPERATIONS.md`](./CONCURRENT_AGENT_OPERATIONS.md)
- **Use agent bootstrap**: `./scripts/bootstrap-agent.sh` for automatic environment setup
- **Use build wrapper**: `./scripts/build-wrapper.sh` instead of raw `cargo` commands
- **Follow isolation principles**: Each agent gets unique write paths and shared read caches

**Example concurrent setup:**

```bash
# Terminal 1: Development agent
AGENT_ID="dev-agent-1" ./scripts/bootstrap-agent.sh
AGENT_ID="dev-agent-1" ./scripts/build-wrapper.sh dev --package council &

# Terminal 2: Test agent
AGENT_ID="test-agent-2" ./scripts/bootstrap-agent.sh
AGENT_ID="test-agent-2" ./scripts/build-wrapper.sh test --workspace &

# Terminal 3: Documentation agent
AGENT_ID="docs-agent-3" ./scripts/bootstrap-agent.sh
AGENT_ID="docs-agent-3" cargo doc --workspace &
   ```

## Usage Examples

### Council Coordination

```rust
use agent_agency_council::{ConsensusCoordinator, CouncilConfig, TaskSpec, RiskTier};

let config = CouncilConfig::default();
let coordinator = ConsensusCoordinator::new(config);

let task_spec = TaskSpec {
    id: Uuid::new_v4(),
    title: "Implement user authentication".into(),
    description: "Add JWT-based authentication system".into(),
    risk_tier: RiskTier::Tier1,
    scope: TaskScope {
        files_affected: vec!["src/auth/".into()],
        max_files: Some(5),
        max_loc: Some(1000),
        domains: vec!["authentication".into()],
    },
    acceptance_criteria: vec![],
    context: TaskContext::development("/workspace", "main"),
    worker_output: WorkerOutput::default(),
    caws_spec: None,
};

let result = coordinator.evaluate_task(task_spec).await?;
println!("Consensus result: {:?}", result.final_verdict);
```

### Database Operations

```rust
use agent_agency_database::{CreateJudge, DatabaseClient, DatabaseConfig};

let db = DatabaseClient::new(DatabaseConfig::default()).await?;

let judge = db.create_judge(CreateJudge {
    name: "Custom Judge".into(),
    model_name: "custom-model".into(),
    endpoint: "http://localhost:11434".into(),
    weight: 0.3,
    timeout_ms: 300,
    optimization_target: "CPU".into(),
}).await?;

println!("Created judge: {:?}", judge);
```

## Operational Targets

### Performance

- Constitutional judge inference: <100 ms (ANE-optimized)
- Technical auditor inference: <500 ms (GPU-accelerated)
- Quality evaluator inference: <200 ms
- Integration validator inference: <150 ms
- Worker task execution: <2 s per request with parallel workers

### Throughput

- 10+ concurrent workers on M3 Max (64 GB)
- 5+ concurrent workers on M3 Pro (32 GB)
- Council evaluation: <1 s for Tier 2/3 tasks, <3 s for Tier 1 tasks

### Resource Utilization

- Peak memory budget: 48 GB on M3 Max
- Idle footprint: ~12 GB with base models loaded
- Sustained thermal envelope: <80 °C
- Average power draw: <30 W under mixed ANE/CPU load

### Quality Gates

- CAWS compliance rate ≥95%
- Branch coverage ≥85% across crates
- Mutation testing thresholds and security scanning enforced before release
- Immutable provenance stored for each council decision

## Roadmap

- Finalize comprehensive automated testing and benchmarking suites.
- Complete production monitoring, alerting, and error-recovery paths.
- Deliver fine-tuned model artifacts and integration playbooks.
- Expand observer tooling for real-time deliberation visibility.

## Documentation

- [`CONCURRENT_AGENT_OPERATIONS.md`](./CONCURRENT_AGENT_OPERATIONS.md) - Comprehensive guide for running multiple agents concurrently across Rust, Python, and Node/TypeScript
- [`docs/coordinating-concurrency.md`](./docs/coordinating-concurrency.md) - Framework for concurrent agent coordination within council-based systems and concurrent documentation development
- [`docs/BUILD_OPTIMIZATION.md`](./docs/BUILD_OPTIMIZATION.md) - Rust build performance optimization guide and agent isolation
- `/docs` contains persistent architecture references, contracts, ADRs, and integration guidance
- `/docs-status` tracks implementation progress, gap analyses, and project status (git-ignored)
- `/archive` retains superseded research material for historical reference

## Contributing

1. Follow existing module boundaries and code patterns.
2. Add or update tests alongside any behavior change.
3. Refresh relevant documentation when APIs or workflows shift.
4. Ensure `cargo fmt`, `cargo clippy`, and `cargo test` pass before submitting.
5. Maintain CAWS compliance and update provenance records through the provided tooling.

## License

MIT License – see `LICENSE` for details.
