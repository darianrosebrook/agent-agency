# Type System Documentation

**Last Updated**: October 11, 2025  
**Purpose**: Comprehensive guide to V2 type system organization and usage

---

## Overview

The V2 type system is organized by component responsibility, with clear boundaries between orchestration, RL, CAWS, and performance tracking concerns.

---

## Core Type Files

### `agent-registry.ts`
**Purpose**: Agent profiles, capabilities, queries, and registry operations

**Key Types**:
- `AgentProfile`: Complete agent metadata and performance history
- `AgentCapabilities`: Task types, languages, specializations
- `AgentQuery`: Capability-based agent search
- `AgentQueryResult`: Search results with match scores (uses `agent` property, not `profile`)
- `PerformanceMetrics`: Agent performance tracking

**Usage Example**:
```typescript
import { AgentProfile, AgentQuery } from '../types/agent-registry';

// Query for agents
const query: AgentQuery = {
  languages: ['TypeScript'],
  taskType: 'code-editing', // Note: singular, not plural
};

const results = await registry.getAgentsByCapability(query);
// Access result: results[0].agent (not results[0].profile)
```

---

### `arbiter-orchestration.ts`
**Purpose**: Task definitions, routing decisions, orchestration state

**Key Types**:
- `Task`: Complete task specification with all required fields
- `TaskRequest`: Simplified task creation (omits generated fields)
- `TaskStatus`: Orchestration lifecycle states
- `RoutingDecision`: Task-to-agent assignment with rationale
- `TaskState`: Complete task execution state

**Important Notes**:
- `Task` interface has many required fields - use test helpers for creation
- `RoutingDecision` here includes `routingStrategy`, `alternativesConsidered`, `rationale`
- This is the **canonical** routing decision type for orchestration

**Usage Example**:
```typescript
import { Task, TaskType } from '../types/arbiter-orchestration';

// Creating a task requires many fields
const task: Task = {
  id: 'task-001',
  description: 'Refactor module',
  type: 'code-editing' as TaskType,
  requiredCapabilities: {
    languages: ['TypeScript'],
    taskTypes: ['code-editing'],
  },
  priority: 5,
  timeoutMs: 30000,
  budget: { maxFiles: 10, maxLoc: 500 },
  createdAt: new Date(),
  metadata: {},
  attempts: 0,
  maxAttempts: 3,
};
```

---

### `agentic-rl.ts`
**Purpose**: Reinforcement learning types, turn-level tracking, multi-armed bandit

**Key Types**:
- `RLTrainingSample`: State-action-reward tuples for training
- `TurnTrajectory`: Multi-turn conversation tracking
- `ToolAdoptionMetrics`: Tool usage patterns
- `ThinkingBudget`: Reasoning resource allocation
- `RoutingDecision` (RL-specific): Simplified routing for RL training

**Important Notes**:
- Contains an **RL-specific** `RoutingDecision` type
- Use `arbiter-orchestration.RoutingDecision` for orchestration
- Use `agentic-rl.RoutingDecision` for RL training data

**Usage Example**:
```typescript
import { RLTrainingSample } from '../types/agentic-rl';

const sample: RLTrainingSample = {
  state: stateVector,
  action: actionVector,
  reward: 0.85,
  nextState: nextStateVector,
  done: false,
};
```

---

### `caws-types.ts`
**Purpose**: CAWS working specifications, validation rules, quality gates

**Key Types**:
- `WorkingSpec`: Complete CAWS specification (many required fields)
- `AcceptanceCriterion`: Given-when-then test specifications
- `QualityGate`: Test, lint, coverage requirements
- `Waiver`: Exception management

**Important Notes**:
- `WorkingSpec` has many required fields - use test helpers
- Required: `blast_radius`, `operational_rollback_slo`, `invariants`, `non_functional`, `contracts`

