//! Failure taxonomy for categorizing and analyzing failures

use schemars::JsonSchema;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::learning::types::*;
use crate::worker_types::{ExecutionOutcome, LearningMode};

/// Categorizes and analyzes failure patterns
pub struct FailureTaxonomy {
    failure_categories: HashMap<FailureCategory, Vec<FailurePattern>>,
    analysis_history: Vec<FailureAnalysis>,
}

impl FailureTaxonomy {
    pub fn new() -> Self {
        Self {
            failure_categories: HashMap::new(),
            analysis_history: Vec::new(),
        }
    }

    /// Analyze a failure and categorize it
    pub async fn analyze_failure(
        &mut self,
        error_message: &str,
        execution_context: &HashMap<String, serde_json::Value>,
    ) -> Result<FailureAnalysis, Box<dyn std::error::Error + Send + Sync>> {
        let category = self.categorize_error(error_message);
        let root_cause = self.identify_root_cause(error_message, execution_context);
        let contributing_factors = self.identify_contributing_factors(error_message, execution_context);
        let prevention_suggestions = self.generate_prevention_suggestions(&category, &root_cause);

        let analysis = FailureAnalysis {
            category: category.clone(),
            root_cause: root_cause.clone(),
            contributing_factors,
            prevention_suggestions,
            confidence: self.calculate_confidence(&category, &root_cause),
        };

        // Store analysis in history
        self.analysis_history.push(analysis.clone());

        // Update failure patterns
        self.update_failure_patterns(&category, &root_cause, execution_context).await?;

        Ok(analysis)
    }

    /// Categorize error message into failure category
    fn categorize_error(&self, error_message: &str) -> FailureCategory {
        let error_lower = error_message.to_lowercase();

        if error_lower.contains("timeout") || error_lower.contains("timed out") {
            FailureCategory::Timeout
        } else if error_lower.contains("memory") || error_lower.contains("out of memory") || error_lower.contains("oom") {
            FailureCategory::ResourceExhaustion
        } else if error_lower.contains("permission") || error_lower.contains("access denied") || error_lower.contains("unauthorized") {
            FailureCategory::WorkerFailure
        } else if error_lower.contains("network") || error_lower.contains("connection") || error_lower.contains("unreachable") {
            FailureCategory::DependencyFailure
        } else if error_lower.contains("syntax") || error_lower.contains("parse") || error_lower.contains("invalid") {
            FailureCategory::TaskFailure
        } else if error_lower.contains("quality") || error_lower.contains("coverage") || error_lower.contains("test") {
            FailureCategory::QualityViolation
        } else if error_lower.contains("config") || error_lower.contains("setting") || error_lower.contains("parameter") {
            FailureCategory::ConfigurationError
        } else {
            FailureCategory::Unknown
        }
    }

    /// Identify root cause of failure
    fn identify_root_cause(&self, error_message: &str, context: &HashMap<String, serde_json::Value>) -> String {
        let error_lower = error_message.to_lowercase();

        // Check context for additional clues
        let mut root_cause = if error_lower.contains("timeout") {
            "Task execution exceeded time limit".to_string()
        } else if error_lower.contains("memory") {
            "Insufficient memory resources available".to_string()
        } else if error_lower.contains("permission") {
            "Insufficient permissions for operation".to_string()
        } else if error_lower.contains("network") {
            "Network connectivity issues".to_string()
        } else if error_lower.contains("syntax") {
            "Invalid syntax or malformed input".to_string()
        } else if error_lower.contains("quality") {
            "Quality requirements not met".to_string()
        } else if error_lower.contains("config") {
            "Configuration parameter error".to_string()
        } else {
            "Unknown failure cause".to_string()
        };

        // Add context-specific information
        if let Some(worker_id) = context.get("worker_id") {
            root_cause.push_str(&format!(" (Worker: {})", worker_id));
        }

        if let Some(task_type) = context.get("task_type") {
            root_cause.push_str(&format!(" (Task: {})", task_type));
        }

        root_cause
    }

    /// Identify contributing factors
    fn identify_contributing_factors(&self, error_message: &str, context: &HashMap<String, serde_json::Value>) -> Vec<String> {
        let mut factors = Vec::new();

        // Analyze error message for contributing factors
        let error_lower = error_message.to_lowercase();
        
        if error_lower.contains("high load") || error_lower.contains("busy") {
            factors.push("High system load".to_string());
        }
        
        if error_lower.contains("resource") || error_lower.contains("limit") {
            factors.push("Resource constraints".to_string());
        }
        
        if error_lower.contains("dependency") || error_lower.contains("service") {
            factors.push("External service dependency".to_string());
        }

        // Analyze context for additional factors
        if let Some(complexity) = context.get("complexity") {
            if complexity.as_f64().unwrap_or(0.0) > 0.8 {
                factors.push("High task complexity".to_string());
            }
        }

        if let Some(priority) = context.get("priority") {
            if priority.as_str().unwrap_or("") == "high" {
                factors.push("High priority task".to_string());
            }
        }

        if factors.is_empty() {
            factors.push("No specific contributing factors identified".to_string());
        }

        factors
    }

