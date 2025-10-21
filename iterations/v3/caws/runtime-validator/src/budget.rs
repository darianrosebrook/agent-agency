//! CAWS Budget Management
//!
//! Consolidated budget checking logic extracted from self-prompting-agent
//! and other implementations.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Budget limits for different resource types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetLimits {
    pub max_files: u32,
    pub max_loc: u32,
    pub max_time_seconds: u64,
    pub max_memory_mb: u64,
    pub max_cost_cents: Option<u64>,
}

/// Current budget state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetState {
    pub files_used: u32,
    pub loc_used: u32,
    pub time_used_seconds: u64,
    pub memory_used_mb: u64,
    pub cost_used_cents: u64,
    pub last_updated: DateTime<Utc>,
}

/// Budget checking result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetCheckResult {
    pub within_limits: bool,
    pub violations: Vec<BudgetViolation>,
    pub utilization_percentage: HashMap<String, f32>,
}

/// Budget violation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetViolation {
    pub resource_type: String,
    pub current_usage: u64,
    pub limit: u64,
    pub percentage_over: f32,
}

/// Budget checker - consolidated from multiple implementations
pub struct BudgetChecker {
    limits: BudgetLimits,
}

impl BudgetChecker {
    pub fn new(limits: BudgetLimits) -> Self {
        Self { limits }
    }

    /// Check if current state is within budget limits
    pub fn check_budget(&self, state: &BudgetState) -> BudgetCheckResult {
        let mut violations = Vec::new();
        let mut utilization = HashMap::new();
        let mut within_limits = true;

        // Check files
        let files_pct = if self.limits.max_files > 0 {
            (state.files_used as f32 / self.limits.max_files as f32) * 100.0
        } else {
            0.0
        };
        utilization.insert("files".to_string(), files_pct);

        if state.files_used > self.limits.max_files {
            within_limits = false;
            violations.push(BudgetViolation {
                resource_type: "files".to_string(),
                current_usage: state.files_used as u64,
                limit: self.limits.max_files as u64,
                percentage_over: ((state.files_used - self.limits.max_files) as f32 / self.limits.max_files as f32) * 100.0,
            });
        }

        // Check lines of code
        let loc_pct = if self.limits.max_loc > 0 {
            (state.loc_used as f32 / self.limits.max_loc as f32) * 100.0
        } else {
            0.0
        };
        utilization.insert("loc".to_string(), loc_pct);

        if state.loc_used > self.limits.max_loc {
            within_limits = false;
            violations.push(BudgetViolation {
                resource_type: "loc".to_string(),
                current_usage: state.loc_used as u64,
                limit: self.limits.max_loc as u64,
                percentage_over: ((state.loc_used - self.limits.max_loc) as f32 / self.limits.max_loc as f32) * 100.0,
            });
        }

        // Check time
        let time_pct = (state.time_used_seconds as f32 / self.limits.max_time_seconds as f32) * 100.0;
        utilization.insert("time".to_string(), time_pct);

        if state.time_used_seconds > self.limits.max_time_seconds {
            within_limits = false;
            violations.push(BudgetViolation {
                resource_type: "time".to_string(),
                current_usage: state.time_used_seconds,
                limit: self.limits.max_time_seconds,
                percentage_over: ((state.time_used_seconds - self.limits.max_time_seconds) as f32 / self.limits.max_time_seconds as f32) * 100.0,
            });
        }

        // Check memory
        let memory_pct = (state.memory_used_mb as f32 / self.limits.max_memory_mb as f32) * 100.0;
        utilization.insert("memory".to_string(), memory_pct);

        if state.memory_used_mb > self.limits.max_memory_mb {
            within_limits = false;
            violations.push(BudgetViolation {
                resource_type: "memory".to_string(),
                current_usage: state.memory_used_mb as u64,
                limit: self.limits.max_memory_mb as u64,
                percentage_over: ((state.memory_used_mb - self.limits.max_memory_mb) as f32 / self.limits.max_memory_mb as f32) * 100.0,
            });
        }

        // Check cost if applicable
        if let Some(max_cost) = self.limits.max_cost_cents {
            let cost_pct = (state.cost_used_cents as f32 / max_cost as f32) * 100.0;
            utilization.insert("cost".to_string(), cost_pct);

            if state.cost_used_cents > max_cost {
                within_limits = false;
                violations.push(BudgetViolation {
                    resource_type: "cost".to_string(),
                    current_usage: state.cost_used_cents,
                    limit: max_cost,
                    percentage_over: ((state.cost_used_cents - max_cost) as f32 / max_cost as f32) * 100.0,
                });
            }
        }

        BudgetCheckResult {
            within_limits,
            violations,
            utilization_percentage: utilization,
        }
    }

