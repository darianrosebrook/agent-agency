/**
 * Unit Tests: FeedbackGenerator
 *
 * Tests feedback generation, recommendation prioritization,
 * confidence scoring, and success pattern tracking.
 *
 * @author @darianrosebrook
 */

import {
  FeedbackGenerator,
  type FeedbackContext,
} from "../../../src/learning/FeedbackGenerator.js";
import {
  FeedbackType,
  LearningCoordinatorEvent,
  RecommendationPriority,
  type LearningIteration,
} from "../../../src/types/learning-coordination.js";

describe("FeedbackGenerator", () => {
  let generator: FeedbackGenerator;

  beforeEach(() => {
    generator = new FeedbackGenerator();
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  const createIteration = (
    overrides?: Partial<LearningIteration>
  ): LearningIteration => ({
    iterationId: "iter-1",
    sessionId: "session-1",
    iterationNumber: 1,
    startTime: new Date(),
    endTime: new Date(),
    durationMs: 1000,
    contextSnapshotId: "snap-1",
    errorDetected: false,
    qualityScore: 0.7,
    improvementDelta: 0.1,
    resourceUsageMB: 100,
    promptModifications: [],
    ...overrides,
  });

  describe("Feedback Generation", () => {
    it("should generate feedback for successful iteration", async () => {
      const context: FeedbackContext = {
        currentIteration: createIteration({
          qualityScore: 0.9,
          improvementDelta: 0.2,
        }),
        previousIterations: [],
        qualityThreshold: 0.8,
        errorPatterns: [],
      };

      const feedback = await generator.generateFeedback(context);

      expect(feedback.feedbackId).toBeDefined();
      expect(feedback.type).toBe(FeedbackType.QUALITY_ENHANCEMENT);
      expect(feedback.confidence).toBeGreaterThan(0);
      expect(feedback.confidence).toBeLessThanOrEqual(1);
      expect(feedback.recommendations).toBeDefined();
    });

    it("should generate feedback for underperforming iteration", async () => {
      const context: FeedbackContext = {
        currentIteration: createIteration({
          qualityScore: 0.3,
          improvementDelta: -0.1,
        }),
        previousIterations: [],
        qualityThreshold: 0.8,
        errorPatterns: [],
      };

      const feedback = await generator.generateFeedback(context);

      expect(feedback.type).toBe(FeedbackType.PERFORMANCE_IMPROVEMENT);
      expect(feedback.recommendations.length).toBeGreaterThan(0);
    });

    it("should generate feedback for error iteration", async () => {
      const context: FeedbackContext = {
        currentIteration: createIteration({
          errorDetected: true,
          qualityScore: 0.2,
        }),
        previousIterations: [],
        qualityThreshold: 0.8,
        errorPatterns: ["TypeError", "SyntaxError"],
      };

      const feedback = await generator.generateFeedback(context);

      expect(feedback.type).toBe(FeedbackType.ERROR_CORRECTION);
      expect(feedback.failurePatterns.length).toBeGreaterThan(0);
    });

    it("should detect stagnation", async () => {
      const context: FeedbackContext = {
        currentIteration: createIteration({
          iterationNumber: 5,
          qualityScore: 0.5,
          improvementDelta: 0,
        }),
        previousIterations: [
          createIteration({ iterationNumber: 1, qualityScore: 0.5 }),
          createIteration({ iterationNumber: 2, qualityScore: 0.5 }),
          createIteration({ iterationNumber: 3, qualityScore: 0.5 }),
          createIteration({ iterationNumber: 4, qualityScore: 0.5 }),
        ],
        qualityThreshold: 0.8,
        errorPatterns: [],
      };

      const feedback = await generator.generateFeedback(context);

      expect(feedback.type).toBe(FeedbackType.APPROACH_SUGGESTION);
      expect(feedback.recommendations.length).toBeGreaterThan(0);
    });
  });

  describe("Recommendations", () => {
    it("should prioritize critical recommendations", async () => {
      const context: FeedbackContext = {
        currentIteration: createIteration({
          errorDetected: true,
          qualityScore: 0.1,
        }),
        previousIterations: [],
        qualityThreshold: 0.8,
        errorPatterns: ["CriticalError"],
      };

      const feedback = await generator.generateFeedback(context);

      const criticalRecs = feedback.recommendations.filter(
        (r) => r.priority === RecommendationPriority.CRITICAL
      );
      expect(criticalRecs.length).toBeGreaterThan(0);
    });

    it("should include high priority recommendations for underperformance", async () => {
      const context: FeedbackContext = {
        currentIteration: createIteration({
          qualityScore: 0.4,
          improvementDelta: -0.2,
        }),
        previousIterations: [],
        qualityThreshold: 0.8,
        errorPatterns: [],
      };

      const feedback = await generator.generateFeedback(context);

      const highPriorityRecs = feedback.recommendations.filter(
        (r) => r.priority === RecommendationPriority.HIGH
      );
      expect(highPriorityRecs.length).toBeGreaterThan(0);
    });

    it("should include actionable next steps in recommendations", async () => {
      const context: FeedbackContext = {
        currentIteration: createIteration({
          qualityScore: 0.5,
        }),
        previousIterations: [],
        qualityThreshold: 0.8,
        errorPatterns: [],
      };

      const feedback = await generator.generateFeedback(context);

      feedback.recommendations.forEach((rec) => {
        expect(rec.action).toBeDefined();
        expect(rec.rationale).toBeDefined();
      });
    });
  });

  describe("Confidence Scoring", () => {
    it("should have high confidence for clear patterns", async () => {
      const context: FeedbackContext = {
        currentIteration: createIteration({
          iterationNumber: 5,
          qualityScore: 0.9,
          improvementDelta: 0.1,
        }),
        previousIterations: [
          createIteration({ iterationNumber: 1, qualityScore: 0.5 }),
          createIteration({ iterationNumber: 2, qualityScore: 0.6 }),
          createIteration({ iterationNumber: 3, qualityScore: 0.7 }),
          createIteration({ iterationNumber: 4, qualityScore: 0.8 }),
        ],
        qualityThreshold: 0.8,
        errorPatterns: [],
      };

      const feedback = await generator.generateFeedback(context);

      expect(feedback.confidence).toBeGreaterThan(0.8);
    });

    it("should have lower confidence for ambiguous situations", async () => {
      const context: FeedbackContext = {
        currentIteration: createIteration({
          iterationNumber: 2,
          qualityScore: 0.6,
        }),
        previousIterations: [
          createIteration({ iterationNumber: 1, qualityScore: 0.5 }),
        ],
        qualityThreshold: 0.8,
        errorPatterns: [],
      };

      const feedback = await generator.generateFeedback(context);

      // Less data = lower confidence
      expect(feedback.confidence).toBeLessThan(0.8);
    });

    it("should adjust confidence based on data quantity", async () => {
      const context1: FeedbackContext = {
        currentIteration: createIteration({ iterationNumber: 2 }),
        previousIterations: [createIteration({ iterationNumber: 1 })],
        qualityThreshold: 0.8,
        errorPatterns: [],
      };

      const context2: FeedbackContext = {
        currentIteration: createIteration({ iterationNumber: 10 }),
        previousIterations: Array.from({ length: 9 }, (_, i) =>
          createIteration({ iterationNumber: i + 1 })
        ),
        qualityThreshold: 0.8,
        errorPatterns: [],
      };

      const feedback1 = await generator.generateFeedback(context1);
      const feedback2 = await generator.generateFeedback(context2);

      expect(feedback2.confidence).toBeGreaterThan(feedback1.confidence);
    });
  });

  describe("Pattern Tracking", () => {
    it("should identify success patterns", async () => {
      const context: FeedbackContext = {
        currentIteration: createIteration({
          qualityScore: 0.9,
          improvementDelta: 0.2,
        }),
        previousIterations: [
          createIteration({ qualityScore: 0.7, improvementDelta: 0.1 }),
        ],
        qualityThreshold: 0.8,
        errorPatterns: [],
      };

      const feedback = await generator.generateFeedback(context);

      expect(feedback.successPatterns.length).toBeGreaterThan(0);
    });

    it("should identify failure patterns", async () => {
      const context: FeedbackContext = {
        currentIteration: createIteration({
          errorDetected: true,
          qualityScore: 0.2,
        }),
        previousIterations: [],
        qualityThreshold: 0.8,
        errorPatterns: ["TypeError", "ReferenceError"],
      };

      const feedback = await generator.generateFeedback(context);

      expect(feedback.failurePatterns.length).toBeGreaterThan(0);
    });

    it("should track patterns across iterations", async () => {
      const sessionId = "session-track";

      const context1: FeedbackContext = {
        currentIteration: createIteration({
          sessionId,
          iterationNumber: 1,
          qualityScore: 0.9,
        }),
        previousIterations: [],
        qualityThreshold: 0.8,
        errorPatterns: [],
      };

      await generator.generateFeedback(context1);

      const history = generator.getFeedbackHistory(sessionId);
      expect(history).toHaveLength(1);
    });
  });

  describe("Feedback History", () => {
    it("should store feedback history per session", async () => {
      const sessionId = "session-history";

      const contexts = Array.from({ length: 3 }, (_, i) => ({
        currentIteration: createIteration({
          sessionId,
          iterationNumber: i + 1,
        }),
        previousIterations: [],
        qualityThreshold: 0.8,
        errorPatterns: [],
      }));

      for (const context of contexts) {
        await generator.generateFeedback(context);
      }

      const history = generator.getFeedbackHistory(sessionId);
      expect(history).toHaveLength(3);
    });

    it("should return empty array for non-existent session", () => {
      const history = generator.getFeedbackHistory("non-existent");
      expect(history).toEqual([]);
    });
  });

  describe("Statistics", () => {
    it("should provide feedback statistics", async () => {
      const sessionId = "session-stats";

      // Generate some feedback
      for (let i = 1; i <= 3; i++) {
        await generator.generateFeedback({
          currentIteration: createIteration({
            sessionId,
            iterationNumber: i,
            qualityScore: 0.5 + i * 0.1,
          }),
          previousIterations: [],
          qualityThreshold: 0.8,
          errorPatterns: [],
        });
      }

      const stats = generator.getStatistics(sessionId);

      expect(stats).toHaveProperty("totalFeedback");
      expect(stats).toHaveProperty("averageConfidence");
      expect(stats).toHaveProperty("totalRecommendations");
      expect(stats).toHaveProperty("recommendationsByPriority");
      expect(stats.totalFeedback).toBe(3);
    });

    it("should track recommendations by priority", async () => {
      const sessionId = "session-types";

      // Generate quality enhancement feedback
      await generator.generateFeedback({
        currentIteration: createIteration({
          sessionId,
          iterationNumber: 1,
          qualityScore: 0.9,
        }),
        previousIterations: [],
        qualityThreshold: 0.8,
        errorPatterns: [],
      });

      // Generate performance improvement feedback
      await generator.generateFeedback({
        currentIteration: createIteration({
          sessionId,
          iterationNumber: 2,
          qualityScore: 0.3,
        }),
        previousIterations: [],
        qualityThreshold: 0.8,
        errorPatterns: [],
      });

      const stats = generator.getStatistics(sessionId);

      expect(stats.totalRecommendations).toBeGreaterThan(0);
      expect(
        Object.keys(stats.recommendationsByPriority).length
      ).toBeGreaterThan(0);
    });
  });

  describe("Event Emission", () => {
    it("should emit feedback generated event", (done) => {
      const context: FeedbackContext = {
        currentIteration: createIteration(),
        previousIterations: [],
        qualityThreshold: 0.8,
        errorPatterns: [],
      };

      generator.on(LearningCoordinatorEvent.FEEDBACK_GENERATED, (event) => {
        expect(event.sessionId).toBe("session-1");
        expect(event.data.feedbackType).toBeDefined();
        done();
      });

      generator.generateFeedback(context);
    });
  });

  describe("Cleanup", () => {
    it("should cleanup session data", async () => {
      const sessionId = "session-cleanup";

      await generator.generateFeedback({
        currentIteration: createIteration({ sessionId }),
        previousIterations: [],
        qualityThreshold: 0.8,
        errorPatterns: [],
      });

      expect(generator.getFeedbackHistory(sessionId)).toHaveLength(1);

      generator.cleanup(sessionId);

      expect(generator.getFeedbackHistory(sessionId)).toEqual([]);
    });
  });

  describe("Edge Cases", () => {
    it("should handle negative quality scores", async () => {
      const context: FeedbackContext = {
        currentIteration: createIteration({
          qualityScore: -0.1,
        }),
        previousIterations: [],
        qualityThreshold: 0.8,
        errorPatterns: [],
      };

      const feedback = await generator.generateFeedback(context);

      expect(feedback).toBeDefined();
      expect(feedback.type).toBeDefined();
    });

    it("should handle quality scores above 1.0", async () => {
      const context: FeedbackContext = {
        currentIteration: createIteration({
          qualityScore: 1.5,
        }),
        previousIterations: [],
        qualityThreshold: 0.8,
        errorPatterns: [],
      };

      const feedback = await generator.generateFeedback(context);

      expect(feedback).toBeDefined();
      expect(feedback.confidence).toBeLessThanOrEqual(1);
    });

    it("should handle empty error patterns array", async () => {
      const context: FeedbackContext = {
        currentIteration: createIteration({ errorDetected: true }),
        previousIterations: [],
        qualityThreshold: 0.8,
        errorPatterns: [],
      };

      const feedback = await generator.generateFeedback(context);

      expect(feedback).toBeDefined();
    });

    it("should handle large iteration history", async () => {
      const context: FeedbackContext = {
        currentIteration: createIteration({ iterationNumber: 101 }),
        previousIterations: Array.from({ length: 100 }, (_, i) =>
          createIteration({ iterationNumber: i + 1 })
        ),
        qualityThreshold: 0.8,
        errorPatterns: [],
      };

      const feedback = await generator.generateFeedback(context);

      expect(feedback).toBeDefined();
      expect(feedback.confidence).toBeGreaterThan(0);
    });
  });

  describe("Performance", () => {
    it("should generate feedback quickly", async () => {
      const context: FeedbackContext = {
        currentIteration: createIteration(),
        previousIterations: [],
        qualityThreshold: 0.8,
        errorPatterns: [],
      };

      const startTime = Date.now();
      await generator.generateFeedback(context);
      const duration = Date.now() - startTime;

      // Should be < 200ms (P95 target)
      expect(duration).toBeLessThan(200);
    });
  });
});
