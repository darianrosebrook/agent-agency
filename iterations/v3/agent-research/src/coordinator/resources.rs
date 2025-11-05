//! Resource monitoring and heuristics
//!
//! Resource utilization tracking, efficiency analysis, and
//! resource optimization for learning coordination.

use schemars::JsonSchema;
use std::collections::HashMap;

/// Resource utilization metrics

use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize) ]
pub struct ResourceMetrics {
    pub cpu_seconds: f64,
    pub memory_bytes: u64,
    pub tokens_used: u64,
    pub execution_time_ms: u64,
}

/// Resource utilization levels

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize) ]
pub enum ResourceLevel {
    Low,
    Moderate,
    High,
    Critical,
}

/// Heuristic mapping for resource utilization

#[derive(Debug, Clone, Serialize, Deserialize) ]
pub struct ResourceHeuristics {
    pub cpu_thresholds: ResourceThresholds,
    pub memory_thresholds: ResourceThresholds,
    pub token_thresholds: ResourceThresholds,
    pub efficiency_weights: EfficiencyWeights,
}

impl ResourceHeuristics {
    /// Create default resource heuristics
    pub fn new() -> Self {
        Self {
            cpu_thresholds: ResourceThresholds {
                low_max: 15.0,
                moderate_max: 30.0,
                high_max: 45.0,
                critical_max: 60.0,
            },
            memory_thresholds: ResourceThresholds {
                low_max: 5_000,
                moderate_max: 10_000,
                high_max: 14_000,
                critical_max: 20_000,
            },
            token_thresholds: ResourceThresholds {
                low_max: 5_000,
                moderate_max: 10_000,
                high_max: 15_000,
                critical_max: 20_000,
            },
            efficiency_weights: EfficiencyWeights::default(),
        }
    }

    /// Classify CPU usage level
    pub fn classify_cpu_usage(&self, cpu_seconds: f64) -> ResourceLevel {
        self.classify_by_thresholds(cpu_seconds, &self.cpu_thresholds)
    }

    /// Classify memory usage level
    pub fn classify_memory_usage(&self, memory_bytes: u64) -> ResourceLevel {
        self.classify_by_thresholds(memory_bytes as f64, &self.memory_thresholds)
    }

    /// Classify token usage level
    pub fn classify_token_usage(&self, tokens: u64) -> ResourceLevel {
        self.classify_by_thresholds(tokens as f64, &self.token_thresholds)
    }

    /// Calculate resource efficiency score
    pub fn calculate_efficiency(&self, metrics: &ResourceMetrics) -> f64 {
        let cpu_efficiency = 1.0 / (1.0 + metrics.cpu_seconds / 60.0); // Normalize CPU usage
        let memory_efficiency = 1.0 / (1.0 + metrics.memory_bytes as f64 / 16_000.0); // Normalize memory
        let token_efficiency = 1.0 / (1.0 + metrics.tokens_used as f64 / 20_000.0); // Normalize tokens
        let time_efficiency = 1.0 / (1.0 + metrics.execution_time_ms as f64 / 60_000.0); // Normalize time

        let weights = &self.efficiency_weights;
        (cpu_efficiency * weights.cpu_efficiency +
         memory_efficiency * weights.memory_efficiency +
         token_efficiency * weights.token_efficiency +
         time_efficiency * weights.time_efficiency) /
        (weights.cpu_efficiency + weights.memory_efficiency + weights.token_efficiency + weights.time_efficiency)
    }

