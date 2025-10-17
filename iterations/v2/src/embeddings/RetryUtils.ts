/**
 * @fileoverview Retry utilities with exponential backoff
 *
 * Provides intelligent retry mechanisms for handling transient failures
 * in embedding API calls and other operations.
 *
 * @author @darianrosebrook
 */

export interface RetryConfig {
  /** Maximum number of retry attempts */
  maxAttempts: number;
  /** Initial delay in milliseconds */
  baseDelay: number;
  /** Maximum delay between retries */
  maxDelay: number;
  /** Exponential backoff multiplier */
  exponentialBase: number;
  /** Add random jitter to prevent thundering herd */
  jitter: boolean;
  /** Function to determine if error is retryable */
  retryCondition?: (error: Error) => boolean;
}

export interface RetryStats {
  attempts: number;
  totalDelay: number;
  lastError?: Error;
  succeeded: boolean;
}

/**
 * Default retry configuration for embedding API calls
 */
export const DEFAULT_RETRY_CONFIG: RetryConfig = {
  maxAttempts: 3,
  baseDelay: 1000, // 1 second
  maxDelay: 10000, // 10 seconds
  exponentialBase: 2, // Double the delay each time
  jitter: true, // Add randomness to prevent thundering herd
};

/**
 * Retry an async operation with exponential backoff
 */
export async function retryWithBackoff<T>(
  operation: () => Promise<T>,
  config: Partial<RetryConfig> = {}
): Promise<T> {
  const fullConfig = { ...DEFAULT_RETRY_CONFIG, ...config };
  const stats: RetryStats = {
    attempts: 0,
    totalDelay: 0,
    succeeded: false,
  };

  let lastError: Error;

  for (let attempt = 1; attempt <= fullConfig.maxAttempts; attempt++) {
    stats.attempts = attempt;

    try {
      const result = await operation();
      stats.succeeded = true;
      return result;
    } catch (error) {
      lastError = error as Error;
      stats.lastError = lastError;

      // Check if this error should trigger a retry
      const shouldRetry = fullConfig.retryCondition
        ? fullConfig.retryCondition(lastError)
        : isRetryableError(lastError);

      if (!shouldRetry || attempt === fullConfig.maxAttempts) {
        throw lastError;
      }

      // Calculate delay with exponential backoff
      const delay = calculateDelay(attempt, fullConfig);
      stats.totalDelay += delay;

      console.warn(
        `Retry attempt ${attempt}/${fullConfig.maxAttempts} failed: ${lastError.message}. ` +
          `Retrying in ${delay}ms...`
      );

      await sleep(delay);
    }
  }

  throw lastError!;
}

/**
 * Calculate delay for exponential backoff with optional jitter
 */
function calculateDelay(attempt: number, config: RetryConfig): number {
  const exponentialDelay =
    config.baseDelay * Math.pow(config.exponentialBase, attempt - 1);
  const delay = Math.min(exponentialDelay, config.maxDelay);

  if (config.jitter) {
    // Add random jitter (±25% of delay)
    const jitterRange = delay * 0.25;
    const jitter = (Math.random() - 0.5) * 2 * jitterRange;
    return Math.max(0, Math.floor(delay + jitter));
  }

  return Math.floor(delay);
}

/**
 * Sleep for specified milliseconds
 */
function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * Determine if an error is retryable
 */
export function isRetryableError(error: Error): boolean {
  // Network errors
  if (error.name === "TypeError" && error.message.includes("fetch")) {
    return true; // Network connectivity issues
  }

  // Timeout errors
  if (error.name === "AbortError" || error.message.includes("timeout")) {
    return true;
  }

  // HTTP 5xx server errors
  if (
    error.message.includes("500") ||
    error.message.includes("502") ||
    error.message.includes("503") ||
    error.message.includes("504")
  ) {
    return true;
  }

  // Specific API errors that are transient
  if (
    error.message.includes("rate limit") ||
    error.message.includes("temporary")
  ) {
    return true;
  }

  // Ollama specific errors
  if (
    error.message.includes("connection refused") ||
    error.message.includes("model not loaded") ||
    error.message.includes("server error")
  ) {
    return true;
  }

  return false;
}

/**
 * Retry configuration presets for different scenarios
 */
export class RetryPresets {
  /** Fast retries for user-facing operations */
  static readonly FAST: Partial<RetryConfig> = {
    maxAttempts: 2,
    baseDelay: 500,
    maxDelay: 2000,
  };

  /** Standard retries for most API operations */
  static readonly STANDARD: Partial<RetryConfig> = {
    maxAttempts: 3,
    baseDelay: 1000,
    maxDelay: 10000,
  };

  /** Aggressive retries for critical operations */
  static readonly AGGRESSIVE: Partial<RetryConfig> = {
    maxAttempts: 5,
    baseDelay: 1000,
    maxDelay: 30000,
  };

  /** Conservative retries for external APIs */
  static readonly CONSERVATIVE: Partial<RetryConfig> = {
    maxAttempts: 3,
    baseDelay: 2000,
    maxDelay: 20000,
  };

  /** Minimal retries for development/testing */
  static readonly MINIMAL: Partial<RetryConfig> = {
    maxAttempts: 1,
    baseDelay: 100,
    maxDelay: 1000,
  };
}

/**
 * Circuit breaker aware retry - respects circuit breaker state
 */
export async function retryWithCircuitBreaker<T>(
  operation: () => Promise<T>,
  circuitBreaker: { getState: () => string },
  config: Partial<RetryConfig> = {}
): Promise<T> {
  // If circuit breaker is open, don't retry
  if (circuitBreaker.getState() === "open") {
    throw new Error("Circuit breaker is open - operation rejected");
  }

  return retryWithBackoff(operation, config);
}
