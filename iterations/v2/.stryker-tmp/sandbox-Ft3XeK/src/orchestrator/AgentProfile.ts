/**
 * Agent Profile Management
 *
 * @author @darianrosebrook
 * @module orchestrator/AgentProfile
 *
 * Helper class for managing agent profiles with immutable updates
 * and running average performance calculations.
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
import type { AgentProfile, CurrentLoad, PerformanceHistory, PerformanceMetrics, Timestamp } from "../types/agent-registry";

/**
 * Helper class for agent profile operations.
 * Ensures immutability and correct running average calculations.
 */
export class AgentProfileHelper {
  /**
   * Update performance history with new metrics using running averages.
   *
   * @param history - Current performance history
   * @param metrics - New performance metrics from completed task
   * @returns Updated performance history with new running averages
   *
   * @remarks
   * Uses incremental averaging formula to avoid storing all historical data:
   * newAverage = oldAverage + (newValue - oldAverage) / (count + 1)
   */
  static updatePerformanceHistory(history: PerformanceHistory, metrics: PerformanceMetrics): PerformanceHistory {
    if (stryMutAct_9fa48("0")) {
      {}
    } else {
      stryCov_9fa48("0");
      const newCount = stryMutAct_9fa48("1") ? history.taskCount - 1 : (stryCov_9fa48("1"), history.taskCount + 1);

      // Incremental average updates
      const successValue = metrics.success ? 1.0 : 0.0;
      const newSuccessRate = stryMutAct_9fa48("2") ? history.successRate - (successValue - history.successRate) / newCount : (stryCov_9fa48("2"), history.successRate + (stryMutAct_9fa48("3") ? (successValue - history.successRate) * newCount : (stryCov_9fa48("3"), (stryMutAct_9fa48("4") ? successValue + history.successRate : (stryCov_9fa48("4"), successValue - history.successRate)) / newCount)));
      const newAverageQuality = stryMutAct_9fa48("5") ? history.averageQuality - (metrics.qualityScore - history.averageQuality) / newCount : (stryCov_9fa48("5"), history.averageQuality + (stryMutAct_9fa48("6") ? (metrics.qualityScore - history.averageQuality) * newCount : (stryCov_9fa48("6"), (stryMutAct_9fa48("7") ? metrics.qualityScore + history.averageQuality : (stryCov_9fa48("7"), metrics.qualityScore - history.averageQuality)) / newCount)));
      const newAverageLatency = stryMutAct_9fa48("8") ? history.averageLatency - (metrics.latencyMs - history.averageLatency) / newCount : (stryCov_9fa48("8"), history.averageLatency + (stryMutAct_9fa48("9") ? (metrics.latencyMs - history.averageLatency) * newCount : (stryCov_9fa48("9"), (stryMutAct_9fa48("10") ? metrics.latencyMs + history.averageLatency : (stryCov_9fa48("10"), metrics.latencyMs - history.averageLatency)) / newCount)));
      return stryMutAct_9fa48("11") ? {} : (stryCov_9fa48("11"), {
        successRate: newSuccessRate,
        averageQuality: newAverageQuality,
        averageLatency: newAverageLatency,
        taskCount: newCount
      });
    }
  }

  /**
   * Create initial performance history for a new agent.
   * Uses optimistic initialization to encourage exploration.
   *
   * @returns Initial performance history with optimistic values
   */
  static createInitialPerformanceHistory(): PerformanceHistory {
    if (stryMutAct_9fa48("12")) {
      {}
    } else {
      stryCov_9fa48("12");
      return stryMutAct_9fa48("13") ? {} : (stryCov_9fa48("13"), {
        successRate: 0.8,
        // Optimistic initialization
        averageQuality: 0.7,
        // Moderate quality assumption
        averageLatency: 5000,
        // 5 second baseline
        taskCount: 0
      });
    }
  }

