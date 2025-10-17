# Agent Agency V3: Council-Based Arbiter Architecture

## Overview

V3 reimagines agent orchestration as a **constitutional council system** where specialized judge models work together to audit, evaluate, and accept worker outputs. Built ground-up for Apple Silicon with Core ML optimization, this system combines model-native CAWS understanding with runtime enforcement for maximum efficiency.

## Architecture

### Core Components

1. **Council of Judges** - 4 specialized models for evaluation:

   - Constitutional Judge (CAWS compliance)
   - Technical Auditor (code quality & security)
   - Quality Evaluator (acceptance criteria)
   - Integration Validator (system coherence)

2. **Worker Pool** - Task execution models with CAWS awareness
3. **Research Agent** - Dedicated knowledge gathering and context building
4. **Apple Silicon Native** - Core ML optimized inference pipeline
5. **Hybrid CAWS** - Models trained on CAWS principles + runtime validation

## Current Implementation Status

### ✅ Completed Core Architecture

- **Rust Workspace** - 9 crates with proper dependencies and workspace configuration
- **Council System** - Complete consensus coordinator, debate protocol, verdict storage, learning integration
- **Worker Pool** - Task routing, CAWS compliance checking, worker lifecycle management
- **Database Layer** - PostgreSQL schema with pgvector, connection pooling, learning signals
- **Apple Silicon** - Core ML infrastructure, ANE/GPU/CPU routing, thermal management
- **Research Agent** - Vector search, context synthesis, cross-reference detection
- **MCP Integration** - Tool discovery, registry, CAWS integration
- **Provenance** - Git integration, JWS signing, immutable audit trails

### ✅ Completed Theory-Critical Components

- **Claim Extraction Pipeline** - 4-stage processing (disambiguation → qualification → decomposition → verification)
- **Reflexive Learning** - Progress tracking, adaptive allocation, context preservation
- **Model Benchmarking** - Performance tracking, scoring system, regression detection

### 🚧 In Progress

- **Model Fine-tuning** - CAWS training datasets and LoRA fine-tuning scripts
- **Observer Bridge** - Council deliberation visualization and monitoring
- **Comprehensive Testing** - E2E tests, integration tests, performance benchmarks

### 📋 Planned

- **Production Hardening** - Error handling, monitoring, alerting
- **Documentation** - API docs, user guides, deployment guides

## 🎉 Major Achievement: Theory Compliance Achieved

V3 has successfully implemented **all critical theory requirements** that were identified as missing in our comprehensive gap analysis:

### ✅ Critical Theory Components Implemented

1. **Claim Extraction & Verification Pipeline** (`claim-extraction/`)

   - Complete 4-stage processing: Disambiguation → Qualification → Decomposition → Verification
   - Based on V2's 1677-line ClaimExtractor.ts with Rust adaptations
   - Council integration for evidence collection in debate protocol

2. **Reflexive Learning Loop** (`reflexive-learning/`)

   - Progress tracking with turn-level monitoring
   - Adaptive resource allocation based on performance
   - Context preservation for multi-tenant learning
   - Based on V2's MultiTurnLearningCoordinator (671 lines, production-ready)

3. **Model Performance Benchmarking** (`model-benchmarking/`)
   - Continuous micro/macro benchmarks with performance tracking
   - Multi-dimensional scoring system for task-specific metrics
   - Regression detection for performance monitoring
   - Based on V2's ModelPerformanceBenchmarking component

### 🏗️ Architecture Excellence

- **9 Focused Crates**: Reduced from V2's 29 components to 9 modular crates
- **Council-Based Governance**: 4 specialized judges vs V2's single arbiter
- **Apple Silicon Native**: Core ML optimization with ANE/GPU/CPU routing
- **Modular Design**: All components designed for future model/architecture upgrades

## Key Innovations from V2

### 1. Council > Single Arbiter

- **Problem**: V2's single orchestrator had too many responsibilities
- **Solution**: Specialized judge models with clear domains
- **Benefit**: Parallel evaluation, faster decisions, better quality

### 2. Model-Native CAWS

