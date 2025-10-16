/**
 * Iteration Manager
 *
 * Manages iteration lifecycle with hard limits, progress detection,
 * and resource timeout enforcement to prevent infinite loops and
 * resource exhaustion in multi-turn learning.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import { EventEmitter } from "events";
import type {
  LearningIteration,
  LearningSessionConfig,
  ProgressDetection,
  ResourceMonitoring,
} from "../types/learning-coordination.js";
import { LearningCoordinatorEvent } from "../types/learning-coordination.js";

/**
 * Iteration context for tracking state
 */
export interface IterationContext {
  sessionId: string;
  currentIteration: number;
  maxIterations: number;
  startTime: Date;
  lastIterationTime: Date;
  consecutiveNoProgress: number;
  qualityScores: number[];
  resourceUsageMB: number[];
}

/**
 * Iteration Manager
 *
 * Enforces iteration limits, monitors progress, and manages
 * resource constraints during learning sessions.
 */
export class IterationManager extends EventEmitter {
  private contexts: Map<string, IterationContext>;
  private config: LearningSessionConfig;

  constructor(config: LearningSessionConfig) {
    super();
    this.config = config;
    this.contexts = new Map();
  }

  /**
   * Initialize iteration context for a session
   *
   * @param sessionId - Learning session ID
   * @returns Initial iteration context
   */
  initializeSession(sessionId: string): IterationContext {
    const context: IterationContext = {
      sessionId,
      currentIteration: 0,
      maxIterations: this.config.maxIterations,
      startTime: new Date(),
      lastIterationTime: new Date(),
      consecutiveNoProgress: 0,
      qualityScores: [],
      resourceUsageMB: [],
    };

    this.contexts.set(sessionId, context);

    this.emit(LearningCoordinatorEvent.SESSION_STARTED, {
      sessionId,
      timestamp: new Date(),
      eventType: LearningCoordinatorEvent.SESSION_STARTED,
      data: context,
    });

    return context;
  }

  /**
   * Check if session can start next iteration
   *
   * @param sessionId - Session ID to check
   * @returns Whether next iteration is allowed
   */
  canStartIteration(sessionId: string): {
    allowed: boolean;
    reason?: string;
  } {
    const context = this.contexts.get(sessionId);

    if (!context) {
      return { allowed: false, reason: "Session not initialized" };
    }

    // Check iteration limit
    if (context.currentIteration >= context.maxIterations) {
      return {
        allowed: false,
        reason: `Maximum iterations reached (${context.maxIterations})`,
      };
    }

    // Check no-progress limit
    if (context.consecutiveNoProgress >= this.config.noProgressLimit) {
      return {
        allowed: false,
        reason: `No progress for ${this.config.noProgressLimit} consecutive iterations`,
      };
    }

    // Check session timeout (5 minutes max)
    const sessionDuration = Date.now() - context.startTime.getTime();
    const maxSessionTime = 5 * 60 * 1000; // 5 minutes

    if (sessionDuration > maxSessionTime) {
      return {
        allowed: false,
        reason: "Session timeout exceeded (5 minutes)",
      };
    }

    // Check resource budget
    if (context.resourceUsageMB.length > 0) {
      const currentUsage =
        context.resourceUsageMB[context.resourceUsageMB.length - 1];
      if (currentUsage >= this.config.resourceBudgetMB) {
        this.emit(LearningCoordinatorEvent.RESOURCE_EXHAUSTED, {
          sessionId,
          timestamp: new Date(),
          eventType: LearningCoordinatorEvent.RESOURCE_EXHAUSTED,
          data: {
            usageMB: currentUsage,
            budgetMB: this.config.resourceBudgetMB,
          },
        });

        return {
          allowed: false,
          reason: `Resource budget exceeded (${currentUsage.toFixed(2)}MB / ${
            this.config.resourceBudgetMB
          }MB)`,
        };
      }
    }

    return { allowed: true };
  }

  /**
   * Start next iteration
   *
   * @param sessionId - Session ID
   * @returns Iteration number
   */
  startIteration(sessionId: string): number {
    const context = this.contexts.get(sessionId);

    if (!context) {
      throw new Error(`Session ${sessionId} not initialized`);
    }

    const canStart = this.canStartIteration(sessionId);
    if (!canStart.allowed) {
      throw new Error(`Cannot start iteration: ${canStart.reason}`);
    }

    context.currentIteration++;
    context.lastIterationTime = new Date();

    this.emit(LearningCoordinatorEvent.ITERATION_STARTED, {
      sessionId,
      timestamp: new Date(),
      eventType: LearningCoordinatorEvent.ITERATION_STARTED,
      data: { iterationNumber: context.currentIteration },
    });

    return context.currentIteration;
  }

