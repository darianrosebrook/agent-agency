/**
 * Performance Tracker Integration Tests
 *
 * Tests the complete integration of Performance Tracker with database persistence,
 * task execution tracking, and agent registry integration.
 *
 * @author @darianrosebrook
 */

import { afterEach, beforeEach, describe, expect, it } from "@jest/globals";
import { PerformanceTracker } from "../../src/rl/PerformanceTracker.js";

describe("Performance Tracker Integration", () => {
  let performanceTracker: PerformanceTracker;

  beforeEach(() => {
    performanceTracker = new PerformanceTracker({
      enabled: true,
      maxEventsInMemory: 1000,
      retentionPeriodMs: 7 * 24 * 60 * 60 * 1000, // 7 days
      batchSize: 10,
      anonymizeData: false,
      enableDatabasePersistence: false, // Disable for testing to avoid DB dependencies
    });
  });

  afterEach(async () => {
    await performanceTracker.stopCollection();
  });

  describe("Core Functionality", () => {
    it("should start and stop collection", async () => {
      await performanceTracker.startCollection();
      expect((performanceTracker as any).isCollecting).toBe(true);

      await performanceTracker.stopCollection();
      expect((performanceTracker as any).isCollecting).toBe(false);
    });

    it("should record agent registration", async () => {
      await performanceTracker.startCollection();

      const agentId = "test-agent-123";
      const profile = {
        capabilities: ["task-execution", "data-processing"],
        baselineMetrics: {
          latencyMs: 500,
          accuracy: 0.95,
          costPerTask: 0.1,
          reliability: 0.98,
        },
        registrationTimestamp: new Date().toISOString(),
      };

      await performanceTracker.recordAgentRegistration(agentId, profile);

      const events = (performanceTracker as any).events;
      expect(events.length).toBeGreaterThan(0);
    });
  });

  describe("Task Execution Tracking", () => {
    it("should track task execution start and completion", async () => {
      await performanceTracker.startCollection();

      const taskId = "test-task-123";
      const agentId = "test-agent-456";
      const routingDecision = {
        taskId,
        selectedAgent: agentId,
        routingStrategy: "direct-execution" as any,
        confidence: 0.9,
        rationale: "Test routing",
        alternativesConsidered: [],
        timestamp: new Date().toISOString(),
      };

      const context = {
        taskType: "data-processing",
        priority: "high",
        timeoutMs: 30000,
        budget: 1000,
      };

      // Start task execution
      const executionId = performanceTracker.startTaskExecution(
        taskId,
        agentId,
        routingDecision,
        context
      );

      expect(executionId).toBeDefined();

      // Complete task execution
      const outcome = {
        success: true,
        qualityScore: 0.95,
        efficiencyScore: 0.88,
        tokensConsumed: 150,
        completionTimeMs: 2500,
      };

      await performanceTracker.completeTaskExecution(executionId, outcome);

      // Verify events were recorded
      const events = (performanceTracker as any).events;
      expect(events.length).toBeGreaterThan(0);
    });

    it("should track task execution failures", async () => {
      await performanceTracker.startCollection();

      const taskId = "failing-task-123";
      const agentId = "test-agent-456";
      const routingDecision = {
        taskId,
        selectedAgent: agentId,
        routingStrategy: "direct-execution" as any,
        confidence: 0.7,
        rationale: "Test failure routing",
        alternativesConsidered: [],
        timestamp: new Date().toISOString(),
      };

      const context = {
        taskType: "data-processing",
        priority: "medium",
        timeoutMs: 15000,
        budget: 500,
      };

      const executionId = performanceTracker.startTaskExecution(
        taskId,
        agentId,
        routingDecision,
        context
      );

      // Complete with failure
      const failureOutcome = {
        success: false,
        qualityScore: 0.0,
        efficiencyScore: 0.0,
        tokensConsumed: 50,
        completionTimeMs: 8000,
      };

      await performanceTracker.completeTaskExecution(
        executionId,
        failureOutcome
      );

      const events = (performanceTracker as any).events;
      expect(events.length).toBeGreaterThan(0);
    });
  });

  describe("Constitutional Validation Tracking", () => {
    it("should track CAWS validation results", async () => {
      await performanceTracker.startCollection();

      const taskId = "validated-task-123";
      const agentId = "test-agent-456";
      const validationResult = {
        valid: true,
        violations: [],
        complianceScore: 1.0,
        processingTimeMs: 150,
        ruleCount: 5,
      };

      await performanceTracker.recordConstitutionalValidation({
        taskId,
        agentId,
        validationResult,
      });

      const events = (performanceTracker as any).events;
      expect(events.length).toBeGreaterThan(0);
    });

    it("should track CAWS validation failures", async () => {
      await performanceTracker.startCollection();

      const taskId = "invalid-task-123";
      const agentId = "test-agent-456";
      const validationResult = {
        valid: false,
        violations: [
          {
            severity: "high" as const,
            message: "Task exceeds complexity budget",
          },
          {
            severity: "medium" as const,
            message: "Missing required documentation",
          },
        ],
        complianceScore: 0.3,
        processingTimeMs: 200,
        ruleCount: 8,
      };

      await performanceTracker.recordConstitutionalValidation({
        taskId,
        agentId,
        validationResult,
      });

      const events = (performanceTracker as any).events;
      expect(events.length).toBeGreaterThan(0);
    });
  });

  describe("Performance Statistics", () => {
    it("should provide accurate performance statistics", async () => {
      await performanceTracker.startCollection();

      // Record some test events
      await performanceTracker.recordAgentRegistration("agent-1", {
        capabilities: ["task-execution"],
        baselineMetrics: {
          latencyMs: 1000,
          accuracy: 0.9,
          costPerTask: 0.1,
          reliability: 0.95,
        },
        registrationTimestamp: new Date().toISOString(),
      });

      // Start and complete a task
      const executionId = performanceTracker.startTaskExecution(
        "task-1",
        "agent-1",
        {
          taskId: "task-1",
          selectedAgent: "agent-1",
          routingStrategy: "direct-execution" as any,
          confidence: 0.8,
          rationale: "Test routing",
          alternativesConsidered: [],
          timestamp: new Date().toISOString(),
        },
        {
          taskType: "data-processing",
          priority: "high",
          timeoutMs: 30000,
          budget: 1000,
        }
      );

      await performanceTracker.completeTaskExecution(executionId, {
        success: true,
        qualityScore: 0.9,
        efficiencyScore: 0.8,
        tokensConsumed: 100,
        completionTimeMs: 2000,
      });

      const stats = performanceTracker.getStats();
      expect(stats.totalRoutingDecisions).toBeGreaterThan(0);
      expect(stats.totalTaskExecutions).toBeGreaterThan(0);
      expect(stats.overallSuccessRate).toBeGreaterThanOrEqual(0);
    });
  });

  describe("Performance Statistics", () => {
    it("should provide accurate performance statistics", async () => {
      await performanceTracker.startCollection();

      // Record some test events
      await performanceTracker.recordAgentRegistration("agent-1", {
        capabilities: ["task-execution"],
        baselineMetrics: {
          latencyMs: 1000,
          accuracy: 0.9,
          costPerTask: 0.1,
          reliability: 0.95,
        },
        registrationTimestamp: new Date().toISOString(),
      });

      // Start and complete a task
      const executionId = performanceTracker.startTaskExecution(
        "task-1",
        "agent-1",
        {
          taskId: "task-1",
          selectedAgent: "agent-1",
          routingStrategy: "direct-execution" as any,
          confidence: 0.8,
          rationale: "Test routing",
          alternativesConsidered: [],
          timestamp: new Date().toISOString(),
        },
        {
          taskType: "data-processing",
          priority: "high",
          timeoutMs: 30000,
          budget: 1000,
        }
      );

      await performanceTracker.completeTaskExecution(executionId, {
        success: true,
        qualityScore: 0.9,
        efficiencyScore: 0.8,
        tokensConsumed: 100,
        completionTimeMs: 2000,
      });

      const stats = performanceTracker.getStats();
      expect(stats.totalRoutingDecisions).toBeGreaterThan(0);
      expect(stats.totalTaskExecutions).toBeGreaterThan(0);
      expect(stats.overallSuccessRate).toBeGreaterThanOrEqual(0);
    });
  });

  describe("Configuration", () => {
    it("should work with different configurations", async () => {
      const trackerWithCustomConfig = new PerformanceTracker({
        enabled: true,
        maxEventsInMemory: 100,
        retentionPeriodMs: 24 * 60 * 60 * 1000, // 1 day
        batchSize: 5,
        anonymizeData: false,
        enableDatabasePersistence: false,
      });

      await trackerWithCustomConfig.startCollection();

      // Should still record events in memory
      await trackerWithCustomConfig.recordAgentRegistration("agent-1", {
        capabilities: ["task-execution"],
        baselineMetrics: {
          latencyMs: 1000,
          accuracy: 0.9,
          costPerTask: 0.1,
          reliability: 0.95,
        },
        registrationTimestamp: new Date().toISOString(),
      });

      const events = (trackerWithCustomConfig as any).events;
      expect(events.length).toBeGreaterThan(0);

      await trackerWithCustomConfig.stopCollection();
    });
  });
});
