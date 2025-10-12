/**
 * Foundation Components Performance Benchmarks
 * 
 * Measures actual performance characteristics of ARBITER-001 through 004
 * to validate performance claims and establish baseline metrics.
 * 
 * Test Categories:
 * - ARBITER-001: Agent Registry operations
 * - ARBITER-002: Task routing decisions
 * - ARBITER-003: CAWS spec validation
 * - ARBITER-004: Performance tracking overhead
 * - System-wide: End-to-end workflows
 * 
 * @author @darianrosebrook
 */

import { describe, it, expect, beforeEach } from "@jest/globals";
import { AgentRegistryManager } from "../../src/orchestrator/AgentRegistryManager";
import { TaskRoutingManager } from "../../src/orchestrator/TaskRoutingManager";
import { SpecValidator } from "../../src/caws-validator/validation/SpecValidator";
import { PerformanceTracker } from "../../src/rl/PerformanceTracker";
import {
  createTestAgent,
  createMinimalTask,
  createMinimalWorkingSpec,
  createMultipleAgents,
  createTaskBatch,
} from "../helpers/test-fixtures";

interface BenchmarkResult {
  operation: string;
  samples: number;
  avgLatencyMs: number;
  p50LatencyMs: number;
  p95LatencyMs: number;
  p99LatencyMs: number;
  minLatencyMs: number;
  maxLatencyMs: number;
  opsPerSecond: number;
}

/**
 * Run a benchmark with multiple samples and collect latency statistics
 */
async function benchmark(
  name: string,
  operation: () => Promise<void>,
  samples: number = 1000
): Promise<BenchmarkResult> {
  const latencies: number[] = [];

  // Warm-up run
  await operation();

  // Collect samples
  for (let i = 0; i < samples; i++) {
    const start = process.hrtime.bigint();
    await operation();
    const end = process.hrtime.bigint();
    const latencyNs = Number(end - start);
    latencies.push(latencyNs / 1_000_000); // Convert to ms
  }

  // Sort for percentile calculations
  latencies.sort((a, b) => a - b);

  // Calculate statistics
  const avg = latencies.reduce((sum, lat) => sum + lat, 0) / latencies.length;
  const p50 = latencies[Math.floor(latencies.length * 0.5)];
  const p95 = latencies[Math.floor(latencies.length * 0.95)];
  const p99 = latencies[Math.floor(latencies.length * 0.99)];
  const min = latencies[0];
  const max = latencies[latencies.length - 1];
  const opsPerSecond = 1000 / avg;

  return {
    operation: name,
    samples,
    avgLatencyMs: avg,
    p50LatencyMs: p50,
    p95LatencyMs: p95,
    p99LatencyMs: p99,
    minLatencyMs: min,
    maxLatencyMs: max,
    opsPerSecond,
  };
}

/**
 * Format benchmark result for console output
 */
function formatResult(result: BenchmarkResult): string {
  return `
📊 ${result.operation}
   Samples: ${result.samples}
   Average: ${result.avgLatencyMs.toFixed(3)}ms
   P50: ${result.p50LatencyMs.toFixed(3)}ms
   P95: ${result.p95LatencyMs.toFixed(3)}ms
   P99: ${result.p99LatencyMs.toFixed(3)}ms
   Min: ${result.minLatencyMs.toFixed(3)}ms
   Max: ${result.maxLatencyMs.toFixed(3)}ms
   Throughput: ${result.opsPerSecond.toFixed(0)} ops/sec
  `.trim();
}