  /**
   * Update current load by incrementing active tasks.
   *
   * @param load - Current load state
   * @param maxConcurrentTasks - Maximum concurrent tasks allowed
   * @returns Updated load with incremented active tasks
   */
  static incrementActiveTask(load: CurrentLoad, maxConcurrentTasks: number): CurrentLoad {
    if (stryMutAct_9fa48("14")) {
      {}
    } else {
      stryCov_9fa48("14");
      const newActiveTasks = stryMutAct_9fa48("15") ? load.activeTasks - 1 : (stryCov_9fa48("15"), load.activeTasks + 1);
      const utilizationPercent = stryMutAct_9fa48("16") ? newActiveTasks / maxConcurrentTasks / 100 : (stryCov_9fa48("16"), (stryMutAct_9fa48("17") ? newActiveTasks * maxConcurrentTasks : (stryCov_9fa48("17"), newActiveTasks / maxConcurrentTasks)) * 100);
      return stryMutAct_9fa48("18") ? {} : (stryCov_9fa48("18"), {
        activeTasks: newActiveTasks,
        queuedTasks: load.queuedTasks,
        utilizationPercent: stryMutAct_9fa48("19") ? Math.max(100, utilizationPercent) : (stryCov_9fa48("19"), Math.min(100, utilizationPercent))
      });
    }
  }

  /**
   * Update current load by decrementing active tasks.
   *
   * @param load - Current load state
   * @param maxConcurrentTasks - Maximum concurrent tasks allowed
   * @returns Updated load with decremented active tasks
   */
  static decrementActiveTask(load: CurrentLoad, maxConcurrentTasks: number): CurrentLoad {
    if (stryMutAct_9fa48("20")) {
      {}
    } else {
      stryCov_9fa48("20");
      const newActiveTasks = stryMutAct_9fa48("21") ? Math.min(0, load.activeTasks - 1) : (stryCov_9fa48("21"), Math.max(0, stryMutAct_9fa48("22") ? load.activeTasks + 1 : (stryCov_9fa48("22"), load.activeTasks - 1)));
      const utilizationPercent = stryMutAct_9fa48("23") ? newActiveTasks / maxConcurrentTasks / 100 : (stryCov_9fa48("23"), (stryMutAct_9fa48("24") ? newActiveTasks * maxConcurrentTasks : (stryCov_9fa48("24"), newActiveTasks / maxConcurrentTasks)) * 100);
      return stryMutAct_9fa48("25") ? {} : (stryCov_9fa48("25"), {
        activeTasks: newActiveTasks,
        queuedTasks: load.queuedTasks,
        utilizationPercent
      });
    }
  }

  /**
   * Update queued tasks count.
   *
   * @param load - Current load state
   * @param queuedTasks - New queued tasks count
   * @returns Updated load with new queue size
   */
  static updateQueuedTasks(load: CurrentLoad, queuedTasks: number): CurrentLoad {
    if (stryMutAct_9fa48("26")) {
      {}
    } else {
      stryCov_9fa48("26");
      return stryMutAct_9fa48("27") ? {} : (stryCov_9fa48("27"), {
        ...load,
        queuedTasks: stryMutAct_9fa48("28") ? Math.min(0, queuedTasks) : (stryCov_9fa48("28"), Math.max(0, queuedTasks))
      });
    }
  }

  /**
   * Create initial current load for a new agent.
   *
   * @returns Initial load state with zero tasks
   */
  static createInitialLoad(): CurrentLoad {
    if (stryMutAct_9fa48("29")) {
      {}
    } else {
      stryCov_9fa48("29");
      return stryMutAct_9fa48("30") ? {} : (stryCov_9fa48("30"), {
        activeTasks: 0,
        queuedTasks: 0,
        utilizationPercent: 0
      });
    }
  }

  /**
   * Update agent's last active timestamp.
   *
   * @param profile - Current agent profile
   * @param timestamp - New timestamp
   * @returns Updated profile with new lastActiveAt
   */
  static updateLastActive(profile: AgentProfile, timestamp: Timestamp = new Date().toISOString()): AgentProfile {
    if (stryMutAct_9fa48("31")) {
      {}
    } else {
      stryCov_9fa48("31");
      return stryMutAct_9fa48("32") ? {} : (stryCov_9fa48("32"), {
        ...profile,
        lastActiveAt: timestamp
      });
    }
  }

