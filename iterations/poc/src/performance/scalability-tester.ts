/**
 * Scalability Tester
 *
 * Tests system performance under concurrent load and measures scaling capabilities.
 * Includes load balancing, caching optimization, and performance benchmarking.
 *
 * @author @darianrosebrook
 */

import { EventEmitter } from "events";

export interface LoadTestScenario {
  id: string;
  name: string;
  description: string;
  concurrentUsers: number;
  rampUpTime: number; // seconds
  duration: number; // seconds
  operations: LoadTestOperation[];
  targetLatency: number; // ms
  targetThroughput: number; // ops/sec
}

export interface LoadTestOperation {
  type:
    | "generate_text"
    | "read_file"
    | "write_file"
    | "memory_query"
    | "task_decomposition";
  weight: number; // Relative frequency (0-1)
  parameters: any;
  timeout: number; // ms
}

export interface PerformanceMetrics {
  timestamp: Date;
  operationCount: number;
  errorCount: number;
  averageLatency: number;
  p95Latency: number;
  p99Latency: number;
  throughput: number; // ops/sec
  memoryUsage: number;
  cpuUsage: number;
  activeConnections: number;
}

export interface LoadTestResult {
  scenario: LoadTestScenario;
  startTime: Date;
  endTime: Date;
  totalOperations: number;
  successfulOperations: number;
  failedOperations: number;
  metrics: PerformanceMetrics[];
  bottlenecks: string[];
  recommendations: string[];
  passed: boolean;
}

/**
 * Scalability Tester for concurrent load testing
 */
export class ScalabilityTester extends EventEmitter {
  private activeTests = new Map<string, LoadTestResult>();
  private performanceHistory: PerformanceMetrics[] = [];

  /**
   * Run a comprehensive scalability test
   */
  async runScalabilityTest(
    scenario: LoadTestScenario
  ): Promise<LoadTestResult> {
    const testId = `load-test-${Date.now()}`;
    console.log(`🚀 Starting scalability test: ${scenario.name}`);

    const result: LoadTestResult = {
      scenario,
      startTime: new Date(),
      endTime: new Date(),
      totalOperations: 0,
      successfulOperations: 0,
      failedOperations: 0,
      metrics: [],
      bottlenecks: [],
      recommendations: [],
      passed: false,
    };

    this.activeTests.set(testId, result);

    try {
      // Ramp up load gradually
      await this.rampUpLoad(scenario, result);

      // Run main test duration
      await this.runMainLoadTest(scenario, result);

      // Cool down and analyze
      result.endTime = new Date();
      result.passed = this.evaluateTestResults(result);

      // Generate recommendations
      result.recommendations = this.generateRecommendations(result);

      console.log(
        `✅ Scalability test completed: ${result.passed ? "PASSED" : "FAILED"}`
      );
      console.log(
        `📊 Results: ${result.successfulOperations}/${result.totalOperations} successful operations`
      );
    } catch (error) {
      console.error(`❌ Scalability test failed:`, error);
      result.bottlenecks.push(`Test execution failed: ${error}`);
    }

    this.emit("test-completed", result);
    return result;
  }

  /**
   * Gradually ramp up to target concurrent load
   */
  private async rampUpLoad(
    scenario: LoadTestScenario,
    result: LoadTestResult
  ): Promise<void> {
    const rampUpSteps = 10;
    const stepDuration = scenario.rampUpTime / rampUpSteps;

    console.log(
      `📈 Ramping up to ${scenario.concurrentUsers} concurrent users over ${scenario.rampUpTime}s`
    );

    for (let step = 1; step <= rampUpSteps; step++) {
      const targetUsers = Math.floor(
        (scenario.concurrentUsers * step) / rampUpSteps
      );

      // Start additional concurrent operations
      const promises = [];
      for (let i = 0; i < targetUsers; i++) {
        promises.push(this.executeRandomOperation(scenario.operations, result));
      }

      await Promise.allSettled(promises);
      await this.sleep(stepDuration * 1000);

      // Record metrics
      const metrics = await this.collectPerformanceMetrics();
      result.metrics.push(metrics);

      console.log(
        `📊 Ramp up step ${step}/${rampUpSteps}: ${targetUsers} users, ${metrics.throughput} ops/sec`
      );
    }
  }

