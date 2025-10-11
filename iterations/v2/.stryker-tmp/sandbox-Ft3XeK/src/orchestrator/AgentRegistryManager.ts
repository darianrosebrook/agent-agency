/**
 * Agent Registry Manager
 *
 * @author @darianrosebrook
 * @module orchestrator/AgentRegistryManager
 *
 * Central registry for managing agent profiles, capabilities, and performance history.
 * Implements ARBITER-001 specification with capability tracking and atomic updates.
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
import type { AgentId, AgentProfile, AgentQuery, AgentQueryResult, AgentRegistryConfig, PerformanceMetrics, RegistryStats } from "../types/agent-registry";
import { RegistryError, RegistryErrorType } from "../types/agent-registry";
import { AgentProfileHelper } from "./AgentProfile";

/**
 * Default configuration for the agent registry.
 */
const DEFAULT_CONFIG: AgentRegistryConfig = stryMutAct_9fa48("142") ? {} : (stryCov_9fa48("142"), {
  maxAgents: 1000,
  staleAgentThresholdMs: stryMutAct_9fa48("143") ? 24 * 60 * 60 / 1000 : (stryCov_9fa48("143"), (stryMutAct_9fa48("144") ? 24 * 60 / 60 : (stryCov_9fa48("144"), (stryMutAct_9fa48("145") ? 24 / 60 : (stryCov_9fa48("145"), 24 * 60)) * 60)) * 1000),
  // 24 hours
  enableAutoCleanup: stryMutAct_9fa48("146") ? false : (stryCov_9fa48("146"), true),
  cleanupIntervalMs: stryMutAct_9fa48("147") ? 60 * 60 / 1000 : (stryCov_9fa48("147"), (stryMutAct_9fa48("148") ? 60 / 60 : (stryCov_9fa48("148"), 60 * 60)) * 1000) // 1 hour
});

/**
 * Agent Registry Manager
 *
 * Maintains the catalog of available agents with their capabilities,
 * performance history, and current load status.
 *
 * @remarks
 * Thread-safe: Uses Map for O(1) lookups with atomic updates.
 * Invariants:
 * - Agent profiles are immutable except for performance metrics
 * - Performance history updates are atomic and isolated per agent
 * - Registry queries never block agent registration operations
 * - All capability changes are versioned and auditable
 */
export class AgentRegistryManager {
  private readonly agents: Map<AgentId, AgentProfile>;
  private readonly config: AgentRegistryConfig;
  private cleanupTimer?: ReturnType<typeof setInterval>;
  private readonly maxConcurrentTasksPerAgent: number = 10;
  constructor(config: Partial<AgentRegistryConfig> = {}) {
    if (stryMutAct_9fa48("149")) {
      {}
    } else {
      stryCov_9fa48("149");
      this.agents = new Map();
      this.config = stryMutAct_9fa48("150") ? {} : (stryCov_9fa48("150"), {
        ...DEFAULT_CONFIG,
        ...config
      });
      if (stryMutAct_9fa48("152") ? false : stryMutAct_9fa48("151") ? true : (stryCov_9fa48("151", "152"), this.config.enableAutoCleanup)) {
        if (stryMutAct_9fa48("153")) {
          {}
        } else {
          stryCov_9fa48("153");
          this.startAutoCleanup();
        }
      }
    }
  }

