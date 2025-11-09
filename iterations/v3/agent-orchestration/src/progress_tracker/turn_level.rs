//! Turn-Level Progress Tracking
//!
//! Extends the base ProgressTracker with turn-level tracking capabilities
//! for long-horizon tasks, including credit assignment and trajectory analysis.
//!
//! @author @darianrosebrook

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use anyhow::Result;

use crate::progress_tracker::{ProgressTracker, ExecutionProgress};
use agent_agency_contracts::execution_artifacts::ExecutionArtifacts;

use crate::progress_tracker::trajectory_analyzer::TrajectoryAnalyzer;
use crate::progress_tracker::credit_assignment::AdvancedCreditAssigner;

/// Represents an agent action taken during a turn
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAction {
    /// Action type (e.g., "code_generation", "test_execution", "refinement")
    pub action_type: String,
    /// Action description
    pub description: String,
    /// Worker ID that performed the action
    pub worker_id: Option<Uuid>,
    /// Milestone ID if applicable
    pub milestone_id: Option<String>,
    /// Timestamp when action was taken
    pub timestamp: DateTime<Utc>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Represents the outcome of a turn
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnOutcome {
    /// Whether the turn was successful
    pub success: bool,
    /// Quality score for this turn (0.0-1.0)
    pub quality_score: f64,
    /// Execution artifacts from this turn
    pub artifacts: Option<ExecutionArtifacts>,
    /// Error message if turn failed
    pub error: Option<String>,
    /// Performance metrics
    pub execution_time_ms: Option<u64>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Represents a single turn in a multi-turn task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnProgress {
    /// Turn number (1-indexed)
    pub turn_number: u32,
    /// Task ID
    pub task_id: Uuid,
    /// Action taken in this turn
    pub action: AgentAction,
    /// Outcome of this turn
    pub outcome: TurnOutcome,
    /// Reward assigned to this turn (for RL training)
    pub reward: Option<f64>,
    /// Credit assignment for this turn
    pub credit_assignment: Option<CreditAssignment>,
    /// Timestamp when turn started
    pub started_at: DateTime<Utc>,
    /// Timestamp when turn completed
    pub completed_at: DateTime<Utc>,
}

/// Credit assignment for a turn in a multi-turn task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditAssignment {
    /// Turn number this credit is assigned to
    pub turn_number: u32,
    /// Credit value (0.0-1.0) indicating contribution to final outcome
    pub credit_value: f64,
    /// Reasoning for credit assignment
    pub reasoning: String,
    /// Factors that influenced credit assignment
    pub factors: Vec<String>,
}

/// Represents a trajectory of turns for a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnTrajectory {
    /// Task ID
    pub task_id: Uuid,
    /// All turns in this trajectory
    pub turns: Vec<TurnProgress>,
    /// Final outcome of the task
    pub final_outcome: TaskOutcome,
    /// Total number of turns
    pub total_turns: u32,
    /// Trajectory quality score
    pub trajectory_quality: f64,
}

/// Final outcome of a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskOutcome {
    /// Whether task completed successfully
    pub success: bool,
    /// Final quality score
    pub quality_score: f64,
    /// Final execution artifacts
    pub artifacts: Vec<ExecutionArtifacts>,
    /// Completion timestamp
    pub completed_at: DateTime<Utc>,
}

/// Progress update from turn tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdate {
    /// Task ID
    pub task_id: Uuid,
    /// Current turn number
    pub current_turn: u32,
    /// Total turns (if known)
    pub total_turns: Option<u32>,
    /// Overall progress percentage
    pub progress_percentage: f64,
    /// Cumulative quality score
    pub cumulative_quality: f64,
    /// Latest turn progress
    pub latest_turn: Option<TurnProgress>,
}

/// Trait for turn-level progress tracking
#[async_trait::async_trait]
pub trait TurnLevelTracker: Send + Sync {
    /// Track progress for a specific turn
    async fn track_turn_progress(
        &self,
        task_id: Uuid,
        turn_number: u32,
        action: AgentAction,
        outcome: TurnOutcome,
    ) -> Result<ProgressUpdate>;