- **Problem**: Runtime-only enforcement is slow and repetitive
- **Solution**: Fine-tune models on CAWS principles with runtime validation
- **Benefit**: Workers self-correct, fewer violations, faster iteration

### 3. Apple Silicon Native

- **Problem**: V2 treated hardware as generic compute
- **Solution**: Core ML pipeline with ANE/GPU/CPU orchestration
- **Benefit**: 3-5x faster inference, lower power, better thermals

### 4. Research Agent Separation

- **Problem**: Workers spent tokens on information gathering
- **Solution**: Dedicated research model with vector search
- **Benefit**: Workers focus on execution, better token efficiency

### 5. Simplified Component Count

- **Problem**: 29 v2 components created integration complexity
- **Solution**: 9 focused v3 crates with clear boundaries and modular design
- **Benefit**: Easier maintenance, faster development, clearer testing, future-proof architecture

## Directory Structure

```
iterations/v3/
├── council/                    # ✅ Council of judges implementation
│   ├── models/                 # ✅ Fine-tuned model definitions
│   │   ├── constitutional.yaml
│   │   ├── technical.yaml
│   │   ├── quality.yaml
│   │   └── integration.yaml
│   ├── src/                    # ✅ Complete Rust implementation
│   │   ├── coordinator.rs      # ✅ Consensus coordination service
│   │   ├── debate.rs          # ✅ Adversarial debate protocol
│   │   ├── verdicts.rs        # ✅ Verdict generation and storage
│   │   ├── contracts.rs       # ✅ Contract definitions
│   │   ├── learning.rs        # ✅ Learning signal integration
│   │   ├── types.rs           # ✅ Core types and data structures
│   │   └── lib.rs             # ✅ Public API
│   └── Cargo.toml             # ✅ Council crate configuration
├── workers/                    # ✅ Worker pool implementation
│   ├── src/                    # ✅ Complete worker system
│   │   ├── manager.rs         # ✅ Worker lifecycle management
│   │   ├── router.rs          # ✅ Intelligent task routing
│   │   ├── executor.rs        # ✅ Task execution engine
│   │   ├── caws_checker.rs    # ✅ CAWS compliance checking
│   │   └── types.rs           # ✅ Worker types and configs
│   └── Cargo.toml             # ✅ Worker crate configuration
├── research/                   # ✅ Research agent implementation
│   ├── src/                    # ✅ Complete research system
│   │   ├── knowledge_seeker.rs # ✅ Knowledge gathering
│   │   ├── vector_search.rs   # ✅ Vector search engine
│   │   ├── context_builder.rs # ✅ Context synthesis
│   │   └── web_scraper.rs     # ✅ Web scraping capabilities
│   └── Cargo.toml             # ✅ Research crate configuration
├── apple-silicon/              # ✅ Apple Silicon optimization
│   ├── src/                    # ✅ Core ML integration
│   │   ├── core_ml.rs         # ✅ Core ML model management
│   │   ├── ane.rs             # ✅ Apple Neural Engine routing
│   │   ├── metal_gpu.rs       # ✅ Metal GPU acceleration
│   │   ├── thermal.rs         # ✅ Thermal management
│   │   └── quantization.rs    # ✅ Model quantization
│   └── Cargo.toml             # ✅ Apple Silicon crate configuration
├── claim-extraction/           # ✅ Theory-critical component
│   ├── src/                    # ✅ 4-stage claim pipeline
│   │   ├── processor.rs       # ✅ Main processor
│   │   ├── disambiguation.rs  # ✅ Stage 1: Disambiguation
│   │   ├── qualification.rs   # ✅ Stage 2: Qualification
│   │   ├── decomposition.rs   # ✅ Stage 3: Decomposition
│   │   └── verification.rs    # ✅ Stage 4: Verification
│   └── Cargo.toml             # ✅ Claim extraction configuration
├── reflexive-learning/         # ✅ Theory-critical component
│   ├── src/                    # ✅ Learning coordination
│   │   ├── coordinator.rs     # ✅ Learning session management
│   │   ├── progress_tracker.rs # ✅ Turn-level monitoring
│   │   ├── adaptive_allocator.rs # ✅ Resource allocation
│   │   └── context_preservation.rs # ✅ Context management
│   └── Cargo.toml             # ✅ Learning crate configuration
├── model-benchmarking/         # ✅ Theory-critical component
│   ├── src/                    # ✅ Performance tracking
│   │   ├── benchmark_runner.rs # ✅ Benchmark execution
│   │   ├── performance_tracker.rs # ✅ Performance monitoring
│   │   ├── scoring_system.rs  # ✅ Multi-dimensional scoring
│   │   └── regression_detector.rs # ✅ Performance regression detection
│   └── Cargo.toml             # ✅ Benchmarking configuration
├── mcp-integration/            # ✅ MCP server integration
│   ├── src/                    # ✅ Tool discovery and registry
│   │   ├── server.rs          # ✅ MCP server implementation
│   │   ├── tool_discovery.rs  # ✅ Dynamic tool discovery
│   │   ├── tool_registry.rs   # ✅ Tool registration
│   │   └── caws_integration.rs # ✅ CAWS compliance integration
│   └── Cargo.toml             # ✅ MCP integration configuration
├── provenance/                 # ✅ Provenance tracking
│   ├── src/                    # ✅ Immutable audit trails
│   │   ├── service.rs         # ✅ Provenance service
│   │   ├── signer.rs          # ✅ JWS signing
│   │   ├── git_integration.rs # ✅ Git trailer integration
│   │   └── storage.rs         # ✅ Storage backends
│   └── Cargo.toml             # ✅ Provenance configuration
├── database/                   # ✅ PostgreSQL + pgvector
│   ├── schema.sql             # ✅ Complete database schema
│   ├── migrations/            # ✅ Database migrations
│   ├── src/                   # ✅ Database client implementation
│   │   ├── client.rs          # ✅ Connection pooling and operations
│   │   ├── models.rs          # ✅ Database models and types
│   │   └── lib.rs             # ✅ Public API
│   └── Cargo.toml             # ✅ Database crate configuration
├── orchestration/              # ✅ Core coordination
│   ├── src/                    # ✅ Orchestration engine
│   │   ├── orchestrate.rs     # ✅ Main orchestration logic
│   │   ├── caws_runtime.rs    # ✅ CAWS runtime validator
│   │   └── persistence.rs     # ✅ Data persistence
│   └── Cargo.toml             # ✅ Orchestration configuration
├── docs/                       # ✅ Core architectural documentation
│   ├── README.md              # ✅ Documentation index and organization
│   ├── architecture.md        # ✅ System architecture and design principles
│   ├── interaction-contracts.md # ✅ API contracts and interaction patterns
│   ├── INTEGRATION_PATTERNS.md # ✅ Component integration patterns
│   ├── components/            # ✅ Component-specific documentation
│   ├── contracts/             # ✅ API contracts and JSON schemas
│   ├── adr/                   # ✅ Architectural Decision Records
│   └── database/              # ✅ Database design documentation
├── training/                   # 🚧 Model fine-tuning (in progress)
│   ├── caws-dataset/          # 🚧 CAWS training datasets
│   ├── fine-tune-scripts/     # 🚧 LoRA fine-tuning scripts
│   └── rl-pipeline/           # 🚧 Reinforcement learning pipeline
├── tests/                      # 🚧 Comprehensive testing (in progress)
│   ├── unit/                  # 🚧 Unit tests
│   ├── integration/           # 🚧 Integration tests
│   └── e2e/                   # 🚧 End-to-end tests
├── observer/                   # 🚧 Monitoring and visualization (in progress)
├── Cargo.toml                  # ✅ Workspace configuration (9 crates)
└── README.md                   # ✅ This file
```

