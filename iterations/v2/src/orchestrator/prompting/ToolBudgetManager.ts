/**
 * @fileoverview Tool Budget Manager - Cost Control for Agent Actions
 *
 * Manages tool call budgets to prevent over-exploration while ensuring
 * adequate task completion through intelligent resource allocation.
 *
 * @author @darianrosebrook
 */

import {
  AgentControlConfig,
  BudgetEscalationRule,
  Task,
  TaskComplexity,
  ToolBudget,
} from "../../types/agent-prompting";

/**
 * Budget allocation request
 */
export interface BudgetAllocationRequest {
  /** Task requesting budget */
  task: Task;

  /** Task complexity assessment */
  complexity: TaskComplexity;

  /** Expected budget utilization pattern */
  utilizationPattern: "conservative" | "balanced" | "aggressive";

  /** Time constraints */
  timeConstraints?: {
    maxDurationMs: number;
    priority: "low" | "normal" | "high" | "critical";
  };

  /** Historical budget performance */
  historicalPerformance?: {
    averageUtilization: number;
    successRate: number;
    budgetEfficiency: number;
  };
}

/**
 * Budget allocation result
 */
export interface BudgetAllocationResult {
  /** Allocated budget */
  budget: ToolBudget;

  /** Allocation confidence */
  confidence: number;

  /** Allocation reasoning */
  reasoning: string[];

  /** Recommended monitoring thresholds */
  monitoringThresholds: {
    warningThreshold: number;
    criticalThreshold: number;
    escalationPoints: number[];
  };
}

/**
 * Budget monitoring event
 */
export interface BudgetMonitoringEvent {
  /** Task ID */
  taskId: string;

  /** Budget ID */
  budgetId: string;

  /** Event type */
  eventType: "allocated" | "used" | "escalated" | "exhausted" | "completed";

  /** Tool calls used */
  toolCalls: number;

  /** Timestamp */
  timestamp: Date;

  /** Additional context */
  context?: Record<string, any>;
}

/**
 * Budget performance metrics
 */
export interface BudgetPerformanceMetrics {
  /** Total budgets allocated */
  totalAllocated: number;

  /** Total budgets exhausted */
  totalExhausted: number;

  /** Average utilization rate */
  averageUtilization: number;

  /** Budget efficiency score */
  efficiencyScore: number;

  /** Escalation frequency */
  escalationFrequency: number;

  /** Last updated */
  lastUpdated: Date;
}

/**
 * Tool Budget Manager
 *
 * Manages tool call budgets to optimize agent efficiency and prevent
 * excessive resource consumption while ensuring task completion.
 */
export class ToolBudgetManager {
  private config: AgentControlConfig["toolBudget"];
  private activeBudgets: Map<string, ToolBudget>;
  private budgetMetrics: Map<string, BudgetPerformanceMetrics>;
  private escalationHistory: Map<string, BudgetMonitoringEvent[]>;

  /**
   * Create a new ToolBudgetManager
   */
  constructor(config: AgentControlConfig["toolBudget"]) {
    this.config = config;
    this.activeBudgets = new Map();
    this.budgetMetrics = new Map();
    this.escalationHistory = new Map();

    this.initializeMetrics();
  }

  /**
   * Allocate a tool budget for a task
   */
  async allocateBudget(
    task: Task,
    assessment?: any // TaskAssessment from PromptingEngine
  ): Promise<ToolBudget> {
    const request: BudgetAllocationRequest = {
      task,
      complexity: task.complexity,
      utilizationPattern: this.determineUtilizationPattern(task),
      timeConstraints: this.extractTimeConstraints(task),
      historicalPerformance: await this.getHistoricalPerformance(task),
    };

    const result = await this.calculateOptimalBudget(request);
    const budget = result.budget;

    // Store active budget
    this.activeBudgets.set(task.id, budget);

    // Initialize escalation history
    this.escalationHistory.set(task.id, [
      {
        taskId: task.id,
        budgetId: budget.id || task.id,
        eventType: "allocated",
        toolCalls: 0,
        timestamp: new Date(),
        context: { allocatedCalls: budget.maxCalls },
      },
    ]);

    // Log allocation
    this.logBudgetEvent({
      taskId: task.id,
      budgetId: budget.id || task.id,
      eventType: "allocated",
      toolCalls: budget.maxCalls,
      timestamp: new Date(),
      context: {
        complexity: task.complexity,
        confidence: result.confidence,
        reasoning: result.reasoning,
      },
    });

    return budget;
  }

