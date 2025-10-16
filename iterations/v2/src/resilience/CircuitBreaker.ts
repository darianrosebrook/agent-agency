/**
 * Circuit Breaker Pattern Implementation
 *
 * Prevents cascading failures by automatically detecting failures
 * and temporarily stopping requests to failing services.
 *
 * States:
 * - CLOSED: Normal operation
 * - OPEN: Failing, reject all requests
 * - HALF_OPEN: Testing if service has recovered
 *
 * @author @darianrosebrook
 */

// Re-export commonly used types
export { VerificationPriority } from "../types/verification";

/**
 * Error thrown when circuit breaker is open
 */
export class CircuitBreakerOpenError extends Error {
  circuitName?: string;
  stats?: CircuitBreakerStats;

  constructor(
    message: string = "Circuit breaker is open",
    circuitName?: string,
    stats?: CircuitBreakerStats
  ) {
    super(message);
    this.name = "CircuitBreakerOpenError";
    this.circuitName = circuitName;
    this.stats = stats;
  }
}

export enum CircuitState {
  CLOSED = "closed", // Normal operation
  OPEN = "open", // Failing, reject requests
  HALF_OPEN = "half-open", // Testing if recovered
}

export interface CircuitBreakerConfig {
  name?: string; // Optional circuit breaker name
  failureThreshold: number; // Failures before opening
  successThreshold: number; // Successes before closing from half-open
  timeout?: number; // Time to wait before half-open (ms) - deprecated, use resetTimeoutMs
  timeoutMs?: number; // Operation timeout (ms)
  failureWindowMs?: number; // Time window for failure counting (ms)
  resetTimeoutMs?: number; // Time to wait before half-open (ms)
}

export interface CircuitBreakerStats {
  state: CircuitState;
  failureCount: number;
  successCount: number;
  failures: number; // Alias for failureCount
  successes: number; // Alias for successCount
  totalRequests: number; // Total requests processed
  lastFailure: Date | null;
  lastSuccess: Date | null;
}

/**
 * Circuit breaker for resilience
 *
 * Automatically detects failures and stops calling failing operations.
 * Allows for automatic recovery testing after a timeout period.
 */
export class CircuitBreaker {
  private state: CircuitState = CircuitState.CLOSED;
  private failureCount = 0;
  private successCount = 0;
  private totalRequests = 0;
  private nextAttempt = Date.now();
  private lastFailure: Date | null = null;
  private lastSuccess: Date | null = null;

  constructor(private config: CircuitBreakerConfig) {}

  /**
   * Execute an operation with circuit breaker protection
   *
   * @param operation The operation to execute
   * @param fallback Optional fallback if circuit is open
   * @returns Result of operation or fallback
   */
  async execute<T>(
    operation: () => Promise<T>,
    fallback?: () => T | Promise<T>
  ): Promise<T> {
    this.totalRequests++; // Count total requests

    // Check if circuit is open
    if (this.state === CircuitState.OPEN) {
      if (Date.now() < this.nextAttempt) {
        // Still in timeout period
        if (fallback) {
          return await fallback();
        }
        throw new CircuitBreakerOpenError(
          `Circuit breaker is OPEN (next attempt in ${
            this.nextAttempt - Date.now()
          }ms)`,
          this.config.name,
          this.getStats()
        );
      }
      // Try transitioning to half-open
      this.state = CircuitState.HALF_OPEN;
      this.successCount = 0;
    }

    try {
      const result = await this.executeWithTimeout(operation);
      this.onSuccess();
      return result;
    } catch (error) {
      this.onFailure();
      if (fallback) {
        return await fallback();
      }
      throw error;
    }
  }

  /**
   * Execute operation with timeout
   */
  private async executeWithTimeout<T>(operation: () => Promise<T>): Promise<T> {
    let timeoutId: NodeJS.Timeout | undefined;

    const timeoutPromise = new Promise<T>((_, reject) => {
      timeoutId = setTimeout(
        () => reject(new Error("Operation timeout")),
        this.config.timeoutMs
      );
    });

    try {
      const result = await Promise.race([operation(), timeoutPromise]);

      // Clear the timeout since the operation completed successfully
      if (timeoutId) {
        clearTimeout(timeoutId);
      }
      return result;
    } catch (error) {
      // Clear the timeout in case of error too
      if (timeoutId) {
        clearTimeout(timeoutId);
      }
      throw error;
    }
  }

  /**
   * Handle successful operation
   */
  private onSuccess(): void {
    this.failureCount = 0;
    this.lastSuccess = new Date();

    if (this.state === CircuitState.HALF_OPEN) {
      this.successCount++;
      if (this.successCount >= this.config.successThreshold) {
        // Enough successes, close circuit
        this.state = CircuitState.CLOSED;
        this.successCount = 0;
      }
    }
  }

  /**
   * Handle failed operation
   */
  private onFailure(): void {
    this.failureCount++;
    this.lastFailure = new Date();

    if (
      this.state === CircuitState.HALF_OPEN ||
      this.failureCount >= this.config.failureThreshold
    ) {
      // Open circuit
      this.state = CircuitState.OPEN;
      this.nextAttempt =
        Date.now() +
        (this.config.timeout || this.config.resetTimeoutMs || 60000);
      this.successCount = 0;
    }
  }

  /**
   * Get current circuit state
   */
  getState(): CircuitState {
    return this.state;
  }

  /**
   * Get circuit breaker statistics
   */
  getStats(): CircuitBreakerStats {
    return {
      state: this.state,
      failureCount: this.failureCount,
      successCount: this.successCount,
      failures: this.failureCount, // Alias for failureCount
      successes: this.successCount, // Alias for successCount
      totalRequests: this.totalRequests,
      lastFailure: this.lastFailure,
      lastSuccess: this.lastSuccess,
    };
  }

  /**
   * Reset circuit breaker to closed state
   */
  reset(): void {
    this.state = CircuitState.CLOSED;
    this.failureCount = 0;
    this.successCount = 0;
    this.lastFailure = null;
    this.lastSuccess = null;
  }

  /**
   * Force circuit open (for testing or manual intervention)
   */
  forceOpen(timeoutMs?: number): void {
    this.state = CircuitState.OPEN;
    this.nextAttempt =
      Date.now() +
      (timeoutMs || this.config.timeout || this.config.resetTimeoutMs || 60000);
  }

  /**
   * Force circuit closed (for testing or manual intervention)
   */
  forceClosed(): void {
    this.state = CircuitState.CLOSED;
    this.failureCount = 0;
    this.successCount = 0;
  }
}
