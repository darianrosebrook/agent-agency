/**
 * @fileoverview Agent Eagerness Manager - Proactivity Control
 *
 * Manages agent eagerness levels to balance speed vs thoroughness, preventing
 * over-exploration while ensuring adequate task completion.
 *
 * @author @darianrosebrook
 */

import {
  AgentBehaviorMetrics,
  AgentControlConfig,
  AgentEagerness,
  Task,
  TaskComplexity,
  TaskContext,
  TaskType,
} from "../../types/agent-prompting";

/**
 * Eagerness calibration factors
 */
export interface EagernessFactors {
  /** Task type influence */
  taskType: TaskType;

  /** Task complexity influence */
  complexity: TaskComplexity;

  /** Time pressure factor */
  timePressure: "low" | "normal" | "high";

  /** Accuracy requirements */
  accuracyNeeds: "draft" | "standard" | "high" | "critical";

  /** Historical performance patterns */
  historicalPatterns?: {
    overEagernessIncidents: number;
    underPerformanceIncidents: number;
    averageToolCalls: number;
    successRate: number;
  };

  /** Current system context */
  systemContext: {
    concurrentTasks: number;
    systemLoad: "low" | "medium" | "high";
    availableResources: "scarce" | "normal" | "abundant";
  };
}

/**
 * Eagerness performance tracking
 */
export interface EagernessPerformance {
  /** Tool calls made */
  toolCalls: number;

  /** Tasks completed successfully */
  successfulTasks: number;

  /** Average task completion time */
  averageCompletionTime: number;

  /** Over-eagerness incidents (too many tool calls) */
  overEagernessIncidents: number;

  /** Under-performance incidents (insufficient exploration) */
  underPerformanceIncidents: number;

  /** Last updated timestamp */
  lastUpdated: Date;
}

/**
 * Agent Eagerness Manager
 *
 * Calibrates agent proactivity levels to optimize the balance between
 * thorough exploration and efficient task completion.
 */
export class AgentEagernessManager {
  private config: AgentControlConfig["eagerness"];
  private performanceHistory: Map<AgentEagerness, EagernessPerformance>;
  private calibrationThresholds: Map<
    TaskType,
    Map<TaskComplexity, AgentEagerness>
  >;

  /**
   * Create a new AgentEagernessManager
   */
  constructor(config: AgentControlConfig["eagerness"]) {
    this.config = config;
    this.performanceHistory = new Map();
    this.calibrationThresholds = new Map();

    this.initializePerformanceHistory();
    this.initializeCalibrationThresholds();
  }

  /**
   * Calibrate optimal eagerness level for a task
   */
  async calibrateEagerness(
    taskType: TaskType,
    complexity: TaskComplexity,
    context: TaskContext
  ): Promise<AgentEagerness> {
    // Build calibration factors
    const factors = await this.buildCalibrationFactors(
      taskType,
      complexity,
      context
    );

    // Apply eagerness calibration logic
    let calibratedEagerness = this.applyCalibrationLogic(factors);

    // Apply safety bounds and overrides
    calibratedEagerness = this.applySafetyBounds(calibratedEagerness, factors);

    // Apply performance-based adjustments
    calibratedEagerness = await this.applyPerformanceAdjustments(
      calibratedEagerness,
      factors
    );

    // Apply system context adjustments
    calibratedEagerness = this.applySystemAdjustments(
      calibratedEagerness,
      factors.systemContext
    );

    return calibratedEagerness;
  }

  /**
   * Monitor eagerness performance and learn from outcomes
   */
  async monitorEagerness(
    eagerness: AgentEagerness,
    task: Task,
    metrics: AgentBehaviorMetrics
  ): Promise<void> {
    // Update performance history
    await this.updatePerformanceHistory(eagerness, metrics);

    // Analyze for over/under-eagerness patterns
    const analysis = this.analyzeEagernessPatterns(eagerness, metrics);

    // Adapt calibration thresholds
    await this.adaptCalibrationThresholds(analysis);

    // Log insights
    this.logEagernessInsights(eagerness, task, metrics, analysis);
  }