  /**
   * Register a new agent in the registry.
   *
   * @param agent - Agent to register (partial, will be filled with defaults)
   * @returns Complete agent profile with generated fields
   * @throws RegistryError if agent already exists or registry is full
   *
   * @remarks
   * Acceptance Criterion A1: Agent profile created with capability tracking initialized
   */
  async registerAgent(agent: Partial<AgentProfile>): Promise<AgentProfile> {
    if (stryMutAct_9fa48("154")) {
      {}
    } else {
      stryCov_9fa48("154");
      // Validate required fields
      AgentProfileHelper.validateProfile(agent);
      if (stryMutAct_9fa48("157") ? false : stryMutAct_9fa48("156") ? true : stryMutAct_9fa48("155") ? agent.id : (stryCov_9fa48("155", "156", "157"), !agent.id)) {
        if (stryMutAct_9fa48("158")) {
          {}
        } else {
          stryCov_9fa48("158");
          throw new RegistryError(RegistryErrorType.INVALID_AGENT_DATA, stryMutAct_9fa48("159") ? "" : (stryCov_9fa48("159"), "Agent ID is required"));
        }
      }

      // Check if agent already exists
      if (stryMutAct_9fa48("161") ? false : stryMutAct_9fa48("160") ? true : (stryCov_9fa48("160", "161"), this.agents.has(agent.id))) {
        if (stryMutAct_9fa48("162")) {
          {}
        } else {
          stryCov_9fa48("162");
          throw new RegistryError(RegistryErrorType.AGENT_ALREADY_EXISTS, stryMutAct_9fa48("163") ? `` : (stryCov_9fa48("163"), `Agent with ID ${agent.id} already exists`), stryMutAct_9fa48("164") ? {} : (stryCov_9fa48("164"), {
            agentId: agent.id
          }));
        }
      }

      // Check registry capacity
      if (stryMutAct_9fa48("168") ? this.agents.size < this.config.maxAgents : stryMutAct_9fa48("167") ? this.agents.size > this.config.maxAgents : stryMutAct_9fa48("166") ? false : stryMutAct_9fa48("165") ? true : (stryCov_9fa48("165", "166", "167", "168"), this.agents.size >= this.config.maxAgents)) {
        if (stryMutAct_9fa48("169")) {
          {}
        } else {
          stryCov_9fa48("169");
          throw new RegistryError(RegistryErrorType.REGISTRY_FULL, stryMutAct_9fa48("170") ? `` : (stryCov_9fa48("170"), `Registry is full (max: ${this.config.maxAgents} agents)`), stryMutAct_9fa48("171") ? {} : (stryCov_9fa48("171"), {
            maxAgents: this.config.maxAgents,
            currentSize: this.agents.size
          }));
        }
      }

      // Create complete profile with defaults
      const now = new Date().toISOString();
      const profile: AgentProfile = stryMutAct_9fa48("172") ? {} : (stryCov_9fa48("172"), {
        id: agent.id,
        name: agent.name!,
        modelFamily: agent.modelFamily!,
        capabilities: agent.capabilities!,
        performanceHistory: stryMutAct_9fa48("173") ? agent.performanceHistory && AgentProfileHelper.createInitialPerformanceHistory() : (stryCov_9fa48("173"), agent.performanceHistory ?? AgentProfileHelper.createInitialPerformanceHistory()),
        currentLoad: stryMutAct_9fa48("174") ? agent.currentLoad && AgentProfileHelper.createInitialLoad() : (stryCov_9fa48("174"), agent.currentLoad ?? AgentProfileHelper.createInitialLoad()),
        registeredAt: now,
        lastActiveAt: now
      });

      // Initialize capability tracking
      await this.initializeCapabilityTracking(profile);

      // Store in registry
      this.agents.set(profile.id, profile);
      return AgentProfileHelper.cloneProfile(profile);
    }
  }

  /**
   * Get agent profile by ID.
   *
   * @param agentId - ID of the agent to retrieve
   * @returns Agent profile
   * @throws RegistryError if agent not found
   */
  async getProfile(agentId: AgentId): Promise<AgentProfile> {
    if (stryMutAct_9fa48("175")) {
      {}
    } else {
      stryCov_9fa48("175");
      const profile = this.agents.get(agentId);
      if (stryMutAct_9fa48("178") ? false : stryMutAct_9fa48("177") ? true : stryMutAct_9fa48("176") ? profile : (stryCov_9fa48("176", "177", "178"), !profile)) {
        if (stryMutAct_9fa48("179")) {
          {}
        } else {
          stryCov_9fa48("179");
          throw new RegistryError(RegistryErrorType.AGENT_NOT_FOUND, stryMutAct_9fa48("180") ? `` : (stryCov_9fa48("180"), `Agent with ID ${agentId} not found`), stryMutAct_9fa48("181") ? {} : (stryCov_9fa48("181"), {
            agentId
          }));
        }
      }
      return AgentProfileHelper.cloneProfile(profile);
    }
  }

