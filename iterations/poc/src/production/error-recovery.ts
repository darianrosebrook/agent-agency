/**
 * Error Recovery System - Intelligent error handling and recovery
 *
 * @author @darianrosebrook
 * @description Enterprise-grade error recovery with circuit breakers, retries, and graceful degradation
 */

import { Logger } from "../utils/Logger.js";

export interface RecoveryConfig {
  enabled: boolean;
  maxRetries: number;
  retryDelay: number;
  circuitBreakerEnabled: boolean;
  circuitBreakerThreshold: number; // Failures per minute
  circuitBreakerTimeout: number; // Recovery timeout in ms
  gracefulDegradationEnabled: boolean;
  alertOnFailures: boolean;
}

export interface ErrorContext {
  operation: string;
  component: string;
  tenantId?: string;
  userId?: string;
  timestamp: number;
  attempt: number;
  error: Error;
  metadata?: Record<string, any>;
}

export interface RecoveryAction {
  type: "retry" | "circuit_break" | "degrade" | "fail";
  delay?: number;
  alternative?: string;
  message: string;
}

export interface CircuitBreakerState {
  failures: number;
  lastFailureTime: number;
  state: "closed" | "open" | "half_open";
  nextAttemptTime: number;
}

export class ErrorRecoveryManager {
  private config: RecoveryConfig;
  private logger: Logger;
  private circuitBreakers: Map<string, CircuitBreakerState> = new Map();
  private errorHistory: ErrorContext[] = [];
  private recoveryCallbacks: ((
    error: ErrorContext,
    action: RecoveryAction
  ) => void)[] = [];

  constructor(config: RecoveryConfig, logger?: Logger) {
    this.config = config;
    this.logger = logger || new Logger("ErrorRecoveryManager");

    if (config.enabled) {
      this.startMaintenanceCycle();
      this.logger.info("Error recovery manager initialized", {
        retries: config.maxRetries,
        circuitBreaker: config.circuitBreakerEnabled,
        gracefulDegradation: config.gracefulDegradationEnabled,
      });
    }
  }

  async executeWithRecovery<T>(
    operation: () => Promise<T>,
    context: Omit<ErrorContext, "timestamp" | "attempt" | "error">
  ): Promise<T> {
    if (!this.config.enabled) {
      return operation();
    }

    const circuitKey = `${context.component}:${context.operation}`;
    const circuitState = this.getCircuitBreakerState(circuitKey);

    // Check if circuit breaker is open
    if (circuitState.state === "open") {
      if (Date.now() < circuitState.nextAttemptTime) {
        return this.handleCircuitOpen(circuitKey, context);
      } else {
        // Transition to half-open
        circuitState.state = "half_open";
        this.logger.info(
          `Circuit breaker transitioning to half-open: ${circuitKey}`
        );
      }
    }

    let lastError: Error | null = null;

    for (let attempt = 1; attempt <= this.config.maxRetries + 1; attempt++) {
      try {
        const result = await operation();

        // Success - reset circuit breaker
        if (circuitState.failures > 0) {
          circuitState.failures = 0;
          circuitState.state = "closed";
          this.logger.info(`Circuit breaker reset to closed: ${circuitKey}`);
        }

        return result;
      } catch (error) {
        lastError = error as Error;

        const errorContext: ErrorContext = {
          ...context,
          timestamp: Date.now(),
          attempt,
          error: lastError,
        };

        this.recordError(errorContext);

        // Update circuit breaker
        this.recordCircuitFailure(circuitKey, errorContext);

        // Determine recovery action
        const action = this.determineRecoveryAction(errorContext, circuitState);

        // Notify callbacks
        this.notifyRecoveryAction(errorContext, action);

        if (action.type === "retry" && attempt <= this.config.maxRetries) {
          this.logger.warn(
            `Retrying operation (attempt ${attempt}/${this.config.maxRetries})`,
            {
              operation: context.operation,
              component: context.component,
              error: lastError.message,
              delay: action.delay,
            }
          );

          if (action.delay) {
            await new Promise((resolve) => setTimeout(resolve, action.delay));
          }
          continue;
        }

        // Handle final failure
        return this.handleFinalFailure(lastError, errorContext, action);
      }
    }

    // This should never be reached, but just in case
    throw lastError || new Error("Unknown error during operation execution");
  }

