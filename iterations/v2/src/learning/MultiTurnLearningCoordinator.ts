/**
 * Multi-Turn Learning Coordinator
 *
 * Main orchestration layer for iterative agent learning through feedback loops,
 * error pattern recognition, and adaptive prompting. Coordinates all learning
 * components to enable continuous improvement across multiple iterations.
 *
 * Risk Tier: 1 (Critical - affects learning quality and system intelligence)
 *
 * @author @darianrosebrook
 */

import { randomUUID } from "crypto";
import { EventEmitter } from "events";
import type { LearningDatabaseClient } from "../database/LearningDatabaseClient.js";
import type {
  LearningIteration,
  LearningSession,
  LearningSessionConfig,
  LearningSummary,
} from "../types/learning-coordination.js";
import {
  DEFAULT_LEARNING_CONFIG,
  LearningCoordinatorEvent,
  LearningSessionStatus,
} from "../types/learning-coordination.js";
import { ContextPreservationEngine } from "./ContextPreservationEngine.js";
import { ErrorPatternRecognizer } from "./ErrorPatternRecognizer.js";
import { IterationManager } from "./IterationManager.js";

/**
 * Learning task interface
 */
export interface LearningTask {
  taskId: string;
  agentId: string;
  initialContext: unknown;
  qualityEvaluator: (_result: unknown) => Promise<number>;
  executor: (_context: unknown, _iterationNumber: number) => Promise<unknown>;
}

/**
 * Learning result
 */
export interface LearningResult {
  sessionId: string;
  success: boolean;
  finalQualityScore: number;
  iterationsCompleted: number;
  improvementRate: number;
  summary: LearningSummary;
  finalResult?: unknown;
  error?: string;
}

/**
 * Multi-Turn Learning Coordinator
 *
 * Coordinates iterative learning sessions with context preservation,
 * error recognition, and adaptive improvements.
 */
export class MultiTurnLearningCoordinator extends EventEmitter {
  private dbClient: LearningDatabaseClient;
  private contextEngine: ContextPreservationEngine;
  private iterationManager: IterationManager;
  private errorRecognizer: ErrorPatternRecognizer;
  private activeSessions: Map<string, LearningSession>;

  constructor(
    dbClient: LearningDatabaseClient,
    config?: Partial<LearningSessionConfig>
  ) {
    super();

    this.dbClient = dbClient;
    const fullConfig = { ...DEFAULT_LEARNING_CONFIG, ...config };

    this.contextEngine = new ContextPreservationEngine({
      enableCompression: true,
      enableDifferentialStorage: true,
      compressionLevel: 6,
      maxSnapshotSizeMB: 50,
      checksumValidation: true,
    });

    this.iterationManager = new IterationManager(fullConfig);
    this.errorRecognizer = new ErrorPatternRecognizer(dbClient);
    this.activeSessions = new Map();

    this.setupEventHandlers();
  }

  /**
   * Initialize coordinator
   */
  async initialize(): Promise<void> {
    await this.errorRecognizer.initialize();
  }

  /**
   * Start learning session
   *
   * @param task - Learning task to execute
   * @param config - Optional session configuration
   * @returns Learning result
   */
  async startSession(
    task: LearningTask,
    config?: Partial<LearningSessionConfig>
  ): Promise<LearningResult> {
    const sessionId = randomUUID();
    const sessionConfig = { ...DEFAULT_LEARNING_CONFIG, ...config };

    // Initialize session
    const session: LearningSession = {
      sessionId,
      taskId: task.taskId,
      agentId: task.agentId,
      status: LearningSessionStatus.INITIALIZING,
      config: sessionConfig,
      startTime: new Date(),
      iterationCount: 0,
      qualityScore: 0,
      improvementTrajectory: [],
      errorPatterns: [],
    };

    this.activeSessions.set(sessionId, session);
    await this.dbClient.createSession(session);

    // Initialize iteration manager
    this.iterationManager.initializeSession(sessionId);

    // Create initial context snapshot
    const initialSnapshot = await this.contextEngine.createSnapshot(
      sessionId,
      0,
      task.initialContext
    );

    if (!initialSnapshot.success) {
      return this.failSession(
        sessionId,
        "Failed to create initial context snapshot"
      );
    }

    // Update session status
    session.status = LearningSessionStatus.ACTIVE;
    await this.dbClient.updateSession(sessionId, { status: session.status });

    // Emit session started event
    this.emit(LearningCoordinatorEvent.SESSION_STARTED, {
      sessionId: session.sessionId,
      timestamp: new Date(),
      eventType: LearningCoordinatorEvent.SESSION_STARTED,
      data: {
        taskId: task.taskId,
        agentId: task.agentId,
        config: sessionConfig,
      },
    });

    try {
      // Execute learning loop
      const result = await this.executelearningLoop(
        session,
        task,
        initialSnapshot.snapshotId
      );
      return result;
    } catch (error) {
      return this.failSession(
        sessionId,
        error instanceof Error ? error.message : "Unknown error"
      );
    } finally {
      // Cleanup
      this.iterationManager.cleanup(sessionId);
      this.contextEngine.clearSession(sessionId);
      this.activeSessions.delete(sessionId);
    }
  }