  /**
   * Query agents by capability and return sorted by performance.
   *
   * @param query - Query parameters with required capabilities
   * @returns Array of matching agents sorted by success rate (highest first)
   *
   * @remarks
   * Acceptance Criterion A2: Agents matching criteria returned sorted by performance history success rate
   * Performance Target: <50ms P95 latency
   */
  async getAgentsByCapability(query: AgentQuery): Promise<AgentQueryResult[]> {
    if (stryMutAct_9fa48("182")) {
      {}
    } else {
      stryCov_9fa48("182");
      const results: AgentQueryResult[] = stryMutAct_9fa48("183") ? ["Stryker was here"] : (stryCov_9fa48("183"), []);
      for (const profile of Array.from(this.agents.values())) {
        if (stryMutAct_9fa48("184")) {
          {}
        } else {
          stryCov_9fa48("184");
          // Check task type match
          if (stryMutAct_9fa48("187") ? false : stryMutAct_9fa48("186") ? true : stryMutAct_9fa48("185") ? profile.capabilities.taskTypes.includes(query.taskType) : (stryCov_9fa48("185", "186", "187"), !profile.capabilities.taskTypes.includes(query.taskType))) {
            if (stryMutAct_9fa48("188")) {
              {}
            } else {
              stryCov_9fa48("188");
              continue;
            }
          }

          // Check language requirements if specified
          if (stryMutAct_9fa48("191") ? query.languages || query.languages.length > 0 : stryMutAct_9fa48("190") ? false : stryMutAct_9fa48("189") ? true : (stryCov_9fa48("189", "190", "191"), query.languages && (stryMutAct_9fa48("194") ? query.languages.length <= 0 : stryMutAct_9fa48("193") ? query.languages.length >= 0 : stryMutAct_9fa48("192") ? true : (stryCov_9fa48("192", "193", "194"), query.languages.length > 0)))) {
            if (stryMutAct_9fa48("195")) {
              {}
            } else {
              stryCov_9fa48("195");
              const hasAllLanguages = stryMutAct_9fa48("196") ? query.languages.some(lang => profile.capabilities.languages.includes(lang)) : (stryCov_9fa48("196"), query.languages.every(stryMutAct_9fa48("197") ? () => undefined : (stryCov_9fa48("197"), lang => profile.capabilities.languages.includes(lang))));
              if (stryMutAct_9fa48("200") ? false : stryMutAct_9fa48("199") ? true : stryMutAct_9fa48("198") ? hasAllLanguages : (stryCov_9fa48("198", "199", "200"), !hasAllLanguages)) {
                if (stryMutAct_9fa48("201")) {
                  {}
                } else {
                  stryCov_9fa48("201");
                  continue;
                }
              }
            }
          }

          // Check specialization requirements if specified
          if (stryMutAct_9fa48("204") ? query.specializations || query.specializations.length > 0 : stryMutAct_9fa48("203") ? false : stryMutAct_9fa48("202") ? true : (stryCov_9fa48("202", "203", "204"), query.specializations && (stryMutAct_9fa48("207") ? query.specializations.length <= 0 : stryMutAct_9fa48("206") ? query.specializations.length >= 0 : stryMutAct_9fa48("205") ? true : (stryCov_9fa48("205", "206", "207"), query.specializations.length > 0)))) {
            if (stryMutAct_9fa48("208")) {
              {}
            } else {
              stryCov_9fa48("208");
              const hasAllSpecializations = stryMutAct_9fa48("209") ? query.specializations.some(spec => profile.capabilities.specializations.includes(spec)) : (stryCov_9fa48("209"), query.specializations.every(stryMutAct_9fa48("210") ? () => undefined : (stryCov_9fa48("210"), spec => profile.capabilities.specializations.includes(spec))));
              if (stryMutAct_9fa48("213") ? false : stryMutAct_9fa48("212") ? true : stryMutAct_9fa48("211") ? hasAllSpecializations : (stryCov_9fa48("211", "212", "213"), !hasAllSpecializations)) {
                if (stryMutAct_9fa48("214")) {
                  {}
                } else {
                  stryCov_9fa48("214");
                  continue;
                }
              }
            }
          }

          // Check utilization threshold if specified
          if (stryMutAct_9fa48("217") ? query.maxUtilization !== undefined || profile.currentLoad.utilizationPercent > query.maxUtilization : stryMutAct_9fa48("216") ? false : stryMutAct_9fa48("215") ? true : (stryCov_9fa48("215", "216", "217"), (stryMutAct_9fa48("219") ? query.maxUtilization === undefined : stryMutAct_9fa48("218") ? true : (stryCov_9fa48("218", "219"), query.maxUtilization !== undefined)) && (stryMutAct_9fa48("222") ? profile.currentLoad.utilizationPercent <= query.maxUtilization : stryMutAct_9fa48("221") ? profile.currentLoad.utilizationPercent >= query.maxUtilization : stryMutAct_9fa48("220") ? true : (stryCov_9fa48("220", "221", "222"), profile.currentLoad.utilizationPercent > query.maxUtilization)))) {
            if (stryMutAct_9fa48("223")) {
              {}
            } else {
              stryCov_9fa48("223");
              continue;
            }
          }

          // Check minimum success rate if specified
          if (stryMutAct_9fa48("226") ? query.minSuccessRate !== undefined || profile.performanceHistory.successRate < query.minSuccessRate : stryMutAct_9fa48("225") ? false : stryMutAct_9fa48("224") ? true : (stryCov_9fa48("224", "225", "226"), (stryMutAct_9fa48("228") ? query.minSuccessRate === undefined : stryMutAct_9fa48("227") ? true : (stryCov_9fa48("227", "228"), query.minSuccessRate !== undefined)) && (stryMutAct_9fa48("231") ? profile.performanceHistory.successRate >= query.minSuccessRate : stryMutAct_9fa48("230") ? profile.performanceHistory.successRate <= query.minSuccessRate : stryMutAct_9fa48("229") ? true : (stryCov_9fa48("229", "230", "231"), profile.performanceHistory.successRate < query.minSuccessRate)))) {
            if (stryMutAct_9fa48("232")) {
              {}
            } else {
              stryCov_9fa48("232");
              continue;
            }
          }

          // Calculate match score
          const matchScore = this.calculateMatchScore(profile, query);
          const matchReason = this.explainMatchScore(profile, query, matchScore);
          results.push(stryMutAct_9fa48("233") ? {} : (stryCov_9fa48("233"), {
            agent: AgentProfileHelper.cloneProfile(profile),
            matchScore,
            matchReason
          }));
        }
      }

      // Sort by success rate (highest first), then by match score
      return stryMutAct_9fa48("234") ? results : (stryCov_9fa48("234"), results.sort((a, b) => {
        if (stryMutAct_9fa48("235")) {
          {}
        } else {
          stryCov_9fa48("235");
          const successDiff = stryMutAct_9fa48("236") ? b.agent.performanceHistory.successRate + a.agent.performanceHistory.successRate : (stryCov_9fa48("236"), b.agent.performanceHistory.successRate - a.agent.performanceHistory.successRate);
          if (stryMutAct_9fa48("240") ? Math.abs(successDiff) <= 0.01 : stryMutAct_9fa48("239") ? Math.abs(successDiff) >= 0.01 : stryMutAct_9fa48("238") ? false : stryMutAct_9fa48("237") ? true : (stryCov_9fa48("237", "238", "239", "240"), Math.abs(successDiff) > 0.01)) {
            if (stryMutAct_9fa48("241")) {
              {}
            } else {
              stryCov_9fa48("241");
              return successDiff;
            }
          }
          return stryMutAct_9fa48("242") ? b.matchScore + a.matchScore : (stryCov_9fa48("242"), b.matchScore - a.matchScore);
        }
      }));
    }
  }

