/**
 * @fileoverview Retry Policy with Exponential Backoff
 *
 * Provides configurable retry logic with exponential backoff and jitter
 * to prevent thundering herd problems.
 *
 * @author @darianrosebrook
 */

export interface RetryConfig {
  /** Maximum number of retry attempts */
  maxAttempts: number;

  /** Initial delay in milliseconds */
  initialDelayMs: number;

  /** Maximum delay in milliseconds */
  maxDelayMs: number;

  /** Backoff multiplier (e.g., 2 for exponential) */
  backoffMultiplier: number;

  /** Add random jitter to prevent thundering herd */
  jitter: boolean;

  /** Function to determine if error is retryable */
  isRetryable?: (error: any) => boolean;

  /** Name for logging */
  name?: string;
}

export interface RetryStats {
  attempt: number;
  totalAttempts: number;
  lastError?: Error;
  totalDelayMs: number;
}

/**
 * Retry Policy with Exponential Backoff
 */
export class RetryPolicy {
  private static readonly DEFAULT_CONFIG: Partial<RetryConfig> = {
    maxAttempts: 3,
    initialDelayMs: 100,
    maxDelayMs: 5000,
    backoffMultiplier: 2,
    jitter: true,
    isRetryable: () => true,
  };

  private config: RetryConfig;

  constructor(config: Partial<RetryConfig> = {}) {
    this.config = {
      ...RetryPolicy.DEFAULT_CONFIG,
      ...config,
    } as RetryConfig;
  }

  /**
   * Execute a function with retry logic
   */
  async execute<T>(fn: () => Promise<T>): Promise<T> {
    let lastError: Error | undefined;
    let totalDelayMs = 0;

    for (let attempt = 1; attempt <= this.config.maxAttempts; attempt++) {
      try {
        return await fn();
      } catch (error) {
        lastError = error instanceof Error ? error : new Error(String(error));

        // Check if error is retryable
        if (this.config.isRetryable && !this.config.isRetryable(error)) {
          throw lastError;
        }

        // Don't delay after last attempt
        if (attempt < this.config.maxAttempts) {
          const delayMs = this.calculateDelay(attempt);
          totalDelayMs += delayMs;

          this.log(
            `Attempt ${attempt}/${this.config.maxAttempts} failed: ${lastError.message}. Retrying in ${delayMs}ms...`
          );

          await this.sleep(delayMs);
        }
      }
    }

    // All attempts failed
    throw new RetryExhaustedError(
      `Failed after ${this.config.maxAttempts} attempts`,
      {
        attempt: this.config.maxAttempts,
        totalAttempts: this.config.maxAttempts,
        lastError,
        totalDelayMs,
      }
    );
  }

  /**
   * Calculate delay for given attempt with exponential backoff
   */
  private calculateDelay(attempt: number): number {
    // Calculate exponential backoff
    const exponentialDelay =
      this.config.initialDelayMs *
      Math.pow(this.config.backoffMultiplier, attempt - 1);

    // Cap at max delay
    let delay = Math.min(exponentialDelay, this.config.maxDelayMs);

    // Add jitter if enabled
    if (this.config.jitter) {
      delay = delay * (0.5 + Math.random() * 0.5);
    }

    return Math.floor(delay);
  }

  /**
   * Sleep for specified milliseconds
   */
  private sleep(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  /**
   * Log message
   */
  private log(message: string): void {
    const prefix = this.config.name ? `[Retry:${this.config.name}]` : "[Retry]";
    console.log(`${prefix} ${message}`);
  }
}

/**
 * Error thrown when all retry attempts are exhausted
 */
export class RetryExhaustedError extends Error {
  constructor(message: string, public stats: RetryStats) {
    super(message);
    this.name = "RetryExhaustedError";
  }
}

/**
 * Common retry policies for different scenarios
 */
export class RetryPolicies {
  /**
   * Aggressive retry for critical operations
   */
  static aggressive(): RetryPolicy {
    return new RetryPolicy({
      maxAttempts: 5,
      initialDelayMs: 50,
      maxDelayMs: 2000,
      backoffMultiplier: 2,
      jitter: true,
    });
  }

  /**
   * Standard retry for normal operations
   */
  static standard(): RetryPolicy {
    return new RetryPolicy({
      maxAttempts: 3,
      initialDelayMs: 100,
      maxDelayMs: 5000,
      backoffMultiplier: 2,
      jitter: true,
    });
  }

  /**
   * Conservative retry for non-critical operations
   */
  static conservative(): RetryPolicy {
    return new RetryPolicy({
      maxAttempts: 2,
      initialDelayMs: 500,
      maxDelayMs: 10000,
      backoffMultiplier: 3,
      jitter: true,
    });
  }

  /**
   * Database-specific retry (handles connection errors)
   */
  static database(): RetryPolicy {
    return new RetryPolicy({
      maxAttempts: 3,
      initialDelayMs: 200,
      maxDelayMs: 5000,
      backoffMultiplier: 2,
      jitter: true,
      isRetryable: (error: any) => {
        // Retry on connection errors, timeouts, etc.
        const retryableErrors = [
          "ECONNREFUSED",
          "ETIMEDOUT",
          "ENOTFOUND",
          "connection",
          "timeout",
        ];

        const errorMessage = String(error).toLowerCase();
        return retryableErrors.some((keyword) =>
          errorMessage.includes(keyword)
        );
      },
      name: "database",
    });
  }
}