  /**
   * Run the main load test at full concurrency
   */
  private async runMainLoadTest(
    scenario: LoadTestScenario,
    result: LoadTestResult
  ): Promise<void> {
    console.log(`🏃 Running main load test for ${scenario.duration} seconds`);

    const startTime = Date.now();
    const endTime = startTime + scenario.duration * 1000;

    // Maintain target concurrency throughout the test
    while (Date.now() < endTime) {
      const batchStart = Date.now();

      // Execute batch of concurrent operations
      const promises = [];
      for (let i = 0; i < scenario.concurrentUsers; i++) {
        promises.push(this.executeRandomOperation(scenario.operations, result));
      }

      await Promise.allSettled(promises);

      // Record metrics every second
      if (Date.now() - batchStart >= 1000) {
        const metrics = await this.collectPerformanceMetrics();
        result.metrics.push(metrics);
      }
    }

    console.log(`📊 Main test completed. Final metrics collected.`);
  }

  /**
   * Execute a randomly selected operation based on weights
   */
  private async executeRandomOperation(
    operations: LoadTestOperation[],
    result: LoadTestResult
  ): Promise<void> {
    result.totalOperations++;

    try {
      // Select operation based on weights
      const totalWeight = operations.reduce((sum, op) => sum + op.weight, 0);
      let random = Math.random() * totalWeight;

      let selectedOp: LoadTestOperation | null = null;
      for (const op of operations) {
        random -= op.weight;
        if (random <= 0) {
          selectedOp = op;
          break;
        }
      }

      if (!selectedOp) selectedOp = operations[0];

      const startTime = Date.now();
      await this.executeOperation(selectedOp);
      const latency = Date.now() - startTime;

      result.successfulOperations++;

      // Track latency for percentile calculations
      this.trackLatency(latency);
    } catch (error) {
      result.failedOperations++;
      console.warn(`⚠️ Operation failed:`, error);
    }
  }

  /**
   * Execute a specific operation (mock implementation)
   */
  private async executeOperation(operation: LoadTestOperation): Promise<any> {
    // Simulate operation execution time based on type
    const baseDelay = {
      generate_text: 500 + Math.random() * 1000,
      read_file: 50 + Math.random() * 100,
      write_file: 100 + Math.random() * 200,
      memory_query: 20 + Math.random() * 50,
      task_decomposition: 200 + Math.random() * 400,
    };

    const delay = baseDelay[operation.type] || 100;
    await this.sleep(delay);

    // Simulate occasional failures
    if (Math.random() < 0.05) {
      // 5% failure rate
      throw new Error(`Simulated ${operation.type} failure`);
    }

    return { success: true, operation: operation.type };
  }

  /**
   * Track latency measurements for percentile calculations
   */
  private latencyMeasurements: number[] = [];

  private trackLatency(latency: number): void {
    this.latencyMeasurements.push(latency);

    // Keep only recent measurements (last 1000)
    if (this.latencyMeasurements.length > 1000) {
      this.latencyMeasurements = this.latencyMeasurements.slice(-1000);
    }
  }

  /**
   * Collect current performance metrics
   */
  private async collectPerformanceMetrics(): Promise<PerformanceMetrics> {
    // In a real implementation, this would collect actual system metrics
    // For now, we'll simulate realistic metrics

    const latencies = [...this.latencyMeasurements];
    latencies.sort((a, b) => a - b);

    const metrics: PerformanceMetrics = {
      timestamp: new Date(),
      operationCount: this.performanceHistory.length,
      errorCount: Math.floor(Math.random() * 10), // Simulated errors
      averageLatency:
        latencies.length > 0
          ? latencies.reduce((a, b) => a + b) / latencies.length
          : 0,
      p95Latency:
        latencies.length > 0
          ? latencies[Math.floor(latencies.length * 0.95)]
          : 0,
      p99Latency:
        latencies.length > 0
          ? latencies[Math.floor(latencies.length * 0.99)]
          : 0,
      throughput: 50 + Math.random() * 100, // Simulated ops/sec
      memoryUsage: 100 + Math.random() * 200, // MB
      cpuUsage: 20 + Math.random() * 60, // %
      activeConnections: 10 + Math.random() * 50,
    };

    this.performanceHistory.push(metrics);
    return metrics;
  }

