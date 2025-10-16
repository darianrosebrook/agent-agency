/**
 * @fileoverview
 * Type definitions for thinking budget management system.
 * Defines interfaces for adaptive token allocation based on task complexity.
 *
 * @author @darianrosebrook
 */

/**
 * Complexity levels for task categorization
 */
export enum ComplexityLevel {
  /** Simple queries, no tool use, minimal context */
  _TRIVIAL = "trivial",
  /** Multi-step reasoning, basic tools, moderate context */
  _STANDARD = "standard",
  /** Advanced reasoning, multiple tools, extensive context */
  _COMPLEX = "complex",
}

/**
 * Budget allocation configuration by complexity level
 */
export interface BudgetTier {
  /** Maximum tokens allowed for this complexity level */
  maxTokens: number;
  /** Complexity level identifier */
  level: ComplexityLevel;
  /** Description of when this tier applies */
  description: string;
}

/**
 * Default budget tiers matching working spec requirements
 */
export const DEFAULT_BUDGET_TIERS: Record<ComplexityLevel, BudgetTier> = {
  [ComplexityLevel.TRIVIAL]: {
    maxTokens: 500,
    level: ComplexityLevel.TRIVIAL,
    description: "Simple queries, no tools, minimal context",
  },
  [ComplexityLevel.STANDARD]: {
    maxTokens: 2000,
    level: ComplexityLevel.STANDARD,
    description: "Multi-step reasoning, basic tools, moderate context",
  },
  [ComplexityLevel.COMPLEX]: {
    maxTokens: 8000,
    level: ComplexityLevel.COMPLEX,
    description: "Advanced reasoning, multiple tools, extensive context",
  },
};

/**
 * Task characteristics used for complexity assessment
 */
export interface TaskCharacteristics {
  /** Number of tools required */
  toolCount: number;
  /** Estimated context size in tokens */
  contextSize: number;
  /** Number of reasoning steps */
  stepCount: number;
  /** Whether task requires multi-turn interaction */
  multiTurn: boolean;
  /** Whether task involves external API calls */
  hasExternalCalls: boolean;
}

/**
 * Budget allocation result
 */
export interface BudgetAllocation {
  /** Allocated token budget */
  allocatedTokens: number;
  /** Assessed complexity level */
  complexityLevel: ComplexityLevel;
  /** Timestamp of allocation */
  allocatedAt: Date;
  /** Unique allocation ID */
  allocationId: string;
  /** Task characteristics used for assessment */
  taskCharacteristics: TaskCharacteristics;
}

/**
 * Budget usage tracking
 */
export interface BudgetUsage {
  /** Allocation ID this usage belongs to */
  allocationId: string;
  /** Tokens consumed so far */
  tokensUsed: number;
  /** Tokens remaining in budget */
  tokensRemaining: number;
  /** Whether budget has been exhausted */
  isExhausted: boolean;
  /** Timestamp of last usage update */
  lastUpdated: Date;
}

/**
 * Budget enforcement result
 */
export interface EnforcementResult {
  /** Whether the requested usage is allowed */
  allowed: boolean;
  /** Reason for denial if not allowed */
  reason?: string;
  /** Current usage state */
  currentUsage: BudgetUsage;
}

/**
 * Budget manager configuration
 */
export interface ThinkingBudgetConfig {
  /** Custom budget tiers (optional) */
  budgetTiers?: Record<ComplexityLevel, BudgetTier>;
  /** Whether to enable strict enforcement */
  strictEnforcement: boolean;
  /** Maximum concurrent allocations to track */
  maxTrackedAllocations: number;
  /** Enable performance monitoring */
  enableMonitoring: boolean;
}

/**
 * Complexity assessment result
 */
export interface ComplexityAssessment {
  /** Assessed complexity level */
  level: ComplexityLevel;
  /** Confidence score (0-1) */
  confidence: number;
  /** Reasoning for the assessment */
  reasoning: string;
  /** Task characteristics analyzed */
  characteristics: TaskCharacteristics;
  /** Assessment duration in milliseconds */
  assessmentTimeMs: number;
}

/**
 * Budget metrics for monitoring
 */
export interface BudgetMetrics {
  /** Total allocations made */
  totalAllocations: number;
  /** Allocations by complexity level */
  allocationsByLevel: Record<ComplexityLevel, number>;
  /** Average tokens allocated */
  averageTokensAllocated: number;
  /** Budget exhaustion rate (0-1) */
  exhaustionRate: number;
  /** Average allocation time in ms */
  averageAllocationTimeMs: number;
}
