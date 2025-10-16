/**
 * @fileoverview
 * Unit tests for ThinkingBudgetManager
 * Tests integration of complexity analysis, allocation, and enforcement
 */
// @ts-nocheck


import { beforeEach, describe, expect, it } from "@jest/globals";
import { ThinkingBudgetManager } from "../../../src/thinking/ThinkingBudgetManager";
import {
  ComplexityLevel,
  DEFAULT_BUDGET_TIERS,
  TaskCharacteristics,
} from "../../../src/types/thinking-budget";

describe("ThinkingBudgetManager", () => {
  let manager: ThinkingBudgetManager;

  beforeEach(() => {
    manager = new ThinkingBudgetManager();
  });

  describe("Budget Allocation", () => {
    it("should allocate budget based on complexity analysis", () => {
      const characteristics: TaskCharacteristics = {
        toolCount: 0,
        contextSize: 200,
        stepCount: 1,
        multiTurn: false,
        hasExternalCalls: false,
      };

      const { allocation, assessment } =
        manager.allocateBudget(characteristics);

      expect(assessment.level).toBe(ComplexityLevel.TRIVIAL);
      expect(allocation.allocatedTokens).toBe(
        DEFAULT_BUDGET_TIERS[ComplexityLevel.TRIVIAL].maxTokens
      );
      expect(allocation.complexityLevel).toBe(ComplexityLevel.TRIVIAL);
    });

    it("should allocate standard budget for moderate complexity", () => {
      const characteristics: TaskCharacteristics = {
        toolCount: 2,
        contextSize: 2000,
        stepCount: 3,
        multiTurn: false,
        hasExternalCalls: false,
      };

      const { allocation, assessment } =
        manager.allocateBudget(characteristics);

      expect(assessment.level).toBe(ComplexityLevel.STANDARD);
      expect(allocation.allocatedTokens).toBe(
        DEFAULT_BUDGET_TIERS[ComplexityLevel.STANDARD].maxTokens
      );
    });

    it("should allocate complex budget for high complexity", () => {
      const characteristics: TaskCharacteristics = {
        toolCount: 4,
        contextSize: 7000,
        stepCount: 6,
        multiTurn: true,
        hasExternalCalls: true,
      };

      const { allocation, assessment } =
        manager.allocateBudget(characteristics);

      expect(assessment.level).toBe(ComplexityLevel.COMPLEX);
      expect(allocation.allocatedTokens).toBe(
        DEFAULT_BUDGET_TIERS[ComplexityLevel.COMPLEX].maxTokens
      );
    });

    it("should return both allocation and assessment", () => {
      const characteristics: TaskCharacteristics = {
        toolCount: 1,
        contextSize: 1500,
        stepCount: 2,
        multiTurn: false,
        hasExternalCalls: false,
      };

      const result = manager.allocateBudget(characteristics);

      expect(result.allocation).toBeDefined();
      expect(result.assessment).toBeDefined();
      expect(result.allocation.allocationId).toBeDefined();
      expect(result.assessment.reasoning).toBeDefined();
    });

    it("should complete allocation within 50ms", () => {
      const characteristics: TaskCharacteristics = {
        toolCount: 2,
        contextSize: 3000,
        stepCount: 3,
        multiTurn: false,
        hasExternalCalls: true,
      };

      const startTime = Date.now();
      manager.allocateBudget(characteristics);
      const duration = Date.now() - startTime;

      // Should meet P95 budget of 50ms
      expect(duration).toBeLessThan(50);
    });
  });

  describe("Usage Recording", () => {
    it("should record usage successfully", () => {
      const characteristics: TaskCharacteristics = {
        toolCount: 1,
        contextSize: 1000,
        stepCount: 2,
        multiTurn: false,
        hasExternalCalls: false,
      };

      const { allocation } = manager.allocateBudget(characteristics);
      const usage = manager.recordUsage(allocation.allocationId, 500);

      expect(usage.tokensUsed).toBe(500);
      expect(usage.tokensRemaining).toBeGreaterThan(0);
    });

    it("should update usage progressively", () => {
      const characteristics: TaskCharacteristics = {
        toolCount: 1,
        contextSize: 1000,
        stepCount: 2,
        multiTurn: false,
        hasExternalCalls: false,
      };

      const { allocation } = manager.allocateBudget(characteristics);

      manager.recordUsage(allocation.allocationId, 300);
      const usage = manager.recordUsage(allocation.allocationId, 200);

      expect(usage.tokensUsed).toBe(500);
    });
  });

  describe("Budget Enforcement", () => {
    it("should allow usage within budget", () => {
      const characteristics: TaskCharacteristics = {
        toolCount: 0,
        contextSize: 200,
        stepCount: 1,
        multiTurn: false,
        hasExternalCalls: false,
      };

      const { allocation } = manager.allocateBudget(characteristics);
      const result = manager.enforceUsage(allocation.allocationId, 100);

      expect(result.allowed).toBe(true);
    });

    it("should deny usage exceeding budget with lenient enforcement", () => {
      const lenientManager = new ThinkingBudgetManager({
        strictEnforcement: false,
      });

      const characteristics: TaskCharacteristics = {
        toolCount: 0,
        contextSize: 200,
        stepCount: 1,
        multiTurn: false,
        hasExternalCalls: false,
      };

      const { allocation } = lenientManager.allocateBudget(characteristics);
      lenientManager.recordUsage(allocation.allocationId, 400);

      const result = lenientManager.enforceUsage(allocation.allocationId, 200);

      expect(result.allowed).toBe(false);
      expect(result.reason).toMatch(/exceed/i);
    });

    it("should throw when strict enforcement enabled and budget exceeded", () => {
      const strictManager = new ThinkingBudgetManager({
        strictEnforcement: true,
      });

      const characteristics: TaskCharacteristics = {
        toolCount: 0,
        contextSize: 200,
        stepCount: 1,
        multiTurn: false,
        hasExternalCalls: false,
      };

      const { allocation } = strictManager.allocateBudget(characteristics);
      strictManager.recordUsage(allocation.allocationId, 500);

      expect(() =>
        strictManager.enforceUsage(allocation.allocationId, 10)
      ).toThrow(/budget enforcement failed/i);
    });

    it("should not throw when strict enforcement disabled", () => {
      const lenientManager = new ThinkingBudgetManager({
        strictEnforcement: false,
      });

      const characteristics: TaskCharacteristics = {
        toolCount: 0,
        contextSize: 200,
        stepCount: 1,
        multiTurn: false,
        hasExternalCalls: false,
      };

      const { allocation } = lenientManager.allocateBudget(characteristics);
      lenientManager.recordUsage(allocation.allocationId, 500);

      const result = lenientManager.enforceUsage(allocation.allocationId, 10);

      expect(result.allowed).toBe(false);
      // Should not throw
    });
  });

  describe("Usage Retrieval", () => {
    it("should retrieve current usage", () => {
      const characteristics: TaskCharacteristics = {
        toolCount: 1,
        contextSize: 1000,
        stepCount: 2,
        multiTurn: false,
        hasExternalCalls: false,
      };

      const { allocation } = manager.allocateBudget(characteristics);
      manager.recordUsage(allocation.allocationId, 700);

      const usage = manager.getUsage(allocation.allocationId);

      expect(usage).toBeDefined();
      expect(usage?.tokensUsed).toBe(700);
    });

    it("should return undefined for unknown allocation", () => {
      const usage = manager.getUsage("unknown-id");

      expect(usage).toBeUndefined();
    });
  });

  describe("Budget Release", () => {
    it("should release budget successfully", () => {
      const characteristics: TaskCharacteristics = {
        toolCount: 1,
        contextSize: 1000,
        stepCount: 2,
        multiTurn: false,
        hasExternalCalls: false,
      };

      const { allocation } = manager.allocateBudget(characteristics);
      const released = manager.releaseBudget(allocation.allocationId);

      expect(released).toBe(true);
      expect(manager.getUsage(allocation.allocationId)).toBeUndefined();
    });

    it("should return false for unknown allocation", () => {
      const released = manager.releaseBudget("unknown-id");

      expect(released).toBe(false);
    });
  });

  describe("Metrics Tracking", () => {
    it("should initialize metrics correctly", () => {
      const metrics = manager.getMetrics();

      expect(metrics.totalAllocations).toBe(0);
      expect(metrics.averageTokensAllocated).toBe(0);
      expect(metrics.exhaustionRate).toBe(0);
    });

    it("should update metrics after allocation", () => {
      const characteristics: TaskCharacteristics = {
        toolCount: 1,
        contextSize: 1000,
        stepCount: 2,
        multiTurn: false,
        hasExternalCalls: false,
      };

      manager.allocateBudget(characteristics);
      const metrics = manager.getMetrics();

      expect(metrics.totalAllocations).toBe(1);
      expect(metrics.averageTokensAllocated).toBeGreaterThan(0);
    });

    it("should track allocations by complexity level", () => {
      const trivialChars: TaskCharacteristics = {
        toolCount: 0,
        contextSize: 100,
        stepCount: 1,
        multiTurn: false,
        hasExternalCalls: false,
      };

      const complexChars: TaskCharacteristics = {
        toolCount: 3,
        contextSize: 6000,
        stepCount: 5,
        multiTurn: true,
        hasExternalCalls: true,
      };

      manager.allocateBudget(trivialChars);
      manager.allocateBudget(complexChars);

      const metrics = manager.getMetrics();

      expect(metrics.allocationsByLevel[ComplexityLevel.TRIVIAL]).toBe(1);
      expect(metrics.allocationsByLevel[ComplexityLevel.COMPLEX]).toBe(1);
    });

    it("should calculate average tokens allocated", () => {
      const characteristics: TaskCharacteristics = {
        toolCount: 1,
        contextSize: 1000,
        stepCount: 2,
        multiTurn: false,
        hasExternalCalls: false,
      };

      manager.allocateBudget(characteristics);
      manager.allocateBudget(characteristics);

      const metrics = manager.getMetrics();

      expect(metrics.averageTokensAllocated).toBe(
        DEFAULT_BUDGET_TIERS[ComplexityLevel.STANDARD].maxTokens
      );
    });

    it("should reset metrics", () => {
      const characteristics: TaskCharacteristics = {
        toolCount: 1,
        contextSize: 1000,
        stepCount: 2,
        multiTurn: false,
        hasExternalCalls: false,
      };

      manager.allocateBudget(characteristics);
      expect(manager.getMetrics().totalAllocations).toBe(1);

      manager.resetMetrics();

      const metrics = manager.getMetrics();
      expect(metrics.totalAllocations).toBe(0);
      expect(metrics.averageTokensAllocated).toBe(0);
    });

    it("should disable metrics when monitoring disabled", () => {
      const noMonitorManager = new ThinkingBudgetManager({
        enableMonitoring: false,
      });

      const characteristics: TaskCharacteristics = {
        toolCount: 1,
        contextSize: 1000,
        stepCount: 2,
        multiTurn: false,
        hasExternalCalls: false,
      };

      noMonitorManager.allocateBudget(characteristics);

      const metrics = noMonitorManager.getMetrics();

      // Metrics should not update when monitoring disabled
      expect(metrics.totalAllocations).toBe(0);
    });
  });

  describe("Active Allocation Tracking", () => {
    it("should track active allocation count", () => {
      expect(manager.getActiveAllocationCount()).toBe(0);

      const characteristics: TaskCharacteristics = {
        toolCount: 1,
        contextSize: 1000,
        stepCount: 2,
        multiTurn: false,
        hasExternalCalls: false,
      };

      const { allocation } = manager.allocateBudget(characteristics);
      expect(manager.getActiveAllocationCount()).toBe(1);

      manager.releaseBudget(allocation.allocationId);
      expect(manager.getActiveAllocationCount()).toBe(0);
    });
  });

  describe("Performance Requirements", () => {
    it("should handle 500+ allocations per second", () => {
      const characteristics: TaskCharacteristics = {
        toolCount: 1,
        contextSize: 1000,
        stepCount: 2,
        multiTurn: false,
        hasExternalCalls: false,
      };

      const startTime = Date.now();
      const allocationCount = 500;

      for (let i = 0; i < allocationCount; i++) {
        manager.allocateBudget(characteristics);
      }

      const duration = Date.now() - startTime;

      // Should complete 500 allocations in ~1 second
      expect(duration).toBeLessThan(1000);
    });

    it("should track up to 1000 concurrent allocations", () => {
      const characteristics: TaskCharacteristics = {
        toolCount: 1,
        contextSize: 1000,
        stepCount: 2,
        multiTurn: false,
        hasExternalCalls: false,
      };

      for (let i = 0; i < 1000; i++) {
        manager.allocateBudget(characteristics);
      }

      expect(manager.getActiveAllocationCount()).toBe(1000);
    });
  });

  describe("Edge Cases", () => {
    it("should handle zero token usage", () => {
      const characteristics: TaskCharacteristics = {
        toolCount: 0,
        contextSize: 100,
        stepCount: 1,
        multiTurn: false,
        hasExternalCalls: false,
      };

      const { allocation } = manager.allocateBudget(characteristics);
      const usage = manager.recordUsage(allocation.allocationId, 0);

      expect(usage.tokensUsed).toBe(0);
      expect(usage.tokensRemaining).toBe(allocation.allocatedTokens);
    });

    it("should handle exact budget usage", () => {
      const characteristics: TaskCharacteristics = {
        toolCount: 0,
        contextSize: 100,
        stepCount: 1,
        multiTurn: false,
        hasExternalCalls: false,
      };

      const { allocation } = manager.allocateBudget(characteristics);
      const usage = manager.recordUsage(
        allocation.allocationId,
        allocation.allocatedTokens
      );

      expect(usage.tokensUsed).toBe(allocation.allocatedTokens);
      expect(usage.tokensRemaining).toBe(0);
      expect(usage.isExhausted).toBe(true);
    });
  });
});
