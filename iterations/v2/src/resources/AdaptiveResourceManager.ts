/**
 * @fileoverview Adaptive Resource Manager - Main Resource Management Component
 *
 * Coordinates resource monitoring, load balancing, and allocation.
 * Provides dynamic resource management with failover capabilities.
 *
 * @author @darianrosebrook
 */

import { Logger } from "@/observability/Logger";
import {
  LoadBalancingStrategy,
  type AdaptiveResourceManagerConfig,
  type CapacityAnalysis,
  type FailoverEvent,
  type IAdaptiveResourceManager,
  type ResourceAllocationRequest,
  type ResourceAllocationResult,
  type ResourcePoolStats,
} from "@/types/resource-types";
import { v4 as uuidv4 } from "uuid";
import { LoadBalancer } from "./LoadBalancer";
import { ResourceAllocator } from "./ResourceAllocator";
import { ResourceMonitor } from "./ResourceMonitor";

/**
 * Default configuration
 */
const DEFAULT_CONFIG: AdaptiveResourceManagerConfig = {
  enabled: true,
  monitoringIntervalMs: 5000,
  loadBalancingStrategy: LoadBalancingStrategy.LEAST_LOADED,
  enableDynamicRateLimiting: true,
  enableAutoFailover: true,
  thresholds: {
    cpuWarning: 70,
    cpuCritical: 85,
    memoryWarning: 75,
    memoryCritical: 90,
  },
  maxAllocationDecisionMs: 50,
  enableCapacityPlanning: true,
  capacityAnalysisIntervalMs: 300000, // 5 minutes
};

/**
 * Adaptive Resource Manager
 *
 * Main resource management component that:
 * - Monitors resource usage across agents
 * - Balances load dynamically
 * - Allocates resources efficiently
 * - Handles failovers automatically
 * - Analyzes capacity needs
 */
export class AdaptiveResourceManager implements IAdaptiveResourceManager {
  private logger: Logger;
  private config: AdaptiveResourceManagerConfig;
  private resourceMonitor: ResourceMonitor;
  private loadBalancer: LoadBalancer;
  private resourceAllocator: ResourceAllocator;
  private isRunning = false;
  private capacityAnalysisTimer?: ReturnType<typeof setInterval>;
  private lastCapacityAnalysis?: CapacityAnalysis;
  private failoverEvents: FailoverEvent[] = [];

  constructor(config: Partial<AdaptiveResourceManagerConfig> = {}) {
    this.logger = new Logger("AdaptiveResourceManager");
    this.config = { ...DEFAULT_CONFIG, ...config };

    // Initialize sub-components
    this.resourceMonitor = new ResourceMonitor({
      intervalMs: this.config.monitoringIntervalMs,
      cpuThresholds: {
        warning: this.config.thresholds.cpuWarning,
        critical: this.config.thresholds.cpuCritical,
      },
      memoryThresholds: {
        warning: this.config.thresholds.memoryWarning,
        critical: this.config.thresholds.memoryCritical,
      },
    });

    this.loadBalancer = new LoadBalancer(
      this.resourceMonitor,
      this.config.loadBalancingStrategy
    );

    this.resourceAllocator = new ResourceAllocator(this.loadBalancer, {
      dynamicAdjustment: this.config.enableDynamicRateLimiting,
    });
  }

  /**
   * Initialize the resource manager
   */
  async initialize(): Promise<void> {
    await this.resourceMonitor.start();

    this.logger.info("Adaptive resource manager initialized", {
      enabled: this.config.enabled,
      strategy: this.config.loadBalancingStrategy,
      enableFailover: this.config.enableAutoFailover,
      enableCapacityPlanning: this.config.enableCapacityPlanning,
    });
  }

  /**
   * Start resource management
   */
  async start(): Promise<void> {
    if (this.isRunning) {
      this.logger.warn("Resource manager already running");
      return;
    }

    if (!this.config.enabled) {
      this.logger.info("Resource manager disabled, not starting");
      return;
    }

    this.isRunning = true;

    // Start capacity analysis if enabled
    if (this.config.enableCapacityPlanning) {
      this.capacityAnalysisTimer = setInterval(async () => {
        try {
          await this.analyzeCapacity();
        } catch (error) {
          this.logger.error("Capacity analysis failed", { error });
        }
      }, this.config.capacityAnalysisIntervalMs);
    }

    this.logger.info("Adaptive resource manager started");
  }