  /**
   * Check if current tool usage indicates over-eagerness
   */
  isOverEager(
    eagerness: AgentEagerness,
    currentToolCalls: number,
    taskComplexity: TaskComplexity
  ): boolean {
    const maxCalls = this.getMaxToolCallsForEagerness(eagerness);

    // Allow some buffer for complex tasks
    const complexityMultiplier = this.getComplexityMultiplier(taskComplexity);
    const adjustedMax = Math.floor(maxCalls * complexityMultiplier);

    return currentToolCalls > adjustedMax;
  }

  /**
   * Suggest eagerness adjustment based on current performance
   */
  suggestEagernessAdjustment(
    currentEagerness: AgentEagerness,
    recentMetrics: AgentBehaviorMetrics[],
    taskType: TaskType
  ): AgentEagerness | null {
    if (recentMetrics.length < 3) return null; // Need minimum sample size

    const avgEfficiency =
      recentMetrics.reduce((sum, m) => sum + m.toolEfficiency, 0) /
      recentMetrics.length;
    const avgAccuracy =
      recentMetrics.reduce((sum, m) => sum + m.completionAccuracy, 0) /
      recentMetrics.length;

    // If efficiency is low but accuracy is high, might be over-eager
    if (avgEfficiency < 0.6 && avgAccuracy > 0.8) {
      return this.reduceEagerness(currentEagerness);
    }

    // If efficiency is high but accuracy is low, might be under-eager
    if (avgEfficiency > 0.9 && avgAccuracy < 0.7) {
      return this.increaseEagerness(currentEagerness);
    }

    // Performance is balanced
    return null;
  }

  /**
   * Get current performance history
   */
  getPerformanceHistory(): Map<AgentEagerness, EagernessPerformance> {
    return new Map(this.performanceHistory);
  }

  /**
   * Update manager configuration
   */
  async updateConfig(
    newConfig: Partial<AgentControlConfig["eagerness"]>
  ): Promise<void> {
    this.config = { ...this.config, ...newConfig };
  }

  /**
   * Check manager health
   */
  async isHealthy(): Promise<boolean> {
    try {
      const hasValidConfig = this.validateConfig();
      const hasPerformanceHistory = this.performanceHistory.size > 0;
      const hasCalibrationThresholds = this.calibrationThresholds.size > 0;

      return (
        hasValidConfig && hasPerformanceHistory && hasCalibrationThresholds
      );
    } catch (error) {
      console.error("AgentEagernessManager health check failed:", error);
      return false;
    }
  }

  /**
   * Build calibration factors from task characteristics
   */
  private async buildCalibrationFactors(
    taskType: TaskType,
    complexity: TaskComplexity,
    context: TaskContext
  ): Promise<EagernessFactors> {
    return {
      taskType,
      complexity,
      timePressure: this.assessTimePressure(context),
      accuracyNeeds: context.accuracyRequirement,
      historicalPatterns: await this.getHistoricalPatterns(taskType, context),
      systemContext: await this.assessSystemContext(),
    };
  }

  /**
   * Apply calibration logic to determine optimal eagerness
   */
  private applyCalibrationLogic(factors: EagernessFactors): AgentEagerness {
    // Start with configured defaults
    let eagerness = this.config.default;

    // Apply task type mapping
    eagerness = this.config.taskTypeMapping[factors.taskType] || eagerness;

    // Apply complexity adjustments
    eagerness = this.adjustForComplexity(eagerness, factors.complexity);

    // Apply time pressure adjustments
    eagerness = this.adjustForTimePressure(eagerness, factors.timePressure);

    // Apply accuracy adjustments
    eagerness = this.adjustForAccuracy(eagerness, factors.accuracyNeeds);

    return eagerness;
  }

  /**
   * Apply safety bounds and critical overrides
   */
  private applySafetyBounds(
    eagerness: AgentEagerness,
    factors: EagernessFactors
  ): AgentEagerness {
    // Critical accuracy always requires thorough approach
    if (factors.accuracyNeeds === "critical" && eagerness === "minimal") {
      return "balanced";
    }

    // High time pressure reduces eagerness
    if (factors.timePressure === "high" && eagerness === "thorough") {
      return "balanced";
    }

    // Complex tasks should not be minimal
    if (factors.complexity === "expert" && eagerness === "minimal") {
      return "balanced";
    }

    return eagerness;
  }

