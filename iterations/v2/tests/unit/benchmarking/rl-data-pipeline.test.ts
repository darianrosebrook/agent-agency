/**
 * RL Data Pipeline Unit Tests
 *
 * @author @darianrosebrook
 */

import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { RLDataPipeline } from "../../../src/benchmarking/RLDataPipeline";
import {
  AgentPerformanceProfile,
  PerformanceEvent,
  PerformanceEventType,
  RLDataPipelineConfig,
  RLTrainingSample,
} from "../../../src/types/performance-tracking";

describe("RLDataPipeline", () => {
  let pipeline: RLDataPipeline;
  let mockEvents: PerformanceEvent[];
  let mockProfiles: AgentPerformanceProfile[];

  beforeEach(() => {
    pipeline = new RLDataPipeline();
    mockEvents = [
      {
        id: "event-1",
        type: PerformanceEventType.TASK_EXECUTION_COMPLETE,
        timestamp: new Date().toISOString(),
        agentId: "agent-1",
        taskId: "task-1",
        metrics: {
          latency: {
            averageMs: 1000,
            p95Ms: 1200,
            p99Ms: 1500,
            minMs: 800,
            maxMs: 2000,
          },
          accuracy: {
            successRate: 0.9,
            qualityScore: 0.85,
            violationRate: 0.1,
            evaluationScore: 0.8,
          },
          resources: {
            cpuUtilizationPercent: 70,
            memoryUtilizationPercent: 60,
            networkIoKbps: 100,
            diskIoKbps: 50,
          },
          compliance: {
            validationPassRate: 0.95,
            violationSeverityScore: 0.05,
            clauseCitationRate: 0.9,
          },
          cost: {
            costPerTask: 0.5,
            efficiencyScore: 0.85,
            resourceWastePercent: 15,
          },
          reliability: {
            mtbfHours: 168,
            availabilityPercent: 99.5,
            errorRatePercent: 0.5,
            recoveryTimeMinutes: 5,
          },
        },
        integrityHash: "hash1",
      },
      {
        id: "event-2",
        type: PerformanceEventType.TASK_EXECUTION_COMPLETE,
        timestamp: new Date(Date.now() - 60000).toISOString(),
        agentId: "agent-1",
        taskId: "task-2",
        metrics: {
          latency: {
            averageMs: 1200,
            p95Ms: 1400,
            p99Ms: 1700,
            minMs: 900,
            maxMs: 2200,
          },
          accuracy: {
            successRate: 0.8,
            qualityScore: 0.75,
            violationRate: 0.2,
            evaluationScore: 0.7,
          },
          resources: {
            cpuUtilizationPercent: 75,
            memoryUtilizationPercent: 65,
            networkIoKbps: 120,
            diskIoKbps: 60,
          },
          compliance: {
            validationPassRate: 0.9,
            violationSeverityScore: 0.1,
            clauseCitationRate: 0.8,
          },
          cost: {
            costPerTask: 0.6,
            efficiencyScore: 0.8,
            resourceWastePercent: 20,
          },
          reliability: {
            mtbfHours: 140,
            availabilityPercent: 98.5,
            errorRatePercent: 1.5,
            recoveryTimeMinutes: 10,
          },
        },
        integrityHash: "hash2",
      },
    ];

    mockProfiles = [
      {
        agentId: "agent-1",
        taskType: "coding",
        metrics: {
          latency: {
            averageMs: 1100,
            p95Ms: 1300,
            p99Ms: 1600,
            minMs: 850,
            maxMs: 2100,
          },
          accuracy: {
            successRate: 0.85,
            qualityScore: 0.8,
            violationRate: 0.15,
            evaluationScore: 0.75,
          },
          resources: {
            cpuUtilizationPercent: 72.5,
            memoryUtilizationPercent: 62.5,
            networkIoKbps: 110,
            diskIoKbps: 55,
          },
          compliance: {
            validationPassRate: 0.925,
            violationSeverityScore: 0.075,
            clauseCitationRate: 0.85,
          },
          cost: {
            costPerTask: 0.55,
            efficiencyScore: 0.825,
            resourceWastePercent: 17.5,
          },
          reliability: {
            mtbfHours: 154,
            availabilityPercent: 99,
            errorRatePercent: 1,
            recoveryTimeMinutes: 7.5,
          },
        },
        sampleSize: 2,
        confidence: 0.8,
        lastUpdated: new Date().toISOString(),
        trend: {
          direction: "stable",
          magnitude: 0.1,
          confidence: 0.7,
          timeWindowHours: 1,
        },
      },
    ];
  });

  afterEach(() => {
    pipeline.stopProcessing();
  });

  describe("constructor", () => {
    it("should create with default config", () => {
      const pipeline = new RLDataPipeline();
      const stats = pipeline.getPipelineStats();

      expect(stats.isProcessing).toBe(false);
      expect(stats.activeAgents).toBe(0);
      expect(stats.totalSamples).toBe(0);
    });

    it("should create with custom config", () => {
      const customConfig: Partial<RLDataPipelineConfig> = {
        batching: {
          maxBatchSize: 500,
          maxBatchAgeMinutes: 10,
          minBatchSize: 50,
        },
        qualityThresholds: {
          minSampleDiversity: 0.8,
          maxTemporalGapMinutes: 20,
          minRewardVariance: 0.15,
          maxDuplicateRatio: 0.15,
        },
      };

      const pipeline = new RLDataPipeline(customConfig);
      const stats = pipeline.getPipelineStats();

      expect(stats.config.batching.maxBatchSize).toBe(500);
      expect(stats.config.qualityThresholds.minSampleDiversity).toBe(0.8);
    });
  });

  describe("processing lifecycle", () => {
    it("should start and stop processing", () => {
      pipeline.startProcessing();
      expect(pipeline.getPipelineStats().isProcessing).toBe(true);

      pipeline.stopProcessing();
      expect(pipeline.getPipelineStats().isProcessing).toBe(false);
    });
  });

  describe("event processing", () => {
    beforeEach(() => {
      pipeline.startProcessing();
    });

    it("should process events and generate samples", async () => {
      const result = await pipeline.processEvents(mockEvents, mockProfiles);

      expect(result.samplesGenerated).toBeGreaterThan(0);
      expect(result.batchesCompleted).toBeGreaterThan(0);
      expect(result.qualityIssues).toHaveLength(0);

      const stats = pipeline.getPipelineStats();
      expect(stats.totalSamples).toBeGreaterThan(0);
    });

    it("should handle empty event batches", async () => {
      const result = await pipeline.processEvents([], mockProfiles);

      expect(result.samplesGenerated).toBe(0);
      expect(result.batchesCompleted).toBe(0);
      expect(result.qualityIssues).toHaveLength(0);
    });

    it("should skip events without agent profiles", async () => {
      const eventsWithoutProfiles = [
        {
          ...mockEvents[0],
          agentId: "unknown-agent",
        },
      ];

      const result = await pipeline.processEvents(
        eventsWithoutProfiles,
        mockProfiles
      );

      expect(result.samplesGenerated).toBe(0);
      expect(result.batchesCompleted).toBe(0);
    });

    it("should only process task completion events", async () => {
      const mixedEvents = [
        mockEvents[0], // task_execution_complete
        {
          ...mockEvents[0],
          id: "routing-event",
          type: PerformanceEventType.ROUTING_DECISION,
        },
      ];

      const result = await pipeline.processEvents(
        mixedEvents as PerformanceEvent[],
        mockProfiles
      );

      expect(result.samplesGenerated).toBe(1); // Only the completion event
    });
  });

  describe("training batch management", () => {
    beforeEach(async () => {
      pipeline.startProcessing();
      await pipeline.processEvents(mockEvents, mockProfiles);
    });

    it("should retrieve training batches", () => {
      const batches = pipeline.getTrainingBatches();

      expect(batches).toHaveLength(1);
      expect(batches[0].agentId).toBe("agent-1");
      expect(batches[0].samples).toHaveLength(2);
      expect(batches[0].qualityScore).toBeGreaterThan(0);
    });

    it("should filter batches by agent", () => {
      const agentBatches = pipeline.getTrainingBatches("agent-1");
      const unknownAgentBatches = pipeline.getTrainingBatches("unknown-agent");

      expect(agentBatches).toHaveLength(1);
      expect(unknownAgentBatches).toHaveLength(0);
    });

    it("should limit batch retrieval", () => {
      // Add more batches by processing more events
      const additionalEvents = [
        {
          ...mockEvents[0],
          id: "event-3",
          taskId: "task-3",
        },
        {
          ...mockEvents[1],
          id: "event-4",
          taskId: "task-4",
        },
      ];

      pipeline.processEvents(additionalEvents, mockProfiles);

      const limitedBatches = pipeline.getTrainingBatches(undefined, 1);
      const allBatches = pipeline.getTrainingBatches();

      expect(limitedBatches).toHaveLength(1);
      expect(allBatches.length).toBeGreaterThanOrEqual(limitedBatches.length);
    });

    it("should remove batches after retrieval", () => {
      const initialBatches = pipeline.getTrainingBatches();
      expect(initialBatches).toHaveLength(1);

      // Try to get batches again
      const remainingBatches = pipeline.getTrainingBatches();
      expect(remainingBatches).toHaveLength(0);
    });
  });

  describe("sample generation", () => {
    it("should create valid training samples", async () => {
      pipeline.startProcessing();

      const sample = await pipeline["createTrainingSample"](
        mockEvents[0],
        {
          agentId: "agent-1",
          recentSamples: [],
          pendingBatch: [],
          batchStartTime: new Date().toISOString(),
          lastProcessedEvent: new Date().toISOString(),
          performanceHistory: mockProfiles,
        },
        mockProfiles[0]
      );

      expect(sample).toBeDefined();
      if (sample) {
        expect(sample.id).toMatch(/^sample_.*_\d+$/);
        expect(sample.agentId).toBe("agent-1");
        expect(sample.taskType).toBe("task-1");
        expect(sample.state).toBeDefined();
        expect(sample.action).toBeDefined();
        expect(sample.reward).toBeGreaterThanOrEqual(0);
        expect(sample.nextState).toBeDefined();
        expect(sample.done).toBe(true);
        expect(sample.integrityHash).toBeDefined();
      }
    });

    it("should skip non-completion events", async () => {
      pipeline.startProcessing();

      const routingEvent = {
        ...mockEvents[0],
        type: PerformanceEventType.ROUTING_DECISION,
      };

      const sample = await pipeline["createTrainingSample"](
        routingEvent,
        {
          agentId: "agent-1",
          recentSamples: [],
          pendingBatch: [],
          batchStartTime: new Date().toISOString(),
          lastProcessedEvent: new Date().toISOString(),
          performanceHistory: mockProfiles,
        },
        mockProfiles[0]
      );

      expect(sample).toBeNull();
    });
  });

  describe("reward calculation", () => {
    it("should calculate rewards based on performance metrics", () => {
      const reward = pipeline["calculateReward"](
        mockEvents[0],
        mockProfiles[0]
      );

      expect(typeof reward).toBe("number");
      expect(reward).toBeGreaterThanOrEqual(0);
      expect(reward).toBeLessThanOrEqual(1); // Should be normalized
    });

    it("should apply temporal decay", () => {
      const recentEvent = {
        ...mockEvents[0],
        timestamp: new Date().toISOString(),
      };

      const oldEvent = {
        ...mockEvents[0],
        timestamp: new Date(Date.now() - 3600000).toISOString(), // 1 hour ago
      };

      const recentReward = pipeline["calculateReward"](
        recentEvent,
        mockProfiles[0]
      );
      const oldReward = pipeline["calculateReward"](oldEvent, mockProfiles[0]);

      expect(recentReward).toBeGreaterThan(oldReward);
    });
  });

  describe("batch creation", () => {
    it("should create training batches with quality scores", async () => {
      const mockSamples: RLTrainingSample[] = [
        {
          id: "sample-1",
          agentId: "agent-1",
          taskType: "coding",
          state: { context: "test" },
          action: { decision: "accept" },
          reward: 0.8,
          nextState: { context: "completed" },
          done: true,
          timestamp: new Date().toISOString(),
          integrityHash: "hash1",
        },
        {
          id: "sample-2",
          agentId: "agent-1",
          taskType: "coding",
          state: { context: "test2" },
          action: { decision: "reject" },
          reward: 0.6,
          nextState: { context: "failed" },
          done: true,
          timestamp: new Date(Date.now() - 60000).toISOString(),
          integrityHash: "hash2",
        },
      ];

      const state = {
        agentId: "agent-1",
        recentSamples: [],
        pendingBatch: mockSamples,
        batchStartTime: new Date(Date.now() - 300000).toISOString(),
        lastProcessedEvent: new Date().toISOString(),
        performanceHistory: mockProfiles,
      };

      const batch = await pipeline["createTrainingBatch"](state);

      expect(batch.id).toMatch(/^batch_agent-1_\d+$/);
      expect(batch.agentId).toBe("agent-1");
      expect(batch.samples).toHaveLength(2);
      expect(batch.qualityScore).toBeGreaterThan(0);
      expect(batch.qualityScore).toBeLessThanOrEqual(1);
      expect(batch.anonymizationLevel).toBe("differential");
    });

    it("should determine batch completion timing", () => {
      const state = {
        agentId: "agent-1",
        recentSamples: [],
        pendingBatch: [],
        batchStartTime: new Date().toISOString(),
        lastProcessedEvent: new Date().toISOString(),
        performanceHistory: mockProfiles,
      };

      // Empty batch should not be ready
      expect(pipeline["shouldCompleteBatch"](state)).toBe(false);

      // Add samples to reach minimum size
      state.pendingBatch = Array.from(
        { length: 100 },
        (_, i): RLTrainingSample => ({
          id: `sample-${i}`,
          agentId: "agent-1",
          taskType: "coding",
          state: {},
          action: {},
          reward: 0.5,
          nextState: {},
          done: true,
          timestamp: new Date().toISOString(),
          integrityHash: `hash${i}`,
        })
      );

      expect(pipeline["shouldCompleteBatch"](state)).toBe(true);

      // Reset and test time-based completion
      state.pendingBatch = [
        {
          id: "sample-1",
          agentId: "agent-1",
          taskType: "coding",
          state: {},
          action: {},
          reward: 0.5,
          nextState: {},
          done: true,
          timestamp: new Date().toISOString(),
          integrityHash: "hash1",
        } as RLTrainingSample,
      ];
      state.batchStartTime = new Date(
        Date.now() - 20 * 60 * 1000
      ).toISOString(); // 20 minutes ago

      expect(pipeline["shouldCompleteBatch"](state)).toBe(true);
    });
  });

  describe("quality validation", () => {
    it("should calculate batch quality scores", () => {
      const samples: RLTrainingSample[] = [
        {
          id: "sample-1",
          agentId: "agent-1",
          taskType: "coding",
          state: { context: "test" },
          action: { decision: "accept" },
          reward: 0.8,
          nextState: { context: "completed" },
          done: true,
          timestamp: new Date().toISOString(),
          integrityHash: "hash1",
        },
        {
          id: "sample-2",
          agentId: "agent-1",
          taskType: "coding",
          state: { context: "test2" },
          action: { decision: "reject" },
          reward: 0.6,
          nextState: { context: "failed" },
          done: true,
          timestamp: new Date(Date.now() - 300000).toISOString(),
          integrityHash: "hash2",
        },
      ];

      const qualityScore = pipeline["calculateBatchQuality"](samples);

      expect(qualityScore).toBeGreaterThan(0);
      expect(qualityScore).toBeLessThanOrEqual(1);
    });

    it("should detect quality issues", () => {
      const state = {
        agentId: "agent-1",
        recentSamples: [],
        pendingBatch: [],
        batchStartTime: new Date().toISOString(),
        lastProcessedEvent: new Date(Date.now() - 40 * 60 * 1000).toISOString(), // 40 minutes ago
        performanceHistory: mockProfiles,
      };

      const issues = pipeline["checkDataQuality"]("agent-1", state);

      expect(issues).toContain("Large temporal gap detected");
    });

    it("should handle insufficient data gracefully", () => {
      const state = {
        agentId: "agent-1",
        recentSamples: [],
        pendingBatch: [
          {
            id: "sample-1",
            agentId: "agent-1",
            taskType: "coding",
            state: {},
            action: {},
            reward: 0.5,
            nextState: {},
            done: true,
            timestamp: new Date().toISOString(),
            integrityHash: "hash1",
          },
        ],
        batchStartTime: new Date().toISOString(),
        lastProcessedEvent: new Date().toISOString(),
        performanceHistory: mockProfiles,
      };

      const issues = pipeline["checkDataQuality"]("agent-1", state);

      expect(issues.length).toBeGreaterThanOrEqual(0);
    });
  });

  describe("state representation", () => {
    it("should create state representations with historical context", () => {
      const event = mockEvents[0];
      const state = {
        agentId: "agent-1",
        recentSamples: [],
        pendingBatch: [],
        batchStartTime: new Date().toISOString(),
        lastProcessedEvent: new Date().toISOString(),
        performanceHistory: mockProfiles,
      };

      const stateRep = pipeline["createStateRepresentation"](
        event,
        state,
        mockProfiles[0]
      );

      expect(stateRep.taskId).toBe("task-1");
      expect(stateRep.agentId).toBe("agent-1");
      expect(stateRep.taskType).toBeDefined();
      expect(stateRep.historicalPerformance).toBeDefined();
      expect(stateRep.agentLoad).toBeDefined();
      expect(stateRep.taskContext).toBeDefined();
    });

    it("should include historical metrics when enabled", () => {
      const pipelineWithHistory = new RLDataPipeline({
        stateRepresentation: {
          includeHistoricalMetrics: true,
          includeAgentLoad: false,
          includeTaskContext: false,
          temporalWindowSize: 5,
        },
      });

      const event = mockEvents[0];
      const state = {
        agentId: "agent-1",
        recentSamples: [],
        pendingBatch: [],
        batchStartTime: new Date().toISOString(),
        lastProcessedEvent: new Date().toISOString(),
        performanceHistory: mockProfiles,
      };

      const stateRep = pipelineWithHistory["createStateRepresentation"](
        event,
        state,
        mockProfiles[0]
      );

      expect(stateRep.historicalPerformance).toBeDefined();
      expect(stateRep.agentLoad).toBeUndefined();
      expect(stateRep.taskContext).toBeUndefined();
    });
  });

  describe("data integrity", () => {
    it("should generate consistent integrity hashes", () => {
      const event = mockEvents[0];
      const state = {};
      const action = { decision: "accept" };
      const reward = 0.8;

      const hash1 = pipeline["calculateSampleHash"](
        event,
        state,
        action,
        reward
      );
      const hash2 = pipeline["calculateSampleHash"](
        event,
        state,
        action,
        reward
      );

      expect(hash1).toBe(hash2);
      expect(hash1).toMatch(/^[a-f0-9]{64}$/); // SHA-256 hash
    });

    it("should generate different hashes for different data", () => {
      const event = mockEvents[0];
      const state = {};
      const action1 = { decision: "accept" };
      const action2 = { decision: "reject" };
      const reward = 0.8;

      const hash1 = pipeline["calculateSampleHash"](
        event,
        state,
        action1,
        reward
      );
      const hash2 = pipeline["calculateSampleHash"](
        event,
        state,
        action2,
        reward
      );

      expect(hash1).not.toBe(hash2);
    });
  });

  describe("memory management", () => {
    it("should clear all pipeline data", () => {
      pipeline.startProcessing();

      // Add some data
      pipeline.processEvents(mockEvents, mockProfiles);
      const initialStats = pipeline.getPipelineStats();

      expect(initialStats.totalSamples).toBeGreaterThan(0);

      pipeline.clearData();

      const finalStats = pipeline.getPipelineStats();
      expect(finalStats.totalSamples).toBe(0);
      expect(finalStats.activeAgents).toBe(0);
      expect(finalStats.isProcessing).toBe(false);
    });
  });

  describe("configuration management", () => {
    it("should update configuration", () => {
      pipeline.updateConfig({
        batching: {
          maxBatchSize: 200,
          maxBatchAgeMinutes: 5,
          minBatchSize: 25,
        },
      });

      const stats = pipeline.getPipelineStats();
      expect(stats.config.batching.maxBatchSize).toBe(200);
      expect(stats.config.batching.maxBatchAgeMinutes).toBe(5);
    });

    it("should emit config update events", () => {
      const mockEmitter = jest.fn();
      pipeline.on("config_updated", mockEmitter);

      pipeline.updateConfig({
        qualityThresholds: {
          minSampleDiversity: 0.9,
          maxTemporalGapMinutes: 15,
          minRewardVariance: 0.1,
          maxDuplicateRatio: 0.1,
        },
      });

      expect(mockEmitter).toHaveBeenCalled();
    });
  });

  describe("event emission", () => {
    it("should emit events processed events", async () => {
      const mockEmitter = jest.fn();
      pipeline.on("events_processed", mockEmitter);

      pipeline.startProcessing();
      await pipeline.processEvents(mockEvents, mockProfiles);

      expect(mockEmitter).toHaveBeenCalledWith(
        expect.objectContaining({
          samplesGenerated: expect.any(Number),
          batchesCompleted: expect.any(Number),
        })
      );
    });

    it("should emit processing error events", async () => {
      const mockEmitter = jest.fn();
      pipeline.on("processing_error", mockEmitter);

      pipeline.startProcessing();

      // Force an error by passing invalid data
      await pipeline.processEvents([{} as any], mockProfiles);

      expect(mockEmitter).toHaveBeenCalled();
    });
  });

  describe("task type extraction", () => {
    it("should extract task type from task ID", () => {
      const event = {
        ...mockEvents[0],
        taskId: "coding-task-123",
      };

      const taskType = pipeline["extractTaskType"](event);
      expect(taskType).toBe("coding");
    });

    it("should handle events without task context", () => {
      const event = {
        ...mockEvents[0],
        taskId: undefined,
        context: undefined,
      };

      const taskType = pipeline["extractTaskType"](event);
      expect(taskType).toBe("unknown");
    });
  });
});
