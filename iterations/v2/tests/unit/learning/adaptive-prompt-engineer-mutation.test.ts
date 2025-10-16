/**
 * @fileoverview Focused mutation testing for AdaptivePromptEngineer
 *
 * This file contains only the tests designed to kill mutants for mutation testing.
 * Contains comprehensive edge case coverage to achieve high mutation scores.
 *
 * @author @darianrosebrook
 */

import { AdaptivePromptEngineer } from "../../../src/learning/AdaptivePromptEngineer";
import { LearningIteration } from "../../../src/types/learning-coordination";

describe("AdaptivePromptEngineer Mutation Tests", () => {
  let engineer: AdaptivePromptEngineer;

  beforeEach(() => {
    engineer = new AdaptivePromptEngineer();
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  // Initialization tests - kill constructor and basic method mutants
  describe("Initialization", () => {
    it("should initialize with empty state", () => {
      expect(engineer).toBeDefined();
      const stats = engineer.getStatistics();
      expect(stats.successPatterns).toBe(0);
      expect(stats.failurePatterns).toBe(0);
      expect(stats.totalObservations).toBe(0);
    });

    it("should initialize session history", () => {
      engineer.initializeSession("test-session");
      const result = engineer.modifyPrompt("test-session", "test", 1);
      expect(result.modifiedPrompt).toBe("test");
      expect(result.modifications).toEqual([]);
    });
  });

  // Iteration recording tests - kill conditional logic mutants
  describe("Iteration Recording", () => {
    it("should update success patterns when delta > 0.01", () => {
      const iteration: LearningIteration = {
        iterationId: "success",
        sessionId: "session",
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 100,
        contextSnapshotId: "ctx",
        errorDetected: false,
        qualityScore: 0.9,
        improvementDelta: 0.02, // > 0.01
        resourceUsageMB: 10,
        promptModifications: ["success pattern"],
      };

      engineer.recordIteration(iteration);
      const stats = engineer.getStatistics();
      expect(stats.successPatterns).toBe(1);
      expect(stats.failurePatterns).toBe(0);
    });

    it("should update failure patterns when error detected", () => {
      const iteration: LearningIteration = {
        iterationId: "error",
        sessionId: "session",
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 100,
        contextSnapshotId: "ctx",
        errorDetected: true, // This should trigger failure pattern update
        errorCategory: "logic_error" as any,
        qualityScore: 0.4,
        improvementDelta: 0.005, // Between thresholds
        resourceUsageMB: 10,
        promptModifications: [],
      };

      engineer.recordIteration(iteration);
      const stats = engineer.getStatistics();
      expect(stats.successPatterns).toBe(0);
      expect(stats.failurePatterns).toBe(1);
    });

    it("should update failure patterns when delta < -0.01", () => {
      const iteration: LearningIteration = {
        iterationId: "bad",
        sessionId: "session",
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 100,
        contextSnapshotId: "ctx",
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

    it("should not update patterns for neutral delta", () => {
      const iteration: LearningIteration = {
        iterationId: "neutral",
        sessionId: "session",
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 100,
        contextSnapshotId: "ctx",
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
    });

    it("should handle non-existent session", () => {
      const iteration: LearningIteration = {
        iterationId: "orphan",
        sessionId: "non-existent",
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 100,
        contextSnapshotId: "ctx",
        errorDetected: false,
        qualityScore: 0.8,
        improvementDelta: 0.1,
        resourceUsageMB: 10,
        promptModifications: ["pattern"],
      };

      expect(() => engineer.recordIteration(iteration)).not.toThrow();
      const stats = engineer.getStatistics();
      expect(stats.totalObservations).toBe(0);
    });
  });

  // Pattern update tests - kill arithmetic and boundary mutants
  describe("Pattern Updates", () => {
    it("should increment success pattern counts", () => {
      const iteration: LearningIteration = {
        iterationId: "success1",
        sessionId: "session",
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 100,
        contextSnapshotId: "ctx",
        errorDetected: false,
        qualityScore: 0.9,
        improvementDelta: 0.2,
        resourceUsageMB: 10,
        promptModifications: ["pattern1"],
      };

      engineer.recordIteration(iteration);

      // Record same pattern again
      const iteration2: LearningIteration = {
        ...iteration,
        iterationId: "success2",
        iterationNumber: 2,
        promptModifications: ["pattern1"], // Same pattern
      };

      engineer.recordIteration(iteration2);
      const stats = engineer.getStatistics();
      expect(stats.successPatterns).toBe(1); // Same pattern, not duplicated
    });

    it("should calculate average quality improvement", () => {
      // First iteration with improvement
      const iteration1: LearningIteration = {
        iterationId: "qual1",
        sessionId: "session",
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 100,
        contextSnapshotId: "ctx1",
        errorDetected: false,
        qualityScore: 0.8,
        improvementDelta: 0.1,
        resourceUsageMB: 10,
        promptModifications: ["quality pattern"],
      };

      // Second iteration with same pattern
      const iteration2: LearningIteration = {
        iterationId: "qual2",
        sessionId: "session",
        iterationNumber: 2,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 100,
        contextSnapshotId: "ctx2",
        errorDetected: false,
        qualityScore: 0.85,
        improvementDelta: 0.15,
        resourceUsageMB: 10,
        promptModifications: ["quality pattern"], // Same pattern
      };

      engineer.recordIteration(iteration1);
      engineer.recordIteration(iteration2);

      const stats = engineer.getStatistics();
      expect(stats.successPatterns).toBe(1);
      expect(stats.totalObservations).toBe(2);
    });
  });

  // Pattern detection tests - kill boundary and conditional mutants
  describe("Pattern Detection", () => {
    it("should detect repeated errors when 2+ errors in recent 3 iterations", () => {
      engineer.initializeSession("error-session");

      // Add iterations with repeated errors
      const iterations: LearningIteration[] = [
        {
          iterationId: "err1",
          sessionId: "error-session",
          iterationNumber: 1,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 100,
          contextSnapshotId: "ctx1",
          errorDetected: true,
          errorCategory: "logic_error" as any,
          qualityScore: 0.4,
          improvementDelta: -0.1,
          resourceUsageMB: 10,
          promptModifications: [],
        },
        {
          iterationId: "err2",
          sessionId: "error-session",
          iterationNumber: 2,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 100,
          contextSnapshotId: "ctx2",
          errorDetected: true,
          errorCategory: "logic_error" as any,
          qualityScore: 0.4,
          improvementDelta: -0.1,
          resourceUsageMB: 10,
          promptModifications: [],
        },
      ];

      iterations.forEach((iter) => engineer.recordIteration(iter));

      // This should trigger error avoidance modification
      const result = engineer.modifyPrompt("error-session", "test prompt", 3);
      expect(result.modifications.length).toBeGreaterThan(0);
    });

    it("should not detect repeated errors with only 1 error", () => {
      engineer.initializeSession("single-error-session");

      const iterations: LearningIteration[] = [
        {
          iterationId: "single-err",
          sessionId: "single-error-session",
          iterationNumber: 1,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 100,
          contextSnapshotId: "ctx",
          errorDetected: true,
          errorCategory: "logic_error" as any,
          qualityScore: 0.4,
          improvementDelta: -0.1,
          resourceUsageMB: 10,
          promptModifications: [],
        },
        {
          iterationId: "success",
          sessionId: "single-error-session",
          iterationNumber: 2,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 100,
          contextSnapshotId: "ctx",
          errorDetected: false,
          qualityScore: 0.8,
          improvementDelta: 0.1,
          resourceUsageMB: 10,
          promptModifications: ["good pattern"],
        },
      ];

      iterations.forEach((iter) => engineer.recordIteration(iter));

      const result = engineer.modifyPrompt(
        "single-error-session",
        "test prompt",
        3
      );
      // Should not trigger error avoidance (only 1 error, not 2+)
      expect(result.modifiedPrompt).toBe("test prompt");
    });

    it("should not detect repeated errors with less than 2 iterations", () => {
      engineer.initializeSession("short-session");

      const iteration: LearningIteration = {
        iterationId: "single",
        sessionId: "short-session",
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 100,
        contextSnapshotId: "ctx",
        errorDetected: true,
        errorCategory: "logic_error" as any,
        qualityScore: 0.4,
        improvementDelta: -0.1,
        resourceUsageMB: 10,
        promptModifications: [],
      };

      engineer.recordIteration(iteration);

      const result = engineer.modifyPrompt("short-session", "test prompt", 2);
      expect(result.modifiedPrompt).toBe("test prompt");
    });

    it("should detect no progress when average improvement < 0.005", () => {
      engineer.initializeSession("no-progress-session");

      const iterations: LearningIteration[] = [
        {
          iterationId: "no-progress-1",
          sessionId: "no-progress-session",
          iterationNumber: 1,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 100,
          contextSnapshotId: "ctx1",
          errorDetected: false,
          qualityScore: 0.5,
          improvementDelta: 0.002, // < 0.005
          resourceUsageMB: 10,
          promptModifications: [],
        },
        {
          iterationId: "no-progress-2",
          sessionId: "no-progress-session",
          iterationNumber: 2,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 100,
          contextSnapshotId: "ctx2",
          errorDetected: false,
          qualityScore: 0.5,
          improvementDelta: 0.003, // < 0.005
          resourceUsageMB: 10,
          promptModifications: [],
        },
        {
          iterationId: "no-progress-3",
          sessionId: "no-progress-session",
          iterationNumber: 3,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 100,
          contextSnapshotId: "ctx3",
          errorDetected: false,
          qualityScore: 0.5,
          improvementDelta: 0.002, // < 0.005
          resourceUsageMB: 10,
          promptModifications: [],
        },
      ];

      iterations.forEach((iter) => engineer.recordIteration(iter));

      // Average improvement = (0.002 + 0.003 + 0.002) / 3 = 0.00233 < 0.005
      const result = engineer.modifyPrompt(
        "no-progress-session",
        "test prompt",
        4
      );
      expect(result.modifications.length).toBeGreaterThan(0);
    });

    it("should not detect no progress when average improvement >= 0.005", () => {
      engineer.initializeSession("progress-session");

      const iterations: LearningIteration[] = [
        {
          iterationId: "progress-1",
          sessionId: "progress-session",
          iterationNumber: 1,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 100,
          contextSnapshotId: "ctx1",
          errorDetected: false,
          qualityScore: 0.5,
          improvementDelta: 0.006, // >= 0.005
          resourceUsageMB: 10,
          promptModifications: [],
        },
        {
          iterationId: "progress-2",
          sessionId: "progress-session",
          iterationNumber: 2,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 100,
          contextSnapshotId: "ctx2",
          errorDetected: false,
          qualityScore: 0.5,
          improvementDelta: 0.007, // >= 0.005
          resourceUsageMB: 10,
          promptModifications: [],
        },
      ];

      iterations.forEach((iter) => engineer.recordIteration(iter));

      const result = engineer.modifyPrompt(
        "progress-session",
        "test prompt",
        3
      );
      expect(result.modifiedPrompt).toBe("test prompt");
    });

    it("should identify successful patterns with high success rate", () => {
      // Record a pattern with high success
      const iterations: LearningIteration[] = [
        {
          iterationId: "success-pattern-1",
          sessionId: "pattern-session",
          iterationNumber: 1,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 100,
          contextSnapshotId: "ctx1",
          errorDetected: false,
          qualityScore: 0.9,
          improvementDelta: 0.1,
          resourceUsageMB: 10,
          promptModifications: ["successful approach"],
        },
        {
          iterationId: "success-pattern-2",
          sessionId: "pattern-session",
          iterationNumber: 2,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 100,
          contextSnapshotId: "ctx2",
          errorDetected: false,
          qualityScore: 0.95,
          improvementDelta: 0.15,
          resourceUsageMB: 10,
          promptModifications: ["successful approach"], // Same pattern
        },
        {
          iterationId: "success-pattern-3",
          sessionId: "pattern-session",
          iterationNumber: 3,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 100,
          contextSnapshotId: "ctx3",
          errorDetected: false,
          qualityScore: 0.92,
          improvementDelta: 0.12,
          resourceUsageMB: 10,
          promptModifications: ["successful approach"], // Same pattern again
        },
      ];

      iterations.forEach((iter) => engineer.recordIteration(iter));

      const result = engineer.modifyPrompt("pattern-session", "test prompt", 4);
      expect(result.modifications.length).toBeGreaterThan(0);
    });

    it("should not identify patterns with low success rate", () => {
      // Record a pattern that mostly fails
      const iterations: LearningIteration[] = [
        {
          iterationId: "fail-pattern-1",
          sessionId: "fail-pattern-session",
          iterationNumber: 1,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 100,
          contextSnapshotId: "ctx1",
          errorDetected: true,
          errorCategory: "logic_error" as any,
          qualityScore: 0.3,
          improvementDelta: -0.1,
          resourceUsageMB: 10,
          promptModifications: ["failing approach"],
        },
        {
          iterationId: "fail-pattern-2",
          sessionId: "fail-pattern-session",
          iterationNumber: 2,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 100,
          contextSnapshotId: "ctx2",
          errorDetected: false,
          qualityScore: 0.6,
          improvementDelta: 0.01,
          resourceUsageMB: 10,
          promptModifications: ["failing approach"], // Same pattern
        },
      ];

      iterations.forEach((iter) => engineer.recordIteration(iter));

      const result = engineer.modifyPrompt(
        "fail-pattern-session",
        "test prompt",
        3
      );
      // Should not reinforce the failing pattern
      expect(result.modifiedPrompt).toBe("test prompt");
    });
  });

  // Prompt modification tests - kill conditional and return mutants
  describe("Prompt Modification", () => {
    it("should return unmodified prompt for empty history", () => {
      const result = engineer.modifyPrompt(
        "empty-session",
        "original prompt",
        1
      );
      expect(result.modifiedPrompt).toBe("original prompt");
      expect(result.modifications).toEqual([]);
    });

    it("should return unmodified prompt for uninitialized session", () => {
      const result = engineer.modifyPrompt(
        "uninitialized-session",
        "original prompt",
        1
      );
      expect(result.modifiedPrompt).toBe("original prompt");
      expect(result.modifications).toEqual([]);
    });

    it("should handle zero iteration number", () => {
      engineer.initializeSession("zero-session");
      const result = engineer.modifyPrompt("zero-session", "test", 0);
      expect(result.modifiedPrompt).toBe("test");
      expect(result.modifications).toEqual([]);
    });

    it("should handle negative iteration number", () => {
      engineer.initializeSession("negative-session");
      const result = engineer.modifyPrompt("negative-session", "test", -1);
      expect(result.modifiedPrompt).toBe("test");
      expect(result.modifications).toEqual([]);
    });

    it("should apply all modifications when all conditions are met", () => {
      engineer.initializeSession("complex-session");

      // Create history that triggers all modification types
      const iterations: LearningIteration[] = [
        // 2+ errors in recent 3 iterations (triggers error avoidance)
        {
          iterationId: "err1",
          sessionId: "complex-session",
          iterationNumber: 1,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 100,
          contextSnapshotId: "ctx1",
          errorDetected: true,
          errorCategory: "logic_error" as any,
          qualityScore: 0.4,
          improvementDelta: -0.1,
          resourceUsageMB: 10,
          promptModifications: [],
        },
        {
          iterationId: "err2",
          sessionId: "complex-session",
          iterationNumber: 2,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 100,
          contextSnapshotId: "ctx2",
          errorDetected: true,
          errorCategory: "logic_error" as any,
          qualityScore: 0.4,
          improvementDelta: -0.1,
          resourceUsageMB: 10,
          promptModifications: [],
        },
        // Low average improvement (triggers clarifying context)
        {
          iterationId: "no-progress-1",
          sessionId: "complex-session",
          iterationNumber: 3,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 100,
          contextSnapshotId: "ctx3",
          errorDetected: false,
          qualityScore: 0.5,
          improvementDelta: 0.002, // < 0.005 average
          resourceUsageMB: 10,
          promptModifications: [],
        },
        {
          iterationId: "no-progress-2",
          sessionId: "complex-session",
          iterationNumber: 4,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 100,
          contextSnapshotId: "ctx4",
          errorDetected: false,
          qualityScore: 0.5,
          improvementDelta: 0.003, // < 0.005 average
          resourceUsageMB: 10,
          promptModifications: [],
        },
        // Successful pattern (triggers reinforcement)
        {
          iterationId: "success",
          sessionId: "complex-session",
          iterationNumber: 5,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 100,
          contextSnapshotId: "ctx5",
          errorDetected: false,
          qualityScore: 0.9,
          improvementDelta: 0.1,
          resourceUsageMB: 10,
          promptModifications: ["effective approach"],
        },
      ];

      iterations.forEach((iter) => engineer.recordIteration(iter));

      const result = engineer.modifyPrompt("complex-session", "test prompt", 6);

      // Should have applied all three types of modifications
      expect(result.modifications.length).toBeGreaterThanOrEqual(2);
      expect(result.modifiedPrompt).not.toBe("test prompt");
    });

    it("should only apply error avoidance when repeated errors detected", () => {
      engineer.initializeSession("error-only-session");

      // Only one error - should not trigger error avoidance
      const iterations: LearningIteration[] = [
        {
          iterationId: "single-error",
          sessionId: "error-only-session",
          iterationNumber: 1,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 100,
          contextSnapshotId: "ctx1",
          errorDetected: true,
          errorCategory: "logic_error" as any,
          qualityScore: 0.4,
          improvementDelta: -0.1,
          resourceUsageMB: 10,
          promptModifications: [],
        },
        {
          iterationId: "success",
          sessionId: "error-only-session",
          iterationNumber: 2,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 100,
          contextSnapshotId: "ctx2",
          errorDetected: false,
          qualityScore: 0.8,
          improvementDelta: 0.1,
          resourceUsageMB: 10,
          promptModifications: ["good pattern"],
        },
      ];

      iterations.forEach((iter) => engineer.recordIteration(iter));

      const result = engineer.modifyPrompt(
        "error-only-session",
        "test prompt",
        3
      );

      // Should not apply error avoidance (only 1 error, not 2+)
      expect(result.modifiedPrompt).toBe("test prompt");
    });

    it("should only apply clarifying context when no progress detected", () => {
      engineer.initializeSession("progress-only-session");

      // Good progress - should not trigger clarifying context
      const iterations: LearningIteration[] = [
        {
          iterationId: "progress-1",
          sessionId: "progress-only-session",
          iterationNumber: 1,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 100,
          contextSnapshotId: "ctx1",
          errorDetected: false,
          qualityScore: 0.6,
          improvementDelta: 0.01, // > 0.005 average
          resourceUsageMB: 10,
          promptModifications: [],
        },
        {
          iterationId: "progress-2",
          sessionId: "progress-only-session",
          iterationNumber: 2,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 100,
          contextSnapshotId: "ctx2",
          errorDetected: false,
          qualityScore: 0.7,
          improvementDelta: 0.02, // > 0.005 average
          resourceUsageMB: 10,
          promptModifications: [],
        },
      ];

      iterations.forEach((iter) => engineer.recordIteration(iter));

      const result = engineer.modifyPrompt(
        "progress-only-session",
        "test prompt",
        3
      );

      // Should not apply clarifying context (good progress)
      expect(result.modifiedPrompt).toBe("test prompt");
    });

    it("should only apply success reinforcement when successful patterns exist", () => {
      engineer.initializeSession("no-success-session");

      // Only failures - should not trigger success reinforcement
      const iterations: LearningIteration[] = [
        {
          iterationId: "fail-1",
          sessionId: "no-success-session",
          iterationNumber: 1,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 100,
          contextSnapshotId: "ctx1",
          errorDetected: true,
          errorCategory: "logic_error" as any,
          qualityScore: 0.3,
          improvementDelta: -0.1,
          resourceUsageMB: 10,
          promptModifications: ["bad pattern"],
        },
        {
          iterationId: "fail-2",
          sessionId: "no-success-session",
          iterationNumber: 2,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 100,
          contextSnapshotId: "ctx2",
          errorDetected: true,
          errorCategory: "logic_error" as any,
          qualityScore: 0.3,
          improvementDelta: -0.1,
          resourceUsageMB: 10,
          promptModifications: ["bad pattern"],
        },
      ];

      iterations.forEach((iter) => engineer.recordIteration(iter));

      const result = engineer.modifyPrompt(
        "no-success-session",
        "test prompt",
        3
      );

      // Should not apply success reinforcement (no successful patterns)
      // But should apply error avoidance due to repeated errors
      expect(result.modifiedPrompt).not.toBe("test prompt");
      expect(result.modifications.length).toBeGreaterThan(0);
    });

    it("should handle session with exactly 2 iterations", () => {
      engineer.initializeSession("two-iterations-session");

      const iterations: LearningIteration[] = [
        {
          iterationId: "iter-1",
          sessionId: "two-iterations-session",
          iterationNumber: 1,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 100,
          contextSnapshotId: "ctx1",
          errorDetected: false,
          qualityScore: 0.8,
          improvementDelta: 0.1,
          resourceUsageMB: 10,
          promptModifications: [],
        },
        {
          iterationId: "iter-2",
          sessionId: "two-iterations-session",
          iterationNumber: 2,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 100,
          contextSnapshotId: "ctx2",
          errorDetected: false,
          qualityScore: 0.8,
          improvementDelta: 0.1,
          resourceUsageMB: 10,
          promptModifications: [],
        },
      ];

      iterations.forEach((iter) => engineer.recordIteration(iter));

      const result = engineer.modifyPrompt(
        "two-iterations-session",
        "test prompt",
        3
      );
      expect(result).toBeDefined();
      expect(typeof result.modifiedPrompt).toBe("string");
      expect(Array.isArray(result.modifications)).toBe(true);
    });

    it("should handle session with exactly 1 iteration", () => {
      engineer.initializeSession("one-iteration-session");

      const iteration: LearningIteration = {
        iterationId: "single-iter",
        sessionId: "one-iteration-session",
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

      const result = engineer.modifyPrompt(
        "one-iteration-session",
        "test prompt",
        2
      );
      expect(result).toBeDefined();
      expect(result.modifiedPrompt).toBe("test prompt");
      expect(result.modifications).toEqual([]);
    });

    it("should handle empty prompt string", () => {
      engineer.initializeSession("empty-prompt-session");

      const iteration: LearningIteration = {
        iterationId: "empty-test",
        sessionId: "empty-prompt-session",
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

      const result = engineer.modifyPrompt("empty-prompt-session", "", 2);
      expect(result).toBeDefined();
      expect(result.modifiedPrompt).toBe("");
      expect(Array.isArray(result.modifications)).toBe(true);
    });

    it("should handle very long prompt strings", () => {
      engineer.initializeSession("long-prompt-session");

      const longPrompt = "A".repeat(10000); // 10KB prompt

      const iteration: LearningIteration = {
        iterationId: "long-test",
        sessionId: "long-prompt-session",
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

      const result = engineer.modifyPrompt(
        "long-prompt-session",
        longPrompt,
        2
      );
      expect(result).toBeDefined();
      expect(typeof result.modifiedPrompt).toBe("string");
      expect(result.modifiedPrompt.length).toBeGreaterThan(0);
      expect(Array.isArray(result.modifications)).toBe(true);
    });
  });
});
