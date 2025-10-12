# ARBITER-005: Arbiter Orchestrator Implementation Plan

**Date**: October 11, 2025  
**Author**: AI Agent  
**Component**: ARBITER-005 (Arbiter Orchestrator)  
**Working Spec**: `components/arbiter-orchestrator/.caws/working-spec.yaml`  
**Risk Tier**: 1 (Critical - Constitutional Authority Runtime)

---

## Executive Summary

ARBITER-005 is the **central nervous system** of Agent Agency V2 - the orchestrator that integrates all previous components (001-004) into a cohesive, constitutionally-compliant system.

**Core Responsibilities**:

1. End-to-end task orchestration from request to completion
2. Constitutional authority runtime and decision enforcement
3. Integration of all ARBITER components
4. Real-time system health monitoring and recovery
5. Constitutional feedback loop management

**Critical Success Factors**:

- 95% of tasks complete end-to-end within 500ms
- 99.99% constitutional compliance rate
- 2000 concurrent task support
- 99.9% uptime with automatic recovery

---

## Phase 0: Foundation Hardening (1-2 Weeks)

**Purpose**: Ensure ARBITER-001 through 004 are production-ready before building orchestration

### Task 0.1: Complete Integration Testing (3-4 days)

**Goal**: Comprehensive E2E tests for all four components

**Test Scenarios to Implement**:

```typescript
// File: tests/integration/foundation/001-004-integration.test.ts

describe("ARBITER-001 to 004 Integration", () => {
  // Scenario 1: Happy path workflow
  it("should complete agent-routing-tracking workflow", async () => {
    // 1. Register agent (ARBITER-001)
    const agent = await registry.registerAgent(agentData);

    // 2. Route task to agent (ARBITER-002)
    const routing = await router.routeTask(task);
    expect(routing.selectedAgent).toBe(agent.id);

    // 3. Track performance (ARBITER-004)
    await tracker.recordRoutingDecision(routing);
    await tracker.startTaskExecution(task.id, agent.id, routing);

    // 4. Validate result (ARBITER-003)
    const validation = await validator.validateWorkingSpec(result.spec);
    expect(validation.valid).toBe(true);

    // 5. Record completion (ARBITER-004)
    await tracker.completeTaskExecution(task.id, outcome);

    // 6. Verify feedback loop
    const updatedAgent = await registry.getAgent(agent.id);
    expect(updatedAgent.performanceHistory.taskCount).toBe(1);
  });

  // Scenario 2: Constitutional violation
  it("should reject non-compliant task", async () => {
    const invalidSpec = createNonCompliantSpec();
    const validation = await validator.validateWorkingSpec(invalidSpec);

    expect(validation.valid).toBe(false);
    expect(validation.violations).toHaveLength(2);
    // Task should not be routed
  });

  // Scenario 3: Agent failure and recovery
  it("should handle agent failure gracefully", async () => {
    const routing = await router.routeTask(task);

    // Simulate agent failure
    await tracker.recordTaskFailure(task.id, routing.selectedAgent, error);

    // Should penalize failed agent
    const agent = await registry.getAgent(routing.selectedAgent);
    expect(agent.performanceHistory.successRate).toBeLessThan(1.0);

    // Should route retry to different agent
    const retryRouting = await router.routeTask(task);
    expect(retryRouting.selectedAgent).not.toBe(routing.selectedAgent);
  });

  // Scenario 4: Performance degradation detection
  it("should detect and alert on performance degradation", async () => {
    // Execute 100 tasks with degrading performance
    for (let i = 0; i < 100; i++) {
      await tracker.recordRoutingDecision(routing);
      await tracker.completeTaskExecution(task.id, {
        success: true,
        latencyMs: 50 + i * 2, // Increasing latency
      });
    }

    // Should detect trend and alert
    const alerts = await tracker.getActiveAlerts();
    expect(alerts).toContainEqual(
      expect.objectContaining({
        type: "PERFORMANCE_DEGRADATION",
        metric: "latencyMs",
      })
    );
  });

  // Scenario 5: Load testing
  it("should handle 1000 concurrent tasks", async () => {
    const tasks = Array.from({ length: 1000 }, (_, i) =>
      createTask({ id: `task-${i}` })
    );

    const start = Date.now();
    const results = await Promise.all(
      tasks.map(async (task) => {
        const routing = await router.routeTask(task);
        await tracker.recordRoutingDecision(routing);
        return routing;
      })
    );
    const duration = Date.now() - start;

    // All tasks routed successfully
    expect(results).toHaveLength(1000);
    expect(results.every((r) => r.selectedAgent)).toBe(true);

    // Within performance budget (< 10 seconds for 1000 tasks)
    expect(duration).toBeLessThan(10000);
  });
});
```