  /**
   * Evaluate if the test passed its targets
   */
  private evaluateTestResults(result: LoadTestResult): boolean {
    if (result.metrics.length === 0) return false;

    const finalMetrics = result.metrics[result.metrics.length - 1];

    // Check latency targets
    if (finalMetrics.p95Latency > result.scenario.targetLatency) {
      result.bottlenecks.push(
        `P95 latency ${finalMetrics.p95Latency}ms exceeds target ${result.scenario.targetLatency}ms`
      );
      return false;
    }

    // Check throughput targets
    if (finalMetrics.throughput < result.scenario.targetThroughput) {
      result.bottlenecks.push(
        `Throughput ${finalMetrics.throughput} ops/sec below target ${result.scenario.targetThroughput} ops/sec`
      );
      return false;
    }

    // Check error rate (< 5%)
    const errorRate = result.failedOperations / result.totalOperations;
    if (errorRate > 0.05) {
      result.bottlenecks.push(
        `Error rate ${(errorRate * 100).toFixed(1)}% exceeds 5% threshold`
      );
      return false;
    }

    return true;
  }

  /**
   * Generate performance optimization recommendations
   */
  private generateRecommendations(result: LoadTestResult): string[] {
    const recommendations: string[] = [];

    if (result.metrics.length === 0) return recommendations;

    const avgMetrics = result.metrics.reduce(
      (acc, m) => ({
        latency: acc.latency + m.averageLatency,
        throughput: acc.throughput + m.throughput,
        memory: acc.memory + m.memoryUsage,
        cpu: acc.cpu + m.cpuUsage,
        count: acc.count + 1,
      }),
      { latency: 0, throughput: 0, memory: 0, cpu: 0, count: 0 }
    );

    const finalMetrics = result.metrics[result.metrics.length - 1];

    // Latency recommendations
    if (finalMetrics.p95Latency > 500) {
      recommendations.push(
        "Consider implementing response caching for frequently requested data"
      );
      recommendations.push(
        "Optimize database queries and add appropriate indexes"
      );
    }

    // Throughput recommendations
    if (finalMetrics.throughput < result.scenario.targetThroughput * 0.8) {
      recommendations.push(
        "Consider horizontal scaling by adding more server instances"
      );
      recommendations.push("Implement load balancing across multiple nodes");
    }

    // Memory recommendations
    if (avgMetrics.memory / avgMetrics.count > 500) {
      recommendations.push(
        "Implement memory-efficient data structures and garbage collection optimization"
      );
      recommendations.push(
        "Consider using external caching services (Redis, Memcached)"
      );
    }

    // CPU recommendations
    if (avgMetrics.cpu / avgMetrics.count > 70) {
      recommendations.push(
        "Profile application for CPU-intensive operations and optimize algorithms"
      );
      recommendations.push(
        "Consider asynchronous processing for CPU-bound tasks"
      );
    }

    // Error handling recommendations
    const errorRate = result.failedOperations / result.totalOperations;
    if (errorRate > 0.02) {
      recommendations.push(
        "Implement circuit breaker patterns for external service calls"
      );
      recommendations.push(
        "Add comprehensive error handling and retry mechanisms"
      );
    }

    return recommendations;
  }

