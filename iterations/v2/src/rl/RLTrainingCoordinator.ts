/**
 * RL Training Coordinator
 *
 * @author @darianrosebrook
 * @module rl-training-coordinator
 *
 * Orchestrates the complete reinforcement learning training pipeline,
 * coordinating data collection, quality validation, training, and deployment.
 */

import { EventEmitter } from "events";
import type { RLDataPipeline } from "../benchmarking/RLDataPipeline";
import type {
  ConversationTrajectory,
  RLTrainingStats,
} from "../types/agentic-rl";
import type {
  DebateOutcome,
  DebateOutcomeTracker,
} from "./DebateOutcomeTracker";
import type { PerformanceTracker } from "./PerformanceTracker";
import type { TurnLevelRLTrainer } from "./TurnLevelRLTrainer";
import type {
  VerdictQualityEvaluation,
  VerdictQualityScorer,
} from "./VerdictQualityScorer";

/**
 * Training pipeline stage.
 */
export type TrainingStage =
  | "data_collection"
  | "quality_validation"
  | "batch_preparation"
  | "training"
  | "evaluation"
  | "deployment";

/**
 * Training pipeline status.
 */
export interface TrainingPipelineStatus {
  /**
   * Current pipeline stage.
   */
  currentStage: TrainingStage;

  /**
   * Whether the pipeline is active.
   */
  isActive: boolean;

  /**
   * Total debate outcomes collected.
   */
  totalDebateOutcomes: number;

  /**
   * Total training samples generated.
   */
  totalTrainingSamples: number;

  /**
   * Total training batches completed.
   */
  totalTrainingBatches: number;

  /**
   * Training data quality score (0-1).
   */
  dataQualityScore: number;

  /**
   * Current model performance metrics.
   */
  modelPerformance: {
    averageReward: number;
    successRate: number;
    complianceScore: number;
  };

  /**
   * Last training time.
   */
  lastTrainingTime?: string;

  /**
   * Pipeline health status.
   */
  health: "healthy" | "degraded" | "failing";

  /**
   * Any errors encountered.
   */
  errors: string[];
}

/**
 * Configuration for the RL training coordinator.
 */
export interface RLTrainingCoordinatorConfig {
  /**
   * Minimum debate outcomes required before training.
   */
  minDebateOutcomes: number;

  /**
   * Minimum data quality score to proceed with training.
   */
  minDataQualityScore: number;

  /**
   * Maximum data age for training (milliseconds).
   */
  maxDataAgeMs: number;

  /**
   * Training batch size.
   */
  trainingBatchSize: number;

  /**
   * Training interval (milliseconds).
   */
  trainingIntervalMs: number;

  /**
   * Whether to enable automatic training.
   */
  enableAutoTraining: boolean;

  /**
   * Quality validation thresholds.
   */
  qualityThresholds: {
    minVerdictQuality: number;
    minTurnQuality: number;
    minEvidenceQuality: number;
    minComplianceScore: number;
  };

  /**
   * Performance degradation threshold for alerts.
   */
  performanceDegradationThreshold: number;
}

/**
 * Default configuration.
 */
const DEFAULT_CONFIG: RLTrainingCoordinatorConfig = {
  minDebateOutcomes: 100,
  minDataQualityScore: 0.7,
  maxDataAgeMs: 7 * 24 * 60 * 60 * 1000, // 7 days
  trainingBatchSize: 32,
  trainingIntervalMs: 4 * 60 * 60 * 1000, // 4 hours
  enableAutoTraining: true,
  qualityThresholds: {
    minVerdictQuality: 0.6,
    minTurnQuality: 0.5,
    minEvidenceQuality: 0.6,
    minComplianceScore: 0.7,
  },
  performanceDegradationThreshold: 0.1, // 10% drop triggers alert
};

/**
 * RL Training Coordinator for pipeline orchestration.
 *
 * This component coordinates the entire RL training pipeline:
 * 1. Data Collection (via DebateOutcomeTracker)
 * 2. Quality Validation (via VerdictQualityScorer)
 * 3. Batch Preparation (via RLDataPipeline)
 * 4. Training (via TurnLevelRLTrainer)
 * 5. Evaluation & Deployment
 */
export class RLTrainingCoordinator extends EventEmitter {
  private config: RLTrainingCoordinatorConfig;
  private status: TrainingPipelineStatus;
  private trainingTimer?: ReturnType<typeof setInterval>;