  /**
   * Execute learning loop with iterations
   *
   * @param session - Learning session
   * @param task - Learning task
   * @param initialSnapshotId - Initial context snapshot ID
   * @returns Learning result
   */
  private async executelearningLoop(
    session: LearningSession,
    task: LearningTask,
    initialSnapshotId: string
  ): Promise<LearningResult> {
    let currentContext = task.initialContext;
    let previousSnapshotId = initialSnapshotId;
    let currentQualityScore = 0;
    let previousQualityScore = 0;

    // Execute iterations until completion or limit
    while (true) {
      const canStart = this.iterationManager.canStartIteration(
        session.sessionId
      );

      if (!canStart.allowed) {
        // Reached limit - complete session
        return this.completeSession(
          session,
          currentContext,
          currentQualityScore,
          canStart.reason || "Iteration limit reached"
        );
      }

      const iterationNumber = this.iterationManager.startIteration(
        session.sessionId
      );
      const iterationStartTime = Date.now();

      let iterationResult: unknown;
      let errorDetected = false;
      let errorAnalysis;

      try {
        // Execute task iteration
        iterationResult = await task.executor(currentContext, iterationNumber);

        // Evaluate quality
        currentQualityScore = await task.qualityEvaluator(iterationResult);

        currentContext = iterationResult;
      } catch (error) {
        errorDetected = true;
        const errorMessage =
          error instanceof Error ? error.message : "Unknown error";
        const stackTrace = error instanceof Error ? error.stack : undefined;

        // Emit error detected event
        this.emit(LearningCoordinatorEvent.ERROR_DETECTED, {
          sessionId: session.sessionId,
          timestamp: new Date(),
          eventType: LearningCoordinatorEvent.ERROR_DETECTED,
          data: {
            iterationNumber,
            errorMessage,
            stackTrace,
          },
        });

        // Analyze error if recognition enabled
        if (session.config.enableErrorRecognition) {
          const tempIteration: LearningIteration = {
            iterationId: randomUUID(),
            sessionId: session.sessionId,
            iterationNumber,
            startTime: new Date(iterationStartTime),
            durationMs: 0,
            contextSnapshotId: previousSnapshotId,
            errorDetected: true,
            qualityScore: currentQualityScore,
            improvementDelta: 0,
            resourceUsageMB: 0,
            promptModifications: [],
          };

          errorAnalysis = await this.errorRecognizer.analyzeError(
            tempIteration,
            errorMessage,
            stackTrace
          );

          if (
            errorAnalysis.patternId &&
            !session.errorPatterns.includes(errorAnalysis.patternId)
          ) {
            session.errorPatterns.push(errorAnalysis.patternId);
          }
        }

        // Continue with degraded quality for error handling
        currentQualityScore = previousQualityScore * 0.9; // Penalize error
      }

      const iterationDuration = Date.now() - iterationStartTime;

      // Create context snapshot
      const snapshot = await this.contextEngine.createSnapshot(
        session.sessionId,
        iterationNumber,
        currentContext,
        previousSnapshotId
      );

      if (!snapshot.success || !snapshot.snapshotId) {
        // Snapshot failed - use previous snapshot for continuity
        console.warn(
          `Failed to create snapshot for iteration ${iterationNumber}: ${snapshot.error}`
        );
      } else {
        previousSnapshotId = snapshot.snapshotId;
      }

      // Record iteration
      const improvement = currentQualityScore - previousQualityScore;
      const iteration: LearningIteration = {
        iterationId: randomUUID(),
        sessionId: session.sessionId,
        iterationNumber,
        startTime: new Date(iterationStartTime),
        endTime: new Date(),
        durationMs: iterationDuration,
        contextSnapshotId: previousSnapshotId,
        errorDetected,
        errorCategory: errorAnalysis?.category,
        qualityScore: currentQualityScore,
        improvementDelta: improvement,
        resourceUsageMB: this.estimateResourceUsage(currentContext),
        promptModifications: [],
      };

      await this.dbClient.createIteration(iteration);
      this.iterationManager.completeIteration(session.sessionId, iteration);

      // Emit iteration completed event
      this.emit(LearningCoordinatorEvent.ITERATION_COMPLETED, {
        sessionId: session.sessionId,
        timestamp: new Date(),
        eventType: LearningCoordinatorEvent.ITERATION_COMPLETED,
        data: {
          iterationNumber,
          qualityScore: currentQualityScore,
          improvementDelta: improvement,
          errorDetected,
        },
      });

      // Update session trajectory
      session.improvementTrajectory.push(currentQualityScore);
      session.iterationCount = iterationNumber;
      session.qualityScore = currentQualityScore;

      await this.dbClient.updateSession(session.sessionId, {
        iterationCount: session.iterationCount,
        qualityScore: session.qualityScore,
        improvementTrajectory: session.improvementTrajectory,
        errorPatterns: session.errorPatterns,
      });

      previousQualityScore = currentQualityScore;

      // Check for quality threshold after iteration is recorded
      if (currentQualityScore >= session.config.qualityThreshold) {
        this.emit(LearningCoordinatorEvent.QUALITY_THRESHOLD_MET, {
          sessionId: session.sessionId,
          timestamp: new Date(),
          eventType: LearningCoordinatorEvent.QUALITY_THRESHOLD_MET,
          data: {
            qualityScore: currentQualityScore,
            threshold: session.config.qualityThreshold,
          },
        });

        return this.completeSession(
          session,
          iterationResult,
          currentQualityScore,
          "Quality threshold met"
        );
      }
    }
  }

