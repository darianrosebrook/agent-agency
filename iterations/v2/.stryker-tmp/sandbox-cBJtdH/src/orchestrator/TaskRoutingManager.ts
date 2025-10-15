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
import { PerformanceTracker } from "../rl/PerformanceTracker";
import { AgentQuery, AgentQueryResult, TaskType } from "../types/agent-registry";
import { RoutingStrategy } from "../types/agentic-rl";
import { RoutingDecision, Task } from "../types/arbiter-orchestration";
import { AgentRegistryManager } from "./AgentRegistryManager";

// Re-export commonly used types
export { VerificationPriority } from "../types/verification";

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
  private performanceTracker?: PerformanceTracker;
  private multiArmedBandit: MultiArmedBandit | null = null;
  private metrics: RoutingMetrics;
  private routingHistory: Map<string, RoutingDecision> = new Map();
  constructor(agentRegistry: AgentRegistryManager, config?: Partial<TaskRoutingConfig>, performanceTracker?: PerformanceTracker) {
    if (stryMutAct_9fa48("1536")) {
      {}
    } else {
      stryCov_9fa48("1536");
      this.agentRegistry = agentRegistry;
      this.performanceTracker = performanceTracker;
      this.config = stryMutAct_9fa48("1537") ? {} : (stryCov_9fa48("1537"), {
        enableBandit: stryMutAct_9fa48("1538") ? false : (stryCov_9fa48("1538"), true),
        minAgentsRequired: 1,
        maxAgentsToConsider: 10,
        defaultStrategy: "multi-armed-bandit",
        maxRoutingTimeMs: 100,
        loadBalancingWeight: 0.3,
        capabilityMatchWeight: 0.7,
        ...config
      });

      // Initialize multi-armed bandit if enabled
      if (stryMutAct_9fa48("1541") ? false : stryMutAct_9fa48("1540") ? true : (stryCov_9fa48("1540", "1541"), this.config.enableBandit)) {
        if (stryMutAct_9fa48("1542")) {
          {}
        } else {
          stryCov_9fa48("1542");
          this.multiArmedBandit = new MultiArmedBandit(stryMutAct_9fa48("1543") ? {} : (stryCov_9fa48("1543"), {
            explorationRate: 0.1,
            decayFactor: 0.995,
            minSampleSize: 5,
            useUCB: stryMutAct_9fa48("1544") ? false : (stryCov_9fa48("1544"), true),
            ucbConstant: 2.0
          }));
        }
      }

      // Initialize metrics
      this.metrics = stryMutAct_9fa48("1545") ? {} : (stryCov_9fa48("1545"), {
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
   * Set the performance tracker for performance-aware routing.
   *
   * @param tracker - Performance tracker instance
   */
  setPerformanceTracker(tracker: PerformanceTracker): void {
    if (stryMutAct_9fa48("1546")) {
      {}
    } else {
      stryCov_9fa48("1546");
      this.performanceTracker = tracker;
    }
  }

  /**
   * Route a task to the optimal agent
   *
   * @param task - Task to route
   * @returns Routing decision with selected agent and rationale
   */
  async routeTask(task: Task): Promise<RoutingDecision> {
    if (stryMutAct_9fa48("1547")) {
      {}
    } else {
      stryCov_9fa48("1547");
      const startTime = Date.now();
      try {
        if (stryMutAct_9fa48("1548")) {
          {}
        } else {
          stryCov_9fa48("1548");
          // Step 1: Find candidate agents based on task requirements
          const candidateAgents = await this.findCandidateAgents(task);
          if (stryMutAct_9fa48("1551") ? candidateAgents.length !== 0 : stryMutAct_9fa48("1550") ? false : stryMutAct_9fa48("1549") ? true : (stryCov_9fa48("1549", "1550", "1551"), candidateAgents.length === 0)) {
            if (stryMutAct_9fa48("1552")) {
              {}
            } else {
              stryCov_9fa48("1552");
              throw new Error(`No agents available for task type: ${task.type}. ` + `Required capabilities not found in agent registry.`);
            }
          }
          if (stryMutAct_9fa48("1558") ? candidateAgents.length >= this.config.minAgentsRequired : stryMutAct_9fa48("1557") ? candidateAgents.length <= this.config.minAgentsRequired : stryMutAct_9fa48("1556") ? false : stryMutAct_9fa48("1555") ? true : (stryCov_9fa48("1555", "1556", "1557", "1558"), candidateAgents.length < this.config.minAgentsRequired)) {
            if (stryMutAct_9fa48("1559")) {
              {}
            } else {
              stryCov_9fa48("1559");
              throw new Error(`Insufficient agents (${candidateAgents.length}/${this.config.minAgentsRequired}) ` + `available for task type: ${task.type}`);
            }
          }

          // Step 2: Get performance context for routing decision
          const performanceContext = this.performanceTracker ? await this.getPerformanceContext(task, candidateAgents) : null;

          // Step 3: Apply routing strategy with performance context
          const routingDecision = await this.applyRoutingStrategy(task, candidateAgents, performanceContext);

          // Step 4: Record routing decision
          this.recordRoutingDecision(routingDecision);

          // Step 5: Update metrics
          const routingTimeMs = stryMutAct_9fa48("1562") ? Date.now() + startTime : (stryCov_9fa48("1562"), Date.now() - startTime);
          this.updateMetrics(routingTimeMs, routingDecision);

          // Validate routing time is within SLA
          if (stryMutAct_9fa48("1566") ? routingTimeMs <= this.config.maxRoutingTimeMs : stryMutAct_9fa48("1565") ? routingTimeMs >= this.config.maxRoutingTimeMs : stryMutAct_9fa48("1564") ? false : stryMutAct_9fa48("1563") ? true : (stryCov_9fa48("1563", "1564", "1565", "1566"), routingTimeMs > this.config.maxRoutingTimeMs)) {
            if (stryMutAct_9fa48("1567")) {
              {}
            } else {
              stryCov_9fa48("1567");
              console.warn(`Routing decision took ${routingTimeMs}ms, exceeding SLA of ${this.config.maxRoutingTimeMs}ms`);
            }
          }
          return routingDecision;
        }
      } catch (error) {
        if (stryMutAct_9fa48("1569")) {
          {}
        } else {
          stryCov_9fa48("1569");
          const errorMessage = error instanceof Error ? error.message : String(error);

          // Create error routing decision
          const errorDecision: RoutingDecision = stryMutAct_9fa48("1570") ? {} : (stryCov_9fa48("1570"), {
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
          stryMutAct_9fa48("1574") ? this.metrics.totalRoutingDecisions-- : (stryCov_9fa48("1574"), this.metrics.totalRoutingDecisions++);
          throw error;
        }
      }
    }
  }

  /**
   * Find candidate agents that match task requirements
   */
  private async findCandidateAgents(task: Task): Promise<AgentQueryResult[]> {
    if (stryMutAct_9fa48("1575")) {
      {}
    } else {
      stryCov_9fa48("1575");
      // Build agent query from task requirements
      const query: AgentQuery = stryMutAct_9fa48("1576") ? {} : (stryCov_9fa48("1576"), {
        taskType: task.type as TaskType,
        languages: stryMutAct_9fa48("1577") ? task.requiredCapabilities.languages : (stryCov_9fa48("1577"), task.requiredCapabilities?.languages),
        specializations: stryMutAct_9fa48("1578") ? task.requiredCapabilities.specializations : (stryCov_9fa48("1578"), task.requiredCapabilities?.specializations),
        maxUtilization: 90,
        // Don't overload agents
        minSuccessRate: 0.0 // Consider all agents (bandit will optimize)
      });

      // Query agent registry
      const candidates = await this.agentRegistry.getAgentsByCapability(query);

      // Limit to max agents to consider
      return stryMutAct_9fa48("1579") ? candidates : (stryCov_9fa48("1579"), candidates.slice(0, this.config.maxAgentsToConsider));
    }
  }

  /**
   * Apply routing strategy to select optimal agent
   */
  private async applyRoutingStrategy(task: Task, candidates: AgentQueryResult[], performanceContext?: any): Promise<RoutingDecision> {
    if (stryMutAct_9fa48("1580")) {
      {}
    } else {
      stryCov_9fa48("1580");
      // Use multi-armed bandit strategy if enabled
      if (stryMutAct_9fa48("1583") ? this.config.enableBandit && this.multiArmedBandit || this.config.defaultStrategy === "multi-armed-bandit" : stryMutAct_9fa48("1582") ? false : stryMutAct_9fa48("1581") ? true : (stryCov_9fa48("1581", "1582", "1583"), (stryMutAct_9fa48("1585") ? this.config.enableBandit || this.multiArmedBandit : stryMutAct_9fa48("1584") ? true : (stryCov_9fa48("1584", "1585"), this.config.enableBandit && this.multiArmedBandit)) && (stryMutAct_9fa48("1587") ? this.config.defaultStrategy !== "multi-armed-bandit" : stryMutAct_9fa48("1586") ? true : (stryCov_9fa48("1586", "1587"), this.config.defaultStrategy === "multi-armed-bandit")))) {
        if (stryMutAct_9fa48("1589")) {
          {}
        } else {
          stryCov_9fa48("1589");
          return await this.routeWithBandit(task, candidates, performanceContext);
        }
      }

      // Fallback to capability-match strategy
      return this.routeByCapability(task, candidates, performanceContext);
    }
  }

  /**
   * Route using multi-armed bandit algorithm
   */
  private async routeWithBandit(task: Task, candidates: AgentQueryResult[], performanceContext?: any): Promise<RoutingDecision> {
    if (stryMutAct_9fa48("1590")) {
      {}
    } else {
      stryCov_9fa48("1590");
      if (stryMutAct_9fa48("1593") ? false : stryMutAct_9fa48("1592") ? true : stryMutAct_9fa48("1591") ? this.multiArmedBandit : (stryCov_9fa48("1591", "1592", "1593"), !this.multiArmedBandit)) {
        if (stryMutAct_9fa48("1594")) {
          {}
        } else {
          stryCov_9fa48("1594");
          throw new Error("Multi-armed bandit not initialized");
        }
      }

      // Select agent using bandit algorithm
      const selectedAgent = await this.multiArmedBandit.select(candidates, task.type as TaskType);

      // Create detailed routing decision
      const routingDecision = this.multiArmedBandit.createRoutingDecision(task.id, candidates, selectedAgent, task.type as TaskType);
      return stryMutAct_9fa48("1596") ? {} : (stryCov_9fa48("1596"), {
        id: `routing-${task.id}-${Date.now()}`,
        taskId: task.id,
        selectedAgent,
        confidence: routingDecision.confidence,
        reason: routingDecision.rationale,
        strategy: "epsilon-greedy" as const,
        // Bandit uses epsilon-greedy
        alternatives: routingDecision.alternativesConsidered.map(stryMutAct_9fa48("1598") ? () => undefined : (stryCov_9fa48("1598"), alt => stryMutAct_9fa48("1599") ? {} : (stryCov_9fa48("1599"), {
          agent: stryMutAct_9fa48("1600") ? candidates.find(c => c.agent.id === alt.agentId).agent! : (stryCov_9fa48("1600"), candidates.find(stryMutAct_9fa48("1601") ? () => undefined : (stryCov_9fa48("1601"), c => stryMutAct_9fa48("1604") ? c.agent.id !== alt.agentId : stryMutAct_9fa48("1603") ? false : stryMutAct_9fa48("1602") ? true : (stryCov_9fa48("1602", "1603", "1604"), c.agent.id === alt.agentId)))?.agent!),
          score: alt.score,
          reason: alt.reason
        }))),
        timestamp: new Date()
      });
    }
  }

  /**
   * Get performance context for routing decision.
   *
   * @param task - Task being routed
   * @param candidates - Candidate agents
   * @returns Performance context with agent metrics
   */
  private async getPerformanceContext(task: Task, candidates: AgentQueryResult[]): Promise<any> {
    if (stryMutAct_9fa48("1605")) {
      {}
    } else {
      stryCov_9fa48("1605");
      if (stryMutAct_9fa48("1608") ? false : stryMutAct_9fa48("1607") ? true : stryMutAct_9fa48("1606") ? this.performanceTracker : (stryCov_9fa48("1606", "1607", "1608"), !this.performanceTracker)) {
        if (stryMutAct_9fa48("1609")) {
          {}
        } else {
          stryCov_9fa48("1609");
          return null;
        }
      }
      try {
        if (stryMutAct_9fa48("1610")) {
          {}
        } else {
          stryCov_9fa48("1610");
          // Get performance stats to understand overall system performance
          const stats = this.performanceTracker.getStats();

          // Create basic performance context for each candidate
          const agentMetrics: Record<string, any> = {};
          for (const candidate of candidates) {
            if (stryMutAct_9fa48("1611")) {
              {}
            } else {
              stryCov_9fa48("1611");
              // For now, use basic heuristics based on agent capabilities and system stats
              // In a full implementation, this would query historical performance data
              const capabilityScore = stryMutAct_9fa48("1614") ? candidate.matchScore && 0.5 : stryMutAct_9fa48("1613") ? false : stryMutAct_9fa48("1612") ? true : (stryCov_9fa48("1612", "1613", "1614"), candidate.matchScore || 0.5);
              const systemSuccessRate = 0.8; // Default success rate assumption

              // Combine capability match with system performance
              const performanceScore = stryMutAct_9fa48("1615") ? capabilityScore * 0.7 - systemSuccessRate * 0.3 : (stryCov_9fa48("1615"), (stryMutAct_9fa48("1616") ? capabilityScore / 0.7 : (stryCov_9fa48("1616"), capabilityScore * 0.7)) + (stryMutAct_9fa48("1617") ? systemSuccessRate / 0.3 : (stryCov_9fa48("1617"), systemSuccessRate * 0.3)));
              agentMetrics[candidate.agent.id] = stryMutAct_9fa48("1618") ? {} : (stryCov_9fa48("1618"), {
                overallScore: performanceScore,
                confidence: 0.5,
                // Low confidence for now
                recentMetrics: [] // Would be populated with actual historical data
              });
            }
          }
          return stryMutAct_9fa48("1620") ? {} : (stryCov_9fa48("1620"), {
            taskId: task.id,
            taskType: task.type,
            agentMetrics,
            systemStats: stats
          });
        }
      } catch (error) {
        if (stryMutAct_9fa48("1621")) {
          {}
        } else {
          stryCov_9fa48("1621");
          console.warn("Failed to get performance context for routing:", error);
          return null;
        }
      }
    }
  }

  /**
   * Route by capability matching (fallback strategy)
   */
  private routeByCapability(task: Task, candidates: AgentQueryResult[], performanceContext?: any): RoutingDecision {
    if (stryMutAct_9fa48("1623")) {
      {}
    } else {
      stryCov_9fa48("1623");
      let selectedCandidate: AgentQueryResult;
      let strategy: RoutingStrategy = "capability-match";
      if (stryMutAct_9fa48("1627") ? performanceContext.agentMetrics : stryMutAct_9fa48("1626") ? false : stryMutAct_9fa48("1625") ? true : (stryCov_9fa48("1625", "1626", "1627"), performanceContext?.agentMetrics)) {
        if (stryMutAct_9fa48("1628")) {
          {}
        } else {
          stryCov_9fa48("1628");
          // Use performance-weighted scoring (still using capability-match strategy)
          strategy = "capability-match";
          const scoredCandidates = candidates.map(candidate => {
            if (stryMutAct_9fa48("1630")) {
              {}
            } else {
              stryCov_9fa48("1630");
              const capabilityScore = candidate.matchScore;
              const performanceScore = stryMutAct_9fa48("1633") ? performanceContext.agentMetrics[candidate.agent.id]?.overallScore && 0.5 : stryMutAct_9fa48("1632") ? false : stryMutAct_9fa48("1631") ? true : (stryCov_9fa48("1631", "1632", "1633"), (stryMutAct_9fa48("1634") ? performanceContext.agentMetrics[candidate.agent.id].overallScore : (stryCov_9fa48("1634"), performanceContext.agentMetrics[candidate.agent.id]?.overallScore)) || 0.5);
              const confidence = stryMutAct_9fa48("1637") ? performanceContext.agentMetrics[candidate.agent.id]?.confidence && 0.5 : stryMutAct_9fa48("1636") ? false : stryMutAct_9fa48("1635") ? true : (stryCov_9fa48("1635", "1636", "1637"), (stryMutAct_9fa48("1638") ? performanceContext.agentMetrics[candidate.agent.id].confidence : (stryCov_9fa48("1638"), performanceContext.agentMetrics[candidate.agent.id]?.confidence)) || 0.5);

              // Weight: 70% capability, 30% performance history
              const weightedScore = stryMutAct_9fa48("1639") ? capabilityScore * 0.7 - performanceScore * 0.3 : (stryCov_9fa48("1639"), (stryMutAct_9fa48("1640") ? capabilityScore / 0.7 : (stryCov_9fa48("1640"), capabilityScore * 0.7)) + (stryMutAct_9fa48("1641") ? performanceScore / 0.3 : (stryCov_9fa48("1641"), performanceScore * 0.3)));
              return stryMutAct_9fa48("1642") ? {} : (stryCov_9fa48("1642"), {
                ...candidate,
                weightedScore,
                performanceScore,
                confidence
              });
            }
          });

          // Sort by weighted score
          stryMutAct_9fa48("1643") ? scoredCandidates : (stryCov_9fa48("1643"), scoredCandidates.sort(stryMutAct_9fa48("1644") ? () => undefined : (stryCov_9fa48("1644"), (a, b) => stryMutAct_9fa48("1645") ? b.weightedScore + a.weightedScore : (stryCov_9fa48("1645"), b.weightedScore - a.weightedScore))));
          selectedCandidate = scoredCandidates[0];
          return stryMutAct_9fa48("1646") ? {} : (stryCov_9fa48("1646"), {
            id: `routing-${task.id}-${Date.now()}`,
            taskId: task.id,
            selectedAgent: selectedCandidate.agent,
            confidence: selectedCandidate.matchScore,
            reason: `Performance-weighted selection: capability ${selectedCandidate.matchScore.toFixed(2)}`,
            strategy,
            alternatives: [],
            timestamp: new Date()
          });
        }
      } else {
        if (stryMutAct_9fa48("1650")) {
          {}
        } else {
          stryCov_9fa48("1650");
          // Fallback to pure capability matching
          selectedCandidate = candidates[0];
          return stryMutAct_9fa48("1651") ? {} : (stryCov_9fa48("1651"), {
            id: `routing-${task.id}-${Date.now()}`,
            taskId: task.id,
            selectedAgent: selectedCandidate.agent,
            confidence: selectedCandidate.matchScore,
            reason: `Best capability match: ${selectedCandidate.matchReason}`,
            strategy,
            alternatives: stryMutAct_9fa48("1654") ? candidates.map(alt => ({
              agent: alt.agent,
              score: alt.matchScore,
              reason: alt.matchReason
            })) : (stryCov_9fa48("1654"), candidates.slice(1, 3).map(stryMutAct_9fa48("1655") ? () => undefined : (stryCov_9fa48("1655"), alt => stryMutAct_9fa48("1656") ? {} : (stryCov_9fa48("1656"), {
              agent: alt.agent,
              score: alt.matchScore,
              reason: alt.matchReason
            })))),
            timestamp: new Date()
          });
        }
      }
    }
  }

  /**
   * Record routing decision for history and analysis
   */
  private recordRoutingDecision(decision: RoutingDecision): void {
    if (stryMutAct_9fa48("1657")) {
      {}
    } else {
      stryCov_9fa48("1657");
      this.routingHistory.set(decision.id, decision);

      // Keep history size manageable (last 1000 decisions)
      if (stryMutAct_9fa48("1661") ? this.routingHistory.size <= 1000 : stryMutAct_9fa48("1660") ? this.routingHistory.size >= 1000 : stryMutAct_9fa48("1659") ? false : stryMutAct_9fa48("1658") ? true : (stryCov_9fa48("1658", "1659", "1660", "1661"), this.routingHistory.size > 1000)) {
        if (stryMutAct_9fa48("1662")) {
          {}
        } else {
          stryCov_9fa48("1662");
          const firstKey = this.routingHistory.keys().next().value;
          if (stryMutAct_9fa48("1664") ? false : stryMutAct_9fa48("1663") ? true : (stryCov_9fa48("1663", "1664"), firstKey)) {
            if (stryMutAct_9fa48("1665")) {
              {}
            } else {
              stryCov_9fa48("1665");
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
    if (stryMutAct_9fa48("1666")) {
      {}
    } else {
      stryCov_9fa48("1666");
      // Increment count first
      stryMutAct_9fa48("1667") ? this.metrics.totalRoutingDecisions-- : (stryCov_9fa48("1667"), this.metrics.totalRoutingDecisions++);
      const count = this.metrics.totalRoutingDecisions;

      // Update average routing time (incremental average)
      this.metrics.averageRoutingTimeMs = stryMutAct_9fa48("1668") ? (this.metrics.averageRoutingTimeMs * (count - 1) + routingTimeMs) * count : (stryCov_9fa48("1668"), (stryMutAct_9fa48("1669") ? this.metrics.averageRoutingTimeMs * (count - 1) - routingTimeMs : (stryCov_9fa48("1669"), (stryMutAct_9fa48("1670") ? this.metrics.averageRoutingTimeMs / (count - 1) : (stryCov_9fa48("1670"), this.metrics.averageRoutingTimeMs * (stryMutAct_9fa48("1671") ? count + 1 : (stryCov_9fa48("1671"), count - 1)))) + routingTimeMs)) / count);

      // Track exploration vs exploitation
      if (stryMutAct_9fa48("1674") ? decision.strategy !== "epsilon-greedy" : stryMutAct_9fa48("1673") ? false : stryMutAct_9fa48("1672") ? true : (stryCov_9fa48("1672", "1673", "1674"), decision.strategy === "epsilon-greedy")) {
        if (stryMutAct_9fa48("1676")) {
          {}
        } else {
          stryCov_9fa48("1676");
          // Check if this was exploration (lower confidence often means exploration)
          if (stryMutAct_9fa48("1680") ? decision.confidence >= 0.8 : stryMutAct_9fa48("1679") ? decision.confidence <= 0.8 : stryMutAct_9fa48("1678") ? false : stryMutAct_9fa48("1677") ? true : (stryCov_9fa48("1677", "1678", "1679", "1680"), decision.confidence < 0.8)) {
            if (stryMutAct_9fa48("1681")) {
              {}
            } else {
              stryCov_9fa48("1681");
              this.metrics.explorationRate = stryMutAct_9fa48("1682") ? (this.metrics.explorationRate * (count - 1) + 1) * count : (stryCov_9fa48("1682"), (stryMutAct_9fa48("1683") ? this.metrics.explorationRate * (count - 1) - 1 : (stryCov_9fa48("1683"), (stryMutAct_9fa48("1684") ? this.metrics.explorationRate / (count - 1) : (stryCov_9fa48("1684"), this.metrics.explorationRate * (stryMutAct_9fa48("1685") ? count + 1 : (stryCov_9fa48("1685"), count - 1)))) + 1)) / count);
            }
          } else {
            if (stryMutAct_9fa48("1686")) {
              {}
            } else {
              stryCov_9fa48("1686");
              this.metrics.exploitationRate = stryMutAct_9fa48("1687") ? (this.metrics.exploitationRate * (count - 1) + 1) * count : (stryCov_9fa48("1687"), (stryMutAct_9fa48("1688") ? this.metrics.exploitationRate * (count - 1) - 1 : (stryCov_9fa48("1688"), (stryMutAct_9fa48("1689") ? this.metrics.exploitationRate / (count - 1) : (stryCov_9fa48("1689"), this.metrics.exploitationRate * (stryMutAct_9fa48("1690") ? count + 1 : (stryCov_9fa48("1690"), count - 1)))) + 1)) / count);
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
    if (stryMutAct_9fa48("1691")) {
      {}
    } else {
      stryCov_9fa48("1691");
      // Update agent performance in registry
      await this.agentRegistry.updatePerformance(outcome.routingDecision.selectedAgent.id, stryMutAct_9fa48("1692") ? {} : (stryCov_9fa48("1692"), {
        success: outcome.success,
        qualityScore: outcome.qualityScore,
        latencyMs: outcome.latencyMs,
        taskType: outcome.routingDecision.selectedAgent.capabilities.taskTypes[0] as TaskType
      }));

      // Update bandit with outcome
      if (stryMutAct_9fa48("1694") ? false : stryMutAct_9fa48("1693") ? true : (stryCov_9fa48("1693", "1694"), this.multiArmedBandit)) {
        if (stryMutAct_9fa48("1695")) {
          {}
        } else {
          stryCov_9fa48("1695");
          this.multiArmedBandit.updateWithOutcome(outcome.routingDecision.selectedAgent.id, outcome.success, outcome.qualityScore, outcome.latencyMs);
        }
      }

      // Update success rate metric
      const count = this.metrics.totalRoutingDecisions;
      const successValue = outcome.success ? 1 : 0;
      this.metrics.successRate = stryMutAct_9fa48("1696") ? (this.metrics.successRate * count + successValue) * (count + 1) : (stryCov_9fa48("1696"), (stryMutAct_9fa48("1697") ? this.metrics.successRate * count - successValue : (stryCov_9fa48("1697"), (stryMutAct_9fa48("1698") ? this.metrics.successRate / count : (stryCov_9fa48("1698"), this.metrics.successRate * count)) + successValue)) / (stryMutAct_9fa48("1699") ? count - 1 : (stryCov_9fa48("1699"), count + 1)));
    }
  }

  /**
   * Get current routing metrics
   */
  getMetrics(): RoutingMetrics {
    if (stryMutAct_9fa48("1700")) {
      {}
    } else {
      stryCov_9fa48("1700");
      return stryMutAct_9fa48("1701") ? {} : (stryCov_9fa48("1701"), {
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
    if (stryMutAct_9fa48("1702")) {
      {}
    } else {
      stryCov_9fa48("1702");
      // Get recent decisions (last 10)
      const recentDecisions = stryMutAct_9fa48("1703") ? Array.from(this.routingHistory.values()).map(decision => ({
        taskId: decision.taskId,
        agentId: decision.selectedAgent.id,
        strategy: decision.strategy,
        confidence: decision.confidence,
        timestamp: decision.timestamp
      })) : (stryCov_9fa48("1703"), Array.from(this.routingHistory.values()).slice(stryMutAct_9fa48("1704") ? +10 : (stryCov_9fa48("1704"), -10)).map(stryMutAct_9fa48("1705") ? () => undefined : (stryCov_9fa48("1705"), decision => stryMutAct_9fa48("1706") ? {} : (stryCov_9fa48("1706"), {
        taskId: decision.taskId,
        agentId: decision.selectedAgent.id,
        strategy: decision.strategy,
        confidence: decision.confidence,
        timestamp: decision.timestamp
      }))));
      const stats: any = stryMutAct_9fa48("1707") ? {} : (stryCov_9fa48("1707"), {
        metrics: this.getMetrics(),
        recentDecisions
      });

      // Include bandit stats if available
      if (stryMutAct_9fa48("1709") ? false : stryMutAct_9fa48("1708") ? true : (stryCov_9fa48("1708", "1709"), this.multiArmedBandit)) {
        if (stryMutAct_9fa48("1710")) {
          {}
        } else {
          stryCov_9fa48("1710");
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
    if (stryMutAct_9fa48("1711")) {
      {}
    } else {
      stryCov_9fa48("1711");
      this.metrics = stryMutAct_9fa48("1712") ? {} : (stryCov_9fa48("1712"), {
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
    if (stryMutAct_9fa48("1713")) {
      {}
    } else {
      stryCov_9fa48("1713");
      if (stryMutAct_9fa48("1715") ? false : stryMutAct_9fa48("1714") ? true : (stryCov_9fa48("1714", "1715"), this.multiArmedBandit)) {
        if (stryMutAct_9fa48("1716")) {
          {}
        } else {
          stryCov_9fa48("1716");
          this.multiArmedBandit.reset();
        }
      }
    }
  }
}

/**
 * Default Task Routing Manager Configuration
 */
export const defaultTaskRoutingConfig: TaskRoutingConfig = stryMutAct_9fa48("1717") ? {} : (stryCov_9fa48("1717"), {
  enableBandit: stryMutAct_9fa48("1718") ? false : (stryCov_9fa48("1718"), true),
  minAgentsRequired: 1,
  maxAgentsToConsider: 10,
  defaultStrategy: "multi-armed-bandit",
  maxRoutingTimeMs: 100,
  loadBalancingWeight: 0.3,
  capabilityMatchWeight: 0.7
});