  /**
   * Check if an agent is considered stale (inactive for too long).
   *
   * @param profile - Agent profile to check
   * @param staleThresholdMs - Threshold in milliseconds
   * @param currentTime - Current timestamp (defaults to now)
   * @returns True if agent is stale
   */
  static isStale(profile: AgentProfile, staleThresholdMs: number, currentTime: Timestamp = new Date().toISOString()): boolean {
    if (stryMutAct_9fa48("33")) {
      {}
    } else {
      stryCov_9fa48("33");
      const lastActive = new Date(profile.lastActiveAt).getTime();
      const current = new Date(currentTime).getTime();
      return stryMutAct_9fa48("37") ? current - lastActive <= staleThresholdMs : stryMutAct_9fa48("36") ? current - lastActive >= staleThresholdMs : stryMutAct_9fa48("35") ? false : stryMutAct_9fa48("34") ? true : (stryCov_9fa48("34", "35", "36", "37"), (stryMutAct_9fa48("38") ? current + lastActive : (stryCov_9fa48("38"), current - lastActive)) > staleThresholdMs);
    }
  }

  /**
   * Calculate confidence interval for success rate based on task count.
   * Used for Upper Confidence Bound (UCB) calculations in routing.
   *
   * @param history - Performance history
   * @param totalTasks - Total tasks across all agents (for UCB calculation)
   * @returns Confidence interval bonus value
   */
  static calculateConfidenceInterval(history: PerformanceHistory, totalTasks: number): number {
    if (stryMutAct_9fa48("39")) {
      {}
    } else {
      stryCov_9fa48("39");
      if (stryMutAct_9fa48("42") ? history.taskCount !== 0 : stryMutAct_9fa48("41") ? false : stryMutAct_9fa48("40") ? true : (stryCov_9fa48("40", "41", "42"), history.taskCount === 0)) {
        if (stryMutAct_9fa48("43")) {
          {}
        } else {
          stryCov_9fa48("43");
          return 1.0; // Maximum exploration bonus for new agents
        }
      }

      // UCB exploration bonus formula
      return Math.sqrt(stryMutAct_9fa48("44") ? 2 * Math.log(totalTasks) * history.taskCount : (stryCov_9fa48("44"), (stryMutAct_9fa48("45") ? 2 / Math.log(totalTasks) : (stryCov_9fa48("45"), 2 * Math.log(totalTasks))) / history.taskCount));
    }
  }

