/**
 * @fileoverview Chaos Testing Harness - ARBITER-024
 *
 * Provides deterministic chaos engineering capabilities for testing
 * arbiter resilience against worker failures, network issues, and edge cases.
 *
 * @author @darianrosebrook
 */

import { EventEmitter } from "events";

export interface ChaosScenario {
  id: string;
  name: string;
  description: string;
  probability: number; // 0-1, probability of triggering
  duration: number; // milliseconds
  recoveryTime?: number; // milliseconds to recover
  targetWorkers?: string[]; // specific workers to target
  targetCapabilities?: string[]; // capabilities to target
  conditions?: ChaosCondition[];
  severity?: "low" | "medium" | "high" | "critical";
}

export interface ChaosCondition {
  type:
    | "worker_saturation"
    | "task_load"
    | "time_of_day"
    | "resource_usage"
    | "system_load";
  operator: ">" | "<" | "=" | ">=" | "<=";
  value: number;
  field?: string; // specific field to check
}

export interface ChaosEvent {
  id: string;
  scenarioId: string;
  timestamp: Date;
  workerId?: string;
  capability?: string;
  eventType:
    | "failure"
    | "degradation"
    | "recovery"
    | "network_issue"
    | "resource_exhaustion";
  severity: "low" | "medium" | "high" | "critical";
  metadata: Record<string, any>;
}

export interface ChaosMetrics {
  totalScenarios: number;
  activeScenarios: number;
  eventsTriggered: number;
  workersAffected: number;
  averageRecoveryTime: number;
  failureRate: number;
}

export interface DeterministicPRNG {
  seed: number;
  next(): number; // Returns 0-1
  nextInt(_max: number): number; // Returns 0 to max-1
  nextGaussian(): number; // Returns normally distributed value
  reset(_seed: number): void;
}

/**
 * Deterministic pseudo-random number generator using Linear Congruential Generator
 */
export class SeededPRNG implements DeterministicPRNG {
  private currentSeed: number;
  public seed: number;

  constructor(seed: number = 12345) {
    this.currentSeed = seed;
    this.seed = seed;
  }

  next(): number {
    // LCG formula: (a * seed + c) % m
    // Using values from Numerical Recipes
    this.currentSeed = (this.currentSeed * 1664525 + 1013904223) % 4294967296;
    return this.currentSeed / 4294967296;
  }

  nextInt(max: number): number {
    return Math.floor(this.next() * max);
  }

  nextGaussian(): number {
    // Box-Muller transform for normal distribution
    if (this.hasSpare) {
      this.hasSpare = false;
      return this.spare * this.norm;
    }
    this.hasSpare = true;
    const u = this.next();
    const v = this.next();
    this.spare = Math.sqrt(-2 * Math.log(u)) * Math.sin(2 * Math.PI * v);
    return this.spare * this.norm;
  }

  private hasSpare = false;
  private spare = 0;
  private norm = 1;

  reset(seed: number): void {
    this.currentSeed = seed;
    this.seed = seed;
    this.hasSpare = false;
    this.spare = 0;
  }
}

export class ChaosTestingHarness extends EventEmitter {
  private scenarios: Map<string, ChaosScenario> = new Map();
  private activeEvents: Map<string, ChaosEvent> = new Map();
  private prng: DeterministicPRNG;
  private isEnabled = false;
  private monitoringInterval: ReturnType<typeof setInterval> | null = null;
  private metrics: ChaosMetrics;

  constructor(seed: number = 12345) {
    super();
    this.prng = new SeededPRNG(seed);
    this.metrics = {
      totalScenarios: 0,
      activeScenarios: 0,
      eventsTriggered: 0,
      workersAffected: 0,
      averageRecoveryTime: 0,
      failureRate: 0,
    };
  }

  /**
   * Add a chaos scenario to the harness
   */
  addScenario(scenario: ChaosScenario): void {
    this.scenarios.set(scenario.id, scenario);
    this.metrics.totalScenarios++;
    this.emit("scenarioAdded", scenario);
  }

  /**
   * Remove a chaos scenario
   */
  removeScenario(scenarioId: string): boolean {
    const removed = this.scenarios.delete(scenarioId);
    if (removed) {
      this.metrics.totalScenarios--;
      this.emit("scenarioRemoved", scenarioId);
    }
    return removed;
  }

  /**
   * Enable chaos testing
   */
  enable(): void {
    this.isEnabled = true;
    this.startMonitoring();
    this.emit("chaosEnabled");
  }

  /**
   * Disable chaos testing
   */
  disable(): void {
    this.isEnabled = false;
    this.stopMonitoring();
    this.clearAllEvents();
    this.emit("chaosDisabled");
  }

  /**
   * Reset the PRNG with a new seed
   */
  resetSeed(seed: number): void {
    this.prng.reset(seed);
    this.emit("seedReset", seed);
  }

