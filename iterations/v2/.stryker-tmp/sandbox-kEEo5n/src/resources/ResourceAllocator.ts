/**
 * @fileoverview Resource Allocator for Adaptive Resource Manager
 *
 * Manages resource allocation with priority queuing and rate limiting.
 * Ensures fair distribution and prevents resource exhaustion.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck
function stryNS_9fa48() {
  var g = typeof globalThis === 'object' && globalThis && globalThis.Math === Math && globalThis || new Function("return this")();
  var ns = g.__stryker__ || (g.__stryker__ = {});
  if (ns.activeMutant === undefined && g.process && g.process.env && g.process.env.__STRYKER_ACTIVE_MUTANT__) {
    ns.activeMutant = g.process.env.__STRYKER_ACTIVE_MUTANT__;
  }
  function retrieveNS() {
    return ns;
  }
  stryNS_9fa48 = retrieveNS;
  return retrieveNS();
}
stryNS_9fa48();
function stryCov_9fa48() {
  var ns = stryNS_9fa48();
  var cov = ns.mutantCoverage || (ns.mutantCoverage = {
    static: {},
    perTest: {}
  });
  function cover() {
    var c = cov.static;
    if (ns.currentTestId) {
      c = cov.perTest[ns.currentTestId] = cov.perTest[ns.currentTestId] || {};
    }
    var a = arguments;
    for (var i = 0; i < a.length; i++) {
      c[a[i]] = (c[a[i]] || 0) + 1;
    }
  }
  stryCov_9fa48 = cover;
  cover.apply(null, arguments);
}
function stryMutAct_9fa48(id) {
  var ns = stryNS_9fa48();
  function isActive(id) {
    if (ns.activeMutant === id) {
      if (ns.hitCount !== void 0 && ++ns.hitCount > ns.hitLimit) {
        throw new Error('Stryker: Hit count limit reached (' + ns.hitCount + ')');
      }
      return true;
    }
    return false;
  }
  stryMutAct_9fa48 = isActive;
  return isActive(id);
}
import { Logger } from "@/observability/Logger";
import { type IResourceAllocator, type RateLimitConfig, type ResourceAllocationRequest, type ResourceAllocationResult } from "@/types/resource-types";
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
  private allocationStats = stryMutAct_9fa48("298") ? {} : (stryCov_9fa48("298"), {
    totalRequests: 0,
    successfulAllocations: 0,
    failedAllocations: 0,
    totalAllocationTimeMs: 0
  });
  constructor(loadBalancer: LoadBalancer, rateLimitConfig?: Partial<RateLimitConfig>) {
    if (stryMutAct_9fa48("299")) {
      {}
    } else {
      stryCov_9fa48("299");
      this.logger = new Logger("ResourceAllocator");
      this.loadBalancer = loadBalancer;
      this.rateLimitConfig = stryMutAct_9fa48("301") ? {} : (stryCov_9fa48("301"), {
        maxRequests: 1000,
        windowMs: 60000,
        // 1 minute
        currentCount: 0,
        windowStart: new Date(),
        dynamicAdjustment: stryMutAct_9fa48("302") ? false : (stryCov_9fa48("302"), true),
        ...rateLimitConfig
      });
    }
  }

  /**
   * Allocate resources for a task
   *
   * @param request Resource allocation request
   * @returns Allocation result
   */
  async allocate(request: ResourceAllocationRequest): Promise<ResourceAllocationResult> {
    if (stryMutAct_9fa48("303")) {
      {}
    } else {
      stryCov_9fa48("303");
      const startTime = Date.now();
      stryMutAct_9fa48("304") ? this.allocationStats.totalRequests-- : (stryCov_9fa48("304"), this.allocationStats.totalRequests++);
      try {
        if (stryMutAct_9fa48("305")) {
          {}
        } else {
          stryCov_9fa48("305");
          // Check rate limit
          if (stryMutAct_9fa48("308") ? false : stryMutAct_9fa48("307") ? true : stryMutAct_9fa48("306") ? this.checkRateLimit() : (stryCov_9fa48("306", "307", "308"), !this.checkRateLimit())) {
            if (stryMutAct_9fa48("309")) {
              {}
            } else {
              stryCov_9fa48("309");
              return this.createFailureResult(request, "Rate limit exceeded", startTime);
            }
          }

          // Check timeout
          const timeoutTime = stryMutAct_9fa48("311") ? request.requestedAt.getTime() - request.timeoutMs : (stryCov_9fa48("311"), request.requestedAt.getTime() + request.timeoutMs);
          if (stryMutAct_9fa48("315") ? Date.now() <= timeoutTime : stryMutAct_9fa48("314") ? Date.now() >= timeoutTime : stryMutAct_9fa48("313") ? false : stryMutAct_9fa48("312") ? true : (stryCov_9fa48("312", "313", "314", "315"), Date.now() > timeoutTime)) {
            if (stryMutAct_9fa48("316")) {
              {}
            } else {
              stryCov_9fa48("316");
              return this.createFailureResult(request, "Request timeout", startTime);
            }
          }

          // Get available agents
          const availableAgents = await this.getAvailableAgents();
          if (stryMutAct_9fa48("320") ? availableAgents.length !== 0 : stryMutAct_9fa48("319") ? false : stryMutAct_9fa48("318") ? true : (stryCov_9fa48("318", "319", "320"), availableAgents.length === 0)) {
            if (stryMutAct_9fa48("321")) {
              {}
            } else {
              stryCov_9fa48("321");
              return this.createFailureResult(request, "No available agents", startTime);
            }
          }

          // Select agent using load balancer
          const decision = await this.loadBalancer.selectAgent(request, availableAgents);

          // Allocate resources
          const allocatedResources = stryMutAct_9fa48("323") ? {} : (stryCov_9fa48("323"), {
            cpuPercent: stryMutAct_9fa48("324") ? request.requiredResources.cpuPercent && 10 : (stryCov_9fa48("324"), request.requiredResources.cpuPercent ?? 10),
            memoryMb: stryMutAct_9fa48("325") ? request.requiredResources.memoryMb && 128 : (stryCov_9fa48("325"), request.requiredResources.memoryMb ?? 128),
            networkMbps: stryMutAct_9fa48("326") ? request.requiredResources.networkMbps && 10 : (stryCov_9fa48("326"), request.requiredResources.networkMbps ?? 10)
          });

          // Record allocation
          const allocationRecord: AllocationRecord = stryMutAct_9fa48("327") ? {} : (stryCov_9fa48("327"), {
            requestId: request.requestId,
            agentId: decision.selectedAgentId,
            allocatedAt: new Date(),
            resources: allocatedResources
          });
          this.activeAllocations.set(request.requestId, allocationRecord);

          // Update stats
          stryMutAct_9fa48("328") ? this.allocationStats.successfulAllocations-- : (stryCov_9fa48("328"), this.allocationStats.successfulAllocations++);
          const allocationTime = stryMutAct_9fa48("329") ? Date.now() + startTime : (stryCov_9fa48("329"), Date.now() - startTime);
          stryMutAct_9fa48("330") ? this.allocationStats.totalAllocationTimeMs -= allocationTime : (stryCov_9fa48("330"), this.allocationStats.totalAllocationTimeMs += allocationTime);

          // Increment rate limit counter
          stryMutAct_9fa48("331") ? this.rateLimitConfig.currentCount-- : (stryCov_9fa48("331"), this.rateLimitConfig.currentCount++);
          this.logger.debug("Resource allocation successful", stryMutAct_9fa48("333") ? {} : (stryCov_9fa48("333"), {
            requestId: request.requestId,
            taskId: request.taskId,
            agentId: decision.selectedAgentId,
            priority: request.priority,
            allocationTimeMs: allocationTime
          }));
          return stryMutAct_9fa48("334") ? {} : (stryCov_9fa48("334"), {
            requestId: request.requestId,
            success: stryMutAct_9fa48("335") ? false : (stryCov_9fa48("335"), true),
            assignedAgentId: decision.selectedAgentId,
            allocatedResources,
            allocatedAt: new Date(),
            waitTimeMs: stryMutAct_9fa48("336") ? Date.now() + request.requestedAt.getTime() : (stryCov_9fa48("336"), Date.now() - request.requestedAt.getTime())
          });
        }
      } catch (error) {
        if (stryMutAct_9fa48("337")) {
          {}
        } else {
          stryCov_9fa48("337");
          this.logger.error("Resource allocation failed", stryMutAct_9fa48("339") ? {} : (stryCov_9fa48("339"), {
            requestId: request.requestId,
            error
          }));
          return this.createFailureResult(request, error instanceof Error ? error.message : "Unknown error", startTime);
        }
      }
    }
  }

  /**
   * Release allocated resources
   *
   * @param requestId Request identifier
   */
  async release(requestId: string): Promise<void> {
    if (stryMutAct_9fa48("341")) {
      {}
    } else {
      stryCov_9fa48("341");
      const allocation = this.activeAllocations.get(requestId);
      if (stryMutAct_9fa48("344") ? false : stryMutAct_9fa48("343") ? true : stryMutAct_9fa48("342") ? allocation : (stryCov_9fa48("342", "343", "344"), !allocation)) {
        if (stryMutAct_9fa48("345")) {
          {}
        } else {
          stryCov_9fa48("345");
          this.logger.warn("Attempted to release unknown allocation", stryMutAct_9fa48("347") ? {} : (stryCov_9fa48("347"), {
            requestId
          }));
          return;
        }
      }
      this.activeAllocations.delete(requestId);
      this.logger.debug("Resources released", stryMutAct_9fa48("349") ? {} : (stryCov_9fa48("349"), {
        requestId,
        agentId: allocation.agentId,
        durationMs: stryMutAct_9fa48("350") ? Date.now() + allocation.allocatedAt.getTime() : (stryCov_9fa48("350"), Date.now() - allocation.allocatedAt.getTime())
      }));
    }
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
    if (stryMutAct_9fa48("351")) {
      {}
    } else {
      stryCov_9fa48("351");
      const avgAllocationTimeMs = (stryMutAct_9fa48("355") ? this.allocationStats.successfulAllocations <= 0 : stryMutAct_9fa48("354") ? this.allocationStats.successfulAllocations >= 0 : stryMutAct_9fa48("353") ? false : stryMutAct_9fa48("352") ? true : (stryCov_9fa48("352", "353", "354", "355"), this.allocationStats.successfulAllocations > 0)) ? stryMutAct_9fa48("356") ? this.allocationStats.totalAllocationTimeMs * this.allocationStats.successfulAllocations : (stryCov_9fa48("356"), this.allocationStats.totalAllocationTimeMs / this.allocationStats.successfulAllocations) : 0;
      return stryMutAct_9fa48("357") ? {} : (stryCov_9fa48("357"), {
        totalRequests: this.allocationStats.totalRequests,
        successfulAllocations: this.allocationStats.successfulAllocations,
        failedAllocations: this.allocationStats.failedAllocations,
        avgAllocationTimeMs
      });
    }
  }

  /**
   * Update rate limits
   *
   * @param config New rate limit configuration
   */
  updateRateLimits(config: RateLimitConfig): void {
    if (stryMutAct_9fa48("358")) {
      {}
    } else {
      stryCov_9fa48("358");
      this.rateLimitConfig = stryMutAct_9fa48("359") ? {} : (stryCov_9fa48("359"), {
        ...config
      });
      this.logger.info("Rate limits updated", config);
    }
  }

  /**
   * Get active allocations
   *
   * @returns Active allocation count
   */
  getActiveAllocationCount(): number {
    if (stryMutAct_9fa48("361")) {
      {}
    } else {
      stryCov_9fa48("361");
      return this.activeAllocations.size;
    }
  }

  /**
   * Get active allocations for an agent
   *
   * @param agentId Agent identifier
   * @returns Allocation count for agent
   */
  getAgentAllocationCount(agentId: string): number {
    if (stryMutAct_9fa48("362")) {
      {}
    } else {
      stryCov_9fa48("362");
      return stryMutAct_9fa48("363") ? Array.from(this.activeAllocations.values()).length : (stryCov_9fa48("363"), Array.from(this.activeAllocations.values()).filter(stryMutAct_9fa48("364") ? () => undefined : (stryCov_9fa48("364"), a => stryMutAct_9fa48("367") ? a.agentId !== agentId : stryMutAct_9fa48("366") ? false : stryMutAct_9fa48("365") ? true : (stryCov_9fa48("365", "366", "367"), a.agentId === agentId))).length);
    }
  }

  /**
   * Reset statistics
   */
  resetStats(): void {
    if (stryMutAct_9fa48("368")) {
      {}
    } else {
      stryCov_9fa48("368");
      this.allocationStats = stryMutAct_9fa48("369") ? {} : (stryCov_9fa48("369"), {
        totalRequests: 0,
        successfulAllocations: 0,
        failedAllocations: 0,
        totalAllocationTimeMs: 0
      });
      this.logger.info("Allocation statistics reset");
    }
  }

  /**
   * Check rate limit
   *
   * @returns True if within rate limit
   */
  private checkRateLimit(): boolean {
    if (stryMutAct_9fa48("371")) {
      {}
    } else {
      stryCov_9fa48("371");
      const now = new Date();
      const windowElapsed = stryMutAct_9fa48("372") ? now.getTime() + this.rateLimitConfig.windowStart.getTime() : (stryCov_9fa48("372"), now.getTime() - this.rateLimitConfig.windowStart.getTime());

      // Reset window if expired
      if (stryMutAct_9fa48("376") ? windowElapsed < this.rateLimitConfig.windowMs : stryMutAct_9fa48("375") ? windowElapsed > this.rateLimitConfig.windowMs : stryMutAct_9fa48("374") ? false : stryMutAct_9fa48("373") ? true : (stryCov_9fa48("373", "374", "375", "376"), windowElapsed >= this.rateLimitConfig.windowMs)) {
        if (stryMutAct_9fa48("377")) {
          {}
        } else {
          stryCov_9fa48("377");
          this.rateLimitConfig.currentCount = 0;
          this.rateLimitConfig.windowStart = now;
          return stryMutAct_9fa48("378") ? false : (stryCov_9fa48("378"), true);
        }
      }

      // Check if within limit
      return stryMutAct_9fa48("382") ? this.rateLimitConfig.currentCount >= this.rateLimitConfig.maxRequests : stryMutAct_9fa48("381") ? this.rateLimitConfig.currentCount <= this.rateLimitConfig.maxRequests : stryMutAct_9fa48("380") ? false : stryMutAct_9fa48("379") ? true : (stryCov_9fa48("379", "380", "381", "382"), this.rateLimitConfig.currentCount < this.rateLimitConfig.maxRequests);
    }
  }

  /**
   * Get available agents
   * In a real implementation, this would query the agent registry
   *
   * @returns List of available agent IDs
   */
  private async getAvailableAgents(): Promise<string[]> {
    if (stryMutAct_9fa48("383")) {
      {}
    } else {
      stryCov_9fa48("383");
      // Placeholder: return mock agents
      // In real implementation, query agent registry for healthy agents
      return ["agent-1", "agent-2", "agent-3"];
    }
  }

  /**
   * Create failure result
   */
  private createFailureResult(request: ResourceAllocationRequest, reason: string, startTime: number): ResourceAllocationResult {
    if (stryMutAct_9fa48("388")) {
      {}
    } else {
      stryCov_9fa48("388");
      stryMutAct_9fa48("389") ? this.allocationStats.failedAllocations-- : (stryCov_9fa48("389"), this.allocationStats.failedAllocations++);
      return stryMutAct_9fa48("390") ? {} : (stryCov_9fa48("390"), {
        requestId: request.requestId,
        success: stryMutAct_9fa48("391") ? true : (stryCov_9fa48("391"), false),
        failureReason: reason,
        waitTimeMs: stryMutAct_9fa48("392") ? Date.now() + startTime : (stryCov_9fa48("392"), Date.now() - startTime)
      });
    }
  }
}