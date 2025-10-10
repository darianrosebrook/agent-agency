/**
 * @fileoverview Performance Benchmarks for Agent Registry (ARBITER-001)
 *
 * Validates performance SLAs documented in the CAWS specification:
 * - Registry query: <50ms P95
 * - Agent registration: <100ms P95
 * - Performance update: <30ms P95
 * - Concurrent queries: 2000/sec throughput
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import { AgentRegistryManager } from "../src/orchestrator/AgentRegistryManager";
import { AgentProfile, PerformanceMetrics } from "../src/types/agent-registry";

interface BenchmarkResult {
  operation: string;
  samples: number;
  mean: number;
  median: number;
  p95: number;
  p99: number;
  min: number;
  max: number;
  slaTarget: number;
  slaMet: boolean;
}

interface ThroughputResult {
  operation: string;
  duration: number;
  totalOperations: number;
  operationsPerSecond: number;
  slaTarget: number;
  slaMet: boolean;
}

/**
 * Performance benchmark suite
 */
export class AgentRegistryBenchmark {
  private registry: AgentRegistryManager;

  constructor() {
    this.registry = new AgentRegistryManager({
      maxAgents: 10000,
      staleAgentThresholdMs: 3600000,
    });
  }

  /**
   * Run all benchmarks
   */
  async runAll(): Promise<{
    latency: BenchmarkResult[];
    throughput: ThroughputResult[];
    summary: {
      totalTests: number;
      passed: number;
      failed: number;
      overallPass: boolean;
    };
  }> {
    console.log("🚀 Starting ARBITER-001 Performance Benchmarks\n");

    const latencyResults: BenchmarkResult[] = [];
    const throughputResults: ThroughputResult[] = [];

    // Latency benchmarks
    console.log("📊 Latency Benchmarks:");
    latencyResults.push(await this.benchmarkRegistration());
    latencyResults.push(await this.benchmarkQuery());
    latencyResults.push(await this.benchmarkPerformanceUpdate());

    console.log("\n📊 Throughput Benchmarks:");
    throughputResults.push(await this.benchmarkQueryThroughput());

    // Summary
    const totalTests = latencyResults.length + throughputResults.length;
    const passed =
      latencyResults.filter((r) => r.slaMet).length +
      throughputResults.filter((r) => r.slaMet).length;
    const failed = totalTests - passed;

    console.log("\n" + "=".repeat(80));
    console.log("📈 BENCHMARK SUMMARY");
    console.log("=".repeat(80));
    console.log(`Total Tests: ${totalTests}`);
    console.log(`Passed: ${passed}`);
    console.log(`Failed: ${failed}`);
    console.log(
      `Overall: ${failed === 0 ? "✅ ALL SLAS MET" : "❌ SOME SLAS FAILED"}\n`
    );

    return {
      latency: latencyResults,
      throughput: throughputResults,
      summary: {
        totalTests,
        passed,
        failed,
        overallPass: failed === 0,
      },
    };
  }

  /**
   * Benchmark agent registration latency
   */
  private async benchmarkRegistration(): Promise<BenchmarkResult> {
    const samples = 1000;
    const latencies: number[] = [];

    console.log(`  - Agent Registration (${samples} samples)...`);

    for (let i = 0; i < samples; i++) {
      const agent: AgentProfile = {
        id: `benchmark-agent-${i}`,
        name: `Benchmark Agent ${i}`,
        modelFamily: "claude-3.5",
        capabilities: {
          taskTypes: ["code-editing"],
          languages: ["TypeScript"],
          specializations: [],
        },
        performanceHistory: {
          successRate: 0.8,
          averageQuality: 0.75,
          averageLatency: 5000,
          taskCount: 0,
        },
        currentLoad: {
          activeTasks: 0,
          queuedTasks: 0,
          utilizationPercent: 0,
        },
        registeredAt: new Date().toISOString(),
        lastActiveAt: new Date().toISOString(),
      };

      const startTime = Date.now();
      await this.registry.registerAgent(agent);
      const latency = Date.now() - startTime;

      latencies.push(latency);
    }

    return this.analyzeLatencies("Agent Registration", latencies, 100);
  }

