//! Context Budget Management
//!
//! Manages token allocation across working, episodic, and semantic memories
//! using knapsack optimization for maximum utility within budget constraints.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::types::Task;

/// Context budget configuration
#[derive(Clone, Debug)]
pub struct ContextBudget {
    pub max_tokens: usize,
    pub headroom: f32, // keep slack for safety
    pub memory_weights: MemoryWeights,
    pub enable_compression: bool,
}

impl Default for ContextBudget {
    fn default() -> Self {
        Self {
            max_tokens: 8192,
            headroom: 0.2, // 20% headroom
            memory_weights: MemoryWeights::default(),
            enable_compression: true,
        }
    }
}

impl ContextBudget {
    pub fn conservative() -> Self {
        Self {
            max_tokens: 4096,
            headroom: 0.3,
            memory_weights: MemoryWeights::conservative(),
            enable_compression: true,
        }
    }

    pub fn aggressive() -> Self {
        Self {
            max_tokens: 16384,
            headroom: 0.1,
            memory_weights: MemoryWeights::aggressive(),
            enable_compression: false,
        }
    }

    /// Get effective token capacity after headroom
    pub fn effective_capacity(&self) -> usize {
        (self.max_tokens as f32 * (1.0 - self.headroom)).round() as usize
    }

    /// Check if allocation fits budget
    pub fn can_fit(&self, allocation: &Allocation) -> bool {
        let total = allocation.working + allocation.episodic + allocation.semantic;
        total <= self.effective_capacity()
    }
}

/// Memory type weights for utility calculation
#[derive(Clone, Debug)]
pub struct MemoryWeights {
    pub working: f64,
    pub episodic: f64,
    pub semantic: f64,
    pub citations: f64,
}

impl Default for MemoryWeights {
    fn default() -> Self {
        Self {
            working: 1.0,    // High weight for current task context
            episodic: 0.8,   // Good weight for historical patterns
            semantic: 0.6,   // Moderate weight for general knowledge
            citations: 0.2,  // Low weight for attribution
        }
    }
}

impl MemoryWeights {
    pub fn conservative() -> Self {
        Self {
            working: 1.2,    // Favor working memory more
            episodic: 0.6,
            semantic: 0.4,
            citations: 0.1,
        }
    }

    pub fn aggressive() -> Self {
        Self {
            working: 0.8,    // Allow more diverse context
            episodic: 1.0,
            semantic: 0.8,
            citations: 0.3,
        }
    }

    pub fn task_specific(task_type: &str) -> Self {
        match task_type {
            "code" => Self {
                working: 1.0,
                episodic: 0.9,   // Code patterns are valuable
                semantic: 0.5,   // General programming knowledge
                citations: 0.2,
            },
            "research" => Self {
                working: 0.8,
                episodic: 0.6,
                semantic: 1.0,   // Research needs broad knowledge
                citations: 0.4,  // Citations important for research
            },
            "planning" => Self {
                working: 1.0,
                episodic: 1.0,   // Historical planning patterns
                semantic: 0.7,
                citations: 0.3,
            },
            _ => Self::default(),
        }
    }
}

/// Token allocation across memory types
#[derive(Clone, Debug, PartialEq)]
pub struct Allocation {
    pub working: usize,
    pub episodic: usize,
    pub semantic: usize,
    pub citations: usize,
}

impl Allocation {
    pub fn total_tokens(&self) -> usize {
        self.working + self.episodic + self.semantic + self.citations
    }

    pub fn is_empty(&self) -> bool {
        self.total_tokens() == 0
    }

    pub fn scale_to_budget(&self, budget: &ContextBudget) -> Allocation {
        let capacity = budget.effective_capacity();
        let total = self.total_tokens();

        if total <= capacity {
            return self.clone();
        }

        let scale = capacity as f64 / total as f64;
        Allocation {
            working: (self.working as f64 * scale).round() as usize,
            episodic: (self.episodic as f64 * scale).round() as usize,
            semantic: (self.semantic as f64 * scale).round() as usize,
            citations: (self.citations as f64 * scale).round() as usize,
        }
    }
}

/// Context utility estimates
#[derive(Clone, Debug)]
pub struct ContextUtility {
    pub working: f64,
    pub episodic: f64,
    pub semantic: f64,
    pub citations: f64,
}

impl ContextUtility {
    pub fn zero() -> Self {
        Self {
            working: 0.0,
            episodic: 0.0,
            semantic: 0.0,
            citations: 0.0,
        }
    }