  // Component dependencies
  private debateTracker: DebateOutcomeTracker;
  private verdictScorer: VerdictQualityScorer;
  private rlTrainer: TurnLevelRLTrainer;
  private performanceTracker: PerformanceTracker;
  private dataPipeline: RLDataPipeline;

  /**
   * Creates a new RL training coordinator.
   *
   * @param config - Coordinator configuration. Uses defaults if not provided.
   * @param components - Required component dependencies.
   */
  constructor(
    config: Partial<RLTrainingCoordinatorConfig> = {},
    components: {
      debateTracker: DebateOutcomeTracker;
      verdictScorer: VerdictQualityScorer;
      rlTrainer: TurnLevelRLTrainer;
      performanceTracker: PerformanceTracker;
      dataPipeline: RLDataPipeline;
    }
  ) {
    super();
    this.config = { ...DEFAULT_CONFIG, ...config };

    this.debateTracker = components.debateTracker;
    this.verdictScorer = components.verdictScorer;
    this.rlTrainer = components.rlTrainer;
    this.performanceTracker = components.performanceTracker;
    this.dataPipeline = components.dataPipeline;

    this.status = {
      currentStage: "data_collection",
      isActive: false,
      totalDebateOutcomes: 0,
      totalTrainingSamples: 0,
      totalTrainingBatches: 0,
      dataQualityScore: 0,
      modelPerformance: {
        averageReward: 0,
        successRate: 0,
        complianceScore: 0,
      },
      health: "healthy",
      errors: [],
    };
  }

  /**
   * Starts the RL training pipeline.
   */
  startPipeline(): void {
    if (this.status.isActive) {
      throw new Error("Training pipeline is already active");
    }

    this.status.isActive = true;
    this.status.currentStage = "data_collection";

    // Start data collection
    this.debateTracker.startTracking();
    this.performanceTracker.startCollection();
    this.dataPipeline.startProcessing();

    // Start automatic training if enabled
    if (this.config.enableAutoTraining) {
      this.startAutoTraining();
    }

    this.emit("pipeline_started");
  }

  /**
   * Stops the RL training pipeline.
   */
  stopPipeline(): void {
    if (!this.status.isActive) {
      return;
    }

    this.status.isActive = false;

    // Stop data collection
    this.debateTracker.stopTracking();
    this.performanceTracker.stopCollection();
    this.dataPipeline.stopProcessing();

    // Stop automatic training
    if (this.trainingTimer) {
      clearInterval(this.trainingTimer);
      this.trainingTimer = undefined;
    }

    this.emit("pipeline_stopped");
  }

  /**
   * Manually triggers a training cycle.
   *
   * @returns Training results
   */
  async triggerTraining(): Promise<{
    success: boolean;
    stats: RLTrainingStats;
    qualityMetrics: {
      averageVerdictQuality: number;
      dataQualityScore: number;
      trainingQuality: number;
    };
    errors: string[];
  }> {
    const errors: string[] = [];

    try {
      // Stage 1: Data Collection & Validation
      this.status.currentStage = "quality_validation";
      const { validatedOutcomes, qualityMetrics } = await this.validateData();

      if (validatedOutcomes.length < this.config.minDebateOutcomes) {
        errors.push(
          `Insufficient data: ${validatedOutcomes.length} outcomes (min: ${this.config.minDebateOutcomes})`
        );
        this.updateHealth(errors);
        return {
          success: false,
          stats: this.rlTrainer.getTrainingStats(),
          qualityMetrics,
          errors,
        };
      }

      if (qualityMetrics.dataQualityScore < this.config.minDataQualityScore) {
        errors.push(
          `Low data quality: ${qualityMetrics.dataQualityScore.toFixed(
            2
          )} (min: ${this.config.minDataQualityScore})`
        );
        this.updateHealth(errors);
        return {
          success: false,
          stats: this.rlTrainer.getTrainingStats(),
          qualityMetrics,
          errors,
        };
      }

      // Stage 2: Batch Preparation
      this.status.currentStage = "batch_preparation";
      const trajectories = await this.prepareTrainingBatches(validatedOutcomes);

      if (trajectories.length === 0) {
        errors.push("Failed to prepare training batches");
        this.updateHealth(errors);
        return {
          success: false,
          stats: this.rlTrainer.getTrainingStats(),
          qualityMetrics,
          errors,
        };
      }

      // Stage 3: Training
      this.status.currentStage = "training";
      const trainingStats = await this.rlTrainer.trainOnTrajectories(
        trajectories
      );

      // Update status
      this.status.totalTrainingBatches++;
      this.status.lastTrainingTime = new Date().toISOString();
      this.status.modelPerformance.averageReward = trainingStats.averageReward;
      this.status.dataQualityScore = qualityMetrics.dataQualityScore;

      // Stage 4: Evaluation
      this.status.currentStage = "evaluation";
      await this.evaluateTrainingResults(trainingStats);

      // Update health
      this.updateHealth(errors);

      this.emit("training_completed", {
        stats: trainingStats,
        qualityMetrics,
      });

      return {
        success: true,
        stats: trainingStats,
        qualityMetrics,
        errors,
      };
    } catch (error) {
      errors.push(`Training error: ${error}`);
      this.updateHealth(errors);

      this.emit("training_error", error);

      return {
        success: false,
        stats: this.rlTrainer.getTrainingStats(),
        qualityMetrics: {
          averageVerdictQuality: 0,
          dataQualityScore: 0,
          trainingQuality: 0,
        },
        errors,
      };
    }
  }

