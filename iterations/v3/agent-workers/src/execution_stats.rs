//! Execution statistics and metrics for parallel task execution
//! 
//! This module contains structures and utilities for tracking and analyzing
//! execution statistics and performance metrics.

use crate::parallel_types::{TaskId, SubTaskId, WorkerId};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// Statistics for parallel execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelExecutionStats {
    /// Total number of tasks executed
    pub total_tasks: u32,
    /// Number of successful tasks
    pub successful_tasks: u32,
    /// Number of failed tasks
    pub failed_tasks: u32,
    /// Number of cancelled tasks
    pub cancelled_tasks: u32,
    /// Total execution time in milliseconds
    pub total_execution_time_ms: u64,
    /// Average execution time per task in milliseconds
    pub avg_execution_time_ms: f64,
    /// Total number of subtasks created
    pub total_subtasks: u32,
    /// Number of workers used
    pub workers_used: u32,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Throughput (tasks per second)
    pub throughput_tasks_per_second: f64,
    /// Quality score (0.0 to 1.0)
    pub quality_score: f64,
    /// Resource utilization (0.0 to 1.0)
    pub resource_utilization: f64,
    /// Timestamp when stats were calculated
    pub calculated_at: DateTime<Utc>,
}

impl Default for ParallelExecutionStats {
    fn default() -> Self {
        Self {
            total_tasks: 0,
            successful_tasks: 0,
            failed_tasks: 0,
            cancelled_tasks: 0,
            total_execution_time_ms: 0,
            avg_execution_time_ms: 0.0,
            total_subtasks: 0,
            workers_used: 0,
            success_rate: 1.0,
            throughput_tasks_per_second: 0.0,
            quality_score: 1.0,
            resource_utilization: 0.0,
            calculated_at: Utc::now(),
        }
    }
}

impl ParallelExecutionStats {
    /// Create new execution stats
    pub fn new() -> Self {
        Self::default()
    }

    /// Update stats with new task execution data
    pub fn update_with_task(
        &mut self,
        task_id: TaskId,
        execution_time_ms: u64,
        success: bool,
        quality_score: f64,
        subtask_count: u32,
        worker_count: u32,
    ) {
        self.total_tasks += 1;
        self.total_execution_time_ms += execution_time_ms;
        self.total_subtasks += subtask_count;
        self.workers_used = self.workers_used.max(worker_count);

        if success {
            self.successful_tasks += 1;
        } else {
            self.failed_tasks += 1;
        }

        // Recalculate derived metrics
        self.recalculate_metrics();
    }

    /// Update stats with cancellation
    pub fn update_with_cancellation(&mut self) {
        self.total_tasks += 1;
        self.cancelled_tasks += 1;
        self.recalculate_metrics();
    }

    /// Recalculate all derived metrics
    fn recalculate_metrics(&mut self) {
        if self.total_tasks > 0 {
            self.avg_execution_time_ms = self.total_execution_time_ms as f64 / self.total_tasks as f64;
            self.success_rate = self.successful_tasks as f64 / self.total_tasks as f64;
            
            // Calculate throughput (tasks per second)
            if self.total_execution_time_ms > 0 {
                self.throughput_tasks_per_second = (self.total_tasks as f64 * 1000.0) / self.total_execution_time_ms as f64;
            }
        }

        self.calculated_at = Utc::now();
    }

    /// Get performance summary
    pub fn get_performance_summary(&self) -> PerformanceSummary {
        PerformanceSummary {
            success_rate: self.success_rate,
            avg_execution_time_ms: self.avg_execution_time_ms,
            throughput_tasks_per_second: self.throughput_tasks_per_second,
            quality_score: self.quality_score,
            resource_utilization: self.resource_utilization,
            efficiency_score: self.calculate_efficiency_score(),
        }
    }

    /// Calculate overall efficiency score
    fn calculate_efficiency_score(&self) -> f64 {
        let time_efficiency = if self.avg_execution_time_ms > 0.0 {
            (300000.0 / self.avg_execution_time_ms).min(1.0) // 5 minutes baseline
        } else {
            1.0
        };

        let throughput_efficiency = (self.throughput_tasks_per_second / 10.0).min(1.0); // 10 tasks/sec baseline
        let quality_efficiency = self.quality_score;
        let success_efficiency = self.success_rate;

        (time_efficiency * 0.3 + throughput_efficiency * 0.2 + quality_efficiency * 0.3 + success_efficiency * 0.2)
            .min(1.0)
            .max(0.0)
    }

    /// Merge with another stats object
    pub fn merge(&mut self, other: &ParallelExecutionStats) {
        self.total_tasks += other.total_tasks;
        self.successful_tasks += other.successful_tasks;
        self.failed_tasks += other.failed_tasks;
        self.cancelled_tasks += other.cancelled_tasks;
        self.total_execution_time_ms += other.total_execution_time_ms;
        self.total_subtasks += other.total_subtasks;
        self.workers_used = self.workers_used.max(other.workers_used);
        
        self.recalculate_metrics();
    }

    /// Reset all stats
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

/// Performance summary for quick analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub success_rate: f64,
    pub avg_execution_time_ms: f64,
    pub throughput_tasks_per_second: f64,
    pub quality_score: f64,
    pub resource_utilization: f64,
    pub efficiency_score: f64,
}

