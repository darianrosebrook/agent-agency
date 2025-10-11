/**
 * @fileoverview Task Routing Manager - Intelligent Agent Selection (ARBITER-002)
 *
 * Implements intelligent task-to-agent routing using multi-armed bandit algorithms,
 * capability matching, and load balancing to optimize task execution outcomes.
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
import { MultiArmedBandit } from "../rl/MultiArmedBandit";
import { AgentQuery, AgentQueryResult, TaskType } from "../types/agent-registry";
import { RoutingStrategy } from "../types/agentic-rl";
import { RoutingDecision, Task } from "../types/arbiter-orchestration";
import { AgentRegistryManager } from "./AgentRegistryManager";

/**
 * Task Routing Manager Configuration
 */
export interface TaskRoutingConfig {
  /** Enable multi-armed bandit routing */
  enableBandit: boolean;

  /** Minimum agents required for routing */
  minAgentsRequired: number;

  /** Maximum agents to consider per routing */
  maxAgentsToConsider: number;

  /** Default routing strategy fallback */
  defaultStrategy: RoutingStrategy;

  /** Maximum routing decision time (ms) */
  maxRoutingTimeMs: number;

  /** Load balancing weight (0-1) */
  loadBalancingWeight: number;

  /** Capability matching weight (0-1) */
  capabilityMatchWeight: number;
}

/**
 * Routing metrics for monitoring and optimization
 */
export interface RoutingMetrics {
  totalRoutingDecisions: number;
  averageRoutingTimeMs: number;
  explorationRate: number;
  exploitationRate: number;
  capabilityMismatchRate: number;
  loadBalancingEffectiveness: number;
  successRate: number;
}

/**
 * Routing outcome for feedback loop
 */
export interface RoutingOutcome {
  routingDecision: RoutingDecision;
  success: boolean;
  qualityScore: number;
  latencyMs: number;
  errorReason?: string;
}

/**
 * Task Routing Manager
 *
 * Coordinates intelligent task-to-agent routing using multiple strategies:
 * 1. Multi-armed bandit for exploration/exploitation balance
 * 2. Capability matching for task requirements
 * 3. Load balancing for optimal resource utilization
 */
export class TaskRoutingManager {
  private config: TaskRoutingConfig;
  private agentRegistry: AgentRegistryManager;
  private multiArmedBandit: MultiArmedBandit | null = null;
  private metrics: RoutingMetrics;
  private routingHistory: Map<string, RoutingDecision> = new Map();
  constructor(agentRegistry: AgentRegistryManager, config?: Partial<TaskRoutingConfig>) {
    if (stryMutAct_9fa48("1608")) {
      {}
    } else {
      stryCov_9fa48("1608");
      this.agentRegistry = agentRegistry;
      this.config = stryMutAct_9fa48("1609") ? {} : (stryCov_9fa48("1609"), {
        enableBandit: stryMutAct_9fa48("1610") ? false : (stryCov_9fa48("1610"), true),
        minAgentsRequired: 1,
        maxAgentsToConsider: 10,
        defaultStrategy: "multi-armed-bandit",
        maxRoutingTimeMs: 100,
        loadBalancingWeight: 0.3,
        capabilityMatchWeight: 0.7,
        ...config
      });

      // Initialize multi-armed bandit if enabled
      if (stryMutAct_9fa48("1613") ? false : stryMutAct_9fa48("1612") ? true : (stryCov_9fa48("1612", "1613"), this.config.enableBandit)) {
        if (stryMutAct_9fa48("1614")) {
          {}
        } else {
          stryCov_9fa48("1614");
          this.multiArmedBandit = new MultiArmedBandit(stryMutAct_9fa48("1615") ? {} : (stryCov_9fa48("1615"), {
            explorationRate: 0.1,
            decayFactor: 0.995,
            minSampleSize: 5,
            useUCB: stryMutAct_9fa48("1616") ? false : (stryCov_9fa48("1616"), true),
            ucbConstant: 2.0
          }));
        }
      }

      // Initialize metrics
      this.metrics = stryMutAct_9fa48("1617") ? {} : (stryCov_9fa48("1617"), {
        totalRoutingDecisions: 0,
        averageRoutingTimeMs: 0,
        explorationRate: 0,
        exploitationRate: 0,
        capabilityMismatchRate: 0,
        loadBalancingEffectiveness: 0,
        successRate: 0
      });
    }
  }