  /**
   * Track tool usage against budget
   */
  async trackUsage(
    budget: ToolBudget,
    toolCall: any // Would be ToolCall interface
  ): Promise<BudgetStatus> {
    const taskId = this.findTaskIdByBudget(budget);
    if (!taskId) {
      throw new Error("Budget not found in active budgets");
    }

    budget.usedCalls++;

    // Check for escalation triggers
    const escalationTriggered = this.checkEscalationTriggers(budget);

    // Update monitoring
    await this.updateBudgetMonitoring(taskId, budget, toolCall);

    // Determine budget status
    const status = this.calculateBudgetStatus(budget, escalationTriggered);

    // Log usage event
    this.logBudgetEvent({
      taskId,
      budgetId: budget.id || taskId,
      eventType: "used",
      toolCalls: budget.usedCalls,
      timestamp: new Date(),
      context: { status, escalationTriggered },
    });

    return status;
  }

  /**
   * Escalate budget when escalation rules are triggered
   */
  async escalateBudget(budget: ToolBudget, reason: string): Promise<boolean> {
    const taskId = this.findTaskIdByBudget(budget);
    if (!taskId) return false;

    // Check if escalation is allowed
    if (!this.canEscalateBudget(budget)) {
      return false;
    }

    // Find applicable escalation rule
    const rule = this.findEscalationRule(budget);
    if (!rule) return false;

    // Apply escalation
    const oldMax = budget.maxCalls;
    budget.maxCalls += rule.additionalCalls;
    budget.maxCalls = Math.min(budget.maxCalls, rule.maxTotalBudget);

    // Reset escalation tracking
    this.resetEscalationCooldown(budget);

    // Log escalation
    this.logBudgetEvent({
      taskId,
      budgetId: budget.id || taskId,
      eventType: "escalated",
      toolCalls: budget.maxCalls,
      timestamp: new Date(),
      context: {
        reason,
        oldMax,
        newMax: budget.maxCalls,
        rule: rule.trigger,
      },
    });

    return true;
  }

  /**
   * Complete budget tracking for a task
   */
  async completeBudget(taskId: string): Promise<void> {
    const budget = this.activeBudgets.get(taskId);
    if (!budget) return;

    // Calculate final metrics
    const utilization = budget.usedCalls / budget.maxCalls;
    const efficiency = this.calculateBudgetEfficiency(budget);

    // Update performance metrics
    await this.updatePerformanceMetrics(
      taskId,
      budget,
      utilization,
      efficiency
    );

    // Log completion
    this.logBudgetEvent({
      taskId,
      budgetId: budget.id || taskId,
      eventType: "completed",
      toolCalls: budget.usedCalls,
      timestamp: new Date(),
      context: { utilization, efficiency },
    });

    // Clean up
    this.activeBudgets.delete(taskId);
    this.escalationHistory.delete(taskId);
  }

  /**
   * Get active budget count
   */
  async getActiveBudgetCount(): Promise<number> {
    return this.activeBudgets.size;
  }

  /**
   * Get budget performance metrics
   */
  getBudgetMetrics(): Map<string, BudgetPerformanceMetrics> {
    return new Map(this.budgetMetrics);
  }

  /**
   * Update manager configuration
   */
  async updateConfig(
    newConfig: Partial<AgentControlConfig["toolBudget"]>
  ): Promise<void> {
    this.config = { ...this.config, ...newConfig };
  }

  /**
   * Check manager health
   */
  async isHealthy(): Promise<boolean> {
    try {
      const hasValidConfig = this.validateConfig();
      const metricsHealthy = this.budgetMetrics.size > 0;

      return hasValidConfig && metricsHealthy;
    } catch (error) {
      console.error("ToolBudgetManager health check failed:", error);
      return false;
    }
  }