  /**
   * Update performance metrics for an agent after task completion.
   *
   * @param agentId - ID of the agent to update
   * @param metrics - Performance metrics from the completed task
   * @returns Updated agent profile
   * @throws RegistryError if agent not found or update fails
   *
   * @remarks
   * Acceptance Criterion A3: Agent's running average performance history computed and persisted
   * Performance Target: <30ms P95 latency
   * Invariant: Performance history updates are atomic and isolated per agent
   */
  async updatePerformance(agentId: AgentId, metrics: PerformanceMetrics): Promise<AgentProfile> {
    if (stryMutAct_9fa48("243")) {
      {}
    } else {
      stryCov_9fa48("243");
      const profile = this.agents.get(agentId);
      if (stryMutAct_9fa48("246") ? false : stryMutAct_9fa48("245") ? true : stryMutAct_9fa48("244") ? profile : (stryCov_9fa48("244", "245", "246"), !profile)) {
        if (stryMutAct_9fa48("247")) {
          {}
        } else {
          stryCov_9fa48("247");
          throw new RegistryError(RegistryErrorType.AGENT_NOT_FOUND, stryMutAct_9fa48("248") ? `` : (stryCov_9fa48("248"), `Agent with ID ${agentId} not found`), stryMutAct_9fa48("249") ? {} : (stryCov_9fa48("249"), {
            agentId
          }));
        }
      }
      try {
        if (stryMutAct_9fa48("250")) {
          {}
        } else {
          stryCov_9fa48("250");
          // Compute new running average (atomic operation)
          const newPerformanceHistory = AgentProfileHelper.updatePerformanceHistory(profile.performanceHistory, metrics);

          // Update profile with new performance history
          const updatedProfile: AgentProfile = stryMutAct_9fa48("251") ? {} : (stryCov_9fa48("251"), {
            ...profile,
            performanceHistory: newPerformanceHistory,
            lastActiveAt: new Date().toISOString()
          });

          // Atomically update in registry
          this.agents.set(agentId, updatedProfile);
          return AgentProfileHelper.cloneProfile(updatedProfile);
        }
      } catch (error) {
        if (stryMutAct_9fa48("252")) {
          {}
        } else {
          stryCov_9fa48("252");
          throw new RegistryError(RegistryErrorType.UPDATE_FAILED, stryMutAct_9fa48("253") ? `` : (stryCov_9fa48("253"), `Failed to update performance for agent ${agentId}: ${(error as Error).message}`), stryMutAct_9fa48("254") ? {} : (stryCov_9fa48("254"), {
            agentId,
            metrics,
            error
          }));
        }
      }
    }
  }

