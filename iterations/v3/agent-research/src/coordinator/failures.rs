//! Failure analysis and patterns
//!
//! Failure detection, analysis, categorization, and recovery
//! strategies for learning coordination and error handling.

use schemars::JsonSchema;
use std::collections::HashMap;

/// Failure categories for classification

use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum FailureCategory {
    Timeout,
    ResourceExhaustion,
    QualityDegradation,
    ConsensusFailure,
    EvidenceInsufficient,
    RemediationFailed,
    ConstitutionalViolation,
    AlgorithmDivergence,
}

/// Heuristic mapping for failure analysis

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FailureHeuristics {
    pub failure_patterns: HashMap<FailureCategory, FailurePattern>,
    pub remediation_strategies: HashMap<FailureCategory, Vec<String>>,
    pub recovery_weights: HashMap<FailureCategory, f64>,
}

impl FailureHeuristics {
    /// Create default failure heuristics
    pub fn new() -> Self {
        let mut failure_patterns = HashMap::new();
        let mut remediation_strategies = HashMap::new();
        let mut recovery_weights = HashMap::new();

        // Timeout patterns
        failure_patterns.insert(FailureCategory::Timeout, FailurePattern {
            keywords: vec![
                "timeout".to_string(),
                "time out".to_string(),
                "deadline".to_string(),
                "expired".to_string(),
            ],
            severity_indicators: vec![
                "critical".to_string(),
                "blocking".to_string(),
            ],
            recovery_probability: 0.7,
            common_causes: vec![
                "Resource contention".to_string(),
                "Algorithm complexity".to_string(),
                "External service delay".to_string(),
            ],
        });

        remediation_strategies.insert(FailureCategory::Timeout, vec![
            "Increase timeout limits".to_string(),
            "Optimize algorithm complexity".to_string(),
            "Implement circuit breaker".to_string(),
        ]);

        recovery_weights.insert(FailureCategory::Timeout, 0.8);

        // Resource exhaustion patterns
        failure_patterns.insert(FailureCategory::ResourceExhaustion, FailurePattern {
            keywords: vec![
                "resource".to_string(),
                "memory".to_string(),
                "cpu".to_string(),
                "exhaust".to_string(),
                "load".to_string(),
            ],
            severity_indicators: vec![
                "critical".to_string(),
                "exhaustion".to_string(),
            ],
            recovery_probability: 0.6,
            common_causes: vec![
                "Memory leak".to_string(),
                "High concurrency".to_string(),
                "Inefficient algorithms".to_string(),
            ],
        });

        remediation_strategies.insert(FailureCategory::ResourceExhaustion, vec![
            "Implement resource limits".to_string(),
            "Optimize memory usage".to_string(),
            "Reduce concurrency".to_string(),
        ]);

        recovery_weights.insert(FailureCategory::ResourceExhaustion, 0.6);

        // Quality degradation patterns
        failure_patterns.insert(FailureCategory::QualityDegradation, FailurePattern {
            keywords: vec![
                "quality".to_string(),
                "degraded".to_string(),
                "regression".to_string(),
                "inconsistent".to_string(),
            ],
            severity_indicators: vec![
                "medium".to_string(),
                "degradation".to_string(),
            ],
            recovery_probability: 0.8,
            common_causes: vec![
                "Model drift".to_string(),
                "Training data issues".to_string(),
                "Algorithm instability".to_string(),
            ],
        });

        remediation_strategies.insert(FailureCategory::QualityDegradation, vec![
            "Retrain model".to_string(),
            "Validate training data".to_string(),
            "Implement quality monitoring".to_string(),
        ]);

        recovery_weights.insert(FailureCategory::QualityDegradation, 0.7);

        Self {
            failure_patterns,
            remediation_strategies,
            recovery_weights,
        }
    }

    /// Analyze failure from error message and context
    pub fn analyze_failure(&self, error_message: &str, context: &FailureContext) -> FailureAnalysis {
        let category = self.categorize_failure(error_message, context);
        let severity = self.determine_severity(&category, context);
        let recovery_probability = self.recovery_weights.get(&category).copied().unwrap_or(0.5);

        let remediation_suggestions = self.remediation_strategies
            .get(&category)
            .cloned()
            .unwrap_or_default();

        let root_cause_indicators = self.failure_patterns
            .get(&category)
            .map(|pattern| pattern.common_causes.clone())
            .unwrap_or_default();

        FailureAnalysis {
            category,
            severity,
            recovery_probability,
            remediation_suggestions,
            root_cause_indicators,
        }
    }

