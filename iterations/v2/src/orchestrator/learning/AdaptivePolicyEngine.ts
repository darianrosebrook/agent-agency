/**
 * @fileoverview Adaptive Policy Engine - ARBITER-027
 *
 * Loads policy rules from YAML configuration and adjusts task assignment weights,
 * timeout budgets, and retry caps dynamically based on worker performance.
 *
 * @author @darianrosebrook
 */

import { CreditLedger, PerformanceMetrics } from "./CreditLedger";

export interface AdaptivePolicyConfig {
  policies: {
    taskAssignment: {
      enabled: boolean;
      weightAdjustments: {
        excellent: number;
        good: number;
        average: number;
        poor: number;
        critical: number;
      };
      minWeight: number;
      maxWeight: number;
    };
    timeoutBudgets: {
      enabled: boolean;
      multipliers: {
        excellent: number;
        good: number;
        average: number;
        poor: number;
        critical: number;
      };
      minMultiplier: number;
      maxMultiplier: number;
    };
    retryCaps: {
      enabled: boolean;
      adjustments: {
        excellent: number;
        good: number;
        average: number;
        poor: number;
        critical: number;
      };
      minRetries: number;
      maxRetries: number;
    };
    resourceAllocation: {
      enabled: boolean;
      memoryMultipliers: {
        excellent: number;
        good: number;
        average: number;
        poor: number;
        critical: number;
      };
      cpuMultipliers: {
        excellent: number;
        good: number;
        average: number;
        poor: number;
        critical: number;
      };
    };
  };
  thresholds: {
    performanceEvaluationInterval: number; // milliseconds
    policyUpdateInterval: number; // milliseconds
    emergencyThreshold: number; // balance threshold for emergency policies
  };
}

export interface PolicyDecision {
  agentId: string;
  taskAssignmentWeight: number;
  timeoutMultiplier: number;
  retryCap: number;
  memoryMultiplier: number;
  cpuMultiplier: number;
  reasoning: string[];
  lastUpdated: Date;
}

export interface AdaptivePolicyEngine {
  /**
   * Get adjusted weight for task assignment
   */
  adjustWeight(agentId: string): Promise<number>;

  /**
   * Get timeout multiplier for agent
   */
  getTimeoutMultiplier(agentId: string): Promise<number>;

  /**
   * Check if agent should retry based on attempt count
   */
  shouldRetry(agentId: string, attemptCount: number): Promise<boolean>;

  /**
   * Get retry cap for agent
   */
  getRetryCap(agentId: string): Promise<number>;

  /**
   * Get resource allocation multipliers
   */
  getResourceMultipliers(agentId: string): Promise<{
    memory: number;
    cpu: number;
  }>;

  /**
   * Get comprehensive policy decision for agent
   */
  getPolicyDecision(agentId: string): Promise<PolicyDecision>;

  /**
   * Update policies based on recent performance data
   */
  updatePolicies(): Promise<void>;

  /**
   * Get policy engine statistics
   */
  getStatistics(): Promise<{
    totalAgents: number;
    policyDecisions: number;
    lastUpdate: Date;
    config: AdaptivePolicyConfig;
  }>;
}

/**
 * Implementation of AdaptivePolicyEngine with YAML configuration
 */
export class AdaptivePolicyEngineImpl implements AdaptivePolicyEngine {
  private policyCache: Map<string, PolicyDecision> = new Map();
  private lastPolicyUpdate: Date = new Date();
  private lastPerformanceEvaluation: Date = new Date();

  constructor(
    private creditLedger: CreditLedger,
    private config: AdaptivePolicyConfig
  ) {}

  async adjustWeight(agentId: string): Promise<number> {
    const policy = this.config.policies.taskAssignment;

    if (!policy.enabled) {
      return 1.0; // Default weight
    }

    const metrics = await this.creditLedger.getPerformanceMetrics(agentId);
    if (!metrics) {
      return policy.weightAdjustments.average; // Default for new agents
    }

    const baseWeight = policy.weightAdjustments[metrics.performanceTier];
    const adjustedWeight = Math.max(
      policy.minWeight,
      Math.min(policy.maxWeight, baseWeight)
    );

    return adjustedWeight;
  }