/// Worker performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerPerformanceMetrics {
    pub worker_id: WorkerId,
    pub tasks_completed: u32,
    pub tasks_failed: u32,
    pub avg_execution_time_ms: f64,
    pub success_rate: f64,
    pub quality_score: f64,
    pub last_active: DateTime<Utc>,
    pub utilization_percentage: f64,
}

impl WorkerPerformanceMetrics {
    pub fn new(worker_id: WorkerId) -> Self {
        Self {
            worker_id,
            tasks_completed: 0,
            tasks_failed: 0,
            avg_execution_time_ms: 0.0,
            success_rate: 1.0,
            quality_score: 1.0,
            last_active: Utc::now(),
            utilization_percentage: 0.0,
        }
    }

    pub fn update_with_task(&mut self, execution_time_ms: u64, success: bool, quality_score: f64) {
        if success {
            self.tasks_completed += 1;
        } else {
            self.tasks_failed += 1;
        }

        let total_tasks = self.tasks_completed + self.tasks_failed;
        if total_tasks > 0 {
            self.success_rate = self.tasks_completed as f64 / total_tasks as f64;
            self.avg_execution_time_ms = (self.avg_execution_time_ms * (total_tasks - 1) as f64 + execution_time_ms as f64) / total_tasks as f64;
            self.quality_score = (self.quality_score * (total_tasks - 1) as f64 + quality_score) / total_tasks as f64;
        }

        self.last_active = Utc::now();
    }
}

/// Task complexity analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskComplexityAnalysis {
    pub task_id: TaskId,
    pub complexity_score: f64,
    pub estimated_execution_time_ms: u64,
    pub required_workers: u32,
    pub difficulty_factors: Vec<String>,
    pub optimization_suggestions: Vec<String>,
    pub analyzed_at: DateTime<Utc>,
}

impl TaskComplexityAnalysis {
    pub fn new(task_id: TaskId) -> Self {
        Self {
            task_id,
            complexity_score: 0.5,
            estimated_execution_time_ms: 300000, // 5 minutes default
            required_workers: 1,
            difficulty_factors: Vec::new(),
            optimization_suggestions: Vec::new(),
            analyzed_at: Utc::now(),
        }
    }

    pub fn analyze_complexity(&mut self, factors: Vec<String>) {
        self.difficulty_factors = factors.clone();
        
        // Calculate complexity score based on factors
        let mut score = 0.5; // Base score
        
        for factor in &factors {
            match factor.as_str() {
                "large_codebase" => score += 0.2,
                "complex_dependencies" => score += 0.15,
                "multiple_languages" => score += 0.1,
                "external_integrations" => score += 0.15,
                "performance_critical" => score += 0.1,
                "security_sensitive" => score += 0.1,
                "legacy_code" => score += 0.2,
                _ => score += 0.05,
            }
        }
        
        self.complexity_score = score.min(1.0).max(0.0);
        
        // Estimate execution time based on complexity
        self.estimated_execution_time_ms = (300000.0 * (0.5 + self.complexity_score * 1.5)) as u64;
        
        // Estimate required workers
        self.required_workers = if self.complexity_score > 0.8 { 4 } else if self.complexity_score > 0.6 { 3 } else if self.complexity_score > 0.4 { 2 } else { 1 };
        
        // Generate optimization suggestions
        self.generate_optimization_suggestions();
    }

    fn generate_optimization_suggestions(&mut self) {
        self.optimization_suggestions.clear();
        
        if self.complexity_score > 0.8 {
            self.optimization_suggestions.push("Consider breaking into smaller subtasks".to_string());
            self.optimization_suggestions.push("Use specialized workers for different components".to_string());
        }
        
        if self.difficulty_factors.contains(&"large_codebase".to_string()) {
            self.optimization_suggestions.push("Focus on specific modules or files".to_string());
        }
        
        if self.difficulty_factors.contains(&"complex_dependencies".to_string()) {
            self.optimization_suggestions.push("Resolve dependencies in parallel".to_string());
        }
        
        if self.estimated_execution_time_ms > 600000 { // 10 minutes
            self.optimization_suggestions.push("Increase timeout or optimize task".to_string());
        }
    }
}

/// Execution timeline for tracking task progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTimeline {
    pub task_id: TaskId,
    pub events: Vec<TimelineEvent>,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub total_duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub event_type: String,
    pub timestamp: DateTime<Utc>,
    pub description: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ExecutionTimeline {
    pub fn new(task_id: TaskId) -> Self {
        Self {
            task_id,
            events: Vec::new(),
            start_time: Utc::now(),
            end_time: None,
            total_duration_ms: None,
        }
    }

    pub fn add_event(&mut self, event_type: String, description: String, metadata: HashMap<String, serde_json::Value>) {
        let event = TimelineEvent {
            event_type,
            timestamp: Utc::now(),
            description,
            metadata,
        };
        self.events.push(event);
    }

    pub fn finish(&mut self) {
        self.end_time = Some(Utc::now());
        if let Some(end_time) = self.end_time {
            self.total_duration_ms = Some(end_time.signed_duration_since(self.start_time).num_milliseconds() as u64);
        }
    }

    pub fn get_duration_ms(&self) -> Option<u64> {
        self.total_duration_ms.or_else(|| {
            Some(Utc::now().signed_duration_since(self.start_time).num_milliseconds() as u64)
        })
    }
}