  /**
   * Apply performance-based adjustments
   */
  private async applyPerformanceAdjustments(
    baseEagerness: AgentEagerness,
    factors: EagernessFactors
  ): Promise<AgentEagerness> {
    if (!factors.historicalPatterns) return baseEagerness;

    const { overEagernessIncidents, underPerformanceIncidents, successRate } =
      factors.historicalPatterns;

    // If historical over-eagerness is high, reduce eagerness
    if (overEagernessIncidents > underPerformanceIncidents * 2) {
      return this.reduceEagerness(baseEagerness);
    }

    // If historical under-performance is high, increase eagerness
    if (underPerformanceIncidents > overEagernessIncidents * 2) {
      return this.increaseEagerness(baseEagerness);
    }

    // If success rate is excellent, can optimize for efficiency
    if (successRate > 0.95) {
      return this.reduceEagerness(baseEagerness);
    }

    return baseEagerness;
  }

  /**
   * Apply system context adjustments
   */
  private applySystemAdjustments(
    eagerness: AgentEagerness,
    systemContext: EagernessFactors["systemContext"]
  ): AgentEagerness {
    // High system load reduces eagerness to improve responsiveness
    if (systemContext.systemLoad === "high" && eagerness === "thorough") {
      return "balanced";
    }

    // High concurrency suggests need for efficiency
    if (systemContext.concurrentTasks > 50 && eagerness === "exhaustive") {
      return "thorough";
    }

    // Abundant resources allow higher eagerness
    if (
      systemContext.availableResources === "abundant" &&
      eagerness === "balanced"
    ) {
      return "thorough";
    }

    // Scarce resources require lower eagerness
    if (
      systemContext.availableResources === "scarce" &&
      eagerness === "thorough"
    ) {
      return "balanced";
    }

    return eagerness;
  }

  /**
   * Adjust eagerness based on task complexity
   */
  private adjustForComplexity(
    baseEagerness: AgentEagerness,
    complexity: TaskComplexity
  ): AgentEagerness {
    switch (complexity) {
      case "trivial":
        return baseEagerness === "thorough" ? "balanced" : baseEagerness;
      case "expert":
        return baseEagerness === "minimal" ? "balanced" : baseEagerness;
      default:
        return baseEagerness;
    }
  }

  /**
   * Adjust eagerness based on time pressure
   */
  private adjustForTimePressure(
    baseEagerness: AgentEagerness,
    timePressure: "low" | "normal" | "high"
  ): AgentEagerness {
    switch (timePressure) {
      case "high":
        return this.reduceEagerness(baseEagerness);
      case "low":
        return this.increaseEagerness(baseEagerness);
      default:
        return baseEagerness;
    }
  }

  /**
   * Adjust eagerness based on accuracy needs
   */
  private adjustForAccuracy(
    baseEagerness: AgentEagerness,
    accuracy: "draft" | "standard" | "high" | "critical"
  ): AgentEagerness {
    switch (accuracy) {
      case "draft":
        return this.reduceEagerness(baseEagerness);
      case "critical":
        return this.increaseEagerness(baseEagerness);
      default:
        return baseEagerness;
    }
  }

  /**
   * Reduce eagerness level
   */
  private reduceEagerness(eagerness: AgentEagerness): AgentEagerness {
    switch (eagerness) {
      case "exhaustive":
        return "thorough";
      case "thorough":
        return "balanced";
      case "balanced":
        return "minimal";
      default:
        return eagerness;
    }
  }

  /**
   * Increase eagerness level
   */
  private increaseEagerness(eagerness: AgentEagerness): AgentEagerness {
    switch (eagerness) {
      case "minimal":
        return "balanced";
      case "balanced":
        return "thorough";
      case "thorough":
        return "exhaustive";
      default:
        return eagerness;
    }
  }