**Usage Example**:
```typescript
import { WorkingSpec } from '../types/caws-types';

const spec: WorkingSpec = {
  id: 'FEAT-001',
  title: 'Feature implementation',
  mode: 'feature',
  risk_tier: 3,
  change_budget: { max_files: 10, max_loc: 500 },
  blast_radius: { modules: ['auth'], data_migration: false },
  operational_rollback_slo: '5m',
  scope: { in: ['src/auth/'], out: ['node_modules/'] },
  invariants: ['No breaking changes to public API'],
  acceptance: [
    { id: 'A1', given: 'User logged in', when: 'Clicks button', then: 'Action completes' },
  ],
  non_functional: {
    performance: { api_p95_ms: 200 },
  },
  contracts: [],
};
```

---

### `performance-tracking.ts`
**Purpose**: Performance metrics, benchmarking, RL data collection

**Key Types**:
- `PerformanceEvent`: Metric collection events
- `PerformanceMetrics`: Agent/task performance data
- `BenchmarkData`: Aggregated performance statistics
- `PerformanceAlert`: Anomaly detection alerts

---

## Type Conflicts & Resolutions

### Conflict 1: `RoutingDecision`

**Problem**: Two different `RoutingDecision` types exist

**Location 1**: `arbiter-orchestration.ts`
```typescript
export interface RoutingDecision {
  taskId: string;
  selectedAgent: AgentProfile;
  routingStrategy: string;
  confidence: number;
  alternativesConsidered: Array<{
    agentId: string;
    score: number;
    reason: string;
  }>;
  rationale: string;
  timestamp: Date;
}
```

**Location 2**: `agentic-rl.ts`
```typescript
export interface RoutingDecision {
  taskId: TaskId;
  selectedAgent: RLIdentifier;
  alternatives: RLIdentifier[];
  decisionRationale: string;
  timestamp: Timestamp;
}
```

**Resolution**:
- **Orchestration**: Use `arbiter-orchestration.RoutingDecision` (canonical for system)
- **RL Training**: Use `agentic-rl.RoutingDecision` (simplified for training data)
- Import with explicit path to avoid conflicts

**Usage**:
```typescript
// For orchestration
import { RoutingDecision } from '../types/arbiter-orchestration';

// For RL training
import { RoutingDecision as RLRoutingDecision } from '../types/agentic-rl';
```

---

### Conflict 2: Agent ID Types

**Problem**: `selectedAgent` returns different types in different contexts

**In TaskRoutingManager**:
```typescript
routing.selectedAgent: AgentProfile  // Returns full profile object
```

**Expected in PerformanceTracker**:
```typescript
startTaskExecution(taskId: string, agentId: string, ...)  // Expects string ID
```

**Resolution**:
Use `routing.selectedAgent.id` when string ID is needed:
```typescript
await tracker.startTaskExecution(
  task.id,
  routing.selectedAgent.id,  // Extract ID from profile
  routing
);
```

---

## Test Helpers

### Location
`/tests/helpers/test-fixtures.ts`

### Available Helpers

#### `createMinimalTask()`
Creates a valid `Task` with all required fields:

```typescript
import { createMinimalTask } from '../../helpers/test-fixtures';

const task = createMinimalTask({
  description: 'My custom task',
  type: 'code-review',
});
```

#### `createMinimalWorkingSpec()`
Creates a valid `WorkingSpec` with all required fields:

```typescript
import { createMinimalWorkingSpec } from '../../helpers/test-fixtures';

const spec = createMinimalWorkingSpec({
  title: 'My feature',
  risk_tier: 2,
});
```

#### `createTestAgent()`
Creates a test agent profile:

```typescript
import { createTestAgent } from '../../helpers/test-fixtures';

const agent = createTestAgent({
  id: 'my-test-agent',
  capabilities: {
    languages: ['Python'],
  },
});
```

---

## Common Patterns

### Pattern 1: Creating Tasks

**❌ Don't do this** (too verbose, error-prone):
```typescript
const task: Task = {
  id: 'task-001',
  description: 'Do something',
  type: 'code-editing',
  requiredCapabilities: { languages: ['TypeScript'], taskTypes: ['code-editing'] },
  priority: 5,
  timeoutMs: 30000,
  budget: { maxFiles: 10, maxLoc: 500 },
  createdAt: new Date(),
  metadata: {},
  attempts: 0,
  maxAttempts: 3,
};
```

