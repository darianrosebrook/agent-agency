/**
 * Learning Module Exports
 *
 * ARBITER-009 Multi-Turn Learning Coordinator
 * Provides iterative agent learning with error pattern recognition,
 * context preservation, and adaptive improvements.
 *
 * @author @darianrosebrook
 */

// Core Learning Components
export { AdaptivePromptEngineer } from "./AdaptivePromptEngineer.js";
export { ContextPreservationEngine } from "./ContextPreservationEngine.js";
export { ErrorPatternRecognizer } from "./ErrorPatternRecognizer.js";
export { FeedbackGenerator } from "./FeedbackGenerator.js";
export { IterationManager } from "./IterationManager.js";
export { MultiTurnLearningCoordinator } from "./MultiTurnLearningCoordinator.js";

// Type Exports
export type { ContextPreservationConfig } from "./ContextPreservationEngine.js";
export type { ErrorAnalysisResult } from "./ErrorPatternRecognizer.js";
export type { FeedbackContext } from "./FeedbackGenerator.js";
export type { IterationContext } from "./IterationManager.js";
export type {
  LearningResult,
  LearningTask,
} from "./MultiTurnLearningCoordinator.js";
