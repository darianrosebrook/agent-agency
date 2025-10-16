/**
 * @fileoverview Resource Monitor for Adaptive Resource Manager
 *
 * Tracks resource usage (CPU, memory, network) for agent pools.
 * Provides real-time visibility into system resource consumption.
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
import { ResourceType, type AgentResourceProfile, type IResourceMonitor, type ResourcePoolStats, type ResourceUsage } from "@/types/resource-types";

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
const DEFAULT_CONFIG: ResourceMonitorConfig = stryMutAct_9fa48("393") ? {} : (stryCov_9fa48("393"), {
  intervalMs: 5000,
  // 5 seconds
  enableHealthTracking: stryMutAct_9fa48("394") ? false : (stryCov_9fa48("394"), true),
  cpuThresholds: stryMutAct_9fa48("395") ? {} : (stryCov_9fa48("395"), {
    warning: 70,
    critical: 85
  }),
  memoryThresholds: stryMutAct_9fa48("396") ? {} : (stryCov_9fa48("396"), {
    warning: 75,
    critical: 90
  }),
  defaultMaxTaskCapacity: 10
});

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
  private isRunning = stryMutAct_9fa48("397") ? true : (stryCov_9fa48("397"), false);
  private monitoringTimer?: ReturnType<typeof setInterval>;
  constructor(config: Partial<ResourceMonitorConfig> = {}) {
    if (stryMutAct_9fa48("398")) {
      {}
    } else {
      stryCov_9fa48("398");
      this.logger = new Logger("ResourceMonitor");
      this.config = stryMutAct_9fa48("400") ? {} : (stryCov_9fa48("400"), {
        ...DEFAULT_CONFIG,
        ...config
      });
    }
  }

  /**
   * Start resource monitoring
   */
  async start(): Promise<void> {
    if (stryMutAct_9fa48("401")) {
      {}
    } else {
      stryCov_9fa48("401");
      if (stryMutAct_9fa48("403") ? false : stryMutAct_9fa48("402") ? true : (stryCov_9fa48("402", "403"), this.isRunning)) {
        if (stryMutAct_9fa48("404")) {
          {}
        } else {
          stryCov_9fa48("404");
          this.logger.warn("Resource monitor already running");
          return;
        }
      }
      this.isRunning = stryMutAct_9fa48("406") ? false : (stryCov_9fa48("406"), true);

      // Clear any existing timer first to prevent multiple timers
      if (stryMutAct_9fa48("408") ? false : stryMutAct_9fa48("407") ? true : (stryCov_9fa48("407", "408"), this.monitoringTimer)) {
        if (stryMutAct_9fa48("409")) {
          {}
        } else {
          stryCov_9fa48("409");
          clearInterval(this.monitoringTimer);
          this.monitoringTimer = undefined;
        }
      }

      // Start periodic health updates
      if (stryMutAct_9fa48("411") ? false : stryMutAct_9fa48("410") ? true : (stryCov_9fa48("410", "411"), this.config.enableHealthTracking)) {
        if (stryMutAct_9fa48("412")) {
          {}
        } else {
          stryCov_9fa48("412");
          this.monitoringTimer = setInterval(async () => {
            if (stryMutAct_9fa48("413")) {
              {}
            } else {
              stryCov_9fa48("413");
              await this.updateAgentHealth();
            }
          }, this.config.intervalMs);
        }
      }
      this.logger.info("Resource monitor started", stryMutAct_9fa48("415") ? {} : (stryCov_9fa48("415"), {
        intervalMs: this.config.intervalMs,
        healthTracking: this.config.enableHealthTracking
      }));
    }
  }

  /**
   * Stop resource monitoring
   */
  async stop(): Promise<void> {
    if (stryMutAct_9fa48("416")) {
      {}
    } else {
      stryCov_9fa48("416");
      if (stryMutAct_9fa48("419") ? false : stryMutAct_9fa48("418") ? true : stryMutAct_9fa48("417") ? this.isRunning : (stryCov_9fa48("417", "418", "419"), !this.isRunning)) {
        if (stryMutAct_9fa48("420")) {
          {}
        } else {
          stryCov_9fa48("420");
          return;
        }
      }
      if (stryMutAct_9fa48("422") ? false : stryMutAct_9fa48("421") ? true : (stryCov_9fa48("421", "422"), this.monitoringTimer)) {
        if (stryMutAct_9fa48("423")) {
          {}
        } else {
          stryCov_9fa48("423");
          clearInterval(this.monitoringTimer);
          this.monitoringTimer = undefined;
        }
      }
      this.isRunning = stryMutAct_9fa48("424") ? true : (stryCov_9fa48("424"), false);
      this.logger.info("Resource monitor stopped");
    }
  }

  /**
   * Get current resource usage for an agent
   *
   * @param agentId Agent identifier
   * @returns Agent resource profile or null if not found
   */
  async getAgentResources(agentId: string): Promise<AgentResourceProfile | null> {
    if (stryMutAct_9fa48("426")) {
      {}
    } else {
      stryCov_9fa48("426");
      return stryMutAct_9fa48("427") ? this.agentProfiles.get(agentId) && null : (stryCov_9fa48("427"), this.agentProfiles.get(agentId) ?? null);
    }
  }

  /**
   * Get all agent resource profiles
   *
   * @returns All agent resource profiles
   */
  async getAllAgentResources(): Promise<AgentResourceProfile[]> {
    if (stryMutAct_9fa48("428")) {
      {}
    } else {
      stryCov_9fa48("428");
      return Array.from(this.agentProfiles.values());
    }
  }

  /**
   * Record resource usage for an agent
   *
   * @param agentId Agent identifier
   * @param usage Resource usage data
   */
  async recordUsage(agentId: string, usage: ResourceUsage): Promise<void> {
    if (stryMutAct_9fa48("429")) {
      {}
    } else {
      stryCov_9fa48("429");
      let profile = this.agentProfiles.get(agentId);
      if (stryMutAct_9fa48("432") ? false : stryMutAct_9fa48("431") ? true : stryMutAct_9fa48("430") ? profile : (stryCov_9fa48("430", "431", "432"), !profile)) {
        if (stryMutAct_9fa48("433")) {
          {}
        } else {
          stryCov_9fa48("433");
          // Create new profile
          profile = this.createInitialProfile(agentId);
          this.agentProfiles.set(agentId, profile);
        }
      }

      // Update resource usage
      switch (usage.type) {
        case ResourceType.CPU:
          if (stryMutAct_9fa48("434")) {} else {
            stryCov_9fa48("434");
            profile.cpuUsage = usage;
            break;
          }
        case ResourceType.MEMORY:
          if (stryMutAct_9fa48("435")) {} else {
            stryCov_9fa48("435");
            profile.memoryUsage = usage;
            break;
          }
        case ResourceType.NETWORK:
          if (stryMutAct_9fa48("436")) {} else {
            stryCov_9fa48("436");
            profile.networkUsage = usage;
            break;
          }
      }
      profile.lastUpdated = new Date();

      // Update health status
      if (stryMutAct_9fa48("438") ? false : stryMutAct_9fa48("437") ? true : (stryCov_9fa48("437", "438"), this.config.enableHealthTracking)) {
        if (stryMutAct_9fa48("439")) {
          {}
        } else {
          stryCov_9fa48("439");
          profile.healthStatus = this.computeHealthStatus(profile);
        }
      }
      this.logger.debug("Recorded resource usage", stryMutAct_9fa48("441") ? {} : (stryCov_9fa48("441"), {
        agentId,
        resourceType: usage.type,
        usagePercent: usage.usagePercent,
        healthStatus: profile.healthStatus
      }));
    }
  }

  /**
   * Get resource pool statistics
   *
   * @returns Resource pool statistics
   */
  async getPoolStats(): Promise<ResourcePoolStats> {
    if (stryMutAct_9fa48("442")) {
      {}
    } else {
      stryCov_9fa48("442");
      const allProfiles = Array.from(this.agentProfiles.values());
      const activeAgents = stryMutAct_9fa48("443") ? allProfiles.length : (stryCov_9fa48("443"), allProfiles.filter(stryMutAct_9fa48("444") ? () => undefined : (stryCov_9fa48("444"), p => stryMutAct_9fa48("448") ? p.currentTaskCount <= 0 : stryMutAct_9fa48("447") ? p.currentTaskCount >= 0 : stryMutAct_9fa48("446") ? false : stryMutAct_9fa48("445") ? true : (stryCov_9fa48("445", "446", "447", "448"), p.currentTaskCount > 0))).length);
      const idleAgents = stryMutAct_9fa48("449") ? allProfiles.length : (stryCov_9fa48("449"), allProfiles.filter(stryMutAct_9fa48("450") ? () => undefined : (stryCov_9fa48("450"), p => stryMutAct_9fa48("453") ? p.currentTaskCount !== 0 : stryMutAct_9fa48("452") ? false : stryMutAct_9fa48("451") ? true : (stryCov_9fa48("451", "452", "453"), p.currentTaskCount === 0))).length);
      const unhealthyAgents = stryMutAct_9fa48("454") ? allProfiles.length : (stryCov_9fa48("454"), allProfiles.filter(stryMutAct_9fa48("455") ? () => undefined : (stryCov_9fa48("455"), p => stryMutAct_9fa48("458") ? p.healthStatus !== "unhealthy" : stryMutAct_9fa48("457") ? false : stryMutAct_9fa48("456") ? true : (stryCov_9fa48("456", "457", "458"), p.healthStatus === "unhealthy"))).length);
      const totalCpu = allProfiles.reduce(stryMutAct_9fa48("460") ? () => undefined : (stryCov_9fa48("460"), (sum, p) => stryMutAct_9fa48("461") ? sum - p.cpuUsage.maximum : (stryCov_9fa48("461"), sum + p.cpuUsage.maximum)), 0);
      const usedCpu = allProfiles.reduce(stryMutAct_9fa48("462") ? () => undefined : (stryCov_9fa48("462"), (sum, p) => stryMutAct_9fa48("463") ? sum - p.cpuUsage.current : (stryCov_9fa48("463"), sum + p.cpuUsage.current)), 0);
      const totalMemory = allProfiles.reduce(stryMutAct_9fa48("464") ? () => undefined : (stryCov_9fa48("464"), (sum, p) => stryMutAct_9fa48("465") ? sum - p.memoryUsage.maximum : (stryCov_9fa48("465"), sum + p.memoryUsage.maximum)), 0);
      const usedMemory = allProfiles.reduce(stryMutAct_9fa48("466") ? () => undefined : (stryCov_9fa48("466"), (sum, p) => stryMutAct_9fa48("467") ? sum - p.memoryUsage.current : (stryCov_9fa48("467"), sum + p.memoryUsage.current)), 0);
      const totalTasks = allProfiles.reduce(stryMutAct_9fa48("468") ? () => undefined : (stryCov_9fa48("468"), (sum, p) => stryMutAct_9fa48("469") ? sum - p.currentTaskCount : (stryCov_9fa48("469"), sum + p.currentTaskCount)), 0);
      const completionTimes = allProfiles.map(p => p.avgTaskCompletionMs).filter(t => t !== undefined) as number[];
      const avgCompletion = (stryMutAct_9fa48("473") ? completionTimes.length <= 0 : stryMutAct_9fa48("472") ? completionTimes.length >= 0 : stryMutAct_9fa48("471") ? false : stryMutAct_9fa48("470") ? true : (stryCov_9fa48("470", "471", "472", "473"), completionTimes.length > 0)) ? stryMutAct_9fa48("474") ? completionTimes.reduce((sum, t) => sum + t, 0) * completionTimes.length : (stryCov_9fa48("474"), completionTimes.reduce(stryMutAct_9fa48("475") ? () => undefined : (stryCov_9fa48("475"), (sum, t) => stryMutAct_9fa48("476") ? sum - t : (stryCov_9fa48("476"), sum + t)), 0) / completionTimes.length) : 0;
      return stryMutAct_9fa48("477") ? {} : (stryCov_9fa48("477"), {
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
        lastUpdated: new Date()
      });
    }
  }

  /**
   * Update task count for an agent
   *
   * @param agentId Agent identifier
   * @param taskCount Current task count
   */
  async updateTaskCount(agentId: string, taskCount: number): Promise<void> {
    if (stryMutAct_9fa48("478")) {
      {}
    } else {
      stryCov_9fa48("478");
      const profile = this.agentProfiles.get(agentId);
      if (stryMutAct_9fa48("480") ? false : stryMutAct_9fa48("479") ? true : (stryCov_9fa48("479", "480"), profile)) {
        if (stryMutAct_9fa48("481")) {
          {}
        } else {
          stryCov_9fa48("481");
          profile.currentTaskCount = taskCount;
          profile.lastUpdated = new Date();
        }
      }
    }
  }

  /**
   * Update average task completion time for an agent
   *
   * @param agentId Agent identifier
   * @param completionMs Average completion time (ms)
   */
  async updateTaskCompletionTime(agentId: string, completionMs: number): Promise<void> {
    if (stryMutAct_9fa48("482")) {
      {}
    } else {
      stryCov_9fa48("482");
      const profile = this.agentProfiles.get(agentId);
      if (stryMutAct_9fa48("484") ? false : stryMutAct_9fa48("483") ? true : (stryCov_9fa48("483", "484"), profile)) {
        if (stryMutAct_9fa48("485")) {
          {}
        } else {
          stryCov_9fa48("485");
          profile.avgTaskCompletionMs = completionMs;
          profile.lastUpdated = new Date();
        }
      }
    }
  }

  /**
   * Remove agent from monitoring
   *
   * @param agentId Agent identifier
   */
  async removeAgent(agentId: string): Promise<void> {
    if (stryMutAct_9fa48("486")) {
      {}
    } else {
      stryCov_9fa48("486");
      this.agentProfiles.delete(agentId);
      this.logger.info("Agent removed from monitoring", stryMutAct_9fa48("488") ? {} : (stryCov_9fa48("488"), {
        agentId
      }));
    }
  }

  /**
   * Get configuration
   */
  getConfig(): ResourceMonitorConfig {
    if (stryMutAct_9fa48("489")) {
      {}
    } else {
      stryCov_9fa48("489");
      return stryMutAct_9fa48("490") ? {} : (stryCov_9fa48("490"), {
        ...this.config
      });
    }
  }

  /**
   * Update configuration
   */
  updateConfig(config: Partial<ResourceMonitorConfig>): void {
    if (stryMutAct_9fa48("491")) {
      {}
    } else {
      stryCov_9fa48("491");
      this.config = stryMutAct_9fa48("492") ? {} : (stryCov_9fa48("492"), {
        ...this.config,
        ...config
      });
      this.logger.info("Configuration updated", this.config);
    }
  }

  /**
   * Create initial profile for an agent
   */
  private createInitialProfile(agentId: string): AgentResourceProfile {
    if (stryMutAct_9fa48("494")) {
      {}
    } else {
      stryCov_9fa48("494");
      return stryMutAct_9fa48("495") ? {} : (stryCov_9fa48("495"), {
        agentId,
        cpuUsage: stryMutAct_9fa48("496") ? {} : (stryCov_9fa48("496"), {
          type: ResourceType.CPU,
          current: 0,
          maximum: 100,
          usagePercent: 0,
          unit: "%",
          timestamp: new Date(),
          source: agentId
        }),
        memoryUsage: stryMutAct_9fa48("498") ? {} : (stryCov_9fa48("498"), {
          type: ResourceType.MEMORY,
          current: 0,
          maximum: 1024,
          usagePercent: 0,
          unit: "MB",
          timestamp: new Date(),
          source: agentId
        }),
        networkUsage: stryMutAct_9fa48("500") ? {} : (stryCov_9fa48("500"), {
          type: ResourceType.NETWORK,
          current: 0,
          maximum: 1000,
          usagePercent: 0,
          unit: "Mbps",
          timestamp: new Date(),
          source: agentId
        }),
        currentTaskCount: 0,
        maxTaskCapacity: this.config.defaultMaxTaskCapacity,
        healthStatus: "healthy",
        lastUpdated: new Date()
      });
    }
  }

  /**
   * Compute health status for an agent
   */
  private computeHealthStatus(profile: AgentResourceProfile): "healthy" | "degraded" | "unhealthy" {
    if (stryMutAct_9fa48("503")) {
      {}
    } else {
      stryCov_9fa48("503");
      const cpuPercent = profile.cpuUsage.usagePercent;
      const memoryPercent = profile.memoryUsage.usagePercent;

      // Check critical thresholds
      if (stryMutAct_9fa48("506") ? cpuPercent >= this.config.cpuThresholds.critical && memoryPercent >= this.config.memoryThresholds.critical : stryMutAct_9fa48("505") ? false : stryMutAct_9fa48("504") ? true : (stryCov_9fa48("504", "505", "506"), (stryMutAct_9fa48("509") ? cpuPercent < this.config.cpuThresholds.critical : stryMutAct_9fa48("508") ? cpuPercent > this.config.cpuThresholds.critical : stryMutAct_9fa48("507") ? false : (stryCov_9fa48("507", "508", "509"), cpuPercent >= this.config.cpuThresholds.critical)) || (stryMutAct_9fa48("512") ? memoryPercent < this.config.memoryThresholds.critical : stryMutAct_9fa48("511") ? memoryPercent > this.config.memoryThresholds.critical : stryMutAct_9fa48("510") ? false : (stryCov_9fa48("510", "511", "512"), memoryPercent >= this.config.memoryThresholds.critical)))) {
        if (stryMutAct_9fa48("513")) {
          {}
        } else {
          stryCov_9fa48("513");
          return "unhealthy";
        }
      }

      // Check warning thresholds
      if (stryMutAct_9fa48("517") ? cpuPercent >= this.config.cpuThresholds.warning && memoryPercent >= this.config.memoryThresholds.warning : stryMutAct_9fa48("516") ? false : stryMutAct_9fa48("515") ? true : (stryCov_9fa48("515", "516", "517"), (stryMutAct_9fa48("520") ? cpuPercent < this.config.cpuThresholds.warning : stryMutAct_9fa48("519") ? cpuPercent > this.config.cpuThresholds.warning : stryMutAct_9fa48("518") ? false : (stryCov_9fa48("518", "519", "520"), cpuPercent >= this.config.cpuThresholds.warning)) || (stryMutAct_9fa48("523") ? memoryPercent < this.config.memoryThresholds.warning : stryMutAct_9fa48("522") ? memoryPercent > this.config.memoryThresholds.warning : stryMutAct_9fa48("521") ? false : (stryCov_9fa48("521", "522", "523"), memoryPercent >= this.config.memoryThresholds.warning)))) {
        if (stryMutAct_9fa48("524")) {
          {}
        } else {
          stryCov_9fa48("524");
          return "degraded";
        }
      }
      return "healthy";
    }
  }

  /**
   * Update health status for all agents
   */
  private async updateAgentHealth(): Promise<void> {
    if (stryMutAct_9fa48("527")) {
      {}
    } else {
      stryCov_9fa48("527");
      for (const profile of this.agentProfiles.values()) {
        if (stryMutAct_9fa48("528")) {
          {}
        } else {
          stryCov_9fa48("528");
          profile.healthStatus = this.computeHealthStatus(profile);
        }
      }
    }
  }
}