//! Memory Reinforcement Learning
//!
//! Adaptive importance adjustment based on usage patterns and outcomes.

use crate::long_term_management::*;
use serde::{Deserialize, Serialize};
/// Memory reinforcement configuration
#[derive(Debug, Clone)]
pub struct ReinforcementConfig {
    pub learning_rate: f32,
    pub discount_factor: f32,
    pub exploration_rate: f32,
    pub max_importance_score: f32,
    pub reinforcement_window_hours: u64,
}

/// Memory reinforcement engine
pub struct MemoryReinforcementEngine {
    config: ReinforcementConfig,
    reinforcement_history:
        std::collections::HashMap<crate::memory_types::MemoryId, Vec<ReinforcementEvent>>,
}

impl MemoryReinforcementEngine {
    pub fn new(config: ReinforcementConfig) -> Self {
        Self {
            config,
            reinforcement_history: std::collections::HashMap::new(),
        }
    }

    /// Apply reinforcement based on memory usage outcome
    pub async fn apply_reinforcement(
        &mut self,
        memory_id: &crate::memory_types::MemoryId,
        outcome: &ReinforcementOutcome,
        context: &ReinforcementContext,
    ) -> crate::MemoryResult<f32> {
        let reinforcement_value = self.calculate_reinforcement_value(outcome, context);

        // Record the reinforcement event
        let event = ReinforcementEvent {
            timestamp: chrono::Utc::now(),
            outcome: outcome.clone(),
            context: context.clone(),
            reinforcement_value,
        };

        self.reinforcement_history
            .entry(memory_id.clone())
            .or_insert_with(Vec::new)
            .push(event);

        // Apply learning rate to reinforcement
        let adjusted_reinforcement = reinforcement_value * self.config.learning_rate;

        Ok(adjusted_reinforcement)
    }

    /// Calculate reinforcement value based on outcome and context
    fn calculate_reinforcement_value(
        &self,
        outcome: &ReinforcementOutcome,
        context: &ReinforcementContext,
    ) -> f32 {
        let base_value = match outcome {
            ReinforcementOutcome::Success => 1.0,
            ReinforcementOutcome::PartialSuccess => 0.5,
            ReinforcementOutcome::Failure => -1.0,
            ReinforcementOutcome::Neutral => 0.0,
        };

        // Apply context multipliers
        let context_multiplier = self.calculate_context_multiplier(context);

        // Apply temporal discount
        let temporal_discount = self
            .config
            .discount_factor
            .powi(context.temporal_distance as i32);

        base_value * context_multiplier * temporal_discount
    }

    /// Calculate context multiplier for reinforcement
    fn calculate_context_multiplier(&self, context: &ReinforcementContext) -> f32 {
        let mut multiplier = 1.0;

        // Importance of the task that used the memory
        multiplier *= match context.task_importance {
            TaskImportance::Critical => 2.0,
            TaskImportance::High => 1.5,
            TaskImportance::Medium => 1.0,
            TaskImportance::Low => 0.5,
        };

        // User feedback
        multiplier *= match context.user_feedback {
            UserFeedback::VeryPositive => 1.8,
            UserFeedback::Positive => 1.3,
            UserFeedback::Neutral => 1.0,
            UserFeedback::Negative => 0.7,
            UserFeedback::VeryNegative => 0.4,
        };

        // Memory relevance
        multiplier *= context.memory_relevance;

        // Exploration bonus
        if context.was_exploratory {
            multiplier *= (1.0 + self.config.exploration_rate);
        }

        multiplier
    }

