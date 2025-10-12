/**
 * @fileoverview Reasoning Effort Controller - Dynamic Effort Selection
 *
 * Manages GPT-5 reasoning effort levels (low/medium/high) based on task characteristics,
 * performance metrics, and optimization goals.
 *
 * @author @darianrosebrook
 */

import {
  AgentBehaviorMetrics,
  AgentControlConfig,
  ReasoningEffort,
  Task,
  TaskComplexity,
  TaskContext,
} from "../../types/agent-prompting";

/**
 * Reasoning effort selection criteria
 */
export interface EffortSelectionCriteria {
  /** Task complexity assessment */
  complexity: TaskComplexity;

  /** Available time budget */
  timeBudgetMs?: number;

  /** Required accuracy level */
  accuracyRequirement: "draft" | "standard" | "high" | "critical";

  /** Historical performance metrics */
  historicalMetrics?: {
    averageCompletionTime: number;
    successRate: number;
    reasoningEfficiency: number;
  };

  /** Current system load */
  systemLoad: "low" | "medium" | "high";

  /** Task urgency level */
  urgency: "low" | "normal" | "high" | "critical";
}

/**
 * Reasoning effort performance metrics
 */
export interface EffortPerformanceMetrics {
  /** Average processing time for this effort level */
  averageProcessingTime: number;

  /** Success rate for this effort level */
  successRate: number;

  /** Average quality score (0-1) */
  averageQualityScore: number;

  /** Sample size for metrics */
  sampleSize: number;

  /** Last updated timestamp */
  lastUpdated: Date;
}

/**
 * Reasoning Effort Controller
 *
 * Dynamically selects optimal GPT-5 reasoning effort levels based on task analysis
 * and performance optimization goals.
 */
export class ReasoningEffortController {
  private config: AgentControlConfig["reasoningEffort"];
  private performanceMetrics: Map<ReasoningEffort, EffortPerformanceMetrics>;
  private adaptiveThresholds: Map<
    TaskComplexity,
    { minScore: number; maxTime: number }
  >;

  /**
   * Create a new ReasoningEffortController
   */
  constructor(config: AgentControlConfig["reasoningEffort"]) {
    this.config = config;
    this.performanceMetrics = new Map();
    this.adaptiveThresholds = new Map();

    this.initializePerformanceMetrics();
    this.initializeAdaptiveThresholds();
  }

  /**
   * Select the optimal reasoning effort for a task
   */
  async selectOptimalEffort(
    task: Task,
    context: TaskContext,
    assessment?: any // TaskAssessment from PromptingEngine
  ): Promise<ReasoningEffort> {
    // assessment parameter can be undefined, so we handle it safely
    // Build selection criteria
    const criteria = this.buildSelectionCriteria(task, context, assessment);

    // Apply selection strategy based on configuration
    if (this.config.dynamicAdjustment) {
      return this.selectWithDynamicAdjustment(criteria);
    } else {
      return this.selectWithStaticMapping(criteria);
    }
  }

  /**
   * Monitor and learn from effort performance
   */
  async monitorPerformance(
    task: Task,
    selectedEffort: ReasoningEffort,
    actualMetrics: AgentBehaviorMetrics
  ): Promise<void> {
    // Update performance metrics for the selected effort level
    await this.updatePerformanceMetrics(selectedEffort, actualMetrics);

    // Adapt thresholds based on performance trends
    await this.adaptThresholds();

    // Log performance insights
    this.logPerformanceInsights(task, selectedEffort, actualMetrics);
  }

  /**
   * Get current performance metrics for all effort levels
   */
  getPerformanceMetrics(): Map<ReasoningEffort, EffortPerformanceMetrics> {
    return new Map(this.performanceMetrics);
  }

  /**
   * Update controller configuration
   */
  async updateConfig(
    newConfig: Partial<AgentControlConfig["reasoningEffort"]>
  ): Promise<void> {
    this.config = { ...this.config, ...newConfig };
  }

  /**
   * Check controller health
   */
  async isHealthy(): Promise<boolean> {
    try {
      // Basic health checks
      const hasValidConfig = this.validateConfig();
      const hasMetrics = this.performanceMetrics.size > 0;

      return hasValidConfig && hasMetrics;
    } catch (error) {
      console.error("ReasoningEffortController health check failed:", error);
      return false;
    }
  }