    /// Assign credit to turns based on final outcome
    async fn assign_credit(
        &self,
        task_id: Uuid,
        trajectory: Vec<TurnProgress>,
        final_outcome: TaskOutcome,
    ) -> Result<Vec<CreditAssignment>>;

    /// Get all turns for a task
    async fn get_turns(&self, task_id: Uuid) -> Result<Vec<TurnProgress>>;

    /// Get trajectory analysis for a task
    async fn analyze_trajectory(&self, task_id: Uuid) -> Result<TurnTrajectory>;

    /// Detect if task has reached a plateau (no improvement)
    async fn detect_plateau(&self, task_id: Uuid, window_size: usize) -> Result<bool>;
}

/// Turn-level progress tracker implementation
pub struct TurnLevelProgressTracker {
    /// Base progress tracker
    base_tracker: Arc<dyn ProgressTracker>,
    /// Storage for turn progress (task_id -> turns)
    turn_storage: Arc<RwLock<HashMap<Uuid, Vec<TurnProgress>>>>,
    /// Storage for trajectories
    trajectory_storage: Arc<RwLock<HashMap<Uuid, TurnTrajectory>>>,
    /// Trajectory analyzer for advanced pattern detection
    trajectory_analyzer: Arc<TrajectoryAnalyzer>,
    /// Advanced credit assigner with TD learning
    credit_assigner: Arc<AdvancedCreditAssigner>,
}

impl TurnLevelProgressTracker {
    pub fn new(base_tracker: Arc<dyn ProgressTracker>) -> Self {
        Self {
            base_tracker,
            turn_storage: Arc::new(RwLock::new(HashMap::new())),
            trajectory_storage: Arc::new(RwLock::new(HashMap::new())),
            trajectory_analyzer: Arc::new(TrajectoryAnalyzer::new()),
            credit_assigner: Arc::new(AdvancedCreditAssigner::default()),
        }
    }

    /// Calculate credit assignment using temporal difference (TD) learning approach
    fn calculate_credit_assignment(
        &self,
        trajectory: &[TurnProgress],
        final_outcome: &TaskOutcome,
    ) -> Vec<CreditAssignment> {
        // Use advanced TD(λ) credit assignment
        self.credit_assigner.assign_credit_hybrid(trajectory, final_outcome)
    }

    /// Detect plateau: check if quality hasn't improved in recent turns
    fn detect_plateau_internal(&self, turns: &[TurnProgress], window_size: usize) -> bool {
        if turns.len() < window_size {
            return false;
        }

        let recent_turns = &turns[turns.len().saturating_sub(window_size)..];
        let qualities: Vec<f64> = recent_turns.iter()
            .map(|t| t.outcome.quality_score)
            .collect();

        if qualities.is_empty() {
            return false;
        }

        // Check if all qualities are within a small threshold (plateau)
        let min_quality = qualities.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_quality = qualities.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let threshold = 0.05; // 5% threshold

        (max_quality - min_quality) < threshold
    }
}