  /**
   * Validate agent profile data for required fields and constraints.
   *
   * @param profile - Profile to validate
   * @throws Error if validation fails
   */
  static validateProfile(profile: Partial<AgentProfile>): void {
    if (stryMutAct_9fa48("46")) {
      {}
    } else {
      stryCov_9fa48("46");
      if (stryMutAct_9fa48("49") ? !profile.id && profile.id.trim() === "" : stryMutAct_9fa48("48") ? false : stryMutAct_9fa48("47") ? true : (stryCov_9fa48("47", "48", "49"), (stryMutAct_9fa48("50") ? profile.id : (stryCov_9fa48("50"), !profile.id)) || (stryMutAct_9fa48("52") ? profile.id.trim() !== "" : stryMutAct_9fa48("51") ? false : (stryCov_9fa48("51", "52"), (stryMutAct_9fa48("53") ? profile.id : (stryCov_9fa48("53"), profile.id.trim())) === (stryMutAct_9fa48("54") ? "Stryker was here!" : (stryCov_9fa48("54"), "")))))) {
        if (stryMutAct_9fa48("55")) {
          {}
        } else {
          stryCov_9fa48("55");
          throw new Error(stryMutAct_9fa48("56") ? "" : (stryCov_9fa48("56"), "Agent ID is required"));
        }
      }
      if (stryMutAct_9fa48("59") ? !profile.name && profile.name.trim() === "" : stryMutAct_9fa48("58") ? false : stryMutAct_9fa48("57") ? true : (stryCov_9fa48("57", "58", "59"), (stryMutAct_9fa48("60") ? profile.name : (stryCov_9fa48("60"), !profile.name)) || (stryMutAct_9fa48("62") ? profile.name.trim() !== "" : stryMutAct_9fa48("61") ? false : (stryCov_9fa48("61", "62"), (stryMutAct_9fa48("63") ? profile.name : (stryCov_9fa48("63"), profile.name.trim())) === (stryMutAct_9fa48("64") ? "Stryker was here!" : (stryCov_9fa48("64"), "")))))) {
        if (stryMutAct_9fa48("65")) {
          {}
        } else {
          stryCov_9fa48("65");
          throw new Error(stryMutAct_9fa48("66") ? "" : (stryCov_9fa48("66"), "Agent name is required"));
        }
      }
      if (stryMutAct_9fa48("69") ? false : stryMutAct_9fa48("68") ? true : stryMutAct_9fa48("67") ? profile.modelFamily : (stryCov_9fa48("67", "68", "69"), !profile.modelFamily)) {
        if (stryMutAct_9fa48("70")) {
          {}
        } else {
          stryCov_9fa48("70");
          throw new Error(stryMutAct_9fa48("71") ? "" : (stryCov_9fa48("71"), "Model family is required"));
        }
      }
      if (stryMutAct_9fa48("74") ? (!profile.capabilities || !profile.capabilities.taskTypes) && profile.capabilities.taskTypes.length === 0 : stryMutAct_9fa48("73") ? false : stryMutAct_9fa48("72") ? true : (stryCov_9fa48("72", "73", "74"), (stryMutAct_9fa48("76") ? !profile.capabilities && !profile.capabilities.taskTypes : stryMutAct_9fa48("75") ? false : (stryCov_9fa48("75", "76"), (stryMutAct_9fa48("77") ? profile.capabilities : (stryCov_9fa48("77"), !profile.capabilities)) || (stryMutAct_9fa48("78") ? profile.capabilities.taskTypes : (stryCov_9fa48("78"), !profile.capabilities.taskTypes)))) || (stryMutAct_9fa48("80") ? profile.capabilities.taskTypes.length !== 0 : stryMutAct_9fa48("79") ? false : (stryCov_9fa48("79", "80"), profile.capabilities.taskTypes.length === 0)))) {
        if (stryMutAct_9fa48("81")) {
          {}
        } else {
          stryCov_9fa48("81");
          throw new Error(stryMutAct_9fa48("82") ? "" : (stryCov_9fa48("82"), "At least one task type capability is required"));
        }
      }

      // Validate performance history ranges
      if (stryMutAct_9fa48("84") ? false : stryMutAct_9fa48("83") ? true : (stryCov_9fa48("83", "84"), profile.performanceHistory)) {
        if (stryMutAct_9fa48("85")) {
          {}
        } else {
          stryCov_9fa48("85");
          const {
            successRate,
            averageQuality
          } = profile.performanceHistory;
          if (stryMutAct_9fa48("88") ? successRate < 0 && successRate > 1 : stryMutAct_9fa48("87") ? false : stryMutAct_9fa48("86") ? true : (stryCov_9fa48("86", "87", "88"), (stryMutAct_9fa48("91") ? successRate >= 0 : stryMutAct_9fa48("90") ? successRate <= 0 : stryMutAct_9fa48("89") ? false : (stryCov_9fa48("89", "90", "91"), successRate < 0)) || (stryMutAct_9fa48("94") ? successRate <= 1 : stryMutAct_9fa48("93") ? successRate >= 1 : stryMutAct_9fa48("92") ? false : (stryCov_9fa48("92", "93", "94"), successRate > 1)))) {
            if (stryMutAct_9fa48("95")) {
              {}
            } else {
              stryCov_9fa48("95");
              throw new Error(stryMutAct_9fa48("96") ? "" : (stryCov_9fa48("96"), "Success rate must be between 0 and 1"));
            }
          }
          if (stryMutAct_9fa48("99") ? averageQuality < 0 && averageQuality > 1 : stryMutAct_9fa48("98") ? false : stryMutAct_9fa48("97") ? true : (stryCov_9fa48("97", "98", "99"), (stryMutAct_9fa48("102") ? averageQuality >= 0 : stryMutAct_9fa48("101") ? averageQuality <= 0 : stryMutAct_9fa48("100") ? false : (stryCov_9fa48("100", "101", "102"), averageQuality < 0)) || (stryMutAct_9fa48("105") ? averageQuality <= 1 : stryMutAct_9fa48("104") ? averageQuality >= 1 : stryMutAct_9fa48("103") ? false : (stryCov_9fa48("103", "104", "105"), averageQuality > 1)))) {
            if (stryMutAct_9fa48("106")) {
              {}
            } else {
              stryCov_9fa48("106");
              throw new Error(stryMutAct_9fa48("107") ? "" : (stryCov_9fa48("107"), "Average quality must be between 0 and 1"));
            }
          }
        }
      }

      // Validate current load ranges
      if (stryMutAct_9fa48("109") ? false : stryMutAct_9fa48("108") ? true : (stryCov_9fa48("108", "109"), profile.currentLoad)) {
        if (stryMutAct_9fa48("110")) {
          {}
        } else {
          stryCov_9fa48("110");
          const {
            activeTasks,
            queuedTasks,
            utilizationPercent
          } = profile.currentLoad;
          if (stryMutAct_9fa48("114") ? activeTasks >= 0 : stryMutAct_9fa48("113") ? activeTasks <= 0 : stryMutAct_9fa48("112") ? false : stryMutAct_9fa48("111") ? true : (stryCov_9fa48("111", "112", "113", "114"), activeTasks < 0)) {
            if (stryMutAct_9fa48("115")) {
              {}
            } else {
              stryCov_9fa48("115");
              throw new Error(stryMutAct_9fa48("116") ? "" : (stryCov_9fa48("116"), "Active tasks cannot be negative"));
            }
          }
          if (stryMutAct_9fa48("120") ? queuedTasks >= 0 : stryMutAct_9fa48("119") ? queuedTasks <= 0 : stryMutAct_9fa48("118") ? false : stryMutAct_9fa48("117") ? true : (stryCov_9fa48("117", "118", "119", "120"), queuedTasks < 0)) {
            if (stryMutAct_9fa48("121")) {
              {}
            } else {
              stryCov_9fa48("121");
              throw new Error(stryMutAct_9fa48("122") ? "" : (stryCov_9fa48("122"), "Queued tasks cannot be negative"));
            }
          }
          if (stryMutAct_9fa48("125") ? utilizationPercent < 0 && utilizationPercent > 100 : stryMutAct_9fa48("124") ? false : stryMutAct_9fa48("123") ? true : (stryCov_9fa48("123", "124", "125"), (stryMutAct_9fa48("128") ? utilizationPercent >= 0 : stryMutAct_9fa48("127") ? utilizationPercent <= 0 : stryMutAct_9fa48("126") ? false : (stryCov_9fa48("126", "127", "128"), utilizationPercent < 0)) || (stryMutAct_9fa48("131") ? utilizationPercent <= 100 : stryMutAct_9fa48("130") ? utilizationPercent >= 100 : stryMutAct_9fa48("129") ? false : (stryCov_9fa48("129", "130", "131"), utilizationPercent > 100)))) {
            if (stryMutAct_9fa48("132")) {
              {}
            } else {
              stryCov_9fa48("132");
              throw new Error(stryMutAct_9fa48("133") ? "" : (stryCov_9fa48("133"), "Utilization percent must be between 0 and 100"));
            }
          }
        }
      }
    }
  }