  /**
   * Complete learning session successfully
   *
   * @param session - Learning session
   * @param finalResult - Final result
   * @param finalQualityScore - Final quality score
   * @param reason - Completion reason
   * @returns Learning result
   */
  private async completeSession(
    session: LearningSession,
    finalResult: unknown,
    finalQualityScore: number,
    reason: string
  ): Promise<LearningResult> {
    session.status = LearningSessionStatus.COMPLETED;
    session.endTime = new Date();
    session.finalResult = finalResult;
    session.qualityScore = finalQualityScore;

    // Generate learning summary
    const summary = this.generateLearningSummary(session, reason);
    session.learningSummary = summary;

    await this.dbClient.updateSession(session.sessionId, {
      status: session.status,
      endTime: session.endTime,
      finalResult,
      qualityScore: finalQualityScore,
      learningSummary: summary,
    });

    this.emit(LearningCoordinatorEvent.SESSION_COMPLETED, {
      sessionId: session.sessionId,
      timestamp: new Date(),
      eventType: LearningCoordinatorEvent.SESSION_COMPLETED,
      data: { summary, reason },
    });

    const initialQualityScore = session.improvementTrajectory[0] || 0;
    const improvementRate =
      initialQualityScore > 0
        ? (finalQualityScore - initialQualityScore) / initialQualityScore
        : 0;

    return {
      sessionId: session.sessionId,
      success: true,
      finalQualityScore,
      iterationsCompleted: session.iterationCount,
      improvementRate,
      summary,
      finalResult,
    };
  }

  /**
   * Fail learning session
   *
   * @param sessionId - Session ID
   * @param error - Error message
   * @returns Learning result
   */
  private async failSession(
    sessionId: string,
    error: string
  ): Promise<LearningResult> {
    const session = this.activeSessions.get(sessionId);

    if (session) {
      session.status = LearningSessionStatus.FAILED;
      session.endTime = new Date();

      const summary = this.generateLearningSummary(session, `Failed: ${error}`);
      session.learningSummary = summary;

      await this.dbClient.updateSession(sessionId, {
        status: session.status,
        endTime: session.endTime,
        learningSummary: summary,
      });

      this.emit(LearningCoordinatorEvent.SESSION_FAILED, {
        sessionId,
        timestamp: new Date(),
        eventType: LearningCoordinatorEvent.SESSION_FAILED,
        data: { error, summary },
      });

      return {
        sessionId,
        success: false,
        finalQualityScore: session.qualityScore,
        iterationsCompleted: session.iterationCount,
        improvementRate: 0,
        summary,
        error,
      };
    }

    // Session not found
    return {
      sessionId,
      success: false,
      finalQualityScore: 0,
      iterationsCompleted: 0,
      improvementRate: 0,
      summary: {
        sessionId,
        totalIterations: 0,
        successfulIterations: 0,
        failedIterations: 0,
        improvementRate: 0,
        finalQualityScore: 0,
        initialQualityScore: 0,
        totalDurationMs: 0,
        errorsDetected: 0,
        errorsCorrected: 0,
        patternsLearned: [],
        keyInsights: [`Failed to initialize: ${error}`],
        recommendationsApplied: 0,
        recommendationsSuccessful: 0,
      },
      error,
    };
  }

