/**
 * @fileoverview Resource Allocator for Adaptive Resource Manager
 *
 * Manages resource allocation with priority queuing and rate limiting.
 * Ensures fair distribution and prevents resource exhaustion.
 *
 * @author @darianrosebrook
 */

import { Logger } from "@/observability/Logger";
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
    rateLimitConfig?: Partial<RateLimitConfig>
  ) {
    this.logger = new Logger("ResourceAllocator");
    this.loadBalancer = loadBalancer;
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
   * In a real implementation, this would query the agent registry
   *
   * @returns List of available agent IDs
   */
  private async getAvailableAgents(): Promise<string[]> {
    // Placeholder: return mock agents
    // In real implementation, query agent registry for healthy agents
    return ["agent-1", "agent-2", "agent-3"];
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