  /**
   * Calculate optimal budget for a request
   */
  private async calculateOptimalBudget(
    request: BudgetAllocationRequest
  ): Promise<BudgetAllocationResult> {
    // Start with configured default for task type
    const baseBudget = this.config.defaultBudgets[request.task.type] || {
      maxCalls: 20,
      usedCalls: 0,
      resetIntervalMs: 3600000, // 1 hour
      lastResetAt: new Date(),
      escalationRules: [],
    };

    let allocatedCalls = baseBudget.maxCalls;
    const reasoning: string[] = [];

    // Adjust based on complexity
    allocatedCalls = this.adjustForComplexity(
      allocatedCalls,
      request.complexity,
      reasoning
    );

    // Adjust based on utilization pattern
    allocatedCalls = this.adjustForUtilizationPattern(
      allocatedCalls,
      request.utilizationPattern,
      reasoning
    );

    // Adjust based on time constraints
    if (request.timeConstraints) {
      allocatedCalls = this.adjustForTimeConstraints(
        allocatedCalls,
        request.timeConstraints,
        reasoning
      );
    }

    // Adjust based on historical performance
    if (request.historicalPerformance) {
      allocatedCalls = this.adjustForHistoricalPerformance(
        allocatedCalls,
        request.historicalPerformance,
        reasoning
      );
    }

    // Apply global limits
    allocatedCalls = Math.min(
      allocatedCalls,
      this.config.globalLimits.totalDailyCalls / 10
    ); // Rough daily limit per task

    // Create budget object
    const budget: ToolBudget = {
      maxCalls: allocatedCalls,
      usedCalls: 0,
      resetIntervalMs: baseBudget.resetIntervalMs,
      lastResetAt: new Date(),
      escalationRules: this.generateEscalationRules(request),
    };

    // Calculate confidence
    const confidence = this.calculateAllocationConfidence(request, budget);

    // Define monitoring thresholds
    const monitoringThresholds = {
      warningThreshold: Math.floor(allocatedCalls * 0.7),
      criticalThreshold: Math.floor(allocatedCalls * 0.9),
      escalationPoints: budget.escalationRules.map(
        (rule) => rule.additionalCalls
      ),
    };

    return {
      budget,
      confidence,
      reasoning,
      monitoringThresholds,
    };
  }

  /**
   * Adjust budget based on task complexity
   */
  private adjustForComplexity(
    baseCalls: number,
    complexity: TaskComplexity,
    reasoning: string[]
  ): number {
    const multipliers = {
      trivial: 0.3,
      standard: 1.0,
      complex: 2.0,
      expert: 3.0,
    };

    const adjusted = Math.floor(baseCalls * multipliers[complexity]);
    reasoning.push(`Complexity ${complexity}: adjusted to ${adjusted} calls`);
    return adjusted;
  }

  /**
   * Adjust budget based on utilization pattern
   */
  private adjustForUtilizationPattern(
    baseCalls: number,
    pattern: BudgetAllocationRequest["utilizationPattern"],
    reasoning: string[]
  ): number {
    const multipliers = {
      conservative: 0.7,
      balanced: 1.0,
      aggressive: 1.5,
    };

    const adjusted = Math.floor(baseCalls * multipliers[pattern]);
    reasoning.push(
      `Utilization pattern ${pattern}: adjusted to ${adjusted} calls`
    );
    return adjusted;
  }

  /**
   * Adjust budget based on time constraints
   */
  private adjustForTimeConstraints(
    baseCalls: number,
    constraints: NonNullable<BudgetAllocationRequest["timeConstraints"]>,
    reasoning: string[]
  ): number {
    let adjusted = baseCalls;

    // High/critical priority gets more budget
    if (constraints.priority === "critical") {
      adjusted = Math.floor(adjusted * 1.5);
      reasoning.push("Critical priority: increased budget by 50%");
    } else if (constraints.priority === "high") {
      adjusted = Math.floor(adjusted * 1.2);
      reasoning.push("High priority: increased budget by 20%");
    }

    // Short time constraints may need more efficient tool usage
    if (constraints.maxDurationMs < 300000) {
      // 5 minutes
      adjusted = Math.floor(adjusted * 0.8);
      reasoning.push(
        "Tight time constraint: reduced budget to encourage efficiency"
      );
    }

    return adjusted;
  }

  /**
   * Adjust budget based on historical performance
   */
  private adjustForHistoricalPerformance(
    baseCalls: number,
    performance: NonNullable<BudgetAllocationRequest["historicalPerformance"]>,
    reasoning: string[]
  ): number {
    let adjusted = baseCalls;

    // High success rate allows optimization
    if (performance.successRate > 0.9) {
      adjusted = Math.floor(adjusted * 0.9);
      reasoning.push("High historical success rate: optimized budget downward");
    }

    // High utilization suggests needs more budget
    if (performance.averageUtilization > 0.8) {
      adjusted = Math.floor(adjusted * 1.1);
      reasoning.push(
        "High historical utilization: increased budget allocation"
      );
    }

    // High efficiency allows slight reduction
    if (performance.budgetEfficiency > 0.85) {
      adjusted = Math.floor(adjusted * 0.95);
      reasoning.push("High historical efficiency: fine-tuned budget");
    }

    return adjusted;
  }