    /// Categorize failure based on error message and context
    fn categorize_failure(&self, error_message: &str, context: &FailureContext) -> FailureCategory {
        let message_lower = error_message.to_lowercase();

        // Check timeout indicators
        if context.execution_time > context.timeout_threshold ||
           message_lower.contains("timeout") || message_lower.contains("deadline") {
            return FailureCategory::Timeout;
        }

        // Check resource indicators
        if context.resource_usage.cpu_seconds > 45.0 ||
           context.resource_usage.memory_bytes > 14_000 ||
           message_lower.contains("resource") || message_lower.contains("memory") {
            return FailureCategory::ResourceExhaustion;
        }

        // Check quality indicators
        if context.quality_score < 0.6 ||
           message_lower.contains("quality") || message_lower.contains("degraded") {
            return FailureCategory::QualityDegradation;
        }

        // Check consensus indicators
        if message_lower.contains("consensus") || message_lower.contains("dissent") {
            return FailureCategory::ConsensusFailure;
        }

        // Default to algorithm divergence
        FailureCategory::AlgorithmDivergence
    }

    /// Determine failure severity
    fn determine_severity(&self, category: &FailureCategory, context: &FailureContext) -> FailureSeverity {
        match category {
            FailureCategory::Timeout | FailureCategory::ResourceExhaustion => {
                if context.attempt_count > 3 {
                    FailureSeverity::Critical
                } else if context.attempt_count > 1 {
                    FailureSeverity::High
                } else {
                    FailureSeverity::Medium
                }
            }
            FailureCategory::QualityDegradation => {
                if context.quality_score < 0.5 {
                    FailureSeverity::High
                } else {
                    FailureSeverity::Medium
                }
            }
            FailureCategory::ConstitutionalViolation => FailureSeverity::Critical,
            _ => FailureSeverity::Medium,
        }
    }
}

/// Pattern for failure analysis

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FailurePattern {
    pub keywords: Vec<String>,
    pub severity_indicators: Vec<String>,
    pub recovery_probability: f64,
    pub common_causes: Vec<String>,
}

/// Failure analysis result

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FailureAnalysis {
    pub category: FailureCategory,
    pub severity: FailureSeverity,
    pub recovery_probability: f64,
    pub remediation_suggestions: Vec<String>,
    pub root_cause_indicators: Vec<String>,
}

/// Severity levels for failure analysis

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum FailureSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Context information for failure analysis

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FailureContext {
    pub execution_time: f64,
    pub timeout_threshold: f64,
    pub resource_usage: super::resources::ResourceMetrics,
    pub quality_score: f64,
    pub attempt_count: u32,
    pub error_message: String,
}

/// Failure recovery strategy

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum RecoveryStrategy {
    RetryWithBackoff,
    ReduceComplexity,
    IncreaseResources,
    RetrainModel,
    FallbackMode,
    ManualIntervention,
}

impl FailureAnalysis {
    /// Get recommended recovery strategy
    pub fn get_recovery_strategy(&self) -> RecoveryStrategy {
        match (&self.category, &self.severity) {
            (FailureCategory::Timeout, FailureSeverity::Low | FailureSeverity::Medium) => {
                RecoveryStrategy::RetryWithBackoff
            }
            (FailureCategory::ResourceExhaustion, _) => {
                RecoveryStrategy::IncreaseResources
            }
            (FailureCategory::QualityDegradation, _) => {
                RecoveryStrategy::RetrainModel
            }
            (FailureCategory::AlgorithmDivergence, _) => {
                RecoveryStrategy::ReduceComplexity
            }
            (_, FailureSeverity::Critical) => {
                RecoveryStrategy::ManualIntervention
            }
            _ => RecoveryStrategy::RetryWithBackoff,
        }
    }

    /// Check if failure is recoverable
    pub fn is_recoverable(&self) -> bool {
        self.recovery_probability > 0.5
    }

    /// Get failure summary
    pub fn get_summary(&self) -> String {
        format!(
            "Failure: {:?} (Severity: {:?}, Recovery Probability: {:.1}%)",
            self.category,
            self.severity,
            self.recovery_probability * 100.0
        )
    }
}


