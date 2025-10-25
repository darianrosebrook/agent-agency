//! Context monitoring and progress tracking for the self-prompting loop

use std::cell::RefCell;
use chrono::{DateTime, Utc};

/// Metrics for context utilization monitoring
#[derive(Debug, Clone)]
pub struct ContextMetrics {
    /// Current prompt size in tokens
    pub prompt_size_tokens: usize,
    /// Context window utilization (0.0-1.0)
    pub context_window_utilization: f64,
    /// Number of files currently in scope
    pub files_in_scope: usize,
    /// Dependency depth of current analysis
    pub dependency_depth: usize,
    /// Timestamp of last measurement
    pub timestamp: DateTime<Utc>,
}

/// Context utilization monitor to prevent overload
#[derive(Debug, Clone)]
pub struct ContextMonitor {
    /// Current context metrics
    pub metrics: ContextMetrics,
    /// Overload threshold (0.0-1.0)
    pub overload_threshold: f64,
}

impl ContextMonitor {
    /// Create a new context monitor with default settings
    pub fn new(overload_threshold: f64) -> Self {
        Self {
            metrics: ContextMetrics {
                prompt_size_tokens: 0,
                context_window_utilization: 0.0,
                files_in_scope: 0,
                dependency_depth: 0,
                timestamp: Utc::now(),
            },
            overload_threshold,
        }
    }

    /// Update context metrics
    pub fn update_metrics(&mut self, metrics: ContextMetrics) {
        self.metrics = metrics;
    }

    /// Check if context is overloaded
    pub fn is_overloaded(&self) -> bool {
        self.metrics.context_window_utilization >= self.overload_threshold
    }

    /// Get current utilization percentage
    pub fn utilization_percentage(&self) -> f64 {
        self.metrics.context_window_utilization
    }

    /// Check if context needs optimization
    pub fn needs_optimization(&self) -> bool {
        self.metrics.context_window_utilization > 0.6 // 60% threshold
    }
}

/// Progress tracking for iteration-based execution
#[derive(Debug, Clone)]
pub struct IterationProgress {
    /// Iteration number
    pub iteration: usize,
    /// Quantitative progress score (0.0-1.0)
    pub progress_score: f64,
    /// Artifacts generated in this iteration
    pub artifacts_generated: usize,
    /// Models used in this iteration
    pub models_used: Vec<String>,
    /// Timestamp of this progress measurement
    pub timestamp: DateTime<Utc>,
}

/// Progress tracker for detecting plateaus and optimization opportunities
#[derive(Debug)]
pub struct ProgressTracker {
    /// History of iteration progress
    history: RefCell<Vec<IterationProgress>>,
    /// Maximum history size to prevent unbounded growth
    max_history_size: usize,
}

impl ProgressTracker {
    /// Create a new progress tracker
    pub fn new(max_history_size: usize) -> Self {
        Self {
            history: RefCell::new(Vec::new()),
            max_history_size,
        }
    }

    /// Record progress for an iteration
    pub fn record_progress(&self, progress: IterationProgress) {
        let mut history = self.history.borrow_mut();
        history.push(progress);

        // Maintain maximum history size
        if history.len() > self.max_history_size {
            history.remove(0);
        }
    }

    /// Check if progress has plateaued (no significant improvement in last N iterations)
    pub fn has_plateaued(&self, window_size: usize, improvement_threshold: f64) -> bool {
        let history = self.history.borrow();
        if history.len() < window_size {
            return false;
        }

        let recent = &history[history.len().saturating_sub(window_size)..];
        if recent.len() < 2 {
            return false;
        }

        let first_score = recent[0].progress_score;
        let last_score = recent[recent.len() - 1].progress_score;

        // Check if improvement is below threshold
        (last_score - first_score) < improvement_threshold
    }

    /// Get recent progress trend (positive = improving, negative = declining)
    pub fn get_progress_trend(&self, window_size: usize) -> f64 {
        let history = self.history.borrow();
        if history.len() < window_size {
            return 0.0;
        }

        let recent = &history[history.len().saturating_sub(window_size)..];
        if recent.len() < 2 {
            return 0.0;
        }

        // Simple linear trend calculation
        let n = recent.len() as f64;
        let sum_x: f64 = (0..recent.len()).map(|i| i as f64).sum();
        let sum_y: f64 = recent.iter().map(|p| p.progress_score).sum();
        let sum_xy: f64 = recent.iter().enumerate()
            .map(|(i, p)| (i as f64) * p.progress_score)
            .sum();
        let sum_x2: f64 = (0..recent.len()).map(|i| (i as f64).powi(2)).sum();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x.powi(2));
        slope
    }

    /// Get latest progress
    pub fn latest_progress(&self) -> Option<IterationProgress> {
        self.history.borrow().last().cloned()
    }

    /// Get progress history
    pub fn get_history(&self) -> Vec<IterationProgress> {
        self.history.borrow().clone()
    }
}
