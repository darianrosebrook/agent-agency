/**
 * @fileoverview Chaos Test Suite - ARBITER-025
 *
 * Comprehensive test suite for chaos engineering scenarios,
 * testing arbiter resilience against various failure modes.
 *
 * @author @darianrosebrook
 */

import { TaskOrchestrator } from "../orchestrator/TaskOrchestrator";
import { WorkerCapabilityRegistry } from "../orchestrator/resources/WorkerCapabilityRegistry";
import {
  ChaosEvent,
  ChaosMetrics,
  ChaosTestingHarness,
} from "./ChaosTestingHarness";
import { CHAOS_SCENARIOS } from "./chaos-scenarios";

export interface ChaosTestConfig {
  seed: number;
  duration: number; // milliseconds
  scenarios: string[]; // scenario IDs to test
  targetWorkers: string[];
  metricsInterval: number; // milliseconds
  failureThreshold: number; // 0-1, maximum acceptable failure rate
}

export interface ChaosTestResult {
  testId: string;
  config: ChaosTestConfig;
  startTime: Date;
  endTime: Date;
  duration: number;
  events: ChaosEvent[];
  metrics: ChaosMetrics;
  success: boolean;
  failureRate: number;
  recoveryTime: number;
  workersAffected: number;
  scenarios: {
    id: string;
    triggered: number;
    successful: number;
    failed: number;
  }[];
}

export class ChaosTestSuite {
  private harness: ChaosTestingHarness;
  private orchestrator: TaskOrchestrator;
  private workerRegistry: WorkerCapabilityRegistry;
  private testResults: ChaosTestResult[] = [];

  constructor(
    orchestrator: TaskOrchestrator,
    workerRegistry: WorkerCapabilityRegistry,
    seed: number = 12345
  ) {
    this.orchestrator = orchestrator;
    this.workerRegistry = workerRegistry;
    this.harness = new ChaosTestingHarness(seed);
    this.setupEventHandlers();
  }

  /**
   * Run a comprehensive chaos test
   */
  async runChaosTest(config: ChaosTestConfig): Promise<ChaosTestResult> {
    const testId = `chaos-test-${Date.now()}`;
    const startTime = new Date();

    console.log(`Starting chaos test: ${testId}`);
    console.log(`Configuration:`, config);

    // Setup scenarios
    this.setupScenarios(config.scenarios);

    // Enable chaos testing
    this.harness.enable();

    // Start monitoring metrics
    const metricsInterval = setInterval(() => {
      this.logMetrics(testId);
    }, config.metricsInterval);

    // Run test for specified duration
    await this.sleep(config.duration);

    // Cleanup
    clearInterval(metricsInterval);
    this.harness.disable();

    const endTime = new Date();
    const result = this.generateTestResult(testId, config, startTime, endTime);

    this.testResults.push(result);
    this.logTestResult(result);

    return result;
  }

  /**
   * Run worker failure resilience test
   */
  async runWorkerFailureTest(
    workerIds: string[],
    failureCount: number = 3
  ): Promise<ChaosTestResult> {
    const config: ChaosTestConfig = {
      seed: 12345,
      duration: 300000, // 5 minutes
      scenarios: ["worker-crash-random", "worker-timeout-high-load"],
      targetWorkers: workerIds,
      metricsInterval: 30000, // 30 seconds
      failureThreshold: 0.1, // 10% failure rate
    };

    console.log(`Running worker failure test with ${failureCount} failures`);

    // Simulate specific worker failures
    for (let i = 0; i < failureCount; i++) {
      const workerId = workerIds[i % workerIds.length];
      this.harness.simulateWorkerFailure(workerId, "crash");

      // Wait between failures
      if (i < failureCount - 1) {
        await this.sleep(60000); // 1 minute between failures
      }
    }

    return this.runChaosTest(config);
  }

  /**
   * Run network degradation test
   */
  async runNetworkDegradationTest(
    workerIds: string[]
  ): Promise<ChaosTestResult> {
    const config: ChaosTestConfig = {
      seed: 54321,
      duration: 240000, // 4 minutes
      scenarios: ["network-latency-spike", "network-partition"],
      targetWorkers: workerIds,
      metricsInterval: 30000,
      failureThreshold: 0.15, // 15% failure rate
    };

    console.log("Running network degradation test");

    // Simulate network issues
    this.harness.simulateNetworkDegradation(workerIds, "high");

    return this.runChaosTest(config);
  }

