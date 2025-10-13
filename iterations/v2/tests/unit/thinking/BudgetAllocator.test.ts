/**
 * @fileoverview
 * Unit tests for BudgetAllocator
 * Tests budget allocation, tracking, and enforcement
 */

import { beforeEach, describe, expect, it } from "@jest/globals";
import { BudgetAllocator } from "../../../src/thinking/BudgetAllocator";
import {
  ComplexityLevel,
  DEFAULT_BUDGET_TIERS,
  TaskCharacteristics,
} from "../../../src/types/thinking-budget";

describe("BudgetAllocator", () => {
  let allocator: BudgetAllocator;
  const mockCharacteristics: TaskCharacteristics = {
    toolCount: 1,
    contextSize: 1000,
    stepCount: 2,
    multiTurn: false,
    hasExternalCalls: false,
  };

  beforeEach(() => {
    allocator = new BudgetAllocator();
  });

  describe("Budget Allocation", () => {
    it("should allocate trivial budget correctly", () => {
      const allocation = allocator.allocate(
        ComplexityLevel.TRIVIAL,
        mockCharacteristics
      );

      expect(allocation.allocatedTokens).toBe(
        DEFAULT_BUDGET_TIERS[ComplexityLevel.TRIVIAL].maxTokens
      );
      expect(allocation.complexityLevel).toBe(ComplexityLevel.TRIVIAL);
      expect(allocation.allocationId).toBeDefined();
      expect(allocation.allocatedAt).toBeInstanceOf(Date);
    });

    it("should allocate standard budget correctly", () => {
      const allocation = allocator.allocate(
        ComplexityLevel.STANDARD,
        mockCharacteristics
      );

      expect(allocation.allocatedTokens).toBe(
        DEFAULT_BUDGET_TIERS[ComplexityLevel.STANDARD].maxTokens
      );
      expect(allocation.complexityLevel).toBe(ComplexityLevel.STANDARD);
    });

    it("should allocate complex budget correctly", () => {
      const allocation = allocator.allocate(
        ComplexityLevel.COMPLEX,
        mockCharacteristics
      );

      expect(allocation.allocatedTokens).toBe(
        DEFAULT_BUDGET_TIERS[ComplexityLevel.COMPLEX].maxTokens
      );
      expect(allocation.complexityLevel).toBe(ComplexityLevel.COMPLEX);
    });

    it("should generate unique allocation IDs", () => {
      const allocation1 = allocator.allocate(
        ComplexityLevel.STANDARD,
        mockCharacteristics
      );
      const allocation2 = allocator.allocate(
        ComplexityLevel.STANDARD,
        mockCharacteristics
      );

      expect(allocation1.allocationId).not.toBe(allocation2.allocationId);
    });

    it("should initialize usage tracking", () => {
      const allocation = allocator.allocate(
        ComplexityLevel.STANDARD,
        mockCharacteristics
      );

      const usage = allocator.getUsage(allocation.allocationId);

      expect(usage).toBeDefined();
      expect(usage?.tokensUsed).toBe(0);
      expect(usage?.tokensRemaining).toBe(allocation.allocatedTokens);
      expect(usage?.isExhausted).toBe(false);
    });

    it("should throw when max tracked allocations exceeded", () => {
      const smallAllocator = new BudgetAllocator(undefined, 2);

      smallAllocator.allocate(ComplexityLevel.TRIVIAL, mockCharacteristics);
      smallAllocator.allocate(ComplexityLevel.TRIVIAL, mockCharacteristics);

      expect(() =>
        smallAllocator.allocate(ComplexityLevel.TRIVIAL, mockCharacteristics)
      ).toThrow(/maximum tracked allocations/i);
    });
  });

  describe("Usage Recording", () => {
    it("should record token usage correctly", () => {
      const allocation = allocator.allocate(
        ComplexityLevel.STANDARD,
        mockCharacteristics
      );

      const usage = allocator.recordUsage(allocation.allocationId, 500);

      expect(usage.tokensUsed).toBe(500);
      expect(usage.tokensRemaining).toBe(allocation.allocatedTokens - 500);
      expect(usage.isExhausted).toBe(false);
    });

    it("should accumulate usage across multiple recordings", () => {
      const allocation = allocator.allocate(
        ComplexityLevel.STANDARD,
        mockCharacteristics
      );

      allocator.recordUsage(allocation.allocationId, 300);
      const usage = allocator.recordUsage(allocation.allocationId, 200);

      expect(usage.tokensUsed).toBe(500);
      expect(usage.tokensRemaining).toBe(allocation.allocatedTokens - 500);
    });

    it("should enforce hard ceiling on usage", () => {
      const allocation = allocator.allocate(
        ComplexityLevel.TRIVIAL,
        mockCharacteristics
      );

      // Try to use more than allocated (500 + 600 = 1100, but ceiling is 500)
      allocator.recordUsage(allocation.allocationId, 400);
      const usage = allocator.recordUsage(allocation.allocationId, 600);

      expect(usage.tokensUsed).toBe(500); // Capped at allocation
      expect(usage.tokensRemaining).toBe(0);
      expect(usage.isExhausted).toBe(true);
    });

    it("should mark budget as exhausted when depleted", () => {
      const allocation = allocator.allocate(
        ComplexityLevel.TRIVIAL,
        mockCharacteristics
      );

      const usage = allocator.recordUsage(
        allocation.allocationId,
        allocation.allocatedTokens
      );

      expect(usage.isExhausted).toBe(true);
      expect(usage.tokensRemaining).toBe(0);
    });

    it("should throw when recording usage for unknown allocation", () => {
      expect(() => allocator.recordUsage("unknown-id", 100)).toThrow(
        /allocation not found/i
      );
    });
  });

  describe("Budget Enforcement", () => {
    it("should allow usage within budget", () => {
      const allocation = allocator.allocate(
        ComplexityLevel.STANDARD,
        mockCharacteristics
      );

      const result = allocator.enforce(allocation.allocationId, 500);

      expect(result.allowed).toBe(true);
      expect(result.reason).toBeUndefined();
    });

    it("should deny usage exceeding remaining budget", () => {
      const allocation = allocator.allocate(
        ComplexityLevel.TRIVIAL,
        mockCharacteristics
      );

      allocator.recordUsage(allocation.allocationId, 400);
      const result = allocator.enforce(allocation.allocationId, 200);

      expect(result.allowed).toBe(false);
      expect(result.reason).toMatch(/exceed remaining budget/i);
    });

    it("should deny usage when budget exhausted", () => {
      const allocation = allocator.allocate(
        ComplexityLevel.TRIVIAL,
        mockCharacteristics
      );

      allocator.recordUsage(allocation.allocationId, 500);
      const result = allocator.enforce(allocation.allocationId, 1);

      expect(result.allowed).toBe(false);
      expect(result.reason).toMatch(/exhausted/i);
    });

    it("should deny usage for unknown allocation", () => {
      const result = allocator.enforce("unknown-id", 100);

      expect(result.allowed).toBe(false);
      expect(result.reason).toMatch(/allocation not found/i);
    });

    it("should provide current usage in enforcement result", () => {
      const allocation = allocator.allocate(
        ComplexityLevel.STANDARD,
        mockCharacteristics
      );

      allocator.recordUsage(allocation.allocationId, 800);
      const result = allocator.enforce(allocation.allocationId, 100);

      expect(result.currentUsage.tokensUsed).toBe(800);
      expect(result.currentUsage.tokensRemaining).toBeGreaterThan(0);
    });
  });

  describe("Allocation Management", () => {
    it("should retrieve usage for active allocation", () => {
      const allocation = allocator.allocate(
        ComplexityLevel.STANDARD,
        mockCharacteristics
      );

      allocator.recordUsage(allocation.allocationId, 300);
      const usage = allocator.getUsage(allocation.allocationId);

      expect(usage).toBeDefined();
      expect(usage?.tokensUsed).toBe(300);
    });

    it("should return undefined for unknown allocation", () => {
      const usage = allocator.getUsage("unknown-id");

      expect(usage).toBeUndefined();
    });

    it("should release allocation successfully", () => {
      const allocation = allocator.allocate(
        ComplexityLevel.STANDARD,
        mockCharacteristics
      );

      const released = allocator.release(allocation.allocationId);

      expect(released).toBe(true);
      expect(allocator.getUsage(allocation.allocationId)).toBeUndefined();
    });

    it("should return false when releasing unknown allocation", () => {
      const released = allocator.release("unknown-id");

      expect(released).toBe(false);
    });

    it("should track active allocation count", () => {
      expect(allocator.getActiveAllocationCount()).toBe(0);

      const alloc1 = allocator.allocate(
        ComplexityLevel.TRIVIAL,
        mockCharacteristics
      );
      expect(allocator.getActiveAllocationCount()).toBe(1);

      const alloc2 = allocator.allocate(
        ComplexityLevel.STANDARD,
        mockCharacteristics
      );
      expect(allocator.getActiveAllocationCount()).toBe(2);

      allocator.release(alloc1.allocationId);
      expect(allocator.getActiveAllocationCount()).toBe(1);

      allocator.release(alloc2.allocationId);
      expect(allocator.getActiveAllocationCount()).toBe(0);
    });

    it("should get all active allocation IDs", () => {
      const alloc1 = allocator.allocate(
        ComplexityLevel.TRIVIAL,
        mockCharacteristics
      );
      const alloc2 = allocator.allocate(
        ComplexityLevel.STANDARD,
        mockCharacteristics
      );

      const ids = allocator.getActiveAllocationIds();

      expect(ids).toContain(alloc1.allocationId);
      expect(ids).toContain(alloc2.allocationId);
      expect(ids.length).toBe(2);
    });

    it("should clear all allocations", () => {
      allocator.allocate(ComplexityLevel.TRIVIAL, mockCharacteristics);
      allocator.allocate(ComplexityLevel.STANDARD, mockCharacteristics);

      expect(allocator.getActiveAllocationCount()).toBe(2);

      allocator.clearAll();

      expect(allocator.getActiveAllocationCount()).toBe(0);
      expect(allocator.getActiveAllocationIds()).toHaveLength(0);
    });
  });

  describe("Concurrent Access", () => {
    it("should handle multiple concurrent allocations", () => {
      const allocations = Array.from({ length: 100 }, () =>
        allocator.allocate(ComplexityLevel.STANDARD, mockCharacteristics)
      );

      expect(allocator.getActiveAllocationCount()).toBe(100);

      allocations.forEach((alloc) => {
        const usage = allocator.getUsage(alloc.allocationId);
        expect(usage).toBeDefined();
        expect(usage?.tokensRemaining).toBe(alloc.allocatedTokens);
      });
    });

    it("should correctly track usage for multiple allocations", () => {
      const alloc1 = allocator.allocate(
        ComplexityLevel.TRIVIAL,
        mockCharacteristics
      );
      const alloc2 = allocator.allocate(
        ComplexityLevel.STANDARD,
        mockCharacteristics
      );

      allocator.recordUsage(alloc1.allocationId, 100);
      allocator.recordUsage(alloc2.allocationId, 500);

      const usage1 = allocator.getUsage(alloc1.allocationId);
      const usage2 = allocator.getUsage(alloc2.allocationId);

      expect(usage1?.tokensUsed).toBe(100);
      expect(usage2?.tokensUsed).toBe(500);
    });
  });
});