#[async_trait::async_trait]
impl TurnLevelTracker for TurnLevelProgressTracker {
    async fn track_turn_progress(
        &self,
        task_id: Uuid,
        turn_number: u32,
        action: AgentAction,
        outcome: TurnOutcome,
    ) -> Result<ProgressUpdate> {
        let turn_progress = TurnProgress {
            turn_number,
            task_id,
            action: action.clone(),
            outcome: outcome.clone(),
            reward: None, // Will be assigned later via credit assignment
            credit_assignment: None, // Will be assigned when task completes
            started_at: action.timestamp,
            completed_at: Utc::now(),
        };

        // Store turn progress
        {
            let mut storage = self.turn_storage.write().await;
            let turns = storage.entry(task_id).or_insert_with(Vec::new);
            turns.push(turn_progress.clone());
        }

        // Update base progress tracker
        let progress_percentage = if let Some(total_turns) = self.get_total_turns(task_id).await {
            (turn_number as f64 / total_turns as f64) * 100.0
        } else {
            // Unknown total turns, estimate based on current progress
            (turn_number as f64 / (turn_number + 1) as f64) * 100.0
        };

        let mut base_progress = ExecutionProgress::default();
        base_progress.task_id = task_id;
        base_progress.percentage = progress_percentage;
        base_progress.current_phase = format!("Turn {}", turn_number);
        base_progress.current_phase_index = turn_number as usize;

        // Calculate cumulative quality
        let cumulative_quality = {
            let storage = self.turn_storage.read().await;
            if let Some(turns) = storage.get(&task_id) {
                turns.iter()
                    .map(|t| t.outcome.quality_score)
                    .sum::<f64>() / turns.len() as f64
            } else {
                outcome.quality_score
            }
        };

        base_progress.metrics.processing_rate = cumulative_quality;

        if let Err(e) = self.base_tracker.update_progress(task_id, base_progress).await {
            return Err(anyhow::anyhow!("Failed to update base progress: {}", e));
        }

        Ok(ProgressUpdate {
            task_id,
            current_turn: turn_number,
            total_turns: self.get_total_turns(task_id).await,
            progress_percentage,
            cumulative_quality,
            latest_turn: Some(turn_progress),
        })
    }

    async fn assign_credit(
        &self,
        task_id: Uuid,
        trajectory: Vec<TurnProgress>,
        final_outcome: TaskOutcome,
    ) -> Result<Vec<CreditAssignment>> {
        let assignments = self.calculate_credit_assignment(&trajectory, &final_outcome);

        // Update turn progress with credit assignments
        {
            let mut storage = self.turn_storage.write().await;
            if let Some(turns) = storage.get_mut(&task_id) {
                for assignment in &assignments {
                    if let Some(turn) = turns.iter_mut()
                        .find(|t| t.turn_number == assignment.turn_number) {
                        turn.credit_assignment = Some(assignment.clone());
                    }
                }
            }
        }

        // Store trajectory
        let trajectory_obj = TurnTrajectory {
            task_id,
            turns: trajectory.clone(),
            final_outcome: final_outcome.clone(),
            total_turns: trajectory.len() as u32,
            trajectory_quality: final_outcome.quality_score,
        };

        {
            let mut storage = self.trajectory_storage.write().await;
            storage.insert(task_id, trajectory_obj.clone());
        }

        // Analyze trajectory for patterns and insights
        if let Ok(insights) = self.trajectory_analyzer.analyze_trajectory(&trajectory_obj).await {
            tracing::info!(
                "Trajectory analysis for task {}: {} patterns detected, {} recommendations",
                task_id,
                insights.patterns.len(),
                insights.recommendations.len()
            );
            // Log key insights
            for pattern in &insights.patterns {
                tracing::debug!(
                    "Pattern detected: {:?} (confidence: {:.2}) - {}",
                    pattern.pattern_type,
                    pattern.confidence,
                    pattern.description
                );
            }
        }

        Ok(assignments)
    }

    async fn get_turns(&self, task_id: Uuid) -> Result<Vec<TurnProgress>> {
        let storage = self.turn_storage.read().await;
        Ok(storage.get(&task_id).cloned().unwrap_or_default())
    }

    async fn analyze_trajectory(&self, task_id: Uuid) -> Result<TurnTrajectory> {
        let storage = self.trajectory_storage.read().await;
        storage.get(&task_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No trajectory found for task {}", task_id))
    }

    async fn detect_plateau(&self, task_id: Uuid, window_size: usize) -> Result<bool> {
        let storage = self.turn_storage.read().await;
        if let Some(turns) = storage.get(&task_id) {
            Ok(self.detect_plateau_internal(turns, window_size))
        } else {
            Ok(false)
        }
    }
}

impl TurnLevelProgressTracker {
    /// Get total turns for a task (if known)
    async fn get_total_turns(&self, task_id: Uuid) -> Option<u32> {
        let storage = self.trajectory_storage.read().await;
        storage.get(&task_id).map(|t| t.total_turns)
    }
}