  /**
   * Route a task to the optimal agent
   *
   * @param task - Task to route
   * @returns Routing decision with selected agent and rationale
   */
  async routeTask(task: Task): Promise<RoutingDecision> {
    if (stryMutAct_9fa48("1618")) {
      {}
    } else {
      stryCov_9fa48("1618");
      const startTime = Date.now();
      try {
        if (stryMutAct_9fa48("1619")) {
          {}
        } else {
          stryCov_9fa48("1619");
          // Step 1: Find candidate agents based on task requirements
          const candidateAgents = await this.findCandidateAgents(task);
          if (stryMutAct_9fa48("1622") ? candidateAgents.length !== 0 : stryMutAct_9fa48("1621") ? false : stryMutAct_9fa48("1620") ? true : (stryCov_9fa48("1620", "1621", "1622"), candidateAgents.length === 0)) {
            if (stryMutAct_9fa48("1623")) {
              {}
            } else {
              stryCov_9fa48("1623");
              throw new Error(`No agents available for task type: ${task.type}. ` + `Required capabilities not found in agent registry.`);
            }
          }
          if (stryMutAct_9fa48("1629") ? candidateAgents.length >= this.config.minAgentsRequired : stryMutAct_9fa48("1628") ? candidateAgents.length <= this.config.minAgentsRequired : stryMutAct_9fa48("1627") ? false : stryMutAct_9fa48("1626") ? true : (stryCov_9fa48("1626", "1627", "1628", "1629"), candidateAgents.length < this.config.minAgentsRequired)) {
            if (stryMutAct_9fa48("1630")) {
              {}
            } else {
              stryCov_9fa48("1630");
              throw new Error(`Insufficient agents (${candidateAgents.length}/${this.config.minAgentsRequired}) ` + `available for task type: ${task.type}`);
            }
          }

          // Step 2: Apply routing strategy
          const routingDecision = await this.applyRoutingStrategy(task, candidateAgents);

          // Step 3: Record routing decision
          this.recordRoutingDecision(routingDecision);

          // Step 4: Update metrics
          const routingTimeMs = stryMutAct_9fa48("1633") ? Date.now() + startTime : (stryCov_9fa48("1633"), Date.now() - startTime);
          this.updateMetrics(routingTimeMs, routingDecision);

          // Validate routing time is within SLA
          if (stryMutAct_9fa48("1637") ? routingTimeMs <= this.config.maxRoutingTimeMs : stryMutAct_9fa48("1636") ? routingTimeMs >= this.config.maxRoutingTimeMs : stryMutAct_9fa48("1635") ? false : stryMutAct_9fa48("1634") ? true : (stryCov_9fa48("1634", "1635", "1636", "1637"), routingTimeMs > this.config.maxRoutingTimeMs)) {
            if (stryMutAct_9fa48("1638")) {
              {}
            } else {
              stryCov_9fa48("1638");
              console.warn(`Routing decision took ${routingTimeMs}ms, exceeding SLA of ${this.config.maxRoutingTimeMs}ms`);
            }
          }
          return routingDecision;
        }
      } catch (error) {
        if (stryMutAct_9fa48("1640")) {
          {}
        } else {
          stryCov_9fa48("1640");
          const errorMessage = error instanceof Error ? error.message : String(error);

          // Create error routing decision
          const errorDecision: RoutingDecision = stryMutAct_9fa48("1641") ? {} : (stryCov_9fa48("1641"), {
            id: `routing-error-${Date.now()}`,
            taskId: task.id,
            selectedAgent: null as any,
            // No agent selected
            confidence: 0,
            reason: `Routing failed: ${errorMessage}`,
            strategy: this.config.defaultStrategy as any,
            alternatives: [],
            timestamp: new Date()
          });
          stryMutAct_9fa48("1645") ? this.metrics.totalRoutingDecisions-- : (stryCov_9fa48("1645"), this.metrics.totalRoutingDecisions++);
          throw error;
        }
      }
    }
  }