  async getTimeoutMultiplier(agentId: string): Promise<number> {
    const policy = this.config.policies.timeoutBudgets;

    if (!policy.enabled) {
      return 1.0; // Default multiplier
    }

    const metrics = await this.creditLedger.getPerformanceMetrics(agentId);
    if (!metrics) {
      return policy.multipliers.average; // Default for new agents
    }

    const baseMultiplier = policy.multipliers[metrics.performanceTier];
    const adjustedMultiplier = Math.max(
      policy.minMultiplier,
      Math.min(policy.maxMultiplier, baseMultiplier)
    );

    return adjustedMultiplier;
  }

  async shouldRetry(agentId: string, attemptCount: number): Promise<boolean> {
    const retryCap = await this.getRetryCap(agentId);
    return attemptCount < retryCap;
  }

  async getRetryCap(agentId: string): Promise<number> {
    const policy = this.config.policies.retryCaps;

    if (!policy.enabled) {
      return 3; // Default retry cap
    }

    const metrics = await this.creditLedger.getPerformanceMetrics(agentId);
    if (!metrics) {
      return policy.adjustments.average; // Default for new agents
    }

    const baseRetries = policy.adjustments[metrics.performanceTier];
    const adjustedRetries = Math.max(
      policy.minRetries,
      Math.min(policy.maxRetries, baseRetries)
    );

    return adjustedRetries;
  }

  async getResourceMultipliers(agentId: string): Promise<{
    memory: number;
    cpu: number;
  }> {
    const policy = this.config.policies.resourceAllocation;

    if (!policy.enabled) {
      return { memory: 1.0, cpu: 1.0 }; // Default multipliers
    }

    const metrics = await this.creditLedger.getPerformanceMetrics(agentId);
    if (!metrics) {
      return {
        memory: policy.memoryMultipliers.average,
        cpu: policy.cpuMultipliers.average,
      };
    }

    return {
      memory: policy.memoryMultipliers[metrics.performanceTier],
      cpu: policy.cpuMultipliers[metrics.performanceTier],
    };
  }

  async getPolicyDecision(agentId: string): Promise<PolicyDecision> {
    // Check cache first
    const cached = this.policyCache.get(agentId);
    if (cached && this.isCacheValid(cached)) {
      return cached;
    }

    // Get performance metrics
    const metrics = await this.creditLedger.getPerformanceMetrics(agentId);
    if (!metrics) {
      // Return default policy for new agents
      return this.createDefaultPolicyDecision(agentId);
    }

    // Calculate policy adjustments
    const taskAssignmentWeight = await this.adjustWeight(agentId);
    const timeoutMultiplier = await this.getTimeoutMultiplier(agentId);
    const retryCap = await this.getRetryCap(agentId);
    const resourceMultipliers = await this.getResourceMultipliers(agentId);

    // Generate reasoning
    const reasoning = this.generateReasoning(metrics, {
      taskAssignmentWeight,
      timeoutMultiplier,
      retryCap,
      resourceMultipliers,
    });

    const decision: PolicyDecision = {
      agentId,
      taskAssignmentWeight,
      timeoutMultiplier,
      retryCap,
      memoryMultiplier: resourceMultipliers.memory,
      cpuMultiplier: resourceMultipliers.cpu,
      reasoning,
      lastUpdated: new Date(),
    };

    // Cache the decision
    this.policyCache.set(agentId, decision);

    return decision;
  }

  async updatePolicies(): Promise<void> {
    const now = new Date();
    const timeSinceLastUpdate = now.getTime() - this.lastPolicyUpdate.getTime();

    if (timeSinceLastUpdate < this.config.thresholds.policyUpdateInterval) {
      return; // Too soon to update
    }

    // Clear cache to force recalculation
    this.policyCache.clear();

    // Update last update time
    this.lastPolicyUpdate = now;

    // Emit policy update event
    this.emitPolicyUpdateEvent();
  }