  /**
   * Generate escalation rules for budget
   */
  private generateEscalationRules(
    request: BudgetAllocationRequest
  ): BudgetEscalationRule[] {
    const rules: BudgetEscalationRule[] = [];

    // Base escalation rules
    rules.push({
      trigger: "low-confidence",
      additionalCalls: Math.floor(
        request.task.complexity === "expert" ? 10 : 5
      ),
      maxTotalBudget: request.task.complexity === "expert" ? 100 : 50,
      cooldownMs: 300000, // 5 minutes
    });

    if (request.complexity === "complex" || request.complexity === "expert") {
      rules.push({
        trigger: "verifier-rejection",
        additionalCalls: 8,
        maxTotalBudget: 75,
        cooldownMs: 600000, // 10 minutes
      });
    }

    return rules;
  }

  /**
   * Calculate allocation confidence
   */
  private calculateAllocationConfidence(
    request: BudgetAllocationRequest,
    budget: ToolBudget
  ): number {
    let confidence = 0.8; // Base confidence

    // Higher confidence with historical data
    if (request.historicalPerformance) confidence += 0.1;

    // Lower confidence for complex tasks
    if (request.complexity === "expert") confidence -= 0.1;

    // Adjust based on time constraints
    if (request.timeConstraints?.priority === "critical") confidence -= 0.05;

    return Math.max(0.1, Math.min(1.0, confidence));
  }

  /**
   * Determine utilization pattern for task
   */
  private determineUtilizationPattern(
    task: Task
  ): BudgetAllocationRequest["utilizationPattern"] {
    // Analyze task characteristics to determine pattern
    const description = task.description.toLowerCase();

    if (
      description.includes("quick") ||
      description.includes("fast") ||
      description.includes("simple")
    ) {
      return "conservative";
    }

    if (
      description.includes("thorough") ||
      description.includes("comprehensive") ||
      description.includes("detailed")
    ) {
      return "aggressive";
    }

    return "balanced";
  }

  /**
   * Extract time constraints from task
   */
  private extractTimeConstraints(
    task: Task
  ): BudgetAllocationRequest["timeConstraints"] | undefined {
    // This would analyze task metadata for time constraints
    // For now, return undefined
    return undefined;
  }

  /**
   * Get historical performance for task type
   */
  private async getHistoricalPerformance(
    task: Task
  ): Promise<BudgetAllocationRequest["historicalPerformance"]> {
    // Aggregate historical data for similar tasks
    // This would integrate with your metrics system
    const mockPerformance = {
      averageUtilization: 0.6 + Math.random() * 0.4,
      successRate: 0.7 + Math.random() * 0.3,
      budgetEfficiency: 0.7 + Math.random() * 0.3,
    };

    return mockPerformance;
  }

  /**
   * Find task ID by budget reference
   */
  private findTaskIdByBudget(budget: ToolBudget): string | null {
    for (const [taskId, activeBudget] of this.activeBudgets) {
      if (activeBudget === budget) {
        return taskId;
      }
    }
    return null;
  }

  /**
   * Check if escalation triggers are met
   */
  private checkEscalationTriggers(budget: ToolBudget): boolean {
    if (!budget.escalationRules.length) return false;

    // Check each rule
    for (const rule of budget.escalationRules) {
      // Simple implementation - can be extended based on rule.trigger
      if (budget.usedCalls >= budget.maxCalls * 0.8) {
        return true;
      }
    }

    return false;
  }

  /**
   * Update budget monitoring
   */
  private async updateBudgetMonitoring(
    taskId: string,
    budget: ToolBudget,
    toolCall: any
  ): Promise<void> {
    // Update escalation history
    const history = this.escalationHistory.get(taskId) || [];
    history.push({
      taskId,
      budgetId: budget.id || taskId,
      eventType: "used",
      toolCalls: budget.usedCalls,
      timestamp: new Date(),
      context: { toolCall },
    });

    this.escalationHistory.set(taskId, history);
  }

