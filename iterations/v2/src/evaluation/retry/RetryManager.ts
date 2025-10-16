/**
 * Retry Manager for LLM Providers
 *
 * @author @darianrosebrook
 *
 * Provides robust retry mechanisms with exponential backoff, circuit breaker patterns,
 * and comprehensive error tracking for LLM provider operations.
 *
 * Features:
 * - Exponential backoff with jitter
 * - Circuit breaker pattern for failing services
 * - Retry policies based on error types
 * - Performance tracking and metrics
 * - Graceful degradation
 */

/**
 * Retry configuration
 */
export interface RetryConfig {
  /** Maximum number of retry attempts */
  maxRetries: number;

  /** Base delay between retries in milliseconds */
  baseDelayMs: number;

  /** Maximum delay between retries in milliseconds */
  maxDelayMs: number;

  /** Exponential backoff multiplier */
  backoffMultiplier: number;

  /** Jitter factor (0-1) to randomize delays */
  jitterFactor: number;

  /** Circuit breaker failure threshold */
  circuitBreakerThreshold: number;

  /** Circuit breaker timeout in milliseconds */
  circuitBreakerTimeoutMs: number;

  /** Enable circuit breaker */
  enableCircuitBreaker: boolean;

  /** Retry on specific error types */
  retryableErrors: string[];

  /** Don't retry on specific error types */
  nonRetryableErrors: string[];
}

/**
 * Retry attempt information
 */
export interface RetryAttempt {
  /** Attempt number (1-based) */
  attempt: number;

  /** Delay before this attempt in milliseconds */
  delayMs: number;

  /** Error from previous attempt (if any) */
  error?: Error;

  /** Timestamp of attempt */
  timestamp: Date;
}

/**
 * Retry result
 */
export interface RetryResult<T> {
  /** Success result */
  result?: T;

  /** Final error if all retries failed */
  error?: Error;

  /** Whether the operation succeeded */
  success: boolean;

  /** Total attempts made */
  totalAttempts: number;

  /** Total time spent retrying */
  totalTimeMs: number;

  /** Individual attempt details */
  attempts: RetryAttempt[];

  /** Whether circuit breaker was triggered */
  circuitBreakerTriggered: boolean;
}

/**
 * Circuit breaker state
 */
export enum CircuitBreakerState {
  CLOSED = "closed", // Normal operation
  OPEN = "open", // Failing, blocking requests
  HALF_OPEN = "half_open", // Testing if service recovered
}

/**
 * Circuit breaker information
 */
export interface CircuitBreakerInfo {
  /** Current state */
  state: CircuitBreakerState;

  /** Number of consecutive failures */
  failureCount: number;

  /** Last failure timestamp */
  lastFailureTime?: Date;

  /** Next attempt time (when circuit breaker will allow requests) */
  nextAttemptTime?: Date;
}

/**
 * Retry Manager
 */
export class RetryManager {
  private config: RetryConfig;
  private circuitBreakerState: CircuitBreakerState = CircuitBreakerState.CLOSED;
  private failureCount: number = 0;
  private lastFailureTime?: Date;
  private nextAttemptTime?: Date;

  constructor(config?: Partial<RetryConfig>) {
    this.config = {
      maxRetries: 3,
      baseDelayMs: 1000,
      maxDelayMs: 30000,
      backoffMultiplier: 2,
      jitterFactor: 0.1,
      circuitBreakerThreshold: 5,
      circuitBreakerTimeoutMs: 60000,
      enableCircuitBreaker: true,
      retryableErrors: [
        "ECONNRESET",
        "ETIMEDOUT",
        "ENOTFOUND",
        "ECONNREFUSED",
        "timeout",
        "network",
        "rate limit",
        "server error",
        "service unavailable",
      ],
      nonRetryableErrors: [
        "authentication",
        "authorization",
        "invalid request",
        "bad request",
        "not found",
        "forbidden",
      ],
      ...config,
    };
  }

