/**
 * @fileoverview
 * Main interface for thinking budget management.
 * Coordinates complexity analysis, budget allocation, and usage enforcement.
 *
 * @author @darianrosebrook
 */

import {
  BudgetAllocation,
  BudgetMetrics,
  BudgetUsage,
  ComplexityAssessment,
  ComplexityLevel,
  EnforcementResult,
  TaskCharacteristics,
  ThinkingBudgetConfig,
} from "@/types/thinking-budget";
import { BudgetAllocator } from "./BudgetAllocator";
import { TaskComplexityAnalyzer } from "./TaskComplexityAnalyzer";

/**
 * Manages thinking budgets for RL training tasks
 *
 * Provides adaptive token allocation based on task complexity with
 * enforcement of hard ceilings to prevent resource exhaustion.
 */
export class ThinkingBudgetManager {
  private complexityAnalyzer: TaskComplexityAnalyzer;
  private budgetAllocator: BudgetAllocator;
  private config: ThinkingBudgetConfig;
  private metrics: BudgetMetrics;

  /**
   * Creates a new ThinkingBudgetManager
   *
   * @param config Configuration options
   */
  constructor(config?: Partial<ThinkingBudgetConfig>) {
    this.config = {
      strictEnforcement: true,
      maxTrackedAllocations: 10000,
      enableMonitoring: true,
      ...config,
    };

    this.complexityAnalyzer = new TaskComplexityAnalyzer();
    this.budgetAllocator = new BudgetAllocator(
      this.config.budgetTiers,
      this.config.maxTrackedAllocations
    );

    this.metrics = this.initializeMetrics();
  }

  /**
   * Analyzes task complexity and allocates appropriate budget
   *
   * @param characteristics Task characteristics to analyze
   * @returns Budget allocation with complexity assessment
   */
  allocateBudget(characteristics: TaskCharacteristics): {
    allocation: BudgetAllocation;
    assessment: ComplexityAssessment;
  } {
    const startTime = Date.now();

    // Analyze task complexity
    const assessment = this.complexityAnalyzer.analyze(characteristics);

    // Allocate budget based on complexity
    const allocation = this.budgetAllocator.allocate(
      assessment.level,
      characteristics
    );

    // Update metrics
    if (this.config.enableMonitoring) {
      this.updateMetrics(allocation, Date.now() - startTime);
    }

    return { allocation, assessment };
  }

  /**
   * Records token usage for an allocation
   *
   * @param allocationId Allocation ID
   * @param tokensUsed Number of tokens consumed
   * @returns Updated usage state
   */
  recordUsage(allocationId: string, tokensUsed: number): BudgetUsage {
    return this.budgetAllocator.recordUsage(allocationId, tokensUsed);
  }

  /**
   * Enforces budget constraints for a requested usage
   *
   * @param allocationId Allocation ID
   * @param requestedTokens Number of tokens requested
   * @returns Enforcement result
   * @throws Error if strict enforcement enabled and usage denied
   */
  enforceUsage(
    allocationId: string,
    requestedTokens: number
  ): EnforcementResult {
    const result = this.budgetAllocator.enforce(allocationId, requestedTokens);

    if (this.config.strictEnforcement && !result.allowed) {
      throw new Error(
        `Budget enforcement failed: ${result.reason ?? "Unknown reason"}`
      );
    }

    return result;
  }

  /**
   * Gets current usage for an allocation
   *
   * @param allocationId Allocation ID
   * @returns Current usage state or undefined if not found
   */
  getUsage(allocationId: string): BudgetUsage | undefined {
    return this.budgetAllocator.getUsage(allocationId);
  }

  /**
   * Releases an allocation and stops tracking
   *
   * @param allocationId Allocation ID to release
   * @returns True if released successfully
   */
  releaseBudget(allocationId: string): boolean {
    return this.budgetAllocator.release(allocationId);
  }

  /**
   * Gets current budget metrics
   *
   * @returns Current metrics snapshot
   */
  getMetrics(): BudgetMetrics {
    return { ...this.metrics };
  }

  /**
   * Resets all metrics to initial state
   */
  resetMetrics(): void {
    this.metrics = this.initializeMetrics();
  }

  /**
   * Gets count of active allocations
   *
   * @returns Number of active allocations
   */
  getActiveAllocationCount(): number {
    return this.budgetAllocator.getActiveAllocationCount();
  }

  /**
   * Initializes metrics to default values
   *
   * @returns Initial metrics object
   */
  private initializeMetrics(): BudgetMetrics {
    return {
      totalAllocations: 0,
      allocationsByLevel: {
        [ComplexityLevel.TRIVIAL]: 0,
        [ComplexityLevel.STANDARD]: 0,
        [ComplexityLevel.COMPLEX]: 0,
      },
      averageTokensAllocated: 0,
      exhaustionRate: 0,
      averageAllocationTimeMs: 0,
    };
  }

  /**
   * Updates metrics after an allocation
   *
   * @param allocation The allocation that was made
   * @param allocationTimeMs Time taken for allocation
   */
  private updateMetrics(
    allocation: BudgetAllocation,
    allocationTimeMs: number
  ): void {
    // Increment total allocations
    this.metrics.totalAllocations++;

    // Increment level-specific counter
    this.metrics.allocationsByLevel[allocation.complexityLevel]++;

    // Update running average of allocated tokens
    const totalTokens =
      this.metrics.averageTokensAllocated *
        (this.metrics.totalAllocations - 1) +
      allocation.allocatedTokens;
    this.metrics.averageTokensAllocated =
      totalTokens / this.metrics.totalAllocations;

    // Update running average of allocation time
    const totalTime =
      this.metrics.averageAllocationTimeMs *
        (this.metrics.totalAllocations - 1) +
      allocationTimeMs;
    this.metrics.averageAllocationTimeMs =
      totalTime / this.metrics.totalAllocations;

    // Calculate exhaustion rate
    const exhaustedCount = this.budgetAllocator
      .getActiveAllocationIds()
      .filter((id) => {
        const usage = this.budgetAllocator.getUsage(id);
        return usage?.isExhausted;
      }).length;

    const activeCount = this.budgetAllocator.getActiveAllocationCount();
    this.metrics.exhaustionRate =
      activeCount > 0 ? exhaustedCount / activeCount : 0;
  }
}