  /**
   * Build selection criteria from task and context
   */
  private buildSelectionCriteria(
    task: Task,
    context: TaskContext,
    assessment?: any
  ): EffortSelectionCriteria {
    return {
      complexity: task.complexity,
      timeBudgetMs: context.timeBudgetMs,
      accuracyRequirement: context.accuracyRequirement,
      historicalMetrics: context.historicalMetrics
        ? {
            averageCompletionTime:
              context.historicalMetrics.averageCompletionTime,
            successRate: context.historicalMetrics.successRate,
            reasoningEfficiency: context.historicalMetrics.toolEfficiency, // Proxy for reasoning efficiency
          }
        : undefined,
      systemLoad: this.assessSystemLoad(),
      urgency: this.assessUrgency(task.complexity, context.timeBudgetMs),
    };
  }

  /**
   * Select effort with dynamic adjustment based on performance metrics
   */
  private async selectWithDynamicAdjustment(
    criteria: EffortSelectionCriteria
  ): Promise<ReasoningEffort> {
    // Start with static mapping as baseline
    let selectedEffort = this.selectWithStaticMapping(criteria);

    // Apply dynamic adjustments based on performance data
    selectedEffort = await this.applyPerformanceAdjustments(
      selectedEffort,
      criteria
    );

    // Apply system load adjustments
    selectedEffort = this.applySystemLoadAdjustments(
      selectedEffort,
      criteria.systemLoad
    );

    // Apply urgency adjustments
    selectedEffort = this.applyUrgencyAdjustments(
      selectedEffort,
      criteria.urgency
    );

    // Ensure selection is valid
    return this.validateEffortSelection(selectedEffort, criteria);
  }

  /**
   * Select effort using static complexity mapping
   */
  private selectWithStaticMapping(
    criteria: EffortSelectionCriteria
  ): ReasoningEffort {
    // Use configured complexity mapping
    const baseEffort = this.config.complexityMapping[criteria.complexity];

    // Apply accuracy requirement adjustments
    return this.adjustForAccuracy(baseEffort, criteria.accuracyRequirement);
  }

  /**
   * Apply performance-based adjustments to effort selection
   */
  private async applyPerformanceAdjustments(
    baseEffort: ReasoningEffort,
    criteria: EffortSelectionCriteria
  ): Promise<ReasoningEffort> {
    if (!criteria.historicalMetrics) {
      return baseEffort; // No historical data to adjust with
    }

    const { successRate, reasoningEfficiency } = criteria.historicalMetrics;

    // If historical performance is excellent, consider reducing effort
    if (successRate > 0.95 && reasoningEfficiency > 0.9) {
      return this.reduceEffort(baseEffort);
    }

    // If historical performance is poor, consider increasing effort
    if (successRate < 0.7 || reasoningEfficiency < 0.6) {
      return this.increaseEffort(baseEffort);
    }

    return baseEffort;
  }

  /**
   * Apply system load adjustments
   */
  private applySystemLoadAdjustments(
    effort: ReasoningEffort,
    systemLoad: "low" | "medium" | "high"
  ): ReasoningEffort {
    // Reduce effort under high system load to improve responsiveness
    if (systemLoad === "high" && effort === "high") {
      return "medium";
    }

    // Can increase effort when system load is low
    if (systemLoad === "low" && effort === "low") {
      return "medium";
    }

    return effort;
  }

  /**
   * Apply urgency adjustments
   */
  private applyUrgencyAdjustments(
    effort: ReasoningEffort,
    urgency: "low" | "normal" | "high" | "critical"
  ): ReasoningEffort {
    // Reduce effort for critical urgency to prioritize speed
    if ((urgency === "critical" || urgency === "high") && effort === "high") {
      return "medium";
    }

    // Can afford higher effort for low urgency
    if (urgency === "low" && effort === "medium") {
      return "high";
    }

    return effort;
  }

