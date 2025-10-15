/**
 * Turn-Level RL Training System
 *
 * @author @darianrosebrook
 * @module turn-level-rl-trainer
 *
 * Implements Group Relative Policy Optimization (GRPO) for training on multi-turn
 * conversations with intermediate rewards for tool usage and information gain.
 */
// @ts-nocheck


import { MinimalDiffEvaluator } from "../evaluation/MinimalDiffEvaluator";
import { ModelBasedJudge } from "../evaluation/ModelBasedJudge";
import { ThinkingBudgetManager } from "../thinking/ThinkingBudgetManager";
import {
  ConversationTrajectory,
  RLError,
  RLErrorType,
  RLTrainingConfig,
  RLTrainingStats,
  RLTrajectory,
  TurnData,
  TurnLevelReward,
} from "../types/agentic-rl";
import type { JudgmentInput } from "../types/judge";

/**
 * Turn-Level RL Trainer implementing GRPO algorithm.
 *
 * This trainer processes multi-turn conversation trajectories and computes
 * turn-level rewards for tool choice, information gain, and task progress.
 * Uses Group Relative Policy Optimization to update agent policies.
 */
export class TurnLevelRLTrainer {
  private config: RLTrainingConfig;
  private trainingStats: RLTrainingStats;
  private budgetManager: ThinkingBudgetManager;
  private diffEvaluator: MinimalDiffEvaluator;
  private judge: ModelBasedJudge;

  /**
   * Creates a new turn-level RL trainer.
   *
   * @param config - Training configuration. Uses defaults if not provided.
   */
  constructor(config: Partial<RLTrainingConfig> = {}) {
    this.config = {
      learningRate: 1e-5,
      discountFactor: 0.99,
      batchSize: 32,
      epochs: 3,
      gradientClip: 1.0,
      klPenalty: 0.1,
      minTrajectoryLength: 2,
      maxTrajectoryLength: 50,
      ...config,
    };

    this.trainingStats = {
      trajectoriesProcessed: 0,
      averageReward: 0,
      policyLoss: 0,
      valueLoss: 0,
      klDivergence: 0,
      trainingTimeMs: 0,
      timestamp: new Date().toISOString(),
    };

    // Initialize RL components
    this.budgetManager = new ThinkingBudgetManager();
    this.diffEvaluator = new MinimalDiffEvaluator();
    this.judge = new ModelBasedJudge();
  }

  /**
   * Trains on a batch of conversation trajectories.
   *
   * @param trajectories - Batch of conversation trajectories.
   * @returns Training update result.
   */
  async trainOnTrajectories(
    trajectories: ConversationTrajectory[]
  ): Promise<RLTrainingStats> {
    const startTime = Date.now();

    // Validate trajectories
    const validTrajectories = this.validateTrajectories(trajectories);
    if (validTrajectories.length === 0) {
      throw new RLError(
        RLErrorType.INVALID_TRAJECTORY,
        "No valid trajectories provided for training"
      );
    }

    // Convert to RL trajectories with turn-level data
    const rlTrajectories = await this.convertToRLTrajectories(
      validTrajectories
    );

    // Group trajectories by similarity for GRPO
    const trajectoryGroups = this.groupTrajectoriesBySimilarity(rlTrajectories);

    // Compute advantages using GRPO
    const trajectoriesWithAdvantages = await this.computeGroupAdvantages(
      trajectoryGroups
    );

    // Update policy network
    const policyUpdate = await this.updatePolicy(trajectoriesWithAdvantages);

    // Update training statistics
    const trainingTime = Date.now() - startTime;
    this.updateTrainingStats(rlTrajectories, policyUpdate, trainingTime);

    return { ...this.trainingStats };
  }

  /**
   * Trains on a single conversation.
   *
   * @param conversation - Conversation trajectory.
   * @returns Training update result.
   */
  async trainOnConversation(
    conversation: ConversationTrajectory
  ): Promise<RLTrainingStats> {
    return this.trainOnTrajectories([conversation]);
  }