  /**
   * Run resource exhaustion test
   */
  async runResourceExhaustionTest(): Promise<ChaosTestResult> {
    const config: ChaosTestConfig = {
      seed: 98765,
      duration: 180000, // 3 minutes
      scenarios: ["memory-exhaustion", "cpu-exhaustion"],
      targetWorkers: [],
      metricsInterval: 30000,
      failureThreshold: 0.2, // 20% failure rate
    };

    console.log("Running resource exhaustion test");

    // Simulate resource exhaustion
    this.harness.simulateResourceExhaustion("memory");
    this.harness.simulateResourceExhaustion("cpu");

    return this.runChaosTest(config);
  }

  /**
   * Run cascading failure test
   */
  async runCascadingFailureTest(workerIds: string[]): Promise<ChaosTestResult> {
    const config: ChaosTestConfig = {
      seed: 11111,
      duration: 600000, // 10 minutes
      scenarios: ["cascading-worker-failure"],
      targetWorkers: workerIds,
      metricsInterval: 60000, // 1 minute
      failureThreshold: 0.3, // 30% failure rate (higher for cascading)
    };

    console.log("Running cascading failure test");

    return this.runChaosTest(config);
  }

  /**
   * Run comprehensive resilience test
   */
  async runComprehensiveResilienceTest(
    workerIds: string[]
  ): Promise<ChaosTestResult[]> {
    console.log("Running comprehensive resilience test suite");

    const results: ChaosTestResult[] = [];

    // Test individual failure modes
    results.push(await this.runWorkerFailureTest(workerIds, 2));
    await this.sleep(60000); // Wait between tests

    results.push(await this.runNetworkDegradationTest(workerIds));
    await this.sleep(60000);

    results.push(await this.runResourceExhaustionTest());
    await this.sleep(60000);

    results.push(await this.runCascadingFailureTest(workerIds));

    // Test with all scenarios combined
    const allScenariosConfig: ChaosTestConfig = {
      seed: 99999,
      duration: 900000, // 15 minutes
      scenarios: CHAOS_SCENARIOS.map((s) => s.id),
      targetWorkers: workerIds,
      metricsInterval: 60000,
      failureThreshold: 0.25, // 25% failure rate
    };

    results.push(await this.runChaosTest(allScenariosConfig));

    return results;
  }

  /**
   * Get test results summary
   */
  getTestResultsSummary(): {
    totalTests: number;
    successfulTests: number;
    failedTests: number;
    averageFailureRate: number;
    averageRecoveryTime: number;
    scenariosTested: number;
  } {
    if (this.testResults.length === 0) {
      return {
        totalTests: 0,
        successfulTests: 0,
        failedTests: 0,
        averageFailureRate: 0,
        averageRecoveryTime: 0,
        scenariosTested: 0,
      };
    }

    const successfulTests = this.testResults.filter((r) => r.success).length;
    const averageFailureRate =
      this.testResults.reduce((sum, r) => sum + r.failureRate, 0) /
      this.testResults.length;
    const averageRecoveryTime =
      this.testResults.reduce((sum, r) => sum + r.recoveryTime, 0) /
      this.testResults.length;
    const scenariosTested = new Set(
      this.testResults.flatMap((r) => r.config.scenarios)
    ).size;

    return {
      totalTests: this.testResults.length,
      successfulTests,
      failedTests: this.testResults.length - successfulTests,
      averageFailureRate,
      averageRecoveryTime,
      scenariosTested,
    };
  }

  /**
   * Setup scenarios for testing
   */
  private setupScenarios(scenarioIds: string[]): void {
    // Add predefined scenarios
    CHAOS_SCENARIOS.forEach((scenario) => {
      if (scenarioIds.includes(scenario.id)) {
        this.harness.addScenario(scenario);
      }
    });
  }

  /**
   * Setup event handlers
   */
  private setupEventHandlers(): void {
    this.harness.on("chaosEvent", (event: ChaosEvent) => {
      console.log(
        `Chaos event triggered: ${event.eventType} - ${event.severity}`
      );
      this.handleChaosEvent(event);
    });

    this.harness.on("recovery", (event: ChaosEvent) => {
      console.log(`Recovery event: ${event.metadata.originalEventId}`);
      this.handleRecoveryEvent(event);
    });
  }

