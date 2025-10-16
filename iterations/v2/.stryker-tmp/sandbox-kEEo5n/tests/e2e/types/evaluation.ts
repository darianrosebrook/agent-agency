/**
 * E2E Evaluation Type Definitions
 *
 * @author @darianrosebrook
 * @description Type system for end-to-end agent evaluation and testing
 */
// @ts-nocheck


/**
 * Result of evaluating a single criterion
 */
export interface CriterionResult {
  /** Unique identifier for the criterion */
  id: string;

  /** Human-readable name */
  name: string;

  /** Score (0-1) */
  score: number;

  /** Whether the criterion passed its threshold */
  passed: boolean;

  /** Minimum score required to pass */
  threshold: number;

  /** Explanation of the score */
  reasoning: string;

  /** Optional metadata */
  metadata?: Record<string, unknown>;
}

/**
 * Complete evaluation report for an agent output
 */
export interface EvaluationReport {
  /** Overall score across all criteria (0-1) */
  overallScore: number;

  /** Whether all criteria passed */
  overallPassed: boolean;

  /** Individual criterion results */
  criteria: CriterionResult[];

  /** Total evaluation time in milliseconds */
  executionTime: number;

  /** Optional metadata */
  metadata: Record<string, unknown>;
}

/**
 * Result of running a complete E2E test scenario
 */
export interface TestResult {
  /** Whether the test succeeded */
  success: boolean;

  /** Final output from the agent */
  output: unknown;

  /** Number of iterations performed */
  iterations: number;

  /** Feedback provided in each iteration */
  feedbackHistory: string[];

  /** Final evaluation report */
  report: EvaluationReport;

  /** All agent interactions during the test */
  agentInteractions: AgentInteraction[];

  /** Total test execution time in milliseconds */
  totalExecutionTime: number;

  /** Optional error if test failed */
  error?: string;
}

/**
 * Tracks a single agent interaction
 */
export interface AgentInteraction {
  /** Type of interaction */
  type: "tool_call" | "resource_read" | "evaluation" | "generation";

  /** When the interaction occurred */
  timestamp: Date;

  /** Interaction details */
  details: {
    name?: string;
    arguments?: unknown;
    result?: unknown;
    error?: string;
    [key: string]: unknown;
  };

  /** How long the interaction took */
  duration?: number;
}

/**
 * Configuration for iterative evaluation
 */
export interface IterativeConfig {
  /** Maximum number of iterations before stopping */
  maxIterations: number;

  /** Overall score threshold for passing (0-1) */
  passingThreshold: number;

  /** Whether all criteria must pass (true) or just overall score (false) */
  requireAllCriteriaPassed: boolean;

  /** Timeout in milliseconds for each iteration */
  iterationTimeoutMs: number;

  /** Delay between iterations in milliseconds */
  delayBetweenIterationsMs: number;
}

/**
 * Default iterative configuration
 */
export const DEFAULT_ITERATIVE_CONFIG: IterativeConfig = {
  maxIterations: 3,
  passingThreshold: 0.8,
  requireAllCriteriaPassed: true,
  iterationTimeoutMs: 120000, // 2 minutes
  delayBetweenIterationsMs: 1000, // 1 second
};

/**
 * Evaluation function type for custom criteria
 */
export type EvaluationFunction = (
  output: unknown,
  context: Record<string, unknown>
) => Promise<CriterionResult>;

/**
 * Definition of an evaluation criterion
 */
export interface EvaluationCriterion {
  /** Unique identifier */
  id: string;

  /** Human-readable name */
  name: string;

  /** Description of what this criterion evaluates */
  description: string;

  /** Evaluation function */
  evaluate: EvaluationFunction;

  /** Minimum score to pass (0-1) */
  threshold: number;

  /** Optional weight for overall score calculation */
  weight?: number;
}

/**
 * Context passed to generation functions
 */
export interface GenerationContext {
  /** Current iteration number (1-based) */
  iteration: number;

  /** Previous output (if any) */
  previousOutput: unknown;

  /** Feedback history from previous iterations */
  feedbackHistory: string[];

  /** Additional context data */
  [key: string]: unknown;
}

/**
 * Statistics about a test run
 */
export interface TestStatistics {
  /** Total iterations performed */
  totalIterations: number;

  /** Total time spent generating outputs */
  totalGenerationTimeMs: number;

  /** Total time spent evaluating outputs */
  totalEvaluationTimeMs: number;

  /** Total time including delays */
  totalTestTimeMs: number;

  /** Number of tool calls made */
  totalToolCalls: number;

  /** Number of evaluations performed */
  totalEvaluations: number;

  /** Average score across iterations */
  averageScore: number;

  /** Score improvement from first to last iteration */
  scoreImprovement: number;
}
