/**
 * @fileoverview Circuit Breaker Pattern Implementation
 *
 * Prevents cascade failures by isolating failing services and providing
 * automatic recovery mechanisms.
 *
 * @author @darianrosebrook
 */

export interface CircuitBreakerConfig {
  /** Number of failures before opening the circuit */
  failureThreshold: number;
  /** Time in ms before attempting to close the circuit */
  recoveryTimeout: number;
  /** Time window in ms for monitoring failures */
  monitoringPeriod: number;
  /** Success threshold for closing the circuit */
  successThreshold?: number;
  /** Name for logging and monitoring */
  name?: string;
}

export enum CircuitState {
  CLOSED = "closed", // Normal operation
  OPEN = "open", // Failing, requests rejected
  HALF_OPEN = "half-open", // Testing recovery
}

export interface CircuitBreakerStats {
  state: CircuitState;
  failures: number;
  successes: number;
  lastFailureTime: number | null;
  lastSuccessTime: number | null;
  consecutiveSuccesses: number;
  consecutiveFailures: number;
  totalRequests: number;
  totalFailures: number;
  totalSuccesses: number;
}

/**
 * Circuit Breaker implementation for isolating failing services
 */
export class CircuitBreaker {
  private config: Required<CircuitBreakerConfig>;
  private state: CircuitState = CircuitState.CLOSED;
  private failures: number = 0;
  private successes: number = 0;
  private lastFailureTime: number | null = null;
  private lastSuccessTime: number | null = null;
  private consecutiveSuccesses: number = 0;
  private consecutiveFailures: number = 0;
  private totalRequests: number = 0;
  private totalFailures: number = 0;
  private totalSuccesses: number = 0;
  private nextAttemptTime: number = 0;
  private recoveryTimer?: ReturnType<typeof setTimeout>;

  constructor(config: CircuitBreakerConfig) {
    this.config = {
      successThreshold: 3,
      name: "CircuitBreaker",
      ...config,
    };
  }