  /**
   * Update agent's current load (active and queued tasks).
   *
   * @param agentId - ID of the agent to update
   * @param activeTasks - New active tasks count
   * @param queuedTasks - New queued tasks count
   * @returns Updated agent profile
   * @throws RegistryError if agent not found
   */
  async updateLoad(agentId: AgentId, activeTasks: number, queuedTasks: number): Promise<AgentProfile> {
    if (stryMutAct_9fa48("255")) {
      {}
    } else {
      stryCov_9fa48("255");
      const profile = this.agents.get(agentId);
      if (stryMutAct_9fa48("258") ? false : stryMutAct_9fa48("257") ? true : stryMutAct_9fa48("256") ? profile : (stryCov_9fa48("256", "257", "258"), !profile)) {
        if (stryMutAct_9fa48("259")) {
          {}
        } else {
          stryCov_9fa48("259");
          throw new RegistryError(RegistryErrorType.AGENT_NOT_FOUND, stryMutAct_9fa48("260") ? `` : (stryCov_9fa48("260"), `Agent with ID ${agentId} not found`), stryMutAct_9fa48("261") ? {} : (stryCov_9fa48("261"), {
            agentId
          }));
        }
      }
      const utilizationPercent = stryMutAct_9fa48("262") ? activeTasks / this.maxConcurrentTasksPerAgent / 100 : (stryCov_9fa48("262"), (stryMutAct_9fa48("263") ? activeTasks * this.maxConcurrentTasksPerAgent : (stryCov_9fa48("263"), activeTasks / this.maxConcurrentTasksPerAgent)) * 100);
      const updatedProfile: AgentProfile = stryMutAct_9fa48("264") ? {} : (stryCov_9fa48("264"), {
        ...profile,
        currentLoad: stryMutAct_9fa48("265") ? {} : (stryCov_9fa48("265"), {
          activeTasks,
          queuedTasks,
          utilizationPercent: stryMutAct_9fa48("266") ? Math.max(100, utilizationPercent) : (stryCov_9fa48("266"), Math.min(100, utilizationPercent))
        }),
        lastActiveAt: new Date().toISOString()
      });
      this.agents.set(agentId, updatedProfile);
      return AgentProfileHelper.cloneProfile(updatedProfile);
    }
  }

