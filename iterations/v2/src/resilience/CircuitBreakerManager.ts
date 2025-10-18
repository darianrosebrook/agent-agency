/**
 * Circuit Breaker Manager for V2 Arbiter
 * 
 * Provides centralized circuit breaker management for external dependencies
 * and critical system components to prevent cascade failures.
 * 
 * @author @darianrosebrook
 */

import { EventEmitter } from "events";

export interface CircuitBreakerConfig {
  failureThreshold: number;
  recoveryTimeout: number;
  monitoringPeriod: number;
  halfOpenMaxCalls: number;
}

export interface CircuitBreakerStats {
  state: "closed" | "open" | "half-open";
  failureCount: number;
  successCount: number;
  lastFailureTime?: Date;
  nextAttemptTime?: Date;
}

export class CircuitBreaker extends EventEmitter {
  private state: "closed" | "open" | "half-open" = "closed";
  private failureCount = 0;
  private successCount = 0;
  private lastFailureTime?: Date;
  private nextAttemptTime?: Date;
  private halfOpenCalls = 0;

  constructor(
    private name: string,
    private config: CircuitBreakerConfig
  ) {
    super();
  }

  async execute<T>(operation: () => Promise<T>): Promise<T> {
    if (this.state === "open") {
      if (this.nextAttemptTime && Date.now() < this.nextAttemptTime.getTime()) {
        throw new Error(`Circuit breaker ${this.name} is open`);
      }
      this.state = "half-open";
      this.halfOpenCalls = 0;
      this.emit("state-change", { from: "open", to: "half-open" });
    }

    if (this.state === "half-open" && this.halfOpenCalls >= this.config.halfOpenMaxCalls) {
      throw new Error(`Circuit breaker ${this.name} is half-open and max calls reached`);
    }

    try {
      const result = await operation();
      this.onSuccess();
      return result;
    } catch (error) {
      this.onFailure();
      throw error;
    }
  }

  private onSuccess(): void {
    this.successCount++;
    this.halfOpenCalls++;

    if (this.state === "half-open") {
      this.state = "closed";
      this.failureCount = 0;
      this.nextAttemptTime = undefined;
      this.emit("state-change", { from: "half-open", to: "closed" });
    }
  }

  private onFailure(): void {
    this.failureCount++;
    this.lastFailureTime = new Date();
    this.halfOpenCalls++;

    if (this.state === "half-open") {
      this.state = "open";
      this.nextAttemptTime = new Date(Date.now() + this.config.recoveryTimeout);
      this.emit("state-change", { from: "half-open", to: "open" });
    } else if (this.failureCount >= this.config.failureThreshold) {
      this.state = "open";
      this.nextAttemptTime = new Date(Date.now() + this.config.recoveryTimeout);
      this.emit("state-change", { from: "closed", to: "open" });
    }
  }

  getStats(): CircuitBreakerStats {
    return {
      state: this.state,
      failureCount: this.failureCount,
      successCount: this.successCount,
      lastFailureTime: this.lastFailureTime,
      nextAttemptTime: this.nextAttemptTime,
    };
  }

  reset(): void {
    this.state = "closed";
    this.failureCount = 0;
    this.successCount = 0;
    this.lastFailureTime = undefined;
    this.nextAttemptTime = undefined;
    this.halfOpenCalls = 0;
    this.emit("reset");
  }
}

export class CircuitBreakerManager {
  private breakers = new Map<string, CircuitBreaker>();
  private defaultConfig: CircuitBreakerConfig = {
    failureThreshold: 5,
    recoveryTimeout: 30000, // 30 seconds
    monitoringPeriod: 60000, // 1 minute
    halfOpenMaxCalls: 3,
  };

  getBreaker(name: string, config?: Partial<CircuitBreakerConfig>): CircuitBreaker {
    if (!this.breakers.has(name)) {
      const breakerConfig = { ...this.defaultConfig, ...config };
      const breaker = new CircuitBreaker(name, breakerConfig);
      this.breakers.set(name, breaker);
    }
    return this.breakers.get(name)!;
  }

  getAllStats(): Record<string, CircuitBreakerStats> {
    const stats: Record<string, CircuitBreakerStats> = {};
    for (const [name, breaker] of this.breakers) {
      stats[name] = breaker.getStats();
    }
    return stats;
  }

  resetAll(): void {
    for (const breaker of this.breakers.values()) {
      breaker.reset();
    }
  }
}

// Global instance
export const circuitBreakerManager = new CircuitBreakerManager();
