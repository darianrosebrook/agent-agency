/**
 * Learning Coordination Type Definitions
 *
 * Type contracts for ARBITER-009 Multi-Turn Learning Coordinator
 * Defines interfaces for iterative agent learning, error pattern recognition,
 * and adaptive prompting systems.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


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
  INITIALIZING = "initializing",
  ACTIVE = "active",
  EVALUATING = "evaluating",
  COMPLETED = "completed",
  FAILED = "failed",
  TIMEOUT = "timeout",
  RESOURCE_EXHAUSTED = "resource_exhausted",
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
  SYNTAX_ERROR = "syntax_error",
  TYPE_ERROR = "type_error",
  RUNTIME_ERROR = "runtime_error",
  LOGIC_ERROR = "logic_error",
  RESOURCE_ERROR = "resource_error",
  TIMEOUT_ERROR = "timeout_error",
  VALIDATION_ERROR = "validation_error",
  DEPENDENCY_ERROR = "dependency_error",
  CONFIGURATION_ERROR = "configuration_error",
  UNKNOWN = "unknown",
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
  ERROR_CORRECTION = "error_correction",
  PERFORMANCE_IMPROVEMENT = "performance_improvement",
  QUALITY_ENHANCEMENT = "quality_enhancement",
  APPROACH_SUGGESTION = "approach_suggestion",
  PATTERN_RECOGNITION = "pattern_recognition",
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
  CRITICAL = "critical",
  HIGH = "high",
  MEDIUM = "medium",
  LOW = "low",
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
  ADD_CONTEXT = "add_context",
  REMOVE_NOISE = "remove_noise",
  EMPHASIZE_PATTERN = "emphasize_pattern",
  AVOID_PATTERN = "avoid_pattern",
  CLARIFY_INSTRUCTION = "clarify_instruction",
  ADD_CONSTRAINT = "add_constraint",
  SIMPLIFY = "simplify",
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
  SESSION_STARTED = "learning:session_started",
  SESSION_COMPLETED = "learning:session_completed",
  SESSION_FAILED = "learning:session_failed",
  ITERATION_STARTED = "learning:iteration_started",
  ITERATION_COMPLETED = "learning:iteration_completed",
  ERROR_DETECTED = "learning:error_detected",
  PATTERN_RECOGNIZED = "learning:pattern_recognized",
  FEEDBACK_GENERATED = "learning:feedback_generated",
  PROMPT_MODIFIED = "learning:prompt_modified",
  CONTEXT_PRESERVED = "learning:context_preserved",
  QUALITY_THRESHOLD_MET = "learning:quality_threshold_met",
  RESOURCE_WARNING = "learning:resource_warning",
  RESOURCE_EXHAUSTED = "learning:resource_exhausted",
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
