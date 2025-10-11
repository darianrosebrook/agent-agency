/**
 * Agentic Reinforcement Learning Type Definitions
 *
 * @author @darianrosebrook
 * @module agentic-rl
 *
 * Type definitions for reinforcement learning components in Agent Agency V2.
 * Includes multi-armed bandit, turn-level RL, tool adoption, and performance tracking.
 */
// @ts-nocheck


/**
 * Unique identifier for RL-related entities.
 */
export type RLIdentifier = string;

/**
 * Timestamp in ISO 8601 format with millisecond precision.
 */
export type Timestamp = string;

/**
 * Conversation identifier for tracking multi-turn interactions.
 */
export type ConversationId = string;

/**
 * Task identifier for tracking individual task executions.
 */
export type TaskId = string;

/**
 * Reward value for RL training (typically between -1 and 1).
 */
export type Reward = number;

/**
 * Confidence score (0.0 - 1.0).
 */
export type Confidence = number;

/**
 * Probability value (0.0 - 1.0).
 */
export type Probability = number;

/**
 * Exploration rate for multi-armed bandit algorithms.
 */
export type ExplorationRate = number;

/**
 * Learning rate for RL algorithms.
 */
export type LearningRate = number;

/**
 * Discount factor for temporal credit assignment.
 */
export type DiscountFactor = number;

/**
 * Advantage value for policy gradient algorithms.
 */
export type Advantage = number;

/**
 * Log probability for policy gradient computations.
 */
export type LogProbability = number;

/**
 * Token count for thinking budget management.
 */
export type TokenCount = number;

/**
 * Complexity level for task categorization.
 */
export type ComplexityLevel = "trivial" | "standard" | "complex";

/**
 * Tool call identifier.
 */
export type ToolCallId = string;

/**
 * AST similarity score (0.0 - 1.0).
 */
export type ASTSimilarity = number;

/**
 * File change count for minimal diff analysis.
 */
export type FileChangeCount = number;

/**
 * Judge type for model-based evaluation.
 */
export type JudgeType =
  | "relevance"
  | "faithfulness"
  | "minimality"
  | "safety"
  | "creativity";

/**
 * Routing strategy for task-to-agent assignment.
 */
export type RoutingStrategy =
  | "multi-armed-bandit"
  | "capability-match"
  | "load-balance"
  | "random";

/**
 * Budget escalation trigger types.
 */
export type BudgetTrigger =
  | "low-confidence"
  | "partial-success"
  | "verifier-rejection";

/**
 * Tool reward signal components.
 */
export interface ToolRewardSignal {
  /**
   * Whether the tool call structure is valid (correct JSON format and schema).
   */
  callStructureValid: boolean;

  /**
   * Whether the chosen tool is appropriate for the task.
   */
  toolChoiceAppropriate: boolean;

  /**
   * Information utility score (0-1): How useful the tool result was.
   */
  informationUtility: number;

  /**
   * Whether error handling was correct.
   */
  errorHandlingCorrect: boolean;

  /**
   * Efficiency score based on token usage.
   */
  efficiency: number;

  /**
   * Weighted combination of all factors.
   */
  totalReward: Reward;
}

/**
 * Turn-level reward structure for multi-turn conversations.
 */
export interface TurnLevelReward {
  /**
   * Turn number in the conversation (1-based).
   */
  turnNumber: number;

  /**
   * Tool call made in this turn.
   */
  toolChoice: ToolCall;

  /**
   * Information gain from this turn (0-1).
   */
  informationGain: number;

  /**
   * Format correctness score (0-1).
   */
  formatCorrectness: number;

  /**
   * Task progress contribution (0-1): How much closer to completion.
   */
  taskProgress: number;

  /**
   * Safety score (0-1): Prevention of harmful actions.
   */
  safetyScore: number;

  /**
   * Total reward for this turn.
   */
  totalReward: Reward;
}

/**
 * Conversation trajectory for RL training.
 */
export interface ConversationTrajectory {
  /**
   * Unique conversation identifier.
   */
  conversationId: ConversationId;

  /**
   * Turn-level rewards for each turn in the conversation.
   */
  turns: TurnLevelReward[];

  /**
   * Final task outcome.
   */
  finalOutcome: TaskOutcome;