**Deliverables**:

- ✅ 20+ integration tests covering happy paths and failure scenarios
- ✅ Load tests validating concurrent operation
- ✅ Performance benchmarks for each component
- ✅ Failure injection tests

---

### Task 0.2: Performance Benchmarking (2 days)

**Goal**: Measure actual performance of ARBITER-001, 002, 003

**Benchmark Suite**:

```typescript
// File: tests/performance/001-003-benchmarks.test.ts

describe("Component Performance Benchmarks", () => {
  it("ARBITER-001: Agent queries < 50ms P95", async () => {
    const latencies = await measureLatency(1000, async () => {
      await registry.getAgentsByCapability(query);
    });

    const p95 = percentile(latencies, 0.95);
    expect(p95).toBeLessThan(50);
  });

  it("ARBITER-002: Routing decisions < 100ms P95", async () => {
    const latencies = await measureLatency(1000, async () => {
      await router.routeTask(task);
    });

    const p95 = percentile(latencies, 0.95);
    expect(p95).toBeLessThan(100);
  });

  it("ARBITER-003: Spec validation < 200ms P95", async () => {
    const latencies = await measureLatency(1000, async () => {
      await validator.validateWorkingSpec(spec);
    });

    const p95 = percentile(latencies, 0.95);
    expect(p95).toBeLessThan(200);
  });

  it("End-to-end workflow < 500ms P95", async () => {
    const latencies = await measureLatency(100, async () => {
      // Full workflow: register → route → validate → track
      const agent = await registry.registerAgent(agentData);
      const routing = await router.routeTask(task);
      await tracker.recordRoutingDecision(routing);
      const validation = await validator.validateWorkingSpec(spec);
      await tracker.completeTaskExecution(task.id, outcome);
    });

    const p95 = percentile(latencies, 0.95);
    expect(p95).toBeLessThan(500);
  });
});
```

**Deliverables**:

- ✅ Baseline performance metrics for all components
- ✅ Documentation of actual vs. claimed performance
- ✅ Identification of performance bottlenecks
- ✅ Optimization recommendations

---

### Task 0.3: Production Infrastructure (3 days)

**Goal**: Add observability, configuration, and error handling

**Observability Stack**:

```typescript
// File: src/observability/DistributedTracing.ts

export class DistributedTracing {
  // Trace context propagation
  createSpan(name: string, parent?: SpanContext): Span {
    return {
      traceId: parent?.traceId || generateTraceId(),
      spanId: generateSpanId(),
      parentSpanId: parent?.spanId,
      name,
      startTime: Date.now(),
      attributes: {},
    };
  }

  // Automatic span correlation
  async traceWorkflow<T>(
    name: string,
    fn: (span: Span) => Promise<T>
  ): Promise<T> {
    const span = this.createSpan(name);
    try {
      const result = await fn(span);
      span.endTime = Date.now();
      span.status = "SUCCESS";
      await this.recordSpan(span);
      return result;
    } catch (error) {
      span.endTime = Date.now();
      span.status = "ERROR";
      span.error = error;
      await this.recordSpan(span);
      throw error;
    }
  }
}

// File: src/config/SystemConfig.ts

export class SystemConfig {
  // Externalized configuration with validation
  static load(): SystemConfiguration {
    return {
      agentRegistry: {
        maxAgents: env.get("ARBITER_MAX_AGENTS", 1000),
        cleanupIntervalMs: env.get("ARBITER_CLEANUP_INTERVAL", 300000),
        performanceHistorySize: env.get("ARBITER_PERF_HISTORY", 100),
      },
      taskRouting: {
        explorationRate: env.get("ARBITER_EXPLORATION_RATE", 0.2),
        decayFactor: env.get("ARBITER_DECAY_FACTOR", 0.995),
        minSampleSize: env.get("ARBITER_MIN_SAMPLES", 10),
      },
      performanceTracking: {
        bufferSize: env.get("ARBITER_BUFFER_SIZE", 1000),
        flushIntervalMs: env.get("ARBITER_FLUSH_INTERVAL", 5000),
        anonymization: env.get("ARBITER_ANONYMIZE", false),
      },
      cawsValidator: {
        strictMode: env.get("ARBITER_CAWS_STRICT", true),
        waiverApprovalRequired: env.get("ARBITER_WAIVER_APPROVAL", true),
      },
    };
  }
}

// File: src/resilience/CircuitBreaker.ts

export class CircuitBreaker {
  // Circuit breaker for component interactions
  async execute<T>(
    name: string,
    fn: () => Promise<T>,
    options?: CircuitBreakerOptions
  ): Promise<T> {
    const state = this.getState(name);

    if (state === "OPEN") {
      // Circuit open - fail fast
      throw new CircuitBreakerOpenError(name);
    }

    try {
      const result = await Promise.race([
        fn(),
        timeout(options?.timeoutMs || 5000),
      ]);

      this.recordSuccess(name);
      return result;
    } catch (error) {
      this.recordFailure(name);

      if (this.shouldOpenCircuit(name)) {
        this.openCircuit(name);
      }

      throw error;
    }
  }
}
```