  /**
   * Create a deep clone of an agent profile for immutable updates.
   *
   * @param profile - Profile to clone
   * @returns Deep clone of the profile
   */
  static cloneProfile(profile: AgentProfile): AgentProfile {
    if (stryMutAct_9fa48("134")) {
      {}
    } else {
      stryCov_9fa48("134");
      return stryMutAct_9fa48("135") ? {} : (stryCov_9fa48("135"), {
        ...profile,
        capabilities: stryMutAct_9fa48("136") ? {} : (stryCov_9fa48("136"), {
          ...profile.capabilities,
          taskTypes: stryMutAct_9fa48("137") ? [] : (stryCov_9fa48("137"), [...profile.capabilities.taskTypes]),
          languages: stryMutAct_9fa48("138") ? [] : (stryCov_9fa48("138"), [...profile.capabilities.languages]),
          specializations: stryMutAct_9fa48("139") ? [] : (stryCov_9fa48("139"), [...profile.capabilities.specializations])
        }),
        performanceHistory: stryMutAct_9fa48("140") ? {} : (stryCov_9fa48("140"), {
          ...profile.performanceHistory
        }),
        currentLoad: stryMutAct_9fa48("141") ? {} : (stryCov_9fa48("141"), {
          ...profile.currentLoad
        })
      });
    }
  }
}