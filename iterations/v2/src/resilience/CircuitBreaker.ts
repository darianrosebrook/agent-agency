/**
 * @fileoverview Circuit Breaker Pattern Implementation
 *
 * Prevents cascading failures by temporarily blocking requests to a failing service.
 * Implements the three-state circuit breaker pattern: CLOSED → OPEN → HALF_OPEN
 *
 * @author @darianrosebrook
 */

export enum CircuitState {
  CLOSED = "CLOSED", // Normal operation
  OPEN = "OPEN", // Blocking requests
  HALF_OPEN = "HALF_OPEN", // Testing if service recovered
}

export interface CircuitBreakerConfig {
  /** Threshold of failures before opening circuit */
  failureThreshold: number;

  /** Time window (ms) for counting failures */
  failureWindowMs: number;

  /** Time (ms) to wait before attempting recovery */
  resetTimeoutMs: number;

  /** Number of successful requests needed to close circuit */
  successThreshold: number;

  /** Name for logging and monitoring */
  name: string;
}

export interface CircuitBreakerStats {
  state: CircuitState;
  failures: number;
  successes: number;
  totalRequests: number;
  lastFailureTime?: Date;
  lastSuccessTime?: Date;
  lastStateChange: Date;
}

/**
 * Circuit Breaker for preventing cascading failures
 */
export class CircuitBreaker {
  private state: CircuitState = CircuitState.CLOSED;
  private failures: number = 0;
  private successes: number = 0;
  private totalRequests: number = 0;
  private lastFailureTime?: Date;
  private lastSuccessTime?: Date;
  private lastStateChange: Date = new Date();
  private failureTimestamps: number[] = [];

  constructor(private config: CircuitBreakerConfig) {}

  /**
   * Execute a function with circuit breaker protection
   */
  async execute<T>(fn: () => Promise<T>): Promise<T> {
    this.totalRequests++;

    // Check if circuit is open
    if (this.state === CircuitState.OPEN) {
      if (this.shouldAttemptReset()) {
        this.transitionToHalfOpen();
      } else {
        throw new CircuitBreakerOpenError(
          `Circuit breaker "${this.config.name}" is OPEN`,
          this.config.name,
          this.getStats()
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

  /**
   * Record a success
   */
  private onSuccess(): void {
    this.successes++;
    this.lastSuccessTime = new Date();

    if (this.state === CircuitState.HALF_OPEN) {
      if (this.successes >= this.config.successThreshold) {
        this.transitionToClosed();
      }
    } else if (this.state === CircuitState.CLOSED) {
      // Reset failure count on success
      this.failures = 0;
      this.failureTimestamps = [];
    }
  }

  /**
   * Record a failure
   */
  private onFailure(): void {
    this.failures++;
    this.lastFailureTime = new Date();
    this.failureTimestamps.push(Date.now());

    // Remove old failures outside time window
    const cutoff = Date.now() - this.config.failureWindowMs;
    this.failureTimestamps = this.failureTimestamps.filter((t) => t > cutoff);

    if (this.state === CircuitState.HALF_OPEN) {
      // Immediately open on failure in half-open state
      this.transitionToOpen();
    } else if (
      this.state === CircuitState.CLOSED &&
      this.failureTimestamps.length >= this.config.failureThreshold
    ) {
      this.transitionToOpen();
    }
  }

  /**
   * Check if enough time has passed to attempt reset
   */
  private shouldAttemptReset(): boolean {
    if (!this.lastFailureTime) {
      return true;
    }

    const timeSinceLastFailure = Date.now() - this.lastFailureTime.getTime();
    return timeSinceLastFailure >= this.config.resetTimeoutMs;
  }

  /**
   * Transition to CLOSED state
   */
  private transitionToClosed(): void {
    console.log(`[CircuitBreaker:${this.config.name}] Transitioning to CLOSED`);
    this.state = CircuitState.CLOSED;
    this.failures = 0;
    this.successes = 0;
    this.failureTimestamps = [];
    this.lastStateChange = new Date();
  }

  /**
   * Transition to OPEN state
   */
  private transitionToOpen(): void {
    console.log(`[CircuitBreaker:${this.config.name}] Transitioning to OPEN`);
    this.state = CircuitState.OPEN;
    this.successes = 0;
    this.lastStateChange = new Date();
  }

  /**
   * Transition to HALF_OPEN state
   */
  private transitionToHalfOpen(): void {
    console.log(
      `[CircuitBreaker:${this.config.name}] Transitioning to HALF_OPEN`
    );
    this.state = CircuitState.HALF_OPEN;
    this.successes = 0;
    this.lastStateChange = new Date();
  }

  /**
   * Get current circuit breaker stats
   */
  getStats(): CircuitBreakerStats {
    return {
      state: this.state,
      failures: this.failures,
      successes: this.successes,
      totalRequests: this.totalRequests,
      lastFailureTime: this.lastFailureTime,
      lastSuccessTime: this.lastSuccessTime,
      lastStateChange: this.lastStateChange,
    };
  }

  /**
   * Manually reset the circuit breaker
   */
  reset(): void {
    this.transitionToClosed();
  }

  /**
   * Get current state
   */
  getState(): CircuitState {
    return this.state;
  }
}

/**
 * Error thrown when circuit breaker is open
 */
export class CircuitBreakerOpenError extends Error {
  constructor(
    message: string,
    public circuitName: string,
    public stats: CircuitBreakerStats
  ) {
    super(message);
    this.name = "CircuitBreakerOpenError";
  }
}