**Deliverables**:

- ✅ Distributed tracing across all components
- ✅ Centralized configuration management
- ✅ Circuit breakers for all external calls
- ✅ Structured logging with correlation IDs
- ✅ Health check endpoints

---

## Phase 1: Core Orchestration (2-3 Weeks)

**Purpose**: Implement main orchestration state machine and task lifecycle management

### Task 1.1: Task State Machine Design (2 days)

**Goal**: Define clear state transitions for task lifecycle

**State Machine Specification**:

```typescript
// File: src/types/orchestration-state.ts

export enum TaskState {
  // Initial states
  RECEIVED = "received", // Task just arrived
  QUEUED = "queued", // Waiting for resources

  // Validation states
  VALIDATING_SPEC = "validating_spec",
  SPEC_REJECTED = "spec_rejected",
  SPEC_APPROVED = "spec_approved",

  // Routing states
  ROUTING = "routing",
  ROUTE_FAILED = "route_failed",
  ROUTED = "routed",

  // Execution states
  EXECUTING = "executing",
  EXECUTION_FAILED = "execution_failed",
  EXECUTION_COMPLETE = "execution_complete",

  // Verification states
  VERIFYING = "verifying",
  VERIFICATION_FAILED = "verification_failed",
  VERIFIED = "verified",

  // Terminal states
  COMPLETED = "completed",
  FAILED = "failed",
  CANCELLED = "cancelled",
}

export type TaskTransition =
  | { from: TaskState.RECEIVED; to: TaskState.VALIDATING_SPEC }
  | {
      from: TaskState.VALIDATING_SPEC;
      to: TaskState.SPEC_APPROVED | TaskState.SPEC_REJECTED;
    }
  | { from: TaskState.SPEC_APPROVED; to: TaskState.ROUTING }
  | { from: TaskState.ROUTING; to: TaskState.ROUTED | TaskState.ROUTE_FAILED }
  | { from: TaskState.ROUTED; to: TaskState.EXECUTING }
  | {
      from: TaskState.EXECUTING;
      to: TaskState.EXECUTION_COMPLETE | TaskState.EXECUTION_FAILED;
    }
  | { from: TaskState.EXECUTION_COMPLETE; to: TaskState.VERIFYING }
  | {
      from: TaskState.VERIFYING;
      to: TaskState.VERIFIED | TaskState.VERIFICATION_FAILED;
    }
  | { from: TaskState.VERIFIED; to: TaskState.COMPLETED }
  | {
      from:
        | TaskState.SPEC_REJECTED
        | TaskState.ROUTE_FAILED
        | TaskState.EXECUTION_FAILED
        | TaskState.VERIFICATION_FAILED;
      to: TaskState.FAILED;
    };

export interface TaskStateMachine {
  currentState: TaskState;
  history: TaskStateTransition[];
  metadata: TaskStateMetadata;

  canTransition(to: TaskState): boolean;
  transition(to: TaskState, reason: string): Promise<void>;
  rollback(toState: TaskState): Promise<void>;
}
```

**State Transition Rules**:

```typescript
// File: src/orchestrator/TaskStateMachine.ts

export class TaskStateMachine {
  private validTransitions: Map<TaskState, TaskState[]> = new Map([
    [TaskState.RECEIVED, [TaskState.VALIDATING_SPEC, TaskState.CANCELLED]],
    [
      TaskState.VALIDATING_SPEC,
      [TaskState.SPEC_APPROVED, TaskState.SPEC_REJECTED],
    ],
    [TaskState.SPEC_APPROVED, [TaskState.ROUTING]],
    [TaskState.ROUTING, [TaskState.ROUTED, TaskState.ROUTE_FAILED]],
    [TaskState.ROUTED, [TaskState.EXECUTING, TaskState.CANCELLED]],
    [
      TaskState.EXECUTING,
      [TaskState.EXECUTION_COMPLETE, TaskState.EXECUTION_FAILED],
    ],
    [TaskState.EXECUTION_COMPLETE, [TaskState.VERIFYING]],
    [TaskState.VERIFYING, [TaskState.VERIFIED, TaskState.VERIFICATION_FAILED]],
    [TaskState.VERIFIED, [TaskState.COMPLETED]],
    // Failure states can transition to FAILED
    [TaskState.SPEC_REJECTED, [TaskState.FAILED]],
    [TaskState.ROUTE_FAILED, [TaskState.FAILED]],
    [TaskState.EXECUTION_FAILED, [TaskState.FAILED]],
    [TaskState.VERIFICATION_FAILED, [TaskState.FAILED]],
  ]);

  canTransition(from: TaskState, to: TaskState): boolean {
    const allowedTransitions = this.validTransitions.get(from);
    return allowedTransitions?.includes(to) ?? false;
  }

  async transition(
    taskId: string,
    to: TaskState,
    reason: string
  ): Promise<void> {
    const task = await this.getTask(taskId);

    if (!this.canTransition(task.state, to)) {
      throw new InvalidStateTransitionError(
        `Cannot transition from ${task.state} to ${to}`
      );
    }

    // Record transition
    const transition: TaskStateTransition = {
      from: task.state,
      to,
      reason,
      timestamp: new Date(),
      metadata: this.captureMetadata(),
    };

    // Update state
    task.state = to;
    task.history.push(transition);

    // Persist
    await this.saveTask(task);

    // Emit event
    this.emit("state_transition", { taskId, transition });

    // Trace
    await this.tracing.recordStateTransition(taskId, transition);
  }
}
```

**Deliverables**:

- ✅ Complete state machine specification
- ✅ State transition validation logic
- ✅ State persistence and recovery
- ✅ State transition events and tracing

---

### Task 1.2: Task Orchestrator Implementation (5 days)

**Goal**: Implement end-to-end task orchestration

**Core Orchestrator**:

```typescript
// File: src/orchestrator/TaskOrchestrator.ts

export class TaskOrchestrator {
  constructor(
    private agentRegistry: AgentRegistryManager,
    private taskRouter: TaskRoutingManager,
    private cawsValidator: SpecValidator,
    private performanceTracker: PerformanceTracker,
    private stateMachine: TaskStateMachine,
    private circuitBreaker: CircuitBreaker,
    private tracing: DistributedTracing
  ) {}

  async orchestrateTask(request: TaskRequest): Promise<TaskResult> {
    return await this.tracing.traceWorkflow(
      "orchestrate_task",
      async (span) => {
        const taskId = generateTaskId();
        span.setAttribute("task_id", taskId);

        try {
          // 1. Initialize task
          await this.stateMachine.transition(
            taskId,
            TaskState.RECEIVED,
            "Task received"
          );

          // 2. Validate CAWS spec
          await this.validateSpec(taskId, request.spec, span);

          // 3. Route to agent
          const routing = await this.routeTask(taskId, request, span);

          // 4. Execute task
          const execution = await this.executeTask(
            taskId,
            routing,
            request,
            span
          );

          // 5. Verify result
          await this.verifyResult(taskId, execution, span);

          // 6. Complete
          await this.stateMachine.transition(
            taskId,
            TaskState.COMPLETED,
            "Task completed successfully"
          );

          return {
            taskId,
            success: true,
            result: execution.result,
            performance: execution.metrics,
          };
        } catch (error) {
          await this.handleTaskFailure(taskId, error, span);
          throw error;
        }
      }
    );
  }

  private async validateSpec(
    taskId: string,
    spec: WorkingSpec,
    parentSpan: Span
  ): Promise<void> {
    return await this.tracing.traceWorkflow("validate_spec", async (span) => {
      span.parent = parentSpan;

      await this.stateMachine.transition(
        taskId,
        TaskState.VALIDATING_SPEC,
        "Starting spec validation"
      );

      const validation = await this.circuitBreaker.execute(
        "caws_validator",
        () => this.cawsValidator.validateWorkingSpec(spec),
        { timeoutMs: 5000 }
      );

      if (!validation.valid) {
        await this.stateMachine.transition(
          taskId,
          TaskState.SPEC_REJECTED,
          `Spec validation failed: ${validation.violations
            .map((v) => v.message)
            .join(", ")}`
        );

        throw new SpecValidationError(validation.violations);
      }

      await this.stateMachine.transition(
        taskId,
        TaskState.SPEC_APPROVED,
        "Spec validation passed"
      );

      // Record compliance metrics
      await this.performanceTracker.recordConstitutionalValidation(
        spec.id,
        true,
        0, // No violations
        1.0 // Perfect compliance
      );
    });
  }

  private async routeTask(
    taskId: string,
    request: TaskRequest,
    parentSpan: Span
  ): Promise<RoutingDecision> {
    return await this.tracing.traceWorkflow("route_task", async (span) => {
      span.parent = parentSpan;

      await this.stateMachine.transition(
        taskId,
        TaskState.ROUTING,
        "Starting task routing"
      );

      const routing = await this.circuitBreaker.execute(
        "task_router",
        () =>
          this.taskRouter.routeTask({
            id: taskId,
            ...request,
          }),
        { timeoutMs: 2000 }
      );

      if (!routing.selectedAgent) {
        await this.stateMachine.transition(
          taskId,
          TaskState.ROUTE_FAILED,
          "No agent available for task"
        );

        throw new RoutingFailedError("No suitable agent found");
      }

      await this.stateMachine.transition(
        taskId,
        TaskState.ROUTED,
        `Routed to agent ${routing.selectedAgent}`
      );

      // Record routing decision
      await this.performanceTracker.recordRoutingDecision(routing);

      return routing;
    });
  }

  private async executeTask(
    taskId: string,
    routing: RoutingDecision,
    request: TaskRequest,
    parentSpan: Span
  ): Promise<TaskExecution> {
    return await this.tracing.traceWorkflow("execute_task", async (span) => {
      span.parent = parentSpan;

      await this.stateMachine.transition(
        taskId,
        TaskState.EXECUTING,
        `Executing with agent ${routing.selectedAgent}`
      );

      // Start performance tracking
      await this.performanceTracker.startTaskExecution(
        taskId,
        routing.selectedAgent,
        routing
      );

      const startTime = Date.now();

      try {
        // Get agent from registry
        const agent = await this.agentRegistry.getAgent(routing.selectedAgent);

        // Execute task (actual agent invocation would go here)
        const result = await this.invokeAgent(agent, request, span);

        const duration = Date.now() - startTime;

        await this.stateMachine.transition(
          taskId,
          TaskState.EXECUTION_COMPLETE,
          `Execution completed in ${duration}ms`
        );

        // Record completion
        await this.performanceTracker.completeTaskExecution(taskId, {
          success: true,
          latencyMs: duration,
          qualityScore: result.quality || 0.8,
        });

        return {
          result,
          metrics: { duration, success: true },
        };
      } catch (error) {
        const duration = Date.now() - startTime;

        await this.stateMachine.transition(
          taskId,
          TaskState.EXECUTION_FAILED,
          `Execution failed: ${error.message}`
        );

        // Record failure
        await this.performanceTracker.completeTaskExecution(taskId, {
          success: false,
          latencyMs: duration,
          error: error.message,
        });

        throw error;
      }
    });
  }

  private async verifyResult(
    taskId: string,
    execution: TaskExecution,
    parentSpan: Span
  ): Promise<void> {
    return await this.tracing.traceWorkflow("verify_result", async (span) => {
      span.parent = parentSpan;

      await this.stateMachine.transition(
        taskId,
        TaskState.VERIFYING,
        "Starting result verification"
      );

      // Verify against CAWS criteria
      const verification = await this.circuitBreaker.execute(
        "caws_validator",
        () => this.cawsValidator.validateTaskResult(execution.result),
        { timeoutMs: 5000 }
      );

      if (!verification.passed) {
        await this.stateMachine.transition(
          taskId,
          TaskState.VERIFICATION_FAILED,
          `Verification failed: ${verification.reason}`
        );

        throw new VerificationFailedError(verification.reason);
      }

      await this.stateMachine.transition(
        taskId,
        TaskState.VERIFIED,
        "Verification passed"
      );
    });
  }

  private async handleTaskFailure(
    taskId: string,
    error: Error,
    span: Span
  ): Promise<void> {
    // Transition to failed state
    await this.stateMachine.transition(
      taskId,
      TaskState.FAILED,
      `Task failed: ${error.message}`
    );

    // Record failure
    span.status = "ERROR";
    span.error = error;

    // Log structured error
    logger.error("Task orchestration failed", {
      taskId,
      error: error.message,
      stack: error.stack,
      traceId: span.traceId,
    });
  }
}
```

**Deliverables**:

- ✅ Complete orchestration pipeline
- ✅ All acceptance criteria implemented (A1-A6)
- ✅ Error handling and recovery
- ✅ Performance tracking integration
- ✅ Distributed tracing

---

### Task 1.3: Constitutional Runtime (3 days)

**Goal**: Implement constitutional authority enforcement layer

**Constitutional Runtime**:

```typescript
// File: src/orchestrator/ConstitutionalRuntime.ts

export class ConstitutionalRuntime {
  constructor(
    private cawsValidator: SpecValidator,
    private performanceTracker: PerformanceTracker,
    private tracing: DistributedTracing
  ) {}

  // Pre-execution constitutional checks
  async validatePreExecution(
    request: TaskRequest
  ): Promise<ConstitutionalDecision> {
    return await this.tracing.traceWorkflow(
      "constitutional_pre_check",
      async (span) => {
        const checks: ConstitutionalCheck[] = [];

        // Check 1: Working spec validity
        const specValidation = await this.cawsValidator.validateWorkingSpec(
          request.spec
        );
        checks.push({
          name: "working_spec_validity",
          passed: specValidation.valid,
          violations: specValidation.violations,
        });

        // Check 2: Budget compliance
        const budgetCheck = this.checkBudgetCompliance(request);
        checks.push(budgetCheck);

        // Check 3: Risk tier appropriateness
        const riskCheck = this.checkRiskTierAppropriateness(request);
        checks.push(riskCheck);

        // Check 4: Waiver requirements
        const waiverCheck = await this.checkWaiverRequirements(request);
        checks.push(waiverCheck);

        const allPassed = checks.every((c) => c.passed);

        return {
          allowed: allPassed,
          checks,
          rationale: allPassed
            ? "All constitutional checks passed"
            : `Failed checks: ${checks
                .filter((c) => !c.passed)
                .map((c) => c.name)
                .join(", ")}`,
        };
      }
    );
  }

  // Post-execution constitutional verification
  async validatePostExecution(
    result: TaskResult,
    spec: WorkingSpec
  ): Promise<ConstitutionalVerification> {
    return await this.tracing.traceWorkflow(
      "constitutional_post_check",
      async (span) => {
        const verifications: ConstitutionalVerification[] = [];

        // Verification 1: Budget adherence
        const budgetVerification = this.verifyBudgetAdherence(result, spec);
        verifications.push(budgetVerification);

        // Verification 2: Quality gate results
        const qualityGateVerification = await this.verifyQualityGates(
          result,
          spec
        );
        verifications.push(qualityGateVerification);

        // Verification 3: Acceptance criteria met
        const acceptanceVerification = this.verifyAcceptanceCriteria(
          result,
          spec
        );
        verifications.push(acceptanceVerification);

        // Verification 4: Invariants preserved
        const invariantVerification = this.verifyInvariants(result, spec);
        verifications.push(invariantVerification);

        const allPassed = verifications.every((v) => v.passed);

        // Record constitutional compliance
        await this.performanceTracker.recordConstitutionalValidation(
          spec.id,
          allPassed,
          verifications.filter((v) => !v.passed).length,
          allPassed ? 1.0 : 0.0
        );

        return {
          compliant: allPassed,
          verifications,
          complianceScore: allPassed ? 1.0 : 0.0,
        };
      }
    );
  }

  // Constitutional decision generation
  async generateVerdict(
    taskId: string,
    request: TaskRequest,
    result: TaskResult,
    verification: ConstitutionalVerification
  ): Promise<CAWSVerdict> {
    // Generate structured verdict
    const verdict: CAWSVerdict = {
      verdictId: `VERDICT-${taskId}-${Date.now()}`,
      taskId,
      timestamp: new Date(),
      decision: verification.compliant ? "PASS" : "FAIL",
      checks: {
        preExecution: await this.validatePreExecution(request),
        postExecution: verification,
      },
      rationale: this.generateVerdictRationale(verification),
      waiverRequired:
        !verification.compliant && this.isWaiverEligible(verification),
      nextSteps: this.determineNextSteps(verification),
    };

    // Sign verdict (cryptographic signature for immutability)
    verdict.signature = await this.signVerdict(verdict);

    return verdict;
  }

  // Verdict publication to git (for immutable provenance)
  async publishVerdict(verdict: CAWSVerdict): Promise<void> {
    // This would integrate with git to commit verdict as immutable record
    // For now, we'll persist to database
    await this.persistVerdict(verdict);

    // Emit event for audit trail
    this.emit("verdict_published", verdict);
  }
}
```

**Deliverables**:

- ✅ Pre-execution constitutional validation
- ✅ Post-execution constitutional verification
- ✅ Verdict generation and signing
- ✅ Integration with git for provenance (future)

---

## Phase 2: System Coordination (1-2 Weeks)

**Purpose**: Implement system-level coordination, health monitoring, and recovery

### Task 2.1: System Coordinator (3 days)

**Goal**: Coordinate interactions between all components

