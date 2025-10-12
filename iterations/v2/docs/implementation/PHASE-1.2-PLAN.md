# Phase 1.2: Core Task Orchestrator Implementation Plan

**Date**: October 12, 2025
**Status**: 🔄 In Progress
**Expected Duration**: 3-4 hours

---

## Overview

Implement the core task orchestrator that integrates all ARBITER components (001-004) into a cohesive system. The orchestrator manages end-to-end task lifecycle using the state machine we just built.

---

## Goals

1. **Task Orchestration**: End-to-end task execution from request to completion
2. **Component Integration**: Seamless coordination between all ARBITER modules
3. **Error Handling**: Robust failure recovery and retry logic
4. **Observability**: Full tracing and monitoring of task execution
5. **Performance**: High-throughput task processing

---

## Orchestrator Architecture

```
┌─────────────────────────────────────────────────┐
│              Task Orchestrator                  │
│                                                 │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐        │
│  │  State  │  │  Task   │  │  Retry  │        │
│  │ Machine │  │  Queue  │  │  Logic  │        │
│  └─────────┘  └─────────┘  └─────────┘        │
│                                                 │
│  ┌─────────────────────────────────────────┐   │
│  │          Component Integration          │   │
│  │                                         │   │
│  │  ARBITER-001: Agent Registry            │   │
│  │  ARBITER-002: Task Routing              │   │
│  │  ARBITER-003: CAWS Validation           │   │
│  │  ARBITER-004: Performance Tracking      │   │
│  └─────────────────────────────────────────┘   │
│                                                 │
│  ┌─────────────────────────────────────────┐   │
│  │          Production Infrastructure       │   │
│  │                                         │   │
│  │  Tracing, Health, Config, Resilience    │   │
│  └─────────────────────────────────────────┘   │
└─────────────────────────────────────────────────┘
```

---

## Task Execution Flow

### 1. Task Submission

```typescript
// External API
orchestrator.submitTask({
  id: "task-123",
  type: "code-review",
  payload: { code: "...", requirements: "..." },
  metadata: { priority: "high", timeout: 300000 }
});
```

### 2. Validation Phase

```typescript
// 1. Initialize task state
stateMachine.initializeTask(taskId);

// 2. Validate CAWS spec
const validation = await validator.validateWorkingSpec(task);
if (!validation.valid) {
  stateMachine.transition(taskId, TaskState.CANCELLED, "validation failed");
  throw new ValidationError(validation.errors);
}

// 3. Move to queued state
stateMachine.transition(taskId, TaskState.QUEUED, "validation passed");
```

### 3. Routing Phase

```typescript
// 4. Route to agent
const routing = await router.routeTask(task);
stateMachine.transition(taskId, TaskState.ASSIGNED, `assigned to ${routing.selectedAgent.id}`, {
  agentId: routing.selectedAgent.id,
  confidence: routing.confidence
});

// 5. Record routing decision
tracker.recordRoutingDecision(routing);
```

### 4. Execution Phase

```typescript
// 6. Start execution
stateMachine.transition(taskId, TaskState.RUNNING, "execution started");
tracker.startTaskExecution(taskId, routing.selectedAgent.id, routing);

// 7. Execute task (this would be handled by the agent in real system)
// await agent.executeTask(taskId, task);

// 8. Complete execution
tracker.completeTaskExecution(taskId, {
  success: true,
  qualityScore: 0.95,
  latencyMs: 2500
});
stateMachine.transition(taskId, TaskState.COMPLETED, "execution completed");
```

---

## Implementation Components

### 1. Core Orchestrator Class