  /**
   * Simulate worker failure
   */
  simulateWorkerFailure(
    workerId: string,
    failureType: "crash" | "timeout" | "resource_exhaustion" = "crash"
  ): ChaosEvent {
    const event: ChaosEvent = {
      id: `chaos-${Date.now()}-${this.prng.nextInt(10000)}`,
      scenarioId: "manual",
      timestamp: new Date(),
      workerId,
      eventType: "failure",
      severity: this.getRandomSeverity(),
      metadata: {
        failureType,
        simulated: true,
      },
    };

    this.activeEvents.set(event.id, event);
    this.metrics.eventsTriggered++;
    this.metrics.workersAffected++;

    this.emit("workerFailure", event);
    return event;
  }

  /**
   * Simulate network degradation
   */
  simulateNetworkDegradation(
    workerIds: string[],
    severity: "low" | "medium" | "high" = "medium"
  ): ChaosEvent {
    const event: ChaosEvent = {
      id: `chaos-${Date.now()}-${this.prng.nextInt(10000)}`,
      scenarioId: "manual",
      timestamp: new Date(),
      eventType: "network_issue",
      severity,
      metadata: {
        affectedWorkers: workerIds,
        simulated: true,
        degradationLevel: this.getDegradationLevel(severity),
      },
    };

    this.activeEvents.set(event.id, event);
    this.metrics.eventsTriggered++;
    this.metrics.workersAffected += workerIds.length;

    this.emit("networkDegradation", event);
    return event;
  }

  /**
   * Simulate resource exhaustion
   */
  simulateResourceExhaustion(
    resourceType: "memory" | "cpu" | "disk" | "network",
    workerId?: string
  ): ChaosEvent {
    const event: ChaosEvent = {
      id: `chaos-${Date.now()}-${this.prng.nextInt(10000)}`,
      scenarioId: "manual",
      timestamp: new Date(),
      workerId,
      eventType: "resource_exhaustion",
      severity: this.getRandomSeverity(),
      metadata: {
        resourceType,
        simulated: true,
        exhaustionLevel: this.prng.next(),
      },
    };

    this.activeEvents.set(event.id, event);
    this.metrics.eventsTriggered++;
    if (workerId) {
      this.metrics.workersAffected++;
    }

    this.emit("resourceExhaustion", event);
    return event;
  }

  /**
   * Get current chaos metrics
   */
  getMetrics(): ChaosMetrics {
    this.updateMetrics();
    return { ...this.metrics };
  }

  /**
   * Get active chaos events
   */
  getActiveEvents(): ChaosEvent[] {
    return Array.from(this.activeEvents.values());
  }

  /**
   * Clear all active events
   */
  clearAllEvents(): void {
    this.activeEvents.clear();
    this.emit("eventsCleared");
  }

  /**
   * Generate a deterministic chaos event based on current state
   */
  generateDeterministicEvent(context: {
    workerCount: number;
    taskLoad: number;
    timeOfDay: number; // 0-24
    systemLoad: number; // 0-1
  }): ChaosEvent | null {
    if (!this.isEnabled || this.scenarios.size === 0) {
      return null;
    }

    // Find applicable scenarios
    const applicableScenarios = Array.from(this.scenarios.values()).filter(
      (scenario) => {
        if (!scenario.conditions || scenario.conditions.length === 0) {
          return true;
        }

        return scenario.conditions.every((condition) => {
          switch (condition.type) {
            case "worker_saturation":
              return this.evaluateCondition(
                context.workerCount,
                condition.operator,
                condition.value
              );
            case "task_load":
              return this.evaluateCondition(
                context.taskLoad,
                condition.operator,
                condition.value
              );
            case "time_of_day":
              return this.evaluateCondition(
                context.timeOfDay,
                condition.operator,
                condition.value
              );
            case "resource_usage":
              return this.evaluateCondition(
                context.systemLoad,
                condition.operator,
                condition.value
              );
            default:
              return true;
          }
        });
      }
    );

    if (applicableScenarios.length === 0) {
      return null;
    }

    // Select scenario based on probability and deterministic randomness
    const totalProbability = applicableScenarios.reduce(
      (sum, s) => sum + s.probability,
      0
    );
    const randomValue = this.prng.next() * totalProbability;

    let cumulativeProbability = 0;
    let selectedScenario: ChaosScenario | null = null;

    for (const scenario of applicableScenarios) {
      cumulativeProbability += scenario.probability;
      if (randomValue <= cumulativeProbability) {
        selectedScenario = scenario;
        break;
      }
    }

    if (!selectedScenario) {
      return null;
    }

    // Generate event based on selected scenario
    const event: ChaosEvent = {
      id: `chaos-${Date.now()}-${this.prng.nextInt(10000)}`,
      scenarioId: selectedScenario.id,
      timestamp: new Date(),
      eventType: this.getEventTypeFromScenario(selectedScenario),
      severity: this.getRandomSeverity(),
      metadata: {
        scenario: selectedScenario.name,
        duration: selectedScenario.duration,
        simulated: true,
      },
    };

    this.activeEvents.set(event.id, event);
    this.metrics.eventsTriggered++;

    // Schedule recovery if specified
    if (selectedScenario.recoveryTime) {
      setTimeout(() => {
        this.recoverEvent(event.id);
      }, selectedScenario.recoveryTime!);
    }

    this.emit("chaosEvent", event);
    return event;
  }

