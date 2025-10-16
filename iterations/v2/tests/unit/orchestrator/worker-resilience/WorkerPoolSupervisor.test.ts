/**
 * @fileoverview WorkerPoolSupervisor resilience behaviour tests (Phase 2).
 */

import { WorkerPoolSupervisor } from "../../../../src/orchestrator/worker/WorkerPoolSupervisor";

const SUPERVISOR_CONFIG = {
  maxWorkers: 2,
  backpressure: {
    saturationRatio: 0.8,
    queueDepth: 2,
    cooldownMs: 1000,
  },
  retry: {
    baseDelayMs: 500,
    maxDelayMs: 5000,
    maxAttempts: 3,
  },
};

function buildSupervisor() {
  const supervisor = new WorkerPoolSupervisor(SUPERVISOR_CONFIG);
  supervisor.registerWorker({
    id: "worker-1",
    capabilities: ["analysis"],
  });
  supervisor.registerWorker({
    id: "worker-2",
    capabilities: ["analysis"],
  });
  return supervisor;
}

describe("WorkerPoolSupervisor", () => {
  it("enters backpressure when workers are saturated and queue depth exceeds threshold", () => {
    const supervisor = buildSupervisor();

    supervisor.markWorkerBusy("worker-1", "task-1");
    supervisor.markWorkerBusy("worker-2", "task-2");

    const decision = supervisor.evaluateCapacity({
      queueDepth: 3,
      priority: "normal",
      requiredCapabilities: ["analysis"],
    });

    expect(decision.type).toBe("backpressure");
    expect(decision.metrics.saturationRatio).toBe(1);
    expect(decision.metrics.queueDepth).toBe(3);

    const backpressureState = supervisor.getBackpressureState();
    expect(backpressureState.active).toBe(true);
    expect(backpressureState.reason).toBe("worker_saturation");
  });

  it("releases backpressure when capacity returns and offers assignment", () => {
    const supervisor = buildSupervisor();

    supervisor.markWorkerBusy("worker-1", "task-1");
    supervisor.markWorkerBusy("worker-2", "task-2");

    supervisor.evaluateCapacity({
      queueDepth: 3,
      priority: "normal",
      requiredCapabilities: ["analysis"],
    });

    supervisor.markWorkerIdle("worker-1");

    const decision = supervisor.evaluateCapacity({
      queueDepth: 1,
      priority: "normal",
      requiredCapabilities: ["analysis"],
    });

    expect(decision.type).toBe("assign");
    if (decision.type !== "assign") {
      throw new Error("Expected assignment decision");
    }
    expect(decision.workerId).toBe("worker-1");
    expect(supervisor.getBackpressureState().active).toBe(false);
  });

  it("requeues failed task with exponential backoff snapshot", () => {
    const supervisor = buildSupervisor();

    supervisor.markWorkerBusy("worker-1", "task-1");
    const failurePlan = supervisor.recordWorkerFailure("worker-1", "task-1", {
      errorType: "crash",
    });

    expect(failurePlan.shouldRetry).toBe(true);
    expect(failurePlan.retryAfterMs).toBeGreaterThanOrEqual(
      SUPERVISOR_CONFIG.retry.baseDelayMs
    );
    expect(failurePlan.snapshot.taskId).toBe("task-1");
    expect(failurePlan.snapshot.attempt).toBe(1);

    const retryPlan = supervisor.recordWorkerFailure("worker-1", "task-1", {
      errorType: "timeout",
    });
    expect(retryPlan.retryAfterMs).toBeGreaterThan(failurePlan.retryAfterMs);
    expect(retryPlan.snapshot.attempt).toBe(2);
  });
});