```typescript
// src/orchestrator/TaskOrchestrator.ts

export class TaskOrchestrator {
  constructor(
    private stateMachine: TaskStateMachine,
    private agentRegistry: AgentRegistryManager,
    private taskRouter: TaskRoutingManager,
    private cawsValidator: SpecValidator,
    private performanceTracker: PerformanceTracker,
    private tracing: TracingProvider,
    private health: HealthMonitor
  ) {
    this.setupEventHandlers();
  }

  /**
   * Submit a task for execution
   */
  async submitTask(task: Task): Promise<TaskExecutionResult> {
    return this.tracing.traceOperation("orchestrator:submitTask", async () => {
      // 1. Initialize task state
      this.stateMachine.initializeTask(task.id);

      try {
        // 2. Validate task
        await this.validateTask(task);

        // 3. Route task to agent
        const routing = await this.routeTask(task);

        // 4. Execute task
        const result = await this.executeTask(task, routing);

        return result;
      } catch (error) {
        // Handle errors and update state
        await this.handleTaskError(task.id, error);
        throw error;
      }
    });
  }

  private async validateTask(task: Task): Promise<void> {
    // CAWS validation logic
  }

  private async routeTask(task: Task): Promise<RoutingDecision> {
    // Routing logic
  }

  private async executeTask(task: Task, routing: RoutingDecision): Promise<TaskExecutionResult> {
    // Execution logic
  }

  private async handleTaskError(taskId: string, error: any): Promise<void> {
    // Error handling logic
  }
}
```

---

### 2. Task Queue Management

```typescript
// src/orchestrator/TaskQueue.ts

export class TaskQueue {
  private queue: Task[] = [];
  private processing: Set<string> = new Set();

  enqueue(task: Task): void {
    this.queue.push(task);
    this.emit("task:enqueued", task);
  }

  dequeue(): Task | undefined {
    const task = this.queue.shift();
    if (task) {
      this.processing.add(task.id);
      this.emit("task:dequeued", task);
    }
    return task;
  }

  remove(taskId: string): void {
    this.queue = this.queue.filter(t => t.id !== taskId);
    this.processing.delete(taskId);
  }

  isProcessing(taskId: string): boolean {
    return this.processing.has(taskId);
  }

  getStats(): TaskQueueStats {
    return {
      queued: this.queue.length,
      processing: this.processing.size,
      total: this.queue.length + this.processing.size
    };
  }
}
```

---

### 3. Retry and Error Handling

```typescript
// src/orchestrator/TaskRetryHandler.ts

export class TaskRetryHandler {
  constructor(
    private maxRetries: number = 3,
    private backoffMs: number = 1000
  ) {}

  async executeWithRetry<T>(
    operation: () => Promise<T>,
    taskId: string
  ): Promise<T> {
    let attempts = 0;
    let lastError: any;

    while (attempts < this.maxRetries) {
      try {
        return await operation();
      } catch (error) {
        lastError = error;
        attempts++;

        if (attempts < this.maxRetries) {
          const delay = this.calculateBackoff(attempts);
          await this.delay(delay);
          this.emit("task:retry", { taskId, attempt: attempts, delay, error });
        }
      }
    }

    throw new TaskExecutionError(`Task ${taskId} failed after ${this.maxRetries} attempts`, lastError);
  }

  private calculateBackoff(attempt: number): number {
    return this.backoffMs * Math.pow(2, attempt - 1);
  }

  private delay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}
```

---

### 4. Orchestrator Events

```typescript
// src/types/orchestrator-events.ts

export interface TaskSubmittedEvent {
  taskId: string;
  task: Task;
  timestamp: Date;
}

export interface TaskValidatedEvent {
  taskId: string;
  validationResult: ValidationResult;
  timestamp: Date;
}

export interface TaskRoutedEvent {
  taskId: string;
  routing: RoutingDecision;
  timestamp: Date;
}

export interface TaskStartedEvent {
  taskId: string;
  agentId: string;
  timestamp: Date;
}

export interface TaskCompletedEvent {
  taskId: string;
  result: TaskExecutionResult;
  timestamp: Date;
}

export interface TaskFailedEvent {
  taskId: string;
  error: any;
  attempt: number;
  timestamp: Date;
}

export interface TaskRetryEvent {
  taskId: string;
  attempt: number;
  delay: number;
  error: any;
  timestamp: Date;
}
```

---

## Integration Points

### With ARBITER-001 (Agent Registry)

```typescript
// When routing needs agent data
const candidates = await agentRegistry.getAgentsByCapability({
  taskType: task.type,
  languages: task.requirements.languages
});

// Update agent performance after task completion
await agentRegistry.updatePerformance(agentId, taskResult);
```

### With ARBITER-002 (Task Routing)

