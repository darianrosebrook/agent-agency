/**
 * @fileoverview Worker Capability Registry - ARBITER-022
 *
 * Manages worker capability registration, health tracking, and capacity queries
 * with PostgreSQL backing for real-time orchestration decisions.
 *
 * @author @darianrosebrook
 */

import {
  CapabilityQuery,
  WorkerCapability,
  WorkerCapabilityRepository,
  WorkerRegistrationRequest,
} from "../repositories/WorkerCapabilityRepository";

export interface WorkerCapabilityRegistry {
  /**
   * Register a new worker with capabilities
   */
  register(request: WorkerRegistrationRequest): Promise<WorkerCapability>;

  /**
   * Deregister a worker
   */
  deregister(workerId: string): Promise<void>;

  /**
   * Update worker health and saturation
   */
  updateHealth(
    workerId: string,
    healthStatus: WorkerCapability["healthStatus"],
    saturationRatio: number
  ): Promise<void>;

  /**
   * Send heartbeat to keep worker alive
   */
  heartbeat(workerId: string): Promise<void>;

  /**
   * Query workers by capability requirements
   */
  queryCapabilities(query: CapabilityQuery): Promise<WorkerCapability[]>;

  /**
   * Get all registered workers
   */
  getAllWorkers(): Promise<WorkerCapability[]>;

  /**
   * Get worker by ID
   */
  getWorker(workerId: string): Promise<WorkerCapability | null>;

  /**
   * Clean up stale workers
   */
  cleanupStaleWorkers(staleThresholdMs?: number): Promise<string[]>;

  /**
   * Get registry statistics
   */
  getStatistics(): Promise<WorkerRegistryStatistics>;
}

export interface WorkerRegistryStatistics {
  totalWorkers: number;
  healthyWorkers: number;
  degradedWorkers: number;
  unhealthyWorkers: number;
  averageSaturation: number;
  capabilityDistribution: Record<string, number>;
  lastCleanup: Date | null;
}

/**
 * Implementation of WorkerCapabilityRegistry with PostgreSQL backing
 */
export class WorkerCapabilityRegistryImpl implements WorkerCapabilityRegistry {
  private cleanupInterval: NodeJS.Timeout | null = null;
  private lastCleanup: Date | null = null;

  constructor(
    private repository: WorkerCapabilityRepository,
    private config: {
      cleanupIntervalMs?: number;
      defaultStaleThresholdMs?: number;
    } = {}
  ) {
    this.startPeriodicCleanup();
  }

  async register(
    request: WorkerRegistrationRequest
  ): Promise<WorkerCapability> {
    try {
      const capability = await this.repository.register(request);

      // Emit registration event
      this.emitWorkerEvent("registered", capability);

      return capability;
    } catch (error) {
      throw new Error(
        `Failed to register worker ${request.workerId}: ${error}`
      );
    }
  }

  async deregister(workerId: string): Promise<void> {
    try {
      await this.repository.deregister(workerId);

      // Emit deregistration event
      this.emitWorkerEvent("deregistered", { workerId });
    } catch (error) {
      throw new Error(`Failed to deregister worker ${workerId}: ${error}`);
    }
  }

  async updateHealth(
    workerId: string,
    healthStatus: WorkerCapability["healthStatus"],
    saturationRatio: number
  ): Promise<void> {
    try {
      await this.repository.updateHealth(
        workerId,
        healthStatus,
        saturationRatio
      );

      // Emit health update event
      this.emitWorkerEvent("health_updated", {
        workerId,
        healthStatus,
        saturationRatio,
      });
    } catch (error) {
      throw new Error(
        `Failed to update health for worker ${workerId}: ${error}`
      );
    }
  }

  async heartbeat(workerId: string): Promise<void> {
    try {
      await this.repository.heartbeat(workerId);
    } catch (error) {
      throw new Error(
        `Failed to record heartbeat for worker ${workerId}: ${error}`
      );
    }
  }

  async queryCapabilities(query: CapabilityQuery): Promise<WorkerCapability[]> {
    try {
      return await this.repository.queryCapabilities(query);
    } catch (error) {
      throw new Error(`Failed to query capabilities: ${error}`);
    }
  }

  async getAllWorkers(): Promise<WorkerCapability[]> {
    try {
      return await this.repository.getAllWorkers();
    } catch (error) {
      throw new Error(`Failed to get all workers: ${error}`);
    }
  }