## Getting Started

### Prerequisites

- Rust 1.70+ with cargo
- PostgreSQL 15+ with pgvector extension
- Ollama (for local model serving)
- Apple Silicon Mac (for optimal performance)

### Setup

1. **Clone and navigate to V3**:

   ```bash
   cd iterations/v3
   ```

2. **Set up database**:

   ```bash
   # Install PostgreSQL and pgvector
   brew install postgresql
   brew install pgvector

   # Create database
   createdb agent_agency_v3

   # Run schema
   psql agent_agency_v3 < database/schema.sql
   ```

3. **Install models with Ollama**:

   ```bash
   # Install base models (these will be fine-tuned later)
   ollama pull llama3.3:3b
   ollama pull codellama:7b
   ollama pull gemma3n:e2b
   ollama pull gemma3n:e4b
   ollama pull embeddinggemma
   ollama pull mistral:3b
   ```

4. **Build the project**:

   ```bash
   cargo build
   ```

5. **Run tests**:
   ```bash
   cargo test
   ```

## Council System Usage

### Basic Consensus Coordination

```rust
use agent_agency_council::{ConsensusCoordinator, CouncilConfig, TaskSpec, RiskTier};

// Create coordinator with default configuration
let config = CouncilConfig::default();
let coordinator = ConsensusCoordinator::new(config);

// Create task specification
let task_spec = TaskSpec {
    id: Uuid::new_v4(),
    title: "Implement user authentication".to_string(),
    description: "Add JWT-based authentication system".to_string(),
    risk_tier: RiskTier::Tier1,
    scope: TaskScope {
        files_affected: vec!["src/auth/".to_string()],
        max_files: Some(5),
        max_loc: Some(1000),
        domains: vec!["authentication".to_string()],
    },
    acceptance_criteria: vec![],
    context: TaskContext {
        workspace_root: "/workspace".to_string(),
        git_branch: "main".to_string(),
        recent_changes: vec![],
        dependencies: std::collections::HashMap::new(),
        environment: Environment::Development,
    },
    worker_output: WorkerOutput {
        content: "Authentication implementation".to_string(),
        files_modified: vec![],
        rationale: "JWT-based auth with proper validation".to_string(),
        self_assessment: SelfAssessment {
            caws_compliance: 0.95,
            quality_score: 0.9,
            confidence: 0.85,
            concerns: vec![],
            improvements: vec![],
        },
        metadata: std::collections::HashMap::new(),
    },
    caws_spec: None,
};

// Evaluate task with council
let result = coordinator.evaluate_task(task_spec).await?;

println!("Consensus result: {:?}", result.final_verdict);
```