    /// Get reinforcement history for a memory
    pub fn get_reinforcement_history(
        &self,
        memory_id: &crate::memory_types::MemoryId,
    ) -> Vec<ReinforcementEvent> {
        self.reinforcement_history
            .get(memory_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Calculate long-term importance trend for a memory
    pub fn calculate_importance_trend(
        &self,
        memory_id: &crate::memory_types::MemoryId,
    ) -> ImportanceTrend {
        let history = self.get_reinforcement_history(memory_id);

        if history.is_empty() {
            return ImportanceTrend::Stable;
        }

        // Analyze reinforcement values over time
        let recent_reinforcements: Vec<f32> = history
            .iter()
            .rev()
            .take(10) // Last 10 reinforcements
            .map(|event| event.reinforcement_value)
            .collect();

        let average_recent =
            recent_reinforcements.iter().sum::<f32>() / recent_reinforcements.len() as f32;

        let older_reinforcements: Vec<f32> = history
            .iter()
            .rev()
            .skip(10)
            .take(10) // Previous 10 reinforcements
            .map(|event| event.reinforcement_value)
            .collect();

        let trend = if older_reinforcements.is_empty() {
            ImportanceTrend::Stable
        } else {
            let average_older =
                older_reinforcements.iter().sum::<f32>() / older_reinforcements.len() as f32;
            let change = average_recent - average_older;

            if change > 0.2 {
                ImportanceTrend::Increasing
            } else if change < -0.2 {
                ImportanceTrend::Decreasing
            } else {
                ImportanceTrend::Stable
            }
        };

        trend
    }

    /// Predict future importance based on reinforcement patterns
    pub fn predict_future_importance(
        &self,
        memory_id: &crate::memory_types::MemoryId,
        current_importance: f32,
    ) -> f32 {
        let trend = self.calculate_importance_trend(memory_id);
        let history = self.get_reinforcement_history(memory_id);

        if history.is_empty() {
            return current_importance;
        }

        // Simple trend-based prediction
        let trend_factor = match trend {
            ImportanceTrend::Increasing => 1.1,
            ImportanceTrend::Decreasing => 0.9,
            ImportanceTrend::Stable => 1.0,
        };

        // Apply diminishing returns for very high importance
        let diminishing_factor = if current_importance > 0.8 { 0.95 } else { 1.0 };

        (current_importance * trend_factor * diminishing_factor)
            .min(self.config.max_importance_score)
    }

    /// Clean up old reinforcement history to prevent unbounded growth
    pub fn cleanup_old_history(&mut self, max_age_days: u64) {
        let cutoff = chrono::Utc::now() - chrono::Duration::days(max_age_days as i64);

        for history in self.reinforcement_history.values_mut() {
            history.retain(|event| event.timestamp > cutoff);
        }

        // Remove memories with no history
        self.reinforcement_history
            .retain(|_, history| !history.is_empty());
    }
}

/// Reinforcement outcome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReinforcementOutcome {
    Success,
    PartialSuccess,
    Failure,
    Neutral,
}

/// Reinforcement context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReinforcementContext {
    pub task_importance: TaskImportance,
    pub user_feedback: UserFeedback,
    pub memory_relevance: f32,  // 0.0 to 1.0
    pub temporal_distance: i32, // Hours since memory was created
    pub was_exploratory: bool,
    pub usage_context: UsageContext,
}

/// Task importance levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskImportance {
    Critical,
    High,
    Medium,
    Low,
}

/// User feedback levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserFeedback {
    VeryPositive,
    Positive,
    Neutral,
    Negative,
    VeryNegative,
}

/// Usage context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UsageContext {
    DirectQuery,
    ContextualRetrieval,
    BackgroundProcessing,
    LearningTask,
    CreativeTask,
}

/// Reinforcement event record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReinforcementEvent {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub outcome: ReinforcementOutcome,
    pub context: ReinforcementContext,
    pub reinforcement_value: f32,
}

/// Importance trend
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportanceTrend {
    Increasing,
    Decreasing,
    Stable,
}

/// Reinforcement statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReinforcementStats {
    pub total_reinforcements: usize,
    pub positive_reinforcements: usize,
    pub negative_reinforcements: usize,
    pub average_reinforcement_value: f32,
    pub learning_efficiency: f32,
    pub last_reinforcement: Option<chrono::DateTime<chrono::Utc>>,
}
