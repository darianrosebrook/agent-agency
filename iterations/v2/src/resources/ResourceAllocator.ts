/**
 * @fileoverview Resource Allocator for Adaptive Resource Manager
 *
 * Manages resource allocation with priority queuing and rate limiting.
 * Ensures fair distribution and prevents resource exhaustion.
 *
 * @author @darianrosebrook
 */

import { Logger } from "@/observability/Logger";
import type { AgentRegistry } from "@/types/agent-registry";
import {
  type IResourceAllocator,
  type RateLimitConfig,
  type ResourceAllocationRequest,
  type ResourceAllocationResult,
} from "@/types/resource-types";
import { LoadBalancer } from "./LoadBalancer";

/**
 * Allocation record for tracking
 */
interface AllocationRecord {
  requestId: string;
  agentId: string;
  allocatedAt: Date;
  resources: {
    cpuPercent: number;
    memoryMb: number;
    networkMbps: number;
  };
}

/**
 * Resource Allocator
 *
 * Manages resource allocation:
 * - Priority-based allocation
 * - Rate limiting
 * - Resource tracking
 * - Fast allocation decisions
 */
export class ResourceAllocator implements IResourceAllocator {
  private logger: Logger;
  private loadBalancer: LoadBalancer;
  private agentRegistry: AgentRegistry;
  private activeAllocations: Map<string, AllocationRecord> = new Map();
  private rateLimitConfig: RateLimitConfig;
  private allocationStats = {
    totalRequests: 0,
    successfulAllocations: 0,
    failedAllocations: 0,
    totalAllocationTimeMs: 0,
  };

  constructor(
    loadBalancer: LoadBalancer,
    agentRegistry: AgentRegistry,
    rateLimitConfig?: Partial<RateLimitConfig>
  ) {
    this.logger = new Logger("ResourceAllocator");
    this.loadBalancer = loadBalancer;
    this.agentRegistry = agentRegistry;
    this.rateLimitConfig = {
      maxRequests: 1000,
      windowMs: 60000, // 1 minute
      currentCount: 0,
      windowStart: new Date(),
      dynamicAdjustment: true,
      ...rateLimitConfig,
    };
  }

  /**
   * Allocate resources for a task
   *
   * @param request Resource allocation request
   * @returns Allocation result
   */
  async allocate(
    request: ResourceAllocationRequest
  ): Promise<ResourceAllocationResult> {
    const startTime = Date.now();
    this.allocationStats.totalRequests++;

    try {
      // Check rate limit
      if (!this.checkRateLimit()) {
        return this.createFailureResult(
          request,
          "Rate limit exceeded",
          startTime
        );
      }

      // Check timeout
      const timeoutTime = request.requestedAt.getTime() + request.timeoutMs;
      if (Date.now() > timeoutTime) {
        return this.createFailureResult(request, "Request timeout", startTime);
      }

      // Get available agents
      const availableAgents = await this.getAvailableAgents();
      if (availableAgents.length === 0) {
        return this.createFailureResult(
          request,
          "No available agents",
          startTime
        );
      }

      // Select agent using load balancer
      const decision = await this.loadBalancer.selectAgent(
        request,
        availableAgents
      );

      // Allocate resources
      const allocatedResources = {
        cpuPercent: request.requiredResources.cpuPercent ?? 10,
        memoryMb: request.requiredResources.memoryMb ?? 128,
        networkMbps: request.requiredResources.networkMbps ?? 10,
      };

      // Record allocation
      const allocationRecord: AllocationRecord = {
        requestId: request.requestId,
        agentId: decision.selectedAgentId,
        allocatedAt: new Date(),
        resources: allocatedResources,
      };

      this.activeAllocations.set(request.requestId, allocationRecord);

      // Update stats
      this.allocationStats.successfulAllocations++;
      const allocationTime = Date.now() - startTime;
      this.allocationStats.totalAllocationTimeMs += allocationTime;

      // Increment rate limit counter
      this.rateLimitConfig.currentCount++;

      this.logger.debug("Resource allocation successful", {
        requestId: request.requestId,
        taskId: request.taskId,
        agentId: decision.selectedAgentId,
        priority: request.priority,
        allocationTimeMs: allocationTime,
      });

      return {
        requestId: request.requestId,
        success: true,
        assignedAgentId: decision.selectedAgentId,
        allocatedResources,
        allocatedAt: new Date(),
        waitTimeMs: Date.now() - request.requestedAt.getTime(),
      };
    } catch (error) {
      this.logger.error("Resource allocation failed", {
        requestId: request.requestId,
        error,
      });

      return this.createFailureResult(
        request,
        error instanceof Error ? error.message : "Unknown error",
        startTime
      );
    }
  }