  /**
   * Find candidate agents that match task requirements
   */
  private async findCandidateAgents(task: Task): Promise<AgentQueryResult[]> {
    if (stryMutAct_9fa48("1646")) {
      {}
    } else {
      stryCov_9fa48("1646");
      // Build agent query from task requirements
      const query: AgentQuery = stryMutAct_9fa48("1647") ? {} : (stryCov_9fa48("1647"), {
        taskType: task.type as TaskType,
        languages: stryMutAct_9fa48("1648") ? task.requiredCapabilities.languages : (stryCov_9fa48("1648"), task.requiredCapabilities?.languages),
        specializations: stryMutAct_9fa48("1649") ? task.requiredCapabilities.specializations : (stryCov_9fa48("1649"), task.requiredCapabilities?.specializations),
        maxUtilization: 90,
        // Don't overload agents
        minSuccessRate: 0.0 // Consider all agents (bandit will optimize)
      });

      // Query agent registry
      const candidates = await this.agentRegistry.getAgentsByCapability(query);

      // Limit to max agents to consider
      return stryMutAct_9fa48("1650") ? candidates : (stryCov_9fa48("1650"), candidates.slice(0, this.config.maxAgentsToConsider));
    }
  }

  /**
   * Apply routing strategy to select optimal agent
   */
  private async applyRoutingStrategy(task: Task, candidates: AgentQueryResult[]): Promise<RoutingDecision> {
    if (stryMutAct_9fa48("1651")) {
      {}
    } else {
      stryCov_9fa48("1651");
      // Use multi-armed bandit strategy if enabled
      if (stryMutAct_9fa48("1654") ? this.config.enableBandit && this.multiArmedBandit || this.config.defaultStrategy === "multi-armed-bandit" : stryMutAct_9fa48("1653") ? false : stryMutAct_9fa48("1652") ? true : (stryCov_9fa48("1652", "1653", "1654"), (stryMutAct_9fa48("1656") ? this.config.enableBandit || this.multiArmedBandit : stryMutAct_9fa48("1655") ? true : (stryCov_9fa48("1655", "1656"), this.config.enableBandit && this.multiArmedBandit)) && (stryMutAct_9fa48("1658") ? this.config.defaultStrategy !== "multi-armed-bandit" : stryMutAct_9fa48("1657") ? true : (stryCov_9fa48("1657", "1658"), this.config.defaultStrategy === "multi-armed-bandit")))) {
        if (stryMutAct_9fa48("1660")) {
          {}
        } else {
          stryCov_9fa48("1660");
          return await this.routeWithBandit(task, candidates);
        }
      }

      // Fallback to capability-match strategy
      return this.routeByCapability(task, candidates);
    }
  }

  /**
   * Route using multi-armed bandit algorithm
   */
  private async routeWithBandit(task: Task, candidates: AgentQueryResult[]): Promise<RoutingDecision> {
    if (stryMutAct_9fa48("1661")) {
      {}
    } else {
      stryCov_9fa48("1661");
      if (stryMutAct_9fa48("1664") ? false : stryMutAct_9fa48("1663") ? true : stryMutAct_9fa48("1662") ? this.multiArmedBandit : (stryCov_9fa48("1662", "1663", "1664"), !this.multiArmedBandit)) {
        if (stryMutAct_9fa48("1665")) {
          {}
        } else {
          stryCov_9fa48("1665");
          throw new Error("Multi-armed bandit not initialized");
        }
      }

      // Select agent using bandit algorithm
      const selectedAgent = await this.multiArmedBandit.select(candidates, task.type as TaskType);

      // Create detailed routing decision
      const routingDecision = this.multiArmedBandit.createRoutingDecision(task.id, candidates, selectedAgent, task.type as TaskType);
      return stryMutAct_9fa48("1667") ? {} : (stryCov_9fa48("1667"), {
        id: `routing-${task.id}-${Date.now()}`,
        taskId: task.id,
        selectedAgent,
        confidence: routingDecision.confidence,
        reason: routingDecision.rationale,
        strategy: "epsilon-greedy" as const,
        // Bandit uses epsilon-greedy
        alternatives: routingDecision.alternativesConsidered.map(stryMutAct_9fa48("1669") ? () => undefined : (stryCov_9fa48("1669"), alt => stryMutAct_9fa48("1670") ? {} : (stryCov_9fa48("1670"), {
          agent: stryMutAct_9fa48("1671") ? candidates.find(c => c.agent.id === alt.agentId).agent! : (stryCov_9fa48("1671"), candidates.find(stryMutAct_9fa48("1672") ? () => undefined : (stryCov_9fa48("1672"), c => stryMutAct_9fa48("1675") ? c.agent.id !== alt.agentId : stryMutAct_9fa48("1674") ? false : stryMutAct_9fa48("1673") ? true : (stryCov_9fa48("1673", "1674", "1675"), c.agent.id === alt.agentId)))?.agent!),
          score: alt.score,
          reason: alt.reason
        }))),
        timestamp: new Date()
      });
    }
  }