  private getCircuitBreakerState(key: string): CircuitBreakerState {
    if (!this.circuitBreakers.has(key)) {
      this.circuitBreakers.set(key, {
        failures: 0,
        lastFailureTime: 0,
        state: "closed",
        nextAttemptTime: 0,
      });
    }
    return this.circuitBreakers.get(key)!;
  }

  private recordCircuitFailure(key: string, _errorContext: ErrorContext): void {
    const state = this.circuitBreakers.get(key)!;
    state.failures++;
    state.lastFailureTime = Date.now();

    // Check if circuit should open
    if (
      this.config.circuitBreakerEnabled &&
      state.failures >= this.config.circuitBreakerThreshold
    ) {
      state.state = "open";
      state.nextAttemptTime = Date.now() + this.config.circuitBreakerTimeout;
      this.logger.error(`Circuit breaker opened: ${key}`, {
        failures: state.failures,
        timeout: this.config.circuitBreakerTimeout,
      });
    }
  }

  private determineRecoveryAction(
    errorContext: ErrorContext,
    circuitState: CircuitBreakerState
  ): RecoveryAction {
    const error = errorContext.error;

    // Circuit breaker logic
    if (circuitState.state === "open") {
      return {
        type: "circuit_break",
        message: `Circuit breaker is open for ${errorContext.component}:${errorContext.operation}`,
      };
    }

    // Retryable errors
    if (this.isRetryableError(error)) {
      if (errorContext.attempt <= this.config.maxRetries) {
        return {
          type: "retry",
          delay: this.calculateRetryDelay(errorContext.attempt),
          message: `Retrying after ${error.name}: ${error.message}`,
        };
      }
    }

    // Graceful degradation for non-critical operations
    if (
      this.config.gracefulDegradationEnabled &&
      this.isDegradableOperation(errorContext.operation)
    ) {
      return {
        type: "degrade",
        alternative: this.getDegradationAlternative(errorContext.operation),
        message: `Degrading operation: ${errorContext.operation}`,
      };
    }

    // Final failure
    return {
      type: "fail",
      message: `Operation failed after ${errorContext.attempt} attempts: ${error.message}`,
    };
  }

  private isRetryableError(error: Error): boolean {
    // Network errors
    if (
      error.message.includes("ECONNREFUSED") ||
      error.message.includes("ENOTFOUND") ||
      error.message.includes("ETIMEDOUT")
    ) {
      return true;
    }

    // Temporary server errors
    if (
      error.message.includes("502") ||
      error.message.includes("503") ||
      error.message.includes("504")
    ) {
      return true;
    }

    // Rate limiting
    if (error.message.includes("429") || error.message.includes("rate limit")) {
      return true;
    }

    return false;
  }

  private isDegradableOperation(operation: string): boolean {
    // Operations that can be safely degraded
    const degradableOps = [
      "cache_read",
      "metrics_collection",
      "background_sync",
      "log_aggregation",
    ];

    return degradableOps.some((op) => operation.includes(op));
  }

  private getDegradationAlternative(operation: string): string {
    // Return alternative implementations for degraded operations
    if (operation.includes("cache_read")) {
      return "database_fallback";
    }
    if (operation.includes("metrics")) {
      return "basic_metrics";
    }
    if (operation.includes("background_sync")) {
      return "skip_sync";
    }
    if (operation.includes("log_aggregation")) {
      return "local_only";
    }

    return "skip_operation";
  }

  private calculateRetryDelay(attempt: number): number {
    // Exponential backoff with jitter
    const baseDelay = this.config.retryDelay;
    const exponentialDelay = baseDelay * Math.pow(2, attempt - 1);
    const jitter = Math.random() * 0.1 * exponentialDelay;
    return Math.min(exponentialDelay + jitter, 30000); // Max 30 seconds
  }

  private async handleCircuitOpen(
    circuitKey: string,
    context: Omit<ErrorContext, "timestamp" | "attempt" | "error">
  ): Promise<never> {
    const message = `Circuit breaker is open for ${context.component}:${context.operation}`;
    this.logger.error(message);

    if (this.config.gracefulDegradationEnabled) {
      throw new Error(`${message} - operation degraded`);
    } else {
      throw new Error(`${message} - operation failed`);
    }
  }

