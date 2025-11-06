# V3 Constitutional AI System Architecture

## Overview

Agent Agency V3 implements a **modular constitutional AI system** with 17 specialized crates providing comprehensive autonomous agent capabilities. The system leverages CoreML-optimized Mistral models with Apple Neural Engine acceleration for high-performance local inference, ensuring ethical compliance, technical quality, and system coherence through evidence-based decision making.

The architecture consists of specialized Rust crates with clear responsibilities, communicating via well-defined contracts, with PostgreSQL persistence, comprehensive provenance tracking, and monitoring capabilities.

## Core Problems Solved

### Autonomous Agent Safety & Governance
Modern AI agent systems lack robust safety mechanisms and accountability:

- **Execution Safety**: No safe modes for testing agent behavior without real-world impact
- **Quality Assurance**: Manual code review doesn't scale with autonomous operations
- **Intervention Gaps**: Limited ability to pause, modify, or cancel running agent tasks
- **Audit Trail Deficits**: Poor traceability of agent decisions and actions
- **Compliance Challenges**: Difficulty ensuring ethical and legal compliance in autonomous operations

V3 provides complete governance through constitutional oversight, execution modes, and comprehensive monitoring.

## Core Design Principles

### 1. Execution Mode Safety
**Problem**: Autonomous agents need safe testing without real-world impact.

**Solution**: Three execution modes provide graduated safety levels:
- **Dry-Run Mode**: Complete simulation without filesystem changes
- **Auto Mode**: Automatic execution with quality gate validation
- **Strict Mode**: Manual approval required for each execution phase

**Benefits**:
- Safe testing and validation of agent behavior
- Risk-appropriate governance intensity
- Graduated trust model for agent operations
- Protection against unintended consequences

### 2. Real-time Intervention & Control
**Problem**: Running autonomous tasks lack control mechanisms.

**Solution**: Comprehensive intervention API for task lifecycle management.

**Control Capabilities**:
- **Pause/Resume**: Suspend and restart task execution
- **Cancel**: Terminate running tasks gracefully
- **Override**: Modify task parameters and verdicts
- **Guidance Injection**: Provide additional context during execution

**Benefits**:
- Human oversight of critical operations
- Emergency response capabilities
- Dynamic adaptation to changing requirements
- Audit trail of all interventions

### 3. Constitutional Council Governance
**Problem**: Need for ethical and quality oversight in autonomous operations.

**Solution**: Four-judge council system for governance oversight.

**Judge Framework**:
- **Constitutional Judge**: Ethical compliance and CAWS validation
- **Technical Auditor**: Code quality and security standards
- **Quality Evaluator**: Requirements satisfaction and correctness
- **Integration Validator**: System coherence and architectural integrity

**Benefits**:
- Multi-dimensional quality assessment
- Ethical compliance enforcement
- Automated quality gate validation
- Structured decision-making framework

### 4. Provenance & Audit Tracking
**Problem**: Lack of accountability in autonomous agent operations.

**Solution**: Complete provenance tracking with Git integration.

**Tracking Features**:
- **Cryptographic Signing**: JWS-signed provenance records
- **Git Integration**: Commit trailers for traceability
- **Audit Trails**: Complete decision and action history
- **Compliance Verification**: Runtime validation capabilities

**Benefits**:
- Full accountability for agent actions
- Regulatory compliance support
- Incident investigation capabilities
- Quality assurance through traceability

### 5. CoreML Safety Architecture
**Problem**: Send/Sync violations when integrating CoreML FFI with async Rust.

**Solution**: Thread-confined CoreML operations with channel-based communication.

**Safety Architecture**:
- **Thread Confinement**: Raw CoreML pointers isolated to dedicated threads
- **Opaque References**: `ModelRef(u64)` identifiers safe to send across threads
- **Channel Communication**: Async coordination via `crossbeam::channel`
- **Registry System**: Thread-local mapping of references to handles

**Benefits**:
- Zero Send/Sync violations in async contexts
- Memory safety with proper resource cleanup
- High-performance inference with minimal overhead
- Safe integration with constitutional council operations

## Agent Evaluation Framework

Agent Agency V3 provides a comprehensive, multi-dimensional evaluation framework for measuring autonomous agent intelligence and performance. Unlike traditional software testing that focuses on binary success/failure outcomes, this framework evaluates the **quality of the agent's thinking process** across five key dimensions.

### Evaluation Dimensions

#### 1. Functional Correctness (30% weight)
**What**: Did the agent solve the core problem?
**Metrics**:
- Code compilation success
- Functional requirements satisfaction
- No regressions introduced
- API contract compliance

