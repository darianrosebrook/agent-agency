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
const DEFAULT_CONFIG: AgentRegistryConfig = stryMutAct_9fa48("410") ? {} : (stryCov_9fa48("410"), {
  maxAgents: 1000,
  staleAgentThresholdMs: stryMutAct_9fa48("411") ? 24 * 60 * 60 / 1000 : (stryCov_9fa48("411"), (stryMutAct_9fa48("412") ? 24 * 60 / 60 : (stryCov_9fa48("412"), (stryMutAct_9fa48("413") ? 24 / 60 : (stryCov_9fa48("413"), 24 * 60)) * 60)) * 1000),
  // 24 hours
  enableAutoCleanup: stryMutAct_9fa48("414") ? false : (stryCov_9fa48("414"), true),
  cleanupIntervalMs: stryMutAct_9fa48("415") ? 60 * 60 / 1000 : (stryCov_9fa48("415"), (stryMutAct_9fa48("416") ? 60 / 60 : (stryCov_9fa48("416"), 60 * 60)) * 1000) // 1 hour
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
    if (stryMutAct_9fa48("417")) {
      {}
    } else {
      stryCov_9fa48("417");
      this.agents = new Map();
      this.config = stryMutAct_9fa48("418") ? {} : (stryCov_9fa48("418"), {
        ...DEFAULT_CONFIG,
        ...config
      });
      if (stryMutAct_9fa48("420") ? false : stryMutAct_9fa48("419") ? true : (stryCov_9fa48("419", "420"), this.config.enableAutoCleanup)) {
        if (stryMutAct_9fa48("421")) {
          {}
        } else {
          stryCov_9fa48("421");
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
    if (stryMutAct_9fa48("422")) {
      {}
    } else {
      stryCov_9fa48("422");
      // Validate required fields
      AgentProfileHelper.validateProfile(agent);
      if (stryMutAct_9fa48("425") ? false : stryMutAct_9fa48("424") ? true : stryMutAct_9fa48("423") ? agent.id : (stryCov_9fa48("423", "424", "425"), !agent.id)) {
        if (stryMutAct_9fa48("426")) {
          {}
        } else {
          stryCov_9fa48("426");
          throw new RegistryError(RegistryErrorType.INVALID_AGENT_DATA, stryMutAct_9fa48("427") ? "" : (stryCov_9fa48("427"), "Agent ID is required"));
        }
      }

      // Check if agent already exists
      if (stryMutAct_9fa48("429") ? false : stryMutAct_9fa48("428") ? true : (stryCov_9fa48("428", "429"), this.agents.has(agent.id))) {
        if (stryMutAct_9fa48("430")) {
          {}
        } else {
          stryCov_9fa48("430");
          throw new RegistryError(RegistryErrorType.AGENT_ALREADY_EXISTS, stryMutAct_9fa48("431") ? `` : (stryCov_9fa48("431"), `Agent with ID ${agent.id} already exists`), stryMutAct_9fa48("432") ? {} : (stryCov_9fa48("432"), {
            agentId: agent.id
          }));
        }
      }

      // Check registry capacity
      if (stryMutAct_9fa48("436") ? this.agents.size < this.config.maxAgents : stryMutAct_9fa48("435") ? this.agents.size > this.config.maxAgents : stryMutAct_9fa48("434") ? false : stryMutAct_9fa48("433") ? true : (stryCov_9fa48("433", "434", "435", "436"), this.agents.size >= this.config.maxAgents)) {
        if (stryMutAct_9fa48("437")) {
          {}
        } else {
          stryCov_9fa48("437");
          throw new RegistryError(RegistryErrorType.REGISTRY_FULL, stryMutAct_9fa48("438") ? `` : (stryCov_9fa48("438"), `Registry is full (max: ${this.config.maxAgents} agents)`), stryMutAct_9fa48("439") ? {} : (stryCov_9fa48("439"), {
            maxAgents: this.config.maxAgents,
            currentSize: this.agents.size
          }));
        }
      }

      // Create complete profile with defaults
      const now = new Date().toISOString();
      const profile: AgentProfile = stryMutAct_9fa48("440") ? {} : (stryCov_9fa48("440"), {
        id: agent.id,
        name: agent.name!,
        modelFamily: agent.modelFamily!,
        capabilities: agent.capabilities!,
        performanceHistory: stryMutAct_9fa48("441") ? agent.performanceHistory && AgentProfileHelper.createInitialPerformanceHistory() : (stryCov_9fa48("441"), agent.performanceHistory ?? AgentProfileHelper.createInitialPerformanceHistory()),
        currentLoad: stryMutAct_9fa48("442") ? agent.currentLoad && AgentProfileHelper.createInitialLoad() : (stryCov_9fa48("442"), agent.currentLoad ?? AgentProfileHelper.createInitialLoad()),
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
    if (stryMutAct_9fa48("443")) {
      {}
    } else {
      stryCov_9fa48("443");
      const profile = this.agents.get(agentId);
      if (stryMutAct_9fa48("446") ? false : stryMutAct_9fa48("445") ? true : stryMutAct_9fa48("444") ? profile : (stryCov_9fa48("444", "445", "446"), !profile)) {
        if (stryMutAct_9fa48("447")) {
          {}
        } else {
          stryCov_9fa48("447");
          throw new RegistryError(RegistryErrorType.AGENT_NOT_FOUND, stryMutAct_9fa48("448") ? `` : (stryCov_9fa48("448"), `Agent with ID ${agentId} not found`), stryMutAct_9fa48("449") ? {} : (stryCov_9fa48("449"), {
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
    if (stryMutAct_9fa48("450")) {
      {}
    } else {
      stryCov_9fa48("450");
      const results: AgentQueryResult[] = stryMutAct_9fa48("451") ? ["Stryker was here"] : (stryCov_9fa48("451"), []);
      for (const profile of Array.from(this.agents.values())) {
        if (stryMutAct_9fa48("452")) {
          {}
        } else {
          stryCov_9fa48("452");
          // Check task type match
          if (stryMutAct_9fa48("455") ? false : stryMutAct_9fa48("454") ? true : stryMutAct_9fa48("453") ? profile.capabilities.taskTypes.includes(query.taskType) : (stryCov_9fa48("453", "454", "455"), !profile.capabilities.taskTypes.includes(query.taskType))) {
            if (stryMutAct_9fa48("456")) {
              {}
            } else {
              stryCov_9fa48("456");
              continue;
            }
          }

          // Check language requirements if specified
          if (stryMutAct_9fa48("459") ? query.languages || query.languages.length > 0 : stryMutAct_9fa48("458") ? false : stryMutAct_9fa48("457") ? true : (stryCov_9fa48("457", "458", "459"), query.languages && (stryMutAct_9fa48("462") ? query.languages.length <= 0 : stryMutAct_9fa48("461") ? query.languages.length >= 0 : stryMutAct_9fa48("460") ? true : (stryCov_9fa48("460", "461", "462"), query.languages.length > 0)))) {
            if (stryMutAct_9fa48("463")) {
              {}
            } else {
              stryCov_9fa48("463");
              const hasAllLanguages = stryMutAct_9fa48("464") ? query.languages.some(lang => profile.capabilities.languages.includes(lang)) : (stryCov_9fa48("464"), query.languages.every(stryMutAct_9fa48("465") ? () => undefined : (stryCov_9fa48("465"), lang => profile.capabilities.languages.includes(lang))));
              if (stryMutAct_9fa48("468") ? false : stryMutAct_9fa48("467") ? true : stryMutAct_9fa48("466") ? hasAllLanguages : (stryCov_9fa48("466", "467", "468"), !hasAllLanguages)) {
                if (stryMutAct_9fa48("469")) {
                  {}
                } else {
                  stryCov_9fa48("469");
                  continue;
                }
              }
            }
          }

          // Check specialization requirements if specified
          if (stryMutAct_9fa48("472") ? query.specializations || query.specializations.length > 0 : stryMutAct_9fa48("471") ? false : stryMutAct_9fa48("470") ? true : (stryCov_9fa48("470", "471", "472"), query.specializations && (stryMutAct_9fa48("475") ? query.specializations.length <= 0 : stryMutAct_9fa48("474") ? query.specializations.length >= 0 : stryMutAct_9fa48("473") ? true : (stryCov_9fa48("473", "474", "475"), query.specializations.length > 0)))) {
            if (stryMutAct_9fa48("476")) {
              {}
            } else {
              stryCov_9fa48("476");
              const hasAllSpecializations = stryMutAct_9fa48("477") ? query.specializations.some(spec => profile.capabilities.specializations.includes(spec)) : (stryCov_9fa48("477"), query.specializations.every(stryMutAct_9fa48("478") ? () => undefined : (stryCov_9fa48("478"), spec => profile.capabilities.specializations.includes(spec))));
              if (stryMutAct_9fa48("481") ? false : stryMutAct_9fa48("480") ? true : stryMutAct_9fa48("479") ? hasAllSpecializations : (stryCov_9fa48("479", "480", "481"), !hasAllSpecializations)) {
                if (stryMutAct_9fa48("482")) {
                  {}
                } else {
                  stryCov_9fa48("482");
                  continue;
                }
              }
            }
          }

          // Check utilization threshold if specified
          if (stryMutAct_9fa48("485") ? query.maxUtilization !== undefined || profile.currentLoad.utilizationPercent > query.maxUtilization : stryMutAct_9fa48("484") ? false : stryMutAct_9fa48("483") ? true : (stryCov_9fa48("483", "484", "485"), (stryMutAct_9fa48("487") ? query.maxUtilization === undefined : stryMutAct_9fa48("486") ? true : (stryCov_9fa48("486", "487"), query.maxUtilization !== undefined)) && (stryMutAct_9fa48("490") ? profile.currentLoad.utilizationPercent <= query.maxUtilization : stryMutAct_9fa48("489") ? profile.currentLoad.utilizationPercent >= query.maxUtilization : stryMutAct_9fa48("488") ? true : (stryCov_9fa48("488", "489", "490"), profile.currentLoad.utilizationPercent > query.maxUtilization)))) {
            if (stryMutAct_9fa48("491")) {
              {}
            } else {
              stryCov_9fa48("491");
              continue;
            }
          }

          // Check minimum success rate if specified
          if (stryMutAct_9fa48("494") ? query.minSuccessRate !== undefined || profile.performanceHistory.successRate < query.minSuccessRate : stryMutAct_9fa48("493") ? false : stryMutAct_9fa48("492") ? true : (stryCov_9fa48("492", "493", "494"), (stryMutAct_9fa48("496") ? query.minSuccessRate === undefined : stryMutAct_9fa48("495") ? true : (stryCov_9fa48("495", "496"), query.minSuccessRate !== undefined)) && (stryMutAct_9fa48("499") ? profile.performanceHistory.successRate >= query.minSuccessRate : stryMutAct_9fa48("498") ? profile.performanceHistory.successRate <= query.minSuccessRate : stryMutAct_9fa48("497") ? true : (stryCov_9fa48("497", "498", "499"), profile.performanceHistory.successRate < query.minSuccessRate)))) {
            if (stryMutAct_9fa48("500")) {
              {}
            } else {
              stryCov_9fa48("500");
              continue;
            }
          }

          // Calculate match score
          const matchScore = this.calculateMatchScore(profile, query);
          const matchReason = this.explainMatchScore(profile, query, matchScore);
          results.push(stryMutAct_9fa48("501") ? {} : (stryCov_9fa48("501"), {
            agent: AgentProfileHelper.cloneProfile(profile),
            matchScore,
            matchReason
          }));
        }
      }

      // Sort by success rate (highest first), then by match score
      return stryMutAct_9fa48("502") ? results : (stryCov_9fa48("502"), results.sort((a, b) => {
        if (stryMutAct_9fa48("503")) {
          {}
        } else {
          stryCov_9fa48("503");
          const successDiff = stryMutAct_9fa48("504") ? b.agent.performanceHistory.successRate + a.agent.performanceHistory.successRate : (stryCov_9fa48("504"), b.agent.performanceHistory.successRate - a.agent.performanceHistory.successRate);
          if (stryMutAct_9fa48("508") ? Math.abs(successDiff) <= 0.01 : stryMutAct_9fa48("507") ? Math.abs(successDiff) >= 0.01 : stryMutAct_9fa48("506") ? false : stryMutAct_9fa48("505") ? true : (stryCov_9fa48("505", "506", "507", "508"), Math.abs(successDiff) > 0.01)) {
            if (stryMutAct_9fa48("509")) {
              {}
            } else {
              stryCov_9fa48("509");
              return successDiff;
            }
          }
          return stryMutAct_9fa48("510") ? b.matchScore + a.matchScore : (stryCov_9fa48("510"), b.matchScore - a.matchScore);
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
    if (stryMutAct_9fa48("511")) {
      {}
    } else {
      stryCov_9fa48("511");
      const profile = this.agents.get(agentId);
      if (stryMutAct_9fa48("514") ? false : stryMutAct_9fa48("513") ? true : stryMutAct_9fa48("512") ? profile : (stryCov_9fa48("512", "513", "514"), !profile)) {
        if (stryMutAct_9fa48("515")) {
          {}
        } else {
          stryCov_9fa48("515");
          throw new RegistryError(RegistryErrorType.AGENT_NOT_FOUND, stryMutAct_9fa48("516") ? `` : (stryCov_9fa48("516"), `Agent with ID ${agentId} not found`), stryMutAct_9fa48("517") ? {} : (stryCov_9fa48("517"), {
            agentId
          }));
        }
      }
      try {
        if (stryMutAct_9fa48("518")) {
          {}
        } else {
          stryCov_9fa48("518");
          // Compute new running average (atomic operation)
          const newPerformanceHistory = AgentProfileHelper.updatePerformanceHistory(profile.performanceHistory, metrics);

          // Update profile with new performance history
          const updatedProfile: AgentProfile = stryMutAct_9fa48("519") ? {} : (stryCov_9fa48("519"), {
            ...profile,
            performanceHistory: newPerformanceHistory,
            lastActiveAt: new Date().toISOString()
          });

          // Atomically update in registry
          this.agents.set(agentId, updatedProfile);
          return AgentProfileHelper.cloneProfile(updatedProfile);
        }
      } catch (error) {
        if (stryMutAct_9fa48("520")) {
          {}
        } else {
          stryCov_9fa48("520");
          throw new RegistryError(RegistryErrorType.UPDATE_FAILED, stryMutAct_9fa48("521") ? `` : (stryCov_9fa48("521"), `Failed to update performance for agent ${agentId}: ${(error as Error).message}`), stryMutAct_9fa48("522") ? {} : (stryCov_9fa48("522"), {
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
    if (stryMutAct_9fa48("523")) {
      {}
    } else {
      stryCov_9fa48("523");
      const profile = this.agents.get(agentId);
      if (stryMutAct_9fa48("526") ? false : stryMutAct_9fa48("525") ? true : stryMutAct_9fa48("524") ? profile : (stryCov_9fa48("524", "525", "526"), !profile)) {
        if (stryMutAct_9fa48("527")) {
          {}
        } else {
          stryCov_9fa48("527");
          throw new RegistryError(RegistryErrorType.AGENT_NOT_FOUND, stryMutAct_9fa48("528") ? `` : (stryCov_9fa48("528"), `Agent with ID ${agentId} not found`), stryMutAct_9fa48("529") ? {} : (stryCov_9fa48("529"), {
            agentId
          }));
        }
      }
      const utilizationPercent = stryMutAct_9fa48("530") ? activeTasks / this.maxConcurrentTasksPerAgent / 100 : (stryCov_9fa48("530"), (stryMutAct_9fa48("531") ? activeTasks * this.maxConcurrentTasksPerAgent : (stryCov_9fa48("531"), activeTasks / this.maxConcurrentTasksPerAgent)) * 100);
      const updatedProfile: AgentProfile = stryMutAct_9fa48("532") ? {} : (stryCov_9fa48("532"), {
        ...profile,
        currentLoad: stryMutAct_9fa48("533") ? {} : (stryCov_9fa48("533"), {
          activeTasks,
          queuedTasks,
          utilizationPercent: stryMutAct_9fa48("534") ? Math.max(100, utilizationPercent) : (stryCov_9fa48("534"), Math.min(100, utilizationPercent))
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
    if (stryMutAct_9fa48("535")) {
      {}
    } else {
      stryCov_9fa48("535");
      const allAgents = Array.from(this.agents.values());
      const activeAgents = stryMutAct_9fa48("536") ? allAgents : (stryCov_9fa48("536"), allAgents.filter(stryMutAct_9fa48("537") ? () => undefined : (stryCov_9fa48("537"), a => stryMutAct_9fa48("541") ? a.currentLoad.activeTasks <= 0 : stryMutAct_9fa48("540") ? a.currentLoad.activeTasks >= 0 : stryMutAct_9fa48("539") ? false : stryMutAct_9fa48("538") ? true : (stryCov_9fa48("538", "539", "540", "541"), a.currentLoad.activeTasks > 0))));
      const idleAgents = stryMutAct_9fa48("542") ? allAgents : (stryCov_9fa48("542"), allAgents.filter(stryMutAct_9fa48("543") ? () => undefined : (stryCov_9fa48("543"), a => stryMutAct_9fa48("546") ? a.currentLoad.activeTasks !== 0 : stryMutAct_9fa48("545") ? false : stryMutAct_9fa48("544") ? true : (stryCov_9fa48("544", "545", "546"), a.currentLoad.activeTasks === 0))));
      const totalUtilization = allAgents.reduce(stryMutAct_9fa48("547") ? () => undefined : (stryCov_9fa48("547"), (sum, a) => stryMutAct_9fa48("548") ? sum - a.currentLoad.utilizationPercent : (stryCov_9fa48("548"), sum + a.currentLoad.utilizationPercent)), 0);
      const averageUtilization = (stryMutAct_9fa48("552") ? allAgents.length <= 0 : stryMutAct_9fa48("551") ? allAgents.length >= 0 : stryMutAct_9fa48("550") ? false : stryMutAct_9fa48("549") ? true : (stryCov_9fa48("549", "550", "551", "552"), allAgents.length > 0)) ? stryMutAct_9fa48("553") ? totalUtilization * allAgents.length : (stryCov_9fa48("553"), totalUtilization / allAgents.length) : 0;
      const totalSuccessRate = allAgents.reduce(stryMutAct_9fa48("554") ? () => undefined : (stryCov_9fa48("554"), (sum, a) => stryMutAct_9fa48("555") ? sum - a.performanceHistory.successRate : (stryCov_9fa48("555"), sum + a.performanceHistory.successRate)), 0);
      const averageSuccessRate = (stryMutAct_9fa48("559") ? allAgents.length <= 0 : stryMutAct_9fa48("558") ? allAgents.length >= 0 : stryMutAct_9fa48("557") ? false : stryMutAct_9fa48("556") ? true : (stryCov_9fa48("556", "557", "558", "559"), allAgents.length > 0)) ? stryMutAct_9fa48("560") ? totalSuccessRate * allAgents.length : (stryCov_9fa48("560"), totalSuccessRate / allAgents.length) : 0;
      return stryMutAct_9fa48("561") ? {} : (stryCov_9fa48("561"), {
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
    if (stryMutAct_9fa48("562")) {
      {}
    } else {
      stryCov_9fa48("562");
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
    if (stryMutAct_9fa48("563")) {
      {}
    } else {
      stryCov_9fa48("563");
      let score = 0.0;
      let weights = 0.0;

      // Task type match (required, so always contributes)
      stryMutAct_9fa48("564") ? score -= 0.3 : (stryCov_9fa48("564"), score += 0.3);
      stryMutAct_9fa48("565") ? weights -= 0.3 : (stryCov_9fa48("565"), weights += 0.3);

      // Language matches (if specified)
      if (stryMutAct_9fa48("568") ? query.languages || query.languages.length > 0 : stryMutAct_9fa48("567") ? false : stryMutAct_9fa48("566") ? true : (stryCov_9fa48("566", "567", "568"), query.languages && (stryMutAct_9fa48("571") ? query.languages.length <= 0 : stryMutAct_9fa48("570") ? query.languages.length >= 0 : stryMutAct_9fa48("569") ? true : (stryCov_9fa48("569", "570", "571"), query.languages.length > 0)))) {
        if (stryMutAct_9fa48("572")) {
          {}
        } else {
          stryCov_9fa48("572");
          const matchedLanguages = stryMutAct_9fa48("573") ? query.languages.length : (stryCov_9fa48("573"), query.languages.filter(stryMutAct_9fa48("574") ? () => undefined : (stryCov_9fa48("574"), lang => profile.capabilities.languages.includes(lang))).length);
          stryMutAct_9fa48("575") ? score -= matchedLanguages / query.languages.length * 0.3 : (stryCov_9fa48("575"), score += stryMutAct_9fa48("576") ? matchedLanguages / query.languages.length / 0.3 : (stryCov_9fa48("576"), (stryMutAct_9fa48("577") ? matchedLanguages * query.languages.length : (stryCov_9fa48("577"), matchedLanguages / query.languages.length)) * 0.3));
          stryMutAct_9fa48("578") ? weights -= 0.3 : (stryCov_9fa48("578"), weights += 0.3);
        }
      }

      // Specialization matches (if specified)
      if (stryMutAct_9fa48("581") ? query.specializations || query.specializations.length > 0 : stryMutAct_9fa48("580") ? false : stryMutAct_9fa48("579") ? true : (stryCov_9fa48("579", "580", "581"), query.specializations && (stryMutAct_9fa48("584") ? query.specializations.length <= 0 : stryMutAct_9fa48("583") ? query.specializations.length >= 0 : stryMutAct_9fa48("582") ? true : (stryCov_9fa48("582", "583", "584"), query.specializations.length > 0)))) {
        if (stryMutAct_9fa48("585")) {
          {}
        } else {
          stryCov_9fa48("585");
          const matchedSpecs = stryMutAct_9fa48("586") ? query.specializations.length : (stryCov_9fa48("586"), query.specializations.filter(stryMutAct_9fa48("587") ? () => undefined : (stryCov_9fa48("587"), spec => profile.capabilities.specializations.includes(spec))).length);
          stryMutAct_9fa48("588") ? score -= matchedSpecs / query.specializations.length * 0.2 : (stryCov_9fa48("588"), score += stryMutAct_9fa48("589") ? matchedSpecs / query.specializations.length / 0.2 : (stryCov_9fa48("589"), (stryMutAct_9fa48("590") ? matchedSpecs * query.specializations.length : (stryCov_9fa48("590"), matchedSpecs / query.specializations.length)) * 0.2));
          stryMutAct_9fa48("591") ? weights -= 0.2 : (stryCov_9fa48("591"), weights += 0.2);
        }
      }

      // Performance bonus
      stryMutAct_9fa48("592") ? score -= profile.performanceHistory.successRate * 0.2 : (stryCov_9fa48("592"), score += stryMutAct_9fa48("593") ? profile.performanceHistory.successRate / 0.2 : (stryCov_9fa48("593"), profile.performanceHistory.successRate * 0.2));
      stryMutAct_9fa48("594") ? weights -= 0.2 : (stryCov_9fa48("594"), weights += 0.2);
      return (stryMutAct_9fa48("598") ? weights <= 0 : stryMutAct_9fa48("597") ? weights >= 0 : stryMutAct_9fa48("596") ? false : stryMutAct_9fa48("595") ? true : (stryCov_9fa48("595", "596", "597", "598"), weights > 0)) ? stryMutAct_9fa48("599") ? score * weights : (stryCov_9fa48("599"), score / weights) : 0;
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
    if (stryMutAct_9fa48("600")) {
      {}
    } else {
      stryCov_9fa48("600");
      const reasons: string[] = stryMutAct_9fa48("601") ? ["Stryker was here"] : (stryCov_9fa48("601"), []);
      reasons.push(stryMutAct_9fa48("602") ? `` : (stryCov_9fa48("602"), `Supports ${query.taskType}`));
      if (stryMutAct_9fa48("605") ? query.languages || query.languages.length > 0 : stryMutAct_9fa48("604") ? false : stryMutAct_9fa48("603") ? true : (stryCov_9fa48("603", "604", "605"), query.languages && (stryMutAct_9fa48("608") ? query.languages.length <= 0 : stryMutAct_9fa48("607") ? query.languages.length >= 0 : stryMutAct_9fa48("606") ? true : (stryCov_9fa48("606", "607", "608"), query.languages.length > 0)))) {
        if (stryMutAct_9fa48("609")) {
          {}
        } else {
          stryCov_9fa48("609");
          reasons.push(stryMutAct_9fa48("610") ? `` : (stryCov_9fa48("610"), `Languages: ${query.languages.join(stryMutAct_9fa48("611") ? "" : (stryCov_9fa48("611"), ", "))}`));
        }
      }
      if (stryMutAct_9fa48("614") ? query.specializations || query.specializations.length > 0 : stryMutAct_9fa48("613") ? false : stryMutAct_9fa48("612") ? true : (stryCov_9fa48("612", "613", "614"), query.specializations && (stryMutAct_9fa48("617") ? query.specializations.length <= 0 : stryMutAct_9fa48("616") ? query.specializations.length >= 0 : stryMutAct_9fa48("615") ? true : (stryCov_9fa48("615", "616", "617"), query.specializations.length > 0)))) {
        if (stryMutAct_9fa48("618")) {
          {}
        } else {
          stryCov_9fa48("618");
          reasons.push(stryMutAct_9fa48("619") ? `` : (stryCov_9fa48("619"), `Specializations: ${query.specializations.join(stryMutAct_9fa48("620") ? "" : (stryCov_9fa48("620"), ", "))}`));
        }
      }
      reasons.push(stryMutAct_9fa48("621") ? `` : (stryCov_9fa48("621"), `${(stryMutAct_9fa48("622") ? profile.performanceHistory.successRate / 100 : (stryCov_9fa48("622"), profile.performanceHistory.successRate * 100)).toFixed(1)}% success rate`));
      reasons.push(stryMutAct_9fa48("623") ? `` : (stryCov_9fa48("623"), `${profile.currentLoad.utilizationPercent.toFixed(0)}% utilized`));
      return reasons.join(stryMutAct_9fa48("624") ? "" : (stryCov_9fa48("624"), "; "));
    }
  }

  /**
   * Start automatic cleanup of stale agents.
   */
  private startAutoCleanup(): void {
    if (stryMutAct_9fa48("625")) {
      {}
    } else {
      stryCov_9fa48("625");
      this.cleanupTimer = setInterval(() => {
        if (stryMutAct_9fa48("626")) {
          {}
        } else {
          stryCov_9fa48("626");
          this.cleanupStaleAgents();
        }
      }, this.config.cleanupIntervalMs);
    }
  }

  /**
   * Clean up stale agents (inactive beyond threshold).
   */
  private cleanupStaleAgents(): void {
    if (stryMutAct_9fa48("627")) {
      {}
    } else {
      stryCov_9fa48("627");
      const now = new Date().toISOString();
      const staleAgents: AgentId[] = stryMutAct_9fa48("628") ? ["Stryker was here"] : (stryCov_9fa48("628"), []);
      const agents = Array.from(this.agents.entries());
      for (const [agentId, profile] of agents) {
        if (stryMutAct_9fa48("629")) {
          {}
        } else {
          stryCov_9fa48("629");
          if (stryMutAct_9fa48("631") ? false : stryMutAct_9fa48("630") ? true : (stryCov_9fa48("630", "631"), AgentProfileHelper.isStale(profile, this.config.staleAgentThresholdMs, now))) {
            if (stryMutAct_9fa48("632")) {
              {}
            } else {
              stryCov_9fa48("632");
              staleAgents.push(agentId);
            }
          }
        }
      }
      for (const agentId of staleAgents) {
        if (stryMutAct_9fa48("633")) {
          {}
        } else {
          stryCov_9fa48("633");
          this.agents.delete(agentId);
        }
      }
    }
  }

  /**
   * Shutdown the registry manager and cleanup resources.
   */
  async shutdown(): Promise<void> {
    if (stryMutAct_9fa48("634")) {
      {}
    } else {
      stryCov_9fa48("634");
      if (stryMutAct_9fa48("636") ? false : stryMutAct_9fa48("635") ? true : (stryCov_9fa48("635", "636"), this.cleanupTimer)) {
        if (stryMutAct_9fa48("637")) {
          {}
        } else {
          stryCov_9fa48("637");
          clearInterval(this.cleanupTimer);
        }
      }
    }
  }
}