  /**
   * Total reward accumulated across the conversation.
   */
  totalReward: Reward;
}

/**
 * Turn data point for RL training with advantage computation.
 */
export interface TurnData {
  /**
   * Turn number in the conversation.
   */
  turnNumber: number;

  /**
   * Conversation state at the start of this turn.
   */
  state: ConversationState;

  /**
   * Action taken (tool call).
   */
  action: ToolCall;

  /**
   * Reward received for this turn.
   */
  reward: Reward;

  /**
   * Computed advantage for policy gradient.
   */
  advantage: Advantage;

  /**
   * Log probability of the action under current policy.
   */
  logProb: LogProbability;
}

/**
 * Trajectory for RL training with full turn data.
 */
export interface RLTrajectory {
  /**
   * Conversation identifier.
   */
  conversationId: ConversationId;

  /**
   * Turn-level data points.
   */
  turns: TurnData[];

  /**
   * Final outcome of the conversation.
   */
  finalOutcome: TaskOutcome;

  /**
   * Total accumulated reward.
   */
  totalReward: Reward;
}

/**
 * Conversation state representation.
 */
export interface ConversationState {
  /**
   * Current task context.
   */
  taskContext: TaskContext;

  /**
   * History of previous turns.
   */
  turnHistory: TurnSummary[];

  /**
   * Available tools at this point.
   */
  availableTools: Tool[];

  /**
   * Current thinking budget remaining.
   */
  remainingBudget: TokenCount;
}

/**
 * Summary of a completed turn.
 */
export interface TurnSummary {
  /**
   * Turn number.
   */
  turnNumber: number;

  /**
   * Tool called in this turn.
   */
  toolCalled: ToolCallId | null;

  /**
   * Whether the tool call was successful.
   */
  success: boolean;

  /**
   * Tokens consumed in this turn.
   */
  tokensConsumed: TokenCount;
}

/**
 * Task context information.
 */
export interface TaskContext {
  /**
   * Task identifier.
   */
  taskId: TaskId;

  /**
   * Task type/category.
   */
  taskType: string;

  /**
   * Task complexity level.
   */
  complexity: ComplexityLevel;

  /**
   * Task requirements and constraints.
   */
  requirements: Record<string, unknown>;
}

/**
 * Tool definition for tool adoption training.
 */
export interface Tool {
  /**
   * Tool identifier.
   */
  id: ToolCallId;

  /**
   * Tool name.
   */
  name: string;

  /**
   * Tool description.
   */
  description: string;

  /**
   * Tool parameters schema.
   */
  parameters: JSONSchema;
}

/**
 * Tool call representation.
 */
export interface ToolCall {
  /**
   * Tool identifier.
   */
  toolId: ToolCallId;

  /**
   * Parameters passed to the tool.
   */
  parameters: Record<string, unknown>;

  /**
   * Tool call result.
   */
  result?: unknown;
}

/**
 * Task outcome for evaluation.
 */
export interface TaskOutcome {
  /**
   * Whether the task was completed successfully.
   */
  success: boolean;

  /**
   * Quality score (0-1).
   */
  qualityScore: number;

  /**
   * Efficiency score (0-1).
   */
  efficiencyScore: number;

  /**
   * Tokens consumed.
   */
  tokensConsumed: TokenCount;

  /**
   * Completion time in milliseconds.
   */
  completionTimeMs: number;
}

/**
 * Multi-armed bandit configuration.
 */
export interface BanditConfig {
  /**
   * Exploration rate (0-1). Higher values mean more exploration.
   */
  explorationRate: ExplorationRate;

  /**
   * Rate at which exploration decays over time.
   */
  decayFactor: number;

  /**
   * Minimum samples needed before trusting statistics.
   */
  minSampleSize: number;

  /**
   * Whether to use Upper Confidence Bound scoring.
   */
  useUCB: boolean;

  /**
   * UCB exploration parameter.
   */
  ucbConstant: number;
}

/**
 * Routing decision result from multi-armed bandit.
 */
export interface RoutingDecision {
  /**
   * Task identifier.
   */
  taskId: TaskId;

  /**
   * Selected agent identifier.
   */
  selectedAgent: string;

  /**
   * Routing strategy used.
   */
  routingStrategy: RoutingStrategy;

