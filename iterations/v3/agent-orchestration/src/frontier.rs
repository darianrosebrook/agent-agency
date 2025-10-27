//! Frontier task queue for orchestration
//!
//! This module provides task queue functionality for managing and prioritizing
//! orchestration tasks.
//!
//! @author @darianrosebrook

use crate::types::{TaskDescriptor, TaskPriority};
use anyhow::{Context, Result};
use std::collections::{BinaryHeap, HashMap};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Frontier task queue for orchestration
#[derive(Debug)]
pub struct Frontier {
    /// Task queue with priority ordering
    queue: Arc<RwLock<BinaryHeap<TaskEntry>>>,
    /// Task registry for tracking
    registry: Arc<RwLock<HashMap<String, TaskEntry>>>,
    /// Configuration
    config: FrontierConfig,
    /// Statistics
    stats: Arc<RwLock<FrontierStats>>,
}

/// Configuration for frontier
#[derive(Debug, Clone)]
pub struct FrontierConfig {
    /// Maximum queue size
    pub max_queue_size: usize,
    /// Task timeout in seconds
    pub task_timeout_seconds: u64,
    /// Enable priority boosting for stale tasks
    pub enable_priority_boost: bool,
    /// Priority boost threshold in seconds
    pub priority_boost_threshold_seconds: u64,
}

impl Default for FrontierConfig {
    fn default() -> Self {
        Self {
            max_queue_size: 1000,
            task_timeout_seconds: 3600, // 1 hour
            enable_priority_boost: true,
            priority_boost_threshold_seconds: 300, // 5 minutes
        }
    }
}

/// Task entry in the frontier queue
#[derive(Debug, Clone)]
pub struct TaskEntry {
    /// Task descriptor
    pub descriptor: TaskDescriptor,
    /// Priority score (higher = more priority)
    pub priority_score: u32,
    /// Time when task was added
    pub added_at: Instant,
    /// Time when task was last processed
    pub last_processed_at: Option<Instant>,
    /// Number of processing attempts
    pub attempts: u32,
    /// Task status
    pub status: TaskStatus,
}

/// Task status in frontier
#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
    Timeout,
}

/// Frontier statistics
#[derive(Debug, Clone, Default)]
pub struct FrontierStats {
    /// Total tasks added
    pub total_added: u64,
    /// Total tasks processed
    pub total_processed: u64,
    /// Total tasks completed
    pub total_completed: u64,
    /// Total tasks failed
    pub total_failed: u64,
    /// Total tasks cancelled
    pub total_cancelled: u64,
    /// Total tasks timed out
    pub total_timeout: u64,
    /// Average processing time in seconds
    pub avg_processing_time_seconds: f64,
    /// Current queue size
    pub current_queue_size: usize,
}

impl Frontier {
    /// Create a new frontier
    pub fn new(config: FrontierConfig) -> Self {
        Self {
            queue: Arc::new(RwLock::new(BinaryHeap::new())),
            registry: Arc::new(RwLock::new(HashMap::new())),
            config,
            stats: Arc::new(RwLock::new(FrontierStats::default())),
        }
    }

    /// Add a task to the frontier
    pub async fn add_task(&self, descriptor: TaskDescriptor) -> Result<()> {
        debug!("Adding task to frontier: {}", descriptor.task_id);

        let priority_score = self.calculate_priority_score(&descriptor);
        let task_entry = TaskEntry {
            descriptor,
            priority_score,
            added_at: Instant::now(),
            last_processed_at: None,
            attempts: 0,
            status: TaskStatus::Pending,
        };

        // Check queue size limit
        {
            let queue = self.queue.read().unwrap();
            if queue.len() >= self.config.max_queue_size {
                return Err(anyhow::anyhow!("Frontier queue is full"));
            }
        }

        // Add to queue and registry
        {
            let mut queue = self.queue.write().unwrap();
            let mut registry = self.registry.write().unwrap();
            
            queue.push(task_entry.clone());
            registry.insert(task_entry.descriptor.task_id.clone(), task_entry);
        }

        // Update statistics
        {
            let mut stats = self.stats.write().unwrap();
            stats.total_added += 1;
            stats.current_queue_size += 1;
        }

        info!("Task added to frontier: {}", task_entry.descriptor.task_id);
        Ok(())
    }

    /// Get the next task to process
    pub async fn get_next_task(&self) -> Result<Option<TaskEntry>> {
        debug!("Getting next task from frontier");

        let mut queue = self.queue.write().unwrap();
        let mut registry = self.registry.write().unwrap();

        // Find next pending task
        let mut pending_tasks = Vec::new();
        let mut found_task = None;

        while let Some(task) = queue.pop() {
            if task.status == TaskStatus::Pending {
                found_task = Some(task);
                break;
            } else {
                pending_tasks.push(task);
            }
        }

        // Put back non-pending tasks
        for task in pending_tasks {
            queue.push(task);
        }

        if let Some(mut task) = found_task {
            // Update task status
            task.status = TaskStatus::Processing;
            task.last_processed_at = Some(Instant::now());
            task.attempts += 1;

            // Update registry
            registry.insert(task.descriptor.task_id.clone(), task.clone());

            // Put back in queue
            queue.push(task.clone());

            info!("Retrieved task from frontier: {}", task.descriptor.task_id);
            Ok(Some(task))
        } else {
            debug!("No pending tasks in frontier");
            Ok(None)
        }
    }

