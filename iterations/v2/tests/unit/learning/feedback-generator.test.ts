/**
 * @fileoverview Unit tests for FeedbackGenerator
 *
 * Tests feedback generation, recommendation creation, and historical tracking.
 *
 * @author @darianrosebrook
 */

import {
  FeedbackContext,
  FeedbackGenerator,
} from "../../../src/learning/FeedbackGenerator";
import { LearningIteration } from "../../../src/types/learning-coordination";

describe("FeedbackGenerator", () => {
  let generator: FeedbackGenerator;

  beforeEach(() => {
    generator = new FeedbackGenerator();
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  describe("initialization", () => {
    it("should initialize with empty state", () => {
      expect(generator).toBeDefined();
      expect(generator).toBeInstanceOf(FeedbackGenerator);
    });
  });

  describe("feedback generation", () => {
    it("should generate feedback for low quality iterations", async () => {
      const context: FeedbackContext = {
        currentIteration: {
          iterationId: "iter-1",
          sessionId: "session-1",
          iterationNumber: 1,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 100,
          contextSnapshotId: "ctx-1",
          errorDetected: false,
          qualityScore: 0.6, // Below threshold
          improvementDelta: 0.1,
          resourceUsageMB: 10,
          promptModifications: [],
        },
        previousIterations: [],
        qualityThreshold: 0.8,
        errorPatterns: [],
      };

      const feedback = await generator.generateFeedback(context);

      expect(feedback).toBeDefined();
      expect(feedback.iterationId).toBe("iter-1");
      expect(feedback.type).toBeDefined();
      expect(Array.isArray(feedback.recommendations)).toBe(true);
      expect(typeof feedback.confidence).toBe("number");
      expect(Array.isArray(feedback.successPatterns)).toBe(true);
    });

    it("should generate feedback for iterations with errors", async () => {
      const context: FeedbackContext = {
        currentIteration: {
          iterationId: "iter-2",
          sessionId: "session-2",
          iterationNumber: 1,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 100,
          contextSnapshotId: "ctx-2",
          errorDetected: true,
          errorCategory: "logic_error" as any,
          qualityScore: 0.7,
          improvementDelta: -0.05,
          resourceUsageMB: 12,
          promptModifications: [],
        },
        previousIterations: [],
        qualityThreshold: 0.8,
        errorPatterns: ["logic_error"],
      };

      const feedback = await generator.generateFeedback(context);

      expect(feedback).toBeDefined();
      expect(feedback.iterationId).toBe("iter-2");
      expect(feedback.recommendations.length).toBeGreaterThan(0);
    });

    it("should analyze trends across iterations", async () => {
      const previousIterations: LearningIteration[] = [
        {
          iterationId: "iter-1",
          sessionId: "session-3",
          iterationNumber: 1,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 100,
          contextSnapshotId: "ctx-1",
          errorDetected: false,
          qualityScore: 0.8,
          improvementDelta: 0.1,
          resourceUsageMB: 10,
          promptModifications: [],
        },
        {
          iterationId: "iter-2",
          sessionId: "session-3",
          iterationNumber: 2,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 110,
          contextSnapshotId: "ctx-2",
          errorDetected: false,
          qualityScore: 0.75,
          improvementDelta: -0.05,
          resourceUsageMB: 12,
          promptModifications: [],
        },
      ];

      const context: FeedbackContext = {
        currentIteration: {
          iterationId: "iter-3",
          sessionId: "session-3",
          iterationNumber: 3,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 120,
          contextSnapshotId: "ctx-3",
          errorDetected: false,
          qualityScore: 0.85,
          improvementDelta: 0.1,
          resourceUsageMB: 15,
          promptModifications: [],
        },
        previousIterations,
        qualityThreshold: 0.8,
        errorPatterns: [],
      };

      const feedback = await generator.generateFeedback(context);

      expect(feedback).toBeDefined();
      expect(feedback.recommendations.length).toBeGreaterThan(0);
    });

    it("should handle empty previous iterations", async () => {
      const context: FeedbackContext = {
        currentIteration: {
          iterationId: "iter-4",
          sessionId: "session-4",
          iterationNumber: 1,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 100,
          contextSnapshotId: "ctx-4",
          errorDetected: false,
          qualityScore: 0.9,
          improvementDelta: 0.1,
          resourceUsageMB: 10,
          promptModifications: [],
        },
        previousIterations: [],
        qualityThreshold: 0.8,
        errorPatterns: [],
      };

      const feedback = await generator.generateFeedback(context);

      expect(feedback).toBeDefined();
      expect(feedback.iterationId).toBe("iter-4");
      expect(Array.isArray(feedback.recommendations)).toBe(true);
    });
  });

  describe("feedback history", () => {
    it("should track feedback history by session", async () => {
      const context: FeedbackContext = {
        currentIteration: {
          iterationId: "iter-1",
          sessionId: "session-history",
          iterationNumber: 1,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 100,
          contextSnapshotId: "ctx-1",
          errorDetected: false,
          qualityScore: 0.8,
          improvementDelta: 0.1,
          resourceUsageMB: 10,
          promptModifications: [],
        },
        previousIterations: [],
        qualityThreshold: 0.8,
        errorPatterns: [],
      };

      await generator.generateFeedback(context);

      const history = generator.getFeedbackHistory("session-history");
      expect(Array.isArray(history)).toBe(true);
      expect(history.length).toBe(1);
      expect(history[0].iterationId).toBe("iter-1");
    });

    it("should return empty array for unknown sessions", () => {
      const history = generator.getFeedbackHistory("unknown-session");
      expect(Array.isArray(history)).toBe(true);
      expect(history.length).toBe(0);
    });
  });

  describe("statistics", () => {
    it("should provide feedback statistics", async () => {
      const context: FeedbackContext = {
        currentIteration: {
          iterationId: "iter-stats",
          sessionId: "session-stats",
          iterationNumber: 1,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 100,
          contextSnapshotId: "ctx-stats",
          errorDetected: false,
          qualityScore: 0.8,
          improvementDelta: 0.1,
          resourceUsageMB: 10,
          promptModifications: [],
        },
        previousIterations: [],
        qualityThreshold: 0.8,
        errorPatterns: [],
      };

      await generator.generateFeedback(context);

      const stats = generator.getStatistics("session-stats");

      expect(stats).toBeDefined();
      expect(typeof stats.totalFeedback).toBe("number");
      expect(typeof stats.averageConfidence).toBe("number");
      expect(typeof stats.totalRecommendations).toBe("number");
    });

    it("should return zero stats for unknown sessions", () => {
      const stats = generator.getStatistics("unknown-session");

      expect(stats).toBeDefined();
      expect(stats.totalFeedback).toBe(0);
      expect(stats.averageConfidence).toBe(0);
      expect(stats.totalRecommendations).toBe(0);
    });
  });

  describe("cleanup", () => {
    it("should cleanup session data", async () => {
      const context: FeedbackContext = {
        currentIteration: {
          iterationId: "iter-cleanup",
          sessionId: "session-cleanup",
          iterationNumber: 1,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 100,
          contextSnapshotId: "ctx-cleanup",
          errorDetected: false,
          qualityScore: 0.8,
          improvementDelta: 0.1,
          resourceUsageMB: 10,
          promptModifications: [],
        },
        previousIterations: [],
        qualityThreshold: 0.8,
        errorPatterns: [],
      };

      await generator.generateFeedback(context);

      // Verify data exists
      let history = generator.getFeedbackHistory("session-cleanup");
      expect(history.length).toBe(1);

      // Cleanup
      generator.cleanup("session-cleanup");

      // Verify cleanup
      history = generator.getFeedbackHistory("session-cleanup");
      expect(history.length).toBe(0);
    });
  });
});
