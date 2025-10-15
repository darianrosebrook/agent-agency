/**
 * Adaptive Prompt Engineer
 *
 * Modifies and optimizes prompts based on learning history,
 * success patterns, and failure modes to improve iterative outcomes.
 *
 * @author @darianrosebrook
 */

import { randomUUID } from "crypto";
import { EventEmitter } from "events";
import type {
  LearningIteration,
  PromptModification,
} from "../types/learning-coordination.js";
import {
  LearningCoordinatorEvent,
  PromptModificationType,
} from "../types/learning-coordination.js";

/**
 * Pattern observation for learning
 */
interface PatternObservation {
  pattern: string;
  successCount: number;
  failureCount: number;
  averageQualityImprovement: number;
  lastSeen: Date;
}

/**
 * Adaptive Prompt Engineer
 *
 * Learns from iteration history to adaptively modify prompts
 * for improved performance in subsequent iterations.
 */
export class AdaptivePromptEngineer extends EventEmitter {
  private successPatterns: Map<string, PatternObservation>;
  private failurePatterns: Map<string, PatternObservation>;
  private sessionHistory: Map<string, LearningIteration[]>;

  constructor() {
    super();
    this.successPatterns = new Map();
    this.failurePatterns = new Map();
    this.sessionHistory = new Map();
  }

  /**
   * Initialize session history
   *
   * @param sessionId - Session ID
   */
  initializeSession(sessionId: string): void {
    this.sessionHistory.set(sessionId, []);
  }

  /**
   * Record iteration for learning
   *
   * @param iteration - Completed iteration
   */
  recordIteration(iteration: LearningIteration): void {
    const history = this.sessionHistory.get(iteration.sessionId);

    if (history) {
      history.push(iteration);
    }

    // Update pattern observations
    if (iteration.improvementDelta > 0.01) {
      this.updateSuccessPatterns(iteration);
    } else if (iteration.errorDetected || iteration.improvementDelta < -0.01) {
      this.updateFailurePatterns(iteration);
    }
  }

  /**
   * Generate prompt modifications based on learning
   *
   * @param sessionId - Session ID
   * @param currentPrompt - Current prompt text
   * @param iterationNumber - Current iteration number
   * @returns Modified prompt and modification metadata
   */
  modifyPrompt(
    sessionId: string,
    currentPrompt: string,
    iterationNumber: number
  ): {
    modifiedPrompt: string;
    modifications: PromptModification[];
  } {
    const history = this.sessionHistory.get(sessionId);

    if (!history || history.length === 0) {
      return { modifiedPrompt: currentPrompt, modifications: [] };
    }

    const modifications: PromptModification[] = [];
    let workingPrompt = currentPrompt;

    // Apply modifications based on patterns
    const hasRepeatedErrors = this.detectRepeatedErrors(history);
    if (hasRepeatedErrors) {
      const result = this.emphasizeErrorAvoidance(
        sessionId,
        workingPrompt,
        iterationNumber
      );
      workingPrompt = result.modifiedPrompt;
      if (result.modification) {
        modifications.push(result.modification);
      }
    }

    const hasNoProgress = this.detectNoProgress(history);
    if (hasNoProgress) {
      const result = this.addClarifyingContext(
        sessionId,
        workingPrompt,
        iterationNumber
      );
      workingPrompt = result.modifiedPrompt;
      if (result.modification) {
        modifications.push(result.modification);
      }
    }

    const successfulPatterns = this.identifySuccessfulPatterns(history);
    if (successfulPatterns.length > 0) {
      const result = this.reinforceSuccessPatterns(
        sessionId,
        workingPrompt,
        iterationNumber,
        successfulPatterns
      );
      workingPrompt = result.modifiedPrompt;
      if (result.modification) {
        modifications.push(result.modification);
      }
    }

    // Emit event for modifications
    if (modifications.length > 0) {
      this.emit(LearningCoordinatorEvent.PROMPT_MODIFIED, {
        sessionId,
        timestamp: new Date(),
        eventType: LearningCoordinatorEvent.PROMPT_MODIFIED,
        data: {
          iterationNumber,
          modificationsApplied: modifications.length,
          modifications: modifications.map((m) => m.modificationType),
        },
      });
    }

    return { modifiedPrompt: workingPrompt, modifications };
  }