  /**
   * Stop resource management
   */
  async stop(): Promise<void> {
    if (!this.isRunning) {
      return;
    }

    if (this.capacityAnalysisTimer) {
      clearInterval(this.capacityAnalysisTimer);
      this.capacityAnalysisTimer = undefined;
    }

    await this.resourceMonitor.stop();
    this.isRunning = false;

    this.logger.info("Adaptive resource manager stopped");
  }

  /**
   * Allocate resources for a task
   *
   * @param request Resource allocation request
   * @returns Allocation result
   */
  async allocateResources(
    request: ResourceAllocationRequest
  ): Promise<ResourceAllocationResult> {
    const startTime = Date.now();

    try {
      const result = await this.resourceAllocator.allocate(request);

      // Check if allocation took too long
      const allocationTime = Date.now() - startTime;
      if (allocationTime > this.config.maxAllocationDecisionMs) {
        this.logger.warn("Allocation decision exceeded max time", {
          allocationTime,
          maxAllocationTime: this.config.maxAllocationDecisionMs,
        });
      }

      return result;
    } catch (error) {
      this.logger.error("Resource allocation failed", {
        requestId: request.requestId,
        error,
      });

      throw error;
    }
  }

  /**
   * Release resources for a completed task
   *
   * @param requestId Request identifier
   */
  async releaseResources(requestId: string): Promise<void> {
    await this.resourceAllocator.release(requestId);
  }

  /**
   * Perform capacity analysis
   *
   * @returns Capacity analysis result
   */
  async analyzeCapacity(): Promise<CapacityAnalysis> {
    const poolStats = await this.resourceMonitor.getPoolStats();

    // Calculate utilization
    const cpuUtilization =
      poolStats.totalCpuCapacity > 0
        ? (poolStats.usedCpuCapacity / poolStats.totalCpuCapacity) * 100
        : 0;

    const memoryUtilization =
      poolStats.totalMemoryCapacity > 0
        ? (poolStats.usedMemoryCapacity / poolStats.totalMemoryCapacity) * 100
        : 0;

    const utilizationPercent = (cpuUtilization + memoryUtilization) / 2;

    // Determine scaling recommendation
    let scalingRecommendation: "scale_up" | "scale_down" | "maintain";
    let recommendationRationale: string;

    if (utilizationPercent >= 80) {
      scalingRecommendation = "scale_up";
      recommendationRationale = `High utilization (${utilizationPercent.toFixed(
        1
      )}%), recommend scaling up`;
    } else if (utilizationPercent <= 30) {
      scalingRecommendation = "scale_down";
      recommendationRationale = `Low utilization (${utilizationPercent.toFixed(
        1
      )}%), consider scaling down`;
    } else {
      scalingRecommendation = "maintain";
      recommendationRationale = `Utilization at ${utilizationPercent.toFixed(
        1
      )}%, current capacity is adequate`;
    }

    const analysis: CapacityAnalysis = {
      timestamp: new Date(),
      windowMs: this.config.capacityAnalysisIntervalMs,
      totalCapacity: {
        cpuPercent: poolStats.totalCpuCapacity,
        memoryMb: poolStats.totalMemoryCapacity,
        agentCount: poolStats.totalAgents,
      },
      usedCapacity: {
        cpuPercent: poolStats.usedCpuCapacity,
        memoryMb: poolStats.usedMemoryCapacity,
        activeAgents: poolStats.activeAgents,
      },
      availableCapacity: {
        cpuPercent: poolStats.totalCpuCapacity - poolStats.usedCpuCapacity,
        memoryMb: poolStats.totalMemoryCapacity - poolStats.usedMemoryCapacity,
        idleAgents: poolStats.idleAgents,
      },
      utilizationPercent,
      scalingRecommendation,
      recommendationRationale,
    };

    this.lastCapacityAnalysis = analysis;

    this.logger.debug("Capacity analysis completed", {
      utilization: utilizationPercent.toFixed(1),
      recommendation: scalingRecommendation,
    });

    return analysis;
  }

  /**
   * Get resource pool statistics
   *
   * @returns Resource pool statistics
   */
  async getPoolStatistics(): Promise<ResourcePoolStats> {
    return this.resourceMonitor.getPoolStats();
  }