  /**
   * Get registry statistics.
   *
   * @returns Current registry stats
   */
  async getStats(): Promise<RegistryStats> {
    if (stryMutAct_9fa48("267")) {
      {}
    } else {
      stryCov_9fa48("267");
      const allAgents = Array.from(this.agents.values());
      const activeAgents = stryMutAct_9fa48("268") ? allAgents : (stryCov_9fa48("268"), allAgents.filter(stryMutAct_9fa48("269") ? () => undefined : (stryCov_9fa48("269"), a => stryMutAct_9fa48("273") ? a.currentLoad.activeTasks <= 0 : stryMutAct_9fa48("272") ? a.currentLoad.activeTasks >= 0 : stryMutAct_9fa48("271") ? false : stryMutAct_9fa48("270") ? true : (stryCov_9fa48("270", "271", "272", "273"), a.currentLoad.activeTasks > 0))));
      const idleAgents = stryMutAct_9fa48("274") ? allAgents : (stryCov_9fa48("274"), allAgents.filter(stryMutAct_9fa48("275") ? () => undefined : (stryCov_9fa48("275"), a => stryMutAct_9fa48("278") ? a.currentLoad.activeTasks !== 0 : stryMutAct_9fa48("277") ? false : stryMutAct_9fa48("276") ? true : (stryCov_9fa48("276", "277", "278"), a.currentLoad.activeTasks === 0))));
      const totalUtilization = allAgents.reduce(stryMutAct_9fa48("279") ? () => undefined : (stryCov_9fa48("279"), (sum, a) => stryMutAct_9fa48("280") ? sum - a.currentLoad.utilizationPercent : (stryCov_9fa48("280"), sum + a.currentLoad.utilizationPercent)), 0);
      const averageUtilization = (stryMutAct_9fa48("284") ? allAgents.length <= 0 : stryMutAct_9fa48("283") ? allAgents.length >= 0 : stryMutAct_9fa48("282") ? false : stryMutAct_9fa48("281") ? true : (stryCov_9fa48("281", "282", "283", "284"), allAgents.length > 0)) ? stryMutAct_9fa48("285") ? totalUtilization * allAgents.length : (stryCov_9fa48("285"), totalUtilization / allAgents.length) : 0;
      const totalSuccessRate = allAgents.reduce(stryMutAct_9fa48("286") ? () => undefined : (stryCov_9fa48("286"), (sum, a) => stryMutAct_9fa48("287") ? sum - a.performanceHistory.successRate : (stryCov_9fa48("287"), sum + a.performanceHistory.successRate)), 0);
      const averageSuccessRate = (stryMutAct_9fa48("291") ? allAgents.length <= 0 : stryMutAct_9fa48("290") ? allAgents.length >= 0 : stryMutAct_9fa48("289") ? false : stryMutAct_9fa48("288") ? true : (stryCov_9fa48("288", "289", "290", "291"), allAgents.length > 0)) ? stryMutAct_9fa48("292") ? totalSuccessRate * allAgents.length : (stryCov_9fa48("292"), totalSuccessRate / allAgents.length) : 0;
      return stryMutAct_9fa48("293") ? {} : (stryCov_9fa48("293"), {
        totalAgents: allAgents.length,
        activeAgents: activeAgents.length,
        idleAgents: idleAgents.length,
        averageUtilization,
        averageSuccessRate,
        lastUpdated: new Date().toISOString()
      });
    }
  }

  /**
   * Remove an agent from the registry.
   *
   * @param agentId - ID of the agent to remove
   * @returns True if agent was removed
   */
  async unregisterAgent(agentId: AgentId): Promise<boolean> {
    if (stryMutAct_9fa48("294")) {
      {}
    } else {
      stryCov_9fa48("294");
      return this.agents.delete(agentId);
    }
  }

  /**
   * Initialize capability tracking for a new agent.
   */
  private async initializeCapabilityTracking(
  // eslint-disable-next-line @typescript-eslint/no-unused-vars, no-unused-vars
  _profile: AgentProfile): Promise<void> {
    // Capability tracking initialization
    // In production, this would set up monitoring for capability usage
    // and initialize any external tracking systems
    // For now, this is a no-op, but provides extension point
  }

