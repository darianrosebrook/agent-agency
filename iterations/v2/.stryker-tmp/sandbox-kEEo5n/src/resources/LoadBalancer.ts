/**
 * @fileoverview Load Balancer for Adaptive Resource Manager
 *
 * Distributes tasks across agents using configurable strategies.
 * Supports round-robin, least-loaded, weighted, and priority-based balancing.
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
import { LoadBalancingStrategy, type AgentResourceProfile, type ILoadBalancer, type LoadBalancingDecision, type ResourceAllocationRequest } from "@/types/resource-types";
import { ResourceMonitor } from "./ResourceMonitor";

/**
 * Load Balancer
 *
 * Selects optimal agents for task assignment using:
 * - Multiple balancing strategies
 * - Resource-aware decision making
 * - Load distribution tracking
 * - Fast decision times (<50ms)
 */
export class LoadBalancer implements ILoadBalancer {
  private logger: Logger;
  private strategy: LoadBalancingStrategy;
  private resourceMonitor: ResourceMonitor;
  private roundRobinIndex = 0;
  private loadDistribution: Map<string, number> = new Map();
  constructor(resourceMonitor: ResourceMonitor, strategy: LoadBalancingStrategy = LoadBalancingStrategy.LEAST_LOADED) {
    if (stryMutAct_9fa48("159")) {
      {}
    } else {
      stryCov_9fa48("159");
      this.logger = new Logger("LoadBalancer");
      this.resourceMonitor = resourceMonitor;
      this.strategy = strategy;
    }
  }

  /**
   * Select agent for task assignment
   *
   * @param request Resource allocation request
   * @param availableAgents List of available agent IDs
   * @returns Load balancing decision
   */
  async selectAgent(request: ResourceAllocationRequest, availableAgents: string[]): Promise<LoadBalancingDecision> {
    if (stryMutAct_9fa48("161")) {
      {}
    } else {
      stryCov_9fa48("161");
      const startTime = Date.now();
      if (stryMutAct_9fa48("164") ? availableAgents.length !== 0 : stryMutAct_9fa48("163") ? false : stryMutAct_9fa48("162") ? true : (stryCov_9fa48("162", "163", "164"), availableAgents.length === 0)) {
        if (stryMutAct_9fa48("165")) {
          {}
        } else {
          stryCov_9fa48("165");
          throw new Error("No available agents for task assignment");
        }
      }

      // Get agent resource profiles
      const agentProfiles = await this.getAgentProfiles(availableAgents);

      // Select agent based on strategy
      let selectedAgentId: string;
      let rationale: string;
      let alternativesConsidered: string[] = [];
      switch (this.strategy) {
        case LoadBalancingStrategy.ROUND_ROBIN:
          if (stryMutAct_9fa48("168")) {} else {
            stryCov_9fa48("168");
            ({
              selectedAgentId,
              rationale,
              alternativesConsidered
            } = this.selectRoundRobin(availableAgents));
            break;
          }
        case LoadBalancingStrategy.LEAST_LOADED:
          if (stryMutAct_9fa48("169")) {} else {
            stryCov_9fa48("169");
            ({
              selectedAgentId,
              rationale,
              alternativesConsidered
            } = await this.selectLeastLoaded(agentProfiles));
            break;
          }
        case LoadBalancingStrategy.WEIGHTED:
          if (stryMutAct_9fa48("170")) {} else {
            stryCov_9fa48("170");
            ({
              selectedAgentId,
              rationale,
              alternativesConsidered
            } = await this.selectWeighted(agentProfiles));
            break;
          }
        case LoadBalancingStrategy.PRIORITY_BASED:
          if (stryMutAct_9fa48("171")) {} else {
            stryCov_9fa48("171");
            ({
              selectedAgentId,
              rationale,
              alternativesConsidered
            } = await this.selectPriorityBased(request, agentProfiles));
            break;
          }
        case LoadBalancingStrategy.RANDOM:
          if (stryMutAct_9fa48("172")) {} else {
            stryCov_9fa48("172");
            ({
              selectedAgentId,
              rationale,
              alternativesConsidered
            } = this.selectRandom(availableAgents));
            break;
          }
        default:
          if (stryMutAct_9fa48("173")) {} else {
            stryCov_9fa48("173");
            throw new Error(`Unknown load balancing strategy: ${this.strategy}`);
          }
      }

      // Get agent load
      const agentProfile = agentProfiles.find(stryMutAct_9fa48("175") ? () => undefined : (stryCov_9fa48("175"), p => stryMutAct_9fa48("178") ? p.agentId !== selectedAgentId : stryMutAct_9fa48("177") ? false : stryMutAct_9fa48("176") ? true : (stryCov_9fa48("176", "177", "178"), p.agentId === selectedAgentId)));
      const agentLoad = agentProfile ? stryMutAct_9fa48("179") ? agentProfile.currentTaskCount / agentProfile.maxTaskCapacity / 100 : (stryCov_9fa48("179"), (stryMutAct_9fa48("180") ? agentProfile.currentTaskCount * agentProfile.maxTaskCapacity : (stryCov_9fa48("180"), agentProfile.currentTaskCount / agentProfile.maxTaskCapacity)) * 100) : 0;

      // Update load distribution
      const currentLoad = stryMutAct_9fa48("181") ? this.loadDistribution.get(selectedAgentId) && 0 : (stryCov_9fa48("181"), this.loadDistribution.get(selectedAgentId) ?? 0);
      this.loadDistribution.set(selectedAgentId, stryMutAct_9fa48("182") ? currentLoad - 1 : (stryCov_9fa48("182"), currentLoad + 1));
      const decision: LoadBalancingDecision = stryMutAct_9fa48("183") ? {} : (stryCov_9fa48("183"), {
        selectedAgentId,
        strategy: this.strategy,
        agentLoad,
        timestamp: new Date(),
        rationale,
        alternativesConsidered,
        decisionDurationMs: stryMutAct_9fa48("184") ? Date.now() + startTime : (stryCov_9fa48("184"), Date.now() - startTime)
      });
      this.logger.debug("Load balancing decision made", stryMutAct_9fa48("186") ? {} : (stryCov_9fa48("186"), {
        selectedAgent: selectedAgentId,
        strategy: this.strategy,
        load: agentLoad,
        decisionTimeMs: decision.decisionDurationMs
      }));
      return decision;
    }
  }