  /**
   * Calculate budget status
   */
  private calculateBudgetStatus(
    budget: ToolBudget,
    escalationTriggered: boolean
  ): BudgetStatus {
    const utilization = budget.usedCalls / budget.maxCalls;

    if (budget.usedCalls >= budget.maxCalls) {
      return "EXHAUSTED";
    }

    if (escalationTriggered) {
      return "ESCALATION_NEEDED";
    }

    if (utilization > 0.9) {
      return "CRITICAL";
    }

    if (utilization > 0.7) {
      return "WARNING";
    }

    return "HEALTHY";
  }

  /**
   * Check if budget can be escalated
   */
  private canEscalateBudget(budget: ToolBudget): boolean {
    // Check cooldown periods
    const lastEscalation = this.getLastEscalationTime(budget);
    if (lastEscalation) {
      const cooldownMs = 300000; // 5 minutes default
      if (Date.now() - lastEscalation.getTime() < cooldownMs) {
        return false;
      }
    }

    // Check against max total budget
    return budget.maxCalls < 100; // Arbitrary max
  }

  /**
   * Find applicable escalation rule
   */
  private findEscalationRule(budget: ToolBudget): BudgetEscalationRule | null {
    // Find first applicable rule
    return budget.escalationRules[0] || null;
  }

  /**
   * Reset escalation cooldown
   */
  private resetEscalationCooldown(budget: ToolBudget): void {
    // Implementation would track cooldown state
  }

  /**
   * Get last escalation time for budget
   */
  private getLastEscalationTime(budget: ToolBudget): Date | null {
    // Find most recent escalation event
    const taskId = this.findTaskIdByBudget(budget);
    if (!taskId) return null;

    const history = this.escalationHistory.get(taskId) || [];
    const escalations = history.filter(
      (event) => event.eventType === "escalated"
    );

    if (escalations.length === 0) return null;

    return escalations[escalations.length - 1].timestamp;
  }

  /**
   * Calculate budget efficiency
   */
  private calculateBudgetEfficiency(budget: ToolBudget): number {
    if (budget.maxCalls === 0) return 0;

    // Efficiency is inverse of waste (unused calls)
    const utilization = budget.usedCalls / budget.maxCalls;
    return Math.min(utilization * 1.2, 1.0); // Bonus for high utilization
  }

  /**
   * Update performance metrics
   */
  private async updatePerformanceMetrics(
    taskId: string,
    budget: ToolBudget,
    utilization: number,
    efficiency: number
  ): Promise<void> {
    const existing = this.budgetMetrics.get(taskId) || {
      totalAllocated: 0,
      totalExhausted: 0,
      averageUtilization: 0,
      efficiencyScore: 0,
      escalationFrequency: 0,
      lastUpdated: new Date(),
    };

    // Update metrics
    existing.totalAllocated++;
    if (budget.usedCalls >= budget.maxCalls) {
      existing.totalExhausted++;
    }

    // Update averages
    const totalTasks = existing.totalAllocated;
    existing.averageUtilization =
      (existing.averageUtilization * (totalTasks - 1) + utilization) /
      totalTasks;
    existing.efficiencyScore =
      (existing.efficiencyScore * (totalTasks - 1) + efficiency) / totalTasks;

    // Calculate escalation frequency
    const history = this.escalationHistory.get(taskId) || [];
    const escalations = history.filter(
      (event) => event.eventType === "escalated"
    ).length;
    existing.escalationFrequency = escalations / totalTasks;

    existing.lastUpdated = new Date();

    this.budgetMetrics.set(taskId, existing);
  }

  /**
   * Log budget event
   */
  private logBudgetEvent(event: BudgetMonitoringEvent): void {
    console.log("ToolBudgetManager: Budget event:", {
      ...event,
      context: event.context || {},
    });
  }

  /**
   * Initialize metrics
   */
  private initializeMetrics(): void {
    // Initialize with baseline metrics
    this.budgetMetrics.set("global", {
      totalAllocated: 0,
      totalExhausted: 0,
      averageUtilization: 0.6,
      efficiencyScore: 0.75,
      escalationFrequency: 0.1,
      lastUpdated: new Date(),
    });
  }

  /**
   * Validate manager configuration
   */
  private validateConfig(): boolean {
    return !!(
      this.config &&
      this.config.defaultBudgets &&
      this.config.globalLimits
    );
  }
}

/**
 * Budget status enumeration
 */
export type BudgetStatus =
  | "HEALTHY"
  | "WARNING"
  | "CRITICAL"
  | "ESCALATION_NEEDED"
  | "EXHAUSTED";
