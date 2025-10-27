# Agent Orchestration

**Location**: `agent-orchestration` crate

## Purpose

Coordinate task execution, manage council-based governance decisions, and enforce quality gates across the agent system.

## Core Responsibilities

### Task Coordination
- Receive tasks from data-interfaces
- Route tasks to appropriate agent-workers based on requirements
- Manage task lifecycle from submission to completion
- Handle task cancellation and intervention requests

### Council Governance
- Coordinate council evaluation across governance components
- Aggregate verdicts from distributed quality gates
- Implement risk-tiered execution strategies
- Manage debate protocols for conflicting verdicts

### Quality Assurance
- Enforce CAWS compliance validation
- Coordinate with system-quality-security for quality gates
- Ensure contract compliance via agent-agency-contracts
- Track provenance and audit trails

## Key Components

### Task Router
- Analyzes task requirements and constraints
- Selects optimal worker allocation strategy
- Considers resource availability and specialization
- Implements load balancing across worker pools

### Council Coordinator
- Orchestrates governance evaluation process
- Coordinates with system-quality-security for compliance
- Aggregates verdicts from multiple governance perspectives
- Implements risk-appropriate decision strategies

### Execution Manager
- Manages task execution lifecycle
- Handles retries, timeouts, and failure recovery
- Coordinates with agent-workers for task processing
- Monitors execution progress and intervenes as needed

## Integration Points

### Input Sources
- **data-interfaces**: Task submissions and intervention requests
- **agent-research**: Strategic planning and optimization inputs
- **system-quality-security**: Quality gate results and compliance data

### Output Destinations
- **agent-workers**: Task assignments and execution coordination
- **data-infrastructure**: Task results and provenance updates
- **system-observability**: Metrics and monitoring data

### Cross-Crate Dependencies
- **agent-agency-contracts**: Type definitions and validation
- **system-quality-security**: Quality gate enforcement
- **data-infrastructure**: Persistence and audit trails
- **system-observability**: Performance monitoring

## Performance Characteristics

- **Task Routing**: <50ms decision time
- **Council Coordination**: <200ms governance cycle
- **Quality Gate Integration**: <100ms validation
- **Provenance Updates**: <150ms audit trail recording

## Implementation Example

```rust
use agent_orchestration::*;
use agent_agency_contracts::TaskContract;
use system_quality_security::QualityGate;

pub async fn orchestrate_task_execution(
    task: TaskContract,
    quality_gate: &QualityGate,
) -> Result<TaskResult, OrchestrationError> {
    // 1. Quality gate validation
    let compliance_result = quality_gate.validate_task(&task).await?;

    if !compliance_result.passed {
        return Err(OrchestrationError::QualityGateFailure);
    }

    // 2. Council evaluation
    let council_verdict = self.coordinate_council_evaluation(&task).await?;

    if !council_verdict.approved {
        return Err(OrchestrationError::CouncilRejection(council_verdict.reasoning));
    }

    // 3. Worker routing and execution
    let execution_result = self.route_and_execute_task(task, council_verdict).await?;

    // 4. Provenance recording
    self.record_execution_provenance(&execution_result).await?;

    Ok(execution_result)
}
```

## Metrics and Monitoring

- **council_eval_ms**: Council evaluation duration (p50/p95)
- **task_routing_ms**: Task routing decision time
- **execution_coordination_ms**: Cross-worker coordination time
- **quality_gate_integration_ms**: Quality validation duration
- **provenance_recording_ms**: Audit trail update time

## See Also

- **[../system-overview.md](../system-overview.md)** - Complete system architecture
- **[../contracts/task-request.schema.json](../contracts/task-request.schema.json)** - Task contract specification
- **[../contracts/final-verdict.schema.json](../contracts/final-verdict.schema.json)** - Verdict contract specification
