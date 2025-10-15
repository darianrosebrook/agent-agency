/**
 * @fileoverview
 * Manages token budget allocations and tracking for RL training tasks.
 * Enforces hard ceilings and prevents budget exhaustion.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import {
  BudgetAllocation,
  BudgetTier,
  BudgetUsage,
  ComplexityLevel,
  DEFAULT_BUDGET_TIERS,
  EnforcementResult,
  TaskCharacteristics,
} from "@/types/thinking-budget";
import { randomUUID } from "crypto";

/**
 * Manages budget allocations and usage tracking
 */
export class BudgetAllocator {
  private budgetTiers: Record<ComplexityLevel, BudgetTier>;
  private activeAllocations: Map<string, BudgetAllocation>;
  private usageTracking: Map<string, BudgetUsage>;
  private maxTrackedAllocations: number;

  /**
   * Creates a new BudgetAllocator
   *
   * @param budgetTiers Optional custom budget tiers
   * @param maxTrackedAllocations Maximum number of concurrent allocations to track
   */
  constructor(
    budgetTiers?: Record<ComplexityLevel, BudgetTier>,
    maxTrackedAllocations: number = 10000
  ) {
    this.budgetTiers = budgetTiers ?? DEFAULT_BUDGET_TIERS;
    this.activeAllocations = new Map();
    this.usageTracking = new Map();
    this.maxTrackedAllocations = maxTrackedAllocations;
  }

  /**
   * Allocates a new budget for a task
   *
   * @param complexityLevel Assessed complexity level
   * @param characteristics Task characteristics
   * @returns Budget allocation with ID and token limit
   * @throws Error if maximum tracked allocations exceeded
   */
  allocate(
    complexityLevel: ComplexityLevel,
    characteristics: TaskCharacteristics
  ): BudgetAllocation {
    // Enforce maximum tracked allocations
    if (this.activeAllocations.size >= this.maxTrackedAllocations) {
      throw new Error(
        `Maximum tracked allocations (${this.maxTrackedAllocations}) exceeded`
      );
    }

    const tier = this.budgetTiers[complexityLevel];
    const allocationId = randomUUID();
    const now = new Date();

    const allocation: BudgetAllocation = {
      allocatedTokens: tier.maxTokens,
      complexityLevel,
      allocatedAt: now,
      allocationId,
      taskCharacteristics: characteristics,
    };

    // Track allocation
    this.activeAllocations.set(allocationId, allocation);

    // Initialize usage tracking
    this.usageTracking.set(allocationId, {
      allocationId,
      tokensUsed: 0,
      tokensRemaining: tier.maxTokens,
      isExhausted: false,
      lastUpdated: now,
    });

    return allocation;
  }

  /**
   * Records token usage for an allocation
   *
   * @param allocationId Allocation ID
   * @param tokensUsed Number of tokens consumed
   * @returns Updated usage state
   * @throws Error if allocation not found
   */
  recordUsage(allocationId: string, tokensUsed: number): BudgetUsage {
    const allocation = this.activeAllocations.get(allocationId);
    if (!allocation) {
      throw new Error(`Allocation not found: ${allocationId}`);
    }

    const currentUsage = this.usageTracking.get(allocationId);
    if (!currentUsage) {
      throw new Error(`Usage tracking not found: ${allocationId}`);
    }

    // Calculate new usage (with hard ceiling enforcement)
    const newTokensUsed = Math.min(
      currentUsage.tokensUsed + tokensUsed,
      allocation.allocatedTokens
    );

    const newTokensRemaining = Math.max(
      0,
      allocation.allocatedTokens - newTokensUsed
    );

    const updatedUsage: BudgetUsage = {
      allocationId,
      tokensUsed: newTokensUsed,
      tokensRemaining: newTokensRemaining,
      isExhausted: newTokensRemaining === 0,
      lastUpdated: new Date(),
    };

    this.usageTracking.set(allocationId, updatedUsage);

    return updatedUsage;
  }

  /**
   * Enforces budget constraints for a requested usage
   *
   * @param allocationId Allocation ID
   * @param requestedTokens Number of tokens requested
   * @returns Enforcement result indicating if usage is allowed
   */
  enforce(allocationId: string, requestedTokens: number): EnforcementResult {
    const currentUsage = this.usageTracking.get(allocationId);
    if (!currentUsage) {
      return {
        allowed: false,
        reason: `Allocation not found: ${allocationId}`,
        currentUsage: {
          allocationId,
          tokensUsed: 0,
          tokensRemaining: 0,
          isExhausted: true,
          lastUpdated: new Date(),
        },
      };
    }

    // Check if budget is already exhausted
    if (currentUsage.isExhausted) {
      return {
        allowed: false,
        reason: "Budget exhausted",
        currentUsage,
      };
    }

    // Check if requested tokens exceed remaining budget
    if (requestedTokens > currentUsage.tokensRemaining) {
      return {
        allowed: false,
        reason: `Requested tokens (${requestedTokens}) exceed remaining budget (${currentUsage.tokensRemaining})`,
        currentUsage,
      };
    }

    return {
      allowed: true,
      currentUsage,
    };
  }

  /**
   * Gets current usage for an allocation
   *
   * @param allocationId Allocation ID
   * @returns Current usage state or undefined if not found
   */
  getUsage(allocationId: string): BudgetUsage | undefined {
    return this.usageTracking.get(allocationId);
  }

  /**
   * Releases an allocation and stops tracking usage
   *
   * @param allocationId Allocation ID to release
   * @returns True if allocation was released, false if not found
   */
  release(allocationId: string): boolean {
    const hadAllocation = this.activeAllocations.delete(allocationId);
    const hadUsage = this.usageTracking.delete(allocationId);

    return hadAllocation && hadUsage;
  }

  /**
   * Gets all active allocation IDs
   *
   * @returns Array of active allocation IDs
   */
  getActiveAllocationIds(): string[] {
    return Array.from(this.activeAllocations.keys());
  }

  /**
   * Gets total number of active allocations
   *
   * @returns Count of active allocations
   */
  getActiveAllocationCount(): number {
    return this.activeAllocations.size;
  }

  /**
   * Clears all allocations and usage tracking (for testing)
   */
  clearAll(): void {
    this.activeAllocations.clear();
    this.usageTracking.clear();
  }
}