  private async handleFinalFailure(
    error: Error,
    context: ErrorContext,
    action: RecoveryAction
  ): Promise<never> {
    this.logger.error(`Operation failed after all recovery attempts`, {
      operation: context.operation,
      component: context.component,
      attempts: context.attempt,
      finalError: error.message,
      action: action.type,
    });

    // Alert on persistent failures
    if (this.config.alertOnFailures) {
      this.notifyRecoveryAction(context, action);
    }

    throw error;
  }

  private recordError(context: ErrorContext): void {
    // Keep only recent errors (last 1000)
    if (this.errorHistory.length >= 1000) {
      this.errorHistory.shift();
    }
    this.errorHistory.push(context);
  }

  private notifyRecoveryAction(
    error: ErrorContext,
    action: RecoveryAction
  ): void {
    for (const callback of this.recoveryCallbacks) {
      try {
        callback(error, action);
      } catch (err) {
        this.logger.error("Error in recovery callback", {
          error: (err as Error).message,
        });
      }
    }
  }

  onRecoveryAction(
    callback: (error: ErrorContext, action: RecoveryAction) => void
  ): void {
    this.recoveryCallbacks.push(callback);
  }

  getErrorStats(hours: number = 1): {
    totalErrors: number;
    errorsByComponent: Record<string, number>;
    errorsByOperation: Record<string, number>;
    circuitBreakerStates: Record<string, CircuitBreakerState>;
  } {
    const cutoffTime = Date.now() - hours * 60 * 60 * 1000;
    const recentErrors = this.errorHistory.filter(
      (e) => e.timestamp > cutoffTime
    );

    const errorsByComponent: Record<string, number> = {};
    const errorsByOperation: Record<string, number> = {};

    for (const error of recentErrors) {
      errorsByComponent[error.component] =
        (errorsByComponent[error.component] || 0) + 1;
      errorsByOperation[error.operation] =
        (errorsByOperation[error.operation] || 0) + 1;
    }

    const circuitBreakerStates: Record<string, CircuitBreakerState> = {};
    for (const [key, state] of this.circuitBreakers.entries()) {
      circuitBreakerStates[key] = { ...state };
    }

    return {
      totalErrors: recentErrors.length,
      errorsByComponent,
      errorsByOperation,
      circuitBreakerStates,
    };
  }

  resetCircuitBreaker(key: string): boolean {
    const state = this.circuitBreakers.get(key);
    if (state) {
      state.failures = 0;
      state.state = "closed";
      state.nextAttemptTime = 0;
      this.logger.info(`Circuit breaker manually reset: ${key}`);
      return true;
    }
    return false;
  }

  private startMaintenanceCycle(): void {
    // Clean up old circuit breakers and error history every hour
    setInterval(() => {
      this.performMaintenance();
    }, 60 * 60 * 1000);
  }

  private performMaintenance(): void {
    this.logger.debug("Starting error recovery maintenance");

    // Clean up old error history (keep only last 24 hours)
    const oneDayAgo = Date.now() - 24 * 60 * 60 * 1000;
    const originalLength = this.errorHistory.length;
    this.errorHistory = this.errorHistory.filter(
      (e) => e.timestamp > oneDayAgo
    );

    // Reset old circuit breakers
    for (const [key, state] of this.circuitBreakers.entries()) {
      if (
        state.state === "open" &&
        Date.now() > state.nextAttemptTime + 24 * 60 * 60 * 1000
      ) {
        // Reset circuit breakers that have been open for more than 24 hours
        state.failures = 0;
        state.state = "closed";
        state.nextAttemptTime = 0;
        this.logger.info(`Circuit breaker auto-reset after timeout: ${key}`);
      }
    }

    const cleanedErrors = originalLength - this.errorHistory.length;
    if (cleanedErrors > 0) {
      this.logger.info("Error recovery maintenance completed", {
        errorsCleaned: cleanedErrors,
        circuitBreakersActive: this.circuitBreakers.size,
      });
    }
  }
}