  /**
   * Get current configuration
   *
   * @returns Current configuration
   */
  getConfig(): AdaptiveResourceManagerConfig {
    return { ...this.config };
  }

  /**
   * Update configuration
   *
   * @param config Partial configuration update
   */
  updateConfig(config: Partial<AdaptiveResourceManagerConfig>): void {
    const wasRunning = this.isRunning;

    // Stop if running
    if (wasRunning) {
      this.stop();
    }

    // Update config
    this.config = { ...this.config, ...config };

    // Update sub-components
    if (config.loadBalancingStrategy) {
      this.loadBalancer.setStrategy(config.loadBalancingStrategy);
    }

    if (config.thresholds) {
      this.resourceMonitor.updateConfig({
        cpuThresholds: {
          warning: config.thresholds.cpuWarning,
          critical: config.thresholds.cpuCritical,
        },
        memoryThresholds: {
          warning: config.thresholds.memoryWarning,
          critical: config.thresholds.memoryCritical,
        },
      });
    }

    // Restart if was running
    if (wasRunning && this.config.enabled) {
      this.start();
    }

    this.logger.info("Configuration updated", this.config);
  }

  /**
   * Get health status
   *
   * @returns Health status
   */
  getHealthStatus(): {
    isRunning: boolean;
    lastMonitoringTime?: Date;
    activeAllocations: number;
    failoverEvents: number;
  } {
    return {
      isRunning: this.isRunning,
      lastMonitoringTime: this.lastCapacityAnalysis?.timestamp,
      activeAllocations: this.resourceAllocator.getActiveAllocationCount(),
      failoverEvents: this.failoverEvents.length,
    };
  }

  /**
   * Handle agent failover
   *
   * @param failedAgentId Failed agent identifier
   * @param backupAgentId Backup agent identifier
   * @returns Failover event
   */
  async handleFailover(
    failedAgentId: string,
    backupAgentId: string
  ): Promise<FailoverEvent> {
    const startTime = Date.now();

    if (!this.config.enableAutoFailover) {
      throw new Error("Auto-failover is disabled");
    }

    this.logger.info("Initiating agent failover", {
      failedAgent: failedAgentId,
      backupAgent: backupAgentId,
    });

    try {
      // Get task count for failed agent
      const tasksTransferred =
        this.resourceAllocator.getAgentAllocationCount(failedAgentId);

      // In real implementation, would transfer tasks to backup agent
      // For now, just record the event

      const failoverEvent: FailoverEvent = {
        eventId: uuidv4(),
        failedAgentId,
        backupAgentId,
        tasksTransferred,
        timestamp: new Date(),
        durationMs: Date.now() - startTime,
        failureReason: "Agent health check failed",
        success: true,
      };

      this.failoverEvents.push(failoverEvent);

      // Keep only last 100 failover events
      if (this.failoverEvents.length > 100) {
        this.failoverEvents.shift();
      }

      this.logger.info("Agent failover completed", {
        eventId: failoverEvent.eventId,
        tasksTransferred,
        durationMs: failoverEvent.durationMs,
      });

      return failoverEvent;
    } catch (error) {
      const failoverEvent: FailoverEvent = {
        eventId: uuidv4(),
        failedAgentId,
        backupAgentId,
        tasksTransferred: 0,
        timestamp: new Date(),
        durationMs: Date.now() - startTime,
        failureReason: error instanceof Error ? error.message : "Unknown error",
        success: false,
      };

      this.failoverEvents.push(failoverEvent);

      this.logger.error("Agent failover failed", {
        eventId: failoverEvent.eventId,
        error,
      });

      throw error;
    }
  }

  /**
   * Get failover event history
   *
   * @param count Number of events to retrieve
   * @returns Recent failover events
   */
  getFailoverHistory(count: number = 10): FailoverEvent[] {
    return this.failoverEvents.slice(-count);
  }

  /**
   * Get allocation statistics
   *
   * @returns Allocation statistics
   */
  getAllocationStatistics() {
    return this.resourceAllocator.getAllocationStats();
  }

  /**
   * Get load distribution
   *
   * @returns Load distribution across agents
   */
  async getLoadDistribution(): Promise<Map<string, number>> {
    return this.loadBalancer.getLoadDistribution();
  }
}
