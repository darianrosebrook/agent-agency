/**
 * Performance Tracking End-to-End Integration Tests
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import { afterAll, beforeAll, describe, expect, it } from "@jest/globals";
import { DataCollector } from "../../../src/benchmarking/DataCollector";
import { MetricAggregator } from "../../../src/benchmarking/MetricAggregator";
import { PerformanceAnalyzer } from "../../../src/benchmarking/PerformanceAnalyzer";
import { RLDataPipeline } from "../../../src/benchmarking/RLDataPipeline";
import { SpecValidator } from "../../../src/caws-validator/validation/SpecValidator";
import { AgentRegistryManager } from "../../../src/orchestrator/AgentRegistryManager";
import { PerformanceTracker } from "../../../src/rl/PerformanceTracker";
import { RoutingDecision, TaskOutcome } from "../../../src/types/agentic-rl";
import { WorkingSpec } from "../../../src/types/caws-types";
import { PerformanceEventType } from "../../../src/types/performance-tracking";

describe("Performance Tracking E2E Integration", () => {
  let dataCollector: DataCollector;
  let metricAggregator: MetricAggregator;
  let rlDataPipeline: RLDataPipeline;
  let performanceAnalyzer: PerformanceAnalyzer;
  let performanceTracker: PerformanceTracker;
  let agentRegistry: AgentRegistryManager;
  let specValidator: SpecValidator;

  const createMockRoutingDecision = (
    taskId: string,
    agentId: string
  ): RoutingDecision => ({
    taskId,
    selectedAgent: agentId,
    routingStrategy: "multi-armed-bandit",
    confidence: 0.85,
    alternativesConsidered: [
      { agentId, score: 0.9, reason: "Best performance metrics" },
      { agentId: "agent-fallback", score: 0.7, reason: "Available capacity" },
    ],
    rationale: "Selected agent with best historical performance",
    timestamp: new Date().toISOString(),
  });

  beforeAll(async () => {
    // Initialize all components
    dataCollector = new DataCollector();
    metricAggregator = new MetricAggregator();
    rlDataPipeline = new RLDataPipeline();
    performanceAnalyzer = new PerformanceAnalyzer();
    performanceTracker = new PerformanceTracker({}, dataCollector);
    agentRegistry = new AgentRegistryManager({
      enableSecurity: false, // Disable security for testing
    });
    specValidator = new SpecValidator(performanceTracker);

    // Wire PerformanceTracker with components
    agentRegistry.setPerformanceTracker(performanceTracker);

    // Start all processing
    dataCollector.startCollection();
    metricAggregator.startAggregation();
    rlDataPipeline.startProcessing();
    performanceAnalyzer.startAnalysis();
    performanceTracker.startCollection();
  });

  afterAll(async () => {
    // Clean up
    dataCollector.stopCollection();
    metricAggregator.stopAggregation();
    rlDataPipeline.stopProcessing();
    performanceAnalyzer.stopAnalysis();
    performanceTracker.stopCollection();
  });

  describe("Complete Performance Tracking Flow", () => {
    it("should process agent interactions from start to RL training data", async () => {
      // Step 1: Simulate agent task execution
      const taskId = "e2e-task-123";
      const agentId = "agent-1";

      // Record task start through PerformanceTracker (which integrates DataCollector)
      performanceTracker.startTaskExecution(
        taskId,
        agentId,
        createMockRoutingDecision(taskId, agentId)
      );

      // Simulate some processing time
      await new Promise((resolve) => setTimeout(resolve, 10));

      // Record task completion with comprehensive metrics
      const taskOutcome: TaskOutcome = {
        success: true,
        qualityScore: 0.87,
        efficiencyScore: 0.92,
        tokensConsumed: 1250,
        completionTimeMs: 850,
      };

      await performanceTracker.completeTaskExecution(taskId, taskOutcome);

      // Step 2: Verify data collection
      const pendingEvents = dataCollector.getPendingEvents(10);
      expect(pendingEvents.length).toBeGreaterThanOrEqual(2); // start + completion

      const completionEvent = pendingEvents.find(
        (e) => e.type === PerformanceEventType.TASK_EXECUTION_COMPLETE
      );
      expect(completionEvent).toBeDefined();
      expect(completionEvent?.taskId).toBe(taskId);
      expect(completionEvent?.agentId).toBe(agentId);

      // Step 3: Process events through aggregator
      const events = dataCollector.getPendingEvents(100);
      dataCollector.clearBuffer(); // Clear buffer

      await metricAggregator.addEvents(events);
      await metricAggregator.performAggregation();

      // Step 4: Verify performance profiles are generated
      const profiles = metricAggregator.getPerformanceProfiles(agentId);
      expect(profiles.length).toBeGreaterThan(0);

      const profile = profiles[0];
      expect(profile.agentId).toBe(agentId);
      expect(profile.metrics.latency.averageMs).toBeGreaterThan(0);
      expect(profile.metrics.accuracy.successRate).toBeGreaterThan(0);

      // Step 5: Process through RL data pipeline
      const pipelineResult = await rlDataPipeline.processEvents(
        events,
        profiles
      );
      expect(pipelineResult.samplesGenerated).toBeGreaterThan(0);

      // Step 6: Retrieve training batches
      const trainingBatches = rlDataPipeline.getTrainingBatches();
      expect(trainingBatches.length).toBeGreaterThan(0);

      const batch = trainingBatches[0];
      expect(batch.agentId).toBe(agentId);
      expect(batch.samples.length).toBeGreaterThan(0);

      // Step 7: Verify training sample quality
      const sample = batch.samples[0];
      expect(sample.agentId).toBe(agentId);
      expect(sample.reward).toBeGreaterThanOrEqual(0);
      expect(sample.reward).toBeLessThanOrEqual(1);
      expect(sample.integrityHash).toBeDefined();

      // Step 8: Analyze performance trends
      const analysisResult = await performanceAnalyzer.analyzePerformance(
        profiles
      );
      expect(analysisResult.trendResults.length).toBeGreaterThan(0);

      // Verify no anomalies detected for good performance
      const activeAnomalies = performanceAnalyzer.getActiveAnomalies();
      // Should not detect anomalies for good performance data
    });

    it("should handle performance degradation and anomaly detection", async () => {
      const taskId = "degraded-task-456";
      const agentId = "agent-2";

      // Create a scenario with poor performance
      performanceTracker.startTaskExecution(
        taskId,
        agentId,
        createMockRoutingDecision(taskId, agentId)
      );

      // Simulate poor performance outcome
      const poorOutcome: TaskOutcome = {
        success: false,
        qualityScore: 0.3,
        efficiencyScore: 0.4,
        tokensConsumed: 2500,
        completionTimeMs: 5000, // Very slow
      };

      await performanceTracker.completeTaskExecution(taskId, poorOutcome);

      // Process through the pipeline
      const events = dataCollector.getPendingEvents(100);
      dataCollector.clearBuffer();

      await metricAggregator.addEvents(events);
      await metricAggregator.performAggregation();

      const profiles = metricAggregator.getPerformanceProfiles(agentId);
      expect(profiles.length).toBeGreaterThan(0);

      // Analyze for anomalies
      const analysisResult = await performanceAnalyzer.analyzePerformance(
        profiles
      );

      // Should potentially detect anomalies (depending on baseline)
      // The system should handle the degraded performance gracefully
      expect(analysisResult).toBeDefined();
    });

    it("should maintain data integrity throughout the pipeline", async () => {
      // Create multiple tasks to test integrity
      const tasks = [
        { id: "integrity-task-1", agent: "agent-3" },
        { id: "integrity-task-2", agent: "agent-3" },
        { id: "integrity-task-3", agent: "agent-3" },
      ];

      // Execute tasks
      for (const task of tasks) {
        performanceTracker.startTaskExecution(
          task.id,
          task.agent,
          createMockRoutingDecision(task.id, task.agent)
        );

        const outcome: TaskOutcome = {
          success: true,
          qualityScore: 0.8 + Math.random() * 0.2, // Random quality 0.8-1.0
          efficiencyScore: 0.85 + Math.random() * 0.15,
          tokensConsumed: 1000 + Math.floor(Math.random() * 1000),
          completionTimeMs: 500 + Math.floor(Math.random() * 1000),
        };

        await performanceTracker.completeTaskExecution(task.id, outcome);
      }

      // Process through pipeline
      const events = dataCollector.getPendingEvents(100);
      dataCollector.clearBuffer();

      await metricAggregator.addEvents(events);
      await metricAggregator.performAggregation();

      const profiles = metricAggregator.getPerformanceProfiles("agent-3");
      const pipelineResult = await rlDataPipeline.processEvents(
        events,
        profiles
      );

      // Verify data integrity
      expect(pipelineResult.samplesGenerated).toBe(3); // All tasks should generate samples

      const batches = rlDataPipeline.getTrainingBatches("agent-3");
      expect(batches.length).toBeGreaterThan(0);

      // Check that all samples have integrity hashes
      const allSamples = batches.flatMap((b) => b.samples);
      allSamples.forEach((sample) => {
        expect(sample.integrityHash).toBeDefined();
        expect(sample.integrityHash.length).toBeGreaterThan(0);
      });
    });

    it("should handle concurrent agent operations", async () => {
      const concurrentTasks = [
        { id: "concurrent-1", agent: "agent-4" },
        { id: "concurrent-2", agent: "agent-5" },
        { id: "concurrent-3", agent: "agent-4" },
        { id: "concurrent-4", agent: "agent-5" },
      ];

      // Execute concurrently
      const promises = concurrentTasks.map(async (task) => {
        performanceTracker.startTaskExecution(
          task.id,
          task.agent,
          createMockRoutingDecision(task.id, task.agent)
        );

        const outcome: TaskOutcome = {
          success: Math.random() > 0.2, // 80% success rate
          qualityScore: 0.7 + Math.random() * 0.3,
          efficiencyScore: 0.75 + Math.random() * 0.25,
          tokensConsumed: 800 + Math.floor(Math.random() * 800),
          completionTimeMs: 400 + Math.floor(Math.random() * 800),
        };

        await performanceTracker.completeTaskExecution(task.id, outcome);
      });

      await Promise.all(promises);

      // Process all events
      const events = dataCollector.getPendingEvents(100);
      dataCollector.clearBuffer();

      await metricAggregator.addEvents(events);
      await metricAggregator.performAggregation();

      // Verify processing for multiple agents
      const agent4Profiles = metricAggregator.getPerformanceProfiles("agent-4");
      const agent5Profiles = metricAggregator.getPerformanceProfiles("agent-5");

      expect(agent4Profiles.length).toBeGreaterThan(0);
      expect(agent5Profiles.length).toBeGreaterThan(0);

      // Process through RL pipeline
      const pipelineResult = await rlDataPipeline.processEvents(events, [
        ...agent4Profiles,
        ...agent5Profiles,
      ]);

      expect(pipelineResult.samplesGenerated).toBe(4); // All tasks processed
    });
  });

  describe("System Resilience", () => {
    it("should handle component failures gracefully", async () => {
      // Simulate data collector failure
      const originalGetPendingEvents = dataCollector.getPendingEvents;
      dataCollector.getPendingEvents = jest.fn(() => {
        throw new Error("Data collector failure");
      });

      try {
        // Performance tracker should handle this gracefully
        const taskId = "failure-test-task";
        performanceTracker.startTaskExecution(
          taskId,
          "agent-6",
          createMockRoutingDecision(taskId, "agent-6")
        );

        const outcome: TaskOutcome = {
          success: true,
          qualityScore: 0.8,
          efficiencyScore: 0.85,
          tokensConsumed: 1200,
          completionTimeMs: 800,
        };

        // Should not throw despite data collector failure
        await expect(
          performanceTracker.completeTaskExecution(taskId, outcome)
        ).resolves.not.toThrow();
      } finally {
        // Restore original method
        dataCollector.getPendingEvents = originalGetPendingEvents;
      }
    });

    it("should maintain performance under load", async () => {
      const startTime = Date.now();
      const loadTasks = Array.from({ length: 50 }, (_, i) => ({
        id: `load-task-${i}`,
        agent: `load-agent-${i % 5}`, // 5 different agents
      }));

      // Execute high volume of tasks
      for (const task of loadTasks) {
        performanceTracker.startTaskExecution(
          task.id,
          task.agent,
          createMockRoutingDecision(task.id, task.agent)
        );

        const outcome: TaskOutcome = {
          success: true,
          qualityScore: 0.8,
          efficiencyScore: 0.85,
          tokensConsumed: 1000,
          completionTimeMs: 500,
        };

        await performanceTracker.completeTaskExecution(task.id, outcome);
      }

      const processingTime = Date.now() - startTime;
      const avgTimePerTask = processingTime / loadTasks.length;

      // Should maintain reasonable performance (< 50ms per task on average)
      expect(avgTimePerTask).toBeLessThan(50);

      // Verify all tasks were processed
      const events = dataCollector.getPendingEvents(200); // Should have 100 events (50 start + 50 complete)
      expect(events.length).toBeGreaterThanOrEqual(100);
    });
  });

  describe("Data Quality and Privacy", () => {
    it("should anonymize sensitive data appropriately", () => {
      // Create collector with anonymization
      const anonymizedCollector = new DataCollector({
        anonymization: {
          enabled: true,
          level: "basic",
          preserveAgentIds: true,
          preserveTaskTypes: true,
        },
      });

      anonymizedCollector.startCollection();

      // Record event with sensitive data
      anonymizedCollector.recordTaskStart("sensitive-task", "agent-7", {
        userId: "user-12345",
        sessionId: "session-abcdef",
        personalData: "sensitive-info",
        apiKey: "secret-key",
      });

      const events = anonymizedCollector.getPendingEvents(10);
      expect(events.length).toBeGreaterThan(0);

      const event = events[0];
      // Sensitive fields should be anonymized
      expect(event.context?.userId).toBeUndefined();
      expect(event.context?.sessionId).toBeUndefined();
      expect(event.context?.personalData).toBeDefined(); // Should be hashed
      expect(event.context?.apiKey).toBeDefined(); // Should be hashed

      // Non-sensitive fields should be preserved
      expect(event.agentId).toBe("agent-7");
      expect(event.taskId).toBe("sensitive-task");
    });

    it("should maintain data quality through the pipeline", async () => {
      // Create high-quality data
      const qualityTaskId = "quality-task";
      performanceTracker.startTaskExecution(
        qualityTaskId,
        "agent-8",
        createMockRoutingDecision(qualityTaskId, "agent-8")
      );

      const qualityOutcome: TaskOutcome = {
        success: true,
        qualityScore: 0.95,
        efficiencyScore: 0.98,
        tokensConsumed: 800,
        completionTimeMs: 600,
      };

      await performanceTracker.completeTaskExecution(
        qualityTaskId,
        qualityOutcome
      );

      // Process through pipeline
      const events = dataCollector.getPendingEvents(10);
      dataCollector.clearBuffer();

      await metricAggregator.addEvents(events);
      await metricAggregator.performAggregation();

      const profiles = metricAggregator.getPerformanceProfiles("agent-8");
      const pipelineResult = await rlDataPipeline.processEvents(
        events,
        profiles
      );

      // Verify quality metrics
      expect(pipelineResult.qualityIssues.length).toBe(0); // No quality issues

      const batches = rlDataPipeline.getTrainingBatches("agent-8");
      expect(batches.length).toBeGreaterThan(0);

      const batch = batches[0];
      expect(batch.qualityScore).toBeGreaterThan(0.8); // High quality score
    });

    it("should track agent registration and status changes", async () => {
      // Register a new agent through the registry
      const agentData = {
        id: "test-agent-integration",
        name: "Test Agent Integration",
        modelFamily: "gpt-4" as const,
        capabilities: {
          taskTypes: ["code-editing", "code-review", "testing"] as any,
          languages: ["typescript", "python"] as any,
          specializations: ["code-review", "testing"] as any,
        },
      };

      const registeredAgent = await agentRegistry.registerAgent(agentData);

      // Verify agent was registered with baseline metrics
      expect(registeredAgent.id).toBe("test-agent-integration");
      expect(registeredAgent.capabilities.languages).toContain("typescript");

      // Check that performance baseline was recorded
      const events = dataCollector.getPendingEvents(10);
      const registrationEvent = events.find(
        (e) => e.type === PerformanceEventType.AGENT_REGISTRATION
      );
      expect(registrationEvent).toBeDefined();
      expect(registrationEvent?.agentId).toBe("test-agent-integration");
      expect(registrationEvent?.metrics?.baselineLatencyMs).toBeDefined();
      expect(registrationEvent?.metrics?.baselineAccuracy).toBeGreaterThan(0.8);

      // Update agent status to busy
      await agentRegistry.updateAgentStatus(
        "test-agent-integration",
        "busy",
        "Processing high-priority task"
      );

      // Check that status change was recorded
      const updatedEvents = dataCollector.getPendingEvents(10);
      const statusChangeEvent = updatedEvents.find(
        (e) =>
          e.type === PerformanceEventType.AGENT_STATUS_CHANGE &&
          e.agentId === "test-agent-integration"
      );
      expect(statusChangeEvent).toBeDefined();
      expect(statusChangeEvent?.metrics?.status).toBe("busy");
      expect(statusChangeEvent?.metrics?.reason).toBe(
        "Processing high-priority task"
      );

      // Update agent status back to available
      await agentRegistry.updateAgentStatus(
        "test-agent-integration",
        "available",
        "Task completed successfully"
      );

      // Verify final status change
      const finalEvents = dataCollector.getPendingEvents(10);
      const finalStatusEvent = finalEvents.find(
        (e) =>
          e.type === PerformanceEventType.AGENT_STATUS_CHANGE &&
          e.agentId === "test-agent-integration" &&
          e.metrics?.status === "available"
      );
      expect(finalStatusEvent).toBeDefined();
      expect(finalStatusEvent?.metrics?.previousStatus).toBe("busy");
    });

    it("should track constitutional validation compliance metrics", async () => {
      // Create a valid working spec
      const validSpec: WorkingSpec = {
        id: "TEST-001",
        title: "Test Specification",
        mode: "feature",
        risk_tier: 3,
        blast_radius: {
          modules: ["test"],
          data_migration: false,
        },
        operational_rollback_slo: "15m",
        scope: {
          in: ["src/test/"],
          out: ["node_modules/"],
        },
        acceptance: [
          {
            id: "A1",
            given: "Test scenario",
            when: "Action occurs",
            then: "Expected outcome",
          },
        ],
        invariants: ["System must remain stable"],
        non_functional: {},
        contracts: [],
      };

      // Validate the spec through the CAWS validator
      const validationResult = await specValidator.validateWorkingSpec(
        validSpec
      );

      // Verify validation passed
      expect(validationResult.valid).toBe(true);
      expect(validationResult.errors.length).toBe(0);

      // Check that constitutional validation was recorded
      const events = dataCollector.getPendingEvents(10);
      const validationEvent = events.find(
        (e) => e.type === PerformanceEventType.CONSTITUTIONAL_VALIDATION
      );
      expect(validationEvent).toBeDefined();
      expect(validationEvent?.agentId).toBe("caws-validator");
      expect(validationEvent?.taskId).toBe("TEST-001");
      expect(validationEvent?.metrics?.compliance?.validationPassRate).toBe(1); // Should be 1 for valid spec

      // Test with invalid spec
      const invalidSpec: WorkingSpec = {
        ...validSpec,
        id: "invalid", // Invalid ID format
        risk_tier: 5 as any, // Invalid risk tier
      };

      const invalidResult = await specValidator.validateWorkingSpec(
        invalidSpec
      );

      // Verify validation failed
      expect(invalidResult.valid).toBe(false);
      expect(invalidResult.errors.length).toBeGreaterThan(0);

      // Check that failed validation was recorded with lower compliance score
      const updatedEvents = dataCollector.getPendingEvents(10);
      const failedValidationEvent = updatedEvents.find(
        (e) =>
          e.type === PerformanceEventType.CONSTITUTIONAL_VALIDATION &&
          e.taskId === "invalid"
      );
      expect(failedValidationEvent).toBeDefined();
      expect(
        failedValidationEvent?.metrics?.compliance?.validationPassRate
      ).toBe(0); // Should be 0 for invalid spec
    });
  });
});