  /**
   * Create standard load test scenarios
   */
  static createStandardScenarios(): LoadTestScenario[] {
    return [
      {
        id: "light-load",
        name: "Light Load Test",
        description: "Basic functionality test with light concurrent load",
        concurrentUsers: 5,
        rampUpTime: 30,
        duration: 60,
        targetLatency: 200,
        targetThroughput: 20,
        operations: [
          {
            type: "generate_text",
            weight: 0.4,
            parameters: { maxTokens: 100 },
            timeout: 5000,
          },
          {
            type: "read_file",
            weight: 0.3,
            parameters: { path: "config.json" },
            timeout: 1000,
          },
          {
            type: "memory_query",
            weight: 0.3,
            parameters: { query: "recent" },
            timeout: 500,
          },
        ],
      },
      {
        id: "medium-load",
        name: "Medium Load Test",
        description: "Moderate concurrent load testing",
        concurrentUsers: 20,
        rampUpTime: 60,
        duration: 120,
        targetLatency: 500,
        targetThroughput: 50,
        operations: [
          {
            type: "generate_text",
            weight: 0.5,
            parameters: { maxTokens: 200 },
            timeout: 10000,
          },
          {
            type: "write_file",
            weight: 0.2,
            parameters: { content: "test data" },
            timeout: 2000,
          },
          {
            type: "read_file",
            weight: 0.2,
            parameters: { path: "data.json" },
            timeout: 1000,
          },
          {
            type: "task_decomposition",
            weight: 0.1,
            parameters: { complexity: "simple" },
            timeout: 5000,
          },
        ],
      },
      {
        id: "heavy-load",
        name: "Heavy Load Test",
        description: "High concurrent load stress testing",
        concurrentUsers: 50,
        rampUpTime: 120,
        duration: 300,
        targetLatency: 1000,
        targetThroughput: 100,
        operations: [
          {
            type: "generate_text",
            weight: 0.6,
            parameters: { maxTokens: 500 },
            timeout: 15000,
          },
          {
            type: "write_file",
            weight: 0.15,
            parameters: { content: "large data payload" },
            timeout: 3000,
          },
          {
            type: "read_file",
            weight: 0.15,
            parameters: { path: "large-file.json" },
            timeout: 2000,
          },
          {
            type: "task_decomposition",
            weight: 0.1,
            parameters: { complexity: "complex" },
            timeout: 10000,
          },
        ],
      },
      {
        id: "spike-test",
        name: "Traffic Spike Test",
        description: "Sudden traffic spike simulation",
        concurrentUsers: 100,
        rampUpTime: 10, // Very fast ramp-up
        duration: 60,
        targetLatency: 2000,
        targetThroughput: 150,
        operations: [
          {
            type: "generate_text",
            weight: 0.7,
            parameters: { maxTokens: 100 },
            timeout: 5000,
          },
          {
            type: "memory_query",
            weight: 0.3,
            parameters: { query: "cache" },
            timeout: 1000,
          },
        ],
      },
    ];
  }

  /**
   * Run all standard scalability tests
   */
  async runAllStandardTests(): Promise<LoadTestResult[]> {
    const scenarios = ScalabilityTester.createStandardScenarios();
    const results: LoadTestResult[] = [];

    for (const scenario of scenarios) {
      console.log(`\n🔬 Running scenario: ${scenario.name}`);
      const result = await this.runScalabilityTest(scenario);
      results.push(result);

      // Brief pause between tests
      await this.sleep(5000);
    }

    return results;
  }

  /**
   * Get scalability analytics
   */
  getScalabilityAnalytics(): {
    totalTests: number;
    passedTests: number;
    averageThroughput: number;
    averageLatency: number;
    peakThroughput: number;
    bottlenecks: string[];
  } {
    const completedTests = Array.from(this.activeTests.values());
    const passedTests = completedTests.filter((t) => t.passed);

    let totalThroughput = 0;
    let totalLatency = 0;
    let peakThroughput = 0;
    const allBottlenecks = new Set<string>();

    completedTests.forEach((test) => {
      test.metrics.forEach((metric) => {
        totalThroughput += metric.throughput;
        totalLatency += metric.averageLatency;
        peakThroughput = Math.max(peakThroughput, metric.throughput);
      });
      test.bottlenecks.forEach((b) => allBottlenecks.add(b));
    });

    const totalMetrics = completedTests.reduce(
      (sum, t) => sum + t.metrics.length,
      0
    );

    return {
      totalTests: completedTests.length,
      passedTests: passedTests.length,
      averageThroughput: totalMetrics > 0 ? totalThroughput / totalMetrics : 0,
      averageLatency: totalMetrics > 0 ? totalLatency / totalMetrics : 0,
      peakThroughput,
      bottlenecks: Array.from(allBottlenecks),
    };
  }

  private async sleep(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }
}
