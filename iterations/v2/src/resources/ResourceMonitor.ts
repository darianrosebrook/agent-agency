/**
 * @fileoverview Resource Monitor for Adaptive Resource Manager
 *
 * Tracks resource usage (CPU, memory, network) for agent pools.
 * Provides real-time visibility into system resource consumption.
 *
 * @author @darianrosebrook
 */

import { Logger } from "@/observability/Logger";
import {
  ResourceType,
  type AgentResourceProfile,
  type IResourceMonitor,
  type ResourcePoolStats,
  type ResourceUsage,
} from "@/types/resource-types";

/**
 * Resource Monitor Configuration
 */
export interface ResourceMonitorConfig {
  /** Monitoring interval (ms) */
  intervalMs: number;

  /** Enable automatic health status updates */
  enableHealthTracking: boolean;

  /** CPU usage thresholds (%) */
  cpuThresholds: {
    warning: number;
    critical: number;
  };

  /** Memory usage thresholds (%) */
  memoryThresholds: {
    warning: number;
    critical: number;
  };

  /** Maximum task capacity per agent */
  defaultMaxTaskCapacity: number;
}

/**
 * Default configuration
 */
const DEFAULT_CONFIG: ResourceMonitorConfig = {
  intervalMs: 5000, // 5 seconds
  enableHealthTracking: true,
  cpuThresholds: {
    warning: 70,
    critical: 85,
  },
  memoryThresholds: {
    warning: 75,
    critical: 90,
  },
  defaultMaxTaskCapacity: 10,
};

/**
 * Resource Monitor
 *
 * Monitors resource usage across agent pools:
 * - Real-time resource tracking
 * - Agent health status computation
 * - Resource pool statistics
 * - Configurable thresholds
 */
export class ResourceMonitor implements IResourceMonitor {
  private logger: Logger;
  private config: ResourceMonitorConfig;
  private agentProfiles: Map<string, AgentResourceProfile> = new Map();
  private isRunning = false;
  private monitoringTimer?: ReturnType<typeof setInterval>;

  constructor(config: Partial<ResourceMonitorConfig> = {}) {
    this.logger = new Logger("ResourceMonitor");
    this.config = { ...DEFAULT_CONFIG, ...config };
  }

  /**
   * Start resource monitoring
   */
  async start(): Promise<void> {
    if (this.isRunning) {
      this.logger.warn("Resource monitor already running");
      return;
    }

    this.isRunning = true;

    // Start periodic health updates
    if (this.config.enableHealthTracking) {
      this.monitoringTimer = setInterval(async () => {
        await this.updateAgentHealth();
      }, this.config.intervalMs);
    }

    this.logger.info("Resource monitor started", {
      intervalMs: this.config.intervalMs,
      healthTracking: this.config.enableHealthTracking,
    });
  }

  /**
   * Stop resource monitoring
   */
  async stop(): Promise<void> {
    if (!this.isRunning) {
      return;
    }

    if (this.monitoringTimer) {
      clearInterval(this.monitoringTimer);
      this.monitoringTimer = undefined;
    }

    this.isRunning = false;
    this.logger.info("Resource monitor stopped");
  }

  /**
   * Get current resource usage for an agent
   *
   * @param agentId Agent identifier
   * @returns Agent resource profile or null if not found
   */
  async getAgentResources(
    agentId: string
  ): Promise<AgentResourceProfile | null> {
    return this.agentProfiles.get(agentId) ?? null;
  }

  /**
   * Get all agent resource profiles
   *
   * @returns All agent resource profiles
   */
  async getAllAgentResources(): Promise<AgentResourceProfile[]> {
    return Array.from(this.agentProfiles.values());
  }

  /**
   * Record resource usage for an agent
   *
   * @param agentId Agent identifier
   * @param usage Resource usage data
   */
  async recordUsage(agentId: string, usage: ResourceUsage): Promise<void> {
    let profile = this.agentProfiles.get(agentId);

    if (!profile) {
      // Create new profile
      profile = this.createInitialProfile(agentId);
      this.agentProfiles.set(agentId, profile);
    }

    // Update resource usage
    switch (usage.type) {
      case ResourceType.CPU:
        profile.cpuUsage = usage;
        break;
      case ResourceType.MEMORY:
        profile.memoryUsage = usage;
        break;
      case ResourceType.NETWORK:
        profile.networkUsage = usage;
        break;
    }

    profile.lastUpdated = new Date();

    // Update health status
    if (this.config.enableHealthTracking) {
      profile.healthStatus = this.computeHealthStatus(profile);
    }

    this.logger.debug("Recorded resource usage", {
      agentId,
      resourceType: usage.type,
      usagePercent: usage.usagePercent,
      healthStatus: profile.healthStatus,
    });
  }