    /// Generate prevention suggestions
    fn generate_prevention_suggestions(&self, category: &FailureCategory, root_cause: &str) -> Vec<String> {
        let mut suggestions = Vec::new();

        match category {
            FailureCategory::Timeout => {
                suggestions.push("Increase timeout limits for long-running tasks".to_string());
                suggestions.push("Optimize task execution for better performance".to_string());
                suggestions.push("Consider breaking down complex tasks into smaller subtasks".to_string());
            }
            FailureCategory::ResourceExhaustion => {
                suggestions.push("Increase memory allocation for workers".to_string());
                suggestions.push("Implement memory usage monitoring and alerts".to_string());
                suggestions.push("Optimize memory usage in task execution".to_string());
            }
            FailureCategory::WorkerFailure => {
                suggestions.push("Check worker permissions and access rights".to_string());
                suggestions.push("Implement proper error handling and recovery".to_string());
                suggestions.push("Add worker health monitoring".to_string());
            }
            FailureCategory::DependencyFailure => {
                suggestions.push("Implement circuit breaker pattern for external dependencies".to_string());
                suggestions.push("Add retry logic with exponential backoff".to_string());
                suggestions.push("Implement fallback mechanisms".to_string());
            }
            FailureCategory::TaskFailure => {
                suggestions.push("Validate input data before task execution".to_string());
                suggestions.push("Implement proper error handling in task logic".to_string());
                suggestions.push("Add input sanitization and validation".to_string());
            }
            FailureCategory::QualityViolation => {
                suggestions.push("Review and adjust quality thresholds".to_string());
                suggestions.push("Implement quality checks at multiple stages".to_string());
                suggestions.push("Add quality monitoring and alerting".to_string());
            }
            FailureCategory::ConfigurationError => {
                suggestions.push("Validate configuration parameters before use".to_string());
                suggestions.push("Implement configuration schema validation".to_string());
                suggestions.push("Add configuration testing and validation".to_string());
            }
            FailureCategory::Unknown => {
                suggestions.push("Implement comprehensive error logging and monitoring".to_string());
                suggestions.push("Add detailed error context collection".to_string());
                suggestions.push("Review and improve error handling coverage".to_string());
            }
        }

        suggestions
    }

    /// Calculate confidence in analysis
    fn calculate_confidence(&self, category: &FailureCategory, root_cause: &str) -> f64 {
        let mut confidence = 0.5; // Base confidence

        // Increase confidence based on category specificity
        match category {
            FailureCategory::Timeout | FailureCategory::ResourceExhaustion => confidence += 0.3,
            FailureCategory::WorkerFailure | FailureCategory::DependencyFailure => confidence += 0.2,
            FailureCategory::TaskFailure | FailureCategory::QualityViolation => confidence += 0.25,
            FailureCategory::ConfigurationError => confidence += 0.2,
            FailureCategory::Unknown => confidence -= 0.2,
        }

        // Increase confidence if root cause is specific
        if root_cause.len() > 50 && !root_cause.contains("Unknown") {
            confidence += 0.1;
        }

        confidence.min(1.0).max(0.0)
    }

    /// Update failure patterns based on analysis
    async fn update_failure_patterns(
        &mut self,
        category: &FailureCategory,
        root_cause: &str,
        context: &HashMap<String, serde_json::Value>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Create or update failure pattern
        let pattern = FailurePattern {
            id: Uuid::new_v4(),
            pattern_type: PatternType::TaskComplexity, // Default type
            conditions: context.clone(),
            failure_rate: 1.0, // This is a failure
            common_errors: vec![root_cause.to_string()],
            frequency: 1,
            created_at: Utc::now(),
        };

        // Add to category
        self.failure_categories.entry(category.clone()).or_default().push(pattern);

        Ok(())
    }

    /// Get failure patterns by category
    pub fn get_failure_patterns(&self, category: &FailureCategory) -> Vec<&FailurePattern> {
        self.failure_categories.get(category).map(|patterns| patterns.iter().collect()).unwrap_or_default()
    }

    /// Get all failure categories
    pub fn get_all_categories(&self) -> Vec<FailureCategory> {
        self.failure_categories.keys().cloned().collect()
    }

    /// Get analysis history
    pub fn get_analysis_history(&self) -> &Vec<FailureAnalysis> {
        &self.analysis_history
    }

    /// Get failure statistics
    pub fn get_failure_statistics(&self) -> FailureStatistics {
        let total_failures = self.analysis_history.len();
        let mut category_counts = HashMap::new();

        for analysis in &self.analysis_history {
            *category_counts.entry(analysis.category.clone()).or_insert(0) += 1;
        }

        FailureStatistics {
            total_failures,
            category_counts,
            most_common_category: self.find_most_common_category(),
            average_confidence: self.calculate_average_confidence(),
        }
    }

    /// Find most common failure category
    fn find_most_common_category(&self) -> Option<FailureCategory> {
        let mut category_counts = HashMap::new();
        
        for analysis in &self.analysis_history {
            *category_counts.entry(analysis.category.clone()).or_insert(0) += 1;
        }

        category_counts.into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(category, _)| category)
    }

    /// Calculate average confidence
    fn calculate_average_confidence(&self) -> f64 {
        if self.analysis_history.is_empty() {
            0.0
        } else {
            self.analysis_history.iter().map(|a| a.confidence).sum::<f64>() / self.analysis_history.len() as f64
        }
    }
}

impl Default for FailureTaxonomy {
    fn default() -> Self {
        Self::new()
    }
}

/// Failure statistics

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct FailureStatistics {
    pub total_failures: usize,
    pub category_counts: HashMap<FailureCategory, usize>,
    pub most_common_category: Option<FailureCategory>,
    pub average_confidence: f64,
}