  /**
   * Calculate match score for query result ranking.
   *
   * @param profile - Agent profile
   * @param query - Query parameters
   * @returns Match score (0.0 - 1.0)
   */
  private calculateMatchScore(profile: AgentProfile, query: AgentQuery): number {
    if (stryMutAct_9fa48("295")) {
      {}
    } else {
      stryCov_9fa48("295");
      let score = 0.0;
      let weights = 0.0;

      // Task type match (required, so always contributes)
      stryMutAct_9fa48("296") ? score -= 0.3 : (stryCov_9fa48("296"), score += 0.3);
      stryMutAct_9fa48("297") ? weights -= 0.3 : (stryCov_9fa48("297"), weights += 0.3);

      // Language matches (if specified)
      if (stryMutAct_9fa48("300") ? query.languages || query.languages.length > 0 : stryMutAct_9fa48("299") ? false : stryMutAct_9fa48("298") ? true : (stryCov_9fa48("298", "299", "300"), query.languages && (stryMutAct_9fa48("303") ? query.languages.length <= 0 : stryMutAct_9fa48("302") ? query.languages.length >= 0 : stryMutAct_9fa48("301") ? true : (stryCov_9fa48("301", "302", "303"), query.languages.length > 0)))) {
        if (stryMutAct_9fa48("304")) {
          {}
        } else {
          stryCov_9fa48("304");
          const matchedLanguages = stryMutAct_9fa48("305") ? query.languages.length : (stryCov_9fa48("305"), query.languages.filter(stryMutAct_9fa48("306") ? () => undefined : (stryCov_9fa48("306"), lang => profile.capabilities.languages.includes(lang))).length);
          stryMutAct_9fa48("307") ? score -= matchedLanguages / query.languages.length * 0.3 : (stryCov_9fa48("307"), score += stryMutAct_9fa48("308") ? matchedLanguages / query.languages.length / 0.3 : (stryCov_9fa48("308"), (stryMutAct_9fa48("309") ? matchedLanguages * query.languages.length : (stryCov_9fa48("309"), matchedLanguages / query.languages.length)) * 0.3));
          stryMutAct_9fa48("310") ? weights -= 0.3 : (stryCov_9fa48("310"), weights += 0.3);
        }
      }

      // Specialization matches (if specified)
      if (stryMutAct_9fa48("313") ? query.specializations || query.specializations.length > 0 : stryMutAct_9fa48("312") ? false : stryMutAct_9fa48("311") ? true : (stryCov_9fa48("311", "312", "313"), query.specializations && (stryMutAct_9fa48("316") ? query.specializations.length <= 0 : stryMutAct_9fa48("315") ? query.specializations.length >= 0 : stryMutAct_9fa48("314") ? true : (stryCov_9fa48("314", "315", "316"), query.specializations.length > 0)))) {
        if (stryMutAct_9fa48("317")) {
          {}
        } else {
          stryCov_9fa48("317");
          const matchedSpecs = stryMutAct_9fa48("318") ? query.specializations.length : (stryCov_9fa48("318"), query.specializations.filter(stryMutAct_9fa48("319") ? () => undefined : (stryCov_9fa48("319"), spec => profile.capabilities.specializations.includes(spec))).length);
          stryMutAct_9fa48("320") ? score -= matchedSpecs / query.specializations.length * 0.2 : (stryCov_9fa48("320"), score += stryMutAct_9fa48("321") ? matchedSpecs / query.specializations.length / 0.2 : (stryCov_9fa48("321"), (stryMutAct_9fa48("322") ? matchedSpecs * query.specializations.length : (stryCov_9fa48("322"), matchedSpecs / query.specializations.length)) * 0.2));
          stryMutAct_9fa48("323") ? weights -= 0.2 : (stryCov_9fa48("323"), weights += 0.2);
        }
      }

      // Performance bonus
      stryMutAct_9fa48("324") ? score -= profile.performanceHistory.successRate * 0.2 : (stryCov_9fa48("324"), score += stryMutAct_9fa48("325") ? profile.performanceHistory.successRate / 0.2 : (stryCov_9fa48("325"), profile.performanceHistory.successRate * 0.2));
      stryMutAct_9fa48("326") ? weights -= 0.2 : (stryCov_9fa48("326"), weights += 0.2);
      return (stryMutAct_9fa48("330") ? weights <= 0 : stryMutAct_9fa48("329") ? weights >= 0 : stryMutAct_9fa48("328") ? false : stryMutAct_9fa48("327") ? true : (stryCov_9fa48("327", "328", "329", "330"), weights > 0)) ? stryMutAct_9fa48("331") ? score * weights : (stryCov_9fa48("331"), score / weights) : 0;
    }
  }