  /**
   * Adjust effort based on accuracy requirements
   */
  private adjustForAccuracy(
    baseEffort: ReasoningEffort,
    accuracy: "draft" | "standard" | "high" | "critical"
  ): ReasoningEffort {
    // Higher accuracy requirements may warrant higher effort
    if (
      (accuracy === "critical" || accuracy === "high") &&
      baseEffort === "low"
    ) {
      return "medium";
    }

    // Lower accuracy requirements can use lower effort
    if (accuracy === "draft" && baseEffort === "medium") {
      return "low";
    }

    return baseEffort;
  }

  /**
   * Reduce effort level (with bounds checking)
   */
  private reduceEffort(effort: ReasoningEffort): ReasoningEffort {
    switch (effort) {
      case "high":
        return "medium";
      case "medium":
        return "low";
      default:
        return effort;
    }
  }

  /**
   * Increase effort level (with bounds checking)
   */
  private increaseEffort(effort: ReasoningEffort): ReasoningEffort {
    switch (effort) {
      case "low":
        return "medium";
      case "medium":
        return "high";
      default:
        return effort;
    }
  }

  /**
   * Validate that the effort selection is appropriate
   */
  private validateEffortSelection(
    effort: ReasoningEffort,
    criteria: EffortSelectionCriteria
  ): ReasoningEffort {
    // Ensure high-complexity tasks always get at least medium effort
    if (
      (criteria.complexity === "complex" || criteria.complexity === "expert") &&
      effort === "low"
    ) {
      return "medium";
    }

    // Ensure critical accuracy always gets at least medium effort
    if (criteria.accuracyRequirement === "critical" && effort === "low") {
      return "medium";
    }

    return effort;
  }

  /**
   * Assess current system load
   */
  private assessSystemLoad(): "low" | "medium" | "high" {
    // This would integrate with your system monitoring
    // For now, return a conservative estimate
    return "medium";
  }

  /**
   * Assess task urgency based on time budget and complexity
   */
  private assessUrgency(
    complexity: TaskComplexity,
    timeBudgetMs?: number
  ): "low" | "normal" | "high" | "critical" {
    if (!timeBudgetMs) return "normal";

    const estimatedTime = this.getEstimatedProcessingTime(complexity);
    const timeRatio = timeBudgetMs / estimatedTime;

    if (timeRatio < 0.3) return "critical";
    if (timeRatio < 0.7) return "high";
    if (timeRatio > 2.0) return "low";
    return "normal";
  }

  /**
   * Get estimated processing time for complexity level
   */
  private getEstimatedProcessingTime(complexity: TaskComplexity): number {
    // These are rough estimates based on typical processing times
    const estimates = {
      trivial: 30000, // 30 seconds
      standard: 180000, // 3 minutes
      complex: 600000, // 10 minutes
      expert: 1800000, // 30 minutes
    };

    return estimates[complexity] || estimates.standard;
  }

  /**
   * Update performance metrics for an effort level
   */
  private async updatePerformanceMetrics(
    effort: ReasoningEffort,
    actualMetrics: AgentBehaviorMetrics
  ): Promise<void> {
    const existing = this.performanceMetrics.get(effort) || {
      averageProcessingTime: 0,
      successRate: 1.0,
      averageQualityScore: 0.8,
      sampleSize: 0,
      lastUpdated: new Date(),
    };

    // Calculate new averages using exponential moving average
    const alpha = 0.1; // Smoothing factor
    const newSampleSize = existing.sampleSize + 1;

    const newAvgTime =
      existing.averageProcessingTime * (1 - alpha) +
      actualMetrics.responseTimeMs * alpha;

    const newSuccessRate =
      existing.successRate * (1 - alpha) +
      (actualMetrics.completionAccuracy > 0.8 ? 1 : 0) * alpha;

    const newQualityScore =
      existing.averageQualityScore * (1 - alpha) +
      actualMetrics.completionAccuracy * alpha;

    this.performanceMetrics.set(effort, {
      averageProcessingTime: newAvgTime,
      successRate: newSuccessRate,
      averageQualityScore: newQualityScore,
      sampleSize: newSampleSize,
      lastUpdated: new Date(),
    });
  }

