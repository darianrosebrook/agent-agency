/**
 * Unit Tests: AdaptivePromptEngineer
 *
 * Tests dynamic prompt modification, pattern learning, error emphasis,
 * and context clarification for iterative agent learning.
 *
 * @author @darianrosebrook
 */

import { AdaptivePromptEngineer } from "../../../src/learning/AdaptivePromptEngineer.js";
import {
  PromptModificationType,
  LearningCoordinatorEvent,
  type LearningIteration,
} from "../../../src/types/learning-coordination.js";

describe("AdaptivePromptEngineer", () => {
  let engineer: AdaptivePromptEngineer;

  beforeEach(() => {
    engineer = new AdaptivePromptEngineer();
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  describe("Session Initialization", () => {
    it("should initialize session history", () => {
      const sessionId = "session-1";
      engineer.initializeSession(sessionId);

      // Should not throw when modifying prompt for new session
      const result = engineer.modifyPrompt(
        sessionId,
        "Test prompt",
        1
      );

      expect(result).toBeDefined();
      expect(result.modifiedPrompt).toBe("Test prompt"); // No modifications on empty history
      expect(result.modifications).toEqual([]);
    });

    it("should handle multiple sessions independently", () => {
      engineer.initializeSession("session-1");
      engineer.initializeSession("session-2");

      const result1 = engineer.modifyPrompt("session-1", "Prompt 1", 1);
      const result2 = engineer.modifyPrompt("session-2", "Prompt 2", 1);

      expect(result1.modifiedPrompt).toBe("Prompt 1");
      expect(result2.modifiedPrompt).toBe("Prompt 2");
    });
  });

  describe("Iteration Recording", () => {
    it("should record iteration data", () => {
      const sessionId = "session-2";
      engineer.initializeSession(sessionId);

      const iteration: LearningIteration = {
        iterationId: "iter-1",
        sessionId,
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
      };

      engineer.recordIteration(iteration);

      // Should not throw - successful recording
      const result = engineer.modifyPrompt(sessionId, "Test prompt", 2);
      expect(result).toBeDefined();
    });

    it("should track multiple iterations", () => {
      const sessionId = "session-3";
      engineer.initializeSession(sessionId);

      // Record 3 iterations
      for (let i = 1; i <= 3; i++) {
        engineer.recordIteration({
          iterationId: `iter-${i}`,
          sessionId,
          iterationNumber: i,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 1000,
          contextSnapshotId: `snap-${i}`,
          errorDetected: false,
          qualityScore: 0.5 + i * 0.1,
          improvementDelta: 0.1,
          resourceUsageMB: 100,
          promptModifications: [],
        });
      }

      const result = engineer.modifyPrompt(sessionId, "Test prompt", 4);
      expect(result).toBeDefined();
    });
  });

  describe("Error Detection and Emphasis", () => {
    it("should detect repeated errors", () => {
      const sessionId = "session-4";
      engineer.initializeSession(sessionId);

      // Record iterations with repeated errors
      for (let i = 1; i <= 3; i++) {
        engineer.recordIteration({
          iterationId: `iter-${i}`,
          sessionId,
          iterationNumber: i,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 1000,
          contextSnapshotId: `snap-${i}`,
          errorDetected: true,
          qualityScore: 0.3,
          improvementDelta: 0,
          resourceUsageMB: 100,
          promptModifications: [],
        });
      }

      const result = engineer.modifyPrompt(sessionId, "Complete the task", 4);

      expect(result.modifications.length).toBeGreaterThan(0);
      expect(result.modifications[0].modificationType).toBe(
        PromptModificationType.AVOID_PATTERN
      );
    });

    it("should add error emphasis to prompt on repeated errors", () => {
      const sessionId = "session-5";
      engineer.initializeSession(sessionId);

      // Record multiple error iterations
      for (let i = 1; i <= 2; i++) {
        engineer.recordIteration({
          iterationId: `iter-${i}`,
          sessionId,
          iterationNumber: i,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 1000,
          contextSnapshotId: `snap-${i}`,
          errorDetected: true,
          qualityScore: 0.3,
          improvementDelta: 0,
          resourceUsageMB: 100,
          promptModifications: [],
        });
      }

      const originalPrompt = "Complete the task";
      const result = engineer.modifyPrompt(sessionId, originalPrompt, 3);

      expect(result.modifiedPrompt).not.toBe(originalPrompt);
      expect(result.modifiedPrompt.length).toBeGreaterThan(originalPrompt.length);
    });
  });

  describe("No Progress Detection", () => {
    it("should detect lack of progress", () => {
      const sessionId = "session-6";
      engineer.initializeSession(sessionId);

      // Record iterations with no progress
      for (let i = 1; i <= 3; i++) {
        engineer.recordIteration({
          iterationId: `iter-${i}`,
          sessionId,
          iterationNumber: i,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 1000,
          contextSnapshotId: `snap-${i}`,
          errorDetected: false,
          qualityScore: 0.5,
          improvementDelta: 0,
          resourceUsageMB: 100,
          promptModifications: [],
        });
      }

      const result = engineer.modifyPrompt(sessionId, "Complete the task", 4);

      expect(result.modifications.length).toBeGreaterThan(0);
      expect(
        result.modifications.some(
          (m) => m.modificationType === PromptModificationType.ADD_CONTEXT
        )
      ).toBe(true);
    });

    it("should add clarifying context when no progress detected", () => {
      const sessionId = "session-7";
      engineer.initializeSession(sessionId);

      // Record stagnant iterations
      for (let i = 1; i <= 3; i++) {
        engineer.recordIteration({
          iterationId: `iter-${i}`,
          sessionId,
          iterationNumber: i,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 1000,
          contextSnapshotId: `snap-${i}`,
          errorDetected: false,
          qualityScore: 0.5,
          improvementDelta: 0,
          resourceUsageMB: 100,
          promptModifications: [],
        });
      }

      const originalPrompt = "Complete the task";
      const result = engineer.modifyPrompt(sessionId, originalPrompt, 4);

      expect(result.modifiedPrompt).not.toBe(originalPrompt);
      expect(result.modifiedPrompt.length).toBeGreaterThan(originalPrompt.length);
    });
  });

  describe("Success Pattern Reinforcement", () => {
    it("should identify and reinforce successful patterns", () => {
      const sessionId = "session-8";
      engineer.initializeSession(sessionId);

      // Record improving iterations
      for (let i = 1; i <= 3; i++) {
        engineer.recordIteration({
          iterationId: `iter-${i}`,
          sessionId,
          iterationNumber: i,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 1000,
          contextSnapshotId: `snap-${i}`,
          errorDetected: false,
          qualityScore: 0.5 + i * 0.1,
          improvementDelta: 0.1,
          resourceUsageMB: 100,
          promptModifications: [],
        });
      }

      const result = engineer.modifyPrompt(sessionId, "Complete the task", 4);

      expect(result.modifications.length).toBeGreaterThan(0);
      expect(
        result.modifications.some(
          (m) =>
            m.modificationType === PromptModificationType.EMPHASIZE_PATTERN
        )
      ).toBe(true);
    });

    it("should not reinforce patterns when no success", () => {
      const sessionId = "session-9";
      engineer.initializeSession(sessionId);

      // Record declining iterations
      for (let i = 1; i <= 3; i++) {
        engineer.recordIteration({
          iterationId: `iter-${i}`,
          sessionId,
          iterationNumber: i,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 1000,
          contextSnapshotId: `snap-${i}`,
          errorDetected: false,
          qualityScore: 0.7 - i * 0.1,
          improvementDelta: -0.1,
          resourceUsageMB: 100,
          promptModifications: [],
        });
      }

      const result = engineer.modifyPrompt(sessionId, "Complete the task", 4);

      const hasReinforcement = result.modifications.some(
        (m) =>
          m.modificationType === PromptModificationType.EMPHASIZE_PATTERN
      );
      expect(hasReinforcement).toBe(false);
    });
  });

  describe("Combined Modifications", () => {
    it("should apply multiple modifications when warranted", () => {
      const sessionId = "session-10";
      engineer.initializeSession(sessionId);

      // Record iterations with errors and no progress
      for (let i = 1; i <= 3; i++) {
        engineer.recordIteration({
          iterationId: `iter-${i}`,
          sessionId,
          iterationNumber: i,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 1000,
          contextSnapshotId: `snap-${i}`,
          errorDetected: i % 2 === 0,
          qualityScore: 0.5,
          improvementDelta: 0,
          resourceUsageMB: 100,
          promptModifications: [],
        });
      }

      const result = engineer.modifyPrompt(sessionId, "Complete the task", 4);

      // Should have both error emphasis and context addition
      expect(result.modifications.length).toBeGreaterThan(1);
    });

    it("should preserve original prompt intent", () => {
      const sessionId = "session-11";
      engineer.initializeSession(sessionId);

      // Add some history
      for (let i = 1; i <= 2; i++) {
        engineer.recordIteration({
          iterationId: `iter-${i}`,
          sessionId,
          iterationNumber: i,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 1000,
          contextSnapshotId: `snap-${i}`,
          errorDetected: false,
          qualityScore: 0.5,
          improvementDelta: 0,
          resourceUsageMB: 100,
          promptModifications: [],
        });
      }

      const originalPrompt = "Write a function to calculate fibonacci";
      const result = engineer.modifyPrompt(sessionId, originalPrompt, 3);

      // Modified prompt should still contain original intent
      expect(result.modifiedPrompt).toContain("fibonacci");
    });
  });

  describe("Event Emission", () => {
    it("should emit prompt modified event when modifications applied", (done) => {
      const sessionId = "session-12";
      engineer.initializeSession(sessionId);

      // Record error iteration to trigger modification
      engineer.recordIteration({
        iterationId: "iter-1",
        sessionId,
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 1000,
        contextSnapshotId: "snap-1",
        errorDetected: true,
        qualityScore: 0.3,
        improvementDelta: 0,
        resourceUsageMB: 100,
        promptModifications: [],
      });

      engineer.on(LearningCoordinatorEvent.PROMPT_MODIFIED, (event) => {
        expect(event.sessionId).toBe(sessionId);
        expect(event.data.modificationsApplied).toBeGreaterThan(0);
        done();
      });

      engineer.modifyPrompt(sessionId, "Test prompt", 2);
    });

    it("should not emit event when no modifications applied", () => {
      const sessionId = "session-13";
      engineer.initializeSession(sessionId);

      const eventSpy = jest.fn();
      engineer.on(LearningCoordinatorEvent.PROMPT_MODIFIED, eventSpy);

      // No history, so no modifications
      engineer.modifyPrompt(sessionId, "Test prompt", 1);

      expect(eventSpy).not.toHaveBeenCalled();
    });
  });

  describe("Statistics", () => {
    it("should provide modification statistics", () => {
      const sessionId = "session-14";
      engineer.initializeSession(sessionId);

      // Record some iterations and trigger modifications
      for (let i = 1; i <= 3; i++) {
        engineer.recordIteration({
          iterationId: `iter-${i}`,
          sessionId,
          iterationNumber: i,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 1000,
          contextSnapshotId: `snap-${i}`,
          errorDetected: true,
          qualityScore: 0.3,
          improvementDelta: 0,
          resourceUsageMB: 100,
          promptModifications: [],
        });
      }

      // Trigger modifications
      engineer.modifyPrompt(sessionId, "Test prompt", 4);

      const stats = engineer.getStatistics();

      expect(stats).toHaveProperty("successPatterns");
      expect(stats).toHaveProperty("failurePatterns");
      expect(stats).toHaveProperty("totalObservations");
      expect(stats).toHaveProperty("topSuccessPatterns");
      expect(Array.isArray(stats.topSuccessPatterns)).toBe(true);
    });

    it("should track success patterns", () => {
      const sessionId = "session-15";
      engineer.initializeSession(sessionId);

      // Record improving iterations
      for (let i = 1; i <= 3; i++) {
        engineer.recordIteration({
          iterationId: `iter-${i}`,
          sessionId,
          iterationNumber: i,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 1000,
          contextSnapshotId: `snap-${i}`,
          errorDetected: false,
          qualityScore: 0.5 + i * 0.1,
          improvementDelta: 0.1,
          resourceUsageMB: 100,
          promptModifications: [],
        });
      }

      engineer.modifyPrompt(sessionId, "Test prompt", 4);

      const stats = engineer.getStatistics();
      expect(stats.successPatterns).toBeGreaterThanOrEqual(0);
      expect(stats.totalObservations).toBeGreaterThanOrEqual(0);
    });
  });

  describe("Session Cleanup", () => {
    it("should cleanup session data", () => {
      const sessionId = "session-16";
      engineer.initializeSession(sessionId);

      engineer.recordIteration({
        iterationId: "iter-1",
        sessionId,
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
      });

      engineer.cleanup(sessionId);

      // After cleanup, should behave as if no history
      const result = engineer.modifyPrompt(sessionId, "Test prompt", 2);
      expect(result.modifications).toEqual([]);
    });
  });

  describe("Edge Cases", () => {
    it("should handle prompt modification on uninitialized session", () => {
      const result = engineer.modifyPrompt("non-existent", "Test prompt", 1);

      expect(result.modifiedPrompt).toBe("Test prompt");
      expect(result.modifications).toEqual([]);
    });

    it("should handle empty prompt", () => {
      const sessionId = "session-17";
      engineer.initializeSession(sessionId);

      const result = engineer.modifyPrompt(sessionId, "", 1);

      expect(result.modifiedPrompt).toBe("");
      expect(result.modifications).toEqual([]);
    });

    it("should handle very long prompts", () => {
      const sessionId = "session-18";
      engineer.initializeSession(sessionId);

      const longPrompt = "Test prompt ".repeat(1000);
      const result = engineer.modifyPrompt(sessionId, longPrompt, 1);

      expect(result).toBeDefined();
      expect(typeof result.modifiedPrompt).toBe("string");
    });

    it("should handle iteration with negative quality score", () => {
      const sessionId = "session-19";
      engineer.initializeSession(sessionId);

      engineer.recordIteration({
        iterationId: "iter-1",
        sessionId,
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 1000,
        contextSnapshotId: "snap-1",
        errorDetected: false,
        qualityScore: -0.1, // Invalid but should handle gracefully
        improvementDelta: 0,
        resourceUsageMB: 100,
        promptModifications: [],
      });

      const result = engineer.modifyPrompt(sessionId, "Test prompt", 2);
      expect(result).toBeDefined();
    });
  });
});

