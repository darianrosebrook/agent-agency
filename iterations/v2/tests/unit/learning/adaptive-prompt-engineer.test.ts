/**
 * @fileoverview Unit tests for AdaptivePromptEngineer
 *
 * Tests prompt adaptation, pattern learning, and optimization strategies.
 *
 * @author @darianrosebrook
 */

import { AdaptivePromptEngineer } from "../../../src/learning/AdaptivePromptEngineer";
import { LearningIteration } from "../../../src/types/learning-coordination";

describe("AdaptivePromptEngineer", () => {
  let engineer: AdaptivePromptEngineer;

  beforeEach(() => {
    engineer = new AdaptivePromptEngineer();
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  describe("initialization", () => {
    it("should initialize with empty state", () => {
      expect(engineer).toBeDefined();
      expect(engineer).toBeInstanceOf(AdaptivePromptEngineer);

      const stats = engineer.getStatistics();
      expect(stats.successPatterns).toBe(0);
      expect(stats.failurePatterns).toBe(0);
      expect(stats.totalObservations).toBe(0);
    });
  });

  describe("iteration recording", () => {
    it("should record successful iterations and learn patterns", () => {
      const iteration: LearningIteration = {
        iterationId: "iter-1",
        sessionId: "session-1",
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 150,
        contextSnapshotId: "ctx-1",
        errorDetected: false,
        qualityScore: 0.9,
        improvementDelta: 0.2,
        resourceUsageMB: 10,
        promptModifications: ["Added specific context"],
      };

      engineer.recordIteration(iteration);

      const stats = engineer.getStatistics();
      expect(stats.successPatterns).toBeGreaterThan(0);
      expect(stats.totalObservations).toBeGreaterThan(0);
    });

    it("should record failed iterations and learn failure patterns", () => {
      const iteration: LearningIteration = {
        iterationId: "iter-2",
        sessionId: "session-2",
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 200,
        contextSnapshotId: "ctx-2",
        errorDetected: true,
        errorCategory: "logic_error" as any,
        qualityScore: 0.3,
        improvementDelta: -0.1,
        resourceUsageMB: 15,
        promptModifications: [],
      };

      engineer.recordIteration(iteration);

      const stats = engineer.getStatistics();
      expect(stats.failurePatterns).toBeGreaterThan(0);
      expect(stats.totalObservations).toBeGreaterThan(0);
    });
  });

  describe("prompt modification", () => {
    it("should modify prompts based on learned patterns", () => {
      // Record a successful iteration
      const successfulIteration: LearningIteration = {
        iterationId: "success-iter",
        sessionId: "session-1",
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 100,
        contextSnapshotId: "ctx-1",
        errorDetected: false,
        qualityScore: 0.95,
        improvementDelta: 0.3,
        resourceUsageMB: 10,
        promptModifications: ["Added specific context"],
      };

      engineer.recordIteration(successfulIteration);

      const result = engineer.modifyPrompt("session-1", "Write a function", 2);

      expect(result).toBeDefined();
      expect(typeof result.modifiedPrompt).toBe("string");
      expect(Array.isArray(result.modifications)).toBe(true);
    });

    it("should handle sessions with no learning history", () => {
      const result = engineer.modifyPrompt("new-session", "Simple prompt", 1);

      expect(result).toBeDefined();
      expect(typeof result.modifiedPrompt).toBe("string");
      expect(Array.isArray(result.modifications)).toBe(true);
    });
  });
});
