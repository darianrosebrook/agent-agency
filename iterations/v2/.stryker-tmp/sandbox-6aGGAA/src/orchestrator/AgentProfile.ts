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
    if (stryMutAct_9fa48("268")) {
      {}
    } else {
      stryCov_9fa48("268");
      const newCount = stryMutAct_9fa48("269") ? history.taskCount - 1 : (stryCov_9fa48("269"), history.taskCount + 1);

      // Incremental average updates
      const successValue = metrics.success ? 1.0 : 0.0;
      const newSuccessRate = stryMutAct_9fa48("270") ? history.successRate - (successValue - history.successRate) / newCount : (stryCov_9fa48("270"), history.successRate + (stryMutAct_9fa48("271") ? (successValue - history.successRate) * newCount : (stryCov_9fa48("271"), (stryMutAct_9fa48("272") ? successValue + history.successRate : (stryCov_9fa48("272"), successValue - history.successRate)) / newCount)));
      const newAverageQuality = stryMutAct_9fa48("273") ? history.averageQuality - (metrics.qualityScore - history.averageQuality) / newCount : (stryCov_9fa48("273"), history.averageQuality + (stryMutAct_9fa48("274") ? (metrics.qualityScore - history.averageQuality) * newCount : (stryCov_9fa48("274"), (stryMutAct_9fa48("275") ? metrics.qualityScore + history.averageQuality : (stryCov_9fa48("275"), metrics.qualityScore - history.averageQuality)) / newCount)));
      const newAverageLatency = stryMutAct_9fa48("276") ? history.averageLatency - (metrics.latencyMs - history.averageLatency) / newCount : (stryCov_9fa48("276"), history.averageLatency + (stryMutAct_9fa48("277") ? (metrics.latencyMs - history.averageLatency) * newCount : (stryCov_9fa48("277"), (stryMutAct_9fa48("278") ? metrics.latencyMs + history.averageLatency : (stryCov_9fa48("278"), metrics.latencyMs - history.averageLatency)) / newCount)));
      return stryMutAct_9fa48("279") ? {} : (stryCov_9fa48("279"), {
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
    if (stryMutAct_9fa48("280")) {
      {}
    } else {
      stryCov_9fa48("280");
      return stryMutAct_9fa48("281") ? {} : (stryCov_9fa48("281"), {
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
    if (stryMutAct_9fa48("282")) {
      {}
    } else {
      stryCov_9fa48("282");
      const newActiveTasks = stryMutAct_9fa48("283") ? load.activeTasks - 1 : (stryCov_9fa48("283"), load.activeTasks + 1);
      const utilizationPercent = stryMutAct_9fa48("284") ? newActiveTasks / maxConcurrentTasks / 100 : (stryCov_9fa48("284"), (stryMutAct_9fa48("285") ? newActiveTasks * maxConcurrentTasks : (stryCov_9fa48("285"), newActiveTasks / maxConcurrentTasks)) * 100);
      return stryMutAct_9fa48("286") ? {} : (stryCov_9fa48("286"), {
        activeTasks: newActiveTasks,
        queuedTasks: load.queuedTasks,
        utilizationPercent: stryMutAct_9fa48("287") ? Math.max(100, utilizationPercent) : (stryCov_9fa48("287"), Math.min(100, utilizationPercent))
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
    if (stryMutAct_9fa48("288")) {
      {}
    } else {
      stryCov_9fa48("288");
      const newActiveTasks = stryMutAct_9fa48("289") ? Math.min(0, load.activeTasks - 1) : (stryCov_9fa48("289"), Math.max(0, stryMutAct_9fa48("290") ? load.activeTasks + 1 : (stryCov_9fa48("290"), load.activeTasks - 1)));
      const utilizationPercent = stryMutAct_9fa48("291") ? newActiveTasks / maxConcurrentTasks / 100 : (stryCov_9fa48("291"), (stryMutAct_9fa48("292") ? newActiveTasks * maxConcurrentTasks : (stryCov_9fa48("292"), newActiveTasks / maxConcurrentTasks)) * 100);
      return stryMutAct_9fa48("293") ? {} : (stryCov_9fa48("293"), {
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
    if (stryMutAct_9fa48("294")) {
      {}
    } else {
      stryCov_9fa48("294");
      return stryMutAct_9fa48("295") ? {} : (stryCov_9fa48("295"), {
        ...load,
        queuedTasks: stryMutAct_9fa48("296") ? Math.min(0, queuedTasks) : (stryCov_9fa48("296"), Math.max(0, queuedTasks))
      });
    }
  }

  /**
   * Create initial current load for a new agent.
   *
   * @returns Initial load state with zero tasks
   */
  static createInitialLoad(): CurrentLoad {
    if (stryMutAct_9fa48("297")) {
      {}
    } else {
      stryCov_9fa48("297");
      return stryMutAct_9fa48("298") ? {} : (stryCov_9fa48("298"), {
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
    if (stryMutAct_9fa48("299")) {
      {}
    } else {
      stryCov_9fa48("299");
      return stryMutAct_9fa48("300") ? {} : (stryCov_9fa48("300"), {
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
    if (stryMutAct_9fa48("301")) {
      {}
    } else {
      stryCov_9fa48("301");
      const lastActive = new Date(profile.lastActiveAt).getTime();
      const current = new Date(currentTime).getTime();
      return stryMutAct_9fa48("305") ? current - lastActive <= staleThresholdMs : stryMutAct_9fa48("304") ? current - lastActive >= staleThresholdMs : stryMutAct_9fa48("303") ? false : stryMutAct_9fa48("302") ? true : (stryCov_9fa48("302", "303", "304", "305"), (stryMutAct_9fa48("306") ? current + lastActive : (stryCov_9fa48("306"), current - lastActive)) > staleThresholdMs);
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
    if (stryMutAct_9fa48("307")) {
      {}
    } else {
      stryCov_9fa48("307");
      if (stryMutAct_9fa48("310") ? history.taskCount !== 0 : stryMutAct_9fa48("309") ? false : stryMutAct_9fa48("308") ? true : (stryCov_9fa48("308", "309", "310"), history.taskCount === 0)) {
        if (stryMutAct_9fa48("311")) {
          {}
        } else {
          stryCov_9fa48("311");
          return 1.0; // Maximum exploration bonus for new agents
        }
      }

      // UCB exploration bonus formula
      return Math.sqrt(stryMutAct_9fa48("312") ? 2 * Math.log(totalTasks) * history.taskCount : (stryCov_9fa48("312"), (stryMutAct_9fa48("313") ? 2 / Math.log(totalTasks) : (stryCov_9fa48("313"), 2 * Math.log(totalTasks))) / history.taskCount));
    }
  }

  /**
   * Validate agent profile data for required fields and constraints.
   *
   * @param profile - Profile to validate
   * @throws Error if validation fails
   */
  static validateProfile(profile: Partial<AgentProfile>): void {
    if (stryMutAct_9fa48("314")) {
      {}
    } else {
      stryCov_9fa48("314");
      if (stryMutAct_9fa48("317") ? !profile.id && profile.id.trim() === "" : stryMutAct_9fa48("316") ? false : stryMutAct_9fa48("315") ? true : (stryCov_9fa48("315", "316", "317"), (stryMutAct_9fa48("318") ? profile.id : (stryCov_9fa48("318"), !profile.id)) || (stryMutAct_9fa48("320") ? profile.id.trim() !== "" : stryMutAct_9fa48("319") ? false : (stryCov_9fa48("319", "320"), (stryMutAct_9fa48("321") ? profile.id : (stryCov_9fa48("321"), profile.id.trim())) === (stryMutAct_9fa48("322") ? "Stryker was here!" : (stryCov_9fa48("322"), "")))))) {
        if (stryMutAct_9fa48("323")) {
          {}
        } else {
          stryCov_9fa48("323");
          throw new Error(stryMutAct_9fa48("324") ? "" : (stryCov_9fa48("324"), "Agent ID is required"));
        }
      }
      if (stryMutAct_9fa48("327") ? !profile.name && profile.name.trim() === "" : stryMutAct_9fa48("326") ? false : stryMutAct_9fa48("325") ? true : (stryCov_9fa48("325", "326", "327"), (stryMutAct_9fa48("328") ? profile.name : (stryCov_9fa48("328"), !profile.name)) || (stryMutAct_9fa48("330") ? profile.name.trim() !== "" : stryMutAct_9fa48("329") ? false : (stryCov_9fa48("329", "330"), (stryMutAct_9fa48("331") ? profile.name : (stryCov_9fa48("331"), profile.name.trim())) === (stryMutAct_9fa48("332") ? "Stryker was here!" : (stryCov_9fa48("332"), "")))))) {
        if (stryMutAct_9fa48("333")) {
          {}
        } else {
          stryCov_9fa48("333");
          throw new Error(stryMutAct_9fa48("334") ? "" : (stryCov_9fa48("334"), "Agent name is required"));
        }
      }
      if (stryMutAct_9fa48("337") ? false : stryMutAct_9fa48("336") ? true : stryMutAct_9fa48("335") ? profile.modelFamily : (stryCov_9fa48("335", "336", "337"), !profile.modelFamily)) {
        if (stryMutAct_9fa48("338")) {
          {}
        } else {
          stryCov_9fa48("338");
          throw new Error(stryMutAct_9fa48("339") ? "" : (stryCov_9fa48("339"), "Model family is required"));
        }
      }
      if (stryMutAct_9fa48("342") ? (!profile.capabilities || !profile.capabilities.taskTypes) && profile.capabilities.taskTypes.length === 0 : stryMutAct_9fa48("341") ? false : stryMutAct_9fa48("340") ? true : (stryCov_9fa48("340", "341", "342"), (stryMutAct_9fa48("344") ? !profile.capabilities && !profile.capabilities.taskTypes : stryMutAct_9fa48("343") ? false : (stryCov_9fa48("343", "344"), (stryMutAct_9fa48("345") ? profile.capabilities : (stryCov_9fa48("345"), !profile.capabilities)) || (stryMutAct_9fa48("346") ? profile.capabilities.taskTypes : (stryCov_9fa48("346"), !profile.capabilities.taskTypes)))) || (stryMutAct_9fa48("348") ? profile.capabilities.taskTypes.length !== 0 : stryMutAct_9fa48("347") ? false : (stryCov_9fa48("347", "348"), profile.capabilities.taskTypes.length === 0)))) {
        if (stryMutAct_9fa48("349")) {
          {}
        } else {
          stryCov_9fa48("349");
          throw new Error(stryMutAct_9fa48("350") ? "" : (stryCov_9fa48("350"), "At least one task type capability is required"));
        }
      }

      // Validate performance history ranges
      if (stryMutAct_9fa48("352") ? false : stryMutAct_9fa48("351") ? true : (stryCov_9fa48("351", "352"), profile.performanceHistory)) {
        if (stryMutAct_9fa48("353")) {
          {}
        } else {
          stryCov_9fa48("353");
          const {
            successRate,
            averageQuality
          } = profile.performanceHistory;
          if (stryMutAct_9fa48("356") ? successRate < 0 && successRate > 1 : stryMutAct_9fa48("355") ? false : stryMutAct_9fa48("354") ? true : (stryCov_9fa48("354", "355", "356"), (stryMutAct_9fa48("359") ? successRate >= 0 : stryMutAct_9fa48("358") ? successRate <= 0 : stryMutAct_9fa48("357") ? false : (stryCov_9fa48("357", "358", "359"), successRate < 0)) || (stryMutAct_9fa48("362") ? successRate <= 1 : stryMutAct_9fa48("361") ? successRate >= 1 : stryMutAct_9fa48("360") ? false : (stryCov_9fa48("360", "361", "362"), successRate > 1)))) {
            if (stryMutAct_9fa48("363")) {
              {}
            } else {
              stryCov_9fa48("363");
              throw new Error(stryMutAct_9fa48("364") ? "" : (stryCov_9fa48("364"), "Success rate must be between 0 and 1"));
            }
          }
          if (stryMutAct_9fa48("367") ? averageQuality < 0 && averageQuality > 1 : stryMutAct_9fa48("366") ? false : stryMutAct_9fa48("365") ? true : (stryCov_9fa48("365", "366", "367"), (stryMutAct_9fa48("370") ? averageQuality >= 0 : stryMutAct_9fa48("369") ? averageQuality <= 0 : stryMutAct_9fa48("368") ? false : (stryCov_9fa48("368", "369", "370"), averageQuality < 0)) || (stryMutAct_9fa48("373") ? averageQuality <= 1 : stryMutAct_9fa48("372") ? averageQuality >= 1 : stryMutAct_9fa48("371") ? false : (stryCov_9fa48("371", "372", "373"), averageQuality > 1)))) {
            if (stryMutAct_9fa48("374")) {
              {}
            } else {
              stryCov_9fa48("374");
              throw new Error(stryMutAct_9fa48("375") ? "" : (stryCov_9fa48("375"), "Average quality must be between 0 and 1"));
            }
          }
        }
      }

      // Validate current load ranges
      if (stryMutAct_9fa48("377") ? false : stryMutAct_9fa48("376") ? true : (stryCov_9fa48("376", "377"), profile.currentLoad)) {
        if (stryMutAct_9fa48("378")) {
          {}
        } else {
          stryCov_9fa48("378");
          const {
            activeTasks,
            queuedTasks,
            utilizationPercent
          } = profile.currentLoad;
          if (stryMutAct_9fa48("382") ? activeTasks >= 0 : stryMutAct_9fa48("381") ? activeTasks <= 0 : stryMutAct_9fa48("380") ? false : stryMutAct_9fa48("379") ? true : (stryCov_9fa48("379", "380", "381", "382"), activeTasks < 0)) {
            if (stryMutAct_9fa48("383")) {
              {}
            } else {
              stryCov_9fa48("383");
              throw new Error(stryMutAct_9fa48("384") ? "" : (stryCov_9fa48("384"), "Active tasks cannot be negative"));
            }
          }
          if (stryMutAct_9fa48("388") ? queuedTasks >= 0 : stryMutAct_9fa48("387") ? queuedTasks <= 0 : stryMutAct_9fa48("386") ? false : stryMutAct_9fa48("385") ? true : (stryCov_9fa48("385", "386", "387", "388"), queuedTasks < 0)) {
            if (stryMutAct_9fa48("389")) {
              {}
            } else {
              stryCov_9fa48("389");
              throw new Error(stryMutAct_9fa48("390") ? "" : (stryCov_9fa48("390"), "Queued tasks cannot be negative"));
            }
          }
          if (stryMutAct_9fa48("393") ? utilizationPercent < 0 && utilizationPercent > 100 : stryMutAct_9fa48("392") ? false : stryMutAct_9fa48("391") ? true : (stryCov_9fa48("391", "392", "393"), (stryMutAct_9fa48("396") ? utilizationPercent >= 0 : stryMutAct_9fa48("395") ? utilizationPercent <= 0 : stryMutAct_9fa48("394") ? false : (stryCov_9fa48("394", "395", "396"), utilizationPercent < 0)) || (stryMutAct_9fa48("399") ? utilizationPercent <= 100 : stryMutAct_9fa48("398") ? utilizationPercent >= 100 : stryMutAct_9fa48("397") ? false : (stryCov_9fa48("397", "398", "399"), utilizationPercent > 100)))) {
            if (stryMutAct_9fa48("400")) {
              {}
            } else {
              stryCov_9fa48("400");
              throw new Error(stryMutAct_9fa48("401") ? "" : (stryCov_9fa48("401"), "Utilization percent must be between 0 and 100"));
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
    if (stryMutAct_9fa48("402")) {
      {}
    } else {
      stryCov_9fa48("402");
      return stryMutAct_9fa48("403") ? {} : (stryCov_9fa48("403"), {
        ...profile,
        capabilities: stryMutAct_9fa48("404") ? {} : (stryCov_9fa48("404"), {
          ...profile.capabilities,
          taskTypes: stryMutAct_9fa48("405") ? [] : (stryCov_9fa48("405"), [...profile.capabilities.taskTypes]),
          languages: stryMutAct_9fa48("406") ? [] : (stryCov_9fa48("406"), [...profile.capabilities.languages]),
          specializations: stryMutAct_9fa48("407") ? [] : (stryCov_9fa48("407"), [...profile.capabilities.specializations])
        }),
        performanceHistory: stryMutAct_9fa48("408") ? {} : (stryCov_9fa48("408"), {
          ...profile.performanceHistory
        }),
        currentLoad: stryMutAct_9fa48("409") ? {} : (stryCov_9fa48("409"), {
          ...profile.currentLoad
        })
      });
    }
  }
}