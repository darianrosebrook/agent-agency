/**
 * Learning Coordination Type Definitions
 *
 * Type contracts for ARBITER-009 Multi-Turn Learning Coordinator
 * Defines interfaces for iterative agent learning, error pattern recognition,
 * and adaptive prompting systems.
 *
 * @author @darianrosebrook
 */

/**
 * Learning session configuration
 */
export interface LearningSessionConfig {
  maxIterations: number;
  progressTimeout: number;
  noProgressLimit: number;
  resourceBudgetMB: number;
  compressionRatio: number;
  qualityThreshold: number;
  enableAdaptivePrompting: boolean;
  enableErrorRecognition: boolean;
}

/**
 * Learning session metadata
 */
export interface LearningSession {
  sessionId: string;
  taskId: string;
  agentId: string;
  status: LearningSessionStatus;
  config: LearningSessionConfig;
  startTime: Date;
  endTime?: Date;
  iterationCount: number;
  qualityScore: number;
  improvementTrajectory: number[];
  errorPatterns: string[];
  finalResult?: unknown;
  learningSummary?: LearningSummary;
}

/**
 * Learning session status states
 */
export enum LearningSessionStatus {
  _INITIALIZING = "initializing",
  _ACTIVE = "active",
  _EVALUATING = "evaluating",
  _COMPLETED = "completed",
  _FAILED = "failed",
  _TIMEOUT = "timeout",
  _RESOURCE_EXHAUSTED = "resource_exhausted",
}

/**
 * Individual iteration record
 */
export interface LearningIteration {
  iterationId: string;
  sessionId: string;
  iterationNumber: number;
  startTime: Date;
  endTime?: Date;
  durationMs: number;
  contextSnapshotId: string;
  errorDetected: boolean;
  errorCategory?: ErrorCategory;
  qualityScore: number;
  improvementDelta: number;
  resourceUsageMB: number;
  promptModifications: string[];
  feedback?: IterationFeedback;
}

/**
 * Error categories for pattern recognition
 */
export enum ErrorCategory {
  _SYNTAX_ERROR = "syntax_error",
  _TYPE_ERROR = "type_error",
  _RUNTIME_ERROR = "runtime_error",
  _LOGIC_ERROR = "logic_error",
  _RESOURCE_ERROR = "resource_error",
  _TIMEOUT_ERROR = "timeout_error",
  _VALIDATION_ERROR = "validation_error",
  _DEPENDENCY_ERROR = "dependency_error",
  _CONFIGURATION_ERROR = "configuration_error",
  _UNKNOWN = "unknown",
}

/**
 * Error pattern with detection metadata
 */
export interface ErrorPattern {
  patternId: string;
  category: ErrorCategory;
  pattern: string;
  frequency: number;
  confidence: number;
  detectedAt: Date;
  remediationStrategy: string;
  successRate: number;
  examples: string[];
}

/**
 * Context snapshot for state preservation
 */
export interface ContextSnapshot {
  snapshotId: string;
  sessionId: string;
  iterationNumber: number;
  timestamp: Date;
  fullContext?: unknown;
  compressedContext: string;
  compressionRatio: number;
  checksumMD5: string;
  sizeBytes: number;
  isDiff: boolean;
  basedOnSnapshotId?: string;
}

/**
 * Context preservation result
 */
export interface ContextPreservationResult {
  snapshotId: string;
  success: boolean;
  compressionRatio: number;
  timeMs: number;
  sizeBytes: number;
  error?: string;
}

/**
 * Context restoration result
 */
export interface ContextRestorationResult {
  success: boolean;
  context: unknown;
  timeMs: number;
  checksumValid: boolean;
  error?: string;
}

/**
 * Iteration feedback for improvement
 */
export interface IterationFeedback {
  feedbackId: string;
  iterationId: string;
  type: FeedbackType;
  confidence: number;
  recommendations: FeedbackRecommendation[];
  successPatterns: string[];
  failurePatterns: string[];
  generatedAt: Date;
}

/**
 * Feedback types
 */
export enum FeedbackType {
  _ERROR_CORRECTION = "error_correction",
  _PERFORMANCE_IMPROVEMENT = "performance_improvement",
  _QUALITY_ENHANCEMENT = "quality_enhancement",
  _APPROACH_SUGGESTION = "approach_suggestion",
  _PATTERN_RECOGNITION = "pattern_recognition",
}

