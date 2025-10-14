/**
 * @fileoverview GPT-5 Prompting Control Types and Interfaces
 *
 * Core types for implementing GPT-5 prompting techniques including
 * reasoning effort control, agent eagerness management, and
 * structured prompt processing.
 *
 * @author @darianrosebrook
 */

// Re-export commonly used types
export { VerificationPriority } from "./verification";

/**
 * Reasoning effort levels for GPT-5 API calls
 */
export type ReasoningEffort = "low" | "medium" | "high";

/**
 * Agent eagerness calibration levels
 */
export type AgentEagerness = "minimal" | "balanced" | "thorough" | "exhaustive";

/**
 * Context gathering strategies
 */
export type ContextGatheringStrategy = "serial" | "parallel" | "hybrid";

/**
 * Tool call budget allocation
 */
export interface ToolBudget {
  /** Maximum number of tool calls allowed */
  maxCalls: number;

  /** Current calls used */
  usedCalls: number;

  /** Budget reset interval in milliseconds */
  resetIntervalMs: number;

  /** Last reset timestamp */
  lastResetAt: Date;

  /** Escalation rules for budget increases */
  escalationRules: BudgetEscalationRule[];
}

/**
 * Budget escalation trigger conditions
 */
export interface BudgetEscalationRule {
  /** Trigger condition */
  trigger:
    | "low-confidence"
    | "partial-success"
    | "verifier-rejection"
    | "complexity-increase";

  /** Additional calls to allocate */
  additionalCalls: number;

  /** Maximum total budget after escalation */
  maxTotalBudget: number;

  /** Cooldown period before next escalation */
  cooldownMs: number;
}

/**
 * XML-like structured prompt instruction
 */
export interface StructuredPromptInstruction {
  /** XML tag name */
  tag: string;

  /** Instruction attributes */
  attributes: Record<string, string>;

  /** Nested instructions */
  children?: StructuredPromptInstruction[];

  /** Text content */
  content?: string;
}

/**
 * Context gathering configuration
 */
export interface ContextGatheringConfig {
  /** Knowledge seeker for real search operations */
  knowledgeSeeker?: any; // KnowledgeSeeker instance

  /** Gathering strategy */
  strategy: ContextGatheringStrategy;

  /** Maximum parallel queries */
  maxParallelQueries: number;

  /** Early stop criteria */
  earlyStopCriteria: {
    /** Stop when convergence reaches this percentage */
    convergenceThreshold: number;

    /** Maximum gathering time */
    maxTimeMs: number;

    /** Minimum information quality score */
    minQualityScore: number;
  };

  /** Search depth limits */
  depthLimits: {
    /** Very low depth (fastest) */
    veryLow: number;

    /** Low depth */
    low: number;

    /** Medium depth */
    medium: number;

    /** High depth (slowest) */
    high: number;
  };
}

/**
 * Self-reflection rubric for task planning
 */
export interface SelfReflectionRubric {
  /** Rubric categories */
  categories: RubricCategory[];

  /** Overall quality threshold for acceptance */
  acceptanceThreshold: number;

  /** Maximum iterations allowed */
  maxIterations: number;
}

/**
 * Individual rubric category
 */
export interface RubricCategory {
  /** Category name */
  name: string;

  /** Category description */
  description: string;

  /** Scoring criteria */
  criteria: ScoringCriterion[];

  /** Category weight in overall score */
  weight: number;
}

/**
 * Scoring criterion within a rubric category
 */
export interface ScoringCriterion {
  /** Criterion description */
  description: string;

  /** Point value for this criterion */
  points: number;

  /** Evaluation function */
  evaluate: (task: Task, context: TaskContext) => Promise<number>;
}

/**
 * Agent control configuration
 */
export interface AgentControlConfig {
  /** Reasoning effort settings */
  reasoningEffort: {
    /** Default effort level */
    default: ReasoningEffort;

    /** Task complexity to effort mapping */
    complexityMapping: Record<TaskComplexity, ReasoningEffort>;

    /** Dynamic adjustment enabled */
    dynamicAdjustment: boolean;
  };

  /** Eagerness calibration */
  eagerness: {
    /** Default eagerness level */
    default: AgentEagerness;

    /** Task type to eagerness mapping */
    taskTypeMapping: Record<TaskType, AgentEagerness>;

    /** Maximum tool calls for minimal eagerness */
    minimalMaxCalls: number;

    /** Maximum tool calls for balanced eagerness */
    balancedMaxCalls: number;
  };

  /** Tool budget management */
  toolBudget: {
    /** Enable budget enforcement */
    enabled: boolean;

    /** Default budget per task type */
    defaultBudgets: Record<TaskType, ToolBudget>;

    /** Global budget limits */
    globalLimits: {
      maxConcurrentBudgets: number;
      totalDailyCalls: number;
    };
  };

  /** Context gathering settings */
  contextGathering: ContextGatheringConfig;

  /** Self-reflection settings */
  selfReflection: {
    /** Enable self-reflection for complex tasks */
    enabled: boolean;

    /** Task complexity threshold for reflection */
    complexityThreshold: TaskComplexity;

    /** Default rubric for self-reflection */
    defaultRubric: SelfReflectionRubric;
  };
}

/**
 * Prompting engine result
 */
export interface PromptingResult {
  /** Selected reasoning effort */
  reasoningEffort: ReasoningEffort;

  /** Calibrated eagerness level */
  eagerness: AgentEagerness;

  /** Allocated tool budget */
  toolBudget: ToolBudget;

  /** Context gathering configuration */
  contextConfig: ContextGatheringConfig;

  /** Self-reflection rubric (if applicable) */
  reflectionRubric?: SelfReflectionRubric;

  /** Structured prompt instructions */
  structuredInstructions: StructuredPromptInstruction[];

  /** Processing metadata */
  metadata: {
    processingTimeMs: number;
    confidence: number;
    appliedOptimizations: string[];
  };
}

/**
 * Task complexity assessment
 */
export type TaskComplexity = "trivial" | "standard" | "complex" | "expert";

/**
 * Task type classification
 */
export type TaskType =
  | "analysis"
  | "creation"
  | "modification"
  | "research"
  | "planning"
  | "execution";

/**
 * Task context for prompting decisions
 */
export interface TaskContext {
  /** Task complexity assessment */
  complexity: TaskComplexity;

  /** Task type */
  type: TaskType;

  /** Available time budget */
  timeBudgetMs?: number;

  /** Required accuracy level */
  accuracyRequirement: "draft" | "standard" | "high" | "critical";

  /** Previous task performance metrics */
  historicalMetrics?: {
    averageCompletionTime: number;
    successRate: number;
    toolEfficiency: number;
  };
}

/**
 * Agent behavior metrics
 */
export interface AgentBehaviorMetrics {
  /** Reasoning effort effectiveness */
  reasoningEffectiveness: number;

  /** Tool utilization efficiency */
  toolEfficiency: number;

  /** Context gathering quality */
  contextQuality: number;

  /** Task completion accuracy */
  completionAccuracy: number;

  /** Response time performance */
  responseTimeMs: number;
}

// Re-export for backward compatibility
export interface Task {
  id: string;
  type: TaskType;
  complexity: TaskComplexity;
  description: string;
  context?: TaskContext;
}
