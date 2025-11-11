# API Observational Design - Research Integrity Guarantee

## Overview

The Agent Agency V3 API is designed as a **purely observational interface** - a "doctor's MRI machine" that allows you to see what's happening inside the orchestrator without directly controlling it. This design preserves research integrity by ensuring the orchestrator maintains full autonomy over its execution lifecycle.

## Core Principle

**The API observes and requests. It never manipulates.**

## Design Principles

### 1. Observation Only

All endpoints observe orchestrator state. They never manipulate it directly.

**Examples:**
- `GET /api/v1/tasks/:id/status` - Observes task status
- `GET /api/v1/tasks/:id/chain-of-thought` - Observes reasoning process
- `GET /api/v1/tasks/:id/council-decisions` - Observes council verdicts
- `GET /api/v1/tasks/:id/worker-actions` - Observes worker activities

### 2. Request-Based Control

Control operations (pause/resume/cancel) are **requests**, not commands.

**How it works:**
1. API receives control request (e.g., pause task)
2. Request is logged in chain-of-thought for auditability
3. Request is forwarded to orchestrator
4. Orchestrator decides whether to honor the request based on execution safety
5. API observes the result

**Examples:**
- `POST /api/v1/tasks/:id/pause` - Requests pause (orchestrator decides)
- `POST /api/v1/tasks/:id/resume` - Requests resume (orchestrator decides)
- `POST /api/v1/tasks/:id/cancel` - Requests cancellation (orchestrator decides)

### 3. Research Integrity

No direct manipulation of execution state ensures:
- Orchestrator decisions are autonomous and reproducible
- Research results are not contaminated by external manipulation
- The orchestrator's chain of thought accurately reflects its own reasoning
- Agents maintain their own execution connections independently

### 4. Agent Autonomy

Agents use their own connections to task execution, not through the API. The API is purely for human observation and monitoring.

## API Endpoint Categories

### Observational Endpoints (Read-Only)

These endpoints only observe state - they never change anything:

- **Task Status**: `GET /api/v1/tasks/:id/status`
- **Chain of Thought**: `GET /api/v1/tasks/:id/chain-of-thought`
- **Council Decisions**: `GET /api/v1/tasks/:id/council-decisions`
- **Worker Actions**: `GET /api/v1/tasks/:id/worker-actions`
- **Task Logs**: `GET /api/v1/tasks/:id/logs`
- **Task Progress**: `GET /api/v1/tasks/:id/progress`
- **Task Events**: `GET /api/v1/tasks/:id/events`
- **Analytics**: `GET /api/v1/analytics/*`
- **System Health**: `GET /api/v1/system/health`
- **Database Inspection**: `GET /api/v1/database/*`

### Request-Based Endpoints (Forwarded to Orchestrator)

These endpoints make requests that the orchestrator may or may not honor:

- **Task Submission**: `POST /api/v1/tasks` - Requests orchestrator to start a task
- **Pause Request**: `POST /api/v1/tasks/:id/pause` - Requests pause (orchestrator decides)
- **Resume Request**: `POST /api/v1/tasks/:id/resume` - Requests resume (orchestrator decides)
- **Cancel Request**: `POST /api/v1/tasks/:id/cancel` - Requests cancellation (orchestrator decides)
- **Project Scaffolding**: `POST /api/v1/projects` - Requests project creation (orchestrator handles)

## Implementation Details

### Task Execution Flow

```
1. API receives task submission request
   ↓
2. API creates task descriptor and forwards to orchestrator
   ↓
3. Orchestrator handles execution independently
   ↓
4. API observes state changes and exposes them via endpoints
   ↓
5. API never directly manipulates execution state
```

### Control Request Flow

```
1. API receives control request (e.g., pause)
   ↓
2. API logs request in chain-of-thought
   ↓
3. API forwards request to orchestrator
   ↓
4. Orchestrator evaluates request safety
   ↓
5. Orchestrator decides whether to honor request
   ↓
6. API observes the result (orchestrator may or may not pause)
```

## Code Documentation

All API code includes explicit documentation marking methods as observational:

```rust
/// Get task status (observational only)
///
/// **OBSERVATIONAL API**: This method only observes task state.
/// It never manipulates or changes the execution state.
pub async fn get_task_status(&self, task_id: Uuid) -> Result<Option<TaskExecutionState>> {
    // Implementation only reads state, never modifies it
}
```

```rust
/// Request pause of a task (orchestrator decides if it can pause)
///
/// **OBSERVATIONAL API**: This is a request, not a direct control.
/// The orchestrator maintains execution integrity and decides whether to honor the request.
/// The request is logged in chain-of-thought for auditability.
pub async fn request_pause_task(&self, task_id: Uuid) -> Result<()> {
    // Logs request, forwards to orchestrator, observes result
}
```

## Why This Design?

### Research Integrity

Direct manipulation would compromise research integrity by:
- Contaminating orchestrator decision-making with external influence
- Making results non-reproducible
- Breaking the chain of thought authenticity
- Creating artificial execution paths

### Autonomy

The orchestrator must maintain full autonomy to:
- Make decisions based on its own reasoning
- Handle execution lifecycle independently
- Maintain consistent decision-making patterns
- Preserve the integrity of its internal state

### Reproducibility

Observational design ensures:
- Results are reproducible (orchestrator decisions are autonomous)
- Chain of thought accurately reflects orchestrator reasoning
- No external contamination of execution state
- Research findings are valid and trustworthy

## Usage Guidelines

### ✅ DO

- Use API to observe orchestrator state
- Use API to request task submission
- Use API to request control operations (pause/resume/cancel)
- Monitor orchestrator activity through API endpoints
- Query chain of thought, council decisions, worker actions

### ❌ DON'T

- Attempt to directly manipulate execution state
- Expect control requests to always be honored (orchestrator decides)
- Use API to bypass orchestrator decision-making
- Modify orchestrator state directly through API
- Treat API as a control interface (it's observational)

## Related Documentation

- **API Server Source**: `iterations/v3/data-interfaces-adapters/src/bin/api-server.rs`
- **Orchestrator Service**: `iterations/v3/data-infrastructure/src/orchestrator_service.rs`
- **System Overview**: `iterations/v3/docs/system-overview.md`

## Summary

The API is your window into the orchestrator - use it to observe, monitor, and request, but never to directly control. This design preserves research integrity and ensures the orchestrator maintains full autonomy over its execution lifecycle.