  /**
   * Get resource pool statistics
   *
   * @returns Resource pool statistics
   */
  async getPoolStats(): Promise<ResourcePoolStats> {
    const allProfiles = Array.from(this.agentProfiles.values());

    const activeAgents = allProfiles.filter(
      (p) => p.currentTaskCount > 0
    ).length;
    const idleAgents = allProfiles.filter(
      (p) => p.currentTaskCount === 0
    ).length;
    const unhealthyAgents = allProfiles.filter(
      (p) => p.healthStatus === "unhealthy"
    ).length;

    const totalCpu = allProfiles.reduce(
      (sum, p) => sum + p.cpuUsage.maximum,
      0
    );
    const usedCpu = allProfiles.reduce((sum, p) => sum + p.cpuUsage.current, 0);

    const totalMemory = allProfiles.reduce(
      (sum, p) => sum + p.memoryUsage.maximum,
      0
    );
    const usedMemory = allProfiles.reduce(
      (sum, p) => sum + p.memoryUsage.current,
      0
    );

    const totalTasks = allProfiles.reduce(
      (sum, p) => sum + p.currentTaskCount,
      0
    );

    const completionTimes = allProfiles
      .map((p) => p.avgTaskCompletionMs)
      .filter((t) => t !== undefined) as number[];
    const avgCompletion =
      completionTimes.length > 0
        ? completionTimes.reduce((sum, t) => sum + t, 0) /
          completionTimes.length
        : 0;

    return {
      totalAgents: allProfiles.length,
      activeAgents,
      idleAgents,
      unhealthyAgents,
      totalCpuCapacity: totalCpu,
      usedCpuCapacity: usedCpu,
      totalMemoryCapacity: totalMemory,
      usedMemoryCapacity: usedMemory,
      tasksInProgress: totalTasks,
      avgTaskCompletionMs: avgCompletion,
      lastUpdated: new Date(),
    };
  }

  /**
   * Update task count for an agent
   *
   * @param agentId Agent identifier
   * @param taskCount Current task count
   */
  async updateTaskCount(agentId: string, taskCount: number): Promise<void> {
    const profile = this.agentProfiles.get(agentId);
    if (profile) {
      profile.currentTaskCount = taskCount;
      profile.lastUpdated = new Date();
    }
  }

  /**
   * Update average task completion time for an agent
   *
   * @param agentId Agent identifier
   * @param completionMs Average completion time (ms)
   */
  async updateTaskCompletionTime(
    agentId: string,
    completionMs: number
  ): Promise<void> {
    const profile = this.agentProfiles.get(agentId);
    if (profile) {
      profile.avgTaskCompletionMs = completionMs;
      profile.lastUpdated = new Date();
    }
  }

  /**
   * Remove agent from monitoring
   *
   * @param agentId Agent identifier
   */
  async removeAgent(agentId: string): Promise<void> {
    this.agentProfiles.delete(agentId);
    this.logger.info("Agent removed from monitoring", { agentId });
  }

  /**
   * Get configuration
   */
  getConfig(): ResourceMonitorConfig {
    return { ...this.config };
  }

  /**
   * Update configuration
   */
  updateConfig(config: Partial<ResourceMonitorConfig>): void {
    this.config = { ...this.config, ...config };
    this.logger.info("Configuration updated", this.config);
  }

  /**
   * Create initial profile for an agent
   */
  private createInitialProfile(agentId: string): AgentResourceProfile {
    return {
      agentId,
      cpuUsage: {
        type: ResourceType.CPU,
        current: 0,
        maximum: 100,
        usagePercent: 0,
        unit: "%",
        timestamp: new Date(),
        source: agentId,
      },
      memoryUsage: {
        type: ResourceType.MEMORY,
        current: 0,
        maximum: 1024,
        usagePercent: 0,
        unit: "MB",
        timestamp: new Date(),
        source: agentId,
      },
      networkUsage: {
        type: ResourceType.NETWORK,
        current: 0,
        maximum: 1000,
        usagePercent: 0,
        unit: "Mbps",
        timestamp: new Date(),
        source: agentId,
      },
      currentTaskCount: 0,
      maxTaskCapacity: this.config.defaultMaxTaskCapacity,
      healthStatus: "healthy",
      lastUpdated: new Date(),
    };
  }

  /**
   * Compute health status for an agent
   */
  private computeHealthStatus(
    profile: AgentResourceProfile
  ): "healthy" | "degraded" | "unhealthy" {
    const cpuPercent = profile.cpuUsage.usagePercent;
    const memoryPercent = profile.memoryUsage.usagePercent;

    // Check critical thresholds
    if (
      cpuPercent >= this.config.cpuThresholds.critical ||
      memoryPercent >= this.config.memoryThresholds.critical
    ) {
      return "unhealthy";
    }

    // Check warning thresholds
    if (
      cpuPercent >= this.config.cpuThresholds.warning ||
      memoryPercent >= this.config.memoryThresholds.warning
    ) {
      return "degraded";
    }

    return "healthy";
  }

  /**
   * Update health status for all agents
   */
  private async updateAgentHealth(): Promise<void> {
    for (const profile of this.agentProfiles.values()) {
      profile.healthStatus = this.computeHealthStatus(profile);
    }
  }
}
