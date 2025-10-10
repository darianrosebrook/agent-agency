/**
 * Thinking Budget Manager
 *
 * Manages thinking token allocation, adaptation, and optimization
 * based on task complexity and performance history.
 *
 * @author @darianrosebrook
 */

import {
  AdaptationRecord,
  TaskComplexity,
  ThinkingBudget,
} from "../types/index.js";
import { Logger } from "../utils/Logger.js";

export interface BudgetAllocationRequest {
  taskId: string;
  taskDescription: string;
  complexity?: TaskComplexity;
  context?: Record<string, any>;
}

export interface BudgetAllocationResponse {
  budget: ThinkingBudget;
  recommendation: BudgetRecommendation;
  confidence: number;
}

export interface BudgetRecommendation {
  tokens: number;
  reasoning: string;
  fallbackStrategy: "standard" | "adaptive" | "conservative";
}

export interface ThinkingUsageReport {
  taskId: string;
  allocatedTokens: number;
  usedTokens: number;
  performanceScore: number;
  taskComplexity: TaskComplexity;
  efficiency: number;
  overUnderUsage: number; // Positive = over-used, Negative = under-used
}

export class ThinkingBudgetManager {
  private readonly logger: Logger;
  private readonly budgets = new Map<string, ThinkingBudget>();
  private readonly usageHistory: ThinkingUsageReport[] = [];
  private readonly complexityPatterns = new Map<TaskComplexity, number[]>();

  constructor(logger: Logger) {
    this.logger = logger;
    this.initializeComplexityPatterns();
  }

  /**
   * Allocate thinking budget for a task
   */
  async allocateBudget(
    request: BudgetAllocationRequest
  ): Promise<BudgetAllocationResponse> {
    const estimatedComplexity =
      request.complexity || (await this.estimateComplexity(request));
    const baseTokens = this.getBaseTokensForComplexity(estimatedComplexity);
    const adaptedTokens = await this.adaptBudget(
      baseTokens,
      estimatedComplexity,
      request
    );

    const budget: ThinkingBudget = {
      taskId: request.taskId,
      allocatedTokens: adaptedTokens,
      usedTokens: 0,
      complexity: estimatedComplexity,
      adaptationHistory: [],
    };

    this.budgets.set(request.taskId, budget);

    const recommendation: BudgetRecommendation = {
      tokens: adaptedTokens,
      reasoning: this.generateAllocationReasoning(budget),
      fallbackStrategy: this.determineFallbackStrategy(estimatedComplexity),
    };

    const confidence = this.calculateConfidence(
      estimatedComplexity,
      this.usageHistory.length
    );

    this.logger.debug(
      `Allocated ${adaptedTokens} thinking tokens for task ${request.taskId} (complexity: ${estimatedComplexity})`
    );

    return {
      budget,
      recommendation,
      confidence,
    };
  }

  /**
   * Report thinking token usage after task completion
   */
  async reportUsage(report: ThinkingUsageReport): Promise<void> {
    this.usageHistory.push(report);

    const budget = this.budgets.get(report.taskId);
    if (budget) {
      budget.usedTokens = report.usedTokens;

      // Record adaptation data
      const adaptation: AdaptationRecord = {
        timestamp: new Date().toISOString(),
        complexityEstimate: this.complexityToNumber(report.taskComplexity),
        tokenUsage: report.usedTokens,
        performanceScore: report.performanceScore,
        adjustment: report.overUnderUsage,
      };

      budget.adaptationHistory.push(adaptation);

      // Update complexity patterns for learning
      this.updateComplexityPatterns(report);

      this.logger.debug(
        `Recorded thinking usage for task ${report.taskId}: ${
          report.usedTokens
        }/${budget.allocatedTokens} tokens, efficiency: ${(
          report.efficiency * 100
        ).toFixed(1)}%`
      );
    }
  }

  /**
   * Get current budget for a task
   */
  getBudget(taskId: string): ThinkingBudget | null {
    return this.budgets.get(taskId) || null;
  }

  /**
   * Check if task is within thinking budget
   */
  isWithinBudget(taskId: string, tokensUsed: number): boolean {
    const budget = this.budgets.get(taskId);
    return budget ? tokensUsed <= budget.allocatedTokens : false;
  }

  /**
   * Get remaining tokens for a task
   */
  getRemainingTokens(taskId: string): number {
    const budget = this.budgets.get(taskId);
    return budget ? Math.max(0, budget.allocatedTokens - budget.usedTokens) : 0;
  }