    /// Check if resources are within acceptable bounds
    pub fn check_resource_bounds(&self, metrics: &ResourceMetrics) -> ResourceStatus {
        let cpu_level = self.classify_cpu_usage(metrics.cpu_seconds);
        let memory_level = self.classify_memory_usage(metrics.memory_bytes);
        let token_level = self.classify_token_usage(metrics.tokens_used);

        let efficiency = self.calculate_efficiency(metrics);

        ResourceStatus {
            cpu_level,
            memory_level,
            token_level,
            overall_efficiency: efficiency,
            within_bounds: matches!(cpu_level, ResourceLevel::Low | ResourceLevel::Moderate) &&
                         matches!(memory_level, ResourceLevel::Low | ResourceLevel::Moderate) &&
                         matches!(token_level, ResourceLevel::Low | ResourceLevel::Moderate) &&
                         efficiency >= 0.75, // EFFICIENCY_SUCCESS_THRESHOLD
        }
    }

    fn classify_by_thresholds(&self, value: f64, thresholds: &ResourceThresholds) -> ResourceLevel {
        if value <= thresholds.low_max {
            ResourceLevel::Low
        } else if value <= thresholds.moderate_max {
            ResourceLevel::Moderate
        } else if value <= thresholds.high_max {
            ResourceLevel::High
        } else {
            ResourceLevel::Critical
        }
    }
}

/// Resource usage thresholds

#[derive(Debug, Clone, Serialize, Deserialize) ]
pub struct ResourceThresholds {
    pub low_max: f64,
    pub moderate_max: f64,
    pub high_max: f64,
    pub critical_max: f64,
}

/// Efficiency calculation weights

#[derive(Debug, Clone, Serialize, Deserialize) ]
pub struct EfficiencyWeights {
    pub cpu_efficiency: f64,
    pub memory_efficiency: f64,
    pub token_efficiency: f64,
    pub time_efficiency: f64,
}

impl Default for EfficiencyWeights {
    fn default() -> Self {
        Self {
            cpu_efficiency: 0.25,
            memory_efficiency: 0.25,
            token_efficiency: 0.25,
            time_efficiency: 0.25,
        }
    }
}

/// Resource status assessment

#[derive(Debug, Clone, Serialize, Deserialize) ]
pub struct ResourceStatus {
    pub cpu_level: ResourceLevel,
    pub memory_level: ResourceLevel,
    pub token_level: ResourceLevel,
    pub overall_efficiency: f64,
    pub within_bounds: bool,
}

impl ResourceStatus {
    /// Get resource optimization recommendations
    pub fn get_optimization_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();

        if matches!(self.cpu_level, ResourceLevel::High | ResourceLevel::Critical) {
            recommendations.push("Reduce CPU-intensive operations or optimize algorithms".to_string());
        }

        if matches!(self.memory_level, ResourceLevel::High | ResourceLevel::Critical) {
            recommendations.push("Implement memory optimization or reduce concurrent operations".to_string());
        }

        if matches!(self.token_level, ResourceLevel::High | ResourceLevel::Critical) {
            recommendations.push("Optimize token usage or implement token budgeting".to_string());
        }

        if self.overall_efficiency < 0.75 {
            recommendations.push("Review and optimize resource utilization patterns".to_string());
        }

        recommendations
    }

    /// Check if resource usage indicates potential issues
    pub fn has_resource_warnings(&self) -> bool {
        matches!(self.cpu_level, ResourceLevel::High | ResourceLevel::Critical) ||
        matches!(self.memory_level, ResourceLevel::High | ResourceLevel::Critical) ||
        matches!(self.token_level, ResourceLevel::High | ResourceLevel::Critical) ||
        self.overall_efficiency < 0.8
    }
}

/// Resource usage trend analysis

#[derive(Debug, Clone, Serialize, Deserialize) ]
pub struct ResourceTrend {
    pub metric: ResourceMetric,
    pub trend: Trend,
    pub change_percentage: f64,
    pub period_seconds: u64,
}


#[derive(Debug, Clone, Serialize, Deserialize) ]
pub enum ResourceMetric {
    CpuUsage,
    MemoryUsage,
    TokenUsage,
    Efficiency,
}


#[derive(Debug, Clone, Serialize, Deserialize) ]
pub enum Trend {
    Improving,
    Stable,
    Degrading,
}