  /**
   * Generate human-readable explanation of match score.
   *
   * @param profile - Agent profile
   * @param query - Query parameters
   * @returns Explanation string
   */
  private explainMatchScore(profile: AgentProfile, query: AgentQuery,
  // eslint-disable-next-line @typescript-eslint/no-unused-vars, no-unused-vars
  _score: number): string {
    if (stryMutAct_9fa48("332")) {
      {}
    } else {
      stryCov_9fa48("332");
      const reasons: string[] = stryMutAct_9fa48("333") ? ["Stryker was here"] : (stryCov_9fa48("333"), []);
      reasons.push(stryMutAct_9fa48("334") ? `` : (stryCov_9fa48("334"), `Supports ${query.taskType}`));
      if (stryMutAct_9fa48("337") ? query.languages || query.languages.length > 0 : stryMutAct_9fa48("336") ? false : stryMutAct_9fa48("335") ? true : (stryCov_9fa48("335", "336", "337"), query.languages && (stryMutAct_9fa48("340") ? query.languages.length <= 0 : stryMutAct_9fa48("339") ? query.languages.length >= 0 : stryMutAct_9fa48("338") ? true : (stryCov_9fa48("338", "339", "340"), query.languages.length > 0)))) {
        if (stryMutAct_9fa48("341")) {
          {}
        } else {
          stryCov_9fa48("341");
          reasons.push(stryMutAct_9fa48("342") ? `` : (stryCov_9fa48("342"), `Languages: ${query.languages.join(stryMutAct_9fa48("343") ? "" : (stryCov_9fa48("343"), ", "))}`));
        }
      }
      if (stryMutAct_9fa48("346") ? query.specializations || query.specializations.length > 0 : stryMutAct_9fa48("345") ? false : stryMutAct_9fa48("344") ? true : (stryCov_9fa48("344", "345", "346"), query.specializations && (stryMutAct_9fa48("349") ? query.specializations.length <= 0 : stryMutAct_9fa48("348") ? query.specializations.length >= 0 : stryMutAct_9fa48("347") ? true : (stryCov_9fa48("347", "348", "349"), query.specializations.length > 0)))) {
        if (stryMutAct_9fa48("350")) {
          {}
        } else {
          stryCov_9fa48("350");
          reasons.push(stryMutAct_9fa48("351") ? `` : (stryCov_9fa48("351"), `Specializations: ${query.specializations.join(stryMutAct_9fa48("352") ? "" : (stryCov_9fa48("352"), ", "))}`));
        }
      }
      reasons.push(stryMutAct_9fa48("353") ? `` : (stryCov_9fa48("353"), `${(stryMutAct_9fa48("354") ? profile.performanceHistory.successRate / 100 : (stryCov_9fa48("354"), profile.performanceHistory.successRate * 100)).toFixed(1)}% success rate`));
      reasons.push(stryMutAct_9fa48("355") ? `` : (stryCov_9fa48("355"), `${profile.currentLoad.utilizationPercent.toFixed(0)}% utilized`));
      return reasons.join(stryMutAct_9fa48("356") ? "" : (stryCov_9fa48("356"), "; "));
    }
  }

  /**
   * Start automatic cleanup of stale agents.
   */
  private startAutoCleanup(): void {
    if (stryMutAct_9fa48("357")) {
      {}
    } else {
      stryCov_9fa48("357");
      this.cleanupTimer = setInterval(() => {
        if (stryMutAct_9fa48("358")) {
          {}
        } else {
          stryCov_9fa48("358");
          this.cleanupStaleAgents();
        }
      }, this.config.cleanupIntervalMs);
    }
  }

  /**
   * Clean up stale agents (inactive beyond threshold).
   */
  private cleanupStaleAgents(): void {
    if (stryMutAct_9fa48("359")) {
      {}
    } else {
      stryCov_9fa48("359");
      const now = new Date().toISOString();
      const staleAgents: AgentId[] = stryMutAct_9fa48("360") ? ["Stryker was here"] : (stryCov_9fa48("360"), []);
      const agents = Array.from(this.agents.entries());
      for (const [agentId, profile] of agents) {
        if (stryMutAct_9fa48("361")) {
          {}
        } else {
          stryCov_9fa48("361");
          if (stryMutAct_9fa48("363") ? false : stryMutAct_9fa48("362") ? true : (stryCov_9fa48("362", "363"), AgentProfileHelper.isStale(profile, this.config.staleAgentThresholdMs, now))) {
            if (stryMutAct_9fa48("364")) {
              {}
            } else {
              stryCov_9fa48("364");
              staleAgents.push(agentId);
            }
          }
        }
      }
      for (const agentId of staleAgents) {
        if (stryMutAct_9fa48("365")) {
          {}
        } else {
          stryCov_9fa48("365");
          this.agents.delete(agentId);
        }
      }
    }
  }

  /**
   * Shutdown the registry manager and cleanup resources.
   */
  async shutdown(): Promise<void> {
    if (stryMutAct_9fa48("366")) {
      {}
    } else {
      stryCov_9fa48("366");
      if (stryMutAct_9fa48("368") ? false : stryMutAct_9fa48("367") ? true : (stryCov_9fa48("367", "368"), this.cleanupTimer)) {
        if (stryMutAct_9fa48("369")) {
          {}
        } else {
          stryCov_9fa48("369");
          clearInterval(this.cleanupTimer);
        }
      }
    }
  }
}