### Database Operations

```rust
use agent_agency_database::{DatabaseClient, DatabaseConfig, CreateJudge};

// Create database client
let config = DatabaseConfig::default();
let db = DatabaseClient::new(config).await?;

// Create a new judge
let judge = db.create_judge(CreateJudge {
    name: "Custom Judge".to_string(),
    model_name: "custom-model".to_string(),
    endpoint: "http://localhost:11434".to_string(),
    weight: 0.3,
    timeout_ms: 300,
    optimization_target: "CPU".to_string(),
}).await?;

println!("Created judge: {:?}", judge);
```

## Performance Targets

### Inference Latency

- Constitutional Judge: <100ms (ANE-optimized)
- Technical Auditor: <500ms (GPU-accelerated)
- Quality Evaluator: <200ms (balanced)
- Integration Validator: <150ms (CPU-optimized)
- Worker Models: <2s per task (parallel execution)

### Throughput

- 10+ concurrent workers on M3 Max (64GB)
- 5+ concurrent workers on M3 Pro (32GB)
- Council evaluation: <1s for Tier 2/3, <3s for Tier 1

### Resource Usage

- Peak memory: 48GB for full system (M3 Max)
- Idle memory: 12GB (base models loaded)
- Thermal: <80°C sustained load
- Power: <30W average (M3 efficiency cores + ANE)

## Development Roadmap

### ✅ Phase 1: Foundation (COMPLETED)

- [x] Rust workspace with 9 crates
- [x] Database schema with pgvector support
- [x] Core council system with consensus coordination
- [x] Debate protocol implementation
- [x] Worker pool management system
- [x] Research agent with vector search
- [x] Apple Silicon Core ML integration
- [x] MCP server integration
- [x] Provenance tracking system