  /**
   * Route by capability matching (fallback strategy)
   */
  private routeByCapability(task: Task, candidates: AgentQueryResult[]): RoutingDecision {
    if (stryMutAct_9fa48("1676")) {
      {}
    } else {
      stryCov_9fa48("1676");
      // Sort by match score (already sorted by registry)
      const bestMatch = candidates[0];
      return stryMutAct_9fa48("1677") ? {} : (stryCov_9fa48("1677"), {
        id: `routing-${task.id}-${Date.now()}`,
        taskId: task.id,
        selectedAgent: bestMatch.agent,
        confidence: bestMatch.matchScore,
        reason: `Best capability match: ${bestMatch.matchReason}`,
        strategy: "capability-match" as const,
        alternatives: stryMutAct_9fa48("1680") ? candidates.map(alt => ({
          agent: alt.agent,
          score: alt.matchScore,
          reason: alt.matchReason
        })) : (stryCov_9fa48("1680"), candidates.slice(1, 3).map(stryMutAct_9fa48("1681") ? () => undefined : (stryCov_9fa48("1681"), alt => stryMutAct_9fa48("1682") ? {} : (stryCov_9fa48("1682"), {
          agent: alt.agent,
          score: alt.matchScore,
          reason: alt.matchReason
        })))),
        timestamp: new Date()
      });
    }
  }

  /**
   * Record routing decision for history and analysis
   */
  private recordRoutingDecision(decision: RoutingDecision): void {
    if (stryMutAct_9fa48("1683")) {
      {}
    } else {
      stryCov_9fa48("1683");
      this.routingHistory.set(decision.id, decision);

      // Keep history size manageable (last 1000 decisions)
      if (stryMutAct_9fa48("1687") ? this.routingHistory.size <= 1000 : stryMutAct_9fa48("1686") ? this.routingHistory.size >= 1000 : stryMutAct_9fa48("1685") ? false : stryMutAct_9fa48("1684") ? true : (stryCov_9fa48("1684", "1685", "1686", "1687"), this.routingHistory.size > 1000)) {
        if (stryMutAct_9fa48("1688")) {
          {}
        } else {
          stryCov_9fa48("1688");
          const firstKey = this.routingHistory.keys().next().value;
          if (stryMutAct_9fa48("1690") ? false : stryMutAct_9fa48("1689") ? true : (stryCov_9fa48("1689", "1690"), firstKey)) {
            if (stryMutAct_9fa48("1691")) {
              {}
            } else {
              stryCov_9fa48("1691");
              this.routingHistory.delete(firstKey);
            }
          }
        }
      }
    }
  }

