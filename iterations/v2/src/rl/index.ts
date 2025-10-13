/**
 * RL Training System - Unified Exports
 *
 * @author @darianrosebrook
 * @module rl
 *
 * Reinforcement learning training system for continuous agent improvement.
 * Complete RL pipeline from data collection to model deployment.
 */

// Data Collection & Performance Tracking
export { PerformanceTracker } from "./PerformanceTracker";
export type {
  PerformanceStats,
  PerformanceTrackerConfig,
  TaskExecutionData,
} from "./PerformanceTracker";

// Debate & Arbitration Outcome Tracking
export { DebateOutcomeTracker } from "./DebateOutcomeTracker";
export type {
  DebateOutcome,
  DebateOutcomeTrackerConfig,
  DebateTurn,
} from "./DebateOutcomeTracker";

// Verdict Quality Scoring
export { VerdictQualityScorer } from "./VerdictQualityScorer";
export type {
  VerdictQualityEvaluation,
  VerdictQualityScorerConfig,
} from "./VerdictQualityScorer";

// Turn-Level RL Training
export { TurnLevelRLTrainer } from "./TurnLevelRLTrainer";

// Multi-Armed Bandit for Task Routing
export { MultiArmedBandit } from "./MultiArmedBandit";

// Tool Adoption Training
export { ToolAdoptionTrainer } from "./ToolAdoptionTrainer";
export type {
  ToolAdoptionConfig,
  ToolAdoptionStats,
} from "./ToolAdoptionTrainer";

// RL Training Coordination
export { RLTrainingCoordinator } from "./RLTrainingCoordinator";
export type {
  RLTrainingCoordinatorConfig,
  TrainingPipelineStatus,
  TrainingStage,
} from "./RLTrainingCoordinator";

// Model Deployment Management
export { ModelDeploymentManager } from "./ModelDeploymentManager";
export type {
  ABTestConfig,
  ABTestResult,
  ModelDeploymentManagerConfig,
  ModelVersion,
} from "./ModelDeploymentManager";
