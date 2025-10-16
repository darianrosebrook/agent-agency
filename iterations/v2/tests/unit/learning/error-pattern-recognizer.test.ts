/**
 * Unit Tests: ErrorPatternRecognizer
 *
 * Tests error pattern detection, categorization, similarity matching,
 * and remediation strategy generation.
 *
 * @author @darianrosebrook
 */

import type { LearningDatabaseClient } from "../../../src/database/LearningDatabaseClient.js";
import { ErrorPatternRecognizer } from "../../../src/learning/ErrorPatternRecognizer.js";
import {
  ErrorCategory,
  LearningCoordinatorEvent,
} from "../../../src/types/learning-coordination.js";

// Mock database client
const createMockDbClient = (): jest.Mocked<LearningDatabaseClient> =>
  ({
    upsertErrorPattern: jest.fn(),
    getErrorPatterns: jest.fn(),
    createSession: jest.fn(),
    getSession: jest.fn(),
    updateSession: jest.fn(),
    getSessionsByTask: jest.fn(),
    createIteration: jest.fn(),
    getIterations: jest.fn(),
    saveSnapshot: jest.fn(),
    getSnapshot: jest.fn(),
    transaction: jest.fn(),
    getPool: jest.fn(),
  } as unknown as jest.Mocked<LearningDatabaseClient>);

