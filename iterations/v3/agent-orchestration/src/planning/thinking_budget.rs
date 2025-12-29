//! Thinking Budget Management
//!
//! Adaptive allocation of computational resources (reasoning steps, token budgets,
//! planning depth) based on task complexity. Ensures efficient resource utilization
//! while maintaining quality for complex tasks.
//!
//! @author @darianrosebrook

use anyhow::Result;
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use uuid::Uuid;

use agent_agency_contracts::WorkingSpec;

/// Task complexity assessment
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum TaskComplexity {
    /// Simple tasks (trivial changes, single file)
    Simple,

    /// Moderate tasks (multiple files, some dependencies)
    Moderate,

    /// Complex tasks (architectural changes, many dependencies)
    Complex,

    /// Very complex tasks (system-wide changes, high risk)
    VeryComplex,
}

impl TaskComplexity {
    /// Assess complexity from working spec
    pub fn assess(working_spec: &WorkingSpec) -> Self {
        let mut complexity_score = 0.0;

        // Factor 1: Scope size
        let scope_size = working_spec.allowed_paths().len();
        complexity_score += (scope_size as f64).min(10.0) / 10.0 * 0.2;

        // Factor 2: Change budget
        let file_budget = working_spec.change_budget.max_files as f64;
        let loc_budget = working_spec.change_budget.max_loc as f64;
        complexity_score += (file_budget / 25.0).min(1.0) * 0.2;
        complexity_score += (loc_budget / 1000.0).min(1.0) * 0.2;

        // Factor 3: Risk tier
        let risk_multiplier = match working_spec.risk_tier {
            1 => 1.5, // Critical systems
            2 => 1.0, // Standard
            3 => 0.7, // Low risk
            _ => 1.0,
        };
        complexity_score *= risk_multiplier;

        // Factor 4: Number of milestones
        let milestone_count = working_spec.acceptance_criteria.len();
        complexity_score += (milestone_count as f64 / 5.0).min(1.0) * 0.2;

        // Factor 5: Description length (proxy for task detail)
        let desc_length = working_spec.description.len();
        complexity_score += (desc_length as f64 / 1000.0).min(1.0) * 0.2;

        // Normalize to 0.0-1.0
        complexity_score = complexity_score.min(1.0);

        // Classify
        if complexity_score < 0.25 {
            TaskComplexity::Simple
        } else if complexity_score < 0.5 {
            TaskComplexity::Moderate
        } else if complexity_score < 0.75 {
            TaskComplexity::Complex
        } else {
            TaskComplexity::VeryComplex
        }
    }

    /// Get numeric complexity value (0.0-1.0)
    pub fn as_f64(&self) -> f64 {
        match self {
            TaskComplexity::Simple => 0.25,
            TaskComplexity::Moderate => 0.5,
            TaskComplexity::Complex => 0.75,
            TaskComplexity::VeryComplex => 1.0,
        }
    }
}

impl std::fmt::Display for TaskComplexity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskComplexity::Simple => write!(f, "simple"),
            TaskComplexity::Moderate => write!(f, "moderate"),
            TaskComplexity::Complex => write!(f, "complex"),
            TaskComplexity::VeryComplex => write!(f, "very_complex"),
        }
    }
}

/// Thinking budget allocation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ThinkingBudget {
    /// Maximum reasoning steps allowed
    pub max_reasoning_steps: u32,

    /// Maximum planning depth (recursion levels)
    pub max_planning_depth: u32,

    /// Token budget for reasoning (input + output)
    pub token_budget: u32,

    /// Time budget in milliseconds
    pub time_budget_ms: u64,

    /// Cost budget in cents
    pub cost_budget_cents: u32,

    /// Whether to allow budget expansion
    pub allow_expansion: bool,

    /// Expansion threshold (percentage of budget used before expansion)
    pub expansion_threshold: f64,

    /// Maximum expansion multiplier
    pub max_expansion_multiplier: f64,
}