**✅ Do this** (use helper):
```typescript
const task = createMinimalTask({
  description: 'Do something',
  type: 'code-editing',
});
```

---

### Pattern 2: Querying Agents

**❌ Don't do this** (wrong property names):
```typescript
const query: AgentQuery = {
  taskTypes: ['code-editing'],  // Wrong! Should be taskType (singular)
};
```

**✅ Do this**:
```typescript
const query: AgentQuery = {
  taskType: 'code-editing',  // Correct: singular
  languages: ['TypeScript'],
};
```

---

### Pattern 3: Accessing Query Results

**❌ Don't do this** (wrong property):
```typescript
const results = await registry.getAgentsByCapability(query);
const agentId = results[0].profile.id;  // Wrong! No 'profile' property
```

**✅ Do this**:
```typescript
const results = await registry.getAgentsByCapability(query);
const agentId = results[0].agent.id;  // Correct: 'agent' property
```

---

### Pattern 4: Using Routing Decisions

**❌ Don't do this** (wrong type assumption):
```typescript
await tracker.startTaskExecution(
  task.id,
  routing.selectedAgent,  // Wrong! This is AgentProfile, not string
  routing
);
```

**✅ Do this**:
```typescript
await tracker.startTaskExecution(
  task.id,
  routing.selectedAgent.id,  // Correct: Extract ID
  routing
);
```

---

## Type Import Guidelines

### Rule 1: Import from Specific Files
**✅ Preferred**:
```typescript
import { Task } from '../types/arbiter-orchestration';
import { AgentProfile } from '../types/agent-registry';
```

**❌ Avoid**:
```typescript
import { Task, AgentProfile } from '../types';  // Ambiguous
```

### Rule 2: Handle Conflicts with Aliases
```typescript
import { RoutingDecision as OrchestrationRouting } from '../types/arbiter-orchestration';
import { RoutingDecision as RLRouting } from '../types/agentic-rl';
```

### Rule 3: Use Re-exports Carefully
Some types are re-exported for convenience, but prefer direct imports for clarity.

---

## Migration Guide

### If You're Seeing Type Errors

1. **Check property names**: `agent` vs `profile`, `taskType` vs `taskTypes`
2. **Check ID extraction**: `routing.selectedAgent.id` not `routing.selectedAgent`
3. **Use test helpers**: Don't manually create complex types
4. **Import from correct file**: `arbiter-orchestration` vs `agentic-rl`

### Common Fixes

```typescript
// Old (broken)
const results = await registry.getAgentsByCapability(query);
const agent = results[0].profile;  // ❌ No 'profile' property

// New (working)
const results = await registry.getAgentsByCapability(query);
const agent = results[0].agent;  // ✅ Correct property
```

```typescript
// Old (broken)
await tracker.startTaskExecution(task.id, routing.selectedAgent, routing);
// ❌ selectedAgent is AgentProfile, not string

// New (working)
await tracker.startTaskExecution(task.id, routing.selectedAgent.id, routing);
// ✅ Extract ID from profile
```

---

## Quick Reference

| Type | File | Key Properties | Notes |
|------|------|---------------|-------|
| `Task` | `arbiter-orchestration.ts` | All fields required | Use `createMinimalTask()` |
| `AgentProfile` | `agent-registry.ts` | `id`, `capabilities`, `performanceHistory` | - |
| `AgentQueryResult` | `agent-registry.ts` | `agent`, `matchScore` | Not `profile`! |
| `AgentQuery` | `agent-registry.ts` | `taskType` (singular) | Not `taskTypes`! |
| `RoutingDecision` | `arbiter-orchestration.ts` | `selectedAgent: AgentProfile` | Canonical type |
| `WorkingSpec` | `caws-types.ts` | Many required fields | Use `createMinimalWorkingSpec()` |

---

## Need Help?

1. Check this README first
2. Look at test fixtures in `tests/helpers/test-fixtures.ts`
3. Check existing tests for usage examples
4. Grep for usage: `grep -r "import.*AgentQuery" tests/`

---

**Maintained by**: @darianrosebrook  
**Last Updated**: October 11, 2025

