/**
 * @fileoverview Performance Baselines and Load Tests - ARBITER-028
 *
 * Establishes performance baselines and conducts load tests with 100+ concurrent tasks
 * to validate the arbiter's resilience under high load conditions as defined in the
 * Arbiter Edge Case Capability Plan.
 *
 * @author @darianrosebrook
 */

import {
  afterAll,
  beforeAll,
  beforeEach,
  describe,
  expect,
  it,
} from "@jest/globals";
import { Pool } from "pg";
import { TaskOrchestrator } from "../../src/orchestrator/TaskOrchestrator";
import { TaskRoutingManager } from "../../src/orchestrator/TaskRoutingManager";
import { ArbitrationBoardCoordinator } from "../../src/orchestrator/arbitration/ArbitrationBoardCoordinator";
import { ConfidenceScorer } from "../../src/orchestrator/arbitration/ConfidenceScorer";
import { TaskIntakeProcessor } from "../../src/orchestrator/intake/TaskIntakeProcessor";
import { PostgreSQLCreditLedgerRepositoryImpl } from "../../src/orchestrator/repositories/implementations/PostgreSQLCreditLedgerRepository";
import { PostgreSQLTaskSnapshotRepositoryImpl } from "../../src/orchestrator/repositories/implementations/PostgreSQLTaskSnapshotRepository";
import { PostgreSQLWorkerCapabilityRepositoryImpl } from "../../src/orchestrator/repositories/implementations/PostgreSQLWorkerCapabilityRepository";
import { WorkerCapabilityRegistryImpl } from "../../src/orchestrator/resources/WorkerCapabilityRegistry";
import { TaskSnapshotStoreImpl } from "../../src/orchestrator/state/TaskSnapshotStore";
import { ChaosTestingHarness } from "../../src/testing/ChaosTestingHarness";
import { TaskInput } from "../../src/types/arbiter-orchestration";
import { VerificationType } from "../../src/types/verification";

// Additional imports for missing types
import { CreditLedger } from "../../src/orchestrator/credit/CreditLedger";
import { AdaptivePolicyEngineImpl } from "../../src/orchestrator/policy/AdaptivePolicyEngine";
import { PolicyAuditManagerImpl } from "../../src/orchestrator/policy/PolicyAuditManager";
import { CAWSPolicyEnforcer } from "../../src/orchestrator/security/CAWSPolicyEnforcer";
import { VerificationEngineImpl } from "../../src/verification/VerificationEngine";

// Performance baselines and thresholds
const PERFORMANCE_BASELINES = {
  // Task processing times (milliseconds)
  SIMPLE_TASK_MAX_TIME: 5000, // 5 seconds
  MEDIUM_TASK_MAX_TIME: 15000, // 15 seconds
  COMPLEX_TASK_MAX_TIME: 60000, // 60 seconds

  // Throughput requirements
  MIN_TASKS_PER_SECOND: 2, // At least 2 tasks/second
  CONCURRENT_TASK_LIMIT: 100, // Support 100 concurrent tasks

  // Resource utilization limits
  MAX_MEMORY_USAGE_MB: 512, // 512MB max memory
  MAX_CPU_USAGE_PERCENT: 80, // 80% max CPU

  // Error rates
  MAX_ERROR_RATE_PERCENT: 5, // 5% max error rate
  MAX_TIMEOUT_RATE_PERCENT: 10, // 10% max timeout rate

  // Database performance
  MAX_DB_QUERY_TIME: 1000, // 1 second max DB query
  MAX_DB_CONNECTION_POOL: 20, // Max 20 DB connections
};

interface PerformanceMetrics {
  totalTasks: number;
  completedTasks: number;
  failedTasks: number;
  timeoutTasks: number;
  averageProcessingTime: number;
  minProcessingTime: number;
  maxProcessingTime: number;
  p95ProcessingTime: number;
  p99ProcessingTime: number;
  throughputTasksPerSecond: number;
  errorRate: number;
  timeoutRate: number;
  memoryUsageMB: number;
  cpuUsagePercent: number;
  dbQueryCount: number;
  averageDbQueryTime: number;
}