    /// Mark a task as completed
    pub async fn complete_task(&self, task_id: &str) -> Result<()> {
        debug!("Marking task as completed: {}", task_id);

        self.update_task_status(task_id, TaskStatus::Completed).await?;

        // Update statistics
        {
            let mut stats = self.stats.write().unwrap();
            stats.total_completed += 1;
            stats.current_queue_size = stats.current_queue_size.saturating_sub(1);
        }

        info!("Task completed: {}", task_id);
        Ok(())
    }

    /// Mark a task as failed
    pub async fn fail_task(&self, task_id: &str) -> Result<()> {
        debug!("Marking task as failed: {}", task_id);

        self.update_task_status(task_id, TaskStatus::Failed).await?;

        // Update statistics
        {
            let mut stats = self.stats.write().unwrap();
            stats.total_failed += 1;
            stats.current_queue_size = stats.current_queue_size.saturating_sub(1);
        }

        warn!("Task failed: {}", task_id);
        Ok(())
    }

    /// Cancel a task
    pub async fn cancel_task(&self, task_id: &str) -> Result<()> {
        debug!("Cancelling task: {}", task_id);

        self.update_task_status(task_id, TaskStatus::Cancelled).await?;

        // Update statistics
        {
            let mut stats = self.stats.write().unwrap();
            stats.total_cancelled += 1;
            stats.current_queue_size = stats.current_queue_size.saturating_sub(1);
        }

        info!("Task cancelled: {}", task_id);
        Ok(())
    }

    /// Update task status
    async fn update_task_status(&self, task_id: &str, status: TaskStatus) -> Result<()> {
        let mut registry = self.registry.write().unwrap();
        
        if let Some(task) = registry.get_mut(task_id) {
            task.status = status;
        } else {
            return Err(anyhow::anyhow!("Task not found: {}", task_id));
        }

        Ok(())
    }

    /// Calculate priority score for a task
    fn calculate_priority_score(&self, descriptor: &TaskDescriptor) -> u32 {
        let mut score = 0;

        // Base priority score
        match descriptor.priority {
            TaskPriority::Critical => score += 1000,
            TaskPriority::High => score += 800,
            TaskPriority::Medium => score += 500,
            TaskPriority::Low => score += 200,
        }

        // Boost score based on scope size (smaller scope = higher priority)
        let scope_size = descriptor.scope_in.in_scope.len();
        if scope_size <= 5 {
            score += 200;
        } else if scope_size <= 20 {
            score += 100;
        }

        // Boost score based on change budget (smaller budget = higher priority)
        if descriptor.change_budget.max_files <= 10 {
            score += 100;
        } else if descriptor.change_budget.max_files <= 50 {
            score += 50;
        }

        score
    }

    /// Clean up stale tasks
    pub async fn cleanup_stale_tasks(&self) -> Result<()> {
        debug!("Cleaning up stale tasks");

        let mut queue = self.queue.write().unwrap();
        let mut registry = self.registry.write().unwrap();
        let mut stats = self.stats.write().unwrap();

        let mut active_tasks = Vec::new();
        let mut cleaned_count = 0;

        while let Some(task) = queue.pop() {
            let is_stale = if let Some(last_processed) = task.last_processed_at {
                last_processed.elapsed().as_secs() > self.config.task_timeout_seconds
            } else {
                task.added_at.elapsed().as_secs() > self.config.task_timeout_seconds
            };

            if is_stale {
                cleaned_count += 1;
                stats.total_timeout += 1;
                stats.current_queue_size = stats.current_queue_size.saturating_sub(1);
            } else {
                active_tasks.push(task);
            }
        }

        // Put back active tasks
        for task in active_tasks {
            queue.push(task);
        }

        // Clean registry
        registry.retain(|_, task| {
            let is_stale = if let Some(last_processed) = task.last_processed_at {
                last_processed.elapsed().as_secs() > self.config.task_timeout_seconds
            } else {
                task.added_at.elapsed().as_secs() > self.config.task_timeout_seconds
            };
            !is_stale
        });

        if cleaned_count > 0 {
            info!("Cleaned up {} stale tasks", cleaned_count);
        }

        Ok(())
    }

    /// Get frontier statistics
    pub fn get_stats(&self) -> FrontierStats {
        let stats = self.stats.read().unwrap();
        let queue_size = self.queue.read().unwrap().len();
        
        FrontierStats {
            current_queue_size: queue_size,
            ..stats.clone()
        }
    }

    /// Get task by ID
    pub fn get_task(&self, task_id: &str) -> Option<TaskEntry> {
        let registry = self.registry.read().unwrap();
        registry.get(task_id).cloned()
    }

    /// Get all pending tasks
    pub fn get_pending_tasks(&self) -> Vec<TaskEntry> {
        let registry = self.registry.read().unwrap();
        registry
            .values()
            .filter(|task| task.status == TaskStatus::Pending)
            .cloned()
            .collect()
    }
}

// Implement Ord for TaskEntry to enable priority queue ordering
impl Ord for TaskEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Higher priority score = higher priority
        other.priority_score.cmp(&self.priority_score)
            .then_with(|| {
                // If priority scores are equal, older tasks have higher priority
                other.added_at.cmp(&self.added_at)
            })
    }
}

impl PartialOrd for TaskEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for TaskEntry {
    fn eq(&self, other: &Self) -> bool {
        self.descriptor.task_id == other.descriptor.task_id
    }
}

impl Eq for TaskEntry {}