  /**
   * Benchmark capability query latency
   */
  private async benchmarkQuery(): Promise<BenchmarkResult> {
    const samples = 1000;
    const latencies: number[] = [];

    console.log(`  - Capability Query (${samples} samples)...`);

    for (let i = 0; i < samples; i++) {
      const startTime = Date.now();
      await this.registry.getAgentsByCapability({
        taskType: "code-editing",
        languages: ["TypeScript"],
        maxUtilization: 80,
      });
      const latency = Date.now() - startTime;

      latencies.push(latency);
    }

    return this.analyzeLatencies("Capability Query", latencies, 50);
  }

  /**
   * Benchmark performance update latency
   */
  private async benchmarkPerformanceUpdate(): Promise<BenchmarkResult> {
    const samples = 1000;
    const latencies: number[] = [];

    console.log(`  - Performance Update (${samples} samples)...`);

    // Use first agent for updates
    const agentId = "benchmark-agent-0";

    for (let i = 0; i < samples; i++) {
      const metrics: PerformanceMetrics = {
        success: Math.random() > 0.2,
        qualityScore: Math.random(),
        latencyMs: Math.random() * 5000,
        tokensUsed: Math.floor(Math.random() * 1000),
        taskType: "code-editing",
      };

      const startTime = Date.now();
      await this.registry.updatePerformance(agentId, metrics);
      const latency = Date.now() - startTime;

      latencies.push(latency);
    }

    return this.analyzeLatencies("Performance Update", latencies, 30);
  }

  /**
   * Benchmark query throughput
   */
  private async benchmarkQueryThroughput(): Promise<ThroughputResult> {
    const duration = 5000; // 5 seconds
    let operations = 0;

    console.log(`  - Query Throughput (${duration / 1000}s test)...`);

    const startTime = Date.now();
    const endTime = startTime + duration;

    while (Date.now() < endTime) {
      await this.registry.getAgentsByCapability({
        taskType: "code-editing",
        maxUtilization: 80,
      });
      operations++;
    }

    const actualDuration = Date.now() - startTime;
    const opsPerSec = (operations / actualDuration) * 1000;
    const slaTarget = 2000;
    const slaMet = opsPerSec >= slaTarget;

    console.log(
      `    ${slaMet ? "✅" : "❌"} ${opsPerSec.toFixed(
        0
      )} ops/sec (target: ${slaTarget} ops/sec)`
    );

    return {
      operation: "Query Throughput",
      duration: actualDuration,
      totalOperations: operations,
      operationsPerSecond: opsPerSec,
      slaTarget,
      slaMet,
    };
  }

  /**
   * Analyze latency distribution
   */
  private analyzeLatencies(
    operation: string,
    latencies: number[],
    slaTarget: number
  ): BenchmarkResult {
    latencies.sort((a, b) => a - b);

    const mean = latencies.reduce((a, b) => a + b, 0) / latencies.length;
    const median = latencies[Math.floor(latencies.length / 2)];
    const p95 = latencies[Math.floor(latencies.length * 0.95)];
    const p99 = latencies[Math.floor(latencies.length * 0.99)];
    const min = latencies[0];
    const max = latencies[latencies.length - 1];
    const slaMet = p95 <= slaTarget;

    console.log(
      `    ${
        slaMet ? "✅" : "❌"
      } ${operation}: P95=${p95}ms (target: ${slaTarget}ms)`
    );
    console.log(
      `       Mean=${mean.toFixed(
        1
      )}ms, Median=${median}ms, P99=${p99}ms, Min=${min}ms, Max=${max}ms`
    );

    return {
      operation,
      samples: latencies.length,
      mean,
      median,
      p95,
      p99,
      min,
      max,
      slaTarget,
      slaMet,
    };
  }
}

/**
 * Run benchmarks if executed directly
 */
async function main() {
  const benchmark = new AgentRegistryBenchmark();

  try {
    const results = await benchmark.runAll();

    if (!results.summary.overallPass) {
      console.error("\n❌ Some benchmarks failed to meet SLA targets");
      process.exit(1);
    } else {
      console.log("\n✅ All benchmarks passed!");
      process.exit(0);
    }
  } catch (error) {
    console.error("Benchmark failed:", error);
    process.exit(1);
  }
}

// Run if executed directly
main();