```typescript
// Route task to best agent
const routing = await taskRouter.routeTask(task, {
  agentMetrics: await this.getAgentMetrics(),
  systemLoad: await this.getSystemLoad()
});
```

### With ARBITER-003 (CAWS Validation)

```typescript
// Validate task specification
const validation = await cawsValidator.validateWorkingSpec(task);
if (!validation.valid) {
  throw new ValidationError(validation.errors);
}
```

### With ARBITER-004 (Performance Tracking)

```typescript
// Track task lifecycle events
performanceTracker.recordRoutingDecision(routing);
performanceTracker.startTaskExecution(task.id, agentId, routing);
performanceTracker.completeTaskExecution(task.id, result);
```

---

## Testing Strategy

### Unit Tests

```typescript
// tests/unit/orchestrator/task-orchestrator.test.ts

describe("TaskOrchestrator", () => {
  let orchestrator: TaskOrchestrator;
  let stateMachine: TaskStateMachine;
  let agentRegistry: AgentRegistryManager;
  let taskRouter: TaskRoutingManager;
  let cawsValidator: SpecValidator;
  let performanceTracker: PerformanceTracker;

  beforeEach(() => {
    // Setup mocks and orchestrator
  });

  describe("task submission", () => {
    it("should process valid task end-to-end", async () => {
      // Setup valid task
      const task = createMinimalTask();

      // Mock all dependencies to succeed
      mockValidator.mockResolvedValue({ valid: true });
      mockRouter.mockResolvedValue(createMockRouting());
      mockExecution.mockResolvedValue(createMockResult());

      // Submit task
      const result = await orchestrator.submitTask(task);

      // Verify state transitions
      expect(stateMachine.getState(task.id)).toBe(TaskState.COMPLETED);
      expect(result.success).toBe(true);
    });

    it("should reject invalid task", async () => {
      const invalidTask = createMinimalTask();

      // Mock validation failure
      mockValidator.mockResolvedValue({
        valid: false,
        errors: [{ field: "type", message: "Invalid task type" }]
      });

      // Should throw validation error
      await expect(orchestrator.submitTask(invalidTask))
        .rejects.toThrow(ValidationError);

      // Task should be cancelled
      expect(stateMachine.getState(invalidTask.id)).toBe(TaskState.CANCELLED);
    });

    it("should handle routing failure", async () => {
      const task = createMinimalTask();

      mockValidator.mockResolvedValue({ valid: true });
      mockRouter.mockRejectedValue(new Error("No agents available"));

      await expect(orchestrator.submitTask(task))
        .rejects.toThrow("No agents available");

      expect(stateMachine.getState(task.id)).toBe(TaskState.FAILED);
    });
  });

  describe("retry logic", () => {
    it("should retry failed tasks up to max retries", async () => {
      // Test retry behavior
    });

    it("should apply exponential backoff", async () => {
      // Test backoff timing
    });
  });
});
```

---

## Acceptance Criteria

1. ✅ Task orchestrator accepts and processes task requests
2. ✅ Tasks go through complete lifecycle (PENDING → COMPLETED)
3. ✅ Invalid tasks are rejected with proper error messages
4. ✅ Failed tasks are retried with exponential backoff
5. ✅ All ARBITER components are properly integrated
6. ✅ Performance tracking captures all lifecycle events
7. ✅ Error conditions are handled gracefully
8. ✅ End-to-end tests demonstrate full workflow
9. ✅ High throughput (1000+ tasks/minute)
10. ✅ All tests passing (unit + integration)

---

## Implementation Checklist

- [ ] Create orchestrator types and interfaces
- [ ] Implement core TaskOrchestrator class
- [ ] Add TaskQueue for processing management
- [ ] Implement TaskRetryHandler with backoff
- [ ] Integrate with state machine
- [ ] Connect ARBITER-001 (Agent Registry)
- [ ] Connect ARBITER-002 (Task Routing)
- [ ] Connect ARBITER-003 (CAWS Validation)
- [ ] Connect ARBITER-004 (Performance Tracking)
- [ ] Add production infrastructure integration
- [ ] Write comprehensive unit tests
- [ ] Write integration tests
- [ ] Add performance benchmarks
- [ ] Update documentation

---

**Status**: Ready to implement!