  /**
   * Confidence in the selection (0-1).
   */
  confidence: Confidence;

  /**
   * Alternative agents considered with scores.
   */
  alternativesConsidered: RoutingAlternative[];

  /**
   * Human-readable rationale for the decision.
   */
  rationale: string;

  /**
   * Timestamp of the decision.
   */
  timestamp: Timestamp;
}

/**
 * Alternative agent considered during routing.
 */
export interface RoutingAlternative {
  /**
   * Agent identifier.
   */
  agentId: string;

  /**
   * Selection score.
   */
  score: number;

  /**
   * Reason for the score.
   */
  reason: string;
}

/**
 * Thinking budget configuration.
 */
export interface ThinkingBudgetConfig {
  /**
   * Default token budgets by complexity.
   */
  defaultBudgets: Record<ComplexityLevel, TokenCount>;

  /**
   * Escalation rules for budget increases.
   */
  escalationRules: BudgetEscalationRule[];

  /**
   * Monitoring configuration.
   */
  monitoring: BudgetMonitoringConfig;
}

/**
 * Budget escalation rule.
 */
export interface BudgetEscalationRule {
  /**
   * Trigger condition for escalation.
   */
  trigger: BudgetTrigger;

  /**
   * Additional tokens to allocate on trigger.
   */
  additionalTokens: TokenCount;

  /**
   * Maximum total budget allowed.
   */
  maxTotalBudget: TokenCount;

  /**
   * Cooldown period before another escalation (milliseconds).
   */
  cooldownPeriod: number;
}

/**
 * Budget monitoring configuration.
 */
export interface BudgetMonitoringConfig {
  /**
   * Whether to enable monitoring.
   */
  enabled: boolean;

  /**
   * Monitoring interval (milliseconds).
   */
  intervalMs: number;

  /**
   * Alert thresholds.
   */
  thresholds: {
    warningPercent: number;
    criticalPercent: number;
  };
}

/**
 * Thinking budget for a task.
 */
export interface ThinkingBudget {
  /**
   * Task identifier.
   */
  taskId: TaskId;

  /**
   * Task complexity level.
   */
  complexity: ComplexityLevel;

  /**
   * Initially allocated tokens.
   */
  allocatedTokens: TokenCount;

  /**
   * Tokens consumed so far.
   */
  consumedTokens: TokenCount;

  /**
   * Efficiency ratio (allocated/consumed).
   */
  efficiency: number;

  /**
   * Escalation triggers that have occurred.
   */
  escalationTriggers: BudgetTrigger[];

  /**
   * Maximum total budget allowed.
   */
  maxTotalBudget: TokenCount;

  /**
   * Timestamp of last update.
   */
  lastUpdated: Timestamp;
}

/**
 * Budget action result.
 */
export type BudgetAction = "CONTINUE" | "ESCALATE" | "EXHAUST";

/**
 * Minimal diff analysis result.
 */
export interface DiffAnalysis {
  /**
   * AST similarity score (0-1).
   */
  astSimilarity: ASTSimilarity;

  /**
   * File changes made.
   */
  fileChanges: FileChange[];

  /**
   * Line efficiency ratio.
   */
  lineEfficiency: number;

  /**
   * Scaffolding score (0-1, lower is better).
   */
  scaffoldingScore: number;

  /**
   * Computed reward multiplier.
   */
  rewardMultiplier: number;
}

/**
 * File change in a diff.
 */
export interface FileChange {
  /**
   * File path.
   */
  path: string;

  /**
   * Change type.
   */
  changeType: "add" | "modify" | "delete";

  /**
   * AST delta information.
   */
  astDiff: ASTDelta;

  /**
   * Number of lines changed.
   */
  lineCount: number;
}

/**
 * AST difference representation.
 */
export interface ASTDelta {
  /**
   * Tree edit distance.
   */
  distance: number;

  /**
   * Similarity score (0-1).
   */
  similarity: number;

  /**
   * Changed node types.
   */
  changedNodes: string[];
}

/**
 * Minimal diff metrics for evaluation.
 */
export interface MinimalDiffMetrics {
  /**
   * AST similarity (0-1).
   */
  astSimilarity: ASTSimilarity;