  /**
   * Execute a function with circuit breaker protection
   */
  async execute<T>(fn: () => Promise<T>): Promise<T> {
    this.totalRequests++;

    if (this.state === CircuitState.OPEN) {
      if (Date.now() < this.nextAttemptTime) {
        throw new CircuitBreakerError(
          `Circuit breaker is OPEN for ${this.config.name}`,
          CircuitBreakerError.CIRCUIT_OPEN
        );
      }

      // Transition to half-open for testing
      this.state = CircuitState.HALF_OPEN;
      console.log(
        `Circuit breaker ${this.config.name} transitioning to HALF_OPEN`
      );
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

  /**
   * Handle successful operation
   */
  private onSuccess(): void {
    this.successes++;
    this.totalSuccesses++;
    this.lastSuccessTime = Date.now();
    this.consecutiveSuccesses++;
    this.consecutiveFailures = 0;

    if (this.state === CircuitState.HALF_OPEN) {
      if (this.consecutiveSuccesses >= this.config.successThreshold) {
        this.closeCircuit();
      }
    } else if (this.state === CircuitState.CLOSED) {
      // Reset failure counters on success
      this.failures = 0;
    }
  }

  /**
   * Handle failed operation
   */
  private onFailure(): void {
    this.failures++;
    this.totalFailures++;
    this.lastFailureTime = Date.now();
    this.consecutiveFailures++;
    this.consecutiveSuccesses = 0;

    // Check if we should open the circuit
    if (this.state === CircuitState.CLOSED && this.shouldOpenCircuit()) {
      this.openCircuit();
    } else if (this.state === CircuitState.HALF_OPEN) {
      // Failed during recovery test, go back to open
      this.openCircuit();
    }
  }

  /**
   * Determine if circuit should open based on failure threshold
   */
  private shouldOpenCircuit(): boolean {
    // Simple threshold check
    return this.consecutiveFailures >= this.config.failureThreshold;
  }

  /**
   * Open the circuit breaker
   */
  private openCircuit(): void {
    this.state = CircuitState.OPEN;
    this.nextAttemptTime = Date.now() + this.config.recoveryTimeout;

    console.warn(
      `Circuit breaker ${this.config.name} OPENED after ${this.consecutiveFailures} consecutive failures`
    );

    // Schedule recovery attempt
    this.recoveryTimer = setTimeout(() => {
      if (this.state === CircuitState.OPEN) {
        this.state = CircuitState.HALF_OPEN;
        console.log(
          `Circuit breaker ${this.config.name} transitioning to HALF_OPEN for recovery test`
        );
      }
    }, this.config.recoveryTimeout);
  }

  /**
   * Close the circuit breaker
   */
  private closeCircuit(): void {
    this.state = CircuitState.CLOSED;
    this.failures = 0;
    this.consecutiveFailures = 0;
    this.consecutiveSuccesses = 0;

    if (this.recoveryTimer) {
      clearTimeout(this.recoveryTimer);
      this.recoveryTimer = undefined;
    }

    console.log(
      `Circuit breaker ${this.config.name} CLOSED after successful recovery`
    );
  }

  /**
   * Get current circuit breaker statistics
   */
  getStats(): CircuitBreakerStats {
    return {
      state: this.state,
      failures: this.failures,
      successes: this.successes,
      lastFailureTime: this.lastFailureTime,
      lastSuccessTime: this.lastSuccessTime,
      consecutiveSuccesses: this.consecutiveSuccesses,
      consecutiveFailures: this.consecutiveFailures,
      totalRequests: this.totalRequests,
      totalFailures: this.totalFailures,
      totalSuccesses: this.totalSuccesses,
    };
  }

  /**
   * Get current circuit state
   */
  getState(): CircuitState {
    return this.state;
  }

  /**
   * Force circuit state (for testing/admin purposes)
   */
  forceState(state: CircuitState): void {
    console.warn(
      `Circuit breaker ${this.config.name} manually forced to ${state}`
    );
    this.state = state;

    if (state === CircuitState.CLOSED) {
      this.failures = 0;
      this.consecutiveFailures = 0;
      this.consecutiveSuccesses = 0;
      if (this.recoveryTimer) {
        clearTimeout(this.recoveryTimer);
        this.recoveryTimer = undefined;
      }
    }
  }

  /**
   * Reset circuit breaker statistics
   */
  reset(): void {
    this.failures = 0;
    this.successes = 0;
    this.lastFailureTime = null;
    this.lastSuccessTime = null;
    this.consecutiveSuccesses = 0;
    this.consecutiveFailures = 0;
    this.totalRequests = 0;
    this.totalFailures = 0;
    this.totalSuccesses = 0;
    this.nextAttemptTime = 0;

    if (this.recoveryTimer) {
      clearTimeout(this.recoveryTimer);
      this.recoveryTimer = undefined;
    }

    if (this.state !== CircuitState.CLOSED) {
      this.state = CircuitState.CLOSED;
      console.log(`Circuit breaker ${this.config.name} reset to CLOSED`);
    }
  }

  /**
   * Cleanup resources
   */
  destroy(): void {
    if (this.recoveryTimer) {
      clearTimeout(this.recoveryTimer);
      this.recoveryTimer = undefined;
    }
    console.log(`Circuit breaker ${this.config.name} destroyed`);
  }
}

/**
 * Circuit breaker specific error
 */
export class CircuitBreakerError extends Error {
  static readonly CIRCUIT_OPEN = "CIRCUIT_OPEN";
  static readonly TIMEOUT = "TIMEOUT";
  static readonly FAILURE = "FAILURE";

  public readonly code: string;

  constructor(message: string, code: string) {
    super(message);
    this.name = "CircuitBreakerError";
    this.code = code;
  }
}

/**
 * Circuit breaker factory for common configurations
 */
export class CircuitBreakerFactory {
  static createForOllama(name = "ollama-embedding"): CircuitBreaker {
    return new CircuitBreaker({
      name,
      failureThreshold: 5, // Open after 5 failures
      recoveryTimeout: 60000, // Try to recover after 1 minute
      monitoringPeriod: 10000, // Monitor over 10 second windows
      successThreshold: 3, // Need 3 successes to close
    });
  }

  static createForDatabase(name = "database-connection"): CircuitBreaker {
    return new CircuitBreaker({
      name,
      failureThreshold: 3, // More sensitive for database
      recoveryTimeout: 30000, // Faster recovery for database
      monitoringPeriod: 5000, // Shorter monitoring window
      successThreshold: 2, // Need 2 successes to close
    });
  }

  static createForExternalAPI(
    name: string,
    config?: Partial<CircuitBreakerConfig>
  ): CircuitBreaker {
    return new CircuitBreaker({
      name,
      failureThreshold: 10, // More tolerant for external APIs
      recoveryTimeout: 120000, // Longer recovery time
      monitoringPeriod: 30000, // Longer monitoring period
      successThreshold: 5, // Need more successes to close
      ...config,
    });
  }
}
