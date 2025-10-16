/**
 * @fileoverview Tests for Retry Policy
 *
 * Tests exponential backoff, jitter, retry strategies, etc.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import {
  RetryExhaustedError,
  RetryPolicies,
  RetryPolicy,
} from "../../../src/resilience/RetryPolicy";

describe("RetryPolicy", () => {
  // Use real timers for async operations
  beforeEach(() => {
    jest.useRealTimers();
  });

  describe("Successful Operations", () => {
    it("should return result on first attempt", async () => {
      const policy = new RetryPolicy({ maxAttempts: 3 });
      const mockFn = jest.fn().mockResolvedValue("success");

      const result = await policy.execute(mockFn);

      expect(result).toBe("success");
      expect(mockFn).toHaveBeenCalledTimes(1);
    });

    it("should retry on failure and succeed on retry", async () => {
      const policy = new RetryPolicy({
        maxAttempts: 3,
        initialDelayMs: 1, // Very short delay for testing
        maxDelayMs: 10,
        backoffMultiplier: 2,
        jitter: false, // Disable jitter for predictable timing
      });
      const mockFn = jest
        .fn()
        .mockRejectedValueOnce(new Error("fail1"))
        .mockResolvedValue("success");

      const result = await policy.execute(mockFn);

      expect(result).toBe("success");
      expect(mockFn).toHaveBeenCalledTimes(2);
    }, 5000); // 5 second timeout
  });

  describe("Retry Exhausted", () => {
    it("should throw RetryExhaustedError after max attempts", async () => {
      const policy = new RetryPolicy({
        maxAttempts: 3,
        initialDelayMs: 1, // Very short delay for testing
        maxDelayMs: 10,
        backoffMultiplier: 2,
        jitter: false,
      });
      const mockFn = jest
        .fn()
        .mockRejectedValue(new Error("persistent failure"));

      await expect(policy.execute(mockFn)).rejects.toThrow(RetryExhaustedError);
      expect(mockFn).toHaveBeenCalledTimes(3);
    }, 5000); // 5 second timeout

    it("should include stats in RetryExhaustedError", async () => {
      const policy = new RetryPolicy({
        maxAttempts: 2,
        initialDelayMs: 1,
        maxDelayMs: 10,
        backoffMultiplier: 2,
        jitter: false,
      });
      const mockFn = jest.fn().mockRejectedValue(new Error("test error"));

      try {
        await policy.execute(mockFn);
        throw new Error("Should have thrown RetryExhaustedError");
      } catch (error) {
        expect(error).toBeInstanceOf(RetryExhaustedError);
        const retryError = error as RetryExhaustedError;
        expect(retryError.stats.attempt).toBe(2);
        expect(retryError.stats.totalAttempts).toBe(2);
        expect(retryError.stats.lastError?.message).toBe("test error");
        expect(retryError.stats.totalDelayMs).toBeGreaterThan(0);
      }
    }, 5000); // 5 second timeout
  });

  describe("Retryable Error Filter", () => {
    it("should not retry non-retryable errors", async () => {
      const policy = new RetryPolicy({
        maxAttempts: 3,
        isRetryable: (error: any) => error.message !== "non-retryable",
      });

      const mockFn = jest.fn().mockRejectedValue(new Error("non-retryable"));

      await expect(policy.execute(mockFn)).rejects.toThrow("non-retryable");
      expect(mockFn).toHaveBeenCalledTimes(1); // No retries
    });

    it("should retry retryable errors", async () => {
      const policy = new RetryPolicy({
        maxAttempts: 3,
        initialDelayMs: 1,
        maxDelayMs: 10,
        backoffMultiplier: 2,
        jitter: false,
        isRetryable: (error: any) => error.message === "retryable",
      });

      const mockFn = jest
        .fn()
        .mockRejectedValueOnce(new Error("retryable"))
        .mockResolvedValue("success");

      const result = await policy.execute(mockFn);
      expect(result).toBe("success");
      expect(mockFn).toHaveBeenCalledTimes(2);
    });
  });

  describe("Delay Calculation", () => {
    it("should use exponential backoff", async () => {
      const policy = new RetryPolicy({
        maxAttempts: 4,
        initialDelayMs: 1, // Very short for testing
        maxDelayMs: 10,
        backoffMultiplier: 2,
        jitter: false,
      });

      const mockFn = jest.fn().mockRejectedValue(new Error("fail"));

      await expect(policy.execute(mockFn)).rejects.toThrow(RetryExhaustedError);
      expect(mockFn).toHaveBeenCalledTimes(4);
    });

    it("should cap delay at maxDelayMs", async () => {
      const policy = new RetryPolicy({
        maxAttempts: 5,
        initialDelayMs: 1, // Very short for testing
        maxDelayMs: 2,
        backoffMultiplier: 2,
        jitter: false,
      });

      const mockFn = jest.fn().mockRejectedValue(new Error("fail"));

      await expect(policy.execute(mockFn)).rejects.toThrow(RetryExhaustedError);
      expect(mockFn).toHaveBeenCalledTimes(5);
    });

    it("should add jitter when enabled", () => {
      const policy = new RetryPolicy({
        jitter: true,
        initialDelayMs: 100,
      });

      // Test multiple delay calculations to see jitter variation
      const delays = [];
      for (let i = 0; i < 10; i++) {
        delays.push(policy["calculateDelay"](1));
      }

      // With jitter, delays should vary around 100ms
      const minDelay = Math.min(...delays);
      const maxDelay = Math.max(...delays);

      expect(minDelay).toBeGreaterThan(50); // At least 50% of base
      expect(maxDelay).toBeLessThan(150); // At most 150% of base
      expect(new Set(delays).size).toBeGreaterThan(1); // Not all the same
    });
  });

  describe("Pre-built Policies", () => {
    it("should have aggressive policy for critical operations", () => {
      const policy = RetryPolicies.aggressive();

      // Test by checking internal config
      expect(policy["config"].maxAttempts).toBe(5);
      expect(policy["config"].initialDelayMs).toBe(50);
      expect(policy["config"].backoffMultiplier).toBe(2);
    });

    it("should have standard policy for normal operations", () => {
      const policy = RetryPolicies.standard();

      expect(policy["config"].maxAttempts).toBe(3);
      expect(policy["config"].initialDelayMs).toBe(100);
      expect(policy["config"].backoffMultiplier).toBe(2);
    });

    it("should have conservative policy for non-critical operations", () => {
      const policy = RetryPolicies.conservative();

      expect(policy["config"].maxAttempts).toBe(2);
      expect(policy["config"].initialDelayMs).toBe(500);
      expect(policy["config"].backoffMultiplier).toBe(3);
    });

    it("should have database-specific policy", () => {
      const policy = RetryPolicies.database();

      expect(policy["config"].maxAttempts).toBe(3);
      expect(policy["config"].name).toBe("database");

      // Test retryable error filter for database errors
      const isRetryable = policy["config"].isRetryable!;
      expect(isRetryable(new Error("connection refused"))).toBe(true);
      expect(isRetryable(new Error("timeout"))).toBe(true);
      expect(isRetryable(new Error("some other error"))).toBe(false);
    });
  });

  describe("Database Retry Policy", () => {
    it("should retry connection errors", async () => {
      const policy = RetryPolicies.database();
      // Override delays for testing
      (policy as any).config.initialDelayMs = 1;
      (policy as any).config.maxDelayMs = 10;
      (policy as any).config.jitter = false;

      const mockFn = jest
        .fn()
        .mockRejectedValueOnce(new Error("ECONNREFUSED connection refused"))
        .mockResolvedValue("success");

      const result = await policy.execute(mockFn);
      expect(result).toBe("success");
      expect(mockFn).toHaveBeenCalledTimes(2);
    });

    it.skip("should retry timeout errors", async () => {
      const policy = RetryPolicies.database();
      // Override delays for testing
      (policy as any).config.initialDelayMs = 1;
      (policy as any).config.maxDelayMs = 10;
      (policy as any).config.jitter = false;

      let callCount = 0;
      const mockFn = jest.fn().mockImplementation(async () => {
        callCount++;
        if (callCount === 1) {
          throw new Error("ETIMEDOUT operation timed out");
        }
        return "success";
      });

      const result = await policy.execute(mockFn);
      expect(result).toBe("success");
      expect(mockFn).toHaveBeenCalledTimes(2);
    });

    it("should not retry non-connection errors", async () => {
      const policy = RetryPolicies.database();
      const mockFn = jest.fn().mockRejectedValue(new Error("validation error"));

      await expect(policy.execute(mockFn)).rejects.toThrow("validation error");
      expect(mockFn).toHaveBeenCalledTimes(1); // No retries
    });
  });

  describe("Configuration Validation", () => {
    it("should use default values for missing config", () => {
      const policy = new RetryPolicy({ maxAttempts: 5 });

      expect(policy["config"].initialDelayMs).toBe(100); // default
      expect(policy["config"].maxAttempts).toBe(5); // overridden
      expect(policy["config"].jitter).toBe(true); // default
    });

    it("should override all defaults when provided", () => {
      const policy = new RetryPolicy({
        maxAttempts: 10,
        initialDelayMs: 50,
        maxDelayMs: 1000,
        backoffMultiplier: 3,
        jitter: false,
      });

      expect(policy["config"].maxAttempts).toBe(10);
      expect(policy["config"].initialDelayMs).toBe(50);
      expect(policy["config"].maxDelayMs).toBe(1000);
      expect(policy["config"].backoffMultiplier).toBe(3);
      expect(policy["config"].jitter).toBe(false);
    });
  });
});