  async getStatistics(): Promise<{
    totalAgents: number;
    policyDecisions: number;
    lastUpdate: Date;
    config: AdaptivePolicyConfig;
  }> {
    const ledgerStats = await this.creditLedger.getLedgerStatistics();

    return {
      totalAgents: ledgerStats.totalAgents,
      policyDecisions: this.policyCache.size,
      lastUpdate: this.lastPolicyUpdate,
      config: this.config,
    };
  }

  private createDefaultPolicyDecision(agentId: string): PolicyDecision {
    return {
      agentId,
      taskAssignmentWeight:
        this.config.policies.taskAssignment.weightAdjustments.average,
      timeoutMultiplier:
        this.config.policies.timeoutBudgets.multipliers.average,
      retryCap: this.config.policies.retryCaps.adjustments.average,
      memoryMultiplier:
        this.config.policies.resourceAllocation.memoryMultipliers.average,
      cpuMultiplier:
        this.config.policies.resourceAllocation.cpuMultipliers.average,
      reasoning: ["New agent - using default policy settings"],
      lastUpdated: new Date(),
    };
  }

  private generateReasoning(
    metrics: PerformanceMetrics,
    adjustments: {
      taskAssignmentWeight: number;
      timeoutMultiplier: number;
      retryCap: number;
      resourceMultipliers: { memory: number; cpu: number };
    }
  ): string[] {
    const reasoning: string[] = [];

    // Performance tier reasoning
    reasoning.push(`Performance tier: ${metrics.performanceTier}`);
    reasoning.push(
      `Current balance: ${metrics.currentBalance.toFixed(1)} credits`
    );
    reasoning.push(`Success rate: ${(metrics.successRate * 100).toFixed(1)}%`);

    // Task assignment weight reasoning
    if (adjustments.taskAssignmentWeight > 1.0) {
      reasoning.push(
        `Increased task assignment weight (${adjustments.taskAssignmentWeight.toFixed(
          2
        )}) due to strong performance`
      );
    } else if (adjustments.taskAssignmentWeight < 1.0) {
      reasoning.push(
        `Decreased task assignment weight (${adjustments.taskAssignmentWeight.toFixed(
          2
        )}) due to performance concerns`
      );
    }

    // Timeout multiplier reasoning
    if (adjustments.timeoutMultiplier > 1.0) {
      reasoning.push(
        `Increased timeout budget (${adjustments.timeoutMultiplier.toFixed(
          2
        )}x) for reliable completion`
      );
    } else if (adjustments.timeoutMultiplier < 1.0) {
      reasoning.push(
        `Reduced timeout budget (${adjustments.timeoutMultiplier.toFixed(
          2
        )}x) due to efficiency concerns`
      );
    }

    // Retry cap reasoning
    if (adjustments.retryCap > 3) {
      reasoning.push(
        `Increased retry cap (${adjustments.retryCap}) for reliable completion`
      );
    } else if (adjustments.retryCap < 3) {
      reasoning.push(
        `Reduced retry cap (${adjustments.retryCap}) due to repeated failures`
      );
    }

    // Resource allocation reasoning
    if (
      adjustments.resourceMultipliers.memory > 1.0 ||
      adjustments.resourceMultipliers.cpu > 1.0
    ) {
      reasoning.push(`Increased resource allocation for optimal performance`);
    } else if (
      adjustments.resourceMultipliers.memory < 1.0 ||
      adjustments.resourceMultipliers.cpu < 1.0
    ) {
      reasoning.push(`Reduced resource allocation due to efficiency concerns`);
    }

    // CAWS compliance reasoning
    if (metrics.cawsComplianceRate < 0.9) {
      reasoning.push(
        `CAWS compliance issues detected (${(
          metrics.cawsComplianceRate * 100
        ).toFixed(1)}%)`
      );
    }

    // Recent trend reasoning
    if (metrics.recentTrend === "declining") {
      reasoning.push(`Performance trend is declining - monitoring closely`);
    } else if (metrics.recentTrend === "improving") {
      reasoning.push(`Performance trend is improving`);
    }

    return reasoning;
  }