    pub fn total(&self) -> f64 {
        self.working + self.episodic + self.semantic + self.citations
    }

    pub fn normalize(&self) -> Self {
        let total = self.total();
        if total == 0.0 {
            return Self::zero();
        }

        Self {
            working: self.working / total,
            episodic: self.episodic / total,
            semantic: self.semantic / total,
            citations: self.citations / total,
        }
    }
}

/// Token usage estimates
#[derive(Clone, Debug)]
pub struct TokenEstimates {
    pub working: usize,
    pub episodic: usize,
    pub semantic: usize,
    pub citations: usize,
}

impl TokenEstimates {
    pub fn total(&self) -> usize {
        self.working + self.episodic + self.semantic + self.citations
    }

    pub fn exceeds_budget(&self, budget: &ContextBudget) -> bool {
        self.total() > budget.effective_capacity()
    }
}

/// Context allocator using knapsack optimization
pub struct ContextAllocator {
    optimization_strategy: OptimizationStrategy,
}

impl Default for ContextAllocator {
    fn default() -> Self {
        Self {
            optimization_strategy: OptimizationStrategy::Proportional,
        }
    }
}

impl ContextAllocator {
    pub fn new(strategy: OptimizationStrategy) -> Self {
        Self {
            optimization_strategy: strategy,
        }
    }

    /// Allocate tokens across memories for maximum utility
    pub fn allocate(&self, budget: &ContextBudget, utility: &ContextUtility, estimates: &TokenEstimates) -> Allocation {
        info!("Allocating context budget: {} tokens (effective: {})",
              budget.max_tokens, budget.effective_capacity());

        let allocation = match self.optimization_strategy {
            OptimizationStrategy::Proportional => self.allocate_proportional(budget, utility),
            OptimizationStrategy::Knapsack => self.allocate_knapsack(budget, utility, estimates),
            OptimizationStrategy::Greedy => self.allocate_greedy(budget, utility),
        };

        // Ensure allocation fits budget
        let scaled = allocation.scale_to_budget(budget);

        debug!("Final allocation: working={}, episodic={}, semantic={}, citations={}",
               scaled.working, scaled.episodic, scaled.semantic, scaled.citations);

        scaled
    }

    /// Simple proportional allocation
    fn allocate_proportional(&self, budget: &ContextBudget, utility: &ContextUtility) -> Allocation {
        let capacity = budget.effective_capacity();
        let normalized = utility.normalize();

        Allocation {
            working: (normalized.working * capacity as f64).round() as usize,
            episodic: (normalized.episodic * capacity as f64).round() as usize,
            semantic: (normalized.semantic * capacity as f64).round() as usize,
            citations: (normalized.citations * capacity as f64).round() as usize,
        }
    }

    /// Knapsack optimization for maximum utility
    fn allocate_knapsack(&self, budget: &ContextBudget, utility: &ContextUtility, estimates: &TokenEstimates) -> Allocation {
        // 0/1 Knapsack: maximize utility with token constraints
        let capacity = budget.effective_capacity();

        // Items: (utility_per_token, tokens, memory_type)
        let items = vec![
            (utility.working / estimates.working.max(1) as f64, estimates.working, MemoryType::Working),
            (utility.episodic / estimates.episodic.max(1) as f64, estimates.episodic, MemoryType::Episodic),
            (utility.semantic / estimates.semantic.max(1) as f64, estimates.semantic, MemoryType::Semantic),
            (utility.citations / estimates.citations.max(1) as f64, estimates.citations, MemoryType::Citations),
        ];

        // DP table: dp[i][w] = max utility using first i items with weight <= w
        let n = items.len();
        let mut dp = vec![vec![0.0; capacity + 1]; n + 1];

        // Track which items are selected
        let mut selected = vec![false; n];

        for i in 1..=n {
            let (utility_per_token, tokens, _) = items[i - 1];
            for w in 0..=capacity {
                if tokens <= w {
                    let include_utility = dp[i - 1][w - tokens] + utility_per_token * tokens as f64;
                    let exclude_utility = dp[i - 1][w];

                    if include_utility > exclude_utility {
                        dp[i][w] = include_utility;
                        if w == capacity || i == n {
                            selected[i - 1] = true;
                        }
                    } else {
                        dp[i][w] = exclude_utility;
                    }
                } else {
                    dp[i][w] = dp[i - 1][w];
                }
            }
        }

        // Extract allocation from selected items
        let mut allocation = Allocation {
            working: 0,
            episodic: 0,
            semantic: 0,
            citations: 0,
        };

        for (i, &is_selected) in selected.iter().enumerate() {
            if is_selected {
                let (_, tokens, mem_type) = items[i];
                match mem_type {
                    MemoryType::Working => allocation.working = tokens,
                    MemoryType::Episodic => allocation.episodic = tokens,
                    MemoryType::Semantic => allocation.semantic = tokens,
                    MemoryType::Citations => allocation.citations = tokens,
                }
            }
        }

        allocation
    }