  /**
   * Computes turn-level rewards for a conversation trajectory.
   *
   * @param trajectory - Conversation trajectory.
   * @returns Trajectory with computed turn-level rewards.
   */
  async computeTurnRewards(
    trajectory: ConversationTrajectory
  ): Promise<ConversationTrajectory> {
    const rewardedTurns: TurnLevelReward[] = [];

    for (let i = 0; i < trajectory.turns.length; i++) {
      const turn = trajectory.turns[i];
      const turnReward = await this.computeSingleTurnReward(
        turn,
        trajectory,
        i
      );

      rewardedTurns.push(turnReward);
    }

    return {
      ...trajectory,
      turns: rewardedTurns,
    };
  }

  /**
   * Computes reward for a single turn.
   *
   * @param turn - Turn data.
   * @param trajectory - Full conversation trajectory.
   * @param turnIndex - Index of the turn in the conversation.
   * @returns Computed turn-level reward.
   */
  private async computeSingleTurnReward(
    turn: TurnLevelReward,
    trajectory: ConversationTrajectory,
    turnIndex: number
  ): Promise<TurnLevelReward> {
    // Information gain: How much new relevant information was retrieved
    const informationGain = await this.judgeInformationGain(turn);

    // Format correctness: Was the tool call properly formatted
    const formatCorrectness = this.evaluateFormatCorrectness(turn);

    // Task progress: How much closer to completion this turn brought us
    const taskProgress = await this.assessTaskProgress(
      turn,
      trajectory,
      turnIndex
    );

    // Safety score: Did this turn avoid harmful actions
    const safetyScore = this.evaluateSafety(turn);

    // Minimality score: Evaluate code change quality (if applicable)
    const minimalityFactor = await this.evaluateMinimality(turn);

    // Combine rewards with weights and apply minimality factor
    const baseReward =
      informationGain * 0.4 +
      formatCorrectness * 0.3 +
      taskProgress * 0.2 +
      safetyScore * 0.1;

    // Apply minimality multiplier (0.1-1.0) to encourage concise changes
    const totalReward = baseReward * minimalityFactor;

    return {
      ...turn,
      informationGain,
      formatCorrectness,
      taskProgress,
      safetyScore,
      totalReward,
    };
  }

  /**
   * Judges the information gain of a turn using ModelBasedJudge.
   *
   * @param turn - Turn to evaluate.
   * @returns Information gain score (0-1).
   */
  private async judgeInformationGain(turn: TurnLevelReward): Promise<number> {
    try {
      // Use ModelBasedJudge for subjective assessment
      const judgmentInput: JudgmentInput = {
        task: `Evaluate tool usage: ${turn.toolChoice.toolId}`,
        output: JSON.stringify(turn.toolChoice.parameters),
        context: {
          toolId: turn.toolChoice.toolId,
          turnNumber: turn.turnNumber,
        },
      };

      const result = await this.judge.evaluate(judgmentInput);

      // Use relevance score as information gain proxy
      const relevanceAssessment = result.assessments.find(
        (a) => a.criterion === "relevance"
      );

      return relevanceAssessment?.score ?? 0.5;
    } catch (error) {
      // Fallback to heuristic on error
      const informationTools = ["search", "read_file", "list_dir", "grep"];
      const toolName = turn.toolChoice.toolId.toLowerCase();
      return informationTools.some((tool) => toolName.includes(tool))
        ? 0.8
        : 0.5;
    }
  }

  /**
   * Evaluates format correctness of a tool call.
   *
   * @param turn - Turn to evaluate.
   * @returns Format correctness score (0-1).
   */
  private evaluateFormatCorrectness(turn: TurnLevelReward): number {
    // Check if tool call structure is valid
    try {
      // Basic validation - in practice, this would use schema validation
      if (
        turn.toolChoice.toolId &&
        typeof turn.toolChoice.parameters === "object"
      ) {
        return 1.0; // Perfect format
      }
      return 0.0; // Invalid format
    } catch {
      return 0.0; // Exception during validation
    }
  }

