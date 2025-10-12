/**
 * Task Retry Handler
 *
 * Handles retry logic with exponential backoff for failed task executions.
 *
 * @author @darianrosebrook
 */

import { EventEmitter } from "events";

export interface RetryConfig {
  maxRetries: number;
  initialBackoffMs: number;
  maxBackoffMs: number;
  backoffMultiplier: number;
  jitter: boolean;
}

export interface RetryAttempt {
  taskId: string;
  attempt: number;
  delay: number;
  error: any;
  timestamp: Date;
}

export class TaskExecutionError extends Error {
  constructor(
    message: string,
    public readonly originalError: any,
    public readonly taskId: string,
    public readonly attempts: number
  ) {
    super(message);
    this.name = "TaskExecutionError";
  }
}

export class TaskRetryHandler extends EventEmitter {
  private attempts: Map<string, RetryAttempt[]> = new Map();

  constructor(
    private config: RetryConfig = {
      maxRetries: 3,
      initialBackoffMs: 1000,
      maxBackoffMs: 30000,
      backoffMultiplier: 2,
      jitter: true,
    }
  ) {
    super();
  }

  /**
   * Execute operation with retry logic
   */
  async executeWithRetry<T>(
    operation: () => Promise<T>,
    taskId: string,
    context?: Record<string, any>
  ): Promise<T> {
    let attempts = 0;
    let lastError: any;

    while (attempts < this.config.maxRetries) {
      try {
        const result = await operation();

        // Success - clear any previous attempts
        this.attempts.delete(taskId);

        return result;
      } catch (error) {
        lastError = error;
        attempts++;

        // Record the failed attempt
        this.recordAttempt(taskId, attempts, error, context);

        // Check if we should retry
        if (attempts >= this.config.maxRetries) {
          break;
        }

        // Calculate delay and wait
        const delay = this.calculateBackoff(attempts);
        this.emit("task:retry", {
          taskId,
          attempt: attempts,
          delay,
          error,
          timestamp: new Date(),
        });

        await this.delay(delay);
      }
    }

    // All retries exhausted
    throw new TaskExecutionError(
      `Task ${taskId} failed after ${this.config.maxRetries} attempts`,
      lastError,
      taskId,
      attempts
    );
  }

  /**
   * Execute operation without retry (for operations that shouldn't be retried)
   */
  async executeOnce<T>(
    operation: () => Promise<T>,
    taskId: string
  ): Promise<T> {
    try {
      return await operation();
    } catch (error) {
      this.recordAttempt(taskId, 1, error);
      throw error;
    }
  }

  /**
   * Calculate exponential backoff delay
   */
  private calculateBackoff(attempt: number): number {
    let delay =
      this.config.initialBackoffMs *
      Math.pow(this.config.backoffMultiplier, attempt - 1);

    // Cap at max backoff
    delay = Math.min(delay, this.config.maxBackoffMs);

    // Add jitter if enabled
    if (this.config.jitter) {
      delay = delay * (0.5 + Math.random() * 0.5); // ±50% jitter
    }

    return Math.floor(delay);
  }

  /**
   * Record a retry attempt
   */
  private recordAttempt(
    taskId: string,
    attempt: number,
    error: any,
    context?: Record<string, any>
  ): void {
    const retryAttempt: RetryAttempt = {
      taskId,
      attempt,
      delay: 0, // Will be set when retrying
      error,
      timestamp: new Date(),
    };

    const attempts = this.attempts.get(taskId) || [];
    attempts.push(retryAttempt);
    this.attempts.set(taskId, attempts);
  }

  /**
   * Get retry attempts for a task
   */
  getAttempts(taskId: string): RetryAttempt[] {
    return this.attempts.get(taskId) || [];
  }

  /**
   * Check if task has exceeded retry limit
   */
  hasExceededRetries(taskId: string): boolean {
    const attempts = this.attempts.get(taskId) || [];
    return attempts.length >= this.config.maxRetries;
  }

  /**
   * Clear retry history for a task
   */
  clearAttempts(taskId: string): void {
    this.attempts.delete(taskId);
  }

  /**
   * Get retry statistics
   */
  getStats(): {
    activeRetries: number;
    totalAttempts: number;
    averageRetries: number;
  } {
    const totalAttempts = Array.from(this.attempts.values()).reduce(
      (sum, attempts) => sum + attempts.length,
      0
    );

    const averageRetries =
      this.attempts.size > 0 ? totalAttempts / this.attempts.size : 0;

    return {
      activeRetries: this.attempts.size,
      totalAttempts,
      averageRetries,
    };
  }

  /**
   * Delay helper
   */
  private delay(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  /**
   * Update retry configuration
   */
  updateConfig(newConfig: Partial<RetryConfig>): void {
    this.config = { ...this.config, ...newConfig };
  }

  /**
   * Clear all retry data
   */
  clear(): void {
    this.attempts.clear();
  }
}