  /**
   * Execute operation with retry logic
   */
  async executeWithRetry<T>(
    operation: () => Promise<T>,
    operationName: string = "operation"
  ): Promise<RetryResult<T>> {
    const startTime = Date.now();
    const attempts: RetryAttempt[] = [];
    let lastError: Error | undefined;

    // Check circuit breaker
    if (this.isCircuitBreakerOpen()) {
      return {
        success: false,
        totalAttempts: 0,
        totalTimeMs: Date.now() - startTime,
        attempts,
        circuitBreakerTriggered: true,
        error: new Error(
          `Circuit breaker is open for ${operationName}. Service appears to be down.`
        ),
      };
    }

    for (let attempt = 1; attempt <= this.config.maxRetries + 1; attempt++) {
      const attemptStartTime = Date.now();

      try {
        // Execute the operation
        const result = await operation();

        // Success - reset circuit breaker
        this.onSuccess();

        return {
          result,
          success: true,
          totalAttempts: attempt,
          totalTimeMs: Date.now() - startTime,
          attempts,
          circuitBreakerTriggered: false,
        };
      } catch (error) {
        lastError = error instanceof Error ? error : new Error(String(error));

        // Record attempt
        attempts.push({
          attempt,
          delayMs: attempt === 1 ? 0 : this.calculateDelay(attempt - 1),
          error: lastError,
          timestamp: new Date(),
        });

        // Check if error is retryable
        if (!this.isRetryableError(lastError)) {
          this.onFailure();
          return {
            success: false,
            totalAttempts: attempt,
            totalTimeMs: Date.now() - startTime,
            attempts,
            circuitBreakerTriggered: false,
            error: lastError,
          };
        }

        // If this was the last attempt, fail
        if (attempt > this.config.maxRetries) {
          this.onFailure();
          return {
            success: false,
            totalAttempts: attempt,
            totalTimeMs: Date.now() - startTime,
            attempts,
            circuitBreakerTriggered: false,
            error: lastError,
          };
        }

        // Wait before next attempt (only if not the last attempt)
        if (attempt <= this.config.maxRetries) {
          const delay = this.calculateDelay(attempt);
          if (delay > 0) {
            await this.sleep(delay);
          }
        }
      }
    }

    // This should never be reached, but just in case
    this.onFailure();
    return {
      success: false,
      totalAttempts: attempts.length,
      totalTimeMs: Date.now() - startTime,
      attempts,
      circuitBreakerTriggered: false,
      error: lastError,
    };
  }

  /**
   * Check if circuit breaker is open
   */
  private isCircuitBreakerOpen(): boolean {
    if (!this.config.enableCircuitBreaker) {
      return false;
    }

    if (this.circuitBreakerState === CircuitBreakerState.CLOSED) {
      return false;
    }

    if (this.circuitBreakerState === CircuitBreakerState.OPEN) {
      if (
        this.nextAttemptTime &&
        Date.now() >= this.nextAttemptTime.getTime()
      ) {
        // Move to half-open state
        this.circuitBreakerState = CircuitBreakerState.HALF_OPEN;
        this.nextAttemptTime = undefined;
        return false;
      }
      return true;
    }

    // HALF_OPEN state - allow one attempt
    return false;
  }

  /**
   * Calculate delay for retry attempt
   */
  private calculateDelay(attempt: number): number {
    const exponentialDelay =
      this.config.baseDelayMs *
      Math.pow(this.config.backoffMultiplier, attempt - 1);

    // Apply jitter to prevent thundering herd
    const jitter = exponentialDelay * this.config.jitterFactor * Math.random();
    const delay = exponentialDelay + jitter;

    return Math.min(delay, this.config.maxDelayMs);
  }

  /**
   * Check if error is retryable
   */
  private isRetryableError(error: Error): boolean {
    const errorMessage = error.message.toLowerCase();

    // Check non-retryable errors first
    for (const nonRetryable of this.config.nonRetryableErrors) {
      if (errorMessage.includes(nonRetryable.toLowerCase())) {
        return false;
      }
    }

    // Check retryable errors
    for (const retryable of this.config.retryableErrors) {
      if (errorMessage.includes(retryable.toLowerCase())) {
        return true;
      }
    }

    // Default to retryable for unknown errors
    return true;
  }

  /**
   * Handle successful operation
   */
  private onSuccess(): void {
    if (this.config.enableCircuitBreaker) {
      this.circuitBreakerState = CircuitBreakerState.CLOSED;
      this.failureCount = 0;
      this.lastFailureTime = undefined;
      this.nextAttemptTime = undefined;
    }
  }

  /**
   * Handle failed operation
   */
  private onFailure(): void {
    if (!this.config.enableCircuitBreaker) {
      return;
    }

    this.failureCount++;
    this.lastFailureTime = new Date();

    if (this.circuitBreakerState === CircuitBreakerState.HALF_OPEN) {
      // If we're in half-open state and fail, go back to open
      this.circuitBreakerState = CircuitBreakerState.OPEN;
      this.nextAttemptTime = new Date(
        Date.now() + this.config.circuitBreakerTimeoutMs
      );
    } else if (this.failureCount >= this.config.circuitBreakerThreshold) {
      this.circuitBreakerState = CircuitBreakerState.OPEN;
      this.nextAttemptTime = new Date(
        Date.now() + this.config.circuitBreakerTimeoutMs
      );
    }
  }

  /**
   * Sleep for specified milliseconds
   */
  private sleep(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  /**
   * Get circuit breaker information
   */
  getCircuitBreakerInfo(): CircuitBreakerInfo {
    return {
      state: this.circuitBreakerState,
      failureCount: this.failureCount,
      lastFailureTime: this.lastFailureTime,
      nextAttemptTime: this.nextAttemptTime,
    };
  }

  /**
   * Reset circuit breaker
   */
  resetCircuitBreaker(): void {
    this.circuitBreakerState = CircuitBreakerState.CLOSED;
    this.failureCount = 0;
    this.lastFailureTime = undefined;
    this.nextAttemptTime = undefined;
  }

  /**
   * Update retry configuration
   */
  updateConfig(config: Partial<RetryConfig>): void {
    this.config = { ...this.config, ...config };
  }

  /**
   * Get current configuration
   */
  getConfig(): RetryConfig {
    return { ...this.config };
  }
}
