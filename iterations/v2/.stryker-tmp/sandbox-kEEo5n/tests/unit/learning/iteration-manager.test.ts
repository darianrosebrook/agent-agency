/**
 * Unit Tests: IterationManager
 *
 * Tests iteration lifecycle management, hard limits enforcement,
 * progress detection, and resource monitoring.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import { IterationManager } from "../../../src/learning/IterationManager.js";
import {
  LearningCoordinatorEvent,
  type LearningIteration,
  type LearningSessionConfig,
} from "../../../src/types/learning-coordination.js";

describe("IterationManager", () => {
  let config: LearningSessionConfig;
  let manager: IterationManager;

  beforeEach(() => {
    config = {
      maxIterations: 10,
      progressTimeout: 300000, // 5 minutes
      noProgressLimit: 5,
      resourceBudgetMB: 512,
      compressionRatio: 0.5,
      qualityThreshold: 0.8,
      enableAdaptivePrompting: true,
      enableErrorRecognition: true,
    };
    manager = new IterationManager(config);
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  describe("Session Initialization", () => {
    it("should initialize session context with defaults", () => {
      const sessionId = "session-1";
      const context = manager.initializeSession(sessionId);

      expect(context.sessionId).toBe(sessionId);
      expect(context.currentIteration).toBe(0);
      expect(context.maxIterations).toBe(10);
      expect(context.consecutiveNoProgress).toBe(0);
      expect(context.qualityScores).toEqual([]);
      expect(context.resourceUsageMB).toEqual([]);
      expect(context.startTime).toBeInstanceOf(Date);
      expect(context.lastIterationTime).toBeInstanceOf(Date);
    });

    it("should emit session started event", (done) => {
      const sessionId = "session-2";

      manager.on(LearningCoordinatorEvent.SESSION_STARTED, (event) => {
        expect(event.sessionId).toBe(sessionId);
        expect(event.eventType).toBe(LearningCoordinatorEvent.SESSION_STARTED);
        expect(event.data).toBeDefined();
        done();
      });

      manager.initializeSession(sessionId);
    });

    it("should allow multiple session contexts", () => {
      const session1 = manager.initializeSession("session-1");
      const session2 = manager.initializeSession("session-2");

      expect(session1.sessionId).toBe("session-1");
      expect(session2.sessionId).toBe("session-2");
      expect(manager.getContext("session-1")).toBe(session1);
      expect(manager.getContext("session-2")).toBe(session2);
    });
  });

  describe("Iteration Starting", () => {
    it("should allow starting first iteration", () => {
      const sessionId = "session-3";
      manager.initializeSession(sessionId);

      const canStart = manager.canStartIteration(sessionId);

      expect(canStart.allowed).toBe(true);
    });

    it("should increment iteration counter", () => {
      const sessionId = "session-4";
      manager.initializeSession(sessionId);

      const iteration1 = manager.startIteration(sessionId);
      const iteration2 = manager.startIteration(sessionId);
      const iteration3 = manager.startIteration(sessionId);

      expect(iteration1).toBe(1);
      expect(iteration2).toBe(2);
      expect(iteration3).toBe(3);
    });

    it("should reject starting iteration after max iterations", () => {
      const sessionId = "session-5";
      manager.initializeSession(sessionId);

      // Start max iterations
      for (let i = 0; i < 10; i++) {
        manager.startIteration(sessionId);
      }

      const canStart = manager.canStartIteration(sessionId);

      expect(canStart.allowed).toBe(false);
      expect(canStart.reason).toContain("Maximum iterations");
    });

    it("should throw error when starting iteration for non-existent session", () => {
      expect(() => {
        manager.startIteration("non-existent");
      }).toThrow();
    });
  });

  describe("Iteration Completion", () => {
    it("should record quality scores on completion", () => {
      const sessionId = "session-6";
      manager.initializeSession(sessionId);
      manager.startIteration(sessionId);

      const iteration: LearningIteration = {
        iterationId: "iter-1",
        sessionId,
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 1000,
        contextSnapshotId: "snap-1",
        errorDetected: false,
        qualityScore: 0.75,
        improvementDelta: 0.05,
        resourceUsageMB: 100,
        promptModifications: [],
      };

      manager.completeIteration(sessionId, iteration);

      const context = manager.getContext(sessionId);
      expect(context?.qualityScores).toContain(0.75);
      expect(context?.resourceUsageMB).toContain(100);
    });

    it("should track consecutive no-progress iterations", () => {
      const sessionId = "session-7";
      manager.initializeSession(sessionId);

      // First iteration with some progress
      manager.startIteration(sessionId);
      manager.completeIteration(sessionId, {
        iterationId: "iter-1",
        sessionId,
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 1000,
        contextSnapshotId: "snap-1",
        errorDetected: false,
        qualityScore: 0.5,
        improvementDelta: 0.1,
        resourceUsageMB: 100,
        promptModifications: [],
      });

      // Second iteration with no progress
      manager.startIteration(sessionId);
      manager.completeIteration(sessionId, {
        iterationId: "iter-2",
        sessionId,
        iterationNumber: 2,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 1000,
        contextSnapshotId: "snap-2",
        errorDetected: false,
        qualityScore: 0.5,
        improvementDelta: 0,
        resourceUsageMB: 100,
        promptModifications: [],
      });

      const context = manager.getContext(sessionId);
      expect(context?.consecutiveNoProgress).toBe(1);
    });

    it("should reset no-progress counter on quality improvement", () => {
      const sessionId = "session-8";
      manager.initializeSession(sessionId);

      // First iteration
      manager.startIteration(sessionId);
      manager.completeIteration(sessionId, {
        iterationId: "iter-1",
        sessionId,
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 1000,
        contextSnapshotId: "snap-1",
        errorDetected: false,
        qualityScore: 0.5,
        improvementDelta: 0,
        resourceUsageMB: 100,
        promptModifications: [],
      });

      // Second iteration with no progress
      manager.startIteration(sessionId);
      manager.completeIteration(sessionId, {
        iterationId: "iter-2",
        sessionId,
        iterationNumber: 2,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 1000,
        contextSnapshotId: "snap-2",
        errorDetected: false,
        qualityScore: 0.5,
        improvementDelta: 0,
        resourceUsageMB: 100,
        promptModifications: [],
      });

      // Third iteration with improvement
      manager.startIteration(sessionId);
      manager.completeIteration(sessionId, {
        iterationId: "iter-3",
        sessionId,
        iterationNumber: 3,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 1000,
        contextSnapshotId: "snap-3",
        errorDetected: false,
        qualityScore: 0.65,
        improvementDelta: 0.15,
        resourceUsageMB: 100,
        promptModifications: [],
      });

      const context = manager.getContext(sessionId);
      expect(context?.consecutiveNoProgress).toBe(0);
    });
  });

  describe("Progress Detection", () => {
    it("should detect progress on first iteration", () => {
      const sessionId = "session-progress-1";
      manager.initializeSession(sessionId);
      manager.startIteration(sessionId);

      const iteration: LearningIteration = {
        iterationId: "iter-1",
        sessionId,
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 1000,
        contextSnapshotId: "snap-1",
        errorDetected: false,
        qualityScore: 0.5,
        improvementDelta: 0.1,
        resourceUsageMB: 100,
        promptModifications: [],
      };

      const progressDetection = manager.detectProgress(sessionId, iteration);

      expect(progressDetection.hasProgress).toBe(true);
      expect(progressDetection.shouldContinue).toBe(true);
      expect(progressDetection.reason).toContain("First iteration");
    });

    it("should detect progress when improvement is significant", () => {
      const sessionId = "session-progress-2";
      manager.initializeSession(sessionId);

      // First iteration
      manager.startIteration(sessionId);
      manager.completeIteration(sessionId, {
        iterationId: "iter-1",
        sessionId,
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 1000,
        contextSnapshotId: "snap-1",
        errorDetected: false,
        qualityScore: 0.5,
        improvementDelta: 0.05,
        resourceUsageMB: 100,
        promptModifications: [],
      });

      // Second iteration with significant improvement
      manager.startIteration(sessionId);
      const iteration: LearningIteration = {
        iterationId: "iter-2",
        sessionId,
        iterationNumber: 2,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 1000,
        contextSnapshotId: "snap-2",
        errorDetected: false,
        qualityScore: 0.6,
        improvementDelta: 0.1,
        resourceUsageMB: 100,
        promptModifications: [],
      };

      const progressDetection = manager.detectProgress(sessionId, iteration);

      expect(progressDetection.hasProgress).toBe(true);
      expect(progressDetection.improvementDelta).toBe(0.1);
    });

    it("should detect lack of progress when improvement is minimal", () => {
      const sessionId = "session-progress-3";
      manager.initializeSession(sessionId);

      // First iteration
      manager.startIteration(sessionId);
      manager.completeIteration(sessionId, {
        iterationId: "iter-1",
        sessionId,
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 1000,
        contextSnapshotId: "snap-1",
        errorDetected: false,
        qualityScore: 0.5,
        improvementDelta: 0.05,
        resourceUsageMB: 100,
        promptModifications: [],
      });

      // Second iteration with no progress
      manager.startIteration(sessionId);
      const iteration: LearningIteration = {
        iterationId: "iter-2",
        sessionId,
        iterationNumber: 2,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 1000,
        contextSnapshotId: "snap-2",
        errorDetected: false,
        qualityScore: 0.5,
        improvementDelta: 0.005, // Below threshold
        resourceUsageMB: 100,
        promptModifications: [],
      };

      const progressDetection = manager.detectProgress(sessionId, iteration);

      expect(progressDetection.hasProgress).toBe(false);
      expect(progressDetection.reason).toContain("No significant improvement");
    });
  });

  describe("Resource Monitoring", () => {
    it("should track resource usage", () => {
      const sessionId = "session-9";
      manager.initializeSession(sessionId);
      manager.startIteration(sessionId);

      manager.completeIteration(sessionId, {
        iterationId: "iter-1",
        sessionId,
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 1000,
        contextSnapshotId: "snap-1",
        errorDetected: false,
        qualityScore: 0.5,
        improvementDelta: 0,
        resourceUsageMB: 200,
        promptModifications: [],
      });

      const monitoring = manager.getResourceMonitoring(sessionId);

      expect(monitoring).not.toBeNull();
      expect(monitoring?.memoryUsageMB).toBe(200);
      expect(monitoring?.withinLimits).toBe(true);
      expect(monitoring?.warnings).toEqual([]);
    });

    it("should warn when approaching resource limit", () => {
      const sessionId = "session-10";
      manager.initializeSession(sessionId);
      manager.startIteration(sessionId);

      // Use 90% of resource limit (512 MB * 0.9 = 460.8 MB)
      manager.completeIteration(sessionId, {
        iterationId: "iter-1",
        sessionId,
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 1000,
        contextSnapshotId: "snap-1",
        errorDetected: false,
        qualityScore: 0.5,
        improvementDelta: 0,
        resourceUsageMB: 470,
        promptModifications: [],
      });

      const monitoring = manager.getResourceMonitoring(sessionId);

      expect(monitoring?.warnings.length).toBeGreaterThan(0);
      expect(monitoring?.warnings[0]).toContain("Resource usage");
    });

    it("should return null for non-existent session", () => {
      const monitoring = manager.getResourceMonitoring("non-existent");
      expect(monitoring).toBeNull();
    });
  });

  describe("Graceful Degradation", () => {
    it("should handle graceful degradation with partial results", () => {
      const sessionId = "session-12";
      manager.initializeSession(sessionId);

      // Start an iteration to have partial results
      manager.startIteration(sessionId);

      const degradation = manager.gracefulDegradation(
        sessionId,
        "Resource limit exceeded"
      );

      expect(degradation.success).toBe(false);
      expect(degradation.partialResults).toBe(true);
      expect(degradation.reason).toBeDefined();
    });

    it("should return iteration count in degradation response", () => {
      const sessionId = "session-13";
      manager.initializeSession(sessionId);

      // Complete a few iterations
      for (let i = 0; i < 3; i++) {
        manager.startIteration(sessionId);
        manager.completeIteration(sessionId, {
          iterationId: `iter-${i + 1}`,
          sessionId,
          iterationNumber: i + 1,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 1000,
          contextSnapshotId: `snap-${i + 1}`,
          errorDetected: false,
          qualityScore: 0.5,
          improvementDelta: 0.05,
          resourceUsageMB: 100,
          promptModifications: [],
        });
      }

      const degradation = manager.gracefulDegradation(sessionId, "Manual stop");

      expect(degradation.iterationsCompleted).toBe(3);
    });
  });

  describe("Context Management", () => {
    it("should retrieve session context", () => {
      const sessionId = "session-15";
      const initialContext = manager.initializeSession(sessionId);
      const retrievedContext = manager.getContext(sessionId);

      expect(retrievedContext).toBe(initialContext);
    });

    it("should return undefined for non-existent session", () => {
      const context = manager.getContext("non-existent");
      expect(context).toBeUndefined();
    });

    it("should cleanup session context", () => {
      const sessionId = "session-16";
      manager.initializeSession(sessionId);

      expect(manager.getContext(sessionId)).toBeDefined();

      manager.cleanup(sessionId);

      expect(manager.getContext(sessionId)).toBeUndefined();
    });

    it("should emit session completed event on cleanup", (done) => {
      const sessionId = "session-17";
      manager.initializeSession(sessionId);

      manager.on(LearningCoordinatorEvent.SESSION_COMPLETED, (event) => {
        expect(event.sessionId).toBe(sessionId);
        done();
      });

      manager.cleanup(sessionId);
    });
  });

  describe("Hard Limits Enforcement", () => {
    it("should prevent iteration start after reaching max iterations", () => {
      const sessionId = "session-18";
      manager.initializeSession(sessionId);

      // Exhaust iterations with some progress to avoid no-progress limit
      for (let i = 0; i < 10; i++) {
        manager.startIteration(sessionId);
        manager.completeIteration(sessionId, {
          iterationId: `iter-${i + 1}`,
          sessionId,
          iterationNumber: i + 1,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 1000,
          contextSnapshotId: `snap-${i + 1}`,
          errorDetected: false,
          qualityScore: 0.5 + i * 0.02, // Improvement to avoid no-progress limit
          improvementDelta: 0.02,
          resourceUsageMB: 100,
          promptModifications: [],
        });
      }

      // Try to start one more iteration - should throw error
      expect(() => manager.startIteration(sessionId)).toThrow(
        "Cannot start iteration: Maximum iterations reached"
      );
    });

    it("should stop on excessive no-progress iterations", () => {
      // Create manager with lower threshold for testing
      const testConfig: LearningSessionConfig = {
        ...config,
        maxIterations: 10,
      };
      const testManager = new IterationManager(testConfig);

      const sessionId = "session-19";
      testManager.initializeSession(sessionId);

      // Simulate 5 iterations with no progress
      for (let i = 0; i < 5; i++) {
        testManager.startIteration(sessionId);
        testManager.completeIteration(sessionId, {
          iterationId: `iter-${i + 1}`,
          sessionId,
          iterationNumber: i + 1,
          startTime: new Date(),
          endTime: new Date(),
          durationMs: 1000,
          contextSnapshotId: `snap-${i + 1}`,
          errorDetected: false,
          qualityScore: 0.5,
          improvementDelta: 0,
          resourceUsageMB: 100,
          promptModifications: [],
        });
      }

      const canStart = testManager.canStartIteration(sessionId);
      expect(canStart.allowed).toBe(false);
      expect(canStart.reason).toContain("No progress");
    });

    it("should enforce resource limits", () => {
      const sessionId = "session-20";
      manager.initializeSession(sessionId);
      manager.startIteration(sessionId);

      // Use more than resource limit
      manager.completeIteration(sessionId, {
        iterationId: "iter-1",
        sessionId,
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 1000,
        contextSnapshotId: "snap-1",
        errorDetected: false,
        qualityScore: 0.5,
        improvementDelta: 0,
        resourceUsageMB: 600, // Exceeds 512 MB limit
        promptModifications: [],
      });

      const canStart = manager.canStartIteration(sessionId);
      expect(canStart.allowed).toBe(false);
      expect(canStart.reason).toContain("Resource budget exceeded");
    });
  });

  describe("Event Emission", () => {
    it("should emit iteration started event", (done) => {
      const sessionId = "session-21";
      manager.initializeSession(sessionId);

      manager.on(LearningCoordinatorEvent.ITERATION_STARTED, (event) => {
        expect(event.sessionId).toBe(sessionId);
        expect(event.data.iterationNumber).toBe(1);
        done();
      });

      manager.startIteration(sessionId);
    });

    it("should emit iteration completed event", (done) => {
      const sessionId = "session-22";
      manager.initializeSession(sessionId);
      manager.startIteration(sessionId);

      manager.on(LearningCoordinatorEvent.ITERATION_COMPLETED, (event) => {
        expect(event.sessionId).toBe(sessionId);
        expect(event.data.iterationNumber).toBe(1);
        done();
      });

      manager.completeIteration(sessionId, {
        iterationId: "iter-1",
        sessionId,
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 1000,
        contextSnapshotId: "snap-1",
        errorDetected: false,
        qualityScore: 0.5,
        improvementDelta: 0,
        resourceUsageMB: 100,
        promptModifications: [],
      });
    });

    it("should emit resource warning event", (done) => {
      const sessionId = "session-23";
      manager.initializeSession(sessionId);
      manager.startIteration(sessionId);

      manager.on(LearningCoordinatorEvent.RESOURCE_WARNING, (event) => {
        expect(event.sessionId).toBe(sessionId);
        expect(event.data.usageMB).toBe(470);
        done();
      });

      manager.completeIteration(sessionId, {
        iterationId: "iter-1",
        sessionId,
        iterationNumber: 1,
        startTime: new Date(),
        endTime: new Date(),
        durationMs: 1000,
        contextSnapshotId: "snap-1",
        errorDetected: false,
        qualityScore: 0.5,
        improvementDelta: 0,
        resourceUsageMB: 470, // Triggers warning at 90%
        promptModifications: [],
      });
    });
  });
});