  /**
   * Update routing metrics
   */
  private updateMetrics(routingTimeMs: number, decision: RoutingDecision): void {
    if (stryMutAct_9fa48("1692")) {
      {}
    } else {
      stryCov_9fa48("1692");
      // Increment count first
      stryMutAct_9fa48("1693") ? this.metrics.totalRoutingDecisions-- : (stryCov_9fa48("1693"), this.metrics.totalRoutingDecisions++);
      const count = this.metrics.totalRoutingDecisions;

      // Update average routing time (incremental average)
      this.metrics.averageRoutingTimeMs = stryMutAct_9fa48("1694") ? (this.metrics.averageRoutingTimeMs * (count - 1) + routingTimeMs) * count : (stryCov_9fa48("1694"), (stryMutAct_9fa48("1695") ? this.metrics.averageRoutingTimeMs * (count - 1) - routingTimeMs : (stryCov_9fa48("1695"), (stryMutAct_9fa48("1696") ? this.metrics.averageRoutingTimeMs / (count - 1) : (stryCov_9fa48("1696"), this.metrics.averageRoutingTimeMs * (stryMutAct_9fa48("1697") ? count + 1 : (stryCov_9fa48("1697"), count - 1)))) + routingTimeMs)) / count);

      // Track exploration vs exploitation
      if (stryMutAct_9fa48("1700") ? decision.strategy !== "epsilon-greedy" : stryMutAct_9fa48("1699") ? false : stryMutAct_9fa48("1698") ? true : (stryCov_9fa48("1698", "1699", "1700"), decision.strategy === "epsilon-greedy")) {
        if (stryMutAct_9fa48("1702")) {
          {}
        } else {
          stryCov_9fa48("1702");
          // Check if this was exploration (lower confidence often means exploration)
          if (stryMutAct_9fa48("1706") ? decision.confidence >= 0.8 : stryMutAct_9fa48("1705") ? decision.confidence <= 0.8 : stryMutAct_9fa48("1704") ? false : stryMutAct_9fa48("1703") ? true : (stryCov_9fa48("1703", "1704", "1705", "1706"), decision.confidence < 0.8)) {
            if (stryMutAct_9fa48("1707")) {
              {}
            } else {
              stryCov_9fa48("1707");
              this.metrics.explorationRate = stryMutAct_9fa48("1708") ? (this.metrics.explorationRate * (count - 1) + 1) * count : (stryCov_9fa48("1708"), (stryMutAct_9fa48("1709") ? this.metrics.explorationRate * (count - 1) - 1 : (stryCov_9fa48("1709"), (stryMutAct_9fa48("1710") ? this.metrics.explorationRate / (count - 1) : (stryCov_9fa48("1710"), this.metrics.explorationRate * (stryMutAct_9fa48("1711") ? count + 1 : (stryCov_9fa48("1711"), count - 1)))) + 1)) / count);
            }
          } else {
            if (stryMutAct_9fa48("1712")) {
              {}
            } else {
              stryCov_9fa48("1712");
              this.metrics.exploitationRate = stryMutAct_9fa48("1713") ? (this.metrics.exploitationRate * (count - 1) + 1) * count : (stryCov_9fa48("1713"), (stryMutAct_9fa48("1714") ? this.metrics.exploitationRate * (count - 1) - 1 : (stryCov_9fa48("1714"), (stryMutAct_9fa48("1715") ? this.metrics.exploitationRate / (count - 1) : (stryCov_9fa48("1715"), this.metrics.exploitationRate * (stryMutAct_9fa48("1716") ? count + 1 : (stryCov_9fa48("1716"), count - 1)))) + 1)) / count);
            }
          }
        }
      }
    }
  }

  /**
   * Record routing outcome for feedback loop
   *
   * @param outcome - Routing outcome with success metrics
   */
  async recordRoutingOutcome(outcome: RoutingOutcome): Promise<void> {
    if (stryMutAct_9fa48("1717")) {
      {}
    } else {
      stryCov_9fa48("1717");
      // Update agent performance in registry
      await this.agentRegistry.updatePerformance(outcome.routingDecision.selectedAgent.id, stryMutAct_9fa48("1718") ? {} : (stryCov_9fa48("1718"), {
        success: outcome.success,
        qualityScore: outcome.qualityScore,
        latencyMs: outcome.latencyMs,
        taskType: outcome.routingDecision.selectedAgent.capabilities.taskTypes[0] as TaskType
      }));

      // Update bandit with outcome
      if (stryMutAct_9fa48("1720") ? false : stryMutAct_9fa48("1719") ? true : (stryCov_9fa48("1719", "1720"), this.multiArmedBandit)) {
        if (stryMutAct_9fa48("1721")) {
          {}
        } else {
          stryCov_9fa48("1721");
          this.multiArmedBandit.updateWithOutcome(outcome.routingDecision.selectedAgent.id, outcome.success, outcome.qualityScore, outcome.latencyMs);
        }
      }

      // Update success rate metric
      const count = this.metrics.totalRoutingDecisions;
      const successValue = outcome.success ? 1 : 0;
      this.metrics.successRate = stryMutAct_9fa48("1722") ? (this.metrics.successRate * count + successValue) * (count + 1) : (stryCov_9fa48("1722"), (stryMutAct_9fa48("1723") ? this.metrics.successRate * count - successValue : (stryCov_9fa48("1723"), (stryMutAct_9fa48("1724") ? this.metrics.successRate / count : (stryCov_9fa48("1724"), this.metrics.successRate * count)) + successValue)) / (stryMutAct_9fa48("1725") ? count - 1 : (stryCov_9fa48("1725"), count + 1)));
    }
  }