```typescript
// File: src/orchestrator/SystemCoordinator.ts

export class SystemCoordinator {
  private componentHealth: Map<string, ComponentHealth> = new Map();
  private circuitBreakers: Map<string, CircuitBreaker> = new Map();

  async coordinateComponents(): Promise<SystemStatus> {
    // Check health of all components
    const healthChecks = await Promise.all([
      this.checkComponentHealth("agent_registry", () =>
        this.agentRegistry.healthCheck()
      ),
      this.checkComponentHealth("task_router", () =>
        this.taskRouter.healthCheck()
      ),
      this.checkComponentHealth("caws_validator", () =>
        this.cawsValidator.healthCheck()
      ),
      this.checkComponentHealth("performance_tracker", () =>
        this.performanceTracker.healthCheck()
      ),
    ]);

    // Aggregate system status
    const systemStatus: SystemStatus = {
      healthy: healthChecks.every((h) => h.healthy),
      components: Object.fromEntries(this.componentHealth),
      degradedComponents: healthChecks
        .filter((h) => !h.healthy)
        .map((h) => h.name),
      timestamp: new Date(),
    };

    // If system is degraded, initiate recovery
    if (!systemStatus.healthy) {
      await this.initiateRecovery(systemStatus);
    }

    return systemStatus;
  }

  private async checkComponentHealth(
    name: string,
    healthCheck: () => Promise<HealthCheckResult>
  ): Promise<ComponentHealth> {
    try {
      const result = await Promise.race([
        healthCheck(),
        timeout(5000), // 5 second timeout
      ]);

      const health: ComponentHealth = {
        name,
        healthy: result.healthy,
        lastCheck: new Date(),
        consecutiveFailures: 0,
      };

      this.componentHealth.set(name, health);
      return health;
    } catch (error) {
      const health = this.componentHealth.get(name) || {
        name,
        healthy: false,
        lastCheck: new Date(),
        consecutiveFailures: 0,
      };

      health.healthy = false;
      health.consecutiveFailures++;
      health.lastError = error.message;

      this.componentHealth.set(name, health);
      return health;
    }
  }

  private async initiateRecovery(systemStatus: SystemStatus): Promise<void> {
    // Log degradation
    logger.warn("System degradation detected", systemStatus);

    // For each degraded component, attempt recovery
    for (const componentName of systemStatus.degradedComponents) {
      await this.recoverComponent(componentName);
    }
  }

  private async recoverComponent(name: string): Promise<void> {
    const health = this.componentHealth.get(name);

    if (!health) return;

    // If consecutive failures < threshold, wait and retry
    if (health.consecutiveFailures < 3) {
      logger.info(`Waiting before retry for ${name}`, {
        attempts: health.consecutiveFailures,
      });
      await sleep(5000);
      return;
    }

    // If consecutive failures >= threshold, attempt full recovery
    logger.warn(`Initiating recovery for ${name}`, {
      failures: health.consecutiveFailures,
    });

    try {
      switch (name) {
        case "agent_registry":
          await this.agentRegistry.reinitialize();
          break;
        case "task_router":
          await this.taskRouter.reinitialize();
          break;
        case "caws_validator":
          await this.cawsValidator.reinitialize();
          break;
        case "performance_tracker":
          await this.performanceTracker.reinitialize();
          break;
      }

      // Reset failure count
      health.consecutiveFailures = 0;
      health.healthy = true;

      logger.info(`Successfully recovered ${name}`);
    } catch (error) {
      logger.error(`Failed to recover ${name}`, { error: error.message });

      // Emit alert for manual intervention
      this.emit("recovery_failed", { component: name, error });
    }
  }
}
```

**Deliverables**:

- ✅ Component health monitoring
- ✅ Automatic recovery mechanisms
- ✅ Circuit breakers for all interactions
- ✅ System-level status reporting

---

### Task 2.2: Feedback Loop Manager (2 days)

**Goal**: Implement constitutional feedback loops

```typescript
// File: src/orchestrator/FeedbackLoopManager.ts

export class FeedbackLoopManager {
  // Update agent performance based on task outcomes
  async processTaskOutcome(
    taskId: string,
    agentId: string,
    outcome: TaskOutcome
  ): Promise<void> {
    // Update agent performance history
    await this.agentRegistry.updatePerformance(agentId, {
      success: outcome.success,
      latencyMs: outcome.metrics.duration,
      qualityScore: outcome.quality,
    });

    // Update routing policy if performance changes
    if (outcome.success === false) {
      // Penalize failed agent
      await this.taskRouter.penalizeAgent(agentId, outcome.failureReason);
    } else {
      // Reward successful agent
      await this.taskRouter.rewardAgent(agentId, outcome.quality);
    }

    // Generate RL training data
    await this.performanceTracker.generateTrainingSample(
      taskId,
      agentId,
      outcome
    );
  }

  // Update constitutional rules based on violation patterns
  async processConstitutionalViolations(
    violations: CAWSViolation[]
  ): Promise<void> {
    // Analyze violation patterns
    const patterns = this.analyzeViolationPatterns(violations);

    // If pattern detected, suggest rule updates
    if (patterns.length > 0) {
      logger.info("Constitutional violation patterns detected", { patterns });

      // Emit alert for human review
      this.emit("violation_pattern", { patterns });
    }
  }
}
```

