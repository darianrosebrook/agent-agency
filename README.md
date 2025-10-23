# Agent Agency - Constitutional AI Agent System

## Overview

**Agent Agency** is a production-ready constitutional AI system that implements constitutional governance for autonomous agent operations. The system orchestrates multiple local AI models (Ollama/Gemma + CoreML/FastViT) with hot-swapping capabilities, using a council of specialized AI judges to provide real-time oversight, ensuring ethical compliance, technical quality, and system coherence through evidence-based decision making.

The V3 iteration delivers a **functional constitutional AI system** with a working core execution loop, constitutional governance, and monitoring capabilities. **The core task execution pipeline is operational** with real implementations for the main workflow, including **thread-safe CoreML integration** with Send/Sync safety guarantees. Built in Rust for performance and safety, it provides both CLI and web interfaces for task execution and system management.

**Recent Major Achievement**: Complete resolution of Send/Sync violations in CoreML FFI operations through thread-confined architecture with channel-based communication, enabling safe async coordination across the entire system.

This mono-repo contains multiple iterations examining different approaches to AI agent systems:

- **`iterations/v2/`**: TypeScript implementation investigating multi-component agent orchestration with external service integration
- **`iterations/v3/`**: **Production-ready Rust implementation** with constitutional council governance, multiple execution modes, and comprehensive monitoring
- **`iterations/poc/`**: Reference implementation examining multi-tenant memory systems and federated learning concepts
- **`iterations/main/`**: Reserved for stable research artifacts

## Project Structure

```
agent-agency/
├── iterations/
│   ├── v2/               # TypeScript multi-component agent orchestration
│   ├── v3/               # Rust-based advanced AI capabilities
│   ├── poc/              # Multi-tenant memory systems reference
│   ├── main/             # Reserved for stable research artifacts
│   └── arbiter-poc/      # Arbiter-specific research experiments
├── docs/                 # Research documentation and findings
├── scripts/              # Shared build and utility scripts
├── apps/                 # MCP tools and utilities
├── package.json          # Mono-repo dependency management
└── tsconfig.json         # Base TypeScript configuration
```

## Core Capabilities

Agent Agency V3 delivers a functional constitutional AI system with the following implemented capabilities:

### Constitutional Council Governance
**✅ Core Framework Implemented** - Four specialized AI judges provide oversight framework:
- **Constitutional Judge**: Ethical compliance and CAWS governance (framework implemented)
- **Technical Auditor**: Code quality and security validation (framework implemented)
- **Quality Evaluator**: Requirements satisfaction and correctness (framework implemented)
- **Integration Validator**: System coherence and architectural integrity (framework implemented)

### Task Execution Pipeline
**✅ Operational** - Core execution loop is working:
- **Worker Orchestration**: HTTP-based task distribution with circuit breaker patterns
- **Progress Tracking**: Real-time task status and metrics collection
- **Intervention API**: Pause, resume, cancel operations implemented
- **Multiple Execution Modes**: Strict, Auto, and Dry-Run modes supported

### Monitoring & Control
**🟡 Partially Implemented** - Basic monitoring with room for enhancement:
- Real-time task progress tracking (✅ implemented)
- CLI intervention commands (✅ implemented)
- Web dashboard with metrics (✅ implemented)
- SLO monitoring framework (📋 planned)
- Provenance tracking (✅ basic implementation)

### MCP Tool Ecosystem
**✅ Fully Implemented** - Comprehensive Model Context Protocol (MCP) server with 13 specialized tools:
- **Policy Tools (3)**: `caws_policy_validator`, `waiver_auditor`, `budget_verifier` - Governance and compliance
- **Conflict Resolution Tools (3)**: `debate_orchestrator`, `consensus_builder`, `evidence_synthesizer` - Arbitration and decision-making
- **Evidence Collection Tools (3)**: `claim_extractor`, `fact_verifier`, `source_validator` - Verification and validation
- **Governance Tools (3)**: `audit_logger`, `provenance_tracker`, `compliance_reporter` - Audit trails and compliance
- **Quality Gate Tools (3)**: `code_analyzer`, `test_executor`, `performance_validator` - Code quality and testing
- **Reasoning Tools (2)**: `logic_validator`, `inference_engine` - Logical reasoning and probabilistic inference
- **Workflow Tools (2)**: `progress_tracker`, `resource_allocator` - Project management and resource optimization

All tools leverage existing enterprise-grade systems (claim extraction, council arbitration, provenance service, quality gates, reflexive learning) and are available via standardized MCP protocol for external AI model integration.

## Research Iterations

### V3: Functional Constitutional AI System

The **V3 iteration** delivers a functional constitutional AI system with an operational core execution loop and governance framework, though many advanced features remain as TODOs:

#### Multi-Model AI System ✅ Operational
- **Ollama Integration**: Local Gemma 3N model for general-purpose AI tasks with circuit breaker patterns
- **CoreML Acceleration**: Apple Silicon optimized models including FastViT T8 F16 for vision processing with **thread-safe FFI integration**
- **Model Hot-Swapping**: Zero-downtime model replacement with performance tracking and A/B testing
- **Self-Prompting Loops**: Autonomous agent that iteratively improves outputs until quality thresholds met
- **Model Registry**: Performance-weighted routing with task-specific model affinities (code tasks → Ollama, vision → CoreML)
- **Send/Sync Safety**: **NEW** - CoreML operations safely integrated with async Rust runtime through thread confinement and channel-based communication

#### CoreML Safety Architecture ✅ **NEW** - Production Ready
- **Thread-Confinement**: CoreML raw pointers isolated to dedicated threads, preventing Send/Sync violations
- **Opaque Model References**: `ModelRef(u64)` identifiers safely cross async boundaries
- **Channel-Based Communication**: Async coordination between council and inference threads using `crossbeam::channel`
- **Memory Safety**: Proper resource cleanup and leak prevention with Drop implementations
- **FFI Boundary Control**: All unsafe CoreML operations quarantined with comprehensive validation

#### Core Execution Loop ✅ Operational
- **Task Submission**: REST API and CLI interfaces for task creation
- **Worker Orchestration**: HTTP-based task distribution with circuit breaker patterns
- **Progress Tracking**: Real-time task status and intervention capabilities
- **Execution Modes**: Strict, Auto, and Dry-Run modes supported
- **Intervention API**: Pause, resume, cancel operations implemented

#### Governance Framework ✅ Core Implemented
- **Constitutional Council**: Four-judge framework for oversight (logic partially implemented)
- **CAWS Compliance**: Runtime validation with waiver system for exceptions
- **Provenance Tracking**: Basic Git integration with cryptographic signing framework
- **Quality Gates**: Automated testing and validation pipelines

#### Monitoring & Control 🟡 Partially Implemented
- **Real-time Monitoring**: Task progress and basic system metrics
- **CLI Intervention**: Core intervention commands implemented
- **Web Dashboard**: Basic metrics display and database exploration
- **SLO Monitoring**: Framework implemented, comprehensive monitoring TODO
- **Alert Management**: Basic alerting, advanced features TODO

#### Infrastructure 🟡 Partially Implemented
- **Database Layer**: PostgreSQL persistence with core task storage
- **API Server**: RESTful API with authentication and basic endpoints
- **Task Persistence**: Task lifecycle management implemented
- **Security**: Basic API key authentication implemented
- **Deployment Ready**: Basic Docker setup, production deployment TODO

#### Advanced Features 📋 Planned/Incomplete
- **Multimodal Processing**: Framework exists, CoreML/FastViT vision processing operational, advanced enrichers TODO
- **Apple Silicon Optimization**: CoreML integration operational, advanced thermal management TODO
- **Distributed Processing**: Single-node only, distributed features TODO
- **Advanced Analytics**: Basic metrics, comprehensive analytics TODO

### V2: TypeScript Multi-Component Orchestration

The **V2 iteration** investigates multi-component agent orchestration, examining:

- **External Service Integration**: Patterns for connecting AI agents with enterprise services
- **Component Architecture**: Modular design for agent capabilities and coordination
- **Quality Assurance**: Automated testing and validation approaches
- **Infrastructure Management**: Resource allocation and monitoring strategies

### POC: Multi-Tenant Memory Systems

The **POC iteration** explores foundational concepts for agent memory and learning:

- **Multi-Tenant Memory**: Context isolation and sharing mechanisms
- **Federated Learning**: Privacy-preserving cross-agent knowledge transfer
- **MCP Integration**: Model Context Protocol for agent communication
- **Reinforcement Learning**: Tool optimization and adaptive behavior patterns

## Research Areas

This framework investigates approaches to multimodal AI systems and constitutional governance in several areas:

- **Multimodal RAG Systems**: Processing and retrieval across text, image, audio, video, and document modalities
- **Constitutional Governance**: Real-time decision-making with evidence-based validation and constraint enforcement
- **Vector-Based Knowledge Systems**: High-performance semantic search with pgvector and HNSW indexing
- **Production AI Deployment**: Scalable, monitored, and secure deployment of multimodal AI systems
- **Cross-Modal Validation**: Ensuring consistency and accuracy across different content modalities
- **Hardware-Accelerated Processing**: Leveraging Apple Silicon for efficient multimodal processing and governance

## V3 System Characteristics

### Ideal Use Cases
The V3 system excels in environments requiring **enterprise-grade quality assurance** with **local execution**:

- **Enterprise Development Teams**: CAWS governance ensures production-ready code generation with full audit trails
- **Privacy-Sensitive Organizations**: Local Ollama/CoreML models prevent data leakage to cloud providers
- **Apple Silicon Ecosystems**: Native CoreML/ANE acceleration provides exceptional performance on Mac hardware
- **Quality-Critical Workflows**: Self-prompting loops with satisficing logic prevent over-optimization
- **Cost-Conscious Development**: Eliminates per-API-call costs for high-volume AI-assisted tasks

### System Limitations
While powerful for its target use cases, V3 has specific constraints:

- **Local Model Constraints**: Gemma 3N has smaller parameter count than GPT-4, affecting reasoning depth and training data recency
- **Hardware Dependencies**: CoreML optimizations are Apple Silicon-specific, limiting platform portability
- **Resource Requirements**: Requires powerful local machines (32GB+ RAM, M-series chips) that most developers lack
- **Cold Start Times**: Model loading and initialization can take 30-60 seconds, unsuitable for interactive workflows
- **Scalability Boundaries**: Cannot scale across multiple machines like cloud-based systems

### Comparative Advantages

| Aspect | V3 System | Cloud API (GPT-4) | Traditional IDE Tools |
|--------|-----------|-------------------|----------------------|
| **Privacy** | ✅ Excellent | ❌ Poor | ✅ Good |
| **Safety** | ✅ **NEW** - Thread-safe CoreML | ⚠️ Variable | ✅ Good |
| **Cost** | ✅ Low | ❌ High (scale) | ✅ Low |
| **Quality** | ✅ Self-improving | ✅ High baseline | ❌ Variable |
| **Speed** | ⚠️ Good (local) | ✅ Excellent | ✅ Fast |
| **Complexity** | ❌ High | ✅ Low | ✅ Low |
| **Maintenance** | ❌ High | ✅ Low | ✅ Low |
| **Scalability** | ❌ Limited | ✅ High | ✅ High |

## Technical Approaches

### Constitutional Governance

The framework investigates several approaches to constitutional AI governance:

- **Judge Model Architectures**: Different patterns for specialized evaluation models
- **Evidence-Based Verification**: Mechanisms for validating agent outputs against constitutional requirements
- **Runtime Constraint Enforcement**: Approaches to enforcing governance rules during execution
- **Learning Judge Systems**: How judge models can improve through experience

### Multi-Agent Coordination

Research into coordination mechanisms beyond traditional hierarchies:

- **Constitutional Concurrency**: Agent coordination through agreed-upon principles
- **Evidence-Based Arbitration**: Decision-making based on verifiable evidence rather than authority
- **Scalable Agent Ecosystems**: Patterns for managing large numbers of coordinated agents
- **Conflict Resolution**: Approaches to handling conflicting agent outputs

### Implementation Strategies

Different technical approaches to constitutional governance:

- **TypeScript Orchestration**: Dynamic coordination with comprehensive type safety
- **Rust Governance**: Memory-safe, high-performance governance operations
- **Hardware Acceleration**: Leveraging specialized hardware for governance tasks
- **Federated Learning**: Privacy-preserving knowledge sharing across agent boundaries

## Architecture Patterns

### Constitutional Governance Patterns

The framework explores several architectural patterns for implementing constitutional governance:

- **Judge Model Networks**: Networks of specialized models that evaluate different aspects of agent behavior
- **Evidence Pipelines**: Multi-stage verification systems that validate agent outputs against constitutional requirements
- **Runtime Enforcement**: Mechanisms for applying constitutional constraints during agent execution
- **Feedback Learning Loops**: Systems where governance decisions improve through experience

### Agent Coordination Models

Research into different approaches to agent coordination:

- **Constitutional Concurrency**: Agents coordinate through shared constitutional principles
- **Evidence-Based Arbitration**: Decision-making based on verifiable evidence and constitutional compliance
- **Hierarchical Governance**: Multi-level governance with different scopes of authority
- **Distributed Consensus**: Agreement protocols for constitutional decision-making

## Research Methodology

The project employs progressive research through multiple implementation iterations:

- **V2 (TypeScript)**: Explores multi-component orchestration patterns and external service integration
- **V3 (Rust)**: Investigates memory safety, performance characteristics, and hardware acceleration
- **POC**: Examines foundational concepts in multi-tenant memory and federated learning

### Implementation Strategy

- **Mono-repo Structure**: Enables comparison of different implementation approaches
- **Progressive Research**: Each iteration builds on findings from previous work
- **Cross-iteration Validation**: Concepts tested across different technical stacks
- **Research Documentation**: Findings documented in the docs/ directory

## Getting Started

### Prerequisites

