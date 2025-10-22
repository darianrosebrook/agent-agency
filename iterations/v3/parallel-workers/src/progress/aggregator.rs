//! Progress aggregation across all workers

use crate::types::*;
use crate::error::*;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

/// Aggregates progress across multiple workers for overall task progress
pub struct ProgressAggregator {
    overall_progress: Arc<RwLock<Progress>>,
    worker_contributions: Arc<RwLock<HashMap<WorkerId, WorkerContribution>>>,
    task_id: TaskId,
}

impl ProgressAggregator {
    pub fn new(task_id: TaskId) -> Self {
        Self {
            overall_progress: Arc::new(RwLock::new(Progress {
                task_id: task_id.clone(),
                percentage: 0.0,
                completed_subtasks: 0,
                total_subtasks: 0,
                active_workers: 0,
                blocked_workers: 0,
                failed_workers: 0,
                estimated_completion: None,
                last_update: chrono::Utc::now(),
            })),
            worker_contributions: Arc::new(RwLock::new(HashMap::new())),
            task_id,
        }
    }

    /// Update overall progress from worker progress updates
    pub fn update_from_worker_progress(&self, worker_progress: &WorkerProgress) -> ProgressResult<()> {
        let mut contributions = self.worker_contributions.write();
        let contribution = contributions.entry(worker_progress.worker_id.clone())
            .or_insert_with(|| WorkerContribution {
                worker_id: worker_progress.worker_id.clone(),
                subtask_id: worker_progress.subtask_id.clone(),
                weight: worker_progress.task_weight,
                current_progress: 0.0,
                status: WorkerStatus::Pending,
            });

        // Update contribution based on worker progress
        contribution.current_progress = if worker_progress.total > 0 {
            worker_progress.completed as f32 / worker_progress.total as f32
        } else {
            0.0
        };

        contribution.status = match worker_progress.status.as_str() {
            s if s.starts_with("completed") => WorkerStatus::Completed,
            s if s.starts_with("failed") => WorkerStatus::Failed,
            s if s.starts_with("blocked") => WorkerStatus::Blocked,
            s if s.starts_with("running") => WorkerStatus::Running,
            _ => WorkerStatus::Pending,
        };

        // Recalculate overall progress
        self.recalculate_overall_progress()?;

        Ok(())
    }

    /// Register a new worker contribution
    pub fn register_worker(&self, worker_id: WorkerId, subtask_id: SubTaskId, weight: f32) -> ProgressResult<()> {
        let mut contributions = self.worker_contributions.write();
        contributions.insert(worker_id.clone(), WorkerContribution {
            worker_id,
            subtask_id,
            weight,
            current_progress: 0.0,
            status: WorkerStatus::Pending,
        });

        self.recalculate_overall_progress()?;
        Ok(())
    }

    /// Remove a worker contribution (when worker completes/fails)
    pub fn remove_worker(&self, worker_id: &WorkerId) -> ProgressResult<()> {
        let mut contributions = self.worker_contributions.write();
        contributions.remove(worker_id);

        self.recalculate_overall_progress()?;
        Ok(())
    }

    /// Recalculate overall progress from all worker contributions
    fn recalculate_overall_progress(&self) -> ProgressResult<()> {
        let contributions = self.worker_contributions.read();
        let mut overall_progress = self.overall_progress.write();

        if contributions.is_empty() {
            overall_progress.percentage = 0.0;
            overall_progress.completed_subtasks = 0;
            overall_progress.total_subtasks = 0;
            overall_progress.active_workers = 0;
            overall_progress.blocked_workers = 0;
            overall_progress.failed_workers = 0;
            overall_progress.last_update = chrono::Utc::now();
            return Ok(());
        }

        let total_weight: f32 = contributions.values().map(|c| c.weight).sum();
        let mut completed_weight = 0.0;
        let mut active_workers = 0;
        let mut blocked_workers = 0;
        let mut failed_workers = 0;
        let mut completed_subtasks = 0;

        for contribution in contributions.values() {
            // Add weighted progress contribution
            completed_weight += contribution.current_progress * contribution.weight;

            // Count by status
            match contribution.status {
                WorkerStatus::Running => active_workers += 1,
                WorkerStatus::Blocked => blocked_workers += 1,
                WorkerStatus::Failed => failed_workers += 1,
                WorkerStatus::Completed => {
                    completed_subtasks += 1;
                    active_workers += 1; // Completed workers are still "active" until cleaned up
                }
                WorkerStatus::Pending => {} // Don't count pending workers
            }
        }

        overall_progress.percentage = if total_weight > 0.0 {
            (completed_weight / total_weight * 100.0).min(100.0)
        } else {
            0.0
        };

        overall_progress.completed_subtasks = completed_subtasks;
        overall_progress.total_subtasks = contributions.len();
        overall_progress.active_workers = active_workers;
        overall_progress.blocked_workers = blocked_workers;
        overall_progress.failed_workers = failed_workers;
        overall_progress.last_update = chrono::Utc::now();

        Ok(())
    }

    /// Get current overall progress
    pub fn get_overall_progress(&self) -> Progress {
        self.overall_progress.read().clone()
    }

    /// Get detailed worker contributions
    pub fn get_worker_contributions(&self) -> Vec<WorkerContribution> {
        self.worker_contributions.read().values().cloned().collect()
    }

