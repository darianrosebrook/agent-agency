/**
 * @fileoverview Integration Tests for Performance Tracker (ARBITER-004)
 *
 * Tests real-world integration scenarios with actual data collectors and system components.
 *
 * @author @darianrosebrook
 */

import { DataCollector } from "../../../src/benchmarking/DataCollector";
import { PerformanceTracker } from "../../../src/rl/PerformanceTracker";
import { RoutingDecision, TaskOutcome } from "../../../src/types/agentic-rl";

describe("Performance Tracker - Integration Tests (ARBITER-004)", () => {
  let tracker: PerformanceTracker;
  let dataCollector: DataCollector;

  const createRoutingDecision = (
    taskId: string = "task-1"
  ): RoutingDecision => ({
    taskId,
    selectedAgent: "agent-test",
    routingStrategy: "multi-armed-bandit",
    confidence: 0.85,
    alternativesConsidered: [
      { agentId: "agent-test", score: 0.9, reason: "Best choice" },
      { agentId: "agent-backup", score: 0.7, reason: "Fallback" },
    ],
    rationale: "Selected based on historical performance",
    timestamp: new Date().toISOString(),
  });

  const createTaskOutcome = (success: boolean = true): TaskOutcome => ({
    success,
    qualityScore: success ? 0.88 : 0.35,
    efficiencyScore: success ? 0.92 : 0.4,
    tokensConsumed: success ? 1800 : 900,
    completionTimeMs: success ? 2800 : 1600,
  });

  beforeEach(() => {
    // Create data collector with test configuration
    dataCollector = new DataCollector({
      enabled: true,
      samplingRate: 1.0,
      maxBufferSize: 1000,
      batchSize: 10,
      retentionDays: 1,
      anonymization: {
        enabled: false,
        level: "basic",
        preserveAgentIds: true,
        preserveTaskTypes: true,
      },
    });

    // Create tracker with real data collector
    tracker = new PerformanceTracker(
      {
        enabled: true,
        maxEventsInMemory: 1000,
        retentionPeriodMs: 24 * 60 * 60 * 1000,
        batchSize: 10,
        anonymizeData: false,
      },
      dataCollector
    );

    tracker.startCollection();
  });

  afterEach(() => {
    tracker.stopCollection();
    tracker.clearData();
  });

  describe("End-to-End Task Tracking", () => {
    it("should track complete task lifecycle with all metrics", async () => {
      const taskId = "integration-task-1";
      const agentId = "integration-agent-1";

      // Step 1: Record agent registration
      await tracker.recordAgentRegistration(agentId, {
        capabilities: ["code-editing", "analysis"],
        baselineMetrics: {
          latencyMs: 200,
          accuracy: 0.9,
          costPerTask: 0.05,
          reliability: 0.95,
        },
        registrationTimestamp: new Date().toISOString(),
      });

      // Step 2: Record routing decision
      const routingDecision = createRoutingDecision(taskId);
      routingDecision.selectedAgent = agentId;
      await tracker.recordRoutingDecision(routingDecision);

      // Step 3: Start task execution
      const executionId = await tracker.startTaskExecution(
        taskId,
        agentId,
        routingDecision,
        { priority: "high", complexity: "medium" }
      );

      // Simulate task execution time
      await new Promise((resolve) => setTimeout(resolve, 50));

      // Step 4: Record constitutional validation
      await tracker.recordConstitutionalValidation({
        taskId,
        agentId,
        validationResult: {
          valid: true,
          violations: [],
          complianceScore: 0.98,
          processingTimeMs: 120,
          ruleCount: 25,
        },
      });

      // Step 5: Complete task execution
      await tracker.completeTaskExecution(executionId, createTaskOutcome(true));

      // Step 6: Record evaluation
      await tracker.recordEvaluationOutcome(taskId, {
        passed: true,
        score: 0.92,
        rubricScores: {
          correctness: 0.95,
          completeness: 0.9,
          efficiency: 0.91,
        },
        feedback: "Excellent work",
      });

      // Verify complete data collection
      const stats = tracker.getStats();
      expect(stats.totalRoutingDecisions).toBeGreaterThanOrEqual(1);
      expect(stats.totalTaskExecutions).toBeGreaterThanOrEqual(1);
      expect(stats.totalEvaluationOutcomes).toBeGreaterThanOrEqual(1);
      expect(stats.overallSuccessRate).toBe(1.0);

      const exportedData = tracker.exportTrainingData();
      expect(exportedData.length).toBeGreaterThanOrEqual(5); // All events recorded
    });

    it("should handle failed tasks with full error tracking", async () => {
      const taskId = "failed-task-1";
      const agentId = "agent-prone-to-failure";

      await tracker.recordAgentRegistration(agentId, {
        capabilities: ["testing"],
        baselineMetrics: {
          latencyMs: 300,
          accuracy: 0.75,
          costPerTask: 0.08,
          reliability: 0.8,
        },
        registrationTimestamp: new Date().toISOString(),
      });

      const routingDecision = createRoutingDecision(taskId);
      routingDecision.selectedAgent = agentId;
      await tracker.recordRoutingDecision(routingDecision);

      const executionId = await tracker.startTaskExecution(
        taskId,
        agentId,
        routingDecision
      );

      await new Promise((resolve) => setTimeout(resolve, 30));

      // Record validation failure
      await tracker.recordConstitutionalValidation({
        taskId,
        agentId,
        validationResult: {
          valid: false,
          violations: [
            {
              severity: "high",
              message: "Security violation",
              rule: "SEC-001",
            },
            {
              severity: "medium",
              message: "Style violation",
              rule: "STYLE-015",
            },
          ],
          complianceScore: 0.45,
          processingTimeMs: 150,
          ruleCount: 25,
        },
      });

      await tracker.completeTaskExecution(
        executionId,
        createTaskOutcome(false)
      );

      await tracker.recordEvaluationOutcome(taskId, {
        passed: false,
        score: 0.4,
        feedback: "Multiple violations detected",
      });

      const stats = tracker.getStats();
      expect(stats.overallSuccessRate).toBe(0); // 100% failure
      expect(stats.totalTaskExecutions).toBe(1);

      const exportedData = tracker.exportTrainingData();
      const validationEvents = exportedData.filter(
        (e) => e.type === "constitutional-validation"
      );
      expect(validationEvents.length).toBeGreaterThanOrEqual(1);
      expect((validationEvents[0].data as any).validationResult.valid).toBe(
        false
      );
    });
  });

  describe("Multi-Agent Performance Comparison", () => {
    it("should track performance metrics across multiple agents", async () => {
      const agents = [
        {
          id: "fast-agent",
          latency: 150,
          accuracy: 0.85,
          successRate: 0.9,
        },
        {
          id: "accurate-agent",
          latency: 300,
          accuracy: 0.95,
          successRate: 0.95,
        },
        {
          id: "balanced-agent",
          latency: 200,
          accuracy: 0.9,
          successRate: 0.92,
        },
      ];

      // Register all agents
      for (const agent of agents) {
        await tracker.recordAgentRegistration(agent.id, {
          capabilities: ["code-editing"],
          baselineMetrics: {
            latencyMs: agent.latency,
            accuracy: agent.accuracy,
            costPerTask: 0.05,
            reliability: agent.successRate,
          },
          registrationTimestamp: new Date().toISOString(),
        });
      }

      // Execute tasks with each agent
      for (let i = 0; i < 5; i++) {
        for (const agent of agents) {
          const taskId = `task-${agent.id}-${i}`;
          const routingDecision = createRoutingDecision(taskId);
          routingDecision.selectedAgent = agent.id;

          await tracker.recordRoutingDecision(routingDecision);

          const executionId = await tracker.startTaskExecution(
            taskId,
            agent.id,
            routingDecision
          );

          await new Promise((resolve) => setTimeout(resolve, 10));

          const success = Math.random() < agent.successRate;
          await tracker.completeTaskExecution(
            executionId,
            createTaskOutcome(success)
          );

          await tracker.recordEvaluationOutcome(taskId, {
            passed: success,
            score: success ? agent.accuracy : agent.accuracy * 0.5,
          });
        }
      }

      const stats = tracker.getStats();
      expect(stats.totalRoutingDecisions).toBe(15); // 3 agents * 5 tasks
      expect(stats.totalTaskExecutions).toBe(15);
      expect(stats.totalEvaluationOutcomes).toBe(15);

      // Verify agent-specific data is preserved
      const exportedData = tracker.exportTrainingData();
      const registrations = exportedData.filter(
        (e) => e.type === "agent-registration"
      );
      expect(registrations.length).toBe(3);
    });
  });

  describe("Concurrent Operations and Race Conditions", () => {
    it("should handle concurrent metric collection without data loss", async () => {
      const concurrentTasks = 50;
      const promises = [];

      for (let i = 0; i < concurrentTasks; i++) {
        const taskId = `concurrent-task-${i}`;
        const agentId = `agent-${i % 5}`; // 5 different agents

        promises.push(
          (async () => {
            await tracker.recordRoutingDecision(createRoutingDecision(taskId));

            const executionId = await tracker.startTaskExecution(
              taskId,
              agentId,
              createRoutingDecision(taskId)
            );

            await new Promise((resolve) => setTimeout(resolve, 5));

            await tracker.completeTaskExecution(
              executionId,
              createTaskOutcome(true)
            );

            await tracker.recordEvaluationOutcome(taskId, {
              passed: true,
              score: 0.85,
            });
          })()
        );
      }

      await Promise.all(promises);

      const stats = tracker.getStats();
      expect(stats.totalRoutingDecisions).toBe(concurrentTasks);
      expect(stats.totalTaskExecutions).toBe(concurrentTasks);
      expect(stats.totalEvaluationOutcomes).toBe(concurrentTasks);
    });

    it("should maintain data consistency under heavy load", async () => {
      const heavyLoad = 100;
      const startTime = Date.now();

      for (let i = 0; i < heavyLoad; i++) {
        await tracker.recordRoutingDecision(
          createRoutingDecision(`heavy-load-task-${i}`)
        );

        if (i % 10 === 0) {
          await tracker.recordAgentStatusChange(`agent-${i % 5}`, "busy", {
            previousStatus: "available",
          });
        }
      }

      const duration = Date.now() - startTime;
      const avgLatency = duration / heavyLoad;

      // Should maintain good performance under load
      expect(avgLatency).toBeLessThan(10); // < 10ms per operation

      const stats = tracker.getStats();
      expect(stats.totalRoutingDecisions).toBe(heavyLoad);
    });
  });

  describe("Data Retention and Cleanup", () => {
    it("should respect retention policies under continuous operation", async () => {
      // Set short retention for testing
      tracker.updateConfig({ retentionPeriodMs: 200 }); // 200ms

      // Add events
      for (let i = 0; i < 10; i++) {
        await tracker.recordRoutingDecision(createRoutingDecision(`task-${i}`));
        await new Promise((resolve) => setTimeout(resolve, 30));
      }

      // Wait for retention period
      await new Promise((resolve) => setTimeout(resolve, 250));

      // Add a new event to trigger cleanup
      await tracker.recordRoutingDecision(createRoutingDecision("new-task"));

      const exportedData = tracker.exportTrainingData();

      // Most old events should be cleaned up
      expect(exportedData.length).toBeLessThan(10);
    });

    it("should maintain memory limits during long-running operations", async () => {
      tracker.updateConfig({ maxEventsInMemory: 50 });

      // Generate many events
      for (let i = 0; i < 200; i++) {
        await tracker.recordRoutingDecision(createRoutingDecision(`task-${i}`));
      }

      const exportedData = tracker.exportTrainingData();
      expect(exportedData.length).toBeLessThanOrEqual(50);

      // Verify most recent events are kept
      const recentTask = exportedData.find((e) =>
        JSON.stringify(e.data).includes("task-199")
      );
      expect(recentTask).toBeDefined();
    });
  });

  describe("Integration with RL Training Pipeline", () => {
    it("should provide complete training data for RL algorithms", async () => {
      // Simulate a training scenario
      const trainingTasks = 20;

      for (let i = 0; i < trainingTasks; i++) {
        const taskId = `training-task-${i}`;
        const agentId = `agent-${i % 3}`;

        // Routing
        const routingDecision = createRoutingDecision(taskId);
        routingDecision.selectedAgent = agentId;
        await tracker.recordRoutingDecision(routingDecision);

        // Thinking budget
        await tracker.recordThinkingBudget(taskId, {
          allocatedTokens: 5000,
          complexityLevel: i % 2 === 0 ? "high" : "medium",
          confidence: 0.8 + Math.random() * 0.15,
        });

        // Execution
        const executionId = await tracker.startTaskExecution(
          taskId,
          agentId,
          routingDecision
        );
        await new Promise((resolve) => setTimeout(resolve, 20));
        await tracker.completeTaskExecution(
          executionId,
          createTaskOutcome(true)
        );

        // Budget usage
        await tracker.recordBudgetUsage(taskId, {
          tokensUsed: 4200 + Math.floor(Math.random() * 800),
          tokensAllocated: 5000,
          utilizationRate: 0.84 + Math.random() * 0.15,
        });

        // Evaluation
        await tracker.recordEvaluationOutcome(taskId, {
          passed: true,
          score: 0.8 + Math.random() * 0.15,
        });
      }

      // Export training data
      const trainingData = tracker.exportTrainingData();

      // Verify completeness for RL training
      const eventTypes = new Set(trainingData.map((e) => e.type));
      expect(eventTypes.has("routing-decision")).toBe(true);
      expect(eventTypes.has("task-execution")).toBe(true);
      expect(eventTypes.has("evaluation-outcome")).toBe(true);
      expect(eventTypes.has("thinking-budget-allocation")).toBe(true);
      expect(eventTypes.has("thinking-budget-usage")).toBe(true);

      expect(trainingData.length).toBeGreaterThanOrEqual(trainingTasks * 5);
    });

    it("should support incremental training data exports", async () => {
      // Initial batch
      for (let i = 0; i < 10; i++) {
        await tracker.recordRoutingDecision(
          createRoutingDecision(`batch-1-task-${i}`)
        );
      }

      await new Promise((resolve) => setTimeout(resolve, 100));
      const checkpoint = new Date().toISOString();
      await new Promise((resolve) => setTimeout(resolve, 100));

      // Second batch
      for (let i = 0; i < 10; i++) {
        await tracker.recordRoutingDecision(
          createRoutingDecision(`batch-2-task-${i}`)
        );
      }

      // Export only data since checkpoint
      const incrementalData = tracker.exportTrainingData(checkpoint);

      expect(incrementalData.length).toBeGreaterThanOrEqual(10);
      expect(incrementalData.length).toBeLessThan(20);
    });
  });

  describe("Error Recovery and Resilience", () => {
    it("should recover gracefully from temporary failures", async () => {
      // Simulate normal operation
      await tracker.recordRoutingDecision(createRoutingDecision("task-before"));

      // Simulate temporary failure (stop collection)
      tracker.stopCollection();

      // Try to record during downtime
      await tracker.recordRoutingDecision(createRoutingDecision("task-during"));

      // Recover
      tracker.startCollection();

      // Resume normal operation
      await tracker.recordRoutingDecision(createRoutingDecision("task-after"));

      const exportedData = tracker.exportTrainingData();

      // Should have events before and after, but not during
      const hasBefore = exportedData.some((e) =>
        JSON.stringify(e.data).includes("task-before")
      );
      const hasDuring = exportedData.some((e) =>
        JSON.stringify(e.data).includes("task-during")
      );
      const hasAfter = exportedData.some((e) =>
        JSON.stringify(e.data).includes("task-after")
      );

      expect(hasBefore).toBe(true);
      expect(hasDuring).toBe(false);
      expect(hasAfter).toBe(true);
    });

    it("should maintain data integrity across stop/start cycles", async () => {
      for (let cycle = 0; cycle < 5; cycle++) {
        await tracker.recordRoutingDecision(
          createRoutingDecision(`cycle-${cycle}`)
        );

        tracker.stopCollection();
        await new Promise((resolve) => setTimeout(resolve, 10));
        tracker.startCollection();
      }

      const stats = tracker.getStats();
      expect(stats.totalRoutingDecisions).toBe(5);
    });
  });
});