  /**
   * Gets current pipeline status.
   *
   * @returns Current status
   */
  getStatus(): TrainingPipelineStatus {
    return { ...this.status };
  }

  /**
   * Gets pipeline statistics.
   *
   * @returns Pipeline statistics
   */
  getStats(): {
    debateStats: ReturnType<typeof this.debateTracker.getStats>;
    performanceStats: ReturnType<typeof this.performanceTracker.getStats>;
    trainingStats: RLTrainingStats;
    pipelineStatus: TrainingPipelineStatus;
  } {
    return {
      debateStats: this.debateTracker.getStats(),
      performanceStats: this.performanceTracker.getStats(),
      trainingStats: this.rlTrainer.getTrainingStats(),
      pipelineStatus: this.getStatus(),
    };
  }

  /**
   * Gets current configuration.
   *
   * @returns Current configuration
   */
  getConfig(): RLTrainingCoordinatorConfig {
    return { ...this.config };
  }

  /**
   * Updates configuration.
   *
   * @param config - New configuration to apply
   */
  updateConfig(config: Partial<RLTrainingCoordinatorConfig>): void {
    this.config = { ...this.config, ...config };

    // Restart auto-training if interval changed
    if (config.trainingIntervalMs && this.trainingTimer) {
      this.stopAutoTraining();
      this.startAutoTraining();
    }
  }

  /**
   * Validates collected data for training readiness.
   */
  private async validateData(): Promise<{
    validatedOutcomes: DebateOutcome[];
    qualityMetrics: {
      averageVerdictQuality: number;
      dataQualityScore: number;
      trainingQuality: number;
    };
  }> {
    // Export outcomes from debate tracker
    const allOutcomes = this.debateTracker.exportOutcomes();

    // Filter by age
    const cutoffTime = Date.now() - this.config.maxDataAgeMs;
    const recentOutcomes = allOutcomes.filter(
      (outcome) => new Date(outcome.timestamp).getTime() >= cutoffTime
    );

    this.status.totalDebateOutcomes = recentOutcomes.length;

    // Validate verdict quality for each outcome
    const qualityEvaluations: VerdictQualityEvaluation[] = [];
    const validatedOutcomes: DebateOutcome[] = [];

    for (const outcome of recentOutcomes) {
      if (!outcome.verdict) {
        continue; // Skip outcomes without verdicts
      }

      // Score verdict quality
      const qualityEval = await this.verdictScorer.evaluateVerdict(
        outcome.verdict,
        {
          violation: outcome.sessionId,
          arguments: outcome.turns.map((t) => ({
            content: t.action.content,
            agentId: t.agentId,
          })),
        }
      );

      qualityEvaluations.push(qualityEval);

      // Apply quality thresholds
      if (
        qualityEval.overallScore >=
          this.config.qualityThresholds.minVerdictQuality &&
        outcome.metrics.evidenceQuality >=
          this.config.qualityThresholds.minEvidenceQuality &&
        outcome.metrics.complianceScore >=
          this.config.qualityThresholds.minComplianceScore
      ) {
        validatedOutcomes.push(outcome);
      }
    }

    // Calculate quality metrics
    const averageVerdictQuality =
      qualityEvaluations.length > 0
        ? qualityEvaluations.reduce((sum, e) => sum + e.overallScore, 0) /
          qualityEvaluations.length
        : 0;

    const averageTurnQuality =
      validatedOutcomes.length > 0
        ? validatedOutcomes.reduce(
            (sum, o) =>
              sum +
              o.turns.reduce((tSum, t) => tSum + t.reward, 0) / o.turns.length,
            0
          ) / validatedOutcomes.length
        : 0;

    const dataQualityScore = (averageVerdictQuality + averageTurnQuality) / 2;

    this.emit("data_validated", {
      totalOutcomes: recentOutcomes.length,
      validatedOutcomes: validatedOutcomes.length,
      averageVerdictQuality,
      dataQualityScore,
    });

    return {
      validatedOutcomes,
      qualityMetrics: {
        averageVerdictQuality,
        dataQualityScore,
        trainingQuality: dataQualityScore,
      },
    };
  }