#### 2. Process Quality (25% weight)
**What**: How well did the agent think through the problem?
**Metrics**:
- **Reasoning Depth**: Thoroughness of problem analysis (alternatives considered, evidence gathering)
- **Decision Quality**: Soundness of decision-making process
- **Risk Assessment**: Identification and mitigation of potential issues
- **Coordination Quality**: Effectiveness of component interaction
- **Iterative Improvement**: Learning from partial successes

#### 3. Adaptability (20% weight)
**What**: How well did the agent handle uncertainty and change?
**Metrics**:
- **Uncertainty Management**: Handling ambiguity and unknowns
- **Failure Recovery**: Graceful handling of setbacks
- **Strategy Flexibility**: Ability to switch approaches when needed
- **Learning Velocity**: Speed of adaptation to new information
- **Resource Adaptation**: Efficient resource allocation under changing conditions

#### 4. Safety (15% weight)
**What**: Did the agent avoid dangerous actions?
**Metrics**:
- **Risk Avoidance**: No destructive or unauthorized operations
- **Error Handling**: Proper error recovery and meaningful error information
- **Boundary Compliance**: Respect for operational constraints
- **Recovery Safety**: Safe execution of recovery procedures
- **Audit Completeness**: Full traceability of all actions

#### 5. Efficiency (10% weight)
**What**: Resource usage relative to problem complexity
**Metrics**:
- Time to solution vs. problem complexity
- Resource consumption (CPU, memory, API calls)
- Balance of thoroughness vs. excessive resource usage

### Chain-of-Thought Evaluation

Every agent decision is captured and analyzed through comprehensive tracing:

```rust
DecisionPoint {
    decision_type: DecisionType::WorkerAssignment,
    reasoning: "Worker A has 85% capability match and lower load vs Worker B's 72%",
    alternatives: ["Worker A (85%)", "Worker B (72%)", "Worker C (91%)"],
    chosen_option: "Worker A",
    confidence: 0.8,
    risk_assessment: Some("Worker A has higher utilization - monitor closely")
}
```

**Analysis Criteria**:
- **Reasoning Completeness**: Detailed explanation of decision rationale
- **Alternatives Evaluation**: Multiple approaches properly considered
- **Confidence Calibration**: Realistic confidence estimates
- **Risk Awareness**: Proactive identification of potential issues

### Coordination Event Analysis

Component interactions are tracked for effectiveness evaluation:

```rust
CoordinationEvent {
    event_type: CoordinationEventType::PlanStarted,
    task_id: Some(task_uuid),
    milestone_id: Some("M1"),
    worker_id: Some(worker_uuid),
    resource_id: Some("cpu-core-1"),
    details: { "parallel_limit": "4", "priority": "high" }
}
```

**Coordination Metrics**:
- **Event Diversity**: Range of coordination patterns used
- **Temporal Distribution**: Events spread appropriately over time
- **Resource Awareness**: Proper resource tracking and allocation
- **Dependency Management**: Clear handling of task prerequisites

### Scenario-Based Testing

Agents are evaluated in controlled **playground environments** with known issues:

```rust
EvaluationScenario {
    scenario_id: "memory-leak-fix",
    difficulty: ScenarioDifficulty::Intermediate,
    problem_type: ProblemType::CompilationError,
    expected_behaviors: [
        ExpectedBehavior {
            behavior: "error_diagnosis",
            importance: BehaviorImportance::Critical,
            description: "Correctly identify root cause of compilation errors"
        },
        ExpectedBehavior {
            behavior: "solution_exploration",
            importance: BehaviorImportance::Important,
            description: "Consider multiple potential fix approaches"
        },
        ExpectedBehavior {
            behavior: "risk_assessment",
            importance: BehaviorImportance::Important,
            description: "Evaluate safety and side effects of proposed changes"
        }
    ],
    evaluation_criteria: [
        EvaluationCriterion {
            criterion: "functional_correctness",
            metric: "Code compiles and works after agent intervention",
            weight: 0.3,
            scoring_guide: "1.0 = Perfect fix, 0.8 = Good fix with minor issues, 0.0 = No improvement"
        }
    ]
}
```

### Performance Baselines

**Evaluation ranges establish performance expectations**:

- **0.9-1.0**: Exceptional - Exceeds expectations, shows advanced intelligence
- **0.8-0.9**: Strong - Handles complex scenarios well, good decision quality
- **0.7-0.8**: Good - Reliable for most scenarios, solid process quality
- **0.6-0.7**: Adequate - Works for simple cases, room for improvement
- **0.4-0.6**: Developing - Basic functionality, needs significant improvement
- **0.0-0.4**: Poor - Major issues with process or functionality