/**
 * Individual feedback recommendation
 */
export interface FeedbackRecommendation {
  priority: RecommendationPriority;
  action: string;
  rationale: string;
  expectedImpact: number;
  appliedAt?: Date;
  effectiveness?: number;
}

/**
 * Recommendation priority levels
 */
export enum RecommendationPriority {
  _CRITICAL = "critical",
  _HIGH = "high",
  _MEDIUM = "medium",
  _LOW = "low",
}

/**
 * Learning summary after session completion
 */
export interface LearningSummary {
  sessionId: string;
  totalIterations: number;
  successfulIterations: number;
  failedIterations: number;
  improvementRate: number;
  finalQualityScore: number;
  initialQualityScore: number;
  totalDurationMs: number;
  errorsDetected: number;
  errorsCorrected: number;
  patternsLearned: string[];
  keyInsights: string[];
  recommendationsApplied: number;
  recommendationsSuccessful: number;
}

/**
 * Adaptive prompt modification
 */
export interface PromptModification {
  modificationId: string;
  sessionId: string;
  iterationNumber: number;
  modificationType: PromptModificationType;
  originalPrompt: string;
  modifiedPrompt: string;
  rationale: string;
  successPatterns: string[];
  failurePatterns: string[];
  appliedAt: Date;
  effectiveness?: number;
}

/**
 * Prompt modification types
 */
export enum PromptModificationType {
  _ADD_CONTEXT = "add_context",
  _REMOVE_NOISE = "remove_noise",
  _EMPHASIZE_PATTERN = "emphasize_pattern",
  _AVOID_PATTERN = "avoid_pattern",
  _CLARIFY_INSTRUCTION = "clarify_instruction",
  _ADD_CONSTRAINT = "add_constraint",
  _SIMPLIFY = "simplify",
}

/**
 * Progress detection result
 */
export interface ProgressDetection {
  hasProgress: boolean;
  improvementDelta: number;
  consecutiveNoProgress: number;
  shouldContinue: boolean;
  reason: string;
}

/**
 * Resource monitoring data
 */
export interface ResourceMonitoring {
  sessionId: string;
  timestamp: Date;
  memoryUsageMB: number;
  memoryLimitMB: number;
  iterationCount: number;
  iterationLimit: number;
  durationMs: number;
  durationLimitMs: number;
  withinLimits: boolean;
  warnings: string[];
}

/**
 * Learning coordinator events
 */
export enum LearningCoordinatorEvent {
  _SESSION_STARTED = "learning:session_started",
  _SESSION_COMPLETED = "learning:session_completed",
  _SESSION_FAILED = "learning:session_failed",
  _ITERATION_STARTED = "learning:iteration_started",
  _ITERATION_COMPLETED = "learning:iteration_completed",
  _ERROR_DETECTED = "learning:error_detected",
  _PATTERN_RECOGNIZED = "learning:pattern_recognized",
  _FEEDBACK_GENERATED = "learning:feedback_generated",
  _PROMPT_MODIFIED = "learning:prompt_modified",
  _CONTEXT_PRESERVED = "learning:context_preserved",
  _QUALITY_THRESHOLD_MET = "learning:quality_threshold_met",
  _RESOURCE_WARNING = "learning:resource_warning",
  _RESOURCE_EXHAUSTED = "learning:resource_exhausted",
  _COORDINATOR_SHUTDOWN = "learning:coordinator_shutdown",
}

/**
 * Event payload for learning coordinator events
 */
export interface LearningEventPayload {
  sessionId: string;
  timestamp: Date;
  eventType: LearningCoordinatorEvent;
  data: unknown;
}

/**
 * Default learning session configuration
 */
export const DEFAULT_LEARNING_CONFIG: LearningSessionConfig = {
  maxIterations: 10,
  progressTimeout: 30000,
  noProgressLimit: 3,
  resourceBudgetMB: 100,
  compressionRatio: 0.7,
  qualityThreshold: 0.85,
  enableAdaptivePrompting: true,
  enableErrorRecognition: true,
};