impl ThinkingBudget {
    /// Create default budget for complexity level
    pub fn for_complexity(complexity: TaskComplexity) -> Self {
        match complexity {
            TaskComplexity::Simple => Self {
                max_reasoning_steps: 10,
                max_planning_depth: 2,
                token_budget: 2000,
                time_budget_ms: 30000, // 30 seconds
                cost_budget_cents: 10,
                allow_expansion: false,
                expansion_threshold: 0.0,
                max_expansion_multiplier: 1.0,
            },
            TaskComplexity::Moderate => Self {
                max_reasoning_steps: 25,
                max_planning_depth: 3,
                token_budget: 5000,
                time_budget_ms: 120000, // 2 minutes
                cost_budget_cents: 25,
                allow_expansion: true,
                expansion_threshold: 0.8,
                max_expansion_multiplier: 1.5,
            },
            TaskComplexity::Complex => Self {
                max_reasoning_steps: 50,
                max_planning_depth: 4,
                token_budget: 10000,
                time_budget_ms: 300000, // 5 minutes
                cost_budget_cents: 50,
                allow_expansion: true,
                expansion_threshold: 0.7,
                max_expansion_multiplier: 2.0,
            },
            TaskComplexity::VeryComplex => Self {
                max_reasoning_steps: 100,
                max_planning_depth: 5,
                token_budget: 20000,
                time_budget_ms: 600000, // 10 minutes
                cost_budget_cents: 100,
                allow_expansion: true,
                expansion_threshold: 0.6,
                max_expansion_multiplier: 3.0,
            },
        }
    }

    /// Check if budget can be expanded
    pub fn can_expand(&self, usage_percentage: f64) -> bool {
        self.allow_expansion && usage_percentage >= self.expansion_threshold
    }

    /// Expand budget by multiplier
    pub fn expand(&mut self, multiplier: f64) {
        let effective_multiplier = multiplier.min(self.max_expansion_multiplier);

        self.max_reasoning_steps = (self.max_reasoning_steps as f64 * effective_multiplier) as u32;
        self.max_planning_depth =
            (self.max_planning_depth as f64 * effective_multiplier.sqrt()) as u32;
        self.token_budget = (self.token_budget as f64 * effective_multiplier) as u32;
        self.time_budget_ms = (self.time_budget_ms as f64 * effective_multiplier) as u64;
        self.cost_budget_cents = (self.cost_budget_cents as f64 * effective_multiplier) as u32;

        info!(
            "Expanded thinking budget by {:.2}x: steps={}, depth={}, tokens={}, time={}ms",
            effective_multiplier,
            self.max_reasoning_steps,
            self.max_planning_depth,
            self.token_budget,
            self.time_budget_ms
        );
    }
}

/// Budget usage tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetUsage {
    pub task_id: Uuid,
    pub complexity: TaskComplexity,
    pub reasoning_steps_used: u32,
    pub planning_depth_used: u32,
    pub tokens_used: u32,
    pub time_used_ms: u64,
    pub cost_used_cents: u32,
    pub budget_exceeded: bool,
    pub budget_expanded: bool,
    pub success: bool,
    pub quality_score: f64,
    pub timestamp: DateTime<Utc>,
}

/// Thinking Budget Manager
pub struct ThinkingBudgetManager {
    /// Budget configurations by complexity
    budgets: Arc<RwLock<HashMap<TaskComplexity, ThinkingBudget>>>,

    /// Usage history for adaptive adjustment
    usage_history: Arc<RwLock<Vec<BudgetUsage>>>,

    /// Adaptive adjustment configuration
    adaptive_config: AdaptiveConfig,
}

/// Adaptive adjustment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveConfig {
    /// Enable adaptive adjustment
    pub enabled: bool,

    /// Minimum samples before adjustment
    pub min_samples: usize,

    /// Adjustment learning rate (0.0-1.0)
    pub learning_rate: f64,

    /// Success rate threshold for budget reduction
    pub success_rate_threshold: f64,

    /// Failure rate threshold for budget increase
    pub failure_rate_threshold: f64,
}

impl Default for AdaptiveConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_samples: 20,
            learning_rate: 0.1,
            success_rate_threshold: 0.9, // If 90%+ success, can reduce budget
            failure_rate_threshold: 0.3, // If <30% success, increase budget
        }
    }
}

impl ThinkingBudgetManager {
    /// Create new thinking budget manager
    pub fn new() -> Self {
        let mut budgets = HashMap::new();

        // Initialize budgets for each complexity level
        for complexity in [
            TaskComplexity::Simple,
            TaskComplexity::Moderate,
            TaskComplexity::Complex,
            TaskComplexity::VeryComplex,
        ] {
            budgets.insert(complexity, ThinkingBudget::for_complexity(complexity));
        }

        Self {
            budgets: Arc::new(RwLock::new(budgets)),
            usage_history: Arc::new(RwLock::new(Vec::new())),
            adaptive_config: AdaptiveConfig::default(),
        }
    }