  /**
   * Estimate task complexity from description and context
   */
  private async estimateComplexity(
    request: BudgetAllocationRequest
  ): Promise<TaskComplexity> {
    const { taskDescription, context } = request;

    // Simple heuristic-based estimation (could be enhanced with ML)
    const description = taskDescription.toLowerCase();
    const contextStr = JSON.stringify(context || {}).toLowerCase();

    const complexityIndicators = {
      trivial: ["simple", "basic", "straightforward", "obvious"],
      simple: ["standard", "typical", "common", "regular"],
      moderate: ["complex", "involved", "detailed", "multi-step"],
      complex: ["advanced", "sophisticated", "challenging", "difficult"],
      extreme: ["research", "novel", "breakthrough", "unsolved"],
    };

    // Count complexity indicators
    let complexityScore = 0;
    for (const [level, indicators] of Object.entries(complexityIndicators)) {
      const matches = indicators.filter(
        (indicator) =>
          description.includes(indicator) || contextStr.includes(indicator)
      ).length;
      complexityScore +=
        matches * this.complexityLevelToScore(level as TaskComplexity);
    }

    // Factor in historical performance for similar tasks
    const historicalComplexity = await this.getHistoricalComplexity(
      description
    );
    const finalScore = (complexityScore + historicalComplexity) / 2;

    return this.scoreToComplexity(finalScore);
  }

  /**
   * Adapt budget based on historical performance and patterns
   */
  private async adaptBudget(
    baseTokens: number,
    complexity: TaskComplexity,
    _request: BudgetAllocationRequest
  ): Promise<number> {
    let adaptedTokens = baseTokens;

    // Apply historical adjustments
    const historicalAdjustment = await this.getHistoricalAdjustment(complexity);
    adaptedTokens = Math.round(adaptedTokens * (1 + historicalAdjustment));

    // Apply recent performance trends
    const recentTrend = this.getRecentPerformanceTrend(complexity);
    adaptedTokens = Math.round(adaptedTokens * (1 + recentTrend * 0.1));

    // Ensure reasonable bounds
    const minTokens = 50;
    const maxTokens = 2000;
    adaptedTokens = Math.max(minTokens, Math.min(maxTokens, adaptedTokens));

    return adaptedTokens;
  }

  /**
   * Get base token allocation for complexity level
   */
  private getBaseTokensForComplexity(complexity: TaskComplexity): number {
    const baseAllocations: Record<TaskComplexity, number> = {
      trivial: 100,
      simple: 200,
      moderate: 400,
      complex: 800,
      extreme: 1500,
    };

    return baseAllocations[complexity];
  }

  /**
   * Generate reasoning for budget allocation
   */
  private generateAllocationReasoning(budget: ThinkingBudget): string {
    const efficiency = this.calculateExpectedEfficiency(budget.complexity);
    const historicalData = this.getHistoricalData(budget.complexity);

    return (
      `Allocated ${budget.allocatedTokens} tokens for ${budget.complexity} complexity task. ` +
      `Expected efficiency: ${(efficiency * 100).toFixed(1)}%. ` +
      `Based on ${historicalData.count} similar tasks with ${(
        historicalData.avgEfficiency * 100
      ).toFixed(1)}% average efficiency.`
    );
  }

  /**
   * Determine fallback strategy based on complexity
   */
  private determineFallbackStrategy(
    complexity: TaskComplexity
  ): "standard" | "adaptive" | "conservative" {
    switch (complexity) {
      case "trivial":
      case "simple":
        return "standard";
      case "moderate":
        return "adaptive";
      case "complex":
      case "extreme":
        return "conservative";
      default:
        return "adaptive";
    }
  }

  /**
   * Calculate confidence in allocation
   */
  private calculateConfidence(
    complexity: TaskComplexity,
    historySize: number
  ): number {
    const baseConfidence = 0.5; // Base confidence without history
    const historyBonus = Math.min(0.4, historySize / 100); // Up to 40% bonus with lots of history
    const complexityPenalty = this.complexityToNumber(complexity) * 0.1; // Higher complexity = lower confidence

    return Math.max(
      0.1,
      Math.min(1.0, baseConfidence + historyBonus - complexityPenalty)
    );
  }

  // Helper methods
  private initializeComplexityPatterns(): void {
    // Initialize with reasonable defaults
    this.complexityPatterns.set("trivial", [0.9, 0.85, 0.95]);
    this.complexityPatterns.set("simple", [0.8, 0.75, 0.85]);
    this.complexityPatterns.set("moderate", [0.7, 0.65, 0.75]);
    this.complexityPatterns.set("complex", [0.6, 0.55, 0.65]);
    this.complexityPatterns.set("extreme", [0.5, 0.45, 0.55]);
  }