describe("ErrorPatternRecognizer", () => {
  let mockDbClient: jest.Mocked<LearningDatabaseClient>;
  let recognizer: ErrorPatternRecognizer;

  beforeEach(async () => {
    mockDbClient = createMockDbClient();
    mockDbClient.getErrorPatterns.mockResolvedValue([]);
    recognizer = new ErrorPatternRecognizer(mockDbClient);
    await recognizer.initialize();
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  describe("Error Categorization", () => {
    it("should categorize syntax errors correctly", async () => {
      const iteration = {
        iterationId: "iter-1",
        sessionId: "session-1",
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 1000,
        contextSnapshotId: "snap-1",
        errorDetected: true,
        qualityScore: 0.3,
        improvementDelta: 0,
        resourceUsageMB: 10,
        promptModifications: [],
      };

      const errorMessage = "SyntaxError: Unexpected token } in line 42";

      const result = await recognizer.analyzeError(iteration, errorMessage);

      expect(result.category).toBe(ErrorCategory.SYNTAX_ERROR);
      expect(result.remediationStrategy).toBeDefined();
    });

    it("should categorize type errors correctly", async () => {
      const iteration = {
        iterationId: "iter-2",
        sessionId: "session-2",
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 1000,
        contextSnapshotId: "snap-2",
        errorDetected: true,
        qualityScore: 0.3,
        improvementDelta: 0,
        resourceUsageMB: 10,
        promptModifications: [],
      };

      const errorMessage =
        "TypeError: Cannot read property 'name' of undefined";

      const result = await recognizer.analyzeError(iteration, errorMessage);

      expect(result.category).toBe(ErrorCategory.TYPE_ERROR);
    });

    it("should categorize runtime errors correctly", async () => {
      const iteration = {
        iterationId: "iter-3",
        sessionId: "session-3",
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 1000,
        contextSnapshotId: "snap-3",
        errorDetected: true,
        qualityScore: 0.3,
        improvementDelta: 0,
        resourceUsageMB: 10,
        promptModifications: [],
      };

      const errorMessage = "ReferenceError: myVariable is not defined";

      const result = await recognizer.analyzeError(iteration, errorMessage);

      expect(result.category).toBe(ErrorCategory.RUNTIME_ERROR);
    });

    it("should categorize timeout errors correctly", async () => {
      const iteration = {
        iterationId: "iter-4",
        sessionId: "session-4",
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 1000,
        contextSnapshotId: "snap-4",
        errorDetected: true,
        qualityScore: 0.3,
        improvementDelta: 0,
        resourceUsageMB: 10,
        promptModifications: [],
      };

      const errorMessage = "TimeoutError: Operation timed out after 5000ms";

      const result = await recognizer.analyzeError(iteration, errorMessage);

      expect(result.category).toBe(ErrorCategory.TIMEOUT_ERROR);
    });

    it("should categorize unknown errors as fallback", async () => {
      const iteration = {
        iterationId: "iter-5",
        sessionId: "session-5",
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 1000,
        contextSnapshotId: "snap-5",
        errorDetected: true,
        qualityScore: 0.3,
        improvementDelta: 0,
        resourceUsageMB: 10,
        promptModifications: [],
      };

      const errorMessage =
        "Something weird happened that doesn't match patterns";

      const result = await recognizer.analyzeError(iteration, errorMessage);

      expect(result.category).toBe(ErrorCategory.UNKNOWN);
    });
  });

  describe("Pattern Matching", () => {
    it("should match similar errors to existing patterns", async () => {
      // Pre-populate with a known pattern
      const existingPattern = {
        patternId: "pattern-1",
        category: ErrorCategory.TYPE_ERROR,
        pattern: "Cannot read property of undefined",
        frequency: 5,
        confidence: 0.85,
        detectedAt: new Date(),
        remediationStrategy: "Add null checks before property access",
        successRate: 0.7,
        examples: ["TypeError: Cannot read property 'x' of undefined"],
      };

      mockDbClient.getErrorPatterns.mockResolvedValue([existingPattern]);
      await recognizer.initialize();

      const iteration = {
        iterationId: "iter-6",
        sessionId: "session-6",
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 1000,
        contextSnapshotId: "snap-6",
        errorDetected: true,
        qualityScore: 0.3,
        improvementDelta: 0,
        resourceUsageMB: 10,
        promptModifications: [],
      };

      const errorMessage = "TypeError: Cannot read property 'x' of undefined";

      const result = await recognizer.analyzeError(iteration, errorMessage);

      expect(result.patternId).toBe("pattern-1");
      expect(result.confidence).toBeGreaterThan(0.7);
      expect(result.isKnownPattern).toBe(true);
    });

    it("should create new pattern for novel errors", async () => {
      const iteration = {
        iterationId: "iter-7",
        sessionId: "session-7",
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 1000,
        contextSnapshotId: "snap-7",
        errorDetected: true,
        qualityScore: 0.3,
        improvementDelta: 0,
        resourceUsageMB: 10,
        promptModifications: [],
      };

      const errorMessage = "CustomError: Unique error that has never been seen";

      const result = await recognizer.analyzeError(iteration, errorMessage);

      expect(result.category).toBeDefined();
      expect(result.patternId).toBeDefined();
      expect(result.isKnownPattern).toBe(false);
      expect(mockDbClient.upsertErrorPattern).toHaveBeenCalled();
    });

    it("should update pattern frequency on match", async () => {
      const existingPattern = {
        patternId: "pattern-2",
        category: ErrorCategory.SYNTAX_ERROR,
        pattern: "SyntaxError Unexpected token",
        frequency: 3,
        confidence: 0.8,
        detectedAt: new Date(),
        remediationStrategy: "Check for missing brackets or commas",
        successRate: 0.6,
        examples: [],
      };

      mockDbClient.getErrorPatterns.mockResolvedValue([existingPattern]);
      await recognizer.initialize();

      const iteration = {
        iterationId: "iter-8",
        sessionId: "session-8",
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 1000,
        contextSnapshotId: "snap-8",
        errorDetected: true,
        qualityScore: 0.3,
        improvementDelta: 0,
        resourceUsageMB: 10,
        promptModifications: [],
      };

      const errorMessage = "SyntaxError: Unexpected token } at line 10";

      await recognizer.analyzeError(iteration, errorMessage);

      expect(mockDbClient.upsertErrorPattern).toHaveBeenCalledWith(
        expect.objectContaining({
          patternId: "pattern-2",
          frequency: 4, // Incremented from 3
        })
      );
    });
  });

  describe("Remediation Strategies", () => {
    it("should provide appropriate remediation for syntax errors", async () => {
      const iteration = {
        iterationId: "iter-9",
        sessionId: "session-9",
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 1000,
        contextSnapshotId: "snap-9",
        errorDetected: true,
        qualityScore: 0.3,
        improvementDelta: 0,
        resourceUsageMB: 10,
        promptModifications: [],
      };

      const errorMessage = "SyntaxError: Missing closing bracket";

      const result = await recognizer.analyzeError(iteration, errorMessage);

      expect(result.category).toBe(ErrorCategory.SYNTAX_ERROR);
      expect(result.remediationStrategy).toContain("syntax");
    });

    it("should provide appropriate remediation for type errors", async () => {
      const iteration = {
        iterationId: "iter-10",
        sessionId: "session-10",
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 1000,
        contextSnapshotId: "snap-10",
        errorDetected: true,
        qualityScore: 0.3,
        improvementDelta: 0,
        resourceUsageMB: 10,
        promptModifications: [],
      };

      const errorMessage = "TypeError: Expected number but got string";

      const result = await recognizer.analyzeError(iteration, errorMessage);

      expect(result.category).toBe(ErrorCategory.TYPE_ERROR);
      expect(result.remediationStrategy).toContain("type");
    });
  });

  describe("Event Emission", () => {
    it("should emit error detected event", async () => {
      const eventSpy = jest.fn();
      recognizer.on(LearningCoordinatorEvent.ERROR_DETECTED, eventSpy);

      const iteration = {
        iterationId: "iter-11",
        sessionId: "session-11",
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 1000,
        contextSnapshotId: "snap-11",
        errorDetected: true,
        qualityScore: 0.3,
        improvementDelta: 0,
        resourceUsageMB: 10,
        promptModifications: [],
      };

      const errorMessage = "Test error";

      await recognizer.analyzeError(iteration, errorMessage);

      expect(eventSpy).toHaveBeenCalledWith(
        expect.objectContaining({
          eventType: LearningCoordinatorEvent.ERROR_DETECTED,
          sessionId: "session-11",
        })
      );
    });

    it("should emit pattern recognized event for known patterns", async () => {
      const existingPattern = {
        patternId: "pattern-3",
        category: ErrorCategory.RUNTIME_ERROR,
        pattern: "ReferenceError not defined",
        frequency: 2,
        confidence: 0.9,
        detectedAt: new Date(),
        remediationStrategy: "Ensure variables are declared",
        successRate: 0.8,
        examples: [],
      };

      mockDbClient.getErrorPatterns.mockResolvedValue([existingPattern]);
      await recognizer.initialize();

      const eventSpy = jest.fn();
      recognizer.on(LearningCoordinatorEvent.PATTERN_RECOGNIZED, eventSpy);

      const iteration = {
        iterationId: "iter-12",
        sessionId: "session-12",
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 1000,
        contextSnapshotId: "snap-12",
        errorDetected: true,
        qualityScore: 0.3,
        improvementDelta: 0,
        resourceUsageMB: 10,
        promptModifications: [],
      };

      const errorMessage = "ReferenceError: variable not defined";

      await recognizer.analyzeError(iteration, errorMessage);

      expect(eventSpy).toHaveBeenCalled();
    });
  });

  describe("Pattern Statistics", () => {
    it("should track and report pattern statistics", async () => {
      // Add some patterns
      const patterns = [
        {
          patternId: "p1",
          category: ErrorCategory.SYNTAX_ERROR,
          pattern: "Pattern 1",
          frequency: 10,
          confidence: 0.9,
          detectedAt: new Date(),
          remediationStrategy: "Strategy 1",
          successRate: 0.8,
          examples: [],
        },
        {
          patternId: "p2",
          category: ErrorCategory.TYPE_ERROR,
          pattern: "Pattern 2",
          frequency: 5,
          confidence: 0.85,
          detectedAt: new Date(),
          remediationStrategy: "Strategy 2",
          successRate: 0.7,
          examples: [],
        },
      ];

      mockDbClient.getErrorPatterns.mockResolvedValue(patterns);
      await recognizer.initialize();

      const stats = recognizer.getStatistics();

      expect(stats).toHaveProperty("totalPatterns");
      expect(stats).toHaveProperty("averageConfidence");
      expect(stats).toHaveProperty("averageSuccessRate");
      expect(stats).toHaveProperty("categoryCounts");
      expect(stats.totalPatterns).toBe(2);
    });

    it("should get patterns by category", async () => {
      const patterns = [
        {
          patternId: "p3",
          category: ErrorCategory.TIMEOUT_ERROR,
          pattern: "Timeout pattern",
          frequency: 3,
          confidence: 0.75,
          detectedAt: new Date(),
          remediationStrategy: "Increase timeout",
          successRate: 0.6,
          examples: [],
        },
      ];

      mockDbClient.getErrorPatterns.mockResolvedValue(patterns);

      const result = await recognizer.getPatternsByCategory(
        ErrorCategory.TIMEOUT_ERROR
      );

      expect(result).toHaveLength(1);
      expect(result[0].category).toBe(ErrorCategory.TIMEOUT_ERROR);
    });

    it("should get most common patterns", () => {
      const patterns = [
        {
          patternId: "p4",
          category: ErrorCategory.SYNTAX_ERROR,
          pattern: "Common pattern",
          frequency: 20,
          confidence: 0.9,
          detectedAt: new Date(),
          remediationStrategy: "Fix syntax",
          successRate: 0.85,
          examples: [],
        },
        {
          patternId: "p5",
          category: ErrorCategory.TYPE_ERROR,
          pattern: "Less common",
          frequency: 5,
          confidence: 0.8,
          detectedAt: new Date(),
          remediationStrategy: "Fix types",
          successRate: 0.75,
          examples: [],
        },
      ];

      mockDbClient.getErrorPatterns.mockResolvedValue(patterns);

      // Need to reinitialize to load patterns
      const newRecognizer = new ErrorPatternRecognizer(mockDbClient);

      const common = newRecognizer.getMostCommonPatterns(1);

      expect(common).toHaveLength(1);
      expect(common[0].frequency).toBe(20);
    });
  });

  describe("Pattern Success Tracking", () => {
    it("should update pattern success rate", async () => {
      const pattern = {
        patternId: "pattern-4",
        category: ErrorCategory.LOGIC_ERROR,
        pattern: "Logic error pattern",
        frequency: 10,
        confidence: 0.8,
        detectedAt: new Date(),
        remediationStrategy: "Review logic",
        successRate: 0.5,
        examples: [],
      };

      mockDbClient.getErrorPatterns.mockResolvedValue([pattern]);
      await recognizer.initialize();

      await recognizer.updatePatternSuccess("pattern-4", true);

      expect(mockDbClient.upsertErrorPattern).toHaveBeenCalledWith(
        expect.objectContaining({
          patternId: "pattern-4",
          successRate: expect.any(Number),
        })
      );
    });

    it("should decrease success rate on failure", async () => {
      const pattern = {
        patternId: "pattern-5",
        category: ErrorCategory.VALIDATION_ERROR,
        pattern: "Validation error",
        frequency: 5,
        confidence: 0.7,
        detectedAt: new Date(),
        remediationStrategy: "Validate input",
        successRate: 0.6,
        examples: [],
      };

      mockDbClient.getErrorPatterns.mockResolvedValue([pattern]);
      await recognizer.initialize();

      const initialRate = 0.6;
      await recognizer.updatePatternSuccess("pattern-5", false);

      expect(mockDbClient.upsertErrorPattern).toHaveBeenCalledWith(
        expect.objectContaining({
          patternId: "pattern-5",
          successRate: expect.any(Number),
        })
      );

      // Success rate should be recalculated based on total attempts
      const calls = mockDbClient.upsertErrorPattern.mock.calls;
      const lastCall = calls[calls.length - 1][0];
      expect(lastCall.successRate).toBeLessThanOrEqual(initialRate);
    });
  });

  describe("Edge Cases", () => {
    it("should handle empty error messages", async () => {
      const iteration = {
        iterationId: "iter-13",
        sessionId: "session-13",
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 1000,
        contextSnapshotId: "snap-13",
        errorDetected: true,
        qualityScore: 0.3,
        improvementDelta: 0,
        resourceUsageMB: 10,
        promptModifications: [],
      };

      const result = await recognizer.analyzeError(iteration, "");

      expect(result.category).toBe(ErrorCategory.UNKNOWN);
      expect(result.isKnownPattern).toBe(false);
    });

    it("should handle very long error messages", async () => {
      const iteration = {
        iterationId: "iter-14",
        sessionId: "session-14",
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 1000,
        contextSnapshotId: "snap-14",
        errorDetected: true,
        qualityScore: 0.3,
        improvementDelta: 0,
        resourceUsageMB: 10,
        promptModifications: [],
      };

      const longMessage = "Error: " + "x".repeat(10000);

      const result = await recognizer.analyzeError(iteration, longMessage);

      expect(result.category).toBeDefined();
      expect(result.confidence).toBeGreaterThanOrEqual(0);
      expect(result.remediationStrategy).toBeDefined();
    });

    it("should handle special characters in error messages", async () => {
      const iteration = {
        iterationId: "iter-15",
        sessionId: "session-15",
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 1000,
        contextSnapshotId: "snap-15",
        errorDetected: true,
        qualityScore: 0.3,
        improvementDelta: 0,
        resourceUsageMB: 10,
        promptModifications: [],
      };

      const specialMessage = "Error: Invalid character ♠♣♥♦ in input @#$%^&*()";

      const result = await recognizer.analyzeError(iteration, specialMessage);

      expect(result.category).toBeDefined();
      expect(result.remediationStrategy).toBeDefined();
    });
  });
});