  /**
   * Detect repeated errors in history
   *
   * @param history - Iteration history
   * @returns Whether repeated errors detected
   */
  private detectRepeatedErrors(history: LearningIteration[]): boolean {
    if (history.length < 2) {
      return false;
    }

    const recentIterations = history.slice(-3);
    const errorCount = recentIterations.filter((it) => it.errorDetected).length;

    return errorCount >= 2;
  }

  /**
   * Detect no progress in recent iterations
   *
   * @param history - Iteration history
   * @returns Whether no progress detected
   */
  private detectNoProgress(history: LearningIteration[]): boolean {
    if (history.length < 2) {
      return false;
    }

    const recentIterations = history.slice(-3);
    const improvements = recentIterations.map((it) => it.improvementDelta);
    const avgImprovement =
      improvements.reduce((a, b) => a + b, 0) / improvements.length;

    return avgImprovement < 0.005; // Less than 0.5% average improvement
  }

  /**
   * Identify successful patterns from history
   *
   * @param history - Iteration history
   * @returns Array of successful pattern strings
   */
  private identifySuccessfulPatterns(_history: LearningIteration[]): string[] {
    const patterns: string[] = [];
    for (const pattern of this.successPatterns.values()) {
      if (
        pattern.successCount > pattern.failureCount &&
        pattern.averageQualityImprovement > 0.01
      ) {
        patterns.push(pattern.pattern);
      }
    }

    return patterns.slice(0, 3); // Top 3 patterns
  }

  /**
   * Emphasize error avoidance in prompt
   *
   * @param sessionId - Session ID
   * @param prompt - Current prompt
   * @param iterationNumber - Iteration number
   * @returns Modified prompt and modification
   */
  private emphasizeErrorAvoidance(
    sessionId: string,
    prompt: string,
    iterationNumber: number
  ): {
    modifiedPrompt: string;
    modification?: PromptModification;
  } {
    const errorGuidance = [
      "\n\nIMPORTANT: Previous iterations encountered errors.",
      "Please carefully review:",
      "- Input validation and type checking",
      "- Null/undefined handling",
      "- Edge cases and boundary conditions",
      "- Error handling and recovery",
    ].join("\n");

    const failurePatternsList = Array.from(this.failurePatterns.values())
      .sort((a, b) => b.failureCount - a.failureCount)
      .slice(0, 3)
      .map((p) => p.pattern);

    if (failurePatternsList.length > 0) {
      const avoidanceNote = `\n\nAvoid these failure patterns:\n${failurePatternsList
        .map((p) => `- ${p}`)
        .join("\n")}`;
      const modifiedPrompt = prompt + errorGuidance + avoidanceNote;

      const modification: PromptModification = {
        modificationId: randomUUID(),
        sessionId,
        iterationNumber,
        modificationType: PromptModificationType.AVOID_PATTERN,
        originalPrompt: prompt,
        modifiedPrompt,
        rationale: "Added error avoidance guidance based on previous failures",
        successPatterns: [],
        failurePatterns: failurePatternsList,
        appliedAt: new Date(),
      };

      return { modifiedPrompt, modification };
    }

    const modifiedPrompt = prompt + errorGuidance;
    return { modifiedPrompt };
  }

  /**
   * Add clarifying context to prompt
   *
   * @param sessionId - Session ID
   * @param prompt - Current prompt
   * @param iterationNumber - Iteration number
   * @returns Modified prompt and modification
   */
  private addClarifyingContext(
    sessionId: string,
    prompt: string,
    iterationNumber: number
  ): {
    modifiedPrompt: string;
    modification?: PromptModification;
  } {
    const clarification = [
      "\n\nCLARIFICATION: Progress has been minimal in recent iterations.",
      "Consider:",
      "- Breaking the problem into smaller steps",
      "- Simplifying the approach",
      "- Verifying assumptions and requirements",
      "- Focusing on one aspect at a time",
    ].join("\n");

    const modifiedPrompt = prompt + clarification;

    const modification: PromptModification = {
      modificationId: randomUUID(),
      sessionId,
      iterationNumber,
      modificationType: PromptModificationType.CLARIFY_INSTRUCTION,
      originalPrompt: prompt,
      modifiedPrompt,
      rationale: "Added clarification due to lack of progress",
      successPatterns: [],
      failurePatterns: [],
      appliedAt: new Date(),
    };

    return { modifiedPrompt, modification };
  }