  /**
   * Update load balancing strategy
   *
   * @param strategy New strategy
   */
  setStrategy(strategy: LoadBalancingStrategy): void {
    if (stryMutAct_9fa48("187")) {
      {}
    } else {
      stryCov_9fa48("187");
      this.strategy = strategy;
      this.logger.info("Load balancing strategy updated", stryMutAct_9fa48("189") ? {} : (stryCov_9fa48("189"), {
        strategy
      }));
    }
  }

  /**
   * Get current strategy
   *
   * @returns Current strategy
   */
  getStrategy(): LoadBalancingStrategy {
    if (stryMutAct_9fa48("190")) {
      {}
    } else {
      stryCov_9fa48("190");
      return this.strategy;
    }
  }

  /**
   * Get load distribution statistics
   *
   * @returns Map of agent ID to task count
   */
  async getLoadDistribution(): Promise<Map<string, number>> {
    if (stryMutAct_9fa48("191")) {
      {}
    } else {
      stryCov_9fa48("191");
      return new Map(this.loadDistribution);
    }
  }

  /**
   * Reset load distribution statistics
   */
  resetLoadDistribution(): void {
    if (stryMutAct_9fa48("192")) {
      {}
    } else {
      stryCov_9fa48("192");
      this.loadDistribution.clear();
      this.logger.info("Load distribution statistics reset");
    }
  }

  /**
   * Select agent using round-robin strategy
   */
  private selectRoundRobin(availableAgents: string[]): {
    selectedAgentId: string;
    rationale: string;
    alternativesConsidered: string[];
  } {
    if (stryMutAct_9fa48("194")) {
      {}
    } else {
      stryCov_9fa48("194");
      this.roundRobinIndex = stryMutAct_9fa48("195") ? this.roundRobinIndex * availableAgents.length : (stryCov_9fa48("195"), this.roundRobinIndex % availableAgents.length);
      const selectedAgentId = availableAgents[this.roundRobinIndex];
      stryMutAct_9fa48("196") ? this.roundRobinIndex-- : (stryCov_9fa48("196"), this.roundRobinIndex++);
      return stryMutAct_9fa48("197") ? {} : (stryCov_9fa48("197"), {
        selectedAgentId,
        rationale: "Round-robin selection",
        alternativesConsidered: stryMutAct_9fa48("199") ? availableAgents : (stryCov_9fa48("199"), availableAgents.filter(stryMutAct_9fa48("200") ? () => undefined : (stryCov_9fa48("200"), id => stryMutAct_9fa48("203") ? id === selectedAgentId : stryMutAct_9fa48("202") ? false : stryMutAct_9fa48("201") ? true : (stryCov_9fa48("201", "202", "203"), id !== selectedAgentId))))
      });
    }
  }