  /**
   * Adapt thresholds based on performance trends
   */
  private async adaptThresholds(): Promise<void> {
    // Analyze performance trends and adjust adaptive thresholds
    for (const [complexity, thresholds] of this.adaptiveThresholds) {
      const metrics = this.getComplexityMetrics(complexity);
      if (metrics) {
        // Adjust thresholds based on performance
        const performanceScore =
          metrics.successRate * metrics.averageQualityScore;

        if (performanceScore > 0.9) {
          // Performance is excellent, can be more aggressive with lower thresholds
          thresholds.minScore = Math.max(0.7, thresholds.minScore - 0.05);
        } else if (performanceScore < 0.7) {
          // Performance needs improvement, increase thresholds
          thresholds.minScore = Math.min(0.9, thresholds.minScore + 0.05);
        }
      }
    }
  }

  /**
   * Get aggregated metrics for a complexity level
   */
  private getComplexityMetrics(
    complexity: TaskComplexity
  ): EffortPerformanceMetrics | null {
    // Aggregate metrics across effort levels for this complexity
    // This is a simplified implementation
    const relevantEfforts = this.getEffortsForComplexity(complexity);
    const metrics = relevantEfforts
      .map((effort) => this.performanceMetrics.get(effort))
      .filter(Boolean) as EffortPerformanceMetrics[];

    if (metrics.length === 0) return null;

    return {
      averageProcessingTime:
        metrics.reduce((sum, m) => sum + m.averageProcessingTime, 0) /
        metrics.length,
      successRate:
        metrics.reduce((sum, m) => sum + m.successRate, 0) / metrics.length,
      averageQualityScore:
        metrics.reduce((sum, m) => sum + m.averageQualityScore, 0) /
        metrics.length,
      sampleSize: metrics.reduce((sum, m) => sum + m.sampleSize, 0),
      lastUpdated: new Date(),
    };
  }

  /**
   * Get relevant effort levels for a complexity
   */
  private getEffortsForComplexity(
    complexity: TaskComplexity
  ): ReasoningEffort[] {
    switch (complexity) {
      case "trivial":
        return ["low"];
      case "standard":
        return ["low", "medium"];
      case "complex":
        return ["medium", "high"];
      case "expert":
        return ["high"];
      default:
        return ["medium"];
    }
  }

  /**
   * Log performance insights for monitoring
   */
  private logPerformanceInsights(
    task: Task,
    effort: ReasoningEffort,
    metrics: AgentBehaviorMetrics
  ): void {
    const insights = {
      taskId: task.id,
      complexity: task.complexity,
      selectedEffort: effort,
      processingTime: metrics.responseTimeMs,
      accuracy: metrics.completionAccuracy,
      efficiency: metrics.toolEfficiency,
      timestamp: new Date().toISOString(),
    };

    console.log("ReasoningEffortController: Performance insight:", insights);
  }

  /**
   * Initialize default performance metrics
   */
  private initializePerformanceMetrics(): void {
    // Initialize with baseline metrics
    this.performanceMetrics.set("low", {
      averageProcessingTime: 45000, // 45 seconds
      successRate: 0.85,
      averageQualityScore: 0.75,
      sampleSize: 100,
      lastUpdated: new Date(),
    });

    this.performanceMetrics.set("medium", {
      averageProcessingTime: 120000, // 2 minutes
      successRate: 0.92,
      averageQualityScore: 0.85,
      sampleSize: 100,
      lastUpdated: new Date(),
    });

    this.performanceMetrics.set("high", {
      averageProcessingTime: 300000, // 5 minutes
      successRate: 0.96,
      averageQualityScore: 0.92,
      sampleSize: 100,
      lastUpdated: new Date(),
    });
  }

  /**
   * Initialize adaptive thresholds
   */
  private initializeAdaptiveThresholds(): void {
    this.adaptiveThresholds.set("trivial", { minScore: 0.7, maxTime: 60000 });
    this.adaptiveThresholds.set("standard", { minScore: 0.8, maxTime: 180000 });
    this.adaptiveThresholds.set("complex", { minScore: 0.85, maxTime: 600000 });
    this.adaptiveThresholds.set("expert", { minScore: 0.9, maxTime: 1800000 });
  }

  /**
   * Validate controller configuration
   */
  private validateConfig(): boolean {
    return !!(
      this.config &&
      this.config.complexityMapping &&
      this.config.default
    );
  }
}