  /**
   * Number of files touched.
   */
  fileTouchCount: FileChangeCount;

  /**
   * Ratio of changed lines to total lines.
   */
  lineChangeRatio: number;

  /**
   * Scaffolding penalty (0-1).
   */
  scaffoldingPenalty: number;

  /**
   * Whether the solution is functionally equivalent.
   */
  functionalEquivalence: boolean;
}

/**
 * Model-based judgment result.
 */
export interface JudgmentResult {
  /**
   * Score (0-1 confidence score).
   */
  score: number;

  /**
   * Model's reasoning explanation.
   */
  reasoning: string;

  /**
   * Model's self-assessed confidence.
   */
  confidence: Confidence;

  /**
   * Additional context metadata.
   */
  metadata: Record<string, unknown>;
}

/**
 * Model judge configuration.
 */
export interface ModelJudge {
  /**
   * Judge type.
   */
  type: JudgeType;

  /**
   * Model to use for judging.
   */
  model: string;

  /**
   * Prompt template for the judge.
   */
  promptTemplate: string;

  /**
   * Response schema.
   */
  responseSchema: JSONSchema;

  /**
   * Weight in final scoring (0-1).
   */
  weight: number;

  /**
   * Confidence threshold for acceptance.
   */
  confidenceThreshold: Confidence;
}

/**
 * Tool example for supervised fine-tuning.
 */
export interface ToolExample {
  /**
   * Input prompt.
   */
  prompt: string;

  /**
   * Correct tool call.
   */
  correctToolCall: ToolCall;

  /**
   * Expected reasoning.
   */
  expectedReasoning: string;

  /**
   * Difficulty level.
   */
  difficulty: "easy" | "medium" | "hard";
}

/**
 * RL training configuration.
 */
export interface RLTrainingConfig {
  /**
   * Learning rate for policy updates.
   */
  learningRate: LearningRate;

  /**
   * Discount factor for reward discounting.
   */
  discountFactor: DiscountFactor;

  /**
   * Batch size for training updates.
   */
  batchSize: number;

  /**
   * Number of epochs per training cycle.
   */
  epochs: number;

  /**
   * Gradient clipping threshold.
   */
  gradientClip: number;

  /**
   * KL divergence penalty coefficient.
   */
  klPenalty: number;

  /**
   * Minimum trajectory length for training.
   */
  minTrajectoryLength: number;

  /**
   * Maximum trajectory length for training.
   */
  maxTrajectoryLength: number;
}

/**
 * RL training statistics.
 */
export interface RLTrainingStats {
  /**
   * Number of trajectories processed.
   */
  trajectoriesProcessed: number;

  /**
   * Average reward across trajectories.
   */
  averageReward: Reward;

  /**
   * Policy loss.
   */
  policyLoss: number;

  /**
   * Value loss.
   */
  valueLoss: number;

  /**
   * KL divergence.
   */
  klDivergence: number;

  /**
   * Training time in milliseconds.
   */
  trainingTimeMs: number;

  /**
   * Timestamp of training completion.
   */
  timestamp: Timestamp;
}

/**
 * Performance tracking event.
 */
export interface PerformanceEvent {
  /**
   * Event type.
   */
  type: string;

  /**
   * Timestamp of the event.
   */
  timestamp: Timestamp;

  /**
   * Event data.
   */
  data: Record<string, unknown>;
}

/**
 * JSON Schema definition for validation.
 */
export interface JSONSchema {
  type: string;
  properties?: Record<string, JSONSchema>;
  required?: string[];
  items?: JSONSchema;
  [key: string]: unknown;
}

/**
 * Error types for RL operations.
 */
export enum RLErrorType {
  INVALID_TRAJECTORY = "INVALID_TRAJECTORY",
  TRAINING_FAILED = "TRAINING_FAILED",
  BUDGET_EXCEEDED = "BUDGET_EXCEEDED",
  INVALID_CONFIG = "INVALID_CONFIG",
  MODEL_ERROR = "MODEL_ERROR",
}

/**
 * RL operation error with context.
 */
export class RLError extends Error {
  constructor(
    public readonly type: RLErrorType,
    message: string,
    public readonly context?: Record<string, unknown>
  ) {
    super(message);
    this.name = "RLError";
  }
}
