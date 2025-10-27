# Agent Workers

**Location**: `agent-workers` crate

## Purpose

Parallel task execution and MCP-based worker management for scalable, fault-tolerant agent operations across multiple execution contexts.

## Core Responsibilities

### Parallel Task Execution
- Resource-aware worker allocation and load balancing
- MCP (Model Context Protocol) integration for standardized agent communication
- Fault-tolerant execution with circuit breaker patterns
- Result aggregation and processing from multiple worker instances

### Worker Management
- Dynamic worker discovery and lifecycle management
- Specialization-based routing (generalist vs specialist workers)
- Performance monitoring and optimization
- Capacity planning and scaling coordination

### Execution Coordination
- Deterministic execution with injected time/uuid/random for testability
- Concurrency control and task cancellation capabilities
- Structured outputs with rationale and self-assessment
- Provenance tracking for execution audit trails

## Key Features

### MCP-Based Worker Protocol
- **Standardized Communication**: Model Context Protocol for consistent agent interactions
- **Tool Discovery**: Automatic discovery and management of worker capabilities
- **Cross-Model Coordination**: Seamless coordination between different AI models
- **Extensible Architecture**: Plugin-based worker registration and management

### Parallel Execution Engine
- **Load Balancing**: Intelligent distribution of tasks across available workers
- **Resource Optimization**: CPU, memory, and GPU resource allocation
- **Fault Isolation**: Independent failure domains for worker instances
- **Result Aggregation**: Intelligent merging of outputs from parallel executions

### Quality Assurance Integration
- **Contract Compliance**: Enforcement of scope boundaries and execution budgets
- **Provenance Tracking**: Complete audit trails for execution provenance
- **Quality Metrics**: Execution quality assessment and reporting
- **Deterministic Testing**: Reproducible execution for testing and validation

## Integration Points

### Input Sources
- **agent-orchestration**: Task assignments with execution parameters and risk tiers
- **agent-research**: Context packages and research findings for task execution
- **system-quality-security**: Quality gates and compliance requirements
- **data-infrastructure**: Access to execution history and performance data

### Output Destinations
- **agent-orchestration**: Task execution results and status updates
- **data-infrastructure**: Execution results and provenance storage
- **system-observability**: Performance metrics and execution monitoring
- **agent-memory**: Learning data from execution outcomes

### Cross-Crate Dependencies
- **agent-agency-contracts**: Task execution contracts and result schemas
- **system-quality-security**: Quality validation and provenance tracking
- **system-resilience**: Fault tolerance and recovery mechanisms
- **system-observability**: Performance monitoring and alerting

## Performance Characteristics

- **Concurrent Executions**: Support for 100+ simultaneous task executions
- **Worker Throughput**: 1000+ tasks per minute across distributed workers
- **Response Time**: Sub-100ms task routing and assignment
- **Fault Recovery**: Automatic failover in <5 seconds

## Implementation Example

```rust
use agent_workers::*;
use agent_agency_contracts::{TaskExecution, WorkerResult};

pub async fn execute_parallel_task(
    task: TaskExecution,
    worker_pool: &WorkerPool,
) -> Result<TaskResult, ExecutionError> {
    // 1. Worker selection based on task requirements
    let selected_workers = worker_pool.select_workers(&task).await?;

    // 2. Parallel execution across selected workers
    let execution_handles = selected_workers
        .iter()
        .map(|worker| worker.execute_task(&task))
        .collect::<Vec<_>>();

    // 3. Result aggregation with fault tolerance
    let results = futures::future::join_all(execution_handles).await;
    let successful_results = results
        .into_iter()
        .filter_map(|r| r.ok())
        .collect::<Vec<WorkerResult>>();

    // 4. Intelligent result aggregation
    let aggregated_result = worker_pool.aggregate_results(successful_results).await?;

    // 5. Quality validation and provenance tracking
    let validated_result = worker_pool.validate_and_track(&aggregated_result).await?;

    Ok(TaskResult::from(validated_result))
}
```

## Worker Types and Specialization

### Generalist Workers
- **Purpose**: Broad task execution across multiple domains
- **Capabilities**: Text processing, general reasoning, basic analysis
- **Use Cases**: Standard task execution, prototyping, general-purpose work
- **Scaling**: Horizontal scaling for high-throughput scenarios

### Specialist Workers
- **Purpose**: Domain-specific task execution with specialized capabilities
- **Capabilities**: Code analysis, image processing, mathematical reasoning
- **Use Cases**: Complex domain-specific tasks requiring expertise
- **Scaling**: Targeted scaling based on specialized workload demands

### MCP Tool Workers
- **Purpose**: Standardized tool execution via Model Context Protocol
- **Capabilities**: Tool discovery, parameter validation, result processing
- **Use Cases**: Integration with external tools and services
- **Scaling**: Dynamic scaling based on tool availability and demand

## Execution Modes

### Deterministic Mode
- **Purpose**: Reproducible execution for testing and validation
- **Implementation**: Injected time, UUID, and random number generators
- **Benefits**: Consistent test results, debugging capabilities
- **Use Cases**: Automated testing, validation, and quality assurance

### Concurrent Mode
- **Purpose**: High-throughput parallel execution
- **Implementation**: Asynchronous task distribution and result aggregation
- **Benefits**: Maximum throughput, resource utilization
- **Use Cases**: Batch processing, high-volume task execution

### Interactive Mode
- **Purpose**: Real-time execution with intervention capabilities
- **Implementation**: Streaming results and cancellation support
- **Benefits**: Human oversight, dynamic adaptation
- **Use Cases**: Critical tasks, debugging, manual oversight

## See Also

- **[../system-overview.md](../system-overview.md)** - Complete system architecture
- **[../contracts/worker-output.schema.json](../contracts/worker-output.schema.json)** - Worker output contracts
- **[../testing/testing-strategy.md](../testing/testing-strategy.md)** - Worker testing approach

