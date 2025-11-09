/**
 * Retry utility with exponential backoff
 * 
 * Provides retry logic for API calls with configurable backoff strategies.
 * Adapted from open-webui patterns for agent-agency.
 * 
 * @author @darianrosebrook
 */

import { AppError, isRetryableError } from '../errors/types';

/**
 * Retry configuration options
 */
export interface RetryOptions {
  /**
   * Maximum number of retry attempts
   * @default 3
   */
  maxAttempts?: number;
  
  /**
   * Initial delay in milliseconds
   * @default 1000
   */
  initialDelay?: number;
  
  /**
   * Maximum delay in milliseconds
   * @default 10000
   */
  maxDelay?: number;
  
  /**
   * Randomization factor (0-1) for jitter
   * @default 0.5
   */
  randomizationFactor?: number;
  
  /**
   * Whether to retry on non-retryable errors
   * @default false
   */
  retryOnNonRetryable?: boolean;
  
  /**
   * Custom retry condition function
   */
  shouldRetry?: (error: unknown, attempt: number) => boolean;
  
  /**
   * Callback called before each retry attempt
   */
  onRetry?: (error: unknown, attempt: number, delay: number) => void;
}

/**
 * Default retry options matching open-webui patterns
 */
const DEFAULT_RETRY_OPTIONS: Required<RetryOptions> = {
  maxAttempts: 3,
  initialDelay: 1000,
  maxDelay: 10000,
  randomizationFactor: 0.5,
  retryOnNonRetryable: false,
  shouldRetry: (error: unknown) => isRetryableError(error),
  onRetry: () => {},
};

/**
 * Calculate delay with exponential backoff and jitter
 */
function calculateDelay(
  attempt: number,
  initialDelay: number,
  maxDelay: number,
  randomizationFactor: number
): number {
  // Exponential backoff: delay = initialDelay * 2^attempt
  const exponentialDelay = initialDelay * Math.pow(2, attempt);
  
  // Cap at maxDelay
  const cappedDelay = Math.min(exponentialDelay, maxDelay);
  
  // Add jitter (randomization) to prevent thundering herd
  const jitter = cappedDelay * randomizationFactor * (Math.random() - 0.5);
  
  return Math.max(0, cappedDelay + jitter);
}

/**
 * Sleep for specified milliseconds
 */
function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * Retry a function with exponential backoff
 * 
 * @example
 * ```ts
 * const result = await retry(
 *   () => fetch('/api/data'),
 *   { maxAttempts: 3, initialDelay: 1000 }
 * );
 * ```
 */
export async function retry<T>(
  fn: () => Promise<T>,
  options: RetryOptions = {}
): Promise<T> {
  const opts = { ...DEFAULT_RETRY_OPTIONS, ...options };
  let lastError: unknown;
  
  for (let attempt = 0; attempt <= opts.maxAttempts; attempt++) {
    try {
      return await fn();
    } catch (error) {
      lastError = error;
      
      // Don't retry on last attempt
      if (attempt >= opts.maxAttempts) {
        break;
      }
      
      // Check if we should retry
      const shouldRetry = opts.shouldRetry(error, attempt);
      if (!shouldRetry && !opts.retryOnNonRetryable) {
        throw error;
      }
      
      // Calculate delay
      const delay = calculateDelay(
        attempt,
        opts.initialDelay,
        opts.maxDelay,
        opts.randomizationFactor
      );
      
      // Call onRetry callback
      opts.onRetry(error, attempt + 1, delay);
      
      // Wait before retrying
      await sleep(delay);
    }
  }
  
  // If we get here, all retries failed
  throw lastError;
}

/**
 * Retry with specific error handling
 * 
 * Automatically retries retryable errors (network errors, timeouts, etc.)
 * and throws non-retryable errors immediately.
 */
export async function retryWithBackoff<T>(
  fn: () => Promise<T>,
  options?: RetryOptions
): Promise<T> {
  return retry(fn, {
    ...options,
    shouldRetry: (error) => {
      // Use custom shouldRetry if provided
      if (options?.shouldRetry) {
        return options.shouldRetry(error, 0);
      }
      // Default: only retry retryable errors
      return isRetryableError(error);
    },
  });
}