    /// Greedy allocation by utility density
    fn allocate_greedy(&self, budget: &ContextBudget, utility: &ContextUtility) -> Allocation {
        let capacity = budget.effective_capacity();

        // Sort by utility density (utility per token)
        let mut items = vec![
            (utility.working, 1.0, MemoryType::Working), // Assume 1 token for density calc
            (utility.episodic, 1.0, MemoryType::Episodic),
            (utility.semantic, 1.0, MemoryType::Semantic),
            (utility.citations, 1.0, MemoryType::Citations),
        ];

        items.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        let mut allocation = Allocation {
            working: 0,
            episodic: 0,
            semantic: 0,
            citations: 0,
        };

        let mut remaining = capacity;

        for (utility_score, _, mem_type) in items {
            if utility_score <= 0.0 || remaining == 0 {
                break;
            }

            let allocation_amount = (remaining as f64 * utility_score).round() as usize;
            let actual_amount = allocation_amount.min(remaining);

            match mem_type {
                MemoryType::Working => allocation.working = actual_amount,
                MemoryType::Episodic => allocation.episodic = actual_amount,
                MemoryType::Semantic => allocation.semantic = actual_amount,
                MemoryType::Citations => allocation.citations = actual_amount,
            }

            remaining -= actual_amount;
        }

        allocation
    }
}

/// Optimization strategies
#[derive(Clone, Debug)]
pub enum OptimizationStrategy {
    Proportional,  // Simple proportional allocation
    Knapsack,      // 0/1 Knapsack optimization
    Greedy,        // Greedy by utility density
}

/// Memory types for allocation
#[derive(Clone, Debug)]
enum MemoryType {
    Working,
    Episodic,
    Semantic,
    Citations,
}

/// Budget tracker for monitoring usage
pub struct BudgetTracker {
    budgets: Arc<RwLock<HashMap<String, ContextBudget>>>,
    usage_stats: Arc<RwLock<HashMap<String, Allocation>>>,
}

