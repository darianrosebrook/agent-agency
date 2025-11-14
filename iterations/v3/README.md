# Agent Agency V3

## Overview

Agent Agency V3 provides a modular Rust-based infrastructure for autonomous agent systems with constitutional oversight. The system implements a task execution pipeline with governance controls, monitoring capabilities, and comprehensive evaluation frameworks for measuring agent intelligence and performance.

### Agent Evaluation Framework

Unlike traditional software testing that focuses on binary success/failure, Agent Agency V3 evaluates autonomous agents across **5 key dimensions**:

- **Functional Correctness (30%)**: Code compiles, functionality works, requirements met
- **Process Quality (25%)**: Reasoning depth, decision quality, risk assessment, coordination
- **Adaptability (20%)**: Uncertainty management, failure recovery, strategy flexibility
- **Safety (15%)**: Risk avoidance, error handling, boundary compliance, audit completeness
- **Efficiency (10%)**: Resource usage balanced against problem complexity

**Chain-of-Thought Evaluation**: Every agent decision is captured and analyzed for reasoning quality, alternative consideration, confidence calibration, and risk awareness. See [Evaluation Framework](./docs/evaluation-framework.md) for detailed methodology.

**Scenario-Based Testing**: Agents are evaluated in controlled playground environments with known issues, measuring not just outcomes but the quality of problem-solving approaches.

**Quality Evaluation**: Comprehensive quality assessment framework for evaluating AI agent output against mid-level engineer/writer standards. See [Quality Evaluation Plan](./docs/QUALITY_EVALUATION_PLAN.md) for testing chain-of-thought reasoning, council decisions, and output quality.

## Architecture

### System Components

The system is organized into 17 focused crates with clear responsibilities:

#### Core Agent Systems

- **agent-orchestration**: Task coordination, council governance, and autonomous file editing
- **agent-constitutional-council**: Four-judge constitutional oversight with hybrid CAWS + LLM reasoning
- **agent-workers**: Parallel task execution and MCP-based worker management
- **agent-model-management**: Model lifecycle management, inference, and hot-swapping
- **agent-data-processing**: Data ingestion, enrichment, indexing, and knowledge processing
- **agent-memory**: Agent memory system with knowledge graphs and embeddings
- **agent-research**: Advanced AI research capabilities and reflexive learning
- **agent-mcp**: Model Context Protocol implementation and tool orchestration

#### Infrastructure Services

- **data-infrastructure**: PostgreSQL persistence, API interfaces, and data transformation
- **system-observability**: Monitoring, metrics collection, distributed tracing, and alerting
- **system-quality-security**: Authentication, authorization, quality gates, and integrity verification
- **system-resilience**: Fault tolerance, recovery, and content-addressable storage
- **system-resources**: Resource management and production hardening
- **system-configuration**: Configuration management and common utilities
- **system-federated-ml**: Distributed ML and runtime optimization
- **system-acceleration**: Hardware acceleration (Apple Silicon, quantization)
- **development-tools**: Development workflow and code analysis tools
- **testing-validation**: Comprehensive testing platform and quality assurance

#### Interface Layer

- **data-interfaces**: CLI, API, and web interface components
- **agent-agency-contracts**: Type definitions and API contracts

## Task Execution System

### System Architecture & Connection Flow

The system implements a layered architecture with clear separation between interfaces, adapters, and core orchestration:

```
┌─────────────────────────────────────────────────────────────┐
│                    API Layer (REST/CLI)                      │
│  POST /api/v1/tasks → OrchestratorService                    │
└───────────────────────┬─────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              OrchestratorService (Observational API)          │
│  - Task state management                                     │
│  - Chain-of-thought tracking                                 │
│  - Council decision logging                                  │
│  - Worker action observation                                 │
└───────────────────────┬─────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│         UnifiedOrchestratorTaskExecutor (Bridge)             │
│  - Converts TaskDescriptor → WorkingSpec                    │
│  - Delegates to UnifiedOrchestrator                         │
│  - Returns ExecutionArtifacts                               │
└───────────────────────┬─────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              UnifiedOrchestrator (Core Engine)               │
│  - Plan generation                                          │
│  - Council review (CAWS Examination stage)                   │
│  - Plan execution via WorkerExecutionBridge                 │
│  - Refinement loops                                          │
│  - Git worktree management                                   │
└───────────────────────┬─────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│         WorkerExecutionBridge (Type Conversion)              │
│  - Converts Milestone → TaskDefinition                       │
│  - Delegates to MCPWorkerPool                                │
│  - Converts TaskResult → ExecutionArtifacts                 │
└───────────────────────┬─────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│              MCPWorkerPool (Worker Management)               │
│  - Worker registration and capability matching              │
│  - Task execution via MCPIntegration                         │
│  - Shared memory system integration                          │
│  - HTTP fallback for distributed workers                    │
└───────────────────────┬─────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────┐
│         MCPIntegration (Tool Execution)                      │
│  - MCP protocol communication                                │
│  - Tool registry management                                  │
│  - Execution via MCP tools                                  │
└─────────────────────────────────────────────────────────────┘
```