### Learning and Improvement Tracking

The framework measures **continuous improvement** over time:

- **Pattern Recognition**: Does the agent identify similar problems?
- **Solution Generalization**: Can solutions be applied to new contexts?
- **Feedback Integration**: How well does the agent incorporate results?
- **Self-Optimization**: Does the agent proactively improve?

### Automated Evaluation Engine

```rust
// Comprehensive evaluation with full tracing
let evaluation = engine.evaluate_scenario(
    &scenario_id,
    &chain_of_thought_data,
    &coordination_events,
    &audit_trail_entries
).await?;

println!("Overall Score: {:.2}", evaluation.overall_score);
println!("Process Quality: {:.2}", evaluation.dimensions.process_quality);
println!("Adaptability: {:.2}", evaluation.dimensions.adaptability);
println!("Safety: {:.2}", evaluation.dimensions.safety);
```

## System Architecture Diagram

```mermaid
flowchart LR
  subgraph User
    CLI[CLI Client<br/>execute, intervene, monitor]
    API[REST API<br/>task submission, control]
    Web[Web Dashboard<br/>monitoring, database]
  end

  subgraph API_Server[API Server]
    TaskAPI[Task Management<br/>submit, monitor, control]
    InterventionAPI[Intervention API<br/>pause, resume, override]
    WaiverAPI[Waiver System<br/>exceptions, approvals]
    ProvenanceAPI[Provenance API<br/>audit trails, verification]
    SLOAPI[SLO Monitoring<br/>objectives, alerts]
  end

  subgraph Orchestration[Orchestration Engine]
    TaskRouter[Task Router<br/>mode selection, distribution]
    ProgressTracker[Progress Tracker<br/>real-time status, metrics]
    CircuitBreaker[Circuit Breaker<br/>fault tolerance]
  end

  subgraph Council[Council System]
    Constitutional[Constitutional Judge<br/>ethics, compliance]
    Technical[Technical Auditor<br/>code quality, security]
    Quality[Quality Evaluator<br/>requirements fit]
    Integration[Integration Validator<br/>system coherence]
  end

  subgraph CoreML[CoreML Safety Layer]
    ModelClient[Model Client<br/>Send/Sync interface]
    InferenceThread[Inference Thread<br/>FFI operations]
    Registry[Thread Registry<br/>ModelRef → Handle]
    Channel[Channel Comm<br/>crossbeam::channel]
  end

  subgraph Workers[Worker Pool]
    Executor[Task Executor<br/>HTTP client, retries]
    WorkerService[Worker Service<br/>task execution, simulation]
  end

  subgraph Infrastructure
    Database[(PostgreSQL<br/>tasks, provenance, waivers)]
    Git[Git Repository<br/>provenance trailers]
    CAWS[CAWS Tools<br/>compliance validation]
  end

  CLI --> API_Server
  API --> API_Server
  Web --> API_Server

  API_Server --> Orchestration
  Orchestration --> Council
  Orchestration --> Workers
  Workers --> Orchestration

  Council --> CoreML
  CoreML --> Council

  Orchestration --> Database
  Council --> Database
  API_Server --> Database

  API_Server --> Git
  Council --> CAWS
```

## Component Implementation Status

### Core Services (Operational)

**API Server (`api-server/`)**
- RESTful task management with authentication
- Intervention endpoints for real-time control
- Waiver system for quality gate exceptions
- Basic provenance tracking and verification APIs
- SLO monitoring framework and basic alerts

**Worker Service (`worker/`)**
- HTTP-based task execution with simulation modes
- Circuit breaker pattern for fault tolerance
- Configurable execution timeouts and retries
- Support for Dry-Run, Auto, and Strict modes

**CLI Tool (`cli/`)**
- Multi-mode task execution (strict/auto/dry-run)
- Real-time intervention commands
- Waiver management and approval workflows
- Basic provenance tracking with Git integration

**Orchestration Engine (`orchestration/`)**
- Task routing with execution mode enforcement
- Progress tracking with real-time metrics
- Circuit breaker integration for resilience
- Council coordination for governance

**Database Layer (`database/`)**
- PostgreSQL with ACID-compliant transactions
- Task lifecycle and provenance storage
- Waiver management and audit trails
- Connection pooling and migration framework

### Governance Components

**Council System (`council/`)**
- Four-judge constitutional oversight framework (logic partially implemented)
- Risk-tiered evaluation framework (T1/T2/T3)
- Consensus-based decision making structure
- CAWS compliance validation integration (basic)

