/**
 * @fileoverview
 * Unit tests for RetryManager
 *
 * @author @darianrosebrook
 */

import {
  CircuitBreakerState,
  RetryManager,
} from "@/evaluation/retry/RetryManager";

describe("RetryManager", () => {
  let retryManager: RetryManager;

  beforeEach(() => {
    retryManager = new RetryManager({
      maxRetries: 2,
      baseDelayMs: 10,
      maxDelayMs: 100,
      backoffMultiplier: 2,
      jitterFactor: 0.1,
      circuitBreakerThreshold: 3,
      circuitBreakerTimeoutMs: 100,
      enableCircuitBreaker: true,
    });
    // Reset circuit breaker state before each test
    retryManager.resetCircuitBreaker();
  });

  describe("executeWithRetry", () => {
    it("should succeed on first attempt", async () => {
      const operation = jest.fn().mockResolvedValue("success");

      const result = await retryManager.executeWithRetry(operation, "test");

      expect(result.success).toBe(true);
      expect(result.result).toBe("success");
      expect(result.totalAttempts).toBe(1);
      expect(result.circuitBreakerTriggered).toBe(false);
      expect(operation).toHaveBeenCalledTimes(1);
    });

    it("should retry on failure and eventually succeed", async () => {
      const operation = jest
        .fn()
        .mockRejectedValueOnce(new Error("network error"))
        .mockRejectedValueOnce(new Error("timeout"))
        .mockResolvedValue("success");

      const result = await retryManager.executeWithRetry(operation, "test");

      expect(result.success).toBe(true);
      expect(result.result).toBe("success");
      expect(result.totalAttempts).toBe(3);
      expect(result.attempts).toHaveLength(2); // Only failed attempts are recorded
      expect(operation).toHaveBeenCalledTimes(3);
    });

    it("should fail after max retries", async () => {
      const operation = jest
        .fn()
        .mockRejectedValue(new Error("persistent error"));

      const result = await retryManager.executeWithRetry(operation, "test");

      expect(result.success).toBe(false);
      expect(result.error?.message).toBe("persistent error");
      expect(result.totalAttempts).toBe(3); // 1 initial + 2 retries
      expect(result.attempts).toHaveLength(3);
      expect(operation).toHaveBeenCalledTimes(3);
    });

    it("should not retry non-retryable errors", async () => {
      const operation = jest
        .fn()
        .mockRejectedValue(new Error("authentication failed"));

      const result = await retryManager.executeWithRetry(operation, "test");

      expect(result.success).toBe(false);
      expect(result.error?.message).toBe("authentication failed");
      expect(result.totalAttempts).toBe(1);
      expect(operation).toHaveBeenCalledTimes(1);
    });

    it("should respect delay between retries", async () => {
      const startTime = Date.now();
      const operation = jest
        .fn()
        .mockRejectedValueOnce(new Error("network error"))
        .mockRejectedValueOnce(new Error("timeout"))
        .mockResolvedValue("success");

      const result = await retryManager.executeWithRetry(operation, "test");

      const duration = Date.now() - startTime;
      expect(result.success).toBe(true);
      expect(result.totalAttempts).toBe(3);
      expect(duration).toBeGreaterThanOrEqual(10); // At least base delay
    });

    it("should apply exponential backoff", async () => {
      const operation = jest
        .fn()
        .mockRejectedValueOnce(new Error("error 1"))
        .mockRejectedValueOnce(new Error("error 2"))
        .mockResolvedValue("success");

      const result = await retryManager.executeWithRetry(operation, "test");

      expect(result.success).toBe(true);
      expect(result.attempts[0].delayMs).toBe(0); // First attempt has no delay
      expect(result.attempts[1].delayMs).toBeGreaterThan(0); // Second attempt has delay
    });
  });

  describe("circuit breaker", () => {
    it("should open circuit breaker after threshold failures", async () => {
      const operation = jest.fn().mockRejectedValue(new Error("service down"));

      // Trigger circuit breaker
      for (let i = 0; i < 3; i++) {
        await retryManager.executeWithRetry(operation, "test");
      }

      const circuitInfo = retryManager.getCircuitBreakerInfo();
      expect(circuitInfo.state).toBe(CircuitBreakerState.OPEN);
      expect(circuitInfo.failureCount).toBe(3);
    });

    it("should block requests when circuit breaker is open", async () => {
      const operation = jest.fn().mockRejectedValue(new Error("service down"));

      // Trigger circuit breaker
      for (let i = 0; i < 3; i++) {
        await retryManager.executeWithRetry(operation, "test");
      }

      // Next request should be blocked
      const result = await retryManager.executeWithRetry(operation, "test");

      expect(result.success).toBe(false);
      expect(result.circuitBreakerTriggered).toBe(true);
      expect(result.error?.message).toContain("Circuit breaker is open");
      expect(operation).not.toHaveBeenCalledTimes(4); // Should not call operation
    });

    it("should move to half-open state after timeout", async () => {
      const operation = jest.fn().mockRejectedValue(new Error("service down"));

      // Trigger circuit breaker
      for (let i = 0; i < 3; i++) {
        await retryManager.executeWithRetry(operation, "test");
      }

      // Check that circuit breaker is open
      let circuitInfo = retryManager.getCircuitBreakerInfo();
      expect(circuitInfo.state).toBe(CircuitBreakerState.OPEN);

      // Wait for timeout
      await new Promise((resolve) => setTimeout(resolve, 200));

      circuitInfo = retryManager.getCircuitBreakerInfo();
      expect(circuitInfo.state).toBe(CircuitBreakerState.HALF_OPEN);
    });

    it("should close circuit breaker on successful request", async () => {
      const failingOperation = jest
        .fn()
        .mockRejectedValue(new Error("service down"));
      const successOperation = jest.fn().mockResolvedValue("success");

      // Trigger circuit breaker
      for (let i = 0; i < 3; i++) {
        await retryManager.executeWithRetry(failingOperation, "test");
      }

      // Wait for timeout
      await new Promise((resolve) => setTimeout(resolve, 200));

      // Successful request should close circuit breaker
      await retryManager.executeWithRetry(successOperation, "test");

      const circuitInfo = retryManager.getCircuitBreakerInfo();
      expect(circuitInfo.state).toBe(CircuitBreakerState.CLOSED);
      expect(circuitInfo.failureCount).toBe(0);
    });

    it("should reset circuit breaker manually", async () => {
      const operation = jest.fn().mockRejectedValue(new Error("service down"));

      // Trigger circuit breaker
      for (let i = 0; i < 3; i++) {
        await retryManager.executeWithRetry(operation, "test");
      }

      // Reset circuit breaker
      retryManager.resetCircuitBreaker();

      const circuitInfo = retryManager.getCircuitBreakerInfo();
      expect(circuitInfo.state).toBe(CircuitBreakerState.CLOSED);
      expect(circuitInfo.failureCount).toBe(0);
    });
  });

  describe("configuration", () => {
    it("should update configuration", () => {
      retryManager.updateConfig({
        maxRetries: 5,
        baseDelayMs: 1000,
      });

      const config = retryManager.getConfig();
      expect(config.maxRetries).toBe(5);
      expect(config.baseDelayMs).toBe(1000);
    });

    it("should disable circuit breaker when configured", async () => {
      retryManager.updateConfig({
        enableCircuitBreaker: false,
      });

      const operation = jest.fn().mockRejectedValue(new Error("service down"));

      // Even with many failures, circuit breaker should not open
      for (let i = 0; i < 10; i++) {
        const result = await retryManager.executeWithRetry(operation, "test");
        expect(result.circuitBreakerTriggered).toBe(false);
      }
    });
  });

  describe("error classification", () => {
    it("should retry on retryable errors", async () => {
      const retryableErrors = [
        "ECONNRESET",
        "ETIMEDOUT",
        "ENOTFOUND",
        "ECONNREFUSED",
        "timeout",
        "network",
        "rate limit",
        "server error",
        "service unavailable",
      ];

      for (const errorMessage of retryableErrors) {
        const operation = jest
          .fn()
          .mockRejectedValueOnce(new Error(errorMessage))
          .mockResolvedValue("success");

        const result = await retryManager.executeWithRetry(operation, "test");

        expect(result.success).toBe(true);
        expect(result.totalAttempts).toBe(2);
        operation.mockClear();
      }
    });

    it("should not retry on non-retryable errors", async () => {
      const nonRetryableErrors = [
        "authentication",
        "authorization",
        "invalid request",
        "bad request",
        "not found",
        "forbidden",
      ];

      for (const errorMessage of nonRetryableErrors) {
        const operation = jest.fn().mockRejectedValue(new Error(errorMessage));

        const result = await retryManager.executeWithRetry(operation, "test");

        expect(result.success).toBe(false);
        expect(result.totalAttempts).toBe(1);
        expect(result.attempts).toHaveLength(1);
        expect(operation).toHaveBeenCalledTimes(1);
        operation.mockClear();
      }
    });
  });

  describe("performance tracking", () => {
    it("should track total time spent retrying", async () => {
      const operation = jest
        .fn()
        .mockRejectedValueOnce(new Error("network error"))
        .mockRejectedValueOnce(new Error("timeout"))
        .mockResolvedValue("success");

      const startTime = Date.now();
      const result = await retryManager.executeWithRetry(operation, "test");
      const endTime = Date.now();

      expect(result.totalTimeMs).toBeGreaterThan(0);
      expect(result.totalTimeMs).toBeLessThanOrEqual(endTime - startTime);
    });

    it("should track individual attempt details", async () => {
      const operation = jest
        .fn()
        .mockRejectedValueOnce(new Error("error 1"))
        .mockRejectedValueOnce(new Error("error 2"))
        .mockResolvedValue("success");

      const result = await retryManager.executeWithRetry(operation, "test");

      expect(result.attempts).toHaveLength(2); // Only failed attempts are recorded
      expect(result.attempts[0].attempt).toBe(1);
      expect(result.attempts[0].delayMs).toBe(0);
      expect(result.attempts[1].attempt).toBe(2);
      expect(result.attempts[1].delayMs).toBeGreaterThan(0);
    });
  });
});
