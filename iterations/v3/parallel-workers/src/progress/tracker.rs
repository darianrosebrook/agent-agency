//! Individual worker progress tracking

use crate::types::*;
use crate::error::*;
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

/// Tracks progress for individual workers
pub struct WorkerProgressTracker {
    progress: Arc<RwLock<HashMap<WorkerId, WorkerProgress>>>,
}

impl WorkerProgressTracker {
    pub fn new() -> Self {
        Self {
            progress: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Update progress for a specific worker
    pub fn update_progress(&self, worker_id: WorkerId, completed: u32, total: u32, status: String) -> ProgressResult<()> {
        let mut progress_map = self.progress.write();

        let worker_progress = progress_map.entry(worker_id).or_insert_with(|| WorkerProgress {
            worker_id: WorkerId(String::new()), // Will be set below
            subtask_id: SubTaskId(String::new()), // Will be set when subtask is assigned
            completed: 0,
            total: 0,
            task_weight: 1.0,
            status: String::new(),
            last_update: chrono::Utc::now(),
        });

        worker_progress.completed = completed;
        worker_progress.total = total;
        worker_progress.status = status;
        worker_progress.last_update = chrono::Utc::now();

        Ok(())
    }

    /// Assign a subtask to a worker
    pub fn assign_subtask(&self, worker_id: WorkerId, subtask_id: SubTaskId, task_weight: f32) -> ProgressResult<()> {
        let mut progress_map = self.progress.write();

        let worker_progress = progress_map.entry(worker_id.clone()).or_insert_with(|| WorkerProgress {
            worker_id: worker_id.clone(),
            subtask_id: subtask_id.clone(),
            completed: 0,
            total: 0,
            task_weight,
            status: "assigned".to_string(),
            last_update: chrono::Utc::now(),
        });

        worker_progress.subtask_id = subtask_id;
        worker_progress.task_weight = task_weight;
        worker_progress.status = "assigned".to_string();
        worker_progress.last_update = chrono::Utc::now();

        Ok(())
    }

    /// Mark a worker as started
    pub fn mark_started(&self, worker_id: &WorkerId) -> ProgressResult<()> {
        let mut progress_map = self.progress.write();

        if let Some(worker_progress) = progress_map.get_mut(worker_id) {
            worker_progress.status = "running".to_string();
            worker_progress.last_update = chrono::Utc::now();
        }

        Ok(())
    }

    /// Mark a worker as completed
    pub fn mark_completed(&self, worker_id: &WorkerId) -> ProgressResult<()> {
        let mut progress_map = self.progress.write();

        if let Some(worker_progress) = progress_map.get_mut(worker_id) {
            worker_progress.completed = worker_progress.total;
            worker_progress.status = "completed".to_string();
            worker_progress.last_update = chrono::Utc::now();
        }

        Ok(())
    }

    /// Mark a worker as failed
    pub fn mark_failed(&self, worker_id: &WorkerId, error: &str) -> ProgressResult<()> {
        let mut progress_map = self.progress.write();

        if let Some(worker_progress) = progress_map.get_mut(worker_id) {
            worker_progress.status = format!("failed: {}", error);
            worker_progress.last_update = chrono::Utc::now();
        }

        Ok(())
    }

    /// Mark a worker as blocked
    pub fn mark_blocked(&self, worker_id: &WorkerId, reason: &str) -> ProgressResult<()> {
        let mut progress_map = self.progress.write();

        if let Some(worker_progress) = progress_map.get_mut(worker_id) {
            worker_progress.status = format!("blocked: {}", reason);
            worker_progress.last_update = chrono::Utc::now();
        }

        Ok(())
    }

    /// Get progress for a specific worker
    pub fn get_worker_progress(&self, worker_id: &WorkerId) -> Option<WorkerProgress> {
        self.progress.read().get(worker_id).cloned()
    }

    /// Get all worker progress
    pub fn get_all_progress(&self) -> Vec<WorkerProgress> {
        self.progress.read().values().cloned().collect()
    }

    /// Get progress for workers assigned to a specific subtask
    pub fn get_subtask_progress(&self, subtask_id: &SubTaskId) -> Vec<WorkerProgress> {
        self.progress.read()
            .values()
            .filter(|wp| &wp.subtask_id == subtask_id)
            .cloned()
            .collect()
    }

    /// Remove progress tracking for a worker
    pub fn remove_worker(&self, worker_id: &WorkerId) -> ProgressResult<()> {
        self.progress.write().remove(worker_id);
        Ok(())
    }

    /// Get statistics about worker progress
    pub fn get_stats(&self) -> WorkerProgressStats {
        let progress_map = self.progress.read();

        let mut running = 0;
        let mut completed = 0;
        let mut failed = 0;
        let mut blocked = 0;
        let mut total_progress = 0.0;
        let mut total_weight = 0.0;

        for worker_progress in progress_map.values() {
            total_weight += worker_progress.task_weight;

            match worker_progress.status.as_str() {
                s if s.starts_with("running") => running += 1,
                s if s.starts_with("completed") => {
                    completed += 1;
                    total_progress += worker_progress.task_weight;
                }
                s if s.starts_with("failed") => failed += 1,
                s if s.starts_with("blocked") => blocked += 1,
                _ => {} // assigned, etc.
            }
        }

        let overall_progress = if total_weight > 0.0 {
            total_progress / total_weight
        } else {
            0.0
        };

        WorkerProgressStats {
            total_workers: progress_map.len(),
            running_workers: running,
            completed_workers: completed,
            failed_workers: failed,
            blocked_workers: blocked,
            overall_progress,
        }
    }
}

/// Statistics for worker progress tracking
#[derive(Debug, Clone)]
pub struct WorkerProgressStats {
    pub total_workers: usize,
    pub running_workers: usize,
    pub completed_workers: usize,
    pub failed_workers: usize,
    pub blocked_workers: usize,
    pub overall_progress: f32,
}

/// Progress history for tracking changes over time
pub struct ProgressHistory {
    history: Arc<RwLock<Vec<ProgressSnapshot>>>,
    max_history_size: usize,
}

impl ProgressHistory {
    pub fn new(max_history_size: usize) -> Self {
        Self {
            history: Arc::new(RwLock::new(Vec::new())),
            max_history_size,
        }
    }

    /// Record a progress snapshot
    pub fn record_snapshot(&self, stats: WorkerProgressStats) -> ProgressResult<()> {
        let mut history = self.history.write();

        let snapshot = ProgressSnapshot {
            timestamp: chrono::Utc::now(),
            stats,
        };

        history.push(snapshot);

        // Keep only the most recent snapshots
        if history.len() > self.max_history_size {
            history.remove(0);
        }

        Ok(())
    }

    /// Get the most recent snapshot
    pub fn latest_snapshot(&self) -> Option<ProgressSnapshot> {
        self.history.read().last().cloned()
    }

    /// Get all snapshots within a time range
    pub fn snapshots_in_range(&self, start: chrono::DateTime<chrono::Utc>, end: chrono::DateTime<chrono::Utc>) -> Vec<ProgressSnapshot> {
        self.history.read()
            .iter()
            .filter(|snapshot| snapshot.timestamp >= start && snapshot.timestamp <= end)
            .cloned()
            .collect()
    }

    /// Get progress trend (recent snapshots)
    pub fn get_trend(&self, count: usize) -> Vec<ProgressSnapshot> {
        let history = self.history.read();
        let start_index = history.len().saturating_sub(count);
        history[start_index..].to_vec()
    }
}

/// A snapshot of progress at a specific point in time
#[derive(Debug, Clone)]
pub struct ProgressSnapshot {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub stats: WorkerProgressStats,
}

/// Progress predictor for estimating completion times
pub struct ProgressPredictor {
    history: ProgressHistory,
}

impl ProgressPredictor {
    pub fn new(history: ProgressHistory) -> Self {
        Self { history }
    }

    /// Predict completion time based on current progress
    pub fn predict_completion(&self, current_progress: f32) -> Option<chrono::DateTime<chrono::Utc>> {
        let trend = self.history.get_trend(5); // Use last 5 snapshots

        if trend.len() < 2 {
            return None;
        }

        // Calculate average progress rate
        let mut total_rate = 0.0;
        let mut rate_count = 0;

        for i in 1..trend.len() {
            let time_diff = trend[i].timestamp.signed_duration_since(trend[i-1].timestamp).num_seconds() as f32;
            let progress_diff = trend[i].stats.overall_progress - trend[i-1].stats.overall_progress;

            if time_diff > 0.0 {
                let rate = progress_diff / time_diff;
                if rate > 0.0 {
                    total_rate += rate;
                    rate_count += 1;
                }
            }
        }

        if rate_count == 0 || total_rate <= 0.0 {
            return None;
        }

        let avg_rate = total_rate / rate_count as f32;
        let remaining_progress = 1.0 - current_progress;

        if remaining_progress <= 0.0 {
            return Some(chrono::Utc::now());
        }

        let seconds_remaining = remaining_progress / avg_rate;
        let duration = chrono::Duration::seconds(seconds_remaining as i64);

        Some(chrono::Utc::now() + duration)
    }
}

impl Default for WorkerProgressTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ProgressHistory {
    fn default() -> Self {
        Self::new(100) // Keep last 100 snapshots
    }
}