  /**
   * Select agent with least load
   */
  private async selectLeastLoaded(agentProfiles: AgentResourceProfile[]): Promise<{
    selectedAgentId: string;
    rationale: string;
    alternativesConsidered: string[];
  }> {
    if (stryMutAct_9fa48("204")) {
      {}
    } else {
      stryCov_9fa48("204");
      if (stryMutAct_9fa48("207") ? agentProfiles.length !== 0 : stryMutAct_9fa48("206") ? false : stryMutAct_9fa48("205") ? true : (stryCov_9fa48("205", "206", "207"), agentProfiles.length === 0)) {
        if (stryMutAct_9fa48("208")) {
          {}
        } else {
          stryCov_9fa48("208");
          throw new Error("No agent profiles available for selection");
        }
      }

      // Calculate load score for each agent
      const agentLoads = agentProfiles.map(stryMutAct_9fa48("210") ? () => undefined : (stryCov_9fa48("210"), profile => stryMutAct_9fa48("211") ? {} : (stryCov_9fa48("211"), {
        agentId: profile.agentId,
        load: stryMutAct_9fa48("212") ? profile.currentTaskCount / profile.maxTaskCapacity * 100 - (profile.cpuUsage.usagePercent + profile.memoryUsage.usagePercent) / 2 : (stryCov_9fa48("212"), (stryMutAct_9fa48("213") ? profile.currentTaskCount / profile.maxTaskCapacity / 100 : (stryCov_9fa48("213"), (stryMutAct_9fa48("214") ? profile.currentTaskCount * profile.maxTaskCapacity : (stryCov_9fa48("214"), profile.currentTaskCount / profile.maxTaskCapacity)) * 100)) + (stryMutAct_9fa48("215") ? (profile.cpuUsage.usagePercent + profile.memoryUsage.usagePercent) * 2 : (stryCov_9fa48("215"), (stryMutAct_9fa48("216") ? profile.cpuUsage.usagePercent - profile.memoryUsage.usagePercent : (stryCov_9fa48("216"), profile.cpuUsage.usagePercent + profile.memoryUsage.usagePercent)) / 2)))
      })));

      // Sort by load (ascending)
      stryMutAct_9fa48("217") ? agentLoads : (stryCov_9fa48("217"), agentLoads.sort(stryMutAct_9fa48("218") ? () => undefined : (stryCov_9fa48("218"), (a, b) => stryMutAct_9fa48("219") ? a.load + b.load : (stryCov_9fa48("219"), a.load - b.load))));
      const selectedAgentId = agentLoads[0].agentId;
      const alternativesConsidered = stryMutAct_9fa48("220") ? agentLoads.map(a => a.agentId) : (stryCov_9fa48("220"), agentLoads.slice(1, 4).map(stryMutAct_9fa48("221") ? () => undefined : (stryCov_9fa48("221"), a => a.agentId)));
      return stryMutAct_9fa48("222") ? {} : (stryCov_9fa48("222"), {
        selectedAgentId,
        rationale: `Least loaded agent (${agentLoads[0].load.toFixed(1)}% load)`,
        alternativesConsidered
      });
    }
  }