  private complexityToNumber(complexity: TaskComplexity): number {
    const mapping: Record<TaskComplexity, number> = {
      trivial: 1,
      simple: 2,
      moderate: 3,
      complex: 4,
      extreme: 5,
    };
    return mapping[complexity];
  }

  private complexityLevelToScore(complexity: TaskComplexity): number {
    return this.complexityToNumber(complexity) * 0.2;
  }

  private scoreToComplexity(score: number): TaskComplexity {
    if (score <= 1.5) return "trivial";
    if (score <= 2.5) return "simple";
    if (score <= 3.5) return "moderate";
    if (score <= 4.5) return "complex";
    return "extreme";
  }

  private async getHistoricalComplexity(_description: string): Promise<number> {
    // TODO: Implement similarity matching with historical tasks
    // For now, return neutral score
    return 3.0;
  }

  private async getHistoricalAdjustment(
    complexity: TaskComplexity
  ): Promise<number> {
    const recentReports = this.usageHistory
      .filter((r) => r.taskComplexity === complexity)
      .slice(-10); // Last 10 similar tasks

    if (recentReports.length === 0) return 0;

    const avgOverUnder =
      recentReports.reduce((sum, r) => sum + r.overUnderUsage, 0) /
      recentReports.length;
    return avgOverUnder / 500; // Normalize adjustment
  }

  private getRecentPerformanceTrend(complexity: TaskComplexity): number {
    const recentReports = this.usageHistory
      .filter((r) => r.taskComplexity === complexity)
      .slice(-5); // Last 5 similar tasks

    if (recentReports.length < 2) return 0;

    // Calculate trend in efficiency
    const efficiencies = recentReports.map((r) => r.efficiency);
    const trend = efficiencies[efficiencies.length - 1] - efficiencies[0];

    return trend;
  }

  private updateComplexityPatterns(report: ThinkingUsageReport): void {
    const patterns = this.complexityPatterns.get(report.taskComplexity) || [];
    patterns.push(report.efficiency);

    // Keep only last 20 measurements
    if (patterns.length > 20) {
      patterns.shift();
    }

    this.complexityPatterns.set(report.taskComplexity, patterns);
  }

  private calculateExpectedEfficiency(complexity: TaskComplexity): number {
    const patterns = this.complexityPatterns.get(complexity) || [];
    if (patterns.length === 0) return 0.7; // Default

    return patterns.reduce((sum, eff) => sum + eff, 0) / patterns.length;
  }

  private getHistoricalData(complexity: TaskComplexity): {
    count: number;
    avgEfficiency: number;
  } {
    const reports = this.usageHistory.filter(
      (r) => r.taskComplexity === complexity
    );
    const avgEfficiency =
      reports.length > 0
        ? reports.reduce((sum, r) => sum + r.efficiency, 0) / reports.length
        : 0.7;

    return { count: reports.length, avgEfficiency };
  }

  /**
   * Get budget allocation statistics
   */
  getBudgetStats(): {
    totalAllocations: number;
    averageEfficiency: number;
    overAllocationRate: number;
    underAllocationRate: number;
    complexityDistribution: Record<TaskComplexity, number>;
  } {
    const reports = this.usageHistory;
    if (reports.length === 0) {
      return {
        totalAllocations: 0,
        averageEfficiency: 0,
        overAllocationRate: 0,
        underAllocationRate: 0,
        complexityDistribution: {
          trivial: 0,
          simple: 0,
          moderate: 0,
          complex: 0,
          extreme: 0,
        },
      };
    }

    const avgEfficiency =
      reports.reduce((sum, r) => sum + r.efficiency, 0) / reports.length;
    const overAllocationRate =
      reports.filter((r) => r.overUnderUsage > 0).length / reports.length;
    const underAllocationRate =
      reports.filter((r) => r.overUnderUsage < 0).length / reports.length;

    const complexityDistribution = reports.reduce((dist, r) => {
      dist[r.taskComplexity] = (dist[r.taskComplexity] || 0) + 1;
      return dist;
    }, {} as Record<TaskComplexity, number>);

    return {
      totalAllocations: reports.length,
      averageEfficiency: avgEfficiency,
      overAllocationRate,
      underAllocationRate,
      complexityDistribution,
    };
  }
}