  /**
   * Assesses task progress contribution of a turn.
   *
   * @param turn - Turn to evaluate.
   * @param trajectory - Full conversation trajectory.
   * @param turnIndex - Index of the turn.
   * @returns Task progress score (0-1).
   */
  private async assessTaskProgress(
    turn: TurnLevelReward,
    trajectory: ConversationTrajectory,
    turnIndex: number
  ): Promise<number> {
    const totalTurns = trajectory.turns.length;
    const progressThroughConversation = turnIndex / Math.max(totalTurns - 1, 1);

    // Turns early in conversation get higher progress credit if they use tools
    // that are typically needed for task completion
    const essentialTools = ["read_file", "grep", "search", "run_terminal_cmd"];
    const toolName = turn.toolChoice.toolId.toLowerCase();

    if (essentialTools.some((tool) => toolName.includes(tool))) {
      return Math.max(0.6, 1.0 - progressThroughConversation * 0.4);
    }

    // Later turns get credit for completion-oriented actions
    if (progressThroughConversation > 0.7) {
      return 0.8; // High credit for actions near completion
    }

    return 0.4; // Moderate credit for other actions
  }

  /**
   * Evaluates safety of a turn.
   *
   * @param turn - Turn to evaluate.
   * @returns Safety score (0-1).
   */
  private evaluateSafety(turn: TurnLevelReward): number {
    // Check for potentially harmful tool calls
    const dangerousTools = ["run_terminal_cmd"];
    const riskyCommands = ["rm", "del", "delete", "format", "fdisk"];

    const toolName = turn.toolChoice.toolId.toLowerCase();

    if (dangerousTools.includes(toolName)) {
      // Check if parameters contain risky commands
      const params = JSON.stringify(turn.toolChoice.parameters).toLowerCase();
      if (riskyCommands.some((cmd) => params.includes(cmd))) {
        return 0.2; // Low safety score for risky commands
      }
    }

    return 0.9; // High safety score for most actions
  }

  /**
   * Evaluates minimality of code changes using MinimalDiffEvaluator.
   *
   * @param turn - Turn to evaluate.
   * @returns Minimality factor (0.1-1.0) to apply to reward.
   */
  private async evaluateMinimality(turn: TurnLevelReward): Promise<number> {
    try {
      // Check if this turn involves code changes
      const toolName = turn.toolChoice.toolId.toLowerCase();
      const codeTools = ["write", "edit", "search_replace", "apply_diff"];

      if (!codeTools.some((tool) => toolName.includes(tool))) {
        return 1.0; // No penalty for non-code tools
      }

      // Extract code diff from turn (if available)
      const params = turn.toolChoice.parameters as any;
      const oldCode = params?.old_string || params?.content_before || "";
      const newCode = params?.new_string || params?.content || "";

      if (!oldCode && !newCode) {
        return 1.0; // No diff to evaluate
      }

      // Evaluate minimality using MinimalDiffEvaluator
      const evaluation = await this.diffEvaluator.evaluate({
        before: oldCode,
        after: newCode,
        language: "typescript",
      });

      // Return minimality factor (0.1-1.0)
      return evaluation.minimalityFactor;
    } catch (error) {
      // Return neutral factor on error
      return 1.0;
    }
  }

  /**
   * Allocates thinking budget for a task based on complexity.
   *
   * @param taskId - Task identifier.
   * @param characteristics - Task characteristics for budget allocation.
   * @returns Allocated token budget.
   */
  allocateThinkingBudget(
    taskId: string,
    characteristics: {
      toolCount: number;
      contextSize: number;
      stepCount: number;
      multiTurn: boolean;
      hasExternalCalls: boolean;
    }
  ): number {
    const result = this.budgetManager.allocateBudget(characteristics);

    return result.allocation.allocatedTokens;
  }