  /**
   * Get current routing metrics
   */
  getMetrics(): RoutingMetrics {
    if (stryMutAct_9fa48("1726")) {
      {}
    } else {
      stryCov_9fa48("1726");
      return stryMutAct_9fa48("1727") ? {} : (stryCov_9fa48("1727"), {
        ...this.metrics
      });
    }
  }

  /**
   * Get routing statistics for monitoring
   */
  async getRoutingStats(): Promise<{
    metrics: RoutingMetrics;
    banditStats?: any;
    recentDecisions: Array<{
      taskId: string;
      agentId: string;
      strategy: string;
      confidence: number;
      timestamp: Date;
    }>;
  }> {
    if (stryMutAct_9fa48("1728")) {
      {}
    } else {
      stryCov_9fa48("1728");
      // Get recent decisions (last 10)
      const recentDecisions = stryMutAct_9fa48("1729") ? Array.from(this.routingHistory.values()).map(decision => ({
        taskId: decision.taskId,
        agentId: decision.selectedAgent.id,
        strategy: decision.strategy,
        confidence: decision.confidence,
        timestamp: decision.timestamp
      })) : (stryCov_9fa48("1729"), Array.from(this.routingHistory.values()).slice(stryMutAct_9fa48("1730") ? +10 : (stryCov_9fa48("1730"), -10)).map(stryMutAct_9fa48("1731") ? () => undefined : (stryCov_9fa48("1731"), decision => stryMutAct_9fa48("1732") ? {} : (stryCov_9fa48("1732"), {
        taskId: decision.taskId,
        agentId: decision.selectedAgent.id,
        strategy: decision.strategy,
        confidence: decision.confidence,
        timestamp: decision.timestamp
      }))));
      const stats: any = stryMutAct_9fa48("1733") ? {} : (stryCov_9fa48("1733"), {
        metrics: this.getMetrics(),
        recentDecisions
      });

      // Include bandit stats if available
      if (stryMutAct_9fa48("1735") ? false : stryMutAct_9fa48("1734") ? true : (stryCov_9fa48("1734", "1735"), this.multiArmedBandit)) {
        if (stryMutAct_9fa48("1736")) {
          {}
        } else {
          stryCov_9fa48("1736");
          stats.banditStats = this.multiArmedBandit.getStats();
        }
      }
      return stats;
    }
  }

  /**
   * Reset routing metrics (useful for testing)
   */
  resetMetrics(): void {
    if (stryMutAct_9fa48("1737")) {
      {}
    } else {
      stryCov_9fa48("1737");
      this.metrics = stryMutAct_9fa48("1738") ? {} : (stryCov_9fa48("1738"), {
        totalRoutingDecisions: 0,
        averageRoutingTimeMs: 0,
        explorationRate: 0,
        exploitationRate: 0,
        capabilityMismatchRate: 0,
        loadBalancingEffectiveness: 0,
        successRate: 0
      });
    }
  }

  /**
   * Reset multi-armed bandit state (useful for testing)
   */
  resetBandit(): void {
    if (stryMutAct_9fa48("1739")) {
      {}
    } else {
      stryCov_9fa48("1739");
      if (stryMutAct_9fa48("1741") ? false : stryMutAct_9fa48("1740") ? true : (stryCov_9fa48("1740", "1741"), this.multiArmedBandit)) {
        if (stryMutAct_9fa48("1742")) {
          {}
        } else {
          stryCov_9fa48("1742");
          this.multiArmedBandit.reset();
        }
      }
    }
  }
}

/**
 * Default Task Routing Manager Configuration
 */
export const defaultTaskRoutingConfig: TaskRoutingConfig = stryMutAct_9fa48("1743") ? {} : (stryCov_9fa48("1743"), {
  enableBandit: stryMutAct_9fa48("1744") ? false : (stryCov_9fa48("1744"), true),
  minAgentsRequired: 1,
  maxAgentsToConsider: 10,
  defaultStrategy: "multi-armed-bandit",
  maxRoutingTimeMs: 100,
  loadBalancingWeight: 0.3,
  capabilityMatchWeight: 0.7
});