    /// Update budget state with new usage
    pub fn update_state(&self, mut state: BudgetState, new_files: u32, new_loc: u32,
                       new_time: u64, new_memory: u64, new_cost: u64) -> BudgetState {
        state.files_used += new_files;
        state.loc_used += new_loc;
        state.time_used_seconds += new_time;
        state.memory_used_mb += new_memory;
        state.cost_used_cents += new_cost;
        state.last_updated = Utc::now();
        state
    }

    /// Get warning thresholds (80% utilization)
    pub fn get_warning_thresholds(&self) -> HashMap<String, u64> {
        let mut thresholds = HashMap::new();
        thresholds.insert("files".to_string(), (self.limits.max_files as f32 * 0.8) as u64);
        thresholds.insert("loc".to_string(), (self.limits.max_loc as f32 * 0.8) as u64);
        thresholds.insert("time".to_string(), (self.limits.max_time_seconds as f32 * 0.8) as u64);
        thresholds.insert("memory".to_string(), (self.limits.max_memory_mb as f32 * 0.8) as u64);

        if let Some(cost) = self.limits.max_cost_cents {
            thresholds.insert("cost".to_string(), (cost as f32 * 0.8) as u64);
        }

        thresholds
    }

    /// Check if state is approaching limits (warning zone)
    pub fn check_warnings(&self, state: &BudgetState) -> Vec<String> {
        let thresholds = self.get_warning_thresholds();
        let mut warnings = Vec::new();

        if let Some(threshold) = thresholds.get("files") {
            if u64::from(state.files_used) >= *threshold {
                warnings.push(format!("Files usage at {}% of budget", (state.files_used as f32 / self.limits.max_files as f32 * 100.0) as u32));
            }
        }

        if let Some(threshold) = thresholds.get("loc") {
            if u64::from(state.loc_used) >= *threshold {
                warnings.push(format!("LOC usage at {}% of budget", (state.loc_used as f32 / self.limits.max_loc as f32 * 100.0) as u32));
            }
        }

        if let Some(threshold) = thresholds.get("time") {
            if state.time_used_seconds >= *threshold {
                warnings.push(format!("Time usage at {}% of budget", (state.time_used_seconds as f32 / self.limits.max_time_seconds as f32 * 100.0) as u32));
            }
        }

        if let Some(threshold) = thresholds.get("memory") {
            if u64::from(state.memory_used_mb) >= *threshold {
                warnings.push(format!("Memory usage at {}% of budget", (state.memory_used_mb as f32 / self.limits.max_memory_mb as f32 * 100.0) as u32));
            }
        }

        if let Some(threshold) = thresholds.get("cost") {
            if state.cost_used_cents >= *threshold {
                warnings.push(format!("Cost usage at {}% of budget", (state.cost_used_cents as f32 / self.limits.max_cost_cents.unwrap_or(1) as f32 * 100.0) as u32));
            }
        }

        warnings
    }
}

impl Default for BudgetLimits {
    fn default() -> Self {
        Self {
            max_files: 25,
            max_loc: 1000,
            max_time_seconds: 600,
            max_memory_mb: 1024,
            max_cost_cents: None,
        }
    }
}

impl Default for BudgetState {
    fn default() -> Self {
        Self {
            files_used: 0,
            loc_used: 0,
            time_used_seconds: 0,
            memory_used_mb: 0,
            cost_used_cents: 0,
            last_updated: Utc::now(),
        }
    }
}