  /**
   * Records thinking budget usage for a completed task.
   *
   * @param taskId - Task identifier (used as allocation ID).
   * @param tokensUsed - Number of tokens actually used.
   */
  recordBudgetUsage(taskId: string, tokensUsed: number): void {
    this.budgetManager.recordUsage(taskId, tokensUsed);
  }

  /**
   * Gets thinking budget metrics.
   *
   * @returns Current budget metrics.
   */
  getBudgetMetrics() {
    return this.budgetManager.getMetrics();
  }

  /**
   * Validates trajectories for training.
   *
   * @param trajectories - Trajectories to validate.
   * @returns Valid trajectories.
   */
  private validateTrajectories(
    trajectories: ConversationTrajectory[]
  ): ConversationTrajectory[] {
    return trajectories.filter((trajectory) => {
      // Must have minimum length
      if (trajectory.turns.length < this.config.minTrajectoryLength) {
        return false;
      }

      // Must not exceed maximum length
      if (trajectory.turns.length > this.config.maxTrajectoryLength) {
        return false;
      }

      // Must have valid outcome
      if (!trajectory.finalOutcome) {
        return false;
      }

      return true;
    });
  }

  /**
   * Converts conversation trajectories to RL trajectories with turn data.
   *
   * @param trajectories - Conversation trajectories.
   * @returns RL trajectories with computed advantages.
   */
  private async convertToRLTrajectories(
    trajectories: ConversationTrajectory[]
  ): Promise<RLTrajectory[]> {
    const rlTrajectories: RLTrajectory[] = [];

    for (const trajectory of trajectories) {
      // Compute turn-level rewards if not already computed
      const rewardedTrajectory =
        trajectory.turns[0].totalReward !== undefined
          ? trajectory
          : await this.computeTurnRewards(trajectory);

      // Convert to RL trajectory format
      const rlTrajectory: RLTrajectory = {
        conversationId: rewardedTrajectory.conversationId,
        turns: [], // Will be filled below
        finalOutcome: rewardedTrajectory.finalOutcome,
        totalReward: rewardedTrajectory.turns.reduce(
          (sum, turn) => sum + turn.totalReward,
          0
        ),
      };

      // Convert turns to TurnData format
      for (let i = 0; i < rewardedTrajectory.turns.length; i++) {
        const turn = rewardedTrajectory.turns[i];

        // Create mock state representation (in full implementation, this would be actual state)
        const state = {
          taskContext: {
            taskId: rewardedTrajectory.conversationId,
            taskType: "code-editing",
            complexity: "standard" as const,
            requirements: {},
          },
          turnHistory: [],
          availableTools: [],
          remainingBudget: 1000,
        };

        const turnData: TurnData = {
          turnNumber: turn.turnNumber,
          state,
          action: turn.toolChoice,
          reward: turn.totalReward,
          advantage: 0, // Will be computed later
          logProb: Math.log(0.5), // Mock log probability
        };

        rlTrajectory.turns.push(turnData);
      }

      rlTrajectories.push(rlTrajectory);
    }

    return rlTrajectories;
  }

  /**
   * Groups trajectories by similarity for GRPO.
   *
   * @param trajectories - RL trajectories to group.
   * @returns Groups of similar trajectories.
   */
  private groupTrajectoriesBySimilarity(
    trajectories: RLTrajectory[]
  ): RLTrajectory[][] {
    // Simple grouping by trajectory length (in practice, would use more sophisticated similarity)
    const groups: Map<number, RLTrajectory[]> = new Map();

    for (const trajectory of trajectories) {
      const length = trajectory.turns.length;
      if (!groups.has(length)) {
        groups.set(length, []);
      }
      groups.get(length)!.push(trajectory);
    }

    return Array.from(groups.values()).filter((group) => group.length >= 2);
  }