  /**
   * Prepares training batches from validated outcomes.
   */
  private async prepareTrainingBatches(
    outcomes: DebateOutcome[]
  ): Promise<ConversationTrajectory[]> {
    const trajectories: ConversationTrajectory[] = [];

    for (const outcome of outcomes) {
      // Convert debate outcome to conversation trajectory
      const trajectory: ConversationTrajectory = {
        conversationId: outcome.sessionId,
        turns: outcome.turns.map((turn) => ({
          turnNumber: turn.turnNumber,
          toolChoice: {
            toolId: turn.action.type,
            parameters: { content: turn.action.content },
          },
          totalReward: turn.reward,
          informationGain: turn.metrics.argumentStrength,
          formatCorrectness: 1.0, // Assume valid format if parsed
          taskProgress: turn.metrics.persuasiveness,
          safetyScore: turn.metrics.constitutionalAlignment,
        })),
        finalOutcome: {
          success: outcome.verdict?.outcome === "approved",
          qualityScore: outcome.qualityScore,
          latencyMs: outcome.metrics.resolutionTimeMs,
          tokensUsed: 0, // Would be tracked separately
        },
      };

      trajectories.push(trajectory);
    }

    this.status.totalTrainingSamples += trajectories.reduce(
      (sum, t) => sum + t.turns.length,
      0
    );

    return trajectories;
  }

  /**
   * Evaluates training results and updates model performance.
   */
  private async evaluateTrainingResults(stats: RLTrainingStats): Promise<void> {
    // Update model performance metrics
    this.status.modelPerformance.averageReward = stats.averageReward;

    // Calculate success rate from recent performance data
    const perfStats = this.performanceTracker.getStats();
    this.status.modelPerformance.successRate = perfStats.overallSuccessRate;

    // Check for performance degradation
    const previousReward = this.status.modelPerformance.averageReward;
    if (previousReward > 0) {
      const degradation =
        (previousReward - stats.averageReward) / previousReward;
      if (degradation > this.config.performanceDegradationThreshold) {
        this.emit("performance_degradation", {
          previousReward,
          currentReward: stats.averageReward,
          degradation,
        });
      }
    }

    // Record training metrics for monitoring
    await this.performanceTracker.recordRLTrainingMetrics({
      trajectoriesProcessed: stats.trajectoriesProcessed,
      averageReward: stats.averageReward,
      policyLoss: stats.policyLoss,
      valueLoss: stats.valueLoss,
      klDivergence: stats.klDivergence,
      trainingTimeMs: stats.trainingTimeMs,
    });
  }

  /**
   * Starts automatic training on an interval.
   */
  private startAutoTraining(): void {
    this.trainingTimer = setInterval(async () => {
      if (this.status.isActive) {
        await this.triggerTraining();
      }
    }, this.config.trainingIntervalMs);
  }

  /**
   * Stops automatic training.
   */
  private stopAutoTraining(): void {
    if (this.trainingTimer) {
      clearInterval(this.trainingTimer);
      this.trainingTimer = undefined;
    }
  }

  /**
   * Updates pipeline health status based on errors.
   */
  private updateHealth(errors: string[]): void {
    this.status.errors = errors;

    if (errors.length === 0) {
      this.status.health = "healthy";
    } else if (errors.length < 3) {
      this.status.health = "degraded";
    } else {
      this.status.health = "failing";
    }

    if (this.status.health !== "healthy") {
      this.emit("health_degraded", {
        health: this.status.health,
        errors: this.status.errors,
      });
    }
  }
}