  /**
   * Start monitoring for chaos events
   */
  private startMonitoring(): void {
    if (this.monitoringInterval) {
      return;
    }

    this.monitoringInterval = setInterval(() => {
      this.monitoringTick();
    }, 1000); // Check every second
  }

  /**
   * Stop monitoring
   */
  private stopMonitoring(): void {
    if (this.monitoringInterval) {
      clearInterval(this.monitoringInterval);
      this.monitoringInterval = null;
    }
  }

  /**
   * Monitoring tick - evaluate scenarios and generate events
   */
  private monitoringTick(): void {
    // This would integrate with actual system metrics
    // For now, we'll generate events based on time and deterministic randomness
    const context = {
      workerCount: 10, // Would be actual worker count
      taskLoad: this.prng.next(), // Would be actual task load
      timeOfDay: new Date().getHours(),
      systemLoad: this.prng.next(), // Would be actual system load
    };

    this.generateDeterministicEvent(context);
  }

  /**
   * Recover from a chaos event
   */
  private recoverEvent(eventId: string): void {
    const event = this.activeEvents.get(eventId);
    if (!event) {
      return;
    }

    this.activeEvents.delete(eventId);

    const recoveryEvent: ChaosEvent = {
      id: `recovery-${Date.now()}-${this.prng.nextInt(10000)}`,
      scenarioId: event.scenarioId,
      timestamp: new Date(),
      workerId: event.workerId,
      eventType: "recovery",
      severity: "low",
      metadata: {
        originalEventId: eventId,
        simulated: true,
      },
    };

    this.emit("recovery", recoveryEvent);
  }

  /**
   * Evaluate a condition
   */
  private evaluateCondition(
    value: number,
    operator: string,
    conditionValue: number
  ): boolean {
    switch (operator) {
      case ">":
        return value > conditionValue;
      case "<":
        return value < conditionValue;
      case "=":
        return value === conditionValue;
      case ">=":
        return value >= conditionValue;
      case "<=":
        return value <= conditionValue;
      default:
        return false;
    }
  }

  /**
   * Get random severity based on deterministic PRNG
   */
  private getRandomSeverity(): "low" | "medium" | "high" | "critical" {
    const rand = this.prng.next();
    if (rand < 0.5) return "low";
    if (rand < 0.8) return "medium";
    if (rand < 0.95) return "high";
    return "critical";
  }

  /**
   * Get event type from scenario
   */
  private getEventTypeFromScenario(
    scenario: ChaosScenario
  ): ChaosEvent["eventType"] {
    const name = scenario.name.toLowerCase();
    if (name.includes("failure") || name.includes("crash")) return "failure";
    if (name.includes("degradation") || name.includes("slow"))
      return "degradation";
    if (name.includes("network")) return "network_issue";
    if (
      name.includes("resource") ||
      name.includes("memory") ||
      name.includes("cpu")
    )
      return "resource_exhaustion";
    return "failure"; // default
  }

  /**
   * Get degradation level based on severity
   */
  private getDegradationLevel(severity: string): number {
    switch (severity) {
      case "low":
        return 0.2;
      case "medium":
        return 0.5;
      case "high":
        return 0.8;
      case "critical":
        return 0.95;
      default:
        return 0.5;
    }
  }

  /**
   * Update metrics
   */
  private updateMetrics(): void {
    this.metrics.activeScenarios = this.activeEvents.size;

    if (this.metrics.eventsTriggered > 0) {
      // TODO: Implement comprehensive chaos testing metrics and analysis
      // - Calculate actual recovery time from event timestamps and system monitoring
      // - Implement statistical analysis of recovery time distributions
      // - Add recovery time trend analysis and anomaly detection
      // - Support recovery time correlation with system load and configuration
      // - Implement recovery time forecasting and optimization
      // - Add recovery time comparison across different failure scenarios
      // - Support recovery time-based system resilience scoring
      // - Implement recovery time alerting and threshold management
      this.metrics.averageRecoveryTime = this.prng.nextInt(5000) + 1000;

      // TODO: Implement comprehensive failure rate analysis and monitoring
      // - Calculate actual failure rates from system monitoring and logs
      // - Implement failure rate trend analysis and prediction
      // - Support failure rate correlation with system conditions and load
      // - Add failure rate anomaly detection and alerting
      // - Implement failure rate-based system health scoring
      // - Support failure rate comparison across different scenarios and time periods
      // - Add failure rate root cause analysis and classification
      // - Implement failure rate-based system reliability assessment
      this.metrics.failureRate = Math.min(
        this.metrics.eventsTriggered / 100,
        1
      );
    }
  }
}