  /**
   * Select agent using weighted strategy
   * Weights: 50% task capacity, 30% CPU, 20% memory
   */
  private async selectWeighted(agentProfiles: AgentResourceProfile[]): Promise<{
    selectedAgentId: string;
    rationale: string;
    alternativesConsidered: string[];
  }> {
    if (stryMutAct_9fa48("224")) {
      {}
    } else {
      stryCov_9fa48("224");
      if (stryMutAct_9fa48("227") ? agentProfiles.length !== 0 : stryMutAct_9fa48("226") ? false : stryMutAct_9fa48("225") ? true : (stryCov_9fa48("225", "226", "227"), agentProfiles.length === 0)) {
        if (stryMutAct_9fa48("228")) {
          {}
        } else {
          stryCov_9fa48("228");
          throw new Error("No agent profiles available for selection");
        }
      }

      // Calculate weighted score for each agent (lower is better)
      const agentScores = agentProfiles.map(profile => {
        if (stryMutAct_9fa48("230")) {
          {}
        } else {
          stryCov_9fa48("230");
          const taskLoad = stryMutAct_9fa48("231") ? profile.currentTaskCount / profile.maxTaskCapacity / 100 : (stryCov_9fa48("231"), (stryMutAct_9fa48("232") ? profile.currentTaskCount * profile.maxTaskCapacity : (stryCov_9fa48("232"), profile.currentTaskCount / profile.maxTaskCapacity)) * 100);
          const cpuLoad = profile.cpuUsage.usagePercent;
          const memoryLoad = profile.memoryUsage.usagePercent;
          const score = stryMutAct_9fa48("233") ? taskLoad * 0.5 + cpuLoad * 0.3 - memoryLoad * 0.2 : (stryCov_9fa48("233"), (stryMutAct_9fa48("234") ? taskLoad * 0.5 - cpuLoad * 0.3 : (stryCov_9fa48("234"), (stryMutAct_9fa48("235") ? taskLoad / 0.5 : (stryCov_9fa48("235"), taskLoad * 0.5)) + (stryMutAct_9fa48("236") ? cpuLoad / 0.3 : (stryCov_9fa48("236"), cpuLoad * 0.3)))) + (stryMutAct_9fa48("237") ? memoryLoad / 0.2 : (stryCov_9fa48("237"), memoryLoad * 0.2)));
          return stryMutAct_9fa48("238") ? {} : (stryCov_9fa48("238"), {
            agentId: profile.agentId,
            score
          });
        }
      });

      // Sort by score (ascending)
      stryMutAct_9fa48("239") ? agentScores : (stryCov_9fa48("239"), agentScores.sort(stryMutAct_9fa48("240") ? () => undefined : (stryCov_9fa48("240"), (a, b) => stryMutAct_9fa48("241") ? a.score + b.score : (stryCov_9fa48("241"), a.score - b.score))));
      const selectedAgentId = agentScores[0].agentId;
      const alternativesConsidered = stryMutAct_9fa48("242") ? agentScores.map(a => a.agentId) : (stryCov_9fa48("242"), agentScores.slice(1, 4).map(stryMutAct_9fa48("243") ? () => undefined : (stryCov_9fa48("243"), a => a.agentId)));
      return stryMutAct_9fa48("244") ? {} : (stryCov_9fa48("244"), {
        selectedAgentId,
        rationale: `Weighted selection (score: ${agentScores[0].score.toFixed(1)})`,
        alternativesConsidered
      });
    }
  }