describe("Foundation Performance Benchmarks", () => {
  let registry: AgentRegistryManager;
  let router: TaskRoutingManager;
  let validator: SpecValidator;
  let tracker: PerformanceTracker;

  beforeEach(async () => {
    registry = new AgentRegistryManager({
      maxAgents: 10000,
      enableSecurity: false,
    });
    await registry.initialize();

    tracker = new PerformanceTracker();
    router = new TaskRoutingManager(registry, {}, tracker);
    validator = new SpecValidator();
  });

  describe("ARBITER-001: Agent Registry Performance", () => {
    it("benchmarks agent registration", async () => {
      let counter = 0;
      const result = await benchmark(
        "Agent Registration",
        async () => {
          const agent = createTestAgent({ id: `bench-agent-${counter++}` });
          await registry.registerAgent(agent);
        },
        100 // Fewer samples for registration
      );

      console.log(formatResult(result));

      // Performance assertions
      expect(result.p95LatencyMs).toBeLessThan(10); // P95 < 10ms
      expect(result.avgLatencyMs).toBeLessThan(5); // Average < 5ms
    });

    it("benchmarks agent retrieval by ID", async () => {
      // Pre-populate with 100 agents
      const agents = createMultipleAgents(100);
      for (const agent of agents) {
        await registry.registerAgent(agent);
      }

      const result = await benchmark(
        "Agent Retrieval by ID",
        async () => {
          const randomIdx = Math.floor(Math.random() * 100);
          await registry.getProfile(`test-agent-${randomIdx}`);
        },
        1000
      );

      console.log(formatResult(result));

      // Performance assertions
      expect(result.p95LatencyMs).toBeLessThan(1); // P95 < 1ms (O(1) lookup)
      expect(result.avgLatencyMs).toBeLessThan(0.5); // Average < 0.5ms
    });

    it("benchmarks agent query by capability", async () => {
      // Pre-populate with 100 agents
      const agents = createMultipleAgents(100);
      for (const agent of agents) {
        await registry.registerAgent(agent);
      }

      const result = await benchmark(
        "Agent Query by Capability",
        async () => {
          await registry.getAgentsByCapability({
            taskType: "code-editing",
            languages: ["TypeScript"],
          });
        },
        1000
      );

      console.log(formatResult(result));

      // Performance assertions
      expect(result.p95LatencyMs).toBeLessThan(50); // P95 < 50ms
      expect(result.avgLatencyMs).toBeLessThan(10); // Average < 10ms
    });

    it("benchmarks registry stats calculation", async () => {
      // Pre-populate with 100 agents
      const agents = createMultipleAgents(100);
      for (const agent of agents) {
        await registry.registerAgent(agent);
      }

      const result = await benchmark(
        "Registry Stats Calculation",
        async () => {
          await registry.getStats();
        },
        1000
      );

      console.log(formatResult(result));

      // Performance assertions
      expect(result.p95LatencyMs).toBeLessThan(5); // P95 < 5ms
      expect(result.avgLatencyMs).toBeLessThan(2); // Average < 2ms
    });
  });

  describe("ARBITER-002: Task Routing Performance", () => {
    beforeEach(async () => {
      // Pre-populate with 50 agents for routing tests
      const agents = createMultipleAgents(50);
      for (const agent of agents) {
        await registry.registerAgent(agent);
      }
    });

    it("benchmarks task routing decision", async () => {
      let counter = 0;
      const result = await benchmark(
        "Task Routing Decision",
        async () => {
          const task = createMinimalTask({ id: `bench-task-${counter++}` });
          await router.routeTask(task);
        },
        500
      );

      console.log(formatResult(result));

      // Performance assertions
      expect(result.p95LatencyMs).toBeLessThan(100); // P95 < 100ms (as per config)
      expect(result.avgLatencyMs).toBeLessThan(50); // Average < 50ms
    });

    it("benchmarks concurrent routing (10 tasks)", async () => {
      const tasks = createTaskBatch(10);

      const start = process.hrtime.bigint();
      await Promise.all(tasks.map((task) => router.routeTask(task)));
      const end = process.hrtime.bigint();

      const totalLatencyMs = Number(end - start) / 1_000_000;
      const avgLatencyPerTask = totalLatencyMs / 10;

      console.log(`
📊 Concurrent Routing (10 tasks)
   Total Time: ${totalLatencyMs.toFixed(3)}ms
   Avg per Task: ${avgLatencyPerTask.toFixed(3)}ms
   Throughput: ${(10 / (totalLatencyMs / 1000)).toFixed(0)} tasks/sec
      `.trim());

      // Should handle concurrency efficiently
      expect(totalLatencyMs).toBeLessThan(500); // < 500ms for 10 concurrent
      expect(avgLatencyPerTask).toBeLessThan(50); // < 50ms per task avg
    });
  });

  describe("ARBITER-003: CAWS Validation Performance", () => {
    it("benchmarks working spec validation", async () => {
      const spec = createMinimalWorkingSpec();

      const result = await benchmark(
        "Working Spec Validation",
        async () => {
          await validator.validateWorkingSpec(spec);
        },
        1000
      );

      console.log(formatResult(result));

      // Performance assertions
      expect(result.p95LatencyMs).toBeLessThan(10); // P95 < 10ms
      expect(result.avgLatencyMs).toBeLessThan(5); // Average < 5ms
    });

    it("benchmarks validation with complex spec", async () => {
      const complexSpec = createMinimalWorkingSpec({
        acceptance: Array.from({ length: 10 }, (_, i) => ({
          id: `A${i + 1}`,
          given: `Condition ${i + 1}`,
          when: `Action ${i + 1}`,
          then: `Result ${i + 1}`,
        })),
        invariants: Array.from(
          { length: 5 },
          (_, i) => `Invariant ${i + 1}`
        ),
      });

      const result = await benchmark(
        "Complex Spec Validation (10 acceptance, 5 invariants)",
        async () => {
          await validator.validateWorkingSpec(complexSpec);
        },
        1000
      );

      console.log(formatResult(result));

      // Performance assertions
      expect(result.p95LatencyMs).toBeLessThan(20); // P95 < 20ms
      expect(result.avgLatencyMs).toBeLessThan(10); // Average < 10ms
    });
  });

  describe("ARBITER-004: Performance Tracking Overhead", () => {
    it("benchmarks performance tracking data storage", async () => {
      // Note: Performance tracking is primarily in-memory and very fast
      // This test verifies minimal overhead
      const agent = await registry.registerAgent(createTestAgent({ id: "tracking-agent" }));
      
      const result = await benchmark(
        "Performance Update Recording",
        async () => {
          await registry.updatePerformance(agent.id, {
            success: true,
            qualityScore: 0.85,
            latencyMs: 100,
          });
        },
        1000
      );

      console.log(formatResult(result));

      // Performance assertions (should be very fast)
      expect(result.p95LatencyMs).toBeLessThan(5); // P95 < 5ms
      expect(result.avgLatencyMs).toBeLessThan(2); // Average < 2ms
    });
  });

  describe("System-Wide Performance", () => {
    it("benchmarks end-to-end workflow", async () => {
      // Register agents
      const agents = createMultipleAgents(10);
      for (const agent of agents) {
        await registry.registerAgent(agent);
      }

      let taskCounter = 0;
      const result = await benchmark(
        "End-to-End Workflow (validate → route → track)",
        async () => {
          // 1. Validate spec
          const spec = createMinimalWorkingSpec();
          await validator.validateWorkingSpec(spec);

          // 2. Route task
          const task = createMinimalTask({ id: `e2e-task-${taskCounter++}` });
          await router.routeTask(task);

          // Note: Tracking is measured separately
        },
        200
      );

      console.log(formatResult(result));

      // Performance assertions
      expect(result.p95LatencyMs).toBeLessThan(150); // P95 < 150ms
      expect(result.avgLatencyMs).toBeLessThan(75); // Average < 75ms
    });

    it("benchmarks high-throughput scenario (100 tasks/sec)", async () => {
      // Register agents
      const agents = createMultipleAgents(20);
      for (const agent of agents) {
        await registry.registerAgent(agent);
      }

      // Simulate 100 tasks over 1 second
      const tasks = createTaskBatch(100);
      const startTime = Date.now();

      const results = await Promise.all(
        tasks.map((task) => router.routeTask(task))
      );

      const duration = Date.now() - startTime;
      const throughput = (results.length / duration) * 1000;

      console.log(`
📊 High-Throughput Test
   Tasks: ${results.length}
   Duration: ${duration}ms
   Throughput: ${throughput.toFixed(0)} tasks/sec
   Avg Latency: ${(duration / results.length).toFixed(2)}ms per task
      `.trim());

      // Should handle high throughput
      expect(duration).toBeLessThan(5000); // < 5 seconds for 100 tasks
      expect(throughput).toBeGreaterThan(20); // > 20 tasks/sec
    });
  });

  describe("Memory and Resource Usage", () => {
    it("measures memory usage under load", async () => {
      const initialMemory = process.memoryUsage();

      // Register 1000 agents
      const agents = createMultipleAgents(1000);
      for (const agent of agents) {
        await registry.registerAgent(agent);
      }

      // Route 1000 tasks
      const tasks = createTaskBatch(1000);
      await Promise.all(tasks.map((task) => router.routeTask(task)));

      const finalMemory = process.memoryUsage();

      const heapUsedMB = (finalMemory.heapUsed - initialMemory.heapUsed) / 1024 / 1024;
      const heapTotalMB = (finalMemory.heapTotal - initialMemory.heapTotal) / 1024 / 1024;

      console.log(`
📊 Memory Usage (1000 agents, 1000 tasks)
   Heap Used: ${heapUsedMB.toFixed(2)}MB
   Heap Total: ${heapTotalMB.toFixed(2)}MB
   RSS: ${((finalMemory.rss - initialMemory.rss) / 1024 / 1024).toFixed(2)}MB
      `.trim());

      // Memory should be reasonable (adjust based on actual usage)
      expect(heapUsedMB).toBeLessThan(600); // < 600MB heap growth for 1000 agents + 1000 tasks
    });
  });
});