  /**
   * Generate learning summary
   *
   * @param session - Learning session
   * @param completionReason - Reason for completion
   * @returns Learning summary
   */
  private generateLearningSummary(
    session: LearningSession,
    completionReason: string
  ): LearningSummary {
    const initialQualityScore = session.improvementTrajectory[0] || 0;
    const finalQualityScore = session.qualityScore;
    const totalIterations = session.iterationCount;

    const improvementRate =
      initialQualityScore > 0
        ? (finalQualityScore - initialQualityScore) / initialQualityScore
        : 0;

    const totalDurationMs = session.endTime
      ? session.endTime.getTime() - session.startTime.getTime()
      : Date.now() - session.startTime.getTime();

    const keyInsights: string[] = [
      completionReason,
      `Quality improved from ${(initialQualityScore * 100).toFixed(1)}% to ${(
        finalQualityScore * 100
      ).toFixed(1)}%`,
    ];

    if (session.errorPatterns.length > 0) {
      keyInsights.push(
        `Encountered ${session.errorPatterns.length} unique error patterns`
      );
    }

    if (improvementRate > 0.5) {
      keyInsights.push(
        "Significant improvement achieved through iterative learning"
      );
    } else if (improvementRate < 0) {
      keyInsights.push(
        "Performance degraded - review approach and error patterns"
      );
    }

    return {
      sessionId: session.sessionId,
      totalIterations,
      successfulIterations: totalIterations,
      failedIterations: 0,
      improvementRate,
      finalQualityScore,
      initialQualityScore,
      totalDurationMs,
      errorsDetected: session.errorPatterns.length,
      errorsCorrected: 0,
      patternsLearned: session.errorPatterns,
      keyInsights,
      recommendationsApplied: 0,
      recommendationsSuccessful: 0,
    };
  }

  /**
   * Estimate resource usage for context
   *
   * @param context - Context object
   * @returns Estimated MB usage
   */
  private estimateResourceUsage(context: unknown): number {
    const jsonString = JSON.stringify(context);
    const bytes = Buffer.byteLength(jsonString, "utf-8");
    return bytes / (1024 * 1024);
  }

  /**
   * Setup event handlers for sub-components
   */
  private setupEventHandlers(): void {
    // Forward iteration manager events
    this.iterationManager.on(
      LearningCoordinatorEvent.ITERATION_STARTED,
      (payload) => {
        this.emit(LearningCoordinatorEvent.ITERATION_STARTED, payload);
      }
    );

    this.iterationManager.on(
      LearningCoordinatorEvent.ITERATION_COMPLETED,
      (payload) => {
        this.emit(LearningCoordinatorEvent.ITERATION_COMPLETED, payload);
      }
    );

    this.iterationManager.on(
      LearningCoordinatorEvent.RESOURCE_WARNING,
      (payload) => {
        this.emit(LearningCoordinatorEvent.RESOURCE_WARNING, payload);
      }
    );

    // Forward error recognizer events
    this.errorRecognizer.on(
      LearningCoordinatorEvent.ERROR_DETECTED,
      (payload) => {
        this.emit(LearningCoordinatorEvent.ERROR_DETECTED, payload);
      }
    );

    this.errorRecognizer.on(
      LearningCoordinatorEvent.PATTERN_RECOGNIZED,
      (payload) => {
        this.emit(LearningCoordinatorEvent.PATTERN_RECOGNIZED, payload);
      }
    );
  }

  /**
   * Get active session
   *
   * @param sessionId - Session ID
   * @returns Session or undefined
   */
  getActiveSession(sessionId: string): LearningSession | undefined {
    return this.activeSessions.get(sessionId);
  }

  /**
   * Get session from database
   *
   * @param sessionId - Session ID
   * @returns Session or null
   */
  async getSession(sessionId: string): Promise<LearningSession | null> {
    return this.dbClient.getSession(sessionId);
  }
}