impl BudgetTracker {
    pub fn new() -> Self {
        Self {
            budgets: Arc::new(RwLock::new(HashMap::new())),
            usage_stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn set_budget(&self, task_id: String, budget: ContextBudget) {
        let mut budgets = self.budgets.write().await;
        budgets.insert(task_id, budget);
    }

    pub async fn record_usage(&self, task_id: String, allocation: Allocation) {
        let mut usage = self.usage_stats.write().await;
        usage.insert(task_id, allocation);
    }

    pub async fn get_budget(&self, task_id: &str) -> Option<ContextBudget> {
        let budgets = self.budgets.read().await;
        budgets.get(task_id).cloned()
    }

    pub async fn get_usage_stats(&self) -> HashMap<String, Allocation> {
        let usage = self.usage_stats.read().await;
        usage.clone()
    }

    pub async fn get_efficiency_report(&self) -> BudgetEfficiencyReport {
        let budgets = self.budgets.read().await;
        let usage = self.usage_stats.read().await;

        let mut total_budget = 0;
        let mut total_used = 0;
        let mut task_count = 0;

        for (task_id, budget) in budgets.iter() {
            if let Some(allocation) = usage.get(task_id) {
                total_budget += budget.effective_capacity();
                total_used += allocation.total_tokens();
                task_count += 1;
            }
        }

        let efficiency = if total_budget > 0 {
            total_used as f64 / total_budget as f64
        } else {
            0.0
        };

        BudgetEfficiencyReport {
            total_budget_tokens: total_budget,
            total_used_tokens: total_used,
            average_efficiency: efficiency,
            task_count,
        }
    }
}

/// Budget efficiency report
#[derive(Clone, Debug)]
pub struct BudgetEfficiencyReport {
    pub total_budget_tokens: usize,
    pub total_used_tokens: usize,
    pub average_efficiency: f64,
    pub task_count: usize,
}

/// Dynamic budget adjustment based on task performance
pub struct AdaptiveBudgetAdjuster {
    performance_history: Arc<RwLock<Vec<TaskPerformance>>>,
    adjustment_strategy: AdjustmentStrategy,
}

#[derive(Clone, Debug)]
pub struct TaskPerformance {
    pub task_id: String,
    pub final_score: f64,
    pub context_used: usize,
    pub context_budget: usize,
    pub compression_applied: bool,
}

#[derive(Clone, Debug)]
pub enum AdjustmentStrategy {
    Conservative,  // Small adjustments based on clear patterns
    Aggressive,    // Larger adjustments for optimization
    Adaptive,      // Learn from performance patterns
}

impl AdaptiveBudgetAdjuster {
    pub fn new(strategy: AdjustmentStrategy) -> Self {
        Self {
            performance_history: Arc::new(RwLock::new(Vec::new())),
            adjustment_strategy: strategy,
        }
    }

    pub async fn record_performance(&self, performance: TaskPerformance) {
        let mut history = self.performance_history.write().await;
        history.push(performance);

        // Keep only recent history
        if history.len() > 100 {
            history.remove(0);
        }
    }

    pub async fn suggest_budget_adjustment(&self, task_type: &str, current_budget: &ContextBudget) -> ContextBudget {
        let history = self.performance_history.read().await;

        // Filter by task type
        let relevant_tasks: Vec<&TaskPerformance> = history.iter()
            .filter(|p| p.task_id.contains(task_type))
            .collect();

        if relevant_tasks.is_empty() {
            return current_budget.clone();
        }

        let suggestion = match self.adjustment_strategy {
            AdjustmentStrategy::Conservative => self.conservative_adjustment(&relevant_tasks, current_budget),
            AdjustmentStrategy::Aggressive => self.aggressive_adjustment(&relevant_tasks, current_budget),
            AdjustmentStrategy::Adaptive => self.adaptive_adjustment(&relevant_tasks, current_budget),
        };

        info!("Suggested budget adjustment for {}: {} -> {} tokens",
              task_type, current_budget.max_tokens, suggestion.max_tokens);

        suggestion
    }

    fn conservative_adjustment(&self, tasks: &[&TaskPerformance], current: &ContextBudget) -> ContextBudget {
        let avg_score: f64 = tasks.iter().map(|t| t.final_score).sum::<f64>() / tasks.len() as f64;
        let avg_usage_ratio: f64 = tasks.iter()
            .map(|t| t.context_used as f64 / t.context_budget as f64)
            .sum::<f64>() / tasks.len() as f64;

        let mut adjusted = current.clone();

        // Small adjustments based on performance
        if avg_score > 0.8 && avg_usage_ratio < 0.7 {
            // High performance with low usage - could reduce budget
            adjusted.max_tokens = (adjusted.max_tokens as f64 * 0.95).round() as usize;
        } else if avg_score < 0.6 && avg_usage_ratio > 0.9 {
            // Low performance with high usage - could increase budget
            adjusted.max_tokens = (adjusted.max_tokens as f64 * 1.05).round() as usize;
        }

        adjusted
    }

    fn aggressive_adjustment(&self, tasks: &[&TaskPerformance], current: &ContextBudget) -> ContextBudget {
        let avg_score: f64 = tasks.iter().map(|t| t.final_score).sum::<f64>() / tasks.len() as f64;
        let avg_usage_ratio: f64 = tasks.iter()
            .map(|t| t.context_used as f64 / t.context_budget as f64)
            .sum::<f64>() / tasks.len() as f64;

        let mut adjusted = current.clone();

        if avg_score > 0.8 && avg_usage_ratio < 0.6 {
            // Very efficient - reduce budget significantly
            adjusted.max_tokens = (adjusted.max_tokens as f64 * 0.8).round() as usize;
        } else if avg_score < 0.5 && avg_usage_ratio > 0.8 {
            // Struggling - increase budget significantly
            adjusted.max_tokens = (adjusted.max_tokens as f64 * 1.2).round() as usize;
        }

        adjusted
    }

    fn adaptive_adjustment(&self, tasks: &[&TaskPerformance], current: &ContextBudget) -> ContextBudget {
        // More sophisticated: use regression to predict optimal budget
        // For now, fall back to conservative
        self.conservative_adjustment(tasks, current)
    }
}
