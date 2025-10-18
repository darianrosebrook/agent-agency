# Agent Agency V3: Council-Based Arbiter Architecture

## Overview

Agent Agency V3 implements a constitutional council of specialist judge models that supervise asynchronous worker agents through **constitutional concurrency** - a framework where multiple agents coordinate within agreed-upon bounds rather than competing through traditional parallelism. The workspace is built in Rust with nine coordinated crates, integrates CAWS compliance at runtime, and is optimized for Apple Silicon execution through Core ML, ANE, GPU, and CPU orchestration.

The release bundles council governance, worker execution, research assistance, benchmarking, and provenance into a single workspace. PostgreSQL with pgvector supplies persistence, while MCP integration and Apple Silicon tooling supply execution flexibility.

## Implemented Capabilities

- **Council governance** – consensus coordinator, debate protocol, verdict storage, and learning signals housed in `council/`.
- **Worker services** – task routing, lifecycle management, and compliance checks within `workers/`.
- **Research and context** – dedicated retrieval, vector search, and context synthesis in `research/`.
- **Theory-critical pipelines** – claim extraction, reflexive learning, and benchmarking crates ported from V2 and reimplemented in Rust.
- **Compliance and provenance** – CAWS runtime validator, JWS signing, and Git-backed audit trails across `orchestration/` and `provenance/`.
- **Platform foundations** – Apple Silicon optimized inference layers, database access, and MCP tooling.

## Differentiation from V2

- **Council-based oversight** replaces the single-orchestrator design, separating constitutional review, technical auditing, quality evaluation, and integration validation.
- **Constitutional concurrency** enables multiple agents to work within agreed-upon bounds, eliminating traditional race conditions through consensus-driven coordination.
- **CAWS principles** are embedded in both model training pipelines and runtime validators, reducing the need for manual remediation.
- **Apple Silicon support** is first-class with Core ML integration, unified memory planning, and thermal management.
- **Research responsibilities** are isolated to a dedicated agent, improving worker throughput and token efficiency.
- **V3 consolidates** the component count into nine focused crates, simplifying maintenance and deployment.

## Delivery Status

### ✅ Completed

- **Rust workspace configuration** with nine crates and shared tooling.
- **Council system** with consensus coordination, debate flow, verdict persistence, and learning integration.
- **Worker pool** with routing, lifecycle management, and CAWS compliance checks.
- **Research agent** delivering retrieval, vector search, and context building.
- **PostgreSQL/pgvector data layer**, including schema and migrations.
- **MCP integration** for tool discovery and registration.
- **Provenance subsystem** with Git integration and JWS signing.
- **Apple Silicon execution path** covering Core ML models, ANE/GPU/CPU routing, and thermal management.
- **Database client implementation** with 14/22 critical methods implemented (64% complete).
- **Task management system** with complete CRUD operations and execution tracking.
- **Council database integration** with verdict management and judge evaluation tracking.
- **Code quality improvements** with 100+ linting errors resolved and production-ready codebase.
- **Edge case testing system** with comprehensive intelligent testing framework.
- **Workspace state management** with memory and database storage implementations.

### 🔄 In Progress

- **Remaining database methods** (8/22 methods): knowledge entries, performance metrics, CAWS compliance, audit trails.
- **Model fine-tuning assets** and LoRA scripts in `training/`.
- **Observer bridge** for deliberation visualization.
- **Comprehensive automated testing** across unit, integration, and end-to-end suites.

### 📋 Planned Next

- **Production hardening**: benchmarking, monitoring, alerting, and error recovery.
- **Model performance tuning** and regression tracking.
- **Expanded documentation** for deployment and operations.
- **Knowledge management system** completion.
- **Performance metrics collection** and analysis.

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