  /**
   * Assess time pressure from context
   */
  private assessTimePressure(context: TaskContext): "low" | "normal" | "high" {
    if (!context.timeBudgetMs) return "normal";

    const estimatedTime = this.getEstimatedTaskTime(context);
    const ratio = context.timeBudgetMs / estimatedTime;

    if (ratio < 0.7) return "high";
    if (ratio > 1.5) return "low";
    return "normal";
  }

  /**
   * Get estimated task completion time
   */
  private getEstimatedTaskTime(context: TaskContext): number {
    // Rough estimates based on complexity and type
    const baseTimes = {
      trivial: 60000, // 1 minute
      standard: 300000, // 5 minutes
      complex: 900000, // 15 minutes
      expert: 2700000, // 45 minutes
    };

    const baseTime = baseTimes[context.complexity || "standard"];

    // Adjust based on task type
    const typeMultipliers = {
      analysis: 1.2,
      creation: 1.0,
      modification: 0.8,
      research: 1.5,
      planning: 1.3,
      execution: 0.9,
    };

    return baseTime * (typeMultipliers[context.type || "execution"] || 1.0);
  }

  /**
   * Get historical patterns for task type
   */
  private async getHistoricalPatterns(
    taskType: TaskType,
    context: TaskContext
  ): Promise<EagernessFactors["historicalPatterns"]> {
    // Aggregate historical data for this task type
    // This would integrate with your metrics/monitoring system
    const mockPatterns = {
      overEagernessIncidents: Math.floor(Math.random() * 10),
      underPerformanceIncidents: Math.floor(Math.random() * 8),
      averageToolCalls: 5 + Math.random() * 10,
      successRate: 0.8 + Math.random() * 0.2,
    };

    return mockPatterns;
  }

  /**
   * Assess current system context
   */
  private async assessSystemContext(): Promise<
    EagernessFactors["systemContext"]
  > {
    // This would integrate with your system monitoring
    return {
      concurrentTasks: 25, // Mock value
      systemLoad: "medium",
      availableResources: "normal",
    };
  }

  /**
   * Get maximum tool calls for eagerness level
   */
  private getMaxToolCallsForEagerness(eagerness: AgentEagerness): number {
    const maxCalls = this.config.minimalMaxCalls;

    switch (eagerness) {
      case "minimal":
        return maxCalls;
      case "balanced":
        return maxCalls * 2;
      case "thorough":
        return maxCalls * 4;
      case "exhaustive":
        return maxCalls * 8;
      default:
        return maxCalls * 2;
    }
  }

  /**
   * Get complexity multiplier for tool call limits
   */
  private getComplexityMultiplier(complexity: TaskComplexity): number {
    switch (complexity) {
      case "trivial":
        return 0.5;
      case "standard":
        return 1.0;
      case "complex":
        return 1.5;
      case "expert":
        return 2.0;
      default:
        return 1.0;
    }
  }

  /**
   * Update performance history for eagerness level
   */
  private async updatePerformanceHistory(
    eagerness: AgentEagerness,
    metrics: AgentBehaviorMetrics
  ): Promise<void> {
    const existing = this.performanceHistory.get(eagerness) || {
      toolCalls: 0,
      successfulTasks: 0,
      averageCompletionTime: 0,
      overEagernessIncidents: 0,
      underPerformanceIncidents: 0,
      lastUpdated: new Date(),
    };

    const isSuccessful = metrics.completionAccuracy > 0.8;
    const toolCalls = Math.floor(metrics.toolEfficiency * 10); // Rough estimation

    // Update averages
    const totalTasks = existing.successfulTasks + (isSuccessful ? 1 : 0);
    if (totalTasks > 0) {
      existing.averageCompletionTime =
        (existing.averageCompletionTime * (totalTasks - 1) +
          metrics.responseTimeMs) /
        totalTasks;
    }

    existing.toolCalls += toolCalls;
    if (isSuccessful) existing.successfulTasks++;

    // Detect incidents
    const maxCalls = this.getMaxToolCallsForEagerness(eagerness);
    if (toolCalls > maxCalls * 1.5) {
      existing.overEagernessIncidents++;
    }
    if (!isSuccessful && toolCalls < maxCalls * 0.3) {
      existing.underPerformanceIncidents++;
    }

    existing.lastUpdated = new Date();

    this.performanceHistory.set(eagerness, existing);
  }