### Execution Pipeline

The system implements a constitutional task execution pipeline with the following components:

1. **Task Submission**: CLI and REST API interfaces for task creation
2. **OrchestratorService**: Observational API that tracks task state without direct manipulation
3. **UnifiedOrchestrator**: Core orchestration engine coordinating planning, council review, and execution
4. **Council Governance**: Four-judge constitutional oversight (Constitutional, Technical, Quality, Integration) with hybrid CAWS + LLM reasoning
5. **Worker Execution**: Parallel task processing with MCP-based worker management
6. **Progress Monitoring**: Real-time status tracking and intervention capabilities
7. **Provenance Tracking**: Git-based audit trails and change attribution

### Execution Modes

- **Dry-Run Mode**: Safe testing without filesystem changes
- **Auto Mode**: Automatic execution with quality validation
- **Strict Mode**: Manual approval required for execution phases

### Quality Assurance

The system enforces quality gates through automated validation:

- **Constitutional Council**: Four-judge oversight with hybrid CAWS invariants + LLM reasoning
- **CAWS Compliance**: Runtime validation with waiver system and deterministic rule checking
- **Autonomous File Safety**: Git-worktree isolation, changeset validation, and rollback capabilities
- **Code Analysis**: Static analysis and quality checks
- **Test Coverage**: Automated testing with coverage requirements
- **Security Scanning**: Vulnerability detection and integrity verification
- **Agent Evaluation Framework**: Multi-dimensional assessment of agent intelligence:
  - **Process Quality** (25%): Reasoning depth, decision quality, risk assessment
  - **Adaptability** (20%): Uncertainty management, failure recovery, strategy flexibility
  - **Safety** (15%): Risk avoidance, boundary compliance, audit completeness
  - **Functional Correctness** (30%): Code compilation, functionality, requirements
  - **Efficiency** (10%): Resource usage vs. problem complexity

## Hardware Acceleration (Apple Silicon ANE)

### ANE Performance Investigation - Conclusion

Comprehensive benchmarking with micro models and Mistral 7B FP16 has conclusively demonstrated that **ANE is working correctly** - the ~0.95-1.01x speedup observed is a **platform characteristic**, not a bug.

**Key Findings**:

- ✅ **ANE is functional end-to-end**: Runtime path is correct, ANE is participating (47.4% utilization confirms participation)
- ✅ **Platform characteristic confirmed**: Micro models and Mistral show identical behavior, proving this is hardware/platform behavior, not a runtime or conversion issue
- ✅ **Input pooling is the optimization lever**: ~30ms allocation overhead eliminated with input pooling (~40% latency improvement: 75ms → 46ms)
- ✅ **Backend selection for FP16 Mistral**: CPU and ANE are latency-equivalent (~46ms with input pooling); choose based on power profile, thermal headroom, and CPU resource sharing
- ✅ **For meaningful speedups**: Need to change the problem (quantization INT8/INT4, smaller models) rather than tuning the existing FP16 Mistral graph

**Production Recommendations**:

- **Make input pooling the default** in production path (~40% latency improvement)
- **Backend selection**: For FP16 Mistral, CPU and ANE are equivalent; choose based on power/concurrency, not speed
- **Future track for speedups**: Quantized micro models → quantized small transformer → determine if ANE produces meaningful speedup on quantized workloads

**Documentation**:

- **[ANE Performance Conclusion](./docs/ANE_PERFORMANCE_CONCLUSION.md)**: Complete investigation summary
- **[Benchmark Report](./docs/BENCHMARK_REPORT.md)**: Comprehensive performance analysis
- **[Benchmark Stats Summary](./docs/BENCHMARK_STATS_SUMMARY.md)**: Quick reference statistics