**CAWS Integration (`../apps/tools/caws/`)**
- Runtime compliance validation
- Quality gate enforcement with waivers
- Working specification validation
- Automated testing and linting

**Provenance System (`provenance/`)**
- Git-backed audit trails with JWS signing
- Commit trailer integration
- Basic cryptographic verification capabilities
- Compliance and regulatory reporting framework

### AI/ML Safety Components (Operational)

**CoreML Safety Layer (`council/src/model_client.rs`)**
- Thread-confined FFI operations preventing Send/Sync violations
- Opaque `ModelRef(u64)` identifiers for safe cross-thread communication
- Channel-based async coordination with dedicated inference threads
- Thread-local registry system for handle management

**Apple Silicon Integration (`apple-silicon/`)**
- Safe CoreML model loading and inference
- Memory-safe tensor operations with bounds validation
- FFI boundary control with comprehensive error handling
- ANE acceleration support for vision processing

### Monitoring & Control

**Web Dashboard (`../apps/web-dashboard/`)**
- Real-time task monitoring and basic metrics
- Database exploration with query builder
- SLO status and alert management framework
- Basic system health visualization

**Progress Tracking (`orchestration/tracking/`)**
- Real-time task status updates
- Execution metrics and performance data
- Intervention state management
- Historical progress analysis

### Infrastructure Components

**Configuration (`config/`)**
- Environment-based configuration management
- Validation and type safety
- Hot-reloading capabilities
- Multi-environment support

**Observability (`observability/`)**
- Metrics collection and aggregation
- SLO definition and monitoring
- Alert generation and management
- Performance trend analysis

**Security (`security/`)**
- API key authentication
- Rate limiting and request throttling
- Audit logging and compliance
- Secure communication patterns

### Key Architectural Patterns

**Execution Mode Safety:**
- Three-tiered safety model (Dry-Run, Auto, Strict)
- Risk-appropriate governance intensity
- Safe testing without real-world impact

**Real-time Intervention:**
- HTTP-based task control APIs
- Graceful pause/resume/cancel operations
- Dynamic parameter modification
- Audit trails for all interventions

**Constitutional Governance:**
- Four-judge council oversight framework
- Consensus-based decision validation
- Ethical and quality compliance enforcement
- Runtime CAWS integration

**Provenance & Accountability:**
- Git-backed audit trails with JWS signing
- Cryptographic verification capabilities
- Complete decision traceability
- Regulatory compliance support

## Data Flow Architecture

### Task Execution Flow
1. **Task Submission**: CLI/API receives task with execution mode
2. **Validation**: Council validates against CAWS compliance
3. **Routing**: Orchestrator selects appropriate worker pool
4. **Execution**: Worker processes task with mode-appropriate safety
5. **Monitoring**: Progress tracker provides real-time updates
6. **Intervention**: Human operators can pause/resume/cancel as needed
7. **Completion**: Results stored with full provenance trail

### Governance Flow
1. **Constitutional Review**: Four-judge council evaluates task ethics
2. **Technical Audit**: Code quality and security validation
3. **Quality Assessment**: Requirements satisfaction verification
4. **Integration Check**: System coherence and compatibility validation
5. **Consensus Decision**: Final verdict with audit trail
6. **Waiver Processing**: Exception handling for edge cases

## Performance Characteristics

### Task Execution Performance
- **API Response Time**: <100ms for task submission
- **Worker Communication**: <50ms HTTP round-trip latency
- **Progress Updates**: Real-time streaming (<1s updates)
- **Intervention Commands**: <200ms command execution
- **Task Completion**: 10-30 seconds for typical operations

### System Throughput
- **Concurrent Tasks**: 50+ simultaneous executions
- **API Requests**: 1000+ requests/minute sustained
- **Database Queries**: <10ms average response time
- **Monitoring Updates**: Real-time with <500ms latency

### Reliability Metrics
- **Uptime**: 99.5%+ availability with circuit breakers
- **Error Recovery**: Automatic retry with exponential backoff
- **Data Consistency**: ACID transactions across all operations
- **Monitoring Coverage**: 100% of critical system components

## See Also

- **[system-overview.md](./system-overview.md)** - Complete system capabilities and status
- **[interaction-contracts.md](./interaction-contracts.md)** - API contracts and data schemas
- **[build-optimization.md](./build-optimization.md)** - Build performance and optimization
- **[database/README.md](../database/README.md)** - Database schema and operations
- **[../../docs/quality-assurance/README.md](../../docs/quality-assurance/README.md)** - CAWS and testing framework
- **[../../docs/agents/full-guide.md](../../docs/agents/full-guide.md)** - CAWS workflow implementation guide