**Deliverables**:

- ✅ Agent performance feedback loop
- ✅ Constitutional rule update mechanism
- ✅ RL training data generation

---

## Phase 3: Testing & Validation (1-2 Weeks)

### Task 3.1: Unit Tests (3 days)

**Test Coverage**:

- TaskOrchestrator: All methods, all states
- ConstitutionalRuntime: All checks and verifications
- SystemCoordinator: Health checks, recovery
- FeedbackLoopManager: All feedback loops

**Target**: 90%+ coverage

---

### Task 3.2: Integration Tests (4 days)

**Test Scenarios**:

- End-to-end happy path (A1)
- Constitutional violations (A2)
- Component failures and recovery (A3)
- High-load testing (A4)
- Rule updates (A5)
- Long-running stability (A6)

**Target**: All acceptance criteria validated

---

### Task 3.3: Load & Performance Testing (2 days)

**Performance Targets**:

- 95% of tasks < 500ms end-to-end (A1)
- 2000 concurrent tasks supported (A4)
- 99.9% uptime (A6)
- 99.99% constitutional compliance

**Test Infrastructure**:

- Load generation with realistic workloads
- Performance profiling and optimization
- Memory leak detection

---

## Phase 4: Production Deployment (1 Week)

### Task 4.1: Documentation (2 days)

**Documentation Deliverables**:

- Architecture diagrams
- API documentation
- Deployment guide
- Operations runbook
- Troubleshooting guide

---

### Task 4.2: Deployment Preparation (2 days)

**Infrastructure**:

- Database migrations
- Configuration management
- Monitoring dashboards
- Alert routing

---

### Task 4.3: Production Validation (3 days)

**Validation Checklist**:

- All tests passing
- Performance benchmarks met
- Security audit complete
- Documentation complete
- Deployment automation tested

---

## Risk Assessment & Mitigation

### High-Risk Areas

**Risk 1: Integration Complexity**

- **Mitigation**: Comprehensive integration tests, circuit breakers
- **Contingency**: Graceful degradation, manual failover

**Risk 2: Performance Bottlenecks**

- **Mitigation**: Early performance testing, profiling
- **Contingency**: Horizontal scaling, caching

**Risk 3: State Inconsistency**

- **Mitigation**: Transactional state updates, distributed tracing
- **Contingency**: State reconciliation, rollback procedures

**Risk 4: Constitutional Authority Bypass**

- **Mitigation**: Mandatory pre/post checks, verdict signing
- **Contingency**: Audit trail analysis, automatic shutdown

---

## Success Criteria

### Functional Requirements

- ✅ All acceptance criteria (A1-A6) passing
- ✅ 90%+ test coverage
- ✅ All components integrated successfully
- ✅ Constitutional authority enforced

### Performance Requirements

- ✅ <500ms P95 end-to-end latency
- ✅ 2000 concurrent tasks supported
- ✅ 99.9% uptime
- ✅ 99.99% constitutional compliance

### Quality Requirements

- ✅ Zero TypeScript errors
- ✅ Zero linting errors
- ✅ All tests passing
- ✅ Security audit passed

---

## Timeline Summary

| Phase                        | Duration  | Dependencies       |
| ---------------------------- | --------- | ------------------ |
| Phase 0: Foundation          | 1-2 weeks | ARBITER-001 to 004 |
| Phase 1: Core Orchestration  | 2-3 weeks | Phase 0 complete   |
| Phase 2: System Coordination | 1-2 weeks | Phase 1 complete   |
| Phase 3: Testing             | 1-2 weeks | Phase 2 complete   |
| Phase 4: Production          | 1 week    | Phase 3 complete   |

**Total Duration**: **6-10 weeks**

---

## Conclusion

ARBITER-005 is the culmination of all previous work - the orchestrator that brings everything together under constitutional authority.

**Key Success Factors**:

1. **Solid Foundation**: Complete Phase 0 before starting orchestration
2. **Clear State Management**: State machine prevents invalid transitions
3. **Comprehensive Testing**: Test-first approach ensures correctness
4. **Production Readiness**: Observability and resilience from day one

**Recommendation**: Follow the phased approach. Don't rush. Build it right.

---

**Next Steps**:

1. Review this plan with stakeholders
2. Complete Phase 0 (foundation hardening)
3. Begin Phase 1 (core orchestration)

**Questions for User**:

1. Do you agree with the phased approach?
2. Should we prioritize Phase 0 foundation work first?
3. Any concerns about the 6-10 week timeline?