    /// Get contribution for a specific worker
    pub fn get_worker_contribution(&self, worker_id: &WorkerId) -> Option<WorkerContribution> {
        self.worker_contributions.read().get(worker_id).cloned()
    }

    /// Check if task is completed (all workers done)
    pub fn is_task_completed(&self) -> bool {
        let contributions = self.worker_contributions.read();
        contributions.values().all(|c| matches!(c.status, WorkerStatus::Completed | WorkerStatus::Failed))
    }

    /// Get progress statistics
    pub fn get_stats(&self) -> ProgressStats {
        let contributions = self.worker_contributions.read();
        let progress = self.overall_progress.read();

        let mut pending_workers = 0;
        let mut running_workers = 0;
        let mut completed_workers = 0;
        let mut failed_workers = 0;
        let mut blocked_workers = 0;

        for contribution in contributions.values() {
            match contribution.status {
                WorkerStatus::Pending => pending_workers += 1,
                WorkerStatus::Running => running_workers += 1,
                WorkerStatus::Completed => completed_workers += 1,
                WorkerStatus::Failed => failed_workers += 1,
                WorkerStatus::Blocked => blocked_workers += 1,
            }
        }

        ProgressStats {
            overall_progress: progress.percentage,
            total_workers: contributions.len(),
            pending_workers,
            running_workers,
            completed_workers,
            failed_workers,
            blocked_workers,
            average_completion_rate: self.calculate_average_completion_rate(),
        }
    }

    /// Calculate average completion rate across workers
    fn calculate_average_completion_rate(&self) -> f32 {
        let contributions = self.worker_contributions.read();

        if contributions.is_empty() {
            return 0.0;
        }

        let total_completion: f32 = contributions.values()
            .map(|c| c.current_progress)
            .sum();

        total_completion / contributions.len() as f32
    }

    /// Get estimated completion time based on current progress
    pub fn estimate_completion_time(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        let contributions = self.worker_contributions.read();

        if contributions.is_empty() {
            return None;
        }

        // Find workers that are actively making progress
        let active_workers: Vec<_> = contributions.values()
            .filter(|c| matches!(c.status, WorkerStatus::Running) && c.current_progress > 0.0)
            .collect();

        if active_workers.is_empty() {
            return None;
        }

        // Calculate average progress rate (progress per unit time)
        // This is a simplified estimation - in practice you'd want historical data
        let avg_progress_rate = 0.01; // Assume 1% progress per minute as baseline
        let remaining_progress = 1.0 - self.get_overall_progress().percentage / 100.0;

        if remaining_progress <= 0.0 {
            return Some(chrono::Utc::now());
        }

        let minutes_remaining = remaining_progress / avg_progress_rate;
        let duration = chrono::Duration::minutes(minutes_remaining as i64);

        Some(chrono::Utc::now() + duration)
    }
}

/// Contribution of a single worker to overall progress
#[derive(Debug, Clone)]
pub struct WorkerContribution {
    pub worker_id: WorkerId,
    pub subtask_id: SubTaskId,
    pub weight: f32,
    pub current_progress: f32,
    pub status: WorkerStatus,
}

/// Status of a worker
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkerStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Blocked,
}

/// Statistics for progress tracking
#[derive(Debug, Clone)]
pub struct ProgressStats {
    pub overall_progress: f32,
    pub total_workers: usize,
    pub pending_workers: usize,
    pub running_workers: usize,
    pub completed_workers: usize,
    pub failed_workers: usize,
    pub blocked_workers: usize,
    pub average_completion_rate: f32,
}

/// Progress milestone tracking
pub struct ProgressMilestones {
    milestones: Vec<ProgressMilestone>,
    current_milestone: Option<usize>,
}

impl ProgressMilestones {
    pub fn new(milestones: Vec<ProgressMilestone>) -> Self {
        Self {
            milestones,
            current_milestone: None,
        }
    }

    /// Check if any milestones have been reached
    pub fn check_milestones(&mut self, current_progress: f32) -> Vec<ProgressMilestone> {
        let mut reached = Vec::new();

        for (index, milestone) in self.milestones.iter().enumerate() {
            if current_progress >= milestone.threshold && !milestone.reached {
                reached.push(milestone.clone());
                self.current_milestone = Some(index);
            }
        }

        reached
    }

    /// Get next milestone
    pub fn next_milestone(&self) -> Option<&ProgressMilestone> {
        if let Some(current) = self.current_milestone {
            self.milestones.get(current + 1)
        } else {
            self.milestones.first()
        }
    }

    /// Get all milestones
    pub fn all_milestones(&self) -> &[ProgressMilestone] {
        &self.milestones
    }
}

/// A progress milestone
#[derive(Debug, Clone)]
pub struct ProgressMilestone {
    pub name: String,
    pub description: String,
    pub threshold: f32,
    pub reached: bool,
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

impl ProgressMilestone {
    pub fn new(name: String, description: String, threshold: f32) -> Self {
        Self {
            name,
            description,
            threshold,
            reached: false,
            timestamp: None,
        }
    }

    /// Mark milestone as reached
    pub fn mark_reached(&mut self) {
        self.reached = true;
        self.timestamp = Some(chrono::Utc::now());
    }
}