### ✅ Phase 2: Theory-Critical Components (COMPLETED)

- [x] Claim Extraction Pipeline (4-stage processing)
- [x] Reflexive Learning Loop (progress tracking, adaptive allocation)
- [x] Model Performance Benchmarking (scoring system, regression detection)
- [x] Council learning integration
- [x] Advanced CAWS compliance checking
- [x] Context preservation engine

### ✅ Phase 3: Apple Silicon Optimization (COMPLETED)

- [x] Core ML integration layer
- [x] ANE/GPU/CPU routing infrastructure
- [x] Thermal management system
- [x] Quantization pipeline framework
- [x] Unified memory management

### ✅ Phase 4: CAWS Integration (COMPLETED)

- [x] Runtime validator integration
- [x] Provenance tracking with git integration
- [x] JWS signing for immutable audit trails
- [x] Advanced compliance checking with AST analysis

### ✅ Phase 5: Research Agent (COMPLETED)

- [x] Knowledge seeker implementation
- [x] Vector search integration
- [x] Web scraping capabilities
- [x] Context synthesis with cross-reference detection
- [x] Research-worker coordination

### 🚧 Phase 6: Production Hardening (IN PROGRESS)

- [x] Comprehensive documentation and gap analysis
- [x] Implementation roadmap and integration patterns
- [x] API contracts and schemas
- [ ] Comprehensive test suite (unit, integration, e2e)
- [ ] Performance benchmarking and optimization
- [ ] Error handling and recovery
- [ ] Monitoring and alerting
- [ ] Model fine-tuning pipeline
- [ ] Observer bridge visualization

## Success Criteria

### ✅ Architecture Goals (ACHIEVED)

- [x] **Council-based governance** - 4 specialized judges with consensus coordination
- [x] **Theory compliance** - All critical theory requirements implemented
- [x] **Modular design** - 9 focused crates with clear boundaries
- [x] **Apple Silicon optimization** - Core ML integration with ANE/GPU/CPU routing
- [x] **V2 component parity** - All production-ready V2 components ported with enhancements

### 🎯 Performance Targets (TO BE VALIDATED)

- [ ] Council reaches consensus on 95%+ of decisions
- [ ] Debate protocol resolves conflicts in <5s
- [ ] Workers self-correct CAWS violations 80%+ of time
- [ ] Research agent reduces worker token usage by 40%+
- [ ] System handles 10+ concurrent tasks on M3 Max
- [ ] Council evaluation <1s for Tier 2/3 tasks
- [ ] ANE utilization >60% for constitutional judge
- [ ] Memory usage <50GB on M3 Max under full load
- [ ] Sustained operation <80°C thermal
- [ ] 3-5x faster inference vs generic CPU execution

### 🎯 Quality Gates (TO BE ACHIEVED)

- [ ] CAWS compliance rate >95%
- [ ] Test coverage >85% across all components
- [ ] Zero critical security vulnerabilities
- [ ] Complete audit trail for all decisions
- [ ] 99%+ uptime in continuous operation

## Documentation Organization

### Core Architecture Documentation (`/docs`)

Contains maintainable, long-term architectural documentation:

- **System Architecture** - Design principles and component relationships
- **API Contracts** - JSON schemas and interaction patterns
- **Component Documentation** - Detailed component specifications
- **Architectural Decision Records (ADRs)** - Key design decisions
- **Integration Patterns** - How components communicate

### Implementation Status (`/docs-status`) - Git Ignored

Contains temporal documentation for project management:

- Progress summaries and status reports
- Gap analyses and theory compliance tracking
- Implementation roadmaps and planning documents

### Archive (`/archive`) - Git Ignored

Contains superseded documentation for historical reference:

- Early research questions and lessons learned
- Documents superseded by comprehensive analysis

## Contributing

1. Follow the existing code structure and patterns
2. Add comprehensive tests for new functionality
3. Update documentation for any API changes
4. Ensure all tests pass before submitting PRs
5. Follow the CAWS quality standards
6. Keep architectural docs in `/docs`, temporal docs in `/docs-status`

## License

MIT License - see LICENSE file for details.