## Project Structure

The V3 codebase is organized into focused crates with clear responsibilities:

```
iterations/v3/
├── agent-orchestration/             # Task coordination, council governance, and autonomous file editing
├── agent-constitutional-council/    # Four-judge constitutional oversight framework
├── agent-workers/                   # Parallel execution and MCP workers
├── agent-model-management/          # Model lifecycle and inference
├── agent-data-processing/           # Data pipeline and knowledge processing
├── agent-memory/                    # Agent memory and embeddings
├── agent-research/                  # AI research and reflexive learning
├── agent-mcp/                       # Model Context Protocol implementation with file editing tools
├── data-infrastructure/             # Database, APIs, and persistence
├── system-observability/            # Monitoring and metrics
├── system-common-interfaces/        # Shared traits for clean service boundaries
├── system-quality-security/         # Security and quality gates
├── system-resilience/               # Fault tolerance and recovery
├── system-resources/                # Resource management
├── system-configuration/            # Configuration and utilities
├── system-federated-ml/             # Distributed ML capabilities
├── system-acceleration/             # Hardware acceleration
├── development-tools/               # Development workflow tools
├── testing-validation/              # Testing platform
├── data-interfaces/                 # CLI, API, web interfaces
├── agent-agency-contracts/          # Type definitions and contracts
├── apps/                            # Application interfaces
│   ├── tools/                       # Development tool configurations
│   └── web-dashboard/               # React-based dashboard
├── docs/                            # Core architectural documentation
├── Cargo.toml, Cargo.lock           # Workspace configuration
└── .gitignore                       # Repository hygiene rules
```

## Getting Started

### Prerequisites

- **Rust 1.75+** with Cargo
- **PostgreSQL 14+** with pgvector extension
- **Docker** (optional, for containerized deployment)

### Quick Start

```bash
# Navigate to the project
cd iterations/v3

# Build the workspace
cargo build

# Run tests
cargo test

# Run a specific crate
cargo run --package data-interfaces
```

### Database Setup

The system requires PostgreSQL with vector extensions:

```bash
# Start PostgreSQL with pgvector
docker run -d \
  --name agent-agency-db \
  -e POSTGRES_DB=agent_agency \
  -e POSTGRES_USER=agent_agency \
  -e POSTGRES_PASSWORD=secure_password \
  -p 5432:5432 \
  pgvector/pgvector:pg15

# Enable extensions
docker exec -it agent-agency-db psql -U agent_agency -d agent_agency -c "CREATE EXTENSION IF NOT EXISTS pgvector;"
docker exec -it agent-agency-db psql -U agent_agency -d agent_agency -c "CREATE EXTENSION IF NOT EXISTS uuid_ossp;"
```

### Configuration

Set environment variables or create a `.env` file:

```bash
DATABASE_URL=postgresql://agent_agency:secure_password@localhost:5432/agent_agency
```

## Development Workflow

### Task Execution

The system supports constitutional task execution with quality gates:

```bash
# Dry-run mode (safe testing)
cargo run --package data-interfaces -- execute "Test task" --mode dry-run

# Auto mode (with validation)
cargo run --package data-interfaces -- execute "Implement feature" --mode auto

# Strict mode (manual approval)
cargo run --package data-interfaces -- execute "System changes" --mode strict
```

### Quality Gates

The system enforces CAWS compliance and quality standards:

- **Code Analysis**: Static analysis and linting
- **Test Coverage**: Automated testing with coverage requirements
- **Security Scanning**: Vulnerability detection and integrity verification
- **Provenance Tracking**: Git-based audit trails and change attribution

## Documentation

### Core Documentation

- **[System Architecture](./docs/README.md)**: Component organization and relationships
- **[CAWS Agent Guide](./docs/agents.md)**: Agent workflow and collaboration patterns
- **[Production Deployment](./deploy/README.md)**: Deployment and operational guides

### Component Documentation

- **agent-orchestration**: Task coordination, council governance, and autonomous file editing
- **agent-constitutional-council**: Four-judge constitutional oversight with hybrid CAWS + LLM reasoning
- **agent-mcp**: Model Context Protocol implementation with file editing tools and tool registry
- **system-common-interfaces**: Shared traits for dependency injection and service boundaries
- **data-infrastructure**: Database persistence and API interfaces
- **system-quality-security**: Security controls and quality gates
- **system-observability**: Monitoring and metrics collection

## Author

@darianrosebrook