  /**
   * Release allocated resources
   *
   * @param requestId Request identifier
   */
  async release(requestId: string): Promise<void> {
    const allocation = this.activeAllocations.get(requestId);

    if (!allocation) {
      this.logger.warn("Attempted to release unknown allocation", {
        requestId,
      });
      return;
    }

    this.activeAllocations.delete(requestId);

    this.logger.debug("Resources released", {
      requestId,
      agentId: allocation.agentId,
      durationMs: Date.now() - allocation.allocatedAt.getTime(),
    });
  }

  /**
   * Get allocation statistics
   *
   * @returns Allocation statistics
   */
  getAllocationStats(): {
    totalRequests: number;
    successfulAllocations: number;
    failedAllocations: number;
    avgAllocationTimeMs: number;
  } {
    const avgAllocationTimeMs =
      this.allocationStats.successfulAllocations > 0
        ? this.allocationStats.totalAllocationTimeMs /
          this.allocationStats.successfulAllocations
        : 0;

    return {
      totalRequests: this.allocationStats.totalRequests,
      successfulAllocations: this.allocationStats.successfulAllocations,
      failedAllocations: this.allocationStats.failedAllocations,
      avgAllocationTimeMs,
    };
  }

  /**
   * Update rate limits
   *
   * @param config New rate limit configuration
   */
  updateRateLimits(config: RateLimitConfig): void {
    this.rateLimitConfig = { ...config };
    this.logger.info("Rate limits updated", config);
  }

  /**
   * Get active allocations
   *
   * @returns Active allocation count
   */
  getActiveAllocationCount(): number {
    return this.activeAllocations.size;
  }

  /**
   * Get active allocations for an agent
   *
   * @param agentId Agent identifier
   * @returns Allocation count for agent
   */
  getAgentAllocationCount(agentId: string): number {
    return Array.from(this.activeAllocations.values()).filter(
      (a) => a.agentId === agentId
    ).length;
  }

  /**
   * Reset statistics
   */
  resetStats(): void {
    this.allocationStats = {
      totalRequests: 0,
      successfulAllocations: 0,
      failedAllocations: 0,
      totalAllocationTimeMs: 0,
    };
    this.logger.info("Allocation statistics reset");
  }

  /**
   * Check rate limit
   *
   * @returns True if within rate limit
   */
  private checkRateLimit(): boolean {
    const now = new Date();
    const windowElapsed =
      now.getTime() - this.rateLimitConfig.windowStart.getTime();

    // Reset window if expired
    if (windowElapsed >= this.rateLimitConfig.windowMs) {
      this.rateLimitConfig.currentCount = 0;
      this.rateLimitConfig.windowStart = now;
      return true;
    }

    // Check if within limit
    return this.rateLimitConfig.currentCount < this.rateLimitConfig.maxRequests;
  }

  /**
   * Get available agents
   * Queries the agent registry for agents that can handle general tasks
   *
   * @returns List of available agent IDs
   */
  private async getAvailableAgents(): Promise<string[]> {
    try {
      // Query for agents with basic capabilities (no specific requirements)
      const basicQuery = {
        taskType: "file_editing" as const,
        requiredCapabilities: [],
        minExpertiseLevel: "novice" as const,
        maxResults: 10,
      };

      const results = await this.agentRegistry.getAgentsByCapability(
        basicQuery
      );

      // Return agent IDs from the results
      return results.map((result) => result.agent.id);
    } catch (error) {
      this.logger.warn("Failed to query agent registry for available agents", {
        error,
      });

      // Fallback: try to get registry stats to see if there are any agents
      try {
        const stats = await this.agentRegistry.getStats();
        if (stats.totalAgents > 0) {
          // If we have agents but query failed, return a generic list
          // This is still better than hardcoded mock data
          this.logger.info(
            `Registry has ${stats.totalAgents} agents but query failed`
          );
        }
      } catch (statsError) {
        this.logger.warn("Failed to get registry stats", { statsError });
      }

      // Return empty array as last resort - better than mock data
      return [];
    }
  }

  /**
   * Create failure result
   */
  private createFailureResult(
    request: ResourceAllocationRequest,
    reason: string,
    startTime: number
  ): ResourceAllocationResult {
    this.allocationStats.failedAllocations++;
    return {
      requestId: request.requestId,
      success: false,
      failureReason: reason,
      waitTimeMs: Date.now() - startTime,
    };
  }
}