    /// Get budget for a working spec
    pub async fn get_budget(&self, working_spec: &WorkingSpec) -> ThinkingBudget {
        let complexity = TaskComplexity::assess(working_spec);
        let budgets = self.budgets.read().await;
        budgets
            .get(&complexity)
            .cloned()
            .unwrap_or_else(|| ThinkingBudget::for_complexity(complexity))
    }

    /// Check if budget can be expanded and expand if needed
    pub async fn check_and_expand_budget(
        &self,
        task_id: Uuid,
        working_spec: &WorkingSpec,
        usage_percentage: f64,
    ) -> Result<bool> {
        let complexity = TaskComplexity::assess(working_spec);
        let mut budgets = self.budgets.write().await;

        if let Some(budget) = budgets.get_mut(&complexity) {
            if budget.can_expand(usage_percentage) {
                // Calculate expansion multiplier based on usage
                let expansion_multiplier = 1.0
                    + ((usage_percentage - budget.expansion_threshold)
                        / (1.0 - budget.expansion_threshold))
                        * (budget.max_expansion_multiplier - 1.0);

                budget.expand(expansion_multiplier);
                info!(
                    "Expanded budget for task {} (complexity: {:?})",
                    task_id, complexity
                );
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Record budget usage for adaptive adjustment
    pub async fn record_usage(
        &self,
        task_id: Uuid,
        working_spec: &WorkingSpec,
        reasoning_steps_used: u32,
        planning_depth_used: u32,
        tokens_used: u32,
        time_used_ms: u64,
        cost_used_cents: u32,
        budget_exceeded: bool,
        budget_expanded: bool,
        success: bool,
        quality_score: f64,
    ) {
        let complexity = TaskComplexity::assess(working_spec);

        let usage = BudgetUsage {
            task_id,
            complexity,
            reasoning_steps_used,
            planning_depth_used,
            tokens_used,
            time_used_ms,
            cost_used_cents,
            budget_exceeded,
            budget_expanded,
            success,
            quality_score,
            timestamp: Utc::now(),
        };

        let mut history = self.usage_history.write().await;
        history.push(usage);

        // Trim history to last 1000 entries
        if history.len() > 1000 {
            let excess = history.len() - 1000;
            history.drain(0..excess);
        }

        // Trigger adaptive adjustment if enabled
        if self.adaptive_config.enabled {
            if let Err(e) = self.adjust_budgets_for_complexity(complexity).await {
                warn!("Failed to adjust budgets: {}", e);
            }
        }
    }

    /// Adjust budgets based on usage history
    async fn adjust_budgets_for_complexity(&self, complexity: TaskComplexity) -> Result<()> {
        let history = self.usage_history.read().await;
        let complexity_history: Vec<&BudgetUsage> = history
            .iter()
            .filter(|u| u.complexity == complexity)
            .collect();

        if complexity_history.len() < self.adaptive_config.min_samples {
            return Ok(()); // Not enough samples
        }

        // Calculate success rate
        let success_count = complexity_history.iter().filter(|u| u.success).count();
        let success_rate = success_count as f64 / complexity_history.len() as f64;

        // Calculate average usage percentages
        let mut budgets = self.budgets.write().await;
        if let Some(budget) = budgets.get_mut(&complexity) {
            let avg_reasoning_usage = complexity_history
                .iter()
                .map(|u| u.reasoning_steps_used as f64 / budget.max_reasoning_steps as f64)
                .sum::<f64>()
                / complexity_history.len() as f64;

            let avg_time_usage = complexity_history
                .iter()
                .map(|u| u.time_used_ms as f64 / budget.time_budget_ms as f64)
                .sum::<f64>()
                / complexity_history.len() as f64;

            let avg_token_usage = complexity_history
                .iter()
                .map(|u| u.tokens_used as f64 / budget.token_budget as f64)
                .sum::<f64>()
                / complexity_history.len() as f64;

            // Adjust based on success rate and usage patterns
            if success_rate >= self.adaptive_config.success_rate_threshold {
                // High success rate - can optimize (reduce) budget
                let reduction_factor = 1.0
                    - (self.adaptive_config.learning_rate
                        * (success_rate - self.adaptive_config.success_rate_threshold));

                // Only reduce if usage is consistently low
                if avg_reasoning_usage < 0.7 && avg_time_usage < 0.7 && avg_token_usage < 0.7 {
                    budget.max_reasoning_steps =
                        ((budget.max_reasoning_steps as f64) * reduction_factor) as u32;
                    budget.time_budget_ms =
                        ((budget.time_budget_ms as f64) * reduction_factor) as u64;
                    budget.token_budget = ((budget.token_budget as f64) * reduction_factor) as u32;

                    info!(
                        "Optimized budget for {:?}: success_rate={:.2}, reduced by {:.2}%",
                        complexity,
                        success_rate,
                        (1.0 - reduction_factor) * 100.0
                    );
                }
            } else if success_rate < self.adaptive_config.failure_rate_threshold {
                // Low success rate - increase budget
                let increase_factor = 1.0
                    + (self.adaptive_config.learning_rate
                        * (self.adaptive_config.failure_rate_threshold - success_rate));

                // Cap increase at 2x
                let effective_increase = increase_factor.min(2.0);

                budget.max_reasoning_steps =
                    ((budget.max_reasoning_steps as f64) * effective_increase) as u32;
                budget.time_budget_ms =
                    ((budget.time_budget_ms as f64) * effective_increase) as u64;
                budget.token_budget = ((budget.token_budget as f64) * effective_increase) as u32;

                info!(
                    "Increased budget for {:?}: success_rate={:.2}, increased by {:.2}%",
                    complexity,
                    success_rate,
                    (effective_increase - 1.0) * 100.0
                );
            }

            // Adjust expansion thresholds based on usage patterns
            if avg_reasoning_usage > 0.9 || avg_time_usage > 0.9 {
                // High usage - lower expansion threshold
                budget.expansion_threshold = (budget.expansion_threshold - 0.1).max(0.5);
            } else if avg_reasoning_usage < 0.6 && avg_time_usage < 0.6 {
                // Low usage - raise expansion threshold
                budget.expansion_threshold = (budget.expansion_threshold + 0.1).min(0.9);
            }
        }

        Ok(())
    }

    /// Get usage statistics for a complexity level
    pub async fn get_usage_stats(&self, complexity: TaskComplexity) -> Option<UsageStats> {
        let history = self.usage_history.read().await;
        let complexity_history: Vec<&BudgetUsage> = history
            .iter()
            .filter(|u| u.complexity == complexity)
            .collect();

        if complexity_history.is_empty() {
            return None;
        }

        let success_rate = complexity_history.iter().filter(|u| u.success).count() as f64
            / complexity_history.len() as f64;

        let avg_reasoning_usage = complexity_history
            .iter()
            .map(|u| u.reasoning_steps_used)
            .sum::<u32>() as f64
            / complexity_history.len() as f64;

        let avg_time_usage = complexity_history
            .iter()
            .map(|u| u.time_used_ms)
            .sum::<u64>() as f64
            / complexity_history.len() as f64;

        let avg_token_usage = complexity_history
            .iter()
            .map(|u| u.tokens_used)
            .sum::<u32>() as f64
            / complexity_history.len() as f64;

        let budget_exceeded_rate = complexity_history
            .iter()
            .filter(|u| u.budget_exceeded)
            .count() as f64
            / complexity_history.len() as f64;

        Some(UsageStats {
            complexity,
            total_tasks: complexity_history.len(),
            success_rate,
            average_reasoning_steps: avg_reasoning_usage,
            average_time_ms: avg_time_usage,
            average_tokens: avg_token_usage,
            budget_exceeded_rate,
        })
    }

    /// Get current budget configuration
    pub async fn get_budget_config(&self, complexity: TaskComplexity) -> Option<ThinkingBudget> {
        let budgets = self.budgets.read().await;
        budgets.get(&complexity).cloned()
    }
}

/// Usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    pub complexity: TaskComplexity,
    pub total_tasks: usize,
    pub success_rate: f64,
    pub average_reasoning_steps: f64,
    pub average_time_ms: f64,
    pub average_tokens: f64,
    pub budget_exceeded_rate: f64,
}

impl Default for ThinkingBudgetManager {
    fn default() -> Self {
        Self::new()
    }
}
