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

### ✅ Completed

- **Directory Structure** - Complete V3 project layout
- **Rust Workspace** - Cargo workspace with member crates
- **Council System** - Core types, consensus coordinator, debate protocol, verdict storage
- **Model Specifications** - YAML specs for all 4 judge models with training datasets
- **Database Schema** - Simplified PostgreSQL schema with pgvector support
- **Database Client** - Connection pooling and basic operations

### 🚧 In Progress

- **Worker Pool Implementation** - Task routing and CAWS self-check utilities
- **Core ML Integration** - ANE/GPU/CPU routing and quantization
- **Research Agent** - Vector search and context synthesis

### 📋 Planned

- **Observer Bridge** - Council deliberation visualization
- **Comprehensive Testing** - Council behavior, CAWS compliance, performance tests
- **Model Fine-tuning** - CAWS training datasets and fine-tuning scripts

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
- **Solution**: 15-20 focused v3 components with clear boundaries
- **Benefit**: Easier maintenance, faster development, clearer testing

## Directory Structure

```
iterations/v3/
├── council/                    # Council of judges implementation
│   ├── models/                 # Fine-tuned model definitions
│   │   ├── constitutional.yaml
│   │   ├── technical.yaml
│   │   ├── quality.yaml
│   │   └── integration.yaml
│   ├── src/                    # Rust implementation
│   │   ├── coordinator.rs      # Consensus coordination service
│   │   ├── debate.rs          # Adversarial debate protocol
│   │   ├── verdicts.rs        # Verdict generation and storage
│   │   ├── types.rs           # Core types and data structures
│   │   └── lib.rs             # Public API
│   └── Cargo.toml             # Council crate configuration
├── workers/                    # Worker pool implementation (planned)
├── research/                   # Research agent (planned)
├── orchestration/              # Core coordination (planned)
├── apple-silicon/              # Platform optimization (planned)
├── caws/                       # CAWS enforcement (planned)
├── database/                   # PostgreSQL + pgvector
│   ├── schema.sql             # Complete database schema
│   ├── src/                   # Database client implementation
│   │   ├── client.rs          # Connection pooling and operations
│   │   ├── models.rs          # Database models and types
│   │   └── lib.rs             # Public API
│   └── Cargo.toml             # Database crate configuration
├── observer/                   # Monitoring and visualization (planned)
├── training/                   # Model fine-tuning (planned)
├── tests/                      # Comprehensive testing (planned)
├── docs/                       # Documentation (planned)
├── Cargo.toml                  # Workspace configuration
└── README.md                   # This file
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
   ollama pull gemma2:3b
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

### Phase 1: Foundation ✅

- [x] Core Rust services (router, coordinator, memory manager)
- [x] Database schema and migration system
- [x] Basic council system with consensus coordination
- [x] Debate protocol implementation

### Phase 2: Council 🚧

- [x] Complete judge implementation (4 specialized models)
- [x] Debate protocol implementation
- [x] Verdict generation and storage
- [x] Consensus coordination
- [ ] Council behavior tests

### Phase 3: Apple Silicon 📋

- [ ] Core ML integration layer
- [ ] Model quantization pipeline
- [ ] ANE/GPU/CPU routing
- [ ] Unified memory management
- [ ] Thermal monitoring

### Phase 4: CAWS Integration 📋

- [ ] CAWS training dataset generation
- [ ] Model fine-tuning on CAWS principles
- [ ] Runtime validator integration
- [ ] Provenance tracking
- [ ] Hybrid enforcement testing

### Phase 5: Research Agent 📋

- [ ] Knowledge seeker implementation
- [ ] Vector search integration
- [ ] Web scraping capabilities
- [ ] Context synthesis
- [ ] Research-worker coordination

### Phase 6: Production Hardening 📋

- [ ] Comprehensive test suite
- [ ] Performance benchmarking
- [ ] Error handling and recovery
- [ ] Monitoring and alerting
- [ ] Documentation

## Success Criteria

### Functional

- [ ] Council reaches consensus on 95%+ of decisions
- [ ] Debate protocol resolves conflicts in <5s
- [ ] Workers self-correct CAWS violations 80%+ of time
- [ ] Research agent reduces worker token usage by 40%+
- [ ] System handles 10+ concurrent tasks on M3 Max

### Performance

- [ ] Council evaluation <1s for Tier 2/3 tasks
- [ ] ANE utilization >60% for constitutional judge
- [ ] Memory usage <50GB on M3 Max under full load
- [ ] Sustained operation <80°C thermal
- [ ] 3-5x faster inference vs generic CPU execution

### Quality

- [ ] CAWS compliance rate >95%
- [ ] Test coverage >85% across all components
- [ ] Zero critical security vulnerabilities
- [ ] Complete audit trail for all decisions
- [ ] 99%+ uptime in continuous operation

## Contributing

1. Follow the existing code structure and patterns
2. Add comprehensive tests for new functionality
3. Update documentation for any API changes
4. Ensure all tests pass before submitting PRs
5. Follow the CAWS quality standards

## License

MIT License - see LICENSE file for details.