**For V3 (Constitutional AI System)**:
- Rust 1.75+
- Docker 20.10+ and Docker Compose 2.0+
- PostgreSQL with pgvector extension
- Node.js 18+ (for CAWS tools and web dashboard)
- Apple Silicon recommended for optimal performance

### Installation

```bash
# Clone the repository
git clone <repository-url>
cd agent-agency

# Install Node.js dependencies (for CAWS and dashboard)
npm install

# Install CAWS Git hooks for provenance tracking
cd iterations/v3
./scripts/install-git-hooks.sh
```

### Quick Start - V3 System

```bash
cd iterations/v3

# 1. Verify compilation (includes CoreML safety checks)
cargo check -p agent-agency-council -p agent-agency-apple-silicon
# Should show 0 errors - Send/Sync violations resolved ✅

# 2. Start the database (optional - system has in-memory fallback)
docker run -d --name postgres-v3 -e POSTGRES_PASSWORD=password -p 5432:5432 postgres:15
docker exec -it postgres-v3 psql -U postgres -c "CREATE DATABASE agent_agency_v3;"

# 3. Run database migrations (if using PostgreSQL)
cargo run --bin migrate

# 4. Start the API server
cargo run --bin api-server &
API_PID=$!

# 5. Start the worker service (in another terminal)
cargo run --bin agent-agency-worker &
WORKER_PID=$!

# 6. Execute a task (core execution loop is operational with thread-safe CoreML)
cargo run --bin agent-agency-cli execute "Test the execution pipeline" --mode dry-run

# 7. Monitor progress via CLI
cargo run --bin agent-agency-cli intervene status <task-id>

# Cleanup when done
kill $API_PID $WORKER_PID
```

**Note**: The core task execution pipeline is operational with thread-safe CoreML integration. Send/Sync violations have been resolved through proper FFI boundary control. Many advanced features remain as TODO implementations. Use dry-run mode for safe testing without filesystem changes.

### CLI Usage Examples

```bash
# Dry-run mode (safe testing)
cargo run --bin agent-agency-cli execute "Add user registration" --mode dry-run

# Auto mode with quality gates
cargo run --bin agent-agency-cli execute "Implement payment system" --mode auto --risk-tier 1

# Strict mode with manual approval
cargo run --bin agent-agency-cli execute "Deploy to production" --mode strict --watch

# CLI intervention during execution
cargo run --bin agent-agency-cli intervene pause task-123
cargo run --bin agent-agency-cli intervene resume task-123
cargo run --bin agent-agency-cli intervene cancel task-123
```

### Web Dashboard

```bash
# Start the monitoring dashboard
cd iterations/v3/apps/web-dashboard
npm run dev

# Access at http://localhost:3000
# Features:
# - Real-time task monitoring
# - System metrics and SLOs
# - Database exploration
# - Provenance tracking
# - Alert management
```

### Development Testing

```bash
# Build all components
cd iterations/v3
cargo build --workspace

# Run comprehensive tests
cargo test --workspace

# Run CAWS validation
cd ../../apps/tools/caws
npm run validate -- --spec-file ../../../iterations/v3/.caws/working-spec.yaml

# Test end-to-end integration
cd ../../../iterations/v3
npm run test:integration
```

#### Infrastructure Features
- **Modular Architecture**: Independent components with clear interfaces
- **Comprehensive Testing**: Unit and integration tests for all modules
- **Performance Benchmarks**: Automated benchmarking for optimization components
- **Security Validation**: Security testing and vulnerability scanning
- **Documentation**: Complete API documentation and usage examples

## Documentation

### V3 System Documentation
- **[System Overview](./iterations/v3/docs/SYSTEM_OVERVIEW.md)**: Complete system architecture and capabilities
- **[Architecture Guide](./iterations/v3/docs/architecture.md)**: Technical implementation details
- **[CAWS Workflow Guide](./docs/agents/full-guide.md)**: Comprehensive CAWS implementation guide
- **[CLI Tutorial](./docs/agents/tutorial.md)**: Getting started with the CLI
- **[API Documentation](./iterations/v3/docs/interaction-contracts.md)**: REST API endpoints and contracts

### Component Documentation
- **[Database Schema](./docs/database/README.md)**: Database design and migration guide
- **[Quality Assurance](./docs/quality-assurance/README.md)**: Testing and CAWS compliance
- **[Monitoring Guide](./iterations/v3/docs/observability.md)**: Metrics, SLOs, and alerting
- **[Deployment Guide](./deploy/README.md)**: Production deployment procedures

### Research & Reference
- **[Arbiter Theory](./docs/arbiter/theory.md)**: Constitutional AI governance research
- **[CAWS Framework](https://github.com/paths-design/caws)**: Workflow specification standard
- **[Implementation Roadmap](./docs/P0-IMPLEMENTATION-ROADMAP.md)**: Development progress and priorities

## Author

@darianrosebrook