// Test infrastructure
let dbPool: Pool;
let orchestrator: TaskOrchestrator;
let workerRegistry: WorkerCapabilityRegistryImpl;
let snapshotStore: TaskSnapshotStoreImpl;
let chaosHarness: ChaosTestingHarness;
const performanceMetrics: PerformanceMetrics[] = [];

describe("Arbiter Edge Case Performance Tests", () => {
  beforeAll(async () => {
    // Initialize test database with performance-optimized settings
    dbPool = new Pool({
      connectionString:
        process.env.TEST_DATABASE_URL ||
        "postgresql://localhost:5432/arbiter_test",
      max: PERFORMANCE_BASELINES.MAX_DB_CONNECTION_POOL,
      idleTimeoutMillis: 30000,
      connectionTimeoutMillis: 2000,
      statement_timeout: PERFORMANCE_BASELINES.MAX_DB_QUERY_TIME,
    });

    // Initialize components with performance-optimized configurations
    const workerRepo = new PostgreSQLWorkerCapabilityRepositoryImpl(dbPool);
    const snapshotRepo = new PostgreSQLTaskSnapshotRepositoryImpl(dbPool);
    const creditRepo = new PostgreSQLCreditLedgerRepositoryImpl(dbPool);

    workerRegistry = new WorkerCapabilityRegistryImpl(workerRepo, {
      cleanupIntervalMs: 60000, // Less frequent cleanup for performance
      defaultStaleThresholdMs: 120000,
    });

    snapshotStore = new TaskSnapshotStoreImpl(snapshotRepo, {
      defaultTtlMs: 1800000, // 30 minutes
      cleanupIntervalMs: 600000, // 10 minutes
      maxSnapshotsPerTask: 5, // Reduced for performance
    });

    const creditLedger = new CreditLedger(creditRepo);
    const _confidenceScorer = new ConfidenceScorer();
    const adaptiveEngine = new AdaptivePolicyEngineImpl(creditLedger, {});
    const policyEnforcer = new CAWSPolicyEnforcer();
    const auditManager = new PolicyAuditManagerImpl();

    const intakeProcessor = new TaskIntakeProcessor({
      maxPayloadSize: 2 * 1024 * 1024, // 2MB for performance tests
      enableStreamingParser: true,
      validationRules: ["schema", "security"],
      rejectionThreshold: 0.9, // Higher threshold for performance
    });

    const routingManager = new TaskRoutingManager({
      strategy: "least-loaded",
      loadBalancing: "round-robin",
      failoverEnabled: true,
      timeoutMs: 45000, // Longer timeout for performance tests
    });

    const verificationEngine = new VerificationEngineImpl({
      adapters: ["math", "code", "context"],
      timeoutMs: 90000,
      enableSandboxing: true,
      maxConcurrentVerifications: 20, // Increased for performance
    });

    const arbitrationBoard = new ArbitrationBoardCoordinator({
      confidenceThreshold: 0.6, // Lower threshold for performance
      maxParticipants: 3, // Reduced for performance
      timeoutMs: 60000,
      enableAppeals: false, // Disabled for performance
    });

    orchestrator = new TaskOrchestrator({
      intakeProcessor,
      routingManager,
      verificationEngine,
      arbitrationBoard,
      workerRegistry,
      snapshotStore,
      creditLedger,
      policyEnforcer,
      auditManager,
      adaptiveEngine,
      maxConcurrentTasks: PERFORMANCE_BASELINES.CONCURRENT_TASK_LIMIT,
      defaultTimeoutMs: 120000, // 2 minutes
    });

    chaosHarness = new ChaosTestingHarness(12345);

    await orchestrator.initialize();
  });

  afterAll(async () => {
    await orchestrator.shutdown();
    await dbPool.end();
  });

  beforeEach(async () => {
    // Clean up test data
    await dbPool.query("DELETE FROM credit_ledger");
    await dbPool.query("DELETE FROM task_snapshots");
    await dbPool.query("DELETE FROM worker_capabilities");
    await dbPool.query("DELETE FROM task_memory");

    // Register baseline workers for each test
    await registerBaselineWorkers();
  });

  async function registerBaselineWorkers(): Promise<void> {
    const workerTypes = [
      { id: "math-worker", capabilities: { math: true, computation: true } },
      { id: "code-worker", capabilities: { code: true, programming: true } },
      {
        id: "text-worker",
        capabilities: { text_processing: true, analysis: true },
      },
      {
        id: "data-worker",
        capabilities: { data_analysis: true, statistics: true },
      },
      {
        id: "verification-worker",
        capabilities: { verification: true, fact_checking: true },
      },
    ];

    for (const worker of workerTypes) {
      await workerRegistry.register({
        workerId: worker.id,
        capabilities: worker.capabilities,
        healthStatus: "healthy",
        saturationRatio: 0.1,
      });
    }
  }

  function createTaskInput(
    type: "simple" | "medium" | "complex",
    id: number
  ): TaskInput {
    const baseTask = {
      taskId: id,
      metadata: {
        contentType: "application/json" as const,
        encoding: "utf8" as const,
        priorityHint: "normal" as const,
        surface: "performance_test" as const,
      },
    };

    switch (type) {
      case "simple":
        return {
          payload: {
            type: "simple_computation",
            operation: `${id} + ${id}`,
            expectedResult: id * 2,
          },
          ...baseTask,
        };

      case "medium":
        return {
          payload: {
            type: "medium_computation",
            problem: `Calculate fibonacci(${id % 20})`,
            verificationType: VerificationType.MATH_VERIFICATION,
          },
          ...baseTask,
        };

      case "complex":
        return {
          payload: {
            type: "complex_analysis",
            dataset: Array(100)
              .fill(0)
              .map((_, i) => ({
                id: i,
                value: Math.random() * 1000,
                category: `category_${i % 10}`,
              })),
            analysisType: "statistical_summary",
            verificationType: VerificationType.CODE_VERIFICATION,
          },
          ...baseTask,
        };

      default:
        throw new Error(`Unknown task type: ${type}`);
    }
  }

  async function measurePerformance(
    tasks: TaskInput[],
    testName: string,
    enableChaos: boolean = false
  ): Promise<PerformanceMetrics> {
    const startTime = Date.now();
    const startMemory = process.memoryUsage();
    let dbQueryCount = 0;
    const dbQueryTimes: number[] = [];

    // Monitor database queries
    const originalQuery = dbPool.query.bind(dbPool);
    dbPool.query = function (...args) {
      const queryStart = Date.now();
      dbQueryCount++;
      return originalQuery(...args).then((result: any) => {
        dbQueryTimes.push(Date.now() - queryStart);
        return result;
      });
    };

    if (enableChaos) {
      chaosHarness.enable();
      chaosHarness.addScenario({
        id: "PERFORMANCE_CHAOS",
        name: "Performance Test Chaos",
        description: "Adds controlled chaos during performance testing",
        probability: 0.15, // 15% chance of chaos
        duration: 3000,
      });
    }

    const results = await Promise.allSettled(
      tasks.map((task) => orchestrator.submitTask(task))
    );

    if (enableChaos) {
      chaosHarness.disable();
    }

    const endTime = Date.now();
    const endMemory = process.memoryUsage();

    // Restore original query method
    dbPool.query = originalQuery;

    // Analyze results
    const completedTasks = results.filter(
      (r) => r.status === "fulfilled" && r.value.status === "completed"
    ).length;

    const failedTasks = results.filter(
      (r) => r.status === "fulfilled" && r.value.status === "failed"
    ).length;

    const timeoutTasks = results.filter(
      (r) => r.status === "fulfilled" && r.value.status === "timeout"
    ).length;

    const processingTimes = results
      .filter(
        (r) =>
          r.status === "fulfilled" && r.value.completedAt && r.value.startedAt
      )
      .map((r) => {
        const result = r.value;
        return result.completedAt!.getTime() - result.startedAt!.getTime();
      });

    const totalTime = endTime - startTime;
    const totalTasks = tasks.length;
    const throughputTasksPerSecond = totalTasks / (totalTime / 1000);
    const errorRate = ((failedTasks + timeoutTasks) / totalTasks) * 100;

    const metrics: PerformanceMetrics = {
      totalTasks,
      completedTasks,
      failedTasks,
      timeoutTasks,
      averageProcessingTime:
        processingTimes.length > 0
          ? processingTimes.reduce((a, b) => a + b, 0) / processingTimes.length
          : 0,
      minProcessingTime:
        processingTimes.length > 0 ? Math.min(...processingTimes) : 0,
      maxProcessingTime:
        processingTimes.length > 0 ? Math.max(...processingTimes) : 0,
      p95ProcessingTime:
        processingTimes.length > 0
          ? processingTimes.sort((a, b) => a - b)[
              Math.floor(processingTimes.length * 0.95)
            ]
          : 0,
      p99ProcessingTime:
        processingTimes.length > 0
          ? processingTimes.sort((a, b) => a - b)[
              Math.floor(processingTimes.length * 0.99)
            ]
          : 0,
      throughputTasksPerSecond: throughputTasksPerSecond,
      errorRate,
      timeoutRate: (timeoutTasks / totalTasks) * 100,
      memoryUsageMB: (endMemory.heapUsed - startMemory.heapUsed) / 1024 / 1024,
      cpuUsagePercent: 0, // Would need external monitoring for accurate CPU usage
      dbQueryCount,
      averageDbQueryTime:
        dbQueryTimes.length > 0
          ? dbQueryTimes.reduce((a, b) => a + b, 0) / dbQueryTimes.length
          : 0,
    };

    performanceMetrics.push(metrics);

    console.log(`\n=== Performance Results: ${testName} ===`);
    console.log(`Total Tasks: ${metrics.totalTasks}`);
    console.log(
      `Completed: ${metrics.completedTasks} (${(
        (metrics.completedTasks / metrics.totalTasks) *
        100
      ).toFixed(1)}%)`
    );
    console.log(
      `Failed: ${metrics.failedTasks} (${metrics.errorRate.toFixed(1)}%)`
    );
    console.log(
      `Timeouts: ${metrics.timeoutTasks} (${metrics.timeoutRate.toFixed(1)}%)`
    );
    console.log(
      `Average Processing Time: ${metrics.averageProcessingTime.toFixed(0)}ms`
    );
    console.log(
      `P95 Processing Time: ${metrics.p95ProcessingTime.toFixed(0)}ms`
    );
    console.log(
      `Throughput: ${metrics.throughputTasksPerSecond.toFixed(2)} tasks/sec`
    );
    console.log(`Memory Usage: ${metrics.memoryUsageMB.toFixed(1)}MB`);
    console.log(
      `DB Queries: ${
        metrics.dbQueryCount
      } (avg: ${metrics.averageDbQueryTime.toFixed(0)}ms)`
    );

    return metrics;
  }

  describe("Performance Baselines", () => {
    it("should meet baseline performance for simple tasks", async () => {
      const tasks = Array(20)
        .fill(0)
        .map((_, i) => createTaskInput("simple", i));
      const metrics = await measurePerformance(tasks, "Simple Tasks Baseline");

      expect(metrics.averageProcessingTime).toBeLessThan(
        PERFORMANCE_BASELINES.SIMPLE_TASK_MAX_TIME
      );
      expect(metrics.throughputTasksPerSecond).toBeGreaterThanOrEqual(
        PERFORMANCE_BASELINES.MIN_TASKS_PER_SECOND
      );
      expect(metrics.errorRate).toBeLessThan(
        PERFORMANCE_BASELINES.MAX_ERROR_RATE_PERCENT
      );
      expect(metrics.memoryUsageMB).toBeLessThan(
        PERFORMANCE_BASELINES.MAX_MEMORY_USAGE_MB
      );
    });

    it("should meet baseline performance for medium complexity tasks", async () => {
      const tasks = Array(15)
        .fill(0)
        .map((_, i) => createTaskInput("medium", i));
      const metrics = await measurePerformance(tasks, "Medium Tasks Baseline");

      expect(metrics.averageProcessingTime).toBeLessThan(
        PERFORMANCE_BASELINES.MEDIUM_TASK_MAX_TIME
      );
      expect(metrics.p95ProcessingTime).toBeLessThan(
        PERFORMANCE_BASELINES.MEDIUM_TASK_MAX_TIME * 2
      );
      expect(metrics.errorRate).toBeLessThan(
        PERFORMANCE_BASELINES.MAX_ERROR_RATE_PERCENT
      );
    });

    it("should meet baseline performance for complex tasks", async () => {
      const tasks = Array(10)
        .fill(0)
        .map((_, i) => createTaskInput("complex", i));
      const metrics = await measurePerformance(tasks, "Complex Tasks Baseline");

      expect(metrics.averageProcessingTime).toBeLessThan(
        PERFORMANCE_BASELINES.COMPLEX_TASK_MAX_TIME
      );
      expect(metrics.p95ProcessingTime).toBeLessThan(
        PERFORMANCE_BASELINES.COMPLEX_TASK_MAX_TIME * 1.5
      );
      expect(metrics.errorRate).toBeLessThan(
        PERFORMANCE_BASELINES.MAX_ERROR_RATE_PERCENT * 2
      ); // Allow higher error rate for complex tasks
    });
  });

  describe("Load Testing", () => {
    it("should handle 50 concurrent tasks within performance limits", async () => {
      const tasks = Array(50)
        .fill(0)
        .map((_, i) => createTaskInput("simple", i));
      const metrics = await measurePerformance(tasks, "50 Concurrent Tasks");

      expect(metrics.completedTasks).toBeGreaterThan(45); // At least 90% completion
      expect(metrics.averageProcessingTime).toBeLessThan(
        PERFORMANCE_BASELINES.SIMPLE_TASK_MAX_TIME * 2
      );
      expect(metrics.memoryUsageMB).toBeLessThan(
        PERFORMANCE_BASELINES.MAX_MEMORY_USAGE_MB
      );
      expect(metrics.dbQueryCount).toBeGreaterThan(0);
      expect(metrics.averageDbQueryTime).toBeLessThan(
        PERFORMANCE_BASELINES.MAX_DB_QUERY_TIME
      );
    });

    it("should handle 100 concurrent tasks with graceful degradation", async () => {
      const tasks = Array(100)
        .fill(0)
        .map((_, i) => createTaskInput("simple", i));
      const metrics = await measurePerformance(tasks, "100 Concurrent Tasks");

      expect(metrics.completedTasks).toBeGreaterThan(80); // At least 80% completion under high load
      expect(metrics.errorRate).toBeLessThan(
        PERFORMANCE_BASELINES.MAX_ERROR_RATE_PERCENT * 3
      ); // Allow higher error rate under extreme load
      expect(metrics.timeoutRate).toBeLessThan(
        PERFORMANCE_BASELINES.MAX_TIMEOUT_RATE_PERCENT * 2
      );
      expect(metrics.memoryUsageMB).toBeLessThan(
        PERFORMANCE_BASELINES.MAX_MEMORY_USAGE_MB * 1.5
      );
    });

    it("should handle mixed workload of 75 tasks with different complexities", async () => {
      const simpleTasks = Array(50)
        .fill(0)
        .map((_, i) => createTaskInput("simple", i));
      const mediumTasks = Array(20)
        .fill(0)
        .map((_, i) => createTaskInput("medium", i + 50));
      const complexTasks = Array(5)
        .fill(0)
        .map((_, i) => createTaskInput("complex", i + 70));

      const tasks = [...simpleTasks, ...mediumTasks, ...complexTasks];
      const metrics = await measurePerformance(
        tasks,
        "Mixed Workload 75 Tasks"
      );

      expect(metrics.completedTasks).toBeGreaterThan(65); // At least 85% completion
      expect(metrics.errorRate).toBeLessThan(
        PERFORMANCE_BASELINES.MAX_ERROR_RATE_PERCENT * 2
      );
      expect(metrics.memoryUsageMB).toBeLessThan(
        PERFORMANCE_BASELINES.MAX_MEMORY_USAGE_MB
      );
    });
  });

  describe("Stress Testing with Chaos", () => {
    it("should maintain performance under controlled chaos conditions", async () => {
      const tasks = Array(30)
        .fill(0)
        .map((_, i) => createTaskInput("simple", i));
      const metrics = await measurePerformance(
        tasks,
        "Chaos Stress Test",
        true
      );

      expect(metrics.completedTasks).toBeGreaterThan(20); // At least 65% completion under chaos
      expect(metrics.errorRate).toBeLessThan(20); // Allow higher error rate under chaos
      expect(metrics.memoryUsageMB).toBeLessThan(
        PERFORMANCE_BASELINES.MAX_MEMORY_USAGE_MB * 1.2
      );
    });

    it("should recover gracefully from chaos scenarios", async () => {
      // First, run with chaos
      const chaosTasks = Array(20)
        .fill(0)
        .map((_, i) => createTaskInput("simple", i));
      const chaosMetrics = await measurePerformance(
        chaosTasks,
        "Chaos Recovery Test",
        true
      );

      // Wait a moment for system to stabilize
      await new Promise((resolve) => setTimeout(resolve, 5000));

      // Then run without chaos
      const recoveryTasks = Array(20)
        .fill(0)
        .map((_, i) => createTaskInput("simple", i + 20));
      const recoveryMetrics = await measurePerformance(
        recoveryTasks,
        "Post-Chaos Recovery Test",
        false
      );

      // Performance should recover
      expect(recoveryMetrics.completedTasks).toBeGreaterThan(
        chaosMetrics.completedTasks
      );
      expect(recoveryMetrics.errorRate).toBeLessThan(chaosMetrics.errorRate);
    });
  });

  describe("Resource Utilization", () => {
    it("should maintain stable memory usage under sustained load", async () => {
      // Run multiple batches to test memory stability
      const batches = 3;
      const tasksPerBatch = 25;

      for (let batch = 0; batch < batches; batch++) {
        const tasks = Array(tasksPerBatch)
          .fill(0)
          .map((_, i) => createTaskInput("simple", batch * tasksPerBatch + i));

        const metrics = await measurePerformance(
          tasks,
          `Memory Stability Batch ${batch + 1}`
        );

        // Memory usage should not grow excessively
        expect(metrics.memoryUsageMB).toBeLessThan(
          PERFORMANCE_BASELINES.MAX_MEMORY_USAGE_MB
        );
      }
    });

    it("should efficiently utilize database connections", async () => {
      const tasks = Array(40)
        .fill(0)
        .map((_, i) => createTaskInput("simple", i));
      const metrics = await measurePerformance(
        tasks,
        "Database Connection Efficiency"
      );

      // Should have reasonable number of DB queries
      expect(metrics.dbQueryCount).toBeGreaterThan(0);
      expect(metrics.dbQueryCount).toBeLessThan(tasks.length * 10); // Not excessive
      expect(metrics.averageDbQueryTime).toBeLessThan(
        PERFORMANCE_BASELINES.MAX_DB_QUERY_TIME
      );
    });
  });

  describe("Scalability Limits", () => {
    it("should identify system limits under extreme load", async () => {
      // Test with very high number of concurrent tasks
      const tasks = Array(150)
        .fill(0)
        .map((_, i) => createTaskInput("simple", i));
      const metrics = await measurePerformance(tasks, "Scalability Limit Test");

      // Document the actual limits observed
      console.log(`\n=== Scalability Limits Observed ===`);
      console.log(`Max Concurrent Tasks Handled: ${metrics.completedTasks}`);
      console.log(`Error Rate at Limit: ${metrics.errorRate.toFixed(1)}%`);
      console.log(`Timeout Rate at Limit: ${metrics.timeoutRate.toFixed(1)}%`);
      console.log(
        `Memory Usage at Limit: ${metrics.memoryUsageMB.toFixed(1)}MB`
      );
      console.log(
        `Throughput at Limit: ${metrics.throughputTasksPerSecond.toFixed(
          2
        )} tasks/sec`
      );

      // Should still complete some tasks even under extreme load
      expect(metrics.completedTasks).toBeGreaterThan(0);
    });
  });
});
