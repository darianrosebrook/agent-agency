/**
 * Tests for FeedbackPipeline RL integration
 */

import { ConfigManager } from "../../../src/config/ConfigManager";
import { FeedbackPipeline } from "../../../src/feedback-loop/FeedbackPipeline";
import { RLTrainingCoordinator } from "../../../src/rl/RLTrainingCoordinator";
import {
  FeedbackEvent,
  FeedbackSource,
  FeedbackType,
} from "../../../src/types/feedback-loop";

describe("FeedbackPipeline RL Integration", () => {
  let configManager: ConfigManager;
  let feedbackPipeline: FeedbackPipeline;
  let rlTrainingCoordinator: RLTrainingCoordinator;
  let mockRLTrainer: any;

  beforeEach(() => {
    configManager = ConfigManager.getInstance();

    // Create mock RL trainer
    mockRLTrainer = {
      trainOnTrajectories: jest.fn().mockResolvedValue({
        totalSamples: 100,
        averageReward: 0.75,
        trainingTimeMs: 1500,
        episodesProcessed: 10,
        policyLoss: 0.1,
        valueLoss: 0.05,
      }),
      getTrainingStats: jest.fn().mockReturnValue({
        totalSamples: 100,
        averageReward: 0.75,
        trainingTimeMs: 1500,
        episodesProcessed: 10,
        policyLoss: 0.1,
        valueLoss: 0.05,
      }),
    };

    // Create RL training coordinator with mock dependencies
    rlTrainingCoordinator = new RLTrainingCoordinator(
      {
        minDebateOutcomes: 10,
        minDataQualityScore: 0.5,
        maxDataAgeMs: 86400000,
        trainingBatchSize: 16,
        trainingIntervalMs: 3600000,
        enableAutoTraining: true,
        qualityThresholds: {
          minVerdictQuality: 0.6,
          minTurnQuality: 0.5,
          minEvidenceQuality: 0.6,
          minComplianceScore: 0.7,
        },
        performanceDegradationThreshold: 0.1,
      },
      {
        debateTracker: {} as any,
        verdictScorer: {} as any,
        rlTrainer: mockRLTrainer,
        performanceTracker: {} as any,
        dataPipeline: {} as any,
      }
    );

    // Create feedback pipeline with RL training coordinator
    feedbackPipeline = new FeedbackPipeline(
      configManager,
      rlTrainingCoordinator
    );
  });

  describe("sendToTraining", () => {
    it("should send batch to RL training coordinator when available", async () => {
      const mockEvents: FeedbackEvent[] = [
        {
          id: "event-1",
          timestamp: new Date().toISOString(),
          source: FeedbackSource.USER_RATINGS,
          type: FeedbackType.RATING_SCALE,
          entityId: "task-123",
          entityType: "task",
          value: 4,
          context: { taskType: "coding" },
        },
        {
          id: "event-2",
          timestamp: new Date().toISOString(),
          source: FeedbackSource.PERFORMANCE_METRICS,
          type: FeedbackType.NUMERIC_METRIC,
          entityId: "task-123",
          entityType: "task",
          value: 0.85,
          context: { metricType: "success_rate" },
        },
      ];

      // Process batch through pipeline
      const batch = await feedbackPipeline.processBatch(mockEvents);

      // Verify RL training was called
      expect(mockRLTrainer.trainOnTrajectories).toHaveBeenCalledWith(
        expect.arrayContaining([
          expect.objectContaining({
            conversationId: "task-123",
            turns: expect.any(Array),
            finalOutcome: expect.any(String),
            totalReward: expect.any(Number),
          }),
        ])
      );
    });

    it("should handle RL training failures gracefully", async () => {
      // Mock RL trainer to throw error
      mockRLTrainer.trainOnTrajectories.mockRejectedValue(
        new Error("Training failed")
      );

      const mockEvents: FeedbackEvent[] = [
        {
          id: "event-1",
          timestamp: new Date().toISOString(),
          source: FeedbackSource.USER_RATINGS,
          type: FeedbackType.RATING_SCALE,
          entityId: "task-123",
          entityType: "task",
          value: 4,
          context: {},
        },
      ];

      // Process batch should throw error when RL training fails
      const batch = await feedbackPipeline.processBatch(mockEvents);

      // The sendToTraining method should be called and should throw
      await expect(feedbackPipeline["sendToTraining"](batch)).rejects.toThrow(
        "Training system error: Training failed"
      );
    });

    it("should fall back to simulation when no RL coordinator provided", async () => {
      // Create pipeline without RL coordinator
      const pipelineWithoutRL = new FeedbackPipeline(configManager);

      const mockEvents: FeedbackEvent[] = [
        {
          id: "event-1",
          timestamp: new Date().toISOString(),
          source: FeedbackSource.USER_RATINGS,
          type: FeedbackType.RATING_SCALE,
          entityId: "task-123",
          entityType: "task",
          value: 4,
          context: {},
        },
      ];

      const batch = await pipelineWithoutRL.processBatch(mockEvents);

      // Should complete without errors (simulation mode)
      await expect(
        pipelineWithoutRL["sendToTraining"](batch)
      ).resolves.not.toThrow();
    });

    it("should convert feedback events to valid trajectories", async () => {
      const mockEvents: FeedbackEvent[] = [
        {
          id: "event-1",
          timestamp: new Date().toISOString(),
          source: FeedbackSource.USER_RATINGS,
          type: FeedbackType.BINARY_OUTCOME,
          entityId: "task-123",
          entityType: "task",
          value: true,
          context: { outcome: "success" },
        },
        {
          id: "event-2",
          timestamp: new Date().toISOString(),
          source: FeedbackSource.PERFORMANCE_METRICS,
          type: FeedbackType.NUMERIC_METRIC,
          entityId: "task-123",
          entityType: "task",
          value: 0.9,
          context: { metricType: "performance" },
        },
      ];

      const batch = await feedbackPipeline.processBatch(mockEvents);
      const trajectories =
        feedbackPipeline["convertBatchToTrajectories"](batch);

      expect(trajectories).toHaveLength(1);
      expect(trajectories[0]).toEqual({
        conversationId: "task-123",
        turns: expect.arrayContaining([
          expect.objectContaining({
            turnNumber: 1,
            reward: 1, // binary_outcome: true = 1
            actionLogProbs: [],
            valueEstimate: 0,
            advantages: [],
          }),
          expect.objectContaining({
            turnNumber: 2,
            reward: 0.9, // numeric_metric: 0.9
            actionLogProbs: [],
            valueEstimate: 0,
            advantages: [],
          }),
        ]),
        finalOutcome: expect.any(String),
        totalReward: 1.9, // 1 + 0.9
      });
    });

    it("should extract rewards correctly from different event types", () => {
      const testCases = [
        {
          event: {
            type: FeedbackType.RATING_SCALE,
            value: 5,
          },
          expectedReward: 1, // (5-3)/2 = 1
        },
        {
          event: {
            type: FeedbackType.RATING_SCALE,
            value: 3,
          },
          expectedReward: 0, // (3-3)/2 = 0
        },
        {
          event: {
            type: FeedbackType.RATING_SCALE,
            value: 1,
          },
          expectedReward: -1, // (1-3)/2 = -1
        },
        {
          event: {
            type: FeedbackType.BINARY_OUTCOME,
            value: true,
          },
          expectedReward: 1,
        },
        {
          event: {
            type: FeedbackType.BINARY_OUTCOME,
            value: false,
          },
          expectedReward: -1,
        },
        {
          event: {
            type: FeedbackType.NUMERIC_METRIC,
            value: 0.85,
          },
          expectedReward: 0.85,
        },
        {
          event: {
            type: FeedbackType.CATEGORICAL_EVENT,
            value: "success",
          },
          expectedReward: 1,
        },
        {
          event: {
            type: FeedbackType.CATEGORICAL_EVENT,
            value: "failure",
          },
          expectedReward: -1,
        },
      ];

      for (const testCase of testCases) {
        const reward = feedbackPipeline["extractRewardFromEvent"](
          testCase.event as any
        );
        expect(reward).toBe(testCase.expectedReward);
      }
    });

    it("should emit training completion events", async () => {
      const mockEvents: FeedbackEvent[] = [
        {
          id: "event-1",
          timestamp: new Date().toISOString(),
          source: FeedbackSource.USER_RATINGS,
          type: FeedbackType.RATING_SCALE,
          entityId: "task-123",
          entityType: "task",
          value: 4,
          context: {},
        },
      ];

      const batch = await feedbackPipeline.processBatch(mockEvents);

      // Listen for training completion event
      const trainingCompletedPromise = new Promise((resolve) => {
        feedbackPipeline.once("training_completed", resolve);
      });

      // Trigger training
      await feedbackPipeline["sendToTraining"](batch);

      // Should emit training completion event
      const eventData = await trainingCompletedPromise;
      expect(eventData).toEqual({
        batchId: batch.id,
        trainingStats: expect.objectContaining({
          totalSamples: 100,
          averageReward: 0.75,
          trainingTimeMs: 1500,
        }),
        trajectoriesProcessed: 1,
      });
    });
  });

  describe("convertBatchToTrajectories", () => {
    it("should handle empty batches", () => {
      const batch = {
        id: "empty-batch",
        timestamp: new Date().toISOString(),
        events: [],
        features: {},
        qualityScore: 0,
        metadata: {
          batchSize: 0,
          timeRange: { start: "", end: "" },
          entityTypes: [],
          sources: [],
        },
      };

      const trajectories =
        feedbackPipeline["convertBatchToTrajectories"](batch);
      expect(trajectories).toHaveLength(0);
    });

    it("should group events by entity ID", () => {
      const batch = {
        id: "multi-entity-batch",
        timestamp: new Date().toISOString(),
        events: [
          {
            id: "event-1",
            entityId: "task-123",
            type: FeedbackType.RATING_SCALE,
            value: 4,
          } as any,
          {
            id: "event-2",
            entityId: "task-456",
            type: FeedbackType.RATING_SCALE,
            value: 3,
          } as any,
          {
            id: "event-3",
            entityId: "task-123",
            type: FeedbackType.NUMERIC_METRIC,
            value: 0.8,
          } as any,
        ],
        features: {},
        qualityScore: 0.75,
        metadata: {
          batchSize: 3,
          timeRange: { start: "", end: "" },
          entityTypes: ["task"],
          sources: [],
        },
      };

      const trajectories =
        feedbackPipeline["convertBatchToTrajectories"](batch);
      expect(trajectories).toHaveLength(2); // Two different entity IDs

      const task123Trajectory = trajectories.find(
        (t) => t.conversationId === "task-123"
      );
      const task456Trajectory = trajectories.find(
        (t) => t.conversationId === "task-456"
      );

      expect(task123Trajectory?.turns).toHaveLength(2); // Two events for task-123
      expect(task456Trajectory?.turns).toHaveLength(1); // One event for task-456
    });
  });
});