  /**
   * Select agent using priority-based strategy
   * High priority tasks get the best available agents
   */
  private async selectPriorityBased(request: ResourceAllocationRequest, agentProfiles: AgentResourceProfile[]): Promise<{
    selectedAgentId: string;
    rationale: string;
    alternativesConsidered: string[];
  }> {
    if (stryMutAct_9fa48("246")) {
      {}
    } else {
      stryCov_9fa48("246");
      if (stryMutAct_9fa48("249") ? agentProfiles.length !== 0 : stryMutAct_9fa48("248") ? false : stryMutAct_9fa48("247") ? true : (stryCov_9fa48("247", "248", "249"), agentProfiles.length === 0)) {
        if (stryMutAct_9fa48("250")) {
          {}
        } else {
          stryCov_9fa48("250");
          throw new Error("No agent profiles available for selection");
        }
      }

      // Filter healthy agents for high priority tasks
      const healthyAgents = (stryMutAct_9fa48("255") ? request.priority < 75 : stryMutAct_9fa48("254") ? request.priority > 75 : stryMutAct_9fa48("253") ? false : stryMutAct_9fa48("252") ? true : (stryCov_9fa48("252", "253", "254", "255"), request.priority >= 75)) ? stryMutAct_9fa48("256") ? agentProfiles : (stryCov_9fa48("256"), agentProfiles.filter(stryMutAct_9fa48("257") ? () => undefined : (stryCov_9fa48("257"), p => stryMutAct_9fa48("260") ? p.healthStatus !== "healthy" : stryMutAct_9fa48("259") ? false : stryMutAct_9fa48("258") ? true : (stryCov_9fa48("258", "259", "260"), p.healthStatus === "healthy")))) : agentProfiles;
      if (stryMutAct_9fa48("264") ? healthyAgents.length !== 0 : stryMutAct_9fa48("263") ? false : stryMutAct_9fa48("262") ? true : (stryCov_9fa48("262", "263", "264"), healthyAgents.length === 0)) {
        if (stryMutAct_9fa48("265")) {
          {}
        } else {
          stryCov_9fa48("265");
          // Fall back to all agents if no healthy ones
          return this.selectLeastLoaded(agentProfiles);
        }
      }

      // For high priority, select agent with most available capacity
      const agentCapacities = healthyAgents.map(stryMutAct_9fa48("266") ? () => undefined : (stryCov_9fa48("266"), profile => stryMutAct_9fa48("267") ? {} : (stryCov_9fa48("267"), {
        agentId: profile.agentId,
        availableCapacity: stryMutAct_9fa48("268") ? profile.maxTaskCapacity + profile.currentTaskCount : (stryCov_9fa48("268"), profile.maxTaskCapacity - profile.currentTaskCount),
        resourceAvailability: stryMutAct_9fa48("269") ? 100 + (profile.cpuUsage.usagePercent + profile.memoryUsage.usagePercent) / 2 : (stryCov_9fa48("269"), 100 - (stryMutAct_9fa48("270") ? (profile.cpuUsage.usagePercent + profile.memoryUsage.usagePercent) * 2 : (stryCov_9fa48("270"), (stryMutAct_9fa48("271") ? profile.cpuUsage.usagePercent - profile.memoryUsage.usagePercent : (stryCov_9fa48("271"), profile.cpuUsage.usagePercent + profile.memoryUsage.usagePercent)) / 2)))
      })));

      // Sort by available capacity and resource availability
      stryMutAct_9fa48("272") ? agentCapacities : (stryCov_9fa48("272"), agentCapacities.sort(stryMutAct_9fa48("273") ? () => undefined : (stryCov_9fa48("273"), (a, b) => stryMutAct_9fa48("276") ? b.availableCapacity - a.availableCapacity && b.resourceAvailability - a.resourceAvailability : stryMutAct_9fa48("275") ? false : stryMutAct_9fa48("274") ? true : (stryCov_9fa48("274", "275", "276"), (stryMutAct_9fa48("277") ? b.availableCapacity + a.availableCapacity : (stryCov_9fa48("277"), b.availableCapacity - a.availableCapacity)) || (stryMutAct_9fa48("278") ? b.resourceAvailability + a.resourceAvailability : (stryCov_9fa48("278"), b.resourceAvailability - a.resourceAvailability))))));
      const selectedAgentId = agentCapacities[0].agentId;
      const alternativesConsidered = stryMutAct_9fa48("279") ? agentCapacities.map(a => a.agentId) : (stryCov_9fa48("279"), agentCapacities.slice(1, 4).map(stryMutAct_9fa48("280") ? () => undefined : (stryCov_9fa48("280"), a => a.agentId)));
      return stryMutAct_9fa48("281") ? {} : (stryCov_9fa48("281"), {
        selectedAgentId,
        rationale: `Priority-based selection for ${request.priority} priority task`,
        alternativesConsidered
      });
    }
  }

  /**
   * Select agent randomly
   */
  private selectRandom(availableAgents: string[]): {
    selectedAgentId: string;
    rationale: string;
    alternativesConsidered: string[];
  } {
    if (stryMutAct_9fa48("283")) {
      {}
    } else {
      stryCov_9fa48("283");
      const randomIndex = Math.floor(stryMutAct_9fa48("284") ? Math.random() / availableAgents.length : (stryCov_9fa48("284"), Math.random() * availableAgents.length));
      const selectedAgentId = availableAgents[randomIndex];
      return stryMutAct_9fa48("285") ? {} : (stryCov_9fa48("285"), {
        selectedAgentId,
        rationale: "Random selection",
        alternativesConsidered: stryMutAct_9fa48("287") ? availableAgents : (stryCov_9fa48("287"), availableAgents.filter(stryMutAct_9fa48("288") ? () => undefined : (stryCov_9fa48("288"), id => stryMutAct_9fa48("291") ? id === selectedAgentId : stryMutAct_9fa48("290") ? false : stryMutAct_9fa48("289") ? true : (stryCov_9fa48("289", "290", "291"), id !== selectedAgentId))))
      });
    }
  }

  /**
   * Get agent profiles for available agents
   */
  private async getAgentProfiles(agentIds: string[]): Promise<AgentResourceProfile[]> {
    if (stryMutAct_9fa48("292")) {
      {}
    } else {
      stryCov_9fa48("292");
      const profiles: AgentResourceProfile[] = [];
      for (const agentId of agentIds) {
        if (stryMutAct_9fa48("294")) {
          {}
        } else {
          stryCov_9fa48("294");
          const profile = await this.resourceMonitor.getAgentResources(agentId);
          if (stryMutAct_9fa48("296") ? false : stryMutAct_9fa48("295") ? true : (stryCov_9fa48("295", "296"), profile)) {
            if (stryMutAct_9fa48("297")) {
              {}
            } else {
              stryCov_9fa48("297");
              profiles.push(profile);
            }
          }
        }
      }
      return profiles;
    }
  }
}