  /**
   * Complete iteration and update context
   *
   * @param sessionId - Session ID
   * @param iteration - Completed iteration data
   */
  completeIteration(sessionId: string, iteration: LearningIteration): void {
    const context = this.contexts.get(sessionId);

    if (!context) {
      throw new Error(`Session ${sessionId} not initialized`);
    }

    // Update quality scores
    context.qualityScores.push(iteration.qualityScore);

    // Update resource usage
    context.resourceUsageMB.push(iteration.resourceUsageMB);

    // Check progress
    const progress = this.detectProgress(sessionId, iteration);

    if (!progress.hasProgress) {
      context.consecutiveNoProgress++;
    } else {
      context.consecutiveNoProgress = 0;
    }

    // Emit resource warning if approaching limit
    const usagePercent =
      (iteration.resourceUsageMB / this.config.resourceBudgetMB) * 100;
    if (usagePercent >= 80) {
      this.emit(LearningCoordinatorEvent.RESOURCE_WARNING, {
        sessionId,
        timestamp: new Date(),
        eventType: LearningCoordinatorEvent.RESOURCE_WARNING,
        data: {
          usageMB: iteration.resourceUsageMB,
          budgetMB: this.config.resourceBudgetMB,
          percentUsed: usagePercent,
        },
      });
    }

    this.emit(LearningCoordinatorEvent.ITERATION_COMPLETED, {
      sessionId,
      timestamp: new Date(),
      eventType: LearningCoordinatorEvent.ITERATION_COMPLETED,
      data: {
        iterationNumber: iteration.iterationNumber,
        qualityScore: iteration.qualityScore,
        hasProgress: progress.hasProgress,
      },
    });
  }

  /**
   * Detect progress between iterations
   *
   * @param sessionId - Session ID
   * @param iteration - Current iteration
   * @returns Progress detection result
   */
  detectProgress(
    sessionId: string,
    iteration: LearningIteration
  ): ProgressDetection {
    const context = this.contexts.get(sessionId);

    if (!context) {
      return {
        hasProgress: false,
        improvementDelta: 0,
        consecutiveNoProgress: 0,
        shouldContinue: false,
        reason: "Session not initialized",
      };
    }

    // First iteration always has progress
    if (context.qualityScores.length === 0) {
      return {
        hasProgress: true,
        improvementDelta: iteration.improvementDelta,
        consecutiveNoProgress: 0,
        shouldContinue: true,
        reason: "First iteration",
      };
    }

    // Check if improvement is significant (>1% improvement)
    const improvementThreshold = 0.01;
    const hasProgress = iteration.improvementDelta > improvementThreshold;

    const consecutiveNoProgress = hasProgress
      ? 0
      : context.consecutiveNoProgress + 1;
    const shouldContinue =
      consecutiveNoProgress < this.config.noProgressLimit &&
      context.currentIteration < context.maxIterations;

    return {
      hasProgress,
      improvementDelta: iteration.improvementDelta,
      consecutiveNoProgress,
      shouldContinue,
      reason: hasProgress
        ? `Improvement of ${(iteration.improvementDelta * 100).toFixed(2)}%`
        : `No significant improvement (delta: ${(
            iteration.improvementDelta * 100
          ).toFixed(2)}%)`,
    };
  }

  /**
   * Get resource monitoring data
   *
   * @param sessionId - Session ID
   * @returns Resource monitoring metrics
   */
  getResourceMonitoring(sessionId: string): ResourceMonitoring | null {
    const context = this.contexts.get(sessionId);

    if (!context) {
      return null;
    }

    const currentUsage =
      context.resourceUsageMB.length > 0
        ? context.resourceUsageMB[context.resourceUsageMB.length - 1]
        : 0;

    const durationMs = Date.now() - context.startTime.getTime();
    const maxDurationMs = 5 * 60 * 1000;

    const warnings: string[] = [];

    if (currentUsage >= this.config.resourceBudgetMB * 0.8) {
      warnings.push(
        `Resource usage at ${(
          (currentUsage / this.config.resourceBudgetMB) *
          100
        ).toFixed(1)}%`
      );
    }

    if (context.currentIteration >= context.maxIterations * 0.8) {
      warnings.push(
        `Iteration count at ${(
          (context.currentIteration / context.maxIterations) *
          100
        ).toFixed(1)}%`
      );
    }

    if (durationMs >= maxDurationMs * 0.8) {
      warnings.push(
        `Session duration at ${((durationMs / maxDurationMs) * 100).toFixed(
          1
        )}%`
      );
    }

    if (context.consecutiveNoProgress >= this.config.noProgressLimit - 1) {
      warnings.push(
        `${context.consecutiveNoProgress} consecutive iterations without progress`
      );
    }

    return {
      sessionId,
      timestamp: new Date(),
      memoryUsageMB: currentUsage,
      memoryLimitMB: this.config.resourceBudgetMB,
      iterationCount: context.currentIteration,
      iterationLimit: context.maxIterations,
      durationMs,
      durationLimitMs: maxDurationMs,
      withinLimits: warnings.length === 0,
      warnings,
    };
  }

  /**
   * Get iteration context
   *
   * @param sessionId - Session ID
   * @returns Iteration context or undefined
   */
  getContext(sessionId: string): IterationContext | undefined {
    return this.contexts.get(sessionId);
  }

  /**
   * Clean up session context
   *
   * @param sessionId - Session ID to clean up
   */
  cleanup(sessionId: string): void {
    this.contexts.delete(sessionId);
    this.emit(LearningCoordinatorEvent.SESSION_COMPLETED, {
      sessionId,
      timestamp: new Date().toISOString(),
    });
  }

  /**
   * Gracefully handle degradation when limits reached
   *
   * @param sessionId - Session ID
   * @param reason - Degradation reason
   * @returns Partial results and status
   */
  gracefulDegradation(
    sessionId: string,
    reason: string
  ): {
    success: boolean;
    partialResults: boolean;
    reason: string;
    iterationsCompleted: number;
  } {
    const context = this.contexts.get(sessionId);

    if (!context) {
      return {
        success: false,
        partialResults: false,
        reason: "Session not found",
        iterationsCompleted: 0,
      };
    }

    return {
      success: false,
      partialResults: context.currentIteration > 0,
      reason,
      iterationsCompleted: context.currentIteration,
    };
  }
}
