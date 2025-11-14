//! Pattern analyzer for identifying execution patterns

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::learning::types::*;
use crate::worker_types::{ExecutionOutcome, LearningMode};
use crate::{TaskId, WorkerId};

/// Analyzes execution patterns to identify success and failure patterns
pub struct PatternAnalyzer {
    min_pattern_frequency: u64,
    confidence_threshold: f64,
    success_patterns: Arc<tokio::sync::RwLock<Vec<SuccessPattern>>>,
    failure_patterns: Arc<tokio::sync::RwLock<Vec<FailurePattern>>>,
    optimal_configs: Arc<tokio::sync::RwLock<Vec<OptimalConfig>>>,
}

impl PatternAnalyzer {
    pub fn new(min_pattern_frequency: u64, confidence_threshold: f64) -> Self {
        Self {
            min_pattern_frequency,
            confidence_threshold,
            success_patterns: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            failure_patterns: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            optimal_configs: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }

    /// Analyze execution records to identify patterns
    pub async fn analyze_execution_records(
        &self,
        records: &[ExecutionRecord],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Group records by characteristics
        let success_records: Vec<&ExecutionRecord> = records.iter().filter(|r| r.success).collect();
        let failure_records: Vec<&ExecutionRecord> =
            records.iter().filter(|r| !r.success).collect();

        // Analyze success patterns
        self.analyze_success_patterns(&success_records).await?;

        // Analyze failure patterns
        self.analyze_failure_patterns(&failure_records).await?;

        // Identify optimal configurations
        self.identify_optimal_configs(records).await?;

        Ok(())
    }

    /// Analyze success patterns
    async fn analyze_success_patterns(
        &self,
        records: &[&ExecutionRecord],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if records.len() < self.min_pattern_frequency as usize {
            return Ok(());
        }

        // Group by worker specialty
        let mut specialty_groups: HashMap<String, Vec<&ExecutionRecord>> = HashMap::new();
        for record in records {
            let specialty = record
                .metadata
                .get("specialty")
                .and_then(|v| v.as_str())
                .unwrap_or("General");
            specialty_groups
                .entry(specialty.to_string())
                .or_default()
                .push(record);
        }

        // Create patterns for each specialty group
        for (specialty, group_records) in specialty_groups {
            if group_records.len() >= self.min_pattern_frequency as usize {
                let pattern = SuccessPattern {
                    id: Uuid::new_v4(),
                    pattern_type: PatternType::WorkerCapability,
                    conditions: HashMap::from([
                        (
                            "specialty".to_string(),
                            serde_json::Value::String(specialty),
                        ),
                        (
                            "min_executions".to_string(),
                            serde_json::Value::Number(self.min_pattern_frequency.into()),
                        ),
                    ]),
                    success_rate: 1.0, // All records in this group are successful
                    average_quality: group_records.iter().map(|r| r.quality_score).sum::<f64>()
                        / group_records.len() as f64,
                    frequency: group_records.len() as u64,
                    created_at: Utc::now(),
                };

                let mut patterns = self.success_patterns.write().await;
                patterns.push(pattern);
            }
        }

        Ok(())
    }

    /// Analyze failure patterns
    async fn analyze_failure_patterns(
        &self,
        records: &[&ExecutionRecord],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if records.len() < self.min_pattern_frequency as usize {
            return Ok(());
        }

        // Group by error type
        let mut error_groups: HashMap<String, Vec<&ExecutionRecord>> = HashMap::new();
        for record in records {
            let error_type = record
                .error_message
                .as_ref()
                .map(|msg| classify_error(msg))
                .unwrap_or("Unknown".to_string());
            error_groups.entry(error_type).or_default().push(record);
        }

        // Create patterns for each error group
        for (error_type, group_records) in error_groups {
            if group_records.len() >= self.min_pattern_frequency as usize {
                let pattern = FailurePattern {
                    id: Uuid::new_v4(),
                    pattern_type: PatternType::TaskComplexity,
                    conditions: HashMap::from([
                        (
                            "error_type".to_string(),
                            serde_json::Value::String(error_type),
                        ),
                        (
                            "min_executions".to_string(),
                            serde_json::Value::Number(self.min_pattern_frequency.into()),
                        ),
                    ]),
                    failure_rate: 1.0, // All records in this group are failures
                    common_errors: group_records
                        .iter()
                        .filter_map(|r| r.error_message.clone())
                        .collect::<std::collections::HashSet<_>>()
                        .into_iter()
                        .collect(),
                    frequency: group_records.len() as u64,
                    created_at: Utc::now(),
                };

                let mut patterns = self.failure_patterns.write().await;
                patterns.push(pattern);
            }
        }

        Ok(())
    }

    /// Identify optimal configurations
    async fn identify_optimal_configs(
        &self,
        records: &[ExecutionRecord],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Group records by configuration characteristics
        let mut config_groups: HashMap<String, Vec<&ExecutionRecord>> = HashMap::new();

        for record in records {
            let config_key = format!(
                "worker_{}_quality_{:.2}",
                record.worker_id.0,
                (record.quality_score * 10.0).round() / 10.0
            );
            config_groups.entry(config_key).or_default().push(record);
        }

        // Find optimal configurations
        for (config_key, group_records) in config_groups {
            if group_records.len() >= self.min_pattern_frequency as usize {
                let success_rate = group_records.iter().filter(|r| r.success).count() as f64
                    / group_records.len() as f64;
                let avg_execution_time = group_records
                    .iter()
                    .map(|r| r.execution_time_ms)
                    .sum::<u64>() as f64
                    / group_records.len() as f64;
                let avg_quality = group_records.iter().map(|r| r.quality_score).sum::<f64>()
                    / group_records.len() as f64;

                if success_rate >= self.confidence_threshold && avg_quality >= 0.8 {
                    let config = OptimalConfig {
                        id: Uuid::new_v4(),
                        config_type: ConfigType::WorkerSelection,
                        worker_type: "general".to_string(), // Default worker type
                        task_type: "general".to_string(),   // Default task type
                        config: serde_json::Value::Object(serde_json::Map::new()), // Empty config
                        parameters: HashMap::from([(
                            "config_key".to_string(),
                            serde_json::Value::String(config_key),
                        )]),
                        conditions: HashMap::from([(
                            "min_executions".to_string(),
                            serde_json::Value::Number(self.min_pattern_frequency.into()),
                        )]),
                        performance_metrics: PerformanceMetrics {
                            execution_time_ms: avg_execution_time,
                            quality_score: avg_quality,
                            success_rate,
                            resource_utilization: 0.8, // Default value
                            cost_score: 0.7,           // Default value
                        },
                        confidence: success_rate,
                        expires_at: None,
                        metadata: serde_json::Value::Object(serde_json::Map::new()),
                        created_at: Utc::now(),
                    };

                    let mut configs = self.optimal_configs.write().await;
                    configs.push(config);
                }
            }
        }

        Ok(())
    }

    /// Get all patterns
    pub async fn get_all_patterns(
        &self,
    ) -> (Vec<SuccessPattern>, Vec<FailurePattern>, Vec<OptimalConfig>) {
        let success_patterns = self.success_patterns.read().await.clone();
        let failure_patterns = self.failure_patterns.read().await.clone();
        let optimal_configs = self.optimal_configs.read().await.clone();

        (success_patterns, failure_patterns, optimal_configs)
    }

    /// Match a task against known patterns
    pub async fn match_task_pattern(
        &self,
        task_characteristics: &HashMap<String, serde_json::Value>,
    ) -> Result<Vec<PatternMatch>, Box<dyn std::error::Error + Send + Sync>> {
        let mut matches = Vec::new();

        // Match against success patterns
        {
            let patterns = self.success_patterns.read().await;
            for pattern in patterns.iter() {
                let match_score =
                    calculate_pattern_match_score(task_characteristics, &pattern.conditions);
                if match_score > self.confidence_threshold {
                    matches.push(PatternMatch {
                        pattern_id: pattern.id,
                        match_score,
                        matched_characteristics: pattern.conditions.keys().cloned().collect(),
                        confidence: pattern.success_rate,
                    });
                }
            }
        }

        // Match against failure patterns
        {
            let patterns = self.failure_patterns.read().await;
            for pattern in patterns.iter() {
                let match_score =
                    calculate_pattern_match_score(task_characteristics, &pattern.conditions);
                if match_score > self.confidence_threshold {
                    matches.push(PatternMatch {
                        pattern_id: pattern.id,
                        match_score,
                        matched_characteristics: pattern.conditions.keys().cloned().collect(),
                        confidence: pattern.failure_rate,
                    });
                }
            }
        }

        Ok(matches)
    }
}

/// Classify error message into error type
fn classify_error(error_message: &str) -> String {
    let error_lower = error_message.to_lowercase();

    if error_lower.contains("timeout") {
        "Timeout".to_string()
    } else if error_lower.contains("memory") || error_lower.contains("out of memory") {
        "MemoryError".to_string()
    } else if error_lower.contains("permission") || error_lower.contains("access denied") {
        "PermissionError".to_string()
    } else if error_lower.contains("network") || error_lower.contains("connection") {
        "NetworkError".to_string()
    } else if error_lower.contains("syntax") || error_lower.contains("parse") {
        "SyntaxError".to_string()
    } else {
        "Unknown".to_string()
    }
}

/// Calculate pattern match score
fn calculate_pattern_match_score(
    task_characteristics: &HashMap<String, serde_json::Value>,
    pattern_conditions: &HashMap<String, serde_json::Value>,
) -> f64 {
    let mut matches = 0;
    let mut total_conditions = pattern_conditions.len();

    for (key, pattern_value) in pattern_conditions {
        if let Some(task_value) = task_characteristics.get(key) {
            if task_value == pattern_value {
                matches += 1;
            }
        }
    }

    if total_conditions == 0 {
        0.0
    } else {
        matches as f64 / total_conditions as f64
    }
}