  /**
   * Reinforce successful patterns in prompt
   *
   * @param sessionId - Session ID
   * @param prompt - Current prompt
   * @param iterationNumber - Iteration number
   * @param patterns - Successful patterns to reinforce
   * @returns Modified prompt and modification
   */
  private reinforceSuccessPatterns(
    sessionId: string,
    prompt: string,
    iterationNumber: number,
    patterns: string[]
  ): {
    modifiedPrompt: string;
    modification?: PromptModification;
  } {
    if (patterns.length === 0) {
      return { modifiedPrompt: prompt };
    }

    const reinforcement = [
      "\n\nSUCCESS PATTERNS: Continue using these effective approaches:",
      ...patterns.map((p) => `- ${p}`),
      "",
    ].join("\n");

    const modifiedPrompt = prompt + reinforcement;

    const modification: PromptModification = {
      modificationId: randomUUID(),
      sessionId,
      iterationNumber,
      modificationType: PromptModificationType.EMPHASIZE_PATTERN,
      originalPrompt: prompt,
      modifiedPrompt,
      rationale: "Reinforced successful patterns from previous iterations",
      successPatterns: patterns,
      failurePatterns: [],
      appliedAt: new Date(),
    };

    return { modifiedPrompt, modification };
  }

  /**
   * Update success pattern observations
   *
   * @param iteration - Successful iteration
   */
  private updateSuccessPatterns(iteration: LearningIteration): void {
    for (const modification of iteration.promptModifications) {
      const pattern =
        typeof modification === "string"
          ? modification
          : JSON.stringify(modification);

      const existing = this.successPatterns.get(pattern);
      if (existing) {
        existing.successCount++;
        existing.averageQualityImprovement =
          (existing.averageQualityImprovement * (existing.successCount - 1) +
            iteration.improvementDelta) /
          existing.successCount;
        existing.lastSeen = new Date();
      } else {
        this.successPatterns.set(pattern, {
          pattern,
          successCount: 1,
          failureCount: 0,
          averageQualityImprovement: iteration.improvementDelta,
          lastSeen: new Date(),
        });
      }
    }
  }

  /**
   * Update failure pattern observations
   *
   * @param iteration - Failed iteration
   */
  private updateFailurePatterns(iteration: LearningIteration): void {
    if (iteration.errorCategory) {
      const pattern = iteration.errorCategory;

      const existing = this.failurePatterns.get(pattern);
      if (existing) {
        existing.failureCount++;
        existing.lastSeen = new Date();
      } else {
        this.failurePatterns.set(pattern, {
          pattern,
          successCount: 0,
          failureCount: 1,
          averageQualityImprovement: iteration.improvementDelta,
          lastSeen: new Date(),
        });
      }
    }
  }

  /**
   * Get pattern statistics
   *
   * @returns Pattern learning statistics
   */
  getStatistics(): {
    successPatterns: number;
    failurePatterns: number;
    totalObservations: number;
    topSuccessPatterns: Array<{ pattern: string; successRate: number }>;
  } {
    const successCount = this.successPatterns.size;
    const failureCount = this.failurePatterns.size;
    const totalObservations =
      Array.from(this.successPatterns.values()).reduce(
        (sum, p) => sum + p.successCount + p.failureCount,
        0
      ) +
      Array.from(this.failurePatterns.values()).reduce(
        (sum, p) => sum + p.successCount + p.failureCount,
        0
      );

    const topSuccess = Array.from(this.successPatterns.values())
      .filter((p) => p.successCount > 0)
      .sort((a, b) => b.averageQualityImprovement - a.averageQualityImprovement)
      .slice(0, 5)
      .map((p) => ({
        pattern: p.pattern,
        successRate: p.successCount / (p.successCount + p.failureCount),
      }));

    return {
      successPatterns: successCount,
      failurePatterns: failureCount,
      totalObservations,
      topSuccessPatterns: topSuccess,
    };
  }

  /**
   * Clean up session data
   *
   * @param sessionId - Session ID
   */
  cleanup(sessionId: string): void {
    this.sessionHistory.delete(sessionId);
  }
}