  /**
   * Handle chaos events
   */
  private handleChaosEvent(event: ChaosEvent): void {
    switch (event.eventType) {
      case "failure":
        this.handleWorkerFailure(event);
        break;
      case "network_issue":
        this.handleNetworkIssue(event);
        break;
      case "resource_exhaustion":
        this.handleResourceExhaustion(event);
        break;
      default:
        console.log(`Unhandled chaos event type: ${event.eventType}`);
    }
  }

  /**
   * Handle worker failure events
   */
  private async handleWorkerFailure(event: ChaosEvent): Promise<void> {
    if (!event.workerId) return;

    try {
      // Update worker status in registry
      await this.workerRegistry.updateHealth(event.workerId, "unhealthy", 1.0);
      console.log(
        `Worker ${event.workerId} marked as unhealthy due to chaos event`
      );
    } catch (error) {
      console.error(`Failed to update worker status: ${error}`);
    }
  }

  /**
   * Handle network issue events
   */
  private handleNetworkIssue(event: ChaosEvent): void {
    const affectedWorkers = event.metadata.affectedWorkers as string[];
    if (affectedWorkers) {
      console.log(
        `Network issue affecting workers: ${affectedWorkers.join(", ")}`
      );
      // In a real implementation, this would affect network communication
    }
  }

  /**
   * Handle resource exhaustion events
   */
  private handleResourceExhaustion(event: ChaosEvent): void {
    const resourceType = event.metadata.resourceType as string;
    console.log(`Resource exhaustion: ${resourceType}`);
    // In a real implementation, this would affect resource allocation
  }

  /**
   * Handle recovery events
   */
  private async handleRecoveryEvent(event: ChaosEvent): Promise<void> {
    const originalEventId = event.metadata.originalEventId as string;
    if (event.workerId) {
      try {
        // Restore worker status
        await this.workerRegistry.updateHealth(event.workerId, "healthy", 0.5);
        console.log(
          `Worker ${event.workerId} recovered from chaos event ${originalEventId}`
        );
      } catch (error) {
        console.error(`Failed to restore worker status: ${error}`);
      }
    }
  }

  /**
   * Generate test result
   */
  private generateTestResult(
    testId: string,
    config: ChaosTestConfig,
    startTime: Date,
    endTime: Date
  ): ChaosTestResult {
    const events = this.harness.getActiveEvents();
    const metrics = this.harness.getMetrics();
    const duration = endTime.getTime() - startTime.getTime();

    // Calculate scenario statistics
    const scenarioStats = config.scenarios.map((scenarioId) => {
      const scenarioEvents = events.filter((e) => e.scenarioId === scenarioId);
      const successful = scenarioEvents.filter(
        (e) => e.eventType === "recovery"
      ).length;
      const failed = scenarioEvents.filter(
        (e) => e.eventType === "failure"
      ).length;

      return {
        id: scenarioId,
        triggered: scenarioEvents.length,
        successful,
        failed,
      };
    });

    const failureRate =
      events.length > 0
        ? events.filter((e) => e.eventType === "failure").length / events.length
        : 0;
    const success = failureRate <= config.failureThreshold;

    return {
      testId,
      config,
      startTime,
      endTime,
      duration,
      events,
      metrics,
      success,
      failureRate,
      recoveryTime: metrics.averageRecoveryTime,
      workersAffected: metrics.workersAffected,
      scenarios: scenarioStats,
    };
  }

  /**
   * Log metrics during test
   */
  private logMetrics(testId: string): void {
    const metrics = this.harness.getMetrics();
    console.log(`[${testId}] Metrics:`, {
      activeScenarios: metrics.activeScenarios,
      eventsTriggered: metrics.eventsTriggered,
      workersAffected: metrics.workersAffected,
      failureRate: metrics.failureRate,
    });
  }

  /**
   * Log test result
   */
  private logTestResult(result: ChaosTestResult): void {
    console.log(`\n=== Chaos Test Result: ${result.testId} ===`);
    console.log(`Duration: ${result.duration}ms`);
    console.log(`Success: ${result.success ? "PASS" : "FAIL"}`);
    console.log(`Failure Rate: ${(result.failureRate * 100).toFixed(2)}%`);
    console.log(`Workers Affected: ${result.workersAffected}`);
    console.log(`Average Recovery Time: ${result.recoveryTime}ms`);
    console.log(`Events Triggered: ${result.events.length}`);
    console.log("=====================================\n");
  }

  /**
   * Sleep utility
   */
  private sleep(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }
}