  /**
   * Analyze eagerness patterns for insights
   */
  private analyzeEagernessPatterns(
    eagerness: AgentEagerness,
    metrics: AgentBehaviorMetrics
  ): { overEager: boolean; underPerforming: boolean; efficiency: number } {
    const toolCalls = Math.floor(metrics.toolEfficiency * 10);
    const maxCalls = this.getMaxToolCallsForEagerness(eagerness);
    const isSuccessful = metrics.completionAccuracy > 0.8;

    return {
      overEager: toolCalls > maxCalls * 1.2,
      underPerforming: !isSuccessful && toolCalls < maxCalls * 0.5,
      efficiency: metrics.toolEfficiency,
    };
  }

  /**
   * Adapt calibration thresholds based on performance analysis
   */
  private async adaptCalibrationThresholds(analysis: {
    overEager: boolean;
    underPerforming: boolean;
    efficiency: number;
  }): Promise<void> {
    // Adaptive logic to improve future calibrations
    // This would adjust the calibrationThresholds map based on performance patterns
    if (analysis.overEager) {
      // Reduce eagerness recommendations for similar task patterns
      this.adjustThresholdsDown();
    } else if (analysis.underPerforming) {
      // Increase eagerness recommendations for similar patterns
      this.adjustThresholdsUp();
    }
  }

  /**
   * Adjust thresholds downward (reduce eagerness)
   */
  private adjustThresholdsDown(): void {
    // Implementation would adjust calibrationThresholds
    console.log("Adjusting eagerness thresholds downward");
  }

  /**
   * Adjust thresholds upward (increase eagerness)
   */
  private adjustThresholdsUp(): void {
    // Implementation would adjust calibrationThresholds
    console.log("Adjusting eagerness thresholds upward");
  }

  /**
   * Log eagerness insights
   */
  private logEagernessInsights(
    eagerness: AgentEagerness,
    task: Task,
    metrics: AgentBehaviorMetrics,
    analysis: any
  ): void {
    const insights = {
      taskId: task.id,
      eagerness,
      toolEfficiency: metrics.toolEfficiency,
      completionAccuracy: metrics.completionAccuracy,
      overEager: analysis.overEager,
      underPerforming: analysis.underPerforming,
      timestamp: new Date().toISOString(),
    };

    console.log("AgentEagernessManager: Eagerness insight:", insights);
  }

  /**
   * Initialize performance history with baseline data
   */
  private initializePerformanceHistory(): void {
    const baseline: EagernessPerformance = {
      toolCalls: 0,
      successfulTasks: 0,
      averageCompletionTime: 180000, // 3 minutes
      overEagernessIncidents: 0,
      underPerformanceIncidents: 0,
      lastUpdated: new Date(),
    };

    this.performanceHistory.set("minimal", { ...baseline });
    this.performanceHistory.set("balanced", { ...baseline });
    this.performanceHistory.set("thorough", { ...baseline });
    this.performanceHistory.set("exhaustive", { ...baseline });
  }

  /**
   * Initialize calibration thresholds
   */
  private initializeCalibrationThresholds(): void {
    // Initialize default mappings
    const taskTypes: TaskType[] = [
      "analysis",
      "creation",
      "modification",
      "research",
      "planning",
      "execution",
    ];
    const complexities: TaskComplexity[] = [
      "trivial",
      "standard",
      "complex",
      "expert",
    ];

    for (const taskType of taskTypes) {
      this.calibrationThresholds.set(taskType, new Map());

      for (const complexity of complexities) {
        const eagerness =
          this.config.taskTypeMapping[taskType] || this.config.default;
        this.calibrationThresholds.get(taskType)!.set(complexity, eagerness);
      }
    }
  }

  /**
   * Validate manager configuration
   */
  private validateConfig(): boolean {
    return !!(
      this.config &&
      this.config.default &&
      this.config.taskTypeMapping &&
      this.config.minimalMaxCalls > 0
    );
  }
}