  /**
   * Computes advantages using GRPO across trajectory groups.
   *
   * @param trajectoryGroups - Groups of similar trajectories.
   * @returns Trajectories with computed advantages.
   */
  private async computeGroupAdvantages(
    trajectoryGroups: RLTrajectory[][]
  ): Promise<RLTrajectory[]> {
    const trajectoriesWithAdvantages: RLTrajectory[] = [];

    for (const group of trajectoryGroups) {
      // Compute group-relative advantages
      for (const trajectory of group) {
        const groupMeanReward =
          group.reduce((sum, t) => sum + t.totalReward, 0) / group.length;
        const groupStdReward = Math.sqrt(
          group.reduce(
            (sum, t) => sum + Math.pow(t.totalReward - groupMeanReward, 2),
            0
          ) / group.length
        );

        // Compute advantage for each turn
        const turnsWithAdvantages = trajectory.turns.map((turn) => ({
          ...turn,
          advantage: (turn.reward - groupMeanReward) / (groupStdReward + 1e-8), // Add epsilon to avoid division by zero
        }));

        trajectoriesWithAdvantages.push({
          ...trajectory,
          turns: turnsWithAdvantages,
        });
      }
    }

    return trajectoriesWithAdvantages;
  }

  /**
   * Updates the policy network using computed advantages.
   *
   * @param trajectories - Trajectories with computed advantages.
   * @returns Policy update result.
   */
  private async updatePolicy(trajectories: RLTrajectory[]): Promise<{
    policyLoss: number;
    valueLoss: number;
    klDivergence: number;
  }> {
    // Mock policy update (in practice, this would update actual model parameters)
    const totalTurns = trajectories.reduce((sum, t) => sum + t.turns.length, 0);
    const averageAdvantage =
      trajectories.reduce(
        (sum, t) =>
          sum + t.turns.reduce((turnSum, turn) => turnSum + turn.advantage, 0),
        0
      ) / totalTurns;

    // Simulate policy loss as negative log probability weighted by advantage
    const policyLoss = -averageAdvantage * 0.1;

    // Simulate value loss
    const valueLoss = Math.pow(averageAdvantage, 2) * 0.05;

    // Simulate KL divergence penalty
    const klDivergence = Math.abs(averageAdvantage) * this.config.klPenalty;

    return {
      policyLoss,
      valueLoss,
      klDivergence,
    };
  }

  /**
   * Updates training statistics.
   *
   * @param trajectories - Trajectories processed.
   * @param policyUpdate - Policy update result.
   * @param trainingTime - Time taken for training.
   */
  private updateTrainingStats(
    trajectories: RLTrajectory[],
    policyUpdate: {
      policyLoss: number;
      valueLoss: number;
      klDivergence: number;
    },
    trainingTime: number
  ): void {
    const totalReward = trajectories.reduce((sum, t) => sum + t.totalReward, 0);
    const averageReward = totalReward / trajectories.length;

    this.trainingStats = {
      trajectoriesProcessed:
        this.trainingStats.trajectoriesProcessed + trajectories.length,
      averageReward,
      policyLoss: policyUpdate.policyLoss,
      valueLoss: policyUpdate.valueLoss,
      klDivergence: policyUpdate.klDivergence,
      trainingTimeMs: trainingTime,
      timestamp: new Date().toISOString(),
    };
  }

  /**
   * Gets current training statistics.
   *
   * @returns Current training statistics.
   */
  getTrainingStats(): RLTrainingStats {
    return { ...this.trainingStats };
  }

  /**
   * Gets current configuration.
   *
   * @returns Current configuration.
   */
  getConfig(): RLTrainingConfig {
    return { ...this.config };
  }

  /**
   * Updates configuration.
   *
   * @param config - New configuration to apply.
   */
  updateConfig(config: Partial<RLTrainingConfig>): void {
    this.config = { ...this.config, ...config };
  }
}
