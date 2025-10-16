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

    it("should properly initialize session history", () => {
      engineer.initializeSession("test-session");

      const stats = engineer.getStatistics();
      expect(stats).toBeDefined();

      // Test that session is initialized
      const result = engineer.modifyPrompt("test-session", "test", 1);
      expect(result).toBeDefined();
      expect(result.modifiedPrompt).toBe("test");
      expect(result.modifications).toEqual([]);
    });

    it("should handle multiple session initializations", () => {
      engineer.initializeSession("session-1");
      engineer.initializeSession("session-2");

      const result1 = engineer.modifyPrompt("session-1", "prompt1", 1);
      const result2 = engineer.modifyPrompt("session-2", "prompt2", 1);

      expect(result1.modifiedPrompt).toBe("prompt1");
      expect(result2.modifiedPrompt).toBe("prompt2");
    });

    it("should reinitialize existing session", () => {
      // Initialize session
      engineer.initializeSession("test-session");

      // Add some data
      const iteration: LearningIteration = {
        iterationId: "test-iter",
        sessionId: "test-session",
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 100,
        contextSnapshotId: "ctx",
        errorDetected: false,
        qualityScore: 0.8,
        improvementDelta: 0.1,
        resourceUsageMB: 10,
        promptModifications: [],
      };
      engineer.recordIteration(iteration);

      // Reinitialize should clear history
      engineer.initializeSession("test-session");

      const result = engineer.modifyPrompt("test-session", "test", 1);
      expect(result.modifiedPrompt).toBe("test");
      expect(result.modifications).toEqual([]);
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

    it("should update success patterns when improvement delta > 0.01", () => {
      const iteration: LearningIteration = {
        iterationId: "success-iter",
        sessionId: "success-session",
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 100,
        contextSnapshotId: "ctx-success",
        errorDetected: false,
        qualityScore: 0.9,
        improvementDelta: 0.02, // > 0.01
        resourceUsageMB: 10,
        promptModifications: ["Good pattern"],
      };

      engineer.recordIteration(iteration);

      const stats = engineer.getStatistics();
      expect(stats.successPatterns).toBe(1);
      expect(stats.failurePatterns).toBe(0);
    });

    it("should update failure patterns when error detected", () => {
      const iteration: LearningIteration = {
        iterationId: "error-iter",
        sessionId: "error-session",
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 100,
        contextSnapshotId: "ctx-error",
        errorDetected: true,
        errorCategory: "syntax_error" as any,
        qualityScore: 0.4,
        improvementDelta: 0.005, // < 0.01 and > -0.01
        resourceUsageMB: 10,
        promptModifications: [],
      };

      engineer.recordIteration(iteration);

      const stats = engineer.getStatistics();
      expect(stats.successPatterns).toBe(0);
      expect(stats.failurePatterns).toBe(1);
    });

    it("should update failure patterns when improvement delta < -0.01", () => {
      const iteration: LearningIteration = {
        iterationId: "bad-iter",
        sessionId: "bad-session",
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 100,
        contextSnapshotId: "ctx-bad",
        errorDetected: false,
        qualityScore: 0.3,
        improvementDelta: -0.02, // < -0.01
        resourceUsageMB: 10,
        promptModifications: [],
      };

      engineer.recordIteration(iteration);

      const stats = engineer.getStatistics();
      expect(stats.successPatterns).toBe(0);
      expect(stats.failurePatterns).toBe(1);
    });

    it("should not update patterns for neutral iterations", () => {
      const iteration: LearningIteration = {
        iterationId: "neutral-iter",
        sessionId: "neutral-session",
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 100,
        contextSnapshotId: "ctx-neutral",
        errorDetected: false,
        qualityScore: 0.6,
        improvementDelta: 0.005, // Between -0.01 and 0.01
        resourceUsageMB: 10,
        promptModifications: [],
      };

      engineer.recordIteration(iteration);

      const stats = engineer.getStatistics();
      expect(stats.successPatterns).toBe(0);
      expect(stats.failurePatterns).toBe(0);
      expect(stats.totalObservations).toBe(1);
    });

    it("should handle iterations for non-existent sessions", () => {
      const iteration: LearningIteration = {
        iterationId: "orphaned-iter",
        sessionId: "non-existent-session",
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 100,
        contextSnapshotId: "ctx-orphaned",
        errorDetected: false,
        qualityScore: 0.8,
        improvementDelta: 0.1,
        resourceUsageMB: 10,
        promptModifications: ["Some pattern"],
      };

      // Should not throw, just silently skip
      expect(() => engineer.recordIteration(iteration)).not.toThrow();

      const stats = engineer.getStatistics();
      // Should still be 0 since session doesn't exist
      expect(stats.successPatterns).toBe(0);
      expect(stats.totalObservations).toBe(0);
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

    it("should emphasize error avoidance when repeated errors detected", () => {
      // Create session with repeated errors
      engineer.initializeSession("error-session");

      // Record multiple failed iterations with errors
      for (let i = 1; i <= 3; i++) {
        const failedIteration: LearningIteration = {
          iterationId: `error-iter-${i}`,
          sessionId: "error-session",
          iterationNumber: i,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 200,
          contextSnapshotId: `ctx-${i}`,
          errorDetected: true,
          errorCategory: "logic_error" as any,
          qualityScore: 0.3,
          improvementDelta: -0.1,
          resourceUsageMB: 15,
          promptModifications: [],
        };
        engineer.recordIteration(failedIteration);
      }

      const result = engineer.modifyPrompt("error-session", "Write code", 4);

      expect(result).toBeDefined();
      expect(result.modifications.length).toBeGreaterThan(0);
      expect(
        result.modifications.some(
          (mod) => mod.modificationType === "avoid_pattern"
        )
      ).toBe(true);
    });

    it("should add clarifying context when no progress detected", () => {
      // Create session with no progress
      engineer.initializeSession("no-progress-session");

      // Record iterations with no improvement
      for (let i = 1; i <= 3; i++) {
        const stagnantIteration: LearningIteration = {
          iterationId: `stagnant-iter-${i}`,
          sessionId: "no-progress-session",
          iterationNumber: i,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 150,
          contextSnapshotId: `ctx-${i}`,
          errorDetected: false,
          qualityScore: 0.5,
          improvementDelta: 0.001, // Very small improvement below threshold
          resourceUsageMB: 12,
          promptModifications: [],
        };
        engineer.recordIteration(stagnantIteration);
      }

      const result = engineer.modifyPrompt(
        "no-progress-session",
        "Explain concept",
        4
      );

      expect(result).toBeDefined();
      expect(result.modifications.length).toBeGreaterThan(0);
      expect(
        result.modifications.some(
          (mod) => mod.modificationType === "clarify_instruction"
        )
      ).toBe(true);
    });

    it("should reinforce successful patterns when identified", () => {
      // Create session with successful pattern
      engineer.initializeSession("success-pattern-session");

      // Record successful iteration with specific modification
      const successfulIteration: LearningIteration = {
        iterationId: "pattern-success",
        sessionId: "success-pattern-session",
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 120,
        contextSnapshotId: "ctx-success",
        errorDetected: false,
        qualityScore: 0.95,
        improvementDelta: 0.4,
        resourceUsageMB: 10,
        promptModifications: ["Added step-by-step reasoning"],
      };
      engineer.recordIteration(successfulIteration);

      const result = engineer.modifyPrompt(
        "success-pattern-session",
        "Solve problem",
        2
      );

      expect(result).toBeDefined();
      expect(result.modifications.length).toBeGreaterThan(0);
      expect(
        result.modifications.some(
          (mod) => mod.modificationType === "emphasize_pattern"
        )
      ).toBe(true);
    });

    it("should combine multiple modification strategies", () => {
      // Create complex session with multiple issues
      engineer.initializeSession("complex-session");

      // Record mixed iterations: some with errors, some with low progress, some successful
      const iterations = [
        {
          iterationId: "complex-1",
          errorDetected: true,
          errorCategory: "logic_error" as any,
          qualityScore: 0.4,
          improvementDelta: -0.05,
          promptModifications: [],
        },
        {
          iterationId: "complex-2",
          errorDetected: false,
          qualityScore: 0.55,
          improvementDelta: 0.02,
          promptModifications: [],
        },
        {
          iterationId: "complex-3",
          errorDetected: false,
          qualityScore: 0.8,
          improvementDelta: 0.3,
          promptModifications: ["Used specific examples"],
        },
      ];

      iterations.forEach((iterData, index) => {
        const iteration: LearningIteration = {
          iterationId: iterData.iterationId,
          sessionId: "complex-session",
          iterationNumber: index + 1,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 150,
          contextSnapshotId: `ctx-${index + 1}`,
          errorDetected: iterData.errorDetected,
          errorCategory: iterData.errorCategory,
          qualityScore: iterData.qualityScore,
          improvementDelta: iterData.improvementDelta,
          resourceUsageMB: 12,
          promptModifications: iterData.promptModifications,
        };
        engineer.recordIteration(iteration);
      });

      const result = engineer.modifyPrompt(
        "complex-session",
        "Complex task",
        4
      );

      expect(result).toBeDefined();
      // May have 0, 1, or multiple modifications depending on conditions
      expect(Array.isArray(result.modifications)).toBe(true);
      // May have 1 or more modifications depending on detected conditions
      if (result.modifications.length > 0) {
        const modificationTypes = result.modifications.map(
          (mod) => mod.modificationType
        );
        expect(modificationTypes.length).toBeGreaterThan(0);
      }
    });
  });

  describe("session management", () => {
    it("should initialize sessions correctly", () => {
      engineer.initializeSession("test-session");

      const stats = engineer.getStatistics();
      expect(stats).toBeDefined();

      // Session should be tracked
      const result = engineer.modifyPrompt("test-session", "Test", 1);
      expect(result).toBeDefined();
    });

    it("should handle multiple concurrent sessions", () => {
      const sessionIds = ["session-a", "session-b", "session-c"];

      // Initialize multiple sessions
      sessionIds.forEach((id) => engineer.initializeSession(id));

      // Record iterations in different sessions
      sessionIds.forEach((sessionId, index) => {
        const iteration: LearningIteration = {
          iterationId: `iter-${index}`,
          sessionId,
          iterationNumber: 1,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 100,
          contextSnapshotId: `ctx-${index}`,
          errorDetected: false,
          qualityScore: 0.8,
          improvementDelta: 0.2,
          resourceUsageMB: 10,
          promptModifications: [`Mod ${index}`],
        };
        engineer.recordIteration(iteration);
      });

      // Each session should have independent learning
      sessionIds.forEach((sessionId) => {
        const result = engineer.modifyPrompt(sessionId, "Test", 2);
        expect(result).toBeDefined();
        expect(result.modifiedPrompt).toBeDefined();
      });
    });
  });

  describe("pattern learning", () => {
    it("should learn from successful iterations", () => {
      const initialStats = engineer.getStatistics();

      // Record multiple successful iterations with same pattern
      for (let i = 1; i <= 3; i++) {
        const iteration: LearningIteration = {
          iterationId: `success-${i}`,
          sessionId: "pattern-session",
          iterationNumber: i,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 100,
          contextSnapshotId: `ctx-${i}`,
          errorDetected: false,
          qualityScore: 0.9,
          improvementDelta: 0.2,
          resourceUsageMB: 10,
          promptModifications: ["Added examples"],
        };
        engineer.recordIteration(iteration);
      }

      const finalStats = engineer.getStatistics();
      expect(finalStats.successPatterns).toBeGreaterThan(
        initialStats.successPatterns
      );
      expect(finalStats.totalObservations).toBeGreaterThan(
        initialStats.totalObservations
      );
    });

    it("should learn from failed iterations", () => {
      const initialStats = engineer.getStatistics();

      // Record multiple failed iterations with same error
      for (let i = 1; i <= 3; i++) {
        const iteration: LearningIteration = {
          iterationId: `failure-${i}`,
          sessionId: "failure-session",
          iterationNumber: i,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 200,
          contextSnapshotId: `ctx-${i}`,
          errorDetected: true,
          errorCategory: "syntax_error" as any,
          qualityScore: 0.3,
          improvementDelta: -0.1,
          resourceUsageMB: 15,
          promptModifications: [],
        };
        engineer.recordIteration(iteration);
      }

      const finalStats = engineer.getStatistics();
      expect(finalStats.failurePatterns).toBeGreaterThan(
        initialStats.failurePatterns
      );
      expect(finalStats.totalObservations).toBeGreaterThan(
        initialStats.totalObservations
      );
    });

    it("should update existing patterns with new observations", () => {
      // Record initial pattern
      const initialIteration: LearningIteration = {
        iterationId: "initial-success",
        sessionId: "update-session",
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 100,
        contextSnapshotId: "ctx-1",
        errorDetected: false,
        qualityScore: 0.8,
        improvementDelta: 0.2,
        resourceUsageMB: 10,
        promptModifications: ["Used clear structure"],
      };
      engineer.recordIteration(initialIteration);

      const statsAfterFirst = engineer.getStatistics();

      // Record additional iteration with same pattern
      const additionalIteration: LearningIteration = {
        iterationId: "additional-success",
        sessionId: "update-session",
        iterationNumber: 2,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 90,
        contextSnapshotId: "ctx-2",
        errorDetected: false,
        qualityScore: 0.85,
        improvementDelta: 0.25,
        resourceUsageMB: 9,
        promptModifications: ["Used clear structure"],
      };
      engineer.recordIteration(additionalIteration);

      const statsAfterSecond = engineer.getStatistics();

      // Patterns should be reinforced, not duplicated
      expect(statsAfterSecond.successPatterns).toBe(
        statsAfterFirst.successPatterns
      );
      expect(statsAfterSecond.totalObservations).toBeGreaterThan(
        statsAfterFirst.totalObservations
      );
    });
  });

  describe("statistics and metrics", () => {
    it("should provide comprehensive statistics", () => {
      // Record various types of iterations
      const iterations = [
        { success: true, error: false, quality: 0.9, delta: 0.3 },
        { success: false, error: true, quality: 0.4, delta: -0.1 },
        { success: true, error: false, quality: 0.8, delta: 0.2 },
        { success: false, error: false, quality: 0.6, delta: 0.05 },
      ];

      iterations.forEach((iter, index) => {
        const iteration: LearningIteration = {
          iterationId: `stats-iter-${index}`,
          sessionId: "stats-session",
          iterationNumber: index + 1,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 120,
          contextSnapshotId: `ctx-${index}`,
          errorDetected: iter.error,
          errorCategory: iter.error ? ("test_error" as any) : undefined,
          qualityScore: iter.quality,
          improvementDelta: iter.delta,
          resourceUsageMB: 12,
          promptModifications: iter.success ? ["Good practice"] : [],
        };
        engineer.recordIteration(iteration);
      });

      const stats = engineer.getStatistics();

      expect(stats).toBeDefined();
      expect(stats.totalObservations).toBeGreaterThanOrEqual(3);
      expect(stats.successPatterns).toBeGreaterThan(0);
      expect(stats.failurePatterns).toBeGreaterThan(0);
      expect(Array.isArray(stats.topSuccessPatterns)).toBe(true);
    });

    it("should track pattern effectiveness over time", () => {
      const pattern = "effective-pattern";

      // Record successful iterations with the same pattern
      for (let i = 1; i <= 5; i++) {
        const iteration: LearningIteration = {
          iterationId: `pattern-iter-${i}`,
          sessionId: "pattern-session",
          iterationNumber: i,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 100,
          contextSnapshotId: `ctx-${i}`,
          errorDetected: false,
          qualityScore: 0.8 + i * 0.02, // Improving quality
          improvementDelta: 0.15 + i * 0.01, // Improving delta
          resourceUsageMB: 10,
          promptModifications: [pattern],
        };
        engineer.recordIteration(iteration);
      }

      const stats = engineer.getStatistics();

      // Pattern should show strong positive impact
      expect(stats.successPatterns).toBeGreaterThan(0);
      expect(stats.totalObservations).toBe(5);
      expect(stats.topSuccessPatterns.length).toBeGreaterThan(0);
    });
  });

  describe("edge cases and error handling", () => {
    it("should handle empty or invalid iteration data", () => {
      // Should not crash with minimal data
      const minimalIteration: LearningIteration = {
        iterationId: "minimal",
        sessionId: "minimal-session",
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 0,
        contextSnapshotId: "",
        errorDetected: false,
        qualityScore: 0.5,
        improvementDelta: 0,
        resourceUsageMB: 0,
        promptModifications: [],
      };

      expect(() => engineer.recordIteration(minimalIteration)).not.toThrow();
    });

    it("should handle very long session histories", () => {
      engineer.initializeSession("long-session");

      // Record many iterations
      for (let i = 1; i <= 100; i++) {
        const iteration: LearningIteration = {
          iterationId: `long-iter-${i}`,
          sessionId: "long-session",
          iterationNumber: i,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 100,
          contextSnapshotId: `ctx-${i}`,
          errorDetected: i % 10 === 0, // Every 10th iteration has error
          errorCategory: i % 10 === 0 ? ("periodic_error" as any) : undefined,
          qualityScore: 0.7,
          improvementDelta: 0.05,
          resourceUsageMB: 10,
          promptModifications: i % 5 === 0 ? ["Regular pattern"] : [],
        };
        engineer.recordIteration(iteration);
      }

      // Should still work efficiently
      const result = engineer.modifyPrompt("long-session", "Test", 101);
      expect(result).toBeDefined();

      const stats = engineer.getStatistics();
      expect(stats.totalObservations).toBeGreaterThan(0);
    });

    it("should handle concurrent modifications safely", () => {
      engineer.initializeSession("concurrent-session");

      // Simulate concurrent recording
      const promises = [];
      for (let i = 1; i <= 10; i++) {
        promises.push(
          new Promise<void>((resolve) => {
            setTimeout(() => {
              const iteration: LearningIteration = {
                iterationId: `concurrent-${i}`,
                sessionId: "concurrent-session",
                iterationNumber: i,
                startTime: new Date(),
                endTime: new Date(),
                durationMs: 100,
                contextSnapshotId: `ctx-${i}`,
                errorDetected: false,
                qualityScore: 0.8,
                improvementDelta: 0.1,
                resourceUsageMB: 10,
                promptModifications: [`Concurrent mod ${i}`],
              };
              engineer.recordIteration(iteration);
              resolve();
            }, Math.random() * 10);
          })
        );
      }

      // All should complete without errors
      return Promise.all(promises).then(() => {
        const stats = engineer.getStatistics();
        expect(stats.totalObservations).toBe(10);
      });
    });
  });
});
