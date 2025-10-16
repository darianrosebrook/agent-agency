/**
 * @fileoverview Worker pool supervisor handling saturation and failure recovery.
 */

export type WorkerPriority = "low" | "normal" | "high" | "urgent";

export interface WorkerDescriptor {
  id: string;
  capabilities: string[];
}

export interface SupervisorMetrics {
  saturationRatio: number;
  queueDepth: number;
  idleWorkers: number;
  busyWorkers: number;
  totalWorkers: number;
}

export interface BackpressureState {
  active: boolean;
  reason?: "worker_saturation" | "queue_depth";
  since?: Date;
}

export interface SupervisorDecisionAssign {
  type: "assign";
  workerId: string;
  metrics: SupervisorMetrics;
}

export interface SupervisorDecisionQueue {
  type: "queue";
  metrics: SupervisorMetrics;
}

export interface SupervisorDecisionBackpressure {
  type: "backpressure";
  metrics: SupervisorMetrics;
}

export type SupervisorDecision =
  | SupervisorDecisionAssign
  | SupervisorDecisionQueue
  | SupervisorDecisionBackpressure;

export interface EvaluateCapacityContext {
  queueDepth: number;
  priority: WorkerPriority;
  requiredCapabilities: string[];
}

export interface WorkerPoolSupervisorConfig {
  maxWorkers: number;
  backpressure: {
    saturationRatio: number;
    queueDepth: number;
    cooldownMs: number;
  };
  retry: {
    baseDelayMs: number;
    maxDelayMs: number;
    maxAttempts: number;
  };
}

export interface FailureMetadata {
  errorType: string;
  [key: string]: unknown;
}

export interface TaskSnapshot {
  taskId: string;
  attempt: number;
  lastFailureAt: Date;
  metadata: FailureMetadata;
}

export interface FailurePlan {
  shouldRetry: boolean;
  retryAfterMs: number;
  snapshot: TaskSnapshot;
}

export class WorkerPoolSupervisor {
  private readonly config: WorkerPoolSupervisorConfig;
  private readonly workers = new Map<string, WorkerDescriptor>();
  private readonly idleWorkers = new Set<string>();
  private readonly busyWorkers = new Map<string, { taskId: string; startedAt: number }>();
  private readonly taskSnapshots = new Map<string, TaskSnapshot>();
  private backpressureState: BackpressureState = { active: false };

  constructor(config: WorkerPoolSupervisorConfig) {
    this.config = config;
  }

  registerWorker(worker: WorkerDescriptor): void {
    this.workers.set(worker.id, worker);
    if (!this.busyWorkers.has(worker.id)) {
      this.idleWorkers.add(worker.id);
    }
  }

  markWorkerBusy(workerId: string, taskId: string): void {
    this.idleWorkers.delete(workerId);
    this.busyWorkers.set(workerId, { taskId, startedAt: Date.now() });
  }

  markWorkerIdle(workerId: string): void {
    if (this.busyWorkers.has(workerId)) {
      this.busyWorkers.delete(workerId);
    }
    if (this.workers.has(workerId)) {
      this.idleWorkers.add(workerId);
    }
    // Clear lingering task snapshot entries for this worker
    for (const [taskId, snapshot] of this.taskSnapshots.entries()) {
      if (snapshot.metadata.workerId === workerId) {
        this.taskSnapshots.delete(taskId);
      }
    }
    if (this.backpressureState.active) {
      this.backpressureState = { active: false };
    }
  }

  getBackpressureState(): BackpressureState {
    return this.backpressureState;
  }

  evaluateCapacity(context: EvaluateCapacityContext): SupervisorDecision {
    const metrics = this.computeMetrics(context.queueDepth);
    const workerId = this.findEligibleWorker(context.requiredCapabilities);

    if (workerId) {
      this.backpressureState = { active: false };
      return {
        type: "assign",
        workerId,
        metrics,
      };
    }

    const saturated =
      metrics.saturationRatio >= this.config.backpressure.saturationRatio;
    const queueExceeded =
      context.queueDepth >= this.config.backpressure.queueDepth;

    if (saturated || queueExceeded) {
      const reason = saturated ? "worker_saturation" : "queue_depth";
      this.backpressureState = {
        active: true,
        reason,
        since: this.backpressureState.active ? this.backpressureState.since : new Date(),
      };

      return {
        type: "backpressure",
        metrics,
      };
    }

    return {
      type: "queue",
      metrics,
    };
  }

  recordWorkerFailure(
    workerId: string,
    taskId: string,
    metadata: FailureMetadata
  ): FailurePlan {
    this.busyWorkers.delete(workerId);
    if (this.workers.has(workerId)) {
      this.idleWorkers.add(workerId);
    }

    const prevSnapshot = this.taskSnapshots.get(taskId);
    const attempt = (prevSnapshot?.attempt ?? 0) + 1;

    const retryDelay = this.calculateRetryDelay(attempt);
    const shouldRetry = attempt <= this.config.retry.maxAttempts;

    const snapshot: TaskSnapshot = {
      taskId,
      attempt,
      lastFailureAt: new Date(),
      metadata: {
        ...metadata,
        workerId,
      },
    };

    this.taskSnapshots.set(taskId, snapshot);

    return {
      shouldRetry,
      retryAfterMs: shouldRetry ? retryDelay : 0,
      snapshot,
    };
  }

  private calculateRetryDelay(attempt: number): number {
    const exponentialDelay =
      this.config.retry.baseDelayMs * Math.pow(2, Math.max(0, attempt - 1));
    return Math.min(exponentialDelay, this.config.retry.maxDelayMs);
  }

  private computeMetrics(queueDepth: number): SupervisorMetrics {
    const totalWorkers = Math.max(this.workers.size, this.config.maxWorkers);
    const busyCount = this.busyWorkers.size;
    const saturationRatio =
      totalWorkers === 0 ? 0 : busyCount / totalWorkers;

    return {
      saturationRatio,
      queueDepth,
      idleWorkers: this.idleWorkers.size,
      busyWorkers: busyCount,
      totalWorkers,
    };
  }

  private findEligibleWorker(requiredCapabilities: string[]): string | null {
    if (requiredCapabilities.length === 0) {
      return this.idleWorkers.values().next().value ?? null;
    }

    for (const workerId of this.idleWorkers) {
      const descriptor = this.workers.get(workerId);
      if (!descriptor) {
        continue;
      }

      const hasAllCapabilities = requiredCapabilities.every((cap) =>
        descriptor.capabilities.includes(cap)
      );

      if (hasAllCapabilities) {
        return workerId;
      }
    }

    return null;
  }
}
