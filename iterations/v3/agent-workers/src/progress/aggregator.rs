//! Progress aggregation across all workers

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::parallel_types::{SubTaskId, WorkerId};
use crate::{Progress, WorkerProgress, WorkerProgressStatus};
use crate::error::*;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::sync::Arc;

/// Aggregates progress across multiple workers for overall task progress
pub struct ProgressAggregator {
    overall_progress: Arc<RwLock<Progress>>,
    worker_contributions: Arc<RwLock<HashMap<WorkerId, WorkerContribution>>>,
}

impl ProgressAggregator {
    pub fn new() -> Self {
        Self {
            overall_progress: Arc::new(RwLock::new(Progress::default())),
            worker_contributions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Update overall progress from worker progress updates
    pub fn update_from_worker_progress(&self, worker_progress: &WorkerProgress) -> ProgressResult<()> {
        let mut contributions = self.worker_contributions.write();
        let worker_id = WorkerId(worker_progress.worker_id);
        let subtask_id = SubTaskId(worker_progress.subtask_id);
        let contribution = contributions.entry(worker_id)
            .or_insert_with(|| WorkerContribution {
                worker_id,
                subtask_id: subtask_id.clone(),
                weight: worker_progress.task_weight,
                current_progress: 0.0,
                status: WorkerProgressStatus::Pending,
            });

        // Update contribution based on worker progress
        contribution.current_progress = if worker_progress.total > 0 {
            (worker_progress.completed as f32 / worker_progress.total as f32).clamp(0.0, 1.0)
        } else {
            (worker_progress.progress_percentage / 100.0).clamp(0.0, 1.0)
        };

        contribution.status = worker_progress.status.clone();

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
            status: WorkerProgressStatus::Pending,
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
            *overall_progress = Progress {
                total_tasks: 0,
                completed_tasks: 0,
                failed_tasks: 0,
                in_progress_tasks: 0,
                overall_percentage: 0.0,
                estimated_completion: None,
                last_updated: chrono::Utc::now(),
            };
            return Ok(());
        }

        let total_weight: f32 = contributions.values().map(|c| c.weight.max(0.0)).sum();
        let mut completed_weight = 0.0;
        let mut completed = 0usize;
        let mut failed = 0usize;
        let mut running = 0usize;
        let mut blocked = 0usize;
        let mut pending = 0usize;

        for contribution in contributions.values() {
            completed_weight += contribution.current_progress.clamp(0.0, 1.0) * contribution.weight.max(0.0);

            match contribution.status {
                WorkerProgressStatus::Pending => pending += 1,
                WorkerProgressStatus::Running => running += 1,
                WorkerProgressStatus::Completed => completed += 1,
                WorkerProgressStatus::Failed => failed += 1,
                WorkerProgressStatus::Blocked => blocked += 1,
            }
        }

        let overall_fraction = if total_weight > 0.0 {
            (completed_weight / total_weight).clamp(0.0, 1.0)
        } else {
            0.0
        };

        overall_progress.total_tasks = to_u32(contributions.len());
        overall_progress.completed_tasks = to_u32(completed);
        overall_progress.failed_tasks = to_u32(failed);
        overall_progress.in_progress_tasks = to_u32(running + blocked + pending);
        overall_progress.overall_percentage = overall_fraction * 100.0;
        overall_progress.estimated_completion = compute_estimated_completion(overall_fraction, &contributions);
        overall_progress.last_updated = chrono::Utc::now();

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
        contributions.values().all(|c| matches!(c.status, WorkerProgressStatus::Completed | WorkerProgressStatus::Failed))
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
                WorkerProgressStatus::Pending => pending_workers += 1,
                WorkerProgressStatus::Running => running_workers += 1,
                WorkerProgressStatus::Completed => completed_workers += 1,
                WorkerProgressStatus::Failed => failed_workers += 1,
                WorkerProgressStatus::Blocked => blocked_workers += 1,
            }
        }

        ProgressStats {
            overall_progress: progress.overall_percentage,
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
        let overall_percentage = {
            let progress = self.overall_progress.read();
            progress.overall_percentage / 100.0
        };

        let contributions = self.worker_contributions.read();
        compute_estimated_completion(overall_percentage, &contributions)
    }
}

/// Contribution of a single worker to overall progress

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct WorkerContribution {
    pub worker_id: WorkerId,
    pub subtask_id: SubTaskId,
    pub weight: f32,
    pub current_progress: f32,
    pub status: WorkerProgressStatus,
}

/// Status of a worker
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct ProgressStats {
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct ProgressMilestones {
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

        for (index, milestone) in self.milestones.iter_mut().enumerate() {
            if current_progress >= milestone.threshold && !milestone.reached {
                milestone.mark_reached();
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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

fn to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

fn compute_estimated_completion(
    completion_fraction: f32,
    contributions: &HashMap<WorkerId, WorkerContribution>,
) -> Option<chrono::DateTime<chrono::Utc>> {
    if contributions.is_empty() {
        return None;
    }

    let active: Vec<_> = contributions
        .values()
        .filter(|c| matches!(c.status, WorkerProgressStatus::Running) && c.current_progress > 0.0)
        .collect();

    if active.is_empty() {
        return None;
    }

    let remaining_fraction = (1.0 - completion_fraction).max(0.0);
    if remaining_fraction <= f32::EPSILON {
        return Some(chrono::Utc::now());
    }

    // Baseline assumption: 1% per minute
    let avg_progress_rate = 0.01_f32;
    let minutes_remaining = remaining_fraction / avg_progress_rate;
    let duration = chrono::Duration::minutes(minutes_remaining.max(0.0) as i64);

    Some(chrono::Utc::now() + duration)
}