  async getWorker(workerId: string): Promise<WorkerCapability | null> {
    try {
      return await this.repository.getWorker(workerId);
    } catch (error) {
      throw new Error(`Failed to get worker ${workerId}: ${error}`);
    }
  }

  async cleanupStaleWorkers(staleThresholdMs?: number): Promise<string[]> {
    const threshold =
      staleThresholdMs ?? this.config.defaultStaleThresholdMs ?? 300000; // 5 minutes default

    try {
      const staleWorkerIds = await this.repository.cleanupStaleWorkers(
        threshold
      );
      this.lastCleanup = new Date();

      if (staleWorkerIds.length > 0) {
        // Emit cleanup event
        this.emitWorkerEvent("cleanup", { staleWorkerIds });
      }

      return staleWorkerIds;
    } catch (error) {
      throw new Error(`Failed to cleanup stale workers: ${error}`);
    }
  }

  async getStatistics(): Promise<WorkerRegistryStatistics> {
    try {
      const workers = await this.getAllWorkers();

      const healthyWorkers = workers.filter(
        (w) => w.healthStatus === "healthy"
      ).length;
      const degradedWorkers = workers.filter(
        (w) => w.healthStatus === "degraded"
      ).length;
      const unhealthyWorkers = workers.filter(
        (w) => w.healthStatus === "unhealthy"
      ).length;

      const averageSaturation =
        workers.length > 0
          ? workers.reduce((sum, w) => sum + w.saturationRatio, 0) /
            workers.length
          : 0;

      // Calculate capability distribution
      const capabilityDistribution: Record<string, number> = {};
      for (const worker of workers) {
        for (const capability of Object.keys(worker.capabilities)) {
          capabilityDistribution[capability] =
            (capabilityDistribution[capability] || 0) + 1;
        }
      }

      return {
        totalWorkers: workers.length,
        healthyWorkers,
        degradedWorkers,
        unhealthyWorkers,
        averageSaturation,
        capabilityDistribution,
        lastCleanup: this.lastCleanup,
      };
    } catch (error) {
      throw new Error(`Failed to get registry statistics: ${error}`);
    }
  }

  /**
   * Start periodic cleanup of stale workers
   */
  private startPeriodicCleanup(): void {
    const interval = this.config.cleanupIntervalMs ?? 60000; // 1 minute default

    this.cleanupInterval = setInterval(async () => {
      try {
        await this.cleanupStaleWorkers();
      } catch (error) {
        console.error("Failed to cleanup stale workers:", error);
      }
    }, interval);
  }

  /**
   * Stop periodic cleanup
   */
  public stopPeriodicCleanup(): void {
    if (this.cleanupInterval) {
      clearInterval(this.cleanupInterval);
      this.cleanupInterval = null;
    }
  }

  /**
   * Emit worker events (can be extended with EventEmitter)
   */
  private emitWorkerEvent(event: string, data: any): void {
    // This can be extended to use EventEmitter for real-time notifications
    console.log(`Worker event: ${event}`, data);
  }

  /**
   * Find best workers for a task based on capabilities and load
   */
  async findBestWorkers(
    requiredCapabilities: string[],
    maxSaturationRatio: number = 0.8,
    limit: number = 5
  ): Promise<WorkerCapability[]> {
    const query: CapabilityQuery = {
      requiredCapabilities,
      maxSaturationRatio,
      minHealthStatus: "healthy",
      limit,
    };

    const candidates = await this.queryCapabilities(query);

    // Sort by saturation ratio (lower is better) and then by last heartbeat (more recent is better)
    return candidates.sort((a, b) => {
      const saturationDiff = a.saturationRatio - b.saturationRatio;
      if (Math.abs(saturationDiff) > 0.01) {
        return saturationDiff;
      }
      return b.lastHeartbeat.getTime() - a.lastHeartbeat.getTime();
    });
  }

  /**
   * Check if a worker has the required capabilities
   */
  async hasCapabilities(
    workerId: string,
    requiredCapabilities: string[]
  ): Promise<boolean> {
    const worker = await this.getWorker(workerId);
    if (!worker) {
      return false;
    }

    return requiredCapabilities.every((capability) =>
      worker.capabilities.hasOwnProperty(capability)
    );
  }

  /**
   * Get workers with specific capability
   */
  async getWorkersWithCapability(
    capability: string
  ): Promise<WorkerCapability[]> {
    return this.queryCapabilities({
      requiredCapabilities: [capability],
      limit: 100, // Get all workers with this capability
    });
  }
}

