/**
 * Error Guard Utilities
 * 
 * Provides defensive programming utilities for safe operations
 * with automatic error handling and graceful degradation.
 * 
 * @author @darianrosebrook
 */

import { ErrorCode, AppError } from "../errors/types";

/**
 * Safely execute an async function with error handling
 */
export async function safeAsync<T>(
  fn: () => Promise<T>,
  fallback: T,
  errorContext?: string
): Promise<T> {
  try {
    return await fn();
  } catch (error) {
    const context = errorContext ? `[${errorContext}] ` : "";
    console.error(`${context}Safe async operation failed:`, error);
    return fallback;
  }
}

/**
 * Safely execute a synchronous function with error handling
 */
export function safeSync<T>(
  fn: () => T,
  fallback: T,
  errorContext?: string
): T {
  try {
    return fn();
  } catch (error) {
    const context = errorContext ? `[${errorContext}] ` : "";
    console.error(`${context}Safe sync operation failed:`, error);
    return fallback;
  }
}

/**
 * Safely access nested object properties
 */
export function safeGet<T>(
  obj: unknown,
  path: string,
  fallback: T
): T {
  try {
    const keys = path.split(".");
    let current: unknown = obj;
    for (const key of keys) {
      if (current == null || typeof current !== "object") {
        return fallback;
      }
      current = (current as Record<string, unknown>)[key];
      if (current === undefined) {
        return fallback;
      }
    }
    return current as T;
  } catch {
    return fallback;
  }
}

/**
 * Safely parse JSON with fallback
 */
export function safeParseJSON<T>(
  json: string,
  fallback: T,
  errorContext?: string
): T {
  try {
    return JSON.parse(json) as T;
  } catch (error) {
    const context = errorContext ? `[${errorContext}] ` : "";
    console.warn(`${context}Failed to parse JSON:`, error);
    return fallback;
  }
}

/**
 * Safely execute a function with retry logic
 */
export async function safeRetry<T>(
  fn: () => Promise<T>,
  options: {
    maxRetries?: number;
    delay?: number;
    backoff?: boolean;
    onRetry?: (attempt: number, error: Error) => void;
    shouldRetry?: (error: Error) => boolean;
  } = {}
): Promise<T> {
  const {
    maxRetries = 3,
    delay = 1000,
    backoff = true,
    onRetry,
    shouldRetry = () => true,
  } = options;

  let lastError: Error | null = null;

  for (let attempt = 0; attempt <= maxRetries; attempt++) {
    try {
      return await fn();
    } catch (error) {
      lastError = error instanceof Error ? error : new Error(String(error));

      if (!shouldRetry(lastError)) {
        throw lastError;
      }

      if (attempt < maxRetries) {
        const waitTime = backoff ? delay * Math.pow(2, attempt) : delay;
        onRetry?.(attempt + 1, lastError);
        await new Promise((resolve) => setTimeout(resolve, waitTime));
      }
    }
  }

  throw lastError || new Error("Retry failed");
}

/**
 * Circuit breaker pattern for preventing repeated failures
 */
export class CircuitBreaker {
  private failures = 0;
  private lastFailureTime: number | null = null;
  private state: "closed" | "open" | "half-open" = "closed";

  constructor(
    private options: {
      failureThreshold?: number;
      resetTimeout?: number;
      halfOpenMaxCalls?: number;
    } = {}
  ) {
    this.options = {
      failureThreshold: 5,
      resetTimeout: 60000, // 1 minute
      halfOpenMaxCalls: 1,
      ...options,
    };
  }

  async execute<T>(fn: () => Promise<T>): Promise<T> {
    if (this.state === "open") {
      if (
        this.lastFailureTime &&
        Date.now() - this.lastFailureTime > (this.options.resetTimeout ?? 60000)
      ) {
        this.state = "half-open";
        this.failures = 0;
      } else {
        throw new AppError(
          ErrorCode.CONNECTION_FAILED,
          "Circuit breaker is open. Service is temporarily unavailable."
        );
      }
    }

    try {
      const result = await fn();
      this.onSuccess();
      return result;
    } catch (error) {
      this.onFailure();
      throw error;
    }
  }

  private onSuccess() {
    this.failures = 0;
    if (this.state === "half-open") {
      this.state = "closed";
    }
  }

  private onFailure() {
    this.failures++;
    this.lastFailureTime = Date.now();

    if (this.failures >= (this.options.failureThreshold ?? 5)) {
      this.state = "open";
    }
  }

  reset() {
    this.failures = 0;
    this.lastFailureTime = null;
    this.state = "closed";
  }

  getState() {
    return this.state;
  }
}

/**
 * Timeout wrapper for async operations
 */
export async function withTimeout<T>(
  promise: Promise<T>,
  timeoutMs: number,
  timeoutError?: Error
): Promise<T> {
  const timeout = new Promise<never>((_, reject) => {
    setTimeout(() => {
      reject(
        timeoutError ||
        new AppError(ErrorCode.TIMEOUT, `Operation timed out after ${timeoutMs}ms`)
      );
    }, timeoutMs);
  });

  return Promise.race([promise, timeout]);
}

/**
 * Debounce function execution
 */
export function debounce<T extends (...args: unknown[]) => unknown>(
  fn: T,
  delay: number
): (...args: Parameters<T>) => void {
  let timeoutId: ReturnType<typeof setTimeout> | null = null;

  return (...args: Parameters<T>) => {
    if (timeoutId) {
      clearTimeout(timeoutId);
    }
    timeoutId = setTimeout(() => {
      fn(...args);
    }, delay);
  };
}

/**
 * Throttle function execution
 */
export function throttle<T extends (...args: unknown[]) => unknown>(
  fn: T,
  delay: number
): (...args: Parameters<T>) => void {
  let lastCall = 0;

  return (...args: Parameters<T>) => {
    const now = Date.now();
    if (now - lastCall >= delay) {
      lastCall = now;
      fn(...args);
    }
  };
}