  private isCacheValid(decision: PolicyDecision): boolean {
    const now = new Date();
    const cacheAge = now.getTime() - decision.lastUpdated.getTime();
    return cacheAge < this.config.thresholds.performanceEvaluationInterval;
  }

  private emitPolicyUpdateEvent(): void {
    // This can be extended to use EventEmitter for real-time notifications
    console.log("Policy engine updated", {
      timestamp: this.lastPolicyUpdate,
      cacheSize: this.policyCache.size,
    });
  }

  /**
   * Apply emergency policies for critical performance issues
   */
  async applyEmergencyPolicies(agentId: string): Promise<PolicyDecision> {
    const metrics = await this.creditLedger.getPerformanceMetrics(agentId);

    if (
      !metrics ||
      metrics.currentBalance > this.config.thresholds.emergencyThreshold
    ) {
      return this.getPolicyDecision(agentId);
    }

    // Apply emergency policies for agents with critical balance
    const emergencyDecision: PolicyDecision = {
      agentId,
      taskAssignmentWeight: 0.1, // Minimal task assignment
      timeoutMultiplier: 0.5, // Reduced timeout
      retryCap: 1, // Minimal retries
      memoryMultiplier: 0.5, // Reduced resources
      cpuMultiplier: 0.5,
      reasoning: [
        "Emergency policies applied due to critical performance issues",
        `Balance: ${metrics.currentBalance.toFixed(
          1
        )} credits (below threshold)`,
        "Minimal resources allocated until performance improves",
      ],
      lastUpdated: new Date(),
    };

    this.policyCache.set(agentId, emergencyDecision);
    return emergencyDecision;
  }

  /**
   * Get policy recommendations for improvement
   */
  async getPolicyRecommendations(agentId: string): Promise<{
    recommendations: string[];
    priority: "high" | "medium" | "low";
  }> {
    const metrics = await this.creditLedger.getPerformanceMetrics(agentId);
    if (!metrics) {
      return {
        recommendations: ["New agent - establish performance baseline"],
        priority: "medium",
      };
    }

    const recommendations: string[] = [];
    let priority: "high" | "medium" | "low" = "low";

    // Balance recommendations
    if (metrics.currentBalance < -50) {
      recommendations.push(
        "Critical: Agent balance is severely negative - consider suspension"
      );
      priority = "high";
    } else if (metrics.currentBalance < 0) {
      recommendations.push(
        "High: Agent balance is negative - reduce task assignment"
      );
      priority = priority === "low" ? "high" : priority;
    }

    // Success rate recommendations
    if (metrics.successRate < 0.5) {
      recommendations.push(
        "High: Success rate below 50% - investigate and provide training"
      );
      priority = priority === "low" ? "high" : priority;
    } else if (metrics.successRate < 0.7) {
      recommendations.push("Medium: Success rate below 70% - monitor closely");
      priority = priority === "low" ? "medium" : priority;
    }

    // CAWS compliance recommendations
    if (metrics.cawsComplianceRate < 0.8) {
      recommendations.push(
        "High: CAWS compliance issues - mandatory training required"
      );
      priority = priority === "low" ? "high" : priority;
    }

    // Recent trend recommendations
    if (metrics.recentTrend === "declining") {
      recommendations.push(
        "Medium: Performance declining - investigate root causes"
      );
      priority = priority === "low" ? "medium" : priority;
    }

    // Arbitration performance recommendations
    if (metrics.arbitrationWinRate < 0.3) {
      recommendations.push(
        "Medium: Low arbitration win rate - review decision quality"
      );
      priority = priority === "low" ? "medium" : priority;
    }

    return { recommendations, priority };
  }
}

