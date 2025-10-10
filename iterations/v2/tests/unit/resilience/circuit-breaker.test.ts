/**
 * @fileoverview Tests for Circuit Breaker Pattern
 *
 * Tests circuit breaker state transitions, failure thresholds, recovery, etc.
 *
 * @author @darianrosebrook
 */

import {
  CircuitBreaker,
  CircuitBreakerOpenError,
  CircuitState,
} from "../../../src/resilience/CircuitBreaker";

describe("CircuitBreaker", () => {
  let circuitBreaker: CircuitBreaker;

  beforeEach(() => {
    circuitBreaker = new CircuitBreaker({
      name: "TestBreaker",
      failureThreshold: 3,
      failureWindowMs: 1000,
      resetTimeoutMs: 500,
      successThreshold: 2,
    });
  });

  describe("Initial State", () => {
    it("should start in CLOSED state", () => {
      expect(circuitBreaker.getState()).toBe(CircuitState.CLOSED);
    });

    it("should have zero failures and successes", () => {
      const stats = circuitBreaker.getStats();
      expect(stats.failures).toBe(0);
      expect(stats.successes).toBe(0);
      expect(stats.totalRequests).toBe(0);
    });
  });

  describe("Successful Operations", () => {
    it("should remain CLOSED on successful operations", async () => {
      const mockFn = jest.fn().mockResolvedValue("success");

      const result = await circuitBreaker.execute(mockFn);

      expect(result).toBe("success");
      expect(circuitBreaker.getState()).toBe(CircuitState.CLOSED);
      expect(circuitBreaker.getStats().successes).toBe(1);
      expect(circuitBreaker.getStats().totalRequests).toBe(1);
    });

    it("should handle multiple successful operations", async () => {
      const mockFn = jest.fn().mockResolvedValue("success");

      await circuitBreaker.execute(mockFn);
      await circuitBreaker.execute(mockFn);
      await circuitBreaker.execute(mockFn);

      expect(circuitBreaker.getState()).toBe(CircuitState.CLOSED);
      expect(circuitBreaker.getStats().successes).toBe(3);
      expect(circuitBreaker.getStats().totalRequests).toBe(3);
    });
  });

  describe("Failure Handling", () => {
    it("should remain CLOSED when failures are below threshold", async () => {
      const mockFn = jest.fn().mockRejectedValue(new Error("test error"));

      await expect(circuitBreaker.execute(mockFn)).rejects.toThrow(
        "test error"
      );
      await expect(circuitBreaker.execute(mockFn)).rejects.toThrow(
        "test error"
      );

      expect(circuitBreaker.getState()).toBe(CircuitState.CLOSED);
      expect(circuitBreaker.getStats().failures).toBe(2);
    });

    it("should transition to OPEN when failure threshold is reached", async () => {
      const mockFn = jest.fn().mockRejectedValue(new Error("test error"));

      // Fail 3 times (threshold)
      for (let i = 0; i < 3; i++) {
        await expect(circuitBreaker.execute(mockFn)).rejects.toThrow(
          "test error"
        );
      }

      expect(circuitBreaker.getState()).toBe(CircuitState.OPEN);
      expect(circuitBreaker.getStats().failures).toBe(3);
    });

    it("should reject requests when circuit is OPEN", async () => {
      const mockFn = jest.fn().mockRejectedValue(new Error("test error"));

      // Open the circuit
      for (let i = 0; i < 3; i++) {
        await expect(circuitBreaker.execute(mockFn)).rejects.toThrow(
          "test error"
        );
      }

      // Now requests should be rejected
      await expect(circuitBreaker.execute(mockFn)).rejects.toThrow(
        CircuitBreakerOpenError
      );
      expect(circuitBreaker.getStats().totalRequests).toBe(4); // 3 failures + 1 rejected
    });
  });

  describe("Recovery Mechanism", () => {
    it("should transition to HALF_OPEN after reset timeout", async () => {
      const mockFn = jest.fn().mockRejectedValue(new Error("test error"));

      // Open the circuit
      for (let i = 0; i < 3; i++) {
        await expect(circuitBreaker.execute(mockFn)).rejects.toThrow(
          "test error"
        );
      }

      expect(circuitBreaker.getState()).toBe(CircuitState.OPEN);

      // Wait for reset timeout
      await new Promise((resolve) => setTimeout(resolve, 600));

      // Next request should attempt recovery but fail, returning to OPEN
      await expect(circuitBreaker.execute(mockFn)).rejects.toThrow(
        "test error"
      );
      expect(circuitBreaker.getState()).toBe(CircuitState.OPEN);
    });

    it("should transition back to CLOSED after success threshold in HALF_OPEN", async () => {
      const mockFn = jest.fn().mockRejectedValue(new Error("test error"));

      // Open the circuit
      for (let i = 0; i < 3; i++) {
        await expect(circuitBreaker.execute(mockFn)).rejects.toThrow(
          "test error"
        );
      }

      // Wait for reset timeout
      await new Promise((resolve) => setTimeout(resolve, 600));

      // Now in HALF_OPEN - need 2 successes to close
      const successFn = jest.fn().mockResolvedValue("success");

      await circuitBreaker.execute(successFn);
      expect(circuitBreaker.getState()).toBe(CircuitState.HALF_OPEN);

      await circuitBreaker.execute(successFn);
      expect(circuitBreaker.getState()).toBe(CircuitState.CLOSED);
    });

    it("should transition back to OPEN on failure in HALF_OPEN", async () => {
      const mockFn = jest.fn().mockRejectedValue(new Error("test error"));

      // Open the circuit
      for (let i = 0; i < 3; i++) {
        await expect(circuitBreaker.execute(mockFn)).rejects.toThrow(
          "test error"
        );
      }

      // Wait for reset timeout and transition to HALF_OPEN
      await new Promise((resolve) => setTimeout(resolve, 600));
      await expect(circuitBreaker.execute(mockFn)).rejects.toThrow(
        "test error"
      );

      // Failure in HALF_OPEN should immediately open circuit
      expect(circuitBreaker.getState()).toBe(CircuitState.OPEN);
    });
  });

  describe("Time Window Behavior", () => {
    it("should reset failure count after time window", async () => {
      const mockFn = jest.fn().mockRejectedValue(new Error("test error"));

      // Create circuit breaker with very short window
      const shortWindowBreaker = new CircuitBreaker({
        name: "ShortWindowBreaker",
        failureThreshold: 3,
        failureWindowMs: 100, // 100ms window
        resetTimeoutMs: 500,
        successThreshold: 2,
      });

      // Fail twice quickly
      await expect(shortWindowBreaker.execute(mockFn)).rejects.toThrow(
        "test error"
      );
      await expect(shortWindowBreaker.execute(mockFn)).rejects.toThrow(
        "test error"
      );

      expect(shortWindowBreaker.getState()).toBe(CircuitState.CLOSED);

      // Wait for time window to pass
      await new Promise((resolve) => setTimeout(resolve, 150));

      // Third failure should be the only one in the new window (previous failures expired)
      await expect(shortWindowBreaker.execute(mockFn)).rejects.toThrow(
        "test error"
      );

      expect(shortWindowBreaker.getState()).toBe(CircuitState.CLOSED); // Only 1 failure in window, below threshold
    });
  });

  describe("Error Types", () => {
    it("should include circuit name in open error", async () => {
      const mockFn = jest.fn().mockRejectedValue(new Error("test error"));

      // Open the circuit
      for (let i = 0; i < 3; i++) {
        await expect(circuitBreaker.execute(mockFn)).rejects.toThrow(
          "test error"
        );
      }

      try {
        await circuitBreaker.execute(mockFn);
        fail("Should have thrown CircuitBreakerOpenError");
      } catch (error) {
        expect(error).toBeInstanceOf(CircuitBreakerOpenError);
        expect((error as CircuitBreakerOpenError).circuitName).toBe(
          "TestBreaker"
        );
        expect((error as CircuitBreakerOpenError).stats.failures).toBe(3);
      }
    });
  });

  describe("Manual Reset", () => {
    it("should allow manual reset of circuit breaker", async () => {
      const mockFn = jest.fn().mockRejectedValue(new Error("test error"));

      // Open the circuit
      for (let i = 0; i < 3; i++) {
        await expect(circuitBreaker.execute(mockFn)).rejects.toThrow(
          "test error"
        );
      }

      expect(circuitBreaker.getState()).toBe(CircuitState.OPEN);

      // Manual reset
      circuitBreaker.reset();

      expect(circuitBreaker.getState()).toBe(CircuitState.CLOSED);
      expect(circuitBreaker.getStats().failures).toBe(0);
      expect(circuitBreaker.getStats().successes).toBe(0);
    });
  });
});
