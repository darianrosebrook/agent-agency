/**
 * @fileoverview Adaptive Resource Manager - Main Resource Management Component
 *
 * Coordinates resource monitoring, load balancing, and allocation.
 * Provides dynamic resource management with failover capabilities.
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
import { LoadBalancingStrategy, type AdaptiveResourceManagerConfig, type CapacityAnalysis, type FailoverEvent, type IAdaptiveResourceManager, type ResourceAllocationRequest, type ResourceAllocationResult, type ResourcePoolStats } from "@/types/resource-types";
import { v4 as uuidv4 } from "uuid";
import { LoadBalancer } from "./LoadBalancer";
import { ResourceAllocator } from "./ResourceAllocator";
import { ResourceMonitor } from "./ResourceMonitor";

/**
 * Default configuration
 */
const DEFAULT_CONFIG: AdaptiveResourceManagerConfig = stryMutAct_9fa48("0") ? {} : (stryCov_9fa48("0"), {
  enabled: stryMutAct_9fa48("1") ? false : (stryCov_9fa48("1"), true),
  monitoringIntervalMs: 5000,
  loadBalancingStrategy: LoadBalancingStrategy.LEAST_LOADED,
  enableDynamicRateLimiting: stryMutAct_9fa48("2") ? false : (stryCov_9fa48("2"), true),
  enableAutoFailover: stryMutAct_9fa48("3") ? false : (stryCov_9fa48("3"), true),
  thresholds: stryMutAct_9fa48("4") ? {} : (stryCov_9fa48("4"), {
    cpuWarning: 70,
    cpuCritical: 85,
    memoryWarning: 75,
    memoryCritical: 90
  }),
  maxAllocationDecisionMs: 50,
  enableCapacityPlanning: stryMutAct_9fa48("5") ? false : (stryCov_9fa48("5"), true),
  capacityAnalysisIntervalMs: 300000 // 5 minutes
});

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
  private isRunning = stryMutAct_9fa48("6") ? true : (stryCov_9fa48("6"), false);
  private capacityAnalysisTimer?: ReturnType<typeof setInterval>;
  private lastCapacityAnalysis?: CapacityAnalysis;
  private failoverEvents: FailoverEvent[] = [];
  constructor(config: Partial<AdaptiveResourceManagerConfig> = {}) {
    if (stryMutAct_9fa48("8")) {
      {}
    } else {
      stryCov_9fa48("8");
      this.logger = new Logger("AdaptiveResourceManager");
      this.config = stryMutAct_9fa48("10") ? {} : (stryCov_9fa48("10"), {
        ...DEFAULT_CONFIG,
        ...config
      });

      // Initialize sub-components
      this.resourceMonitor = new ResourceMonitor(stryMutAct_9fa48("11") ? {} : (stryCov_9fa48("11"), {
        intervalMs: this.config.monitoringIntervalMs,
        cpuThresholds: stryMutAct_9fa48("12") ? {} : (stryCov_9fa48("12"), {
          warning: this.config.thresholds.cpuWarning,
          critical: this.config.thresholds.cpuCritical
        }),
        memoryThresholds: stryMutAct_9fa48("13") ? {} : (stryCov_9fa48("13"), {
          warning: this.config.thresholds.memoryWarning,
          critical: this.config.thresholds.memoryCritical
        })
      }));
      this.loadBalancer = new LoadBalancer(this.resourceMonitor, this.config.loadBalancingStrategy);
      this.resourceAllocator = new ResourceAllocator(this.loadBalancer, stryMutAct_9fa48("14") ? {} : (stryCov_9fa48("14"), {
        dynamicAdjustment: this.config.enableDynamicRateLimiting
      }));
    }
  }

  /**
   * Initialize the resource manager
   */
  async initialize(): Promise<void> {
    if (stryMutAct_9fa48("15")) {
      {}
    } else {
      stryCov_9fa48("15");
      // Stop monitor first to prevent multiple timers
      await this.resourceMonitor.stop();
      await this.resourceMonitor.start();
      this.logger.info("Adaptive resource manager initialized", stryMutAct_9fa48("17") ? {} : (stryCov_9fa48("17"), {
        enabled: this.config.enabled,
        strategy: this.config.loadBalancingStrategy,
        enableFailover: this.config.enableAutoFailover,
        enableCapacityPlanning: this.config.enableCapacityPlanning
      }));
    }
  }

  /**
   * Start resource management
   */
  async start(): Promise<void> {
    if (stryMutAct_9fa48("18")) {
      {}
    } else {
      stryCov_9fa48("18");
      if (stryMutAct_9fa48("20") ? false : stryMutAct_9fa48("19") ? true : (stryCov_9fa48("19", "20"), this.isRunning)) {
        if (stryMutAct_9fa48("21")) {
          {}
        } else {
          stryCov_9fa48("21");
          this.logger.warn("Resource manager already running");
          return;
        }
      }
      if (stryMutAct_9fa48("25") ? false : stryMutAct_9fa48("24") ? true : stryMutAct_9fa48("23") ? this.config.enabled : (stryCov_9fa48("23", "24", "25"), !this.config.enabled)) {
        if (stryMutAct_9fa48("26")) {
          {}
        } else {
          stryCov_9fa48("26");
          this.logger.info("Resource manager disabled, not starting");
          return;
        }
      }
      this.isRunning = stryMutAct_9fa48("28") ? false : (stryCov_9fa48("28"), true);

      // Start capacity analysis if enabled
      if (stryMutAct_9fa48("30") ? false : stryMutAct_9fa48("29") ? true : (stryCov_9fa48("29", "30"), this.config.enableCapacityPlanning)) {
        if (stryMutAct_9fa48("31")) {
          {}
        } else {
          stryCov_9fa48("31");
          this.capacityAnalysisTimer = setInterval(async () => {
            if (stryMutAct_9fa48("32")) {
              {}
            } else {
              stryCov_9fa48("32");
              try {
                if (stryMutAct_9fa48("33")) {
                  {}
                } else {
                  stryCov_9fa48("33");
                  await this.analyzeCapacity();
                }
              } catch (error) {
                if (stryMutAct_9fa48("34")) {
                  {}
                } else {
                  stryCov_9fa48("34");
                  this.logger.error("Capacity analysis failed", stryMutAct_9fa48("36") ? {} : (stryCov_9fa48("36"), {
                    error
                  }));
                }
              }
            }
          }, this.config.capacityAnalysisIntervalMs);
        }
      }
      this.logger.info("Adaptive resource manager started");
    }
  }

  /**
   * Stop resource management
   */
  async stop(): Promise<void> {
    if (stryMutAct_9fa48("38")) {
      {}
    } else {
      stryCov_9fa48("38");
      if (stryMutAct_9fa48("41") ? false : stryMutAct_9fa48("40") ? true : stryMutAct_9fa48("39") ? this.isRunning : (stryCov_9fa48("39", "40", "41"), !this.isRunning)) {
        if (stryMutAct_9fa48("42")) {
          {}
        } else {
          stryCov_9fa48("42");
          return;
        }
      }
      if (stryMutAct_9fa48("44") ? false : stryMutAct_9fa48("43") ? true : (stryCov_9fa48("43", "44"), this.capacityAnalysisTimer)) {
        if (stryMutAct_9fa48("45")) {
          {}
        } else {
          stryCov_9fa48("45");
          clearInterval(this.capacityAnalysisTimer);
          this.capacityAnalysisTimer = undefined;
        }
      }
      await this.resourceMonitor.stop();
      this.isRunning = stryMutAct_9fa48("46") ? true : (stryCov_9fa48("46"), false);
      this.logger.info("Adaptive resource manager stopped");
    }
  }

  /**
   * Allocate resources for a task
   *
   * @param request Resource allocation request
   * @returns Allocation result
   */
  async allocateResources(request: ResourceAllocationRequest): Promise<ResourceAllocationResult> {
    if (stryMutAct_9fa48("48")) {
      {}
    } else {
      stryCov_9fa48("48");
      const startTime = Date.now();
      try {
        if (stryMutAct_9fa48("49")) {
          {}
        } else {
          stryCov_9fa48("49");
          const result = await this.resourceAllocator.allocate(request);

          // Check if allocation took too long
          const allocationTime = stryMutAct_9fa48("50") ? Date.now() + startTime : (stryCov_9fa48("50"), Date.now() - startTime);
          if (stryMutAct_9fa48("54") ? allocationTime <= this.config.maxAllocationDecisionMs : stryMutAct_9fa48("53") ? allocationTime >= this.config.maxAllocationDecisionMs : stryMutAct_9fa48("52") ? false : stryMutAct_9fa48("51") ? true : (stryCov_9fa48("51", "52", "53", "54"), allocationTime > this.config.maxAllocationDecisionMs)) {
            if (stryMutAct_9fa48("55")) {
              {}
            } else {
              stryCov_9fa48("55");
              this.logger.warn("Allocation decision exceeded max time", stryMutAct_9fa48("57") ? {} : (stryCov_9fa48("57"), {
                allocationTime,
                maxAllocationTime: this.config.maxAllocationDecisionMs
              }));
            }
          }
          return result;
        }
      } catch (error) {
        if (stryMutAct_9fa48("58")) {
          {}
        } else {
          stryCov_9fa48("58");
          this.logger.error("Resource allocation failed", stryMutAct_9fa48("60") ? {} : (stryCov_9fa48("60"), {
            requestId: request.requestId,
            error
          }));
          throw error;
        }
      }
    }
  }

  /**
   * Release resources for a completed task
   *
   * @param requestId Request identifier
   */
  async releaseResources(requestId: string): Promise<void> {
    if (stryMutAct_9fa48("61")) {
      {}
    } else {
      stryCov_9fa48("61");
      await this.resourceAllocator.release(requestId);
    }
  }

  /**
   * Perform capacity analysis
   *
   * @returns Capacity analysis result
   */
  async analyzeCapacity(): Promise<CapacityAnalysis> {
    if (stryMutAct_9fa48("62")) {
      {}
    } else {
      stryCov_9fa48("62");
      const poolStats = await this.resourceMonitor.getPoolStats();

      // Calculate utilization
      const cpuUtilization = (stryMutAct_9fa48("66") ? poolStats.totalCpuCapacity <= 0 : stryMutAct_9fa48("65") ? poolStats.totalCpuCapacity >= 0 : stryMutAct_9fa48("64") ? false : stryMutAct_9fa48("63") ? true : (stryCov_9fa48("63", "64", "65", "66"), poolStats.totalCpuCapacity > 0)) ? stryMutAct_9fa48("67") ? poolStats.usedCpuCapacity / poolStats.totalCpuCapacity / 100 : (stryCov_9fa48("67"), (stryMutAct_9fa48("68") ? poolStats.usedCpuCapacity * poolStats.totalCpuCapacity : (stryCov_9fa48("68"), poolStats.usedCpuCapacity / poolStats.totalCpuCapacity)) * 100) : 0;
      const memoryUtilization = (stryMutAct_9fa48("72") ? poolStats.totalMemoryCapacity <= 0 : stryMutAct_9fa48("71") ? poolStats.totalMemoryCapacity >= 0 : stryMutAct_9fa48("70") ? false : stryMutAct_9fa48("69") ? true : (stryCov_9fa48("69", "70", "71", "72"), poolStats.totalMemoryCapacity > 0)) ? stryMutAct_9fa48("73") ? poolStats.usedMemoryCapacity / poolStats.totalMemoryCapacity / 100 : (stryCov_9fa48("73"), (stryMutAct_9fa48("74") ? poolStats.usedMemoryCapacity * poolStats.totalMemoryCapacity : (stryCov_9fa48("74"), poolStats.usedMemoryCapacity / poolStats.totalMemoryCapacity)) * 100) : 0;
      const utilizationPercent = stryMutAct_9fa48("75") ? (cpuUtilization + memoryUtilization) * 2 : (stryCov_9fa48("75"), (stryMutAct_9fa48("76") ? cpuUtilization - memoryUtilization : (stryCov_9fa48("76"), cpuUtilization + memoryUtilization)) / 2);

      // Determine scaling recommendation
      let scalingRecommendation: "scale_up" | "scale_down" | "maintain";
      let recommendationRationale: string;
      if (stryMutAct_9fa48("80") ? utilizationPercent < 80 : stryMutAct_9fa48("79") ? utilizationPercent > 80 : stryMutAct_9fa48("78") ? false : stryMutAct_9fa48("77") ? true : (stryCov_9fa48("77", "78", "79", "80"), utilizationPercent >= 80)) {
        if (stryMutAct_9fa48("81")) {
          {}
        } else {
          stryCov_9fa48("81");
          scalingRecommendation = "scale_up";
          recommendationRationale = `High utilization (${utilizationPercent.toFixed(1)}%), recommend scaling up`;
        }
      } else if (stryMutAct_9fa48("87") ? utilizationPercent > 30 : stryMutAct_9fa48("86") ? utilizationPercent < 30 : stryMutAct_9fa48("85") ? false : stryMutAct_9fa48("84") ? true : (stryCov_9fa48("84", "85", "86", "87"), utilizationPercent <= 30)) {
        if (stryMutAct_9fa48("88")) {
          {}
        } else {
          stryCov_9fa48("88");
          scalingRecommendation = "scale_down";
          recommendationRationale = `Low utilization (${utilizationPercent.toFixed(1)}%), consider scaling down`;
        }
      } else {
        if (stryMutAct_9fa48("91")) {
          {}
        } else {
          stryCov_9fa48("91");
          scalingRecommendation = "maintain";
          recommendationRationale = `Utilization at ${utilizationPercent.toFixed(1)}%, current capacity is adequate`;
        }
      }
      const analysis: CapacityAnalysis = stryMutAct_9fa48("94") ? {} : (stryCov_9fa48("94"), {
        timestamp: new Date(),
        windowMs: this.config.capacityAnalysisIntervalMs,
        totalCapacity: stryMutAct_9fa48("95") ? {} : (stryCov_9fa48("95"), {
          cpuPercent: poolStats.totalCpuCapacity,
          memoryMb: poolStats.totalMemoryCapacity,
          agentCount: poolStats.totalAgents
        }),
        usedCapacity: stryMutAct_9fa48("96") ? {} : (stryCov_9fa48("96"), {
          cpuPercent: poolStats.usedCpuCapacity,
          memoryMb: poolStats.usedMemoryCapacity,
          activeAgents: poolStats.activeAgents
        }),
        availableCapacity: stryMutAct_9fa48("97") ? {} : (stryCov_9fa48("97"), {
          cpuPercent: stryMutAct_9fa48("98") ? poolStats.totalCpuCapacity + poolStats.usedCpuCapacity : (stryCov_9fa48("98"), poolStats.totalCpuCapacity - poolStats.usedCpuCapacity),
          memoryMb: stryMutAct_9fa48("99") ? poolStats.totalMemoryCapacity + poolStats.usedMemoryCapacity : (stryCov_9fa48("99"), poolStats.totalMemoryCapacity - poolStats.usedMemoryCapacity),
          idleAgents: poolStats.idleAgents
        }),
        utilizationPercent,
        scalingRecommendation,
        recommendationRationale
      });
      this.lastCapacityAnalysis = analysis;
      this.logger.debug("Capacity analysis completed", stryMutAct_9fa48("101") ? {} : (stryCov_9fa48("101"), {
        utilization: utilizationPercent.toFixed(1),
        recommendation: scalingRecommendation
      }));
      return analysis;
    }
  }

  /**
   * Get resource pool statistics
   *
   * @returns Resource pool statistics
   */
  async getPoolStatistics(): Promise<ResourcePoolStats> {
    if (stryMutAct_9fa48("102")) {
      {}
    } else {
      stryCov_9fa48("102");
      return this.resourceMonitor.getPoolStats();
    }
  }

  /**
   * Get current configuration
   *
   * @returns Current configuration
   */
  getConfig(): AdaptiveResourceManagerConfig {
    if (stryMutAct_9fa48("103")) {
      {}
    } else {
      stryCov_9fa48("103");
      return stryMutAct_9fa48("104") ? {} : (stryCov_9fa48("104"), {
        ...this.config
      });
    }
  }

  /**
   * Update configuration
   *
   * @param config Partial configuration update
   */
  updateConfig(config: Partial<AdaptiveResourceManagerConfig>): void {
    if (stryMutAct_9fa48("105")) {
      {}
    } else {
      stryCov_9fa48("105");
      const wasRunning = this.isRunning;

      // Stop if running
      if (stryMutAct_9fa48("107") ? false : stryMutAct_9fa48("106") ? true : (stryCov_9fa48("106", "107"), wasRunning)) {
        if (stryMutAct_9fa48("108")) {
          {}
        } else {
          stryCov_9fa48("108");
          this.stop();
        }
      }

      // Update config
      this.config = stryMutAct_9fa48("109") ? {} : (stryCov_9fa48("109"), {
        ...this.config,
        ...config
      });

      // Update sub-components
      if (stryMutAct_9fa48("111") ? false : stryMutAct_9fa48("110") ? true : (stryCov_9fa48("110", "111"), config.loadBalancingStrategy)) {
        if (stryMutAct_9fa48("112")) {
          {}
        } else {
          stryCov_9fa48("112");
          this.loadBalancer.setStrategy(config.loadBalancingStrategy);
        }
      }
      if (stryMutAct_9fa48("114") ? false : stryMutAct_9fa48("113") ? true : (stryCov_9fa48("113", "114"), config.thresholds)) {
        if (stryMutAct_9fa48("115")) {
          {}
        } else {
          stryCov_9fa48("115");
          this.resourceMonitor.updateConfig(stryMutAct_9fa48("116") ? {} : (stryCov_9fa48("116"), {
            cpuThresholds: stryMutAct_9fa48("117") ? {} : (stryCov_9fa48("117"), {
              warning: config.thresholds.cpuWarning,
              critical: config.thresholds.cpuCritical
            }),
            memoryThresholds: stryMutAct_9fa48("118") ? {} : (stryCov_9fa48("118"), {
              warning: config.thresholds.memoryWarning,
              critical: config.thresholds.memoryCritical
            })
          }));
        }
      }

      // Restart if was running
      if (stryMutAct_9fa48("121") ? wasRunning || this.config.enabled : stryMutAct_9fa48("120") ? false : stryMutAct_9fa48("119") ? true : (stryCov_9fa48("119", "120", "121"), wasRunning && this.config.enabled)) {
        if (stryMutAct_9fa48("122")) {
          {}
        } else {
          stryCov_9fa48("122");
          this.start();
        }
      }
      this.logger.info("Configuration updated", this.config);
    }
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
    if (stryMutAct_9fa48("124")) {
      {}
    } else {
      stryCov_9fa48("124");
      return stryMutAct_9fa48("125") ? {} : (stryCov_9fa48("125"), {
        isRunning: this.isRunning,
        lastMonitoringTime: stryMutAct_9fa48("126") ? this.lastCapacityAnalysis.timestamp : (stryCov_9fa48("126"), this.lastCapacityAnalysis?.timestamp),
        activeAllocations: this.resourceAllocator.getActiveAllocationCount(),
        failoverEvents: this.failoverEvents.length
      });
    }
  }

  /**
   * Handle agent failover
   *
   * @param failedAgentId Failed agent identifier
   * @param backupAgentId Backup agent identifier
   * @returns Failover event
   */
  async handleFailover(failedAgentId: string, backupAgentId: string): Promise<FailoverEvent> {
    if (stryMutAct_9fa48("127")) {
      {}
    } else {
      stryCov_9fa48("127");
      const startTime = Date.now();
      if (stryMutAct_9fa48("130") ? false : stryMutAct_9fa48("129") ? true : stryMutAct_9fa48("128") ? this.config.enableAutoFailover : (stryCov_9fa48("128", "129", "130"), !this.config.enableAutoFailover)) {
        if (stryMutAct_9fa48("131")) {
          {}
        } else {
          stryCov_9fa48("131");
          throw new Error("Auto-failover is disabled");
        }
      }
      this.logger.info("Initiating agent failover", stryMutAct_9fa48("134") ? {} : (stryCov_9fa48("134"), {
        failedAgent: failedAgentId,
        backupAgent: backupAgentId
      }));
      try {
        if (stryMutAct_9fa48("135")) {
          {}
        } else {
          stryCov_9fa48("135");
          // Get task count for failed agent
          const tasksTransferred = this.resourceAllocator.getAgentAllocationCount(failedAgentId);

          // In real implementation, would transfer tasks to backup agent
          // For now, just record the event

          const failoverEvent: FailoverEvent = stryMutAct_9fa48("136") ? {} : (stryCov_9fa48("136"), {
            eventId: uuidv4(),
            failedAgentId,
            backupAgentId,
            tasksTransferred,
            timestamp: new Date(),
            durationMs: stryMutAct_9fa48("137") ? Date.now() + startTime : (stryCov_9fa48("137"), Date.now() - startTime),
            failureReason: "Agent health check failed",
            success: stryMutAct_9fa48("139") ? false : (stryCov_9fa48("139"), true)
          });
          this.failoverEvents.push(failoverEvent);

          // Keep only last 100 failover events
          if (stryMutAct_9fa48("143") ? this.failoverEvents.length <= 100 : stryMutAct_9fa48("142") ? this.failoverEvents.length >= 100 : stryMutAct_9fa48("141") ? false : stryMutAct_9fa48("140") ? true : (stryCov_9fa48("140", "141", "142", "143"), this.failoverEvents.length > 100)) {
            if (stryMutAct_9fa48("144")) {
              {}
            } else {
              stryCov_9fa48("144");
              this.failoverEvents.shift();
            }
          }
          this.logger.info("Agent failover completed", stryMutAct_9fa48("146") ? {} : (stryCov_9fa48("146"), {
            eventId: failoverEvent.eventId,
            tasksTransferred,
            durationMs: failoverEvent.durationMs
          }));
          return failoverEvent;
        }
      } catch (error) {
        if (stryMutAct_9fa48("147")) {
          {}
        } else {
          stryCov_9fa48("147");
          const failoverEvent: FailoverEvent = stryMutAct_9fa48("148") ? {} : (stryCov_9fa48("148"), {
            eventId: uuidv4(),
            failedAgentId,
            backupAgentId,
            tasksTransferred: 0,
            timestamp: new Date(),
            durationMs: stryMutAct_9fa48("149") ? Date.now() + startTime : (stryCov_9fa48("149"), Date.now() - startTime),
            failureReason: error instanceof Error ? error.message : "Unknown error",
            success: stryMutAct_9fa48("151") ? true : (stryCov_9fa48("151"), false)
          });
          this.failoverEvents.push(failoverEvent);
          this.logger.error("Agent failover failed", stryMutAct_9fa48("153") ? {} : (stryCov_9fa48("153"), {
            eventId: failoverEvent.eventId,
            error
          }));
          throw error;
        }
      }
    }
  }

  /**
   * Get failover event history
   *
   * @param count Number of events to retrieve
   * @returns Recent failover events
   */
  getFailoverHistory(count: number = 10): FailoverEvent[] {
    if (stryMutAct_9fa48("154")) {
      {}
    } else {
      stryCov_9fa48("154");
      return stryMutAct_9fa48("155") ? this.failoverEvents : (stryCov_9fa48("155"), this.failoverEvents.slice(stryMutAct_9fa48("156") ? +count : (stryCov_9fa48("156"), -count)));
    }
  }

  /**
   * Get allocation statistics
   *
   * @returns Allocation statistics
   */
  getAllocationStatistics() {
    if (stryMutAct_9fa48("157")) {
      {}
    } else {
      stryCov_9fa48("157");
      return this.resourceAllocator.getAllocationStats();
    }
  }

  /**
   * Get load distribution
   *
   * @returns Load distribution across agents
   */
  async getLoadDistribution(): Promise<Map<string, number>> {
    if (stryMutAct_9fa48("158")) {
      {}
    } else {
      stryCov_9fa48("158");
      return this.loadBalancer.getLoadDistribution();
    }
  }
}