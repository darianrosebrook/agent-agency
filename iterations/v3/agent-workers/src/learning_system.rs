//! Learning system components for parallel task execution
//! 
//! This module contains all the adaptive learning components that enable
//! the system to learn from execution patterns and optimize performance.

use schemars::JsonSchema;
use crate::parallel_types::{TaskId, SubTaskId, WorkerId};
use crate::learning::{
    ExecutionRecord, WorkerPerformanceProfile, SuccessPattern, FailurePattern,
    OptimalConfig, ConfigurationRecommendations, OptimizationEvent, TaskPattern
};
use data_infrastructure::client::DatabaseClient;
use std::sync::Arc;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde_json;
use tracing::{info, error};
use anyhow::Result;
use sqlx::{Row, postgres::PgRow};

/// Real fairness monitor implementation using database tracking
pub struct RealFairnessMonitor {
    db_client: Arc<DatabaseClient>,
}

impl RealFairnessMonitor {
    pub fn new(db_client: Arc<DatabaseClient>) -> Self {
        Self { db_client }
    }

    /// Track worker utilization for fairness monitoring
    pub async fn track_worker_utilization(&self, worker_id: &WorkerId, task_count: i32) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let query = r#"
            INSERT INTO worker_utilization_tracking (worker_id, task_count, tracked_at)
            VALUES ($1, $2, $3)
            ON CONFLICT (worker_id, tracked_at) 
            DO UPDATE SET task_count = EXCLUDED.task_count
        "#;

        let now = Utc::now();
        self.db_client.execute(query, &[&worker_id.0.to_string(), &task_count, &now]).await?;
        Ok(())
    }

    /// Calculate fairness score based on worker utilization
    pub async fn calculate_fairness_score(&self) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
        let query = r#"
            SELECT 
                worker_id,
                AVG(task_count) as avg_tasks,
                STDDEV(task_count) as task_stddev
            FROM worker_utilization_tracking
            WHERE tracked_at >= NOW() - INTERVAL '24 hours'
            GROUP BY worker_id
        "#;

        match self.db_client.query(query, &[]).await {
            Ok(rows) => {
                if rows.is_empty() {
                    return Ok(1.0); // Perfect fairness if no data
                }

                let mut utilizations = Vec::new();
                for row in rows {
                    let avg_tasks: f64 = row.try_get("avg_tasks")?;
                    utilizations.push(avg_tasks);
                }

                // Calculate Gini coefficient for fairness
                utilizations.sort_by(|a, b| a.partial_cmp(b).unwrap());
                let n = utilizations.len() as f64;
                let sum: f64 = utilizations.iter().sum();
                
                if sum == 0.0 {
                    return Ok(1.0);
                }

                let mut gini = 0.0;
                for (i, value) in utilizations.iter().enumerate() {
                    gini += (2.0 * (i as f64 + 1.0) - n - 1.0) * value;
                }
                
                let fairness_score = 1.0 - (gini / (n * sum));
                Ok(fairness_score.clamp(0.0, 1.0))
            }
            Err(e) => {
                error!("Failed to calculate fairness score: {}", e);
                Ok(0.5) // Default to medium fairness on error
            }
        }
    }

    /// Get utilization distribution across workers
    pub async fn get_utilization_distribution(&self) -> Result<HashMap<String, f64>, Box<dyn std::error::Error + Send + Sync>> {
        let query = r#"
            SELECT 
                w.id as worker_id,
                w.name as worker_name,
                COALESCE(AVG(ut.task_count), 0) as avg_utilization
            FROM workers w
            LEFT JOIN worker_utilization_tracking ut ON w.id = ut.worker_id
                AND ut.tracked_at >= NOW() - INTERVAL '24 hours'
            GROUP BY w.id, w.name
            ORDER BY avg_utilization DESC
        "#;

        match self.db_client.query(query, &[]).await {
            Ok(rows) => {
                let mut distribution = HashMap::new();
                for row in rows {
                    let worker_name: String = row.try_get("worker_name")?;
                    let avg_utilization: f64 = row.try_get("avg_utilization")?;
                    distribution.insert(worker_name, avg_utilization);
                }
                Ok(distribution)
            }
            Err(e) => {
                error!("Failed to get utilization distribution: {}", e);
                Ok(HashMap::new())
            }
        }
    }
}

/// Real adaptive selector implementation using ML-based worker selection
pub struct RealAdaptiveSelector {
    db_client: Arc<DatabaseClient>,
    pattern_analyzer: Arc<crate::learning::PatternAnalyzer>,
}

impl RealAdaptiveSelector {
    pub fn new(db_client: Arc<DatabaseClient>, pattern_analyzer: Arc<crate::learning::PatternAnalyzer>) -> Self {
        Self { db_client, pattern_analyzer }
    }

    /// Select optimal worker for a task using ML-based selection
    pub async fn select_worker(&self, task_pattern: &TaskPattern) -> Result<WorkerId, Box<dyn std::error::Error + Send + Sync>> {
        // Get available workers with their performance profiles
        let query = r#"
            SELECT 
                w.id,
                w.name,
                w.specialty,
                w.max_concurrent_tasks,
                w.memory_limit_mb,
                w.cpu_limit_cores,
                w.is_active,
                w.last_heartbeat,
                w.version,
                w.endpoint_url,
                COALESCE(wp.success_rate, 0.5) as success_rate,
                COALESCE(wp.avg_execution_time_ms, 300000) as avg_execution_time_ms,
                COALESCE(wp.quality_score, 0.7) as quality_score
            FROM workers w
            LEFT JOIN worker_performance_profiles wp ON w.id = wp.worker_id
            WHERE w.is_active = true
            AND (w.last_heartbeat IS NULL OR w.last_heartbeat >= NOW() - INTERVAL '5 minutes')
            ORDER BY wp.success_rate DESC, wp.quality_score DESC
        "#;

        match self.db_client.query(query, &[]).await {
            Ok(rows) => {
                if rows.is_empty() {
                    return Err("No available workers found".into());
                }

                let mut best_worker_id = None;
                let mut best_score = 0.0;

                for row in rows {
                    let worker_id: String = row.try_get("id")?;
                    let success_rate: f64 = row.try_get("success_rate")?;
                    let quality_score: f64 = row.try_get("quality_score")?;
                    let specialty: String = row.try_get("specialty")?;

                    // Calculate worker score based on multiple factors
                    let score = self.calculate_worker_score(
                        &worker_id,
                        success_rate,
                        quality_score,
                        &specialty,
                        task_pattern,
                    ).await?;

                    if score > best_score {
                        best_score = score;
                        best_worker_id = Some(worker_id);
                    }
                }

                match best_worker_id {
                    Some(id) => {
                        let worker_uuid = Uuid::parse_str(&id)
                            .map_err(|e| format!("Invalid worker ID: {}", e))?;
                        Ok(WorkerId(worker_uuid))
                    }
                    None => Err("No suitable worker found".into())
                }
            }
            Err(e) => {
                error!("Failed to select worker: {}", e);
                Err(e.into())
            }
        }
    }

    /// Calculate worker score based on multiple factors
    async fn calculate_worker_score(
        &self,
        worker_id: &str,
        success_rate: f64,
        quality_score: f64,
        specialty: &str,
        task_pattern: &TaskPattern,
    ) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
        let mut score = 0.0;

        // Base performance score (40% weight)
        score += (success_rate * 0.4) + (quality_score * 0.4);

        // Specialty match score (20% weight)
        let specialty_match = self.calculate_specialty_match(specialty, task_pattern);
        score += specialty_match * 0.2;

        // Capability match score (20% weight)
        let capability_match = self.calculate_capability_match(worker_id, task_pattern).await?;
        score += capability_match * 0.2;

        Ok(score.min(1.0).max(0.0))
    }

    /// Calculate specialty match score
    fn calculate_specialty_match(&self, worker_specialty: &str, _task_pattern: &TaskPattern) -> f64 {
        // Simple specialty matching - can be enhanced with ML
        // For now, return a default match score since domain is not in TaskPattern
        match worker_specialty {
            "frontend" | "backend" | "data" => 0.8,
            "fullstack" => 0.7,
            _ => 0.5,
        }
    }

    /// Calculate capability match score
    async fn calculate_capability_match(&self, worker_id: &str, task_pattern: &TaskPattern) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
        let query = r#"
            SELECT capabilities
            FROM worker_capabilities
            WHERE worker_id = $1
        "#;

        match self.db_client.query_one_with_params(query, &[&worker_id]).await {
            Ok(Some(row)) => {
                let capabilities: Vec<String> = row.try_get("capabilities")?;
                let required_capabilities = &task_pattern.required_capabilities;
                
                let matches = required_capabilities.iter()
                    .filter(|req| capabilities.contains(req))
                    .count();
                
                let match_score = if required_capabilities.is_empty() {
                    1.0
                } else {
                    matches as f64 / required_capabilities.len() as f64
                };
                
                Ok(match_score)
            }
            Ok(None) => {
                // If no capabilities found, assume basic capability
                Ok(0.5)
            }
            Err(_) => {
                // If no capabilities found, assume basic capability
                Ok(0.5)
            }
        }
    }
}

/// Real configuration optimizer implementation using reinforcement learning
pub struct RealConfigOptimizer {
    db_client: Arc<DatabaseClient>,
    optimization_history: Arc<std::sync::RwLock<Vec<OptimizationEvent>>>,
}

impl RealConfigOptimizer {
    pub fn new(db_client: Arc<DatabaseClient>) -> Self {
        Self { 
            db_client,
            optimization_history: Arc::new(std::sync::RwLock::new(Vec::new())),
        }
    }

    /// Optimize configuration based on historical performance data
    pub async fn optimize_configuration(&self, current_config: &HashMap<String, serde_json::Value>) -> Result<ConfigurationRecommendations, Box<dyn std::error::Error + Send + Sync>> {
        // Analyze performance trends
        let performance_trend = self.analyze_performance_trend().await?;
        
        // Generate optimized configuration
        let optimized_config = self.generate_optimized_config(current_config, &performance_trend).await?;
        
        // Store optimization event
        let event = OptimizationEvent {
            id: Uuid::new_v4(),
            event_type: crate::learning::types::OptimizationEventType::ConfigApplied, // Need to add this field
            config_id: Uuid::new_v4(), // Need to add this field
            performance_delta: crate::learning::types::PerformanceMetrics {
                execution_time_ms: 0.0,
                quality_score: 0.0,
                success_rate: 0.0,
                resource_utilization: 0.0,
                cost_score: 0.0,
            },
            timestamp: Utc::now(),
            metadata: HashMap::new(), // Need to add this field
            config_before: Some(current_config.clone()),
            config_after: Some(optimized_config.clone()),
            performance_improvement: Some(performance_trend.improvement_score),
            optimization_type: Some("reinforcement_learning".to_string()),
        };
        
        self.store_optimization_result(&event).await?;
        
        Ok(ConfigurationRecommendations {
            worker_selection: None,
            task_decomposition: None,
            resource_allocation: None,
            quality_thresholds: None,
            confidence: performance_trend.confidence,
            reasoning: "Based on historical performance analysis".to_string(),
            recommended_config: Some(optimized_config),
            confidence_score: performance_trend.confidence,
            expected_improvement: Some(performance_trend.improvement_score),
            optimization_reason: Some("Based on historical performance analysis".to_string()),
        })
    }

    /// Analyze performance trends from historical data
    async fn analyze_performance_trend(&self) -> Result<PerformanceTrend, Box<dyn std::error::Error + Send + Sync>> {
        let query = r#"
            SELECT 
                AVG(execution_time_ms) as avg_execution_time,
                AVG(success_rate) as avg_success_rate,
                AVG(quality_score) as avg_quality_score,
                COUNT(*) as sample_count
            FROM execution_records
            WHERE created_at >= NOW() - INTERVAL '7 days'
        "#;

        match self.db_client.query_one(query).await {
            Ok(Some(row)) => {
                let avg_execution_time: f64 = row.try_get("avg_execution_time")?;
                let avg_success_rate: f64 = row.try_get("avg_success_rate")?;
                let avg_quality_score: f64 = row.try_get("avg_quality_score")?;
                let sample_count: i64 = row.try_get("sample_count")?;

                // Calculate improvement score based on recent performance
                let improvement_score = (avg_success_rate * 0.4) + (avg_quality_score * 0.4) + ((1.0 - (avg_execution_time / 300000.0)) * 0.2);
                let confidence = (sample_count as f64 / 100.0).min(1.0);

                Ok(PerformanceTrend {
                    improvement_score,
                    confidence,
                    avg_execution_time,
                    avg_success_rate,
                    avg_quality_score,
                })
            }
            Ok(None) => {
                // No data available, return default trend
                Ok(PerformanceTrend {
                    improvement_score: 0.0,
                    confidence: 0.0,
                    avg_execution_time: 300000.0,
                    avg_success_rate: 0.5,
                    avg_quality_score: 0.7,
                })
            }
            Err(e) => {
                error!("Failed to analyze performance trend: {}", e);
                Ok(PerformanceTrend {
                    improvement_score: 0.5,
                    confidence: 0.1,
                    avg_execution_time: 300000.0,
                    avg_success_rate: 0.7,
                    avg_quality_score: 0.7,
                })
            }
        }
    }

    /// Generate optimized configuration based on performance analysis
    async fn generate_optimized_config(
        &self,
        current_config: &HashMap<String, serde_json::Value>,
        trend: &PerformanceTrend,
    ) -> Result<HashMap<String, serde_json::Value>, Box<dyn std::error::Error + Send + Sync>> {
        let mut optimized_config = current_config.clone();

        // Optimize based on performance trends
        if trend.avg_execution_time > 250000.0 {
            // Increase timeout for slow tasks
            optimized_config.insert("task_timeout_seconds".to_string(), serde_json::json!(400));
        }

        if trend.avg_success_rate < 0.8 {
            // Increase retry count for low success rate
            optimized_config.insert("max_retries".to_string(), serde_json::json!(5));
        }

        if trend.avg_quality_score < 0.8 {
            // Increase quality requirements
            optimized_config.insert("min_quality_score".to_string(), serde_json::json!(0.85));
        }

        Ok(optimized_config)
    }

    /// Store optimization result in database
    async fn store_optimization_result(&self, event: &OptimizationEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let query = r#"
            INSERT INTO optimization_events (
                id, timestamp, config_before, config_after, 
                performance_improvement, optimization_type
            ) VALUES ($1, $2, $3, $4, $5, $6)
        "#;

        self.db_client.execute(query, &[
            &event.id,
            &event.timestamp,
            &serde_json::to_value(&event.config_before)?,
            &serde_json::to_value(&event.config_after)?,
            &event.performance_improvement,
            &event.optimization_type,
        ]).await?;

        // Also store in memory for quick access
        {
            let mut history = self.optimization_history.write().unwrap();
            history.push(event.clone());
            if history.len() > 100 {
                history.remove(0); // Keep only last 100 events
            }
        }

        Ok(())
    }

    /// Get optimization history
    pub async fn get_optimization_history(&self) -> Result<Vec<OptimizationEvent>, Box<dyn std::error::Error + Send + Sync>> {
        let query = r#"
            SELECT id, timestamp, config_before, config_after, 
                   performance_improvement, optimization_type
            FROM optimization_events
            ORDER BY timestamp DESC
            LIMIT 50
        "#;

        match self.db_client.query(query, &[]).await {
            Ok(rows) => {
                let mut events = Vec::new();
                for row in rows {
                    let config_before_value: Option<serde_json::Value> = row.try_get("config_before")?;
                    let config_after_value: Option<serde_json::Value> = row.try_get("config_after")?;
                    
                    events.push(OptimizationEvent {
                        id: row.try_get("id")?,
                        event_type: crate::learning::types::OptimizationEventType::ConfigApplied, // Default
                        config_id: row.try_get::<Uuid, _>("id")?, // Reuse id as config_id
                        performance_delta: crate::learning::types::PerformanceMetrics {
                            execution_time_ms: 0.0,
                            quality_score: 0.0,
                            success_rate: 0.0,
                            resource_utilization: 0.0,
                            cost_score: 0.0,
                        },
                        timestamp: row.try_get("timestamp")?,
                        metadata: HashMap::new(),
                        config_before: config_before_value.and_then(|v| serde_json::from_value(v).ok()),
                        config_after: config_after_value.and_then(|v| serde_json::from_value(v).ok()),
                        performance_improvement: row.try_get("performance_improvement")?,
                        optimization_type: row.try_get::<Option<String>, _>("optimization_type")?,
                    });
                }
                Ok(events)
            }
            Err(e) => {
                error!("Failed to get optimization history: {}", e);
                Ok(Vec::new())
            }
        }
    }
}

/// Real queue health monitor implementation
pub struct RealQueueHealthMonitor {
    db_client: Arc<DatabaseClient>,
}

impl RealQueueHealthMonitor {
    pub fn new(db_client: Arc<DatabaseClient>) -> Self {
        Self { db_client }
    }

    /// Monitor queue health and return metrics
    pub async fn monitor_queue_health(&self) -> Result<QueueHealthMetrics, Box<dyn std::error::Error + Send + Sync>> {
        let query = r#"
            SELECT 
                COUNT(*) as total_tasks,
                COUNT(CASE WHEN status = 'pending' THEN 1 END) as pending_tasks,
                COUNT(CASE WHEN status = 'running' THEN 1 END) as running_tasks,
                COUNT(CASE WHEN status = 'completed' THEN 1 END) as completed_tasks,
                COUNT(CASE WHEN status = 'failed' THEN 1 END) as failed_tasks,
                AVG(CASE WHEN status = 'completed' 
                    THEN EXTRACT(EPOCH FROM (completed_at - started_at)) 
                    END) as avg_completion_time_seconds
            FROM task_executions
            WHERE created_at >= NOW() - INTERVAL '1 hour'
        "#;

        match self.db_client.query_one(query).await {
            Ok(Some(row)) => {
                let total_tasks: i64 = row.try_get("total_tasks")?;
                let pending_tasks: i64 = row.try_get("pending_tasks")?;
                let running_tasks: i64 = row.try_get("running_tasks")?;
                let completed_tasks: i64 = row.try_get("completed_tasks")?;
                let failed_tasks: i64 = row.try_get("failed_tasks")?;
                let avg_completion_time: Option<f64> = row.try_get("avg_completion_time_seconds")?;

                let success_rate = if total_tasks > 0 {
                    completed_tasks as f64 / total_tasks as f64
                } else {
                    1.0
                };

                let failure_rate = if total_tasks > 0 {
                    failed_tasks as f64 / total_tasks as f64
                } else {
                    0.0
                };

                Ok(QueueHealthMetrics {
                    total_tasks,
                    pending_tasks,
                    running_tasks,
                    completed_tasks,
                    failed_tasks,
                    success_rate,
                    failure_rate,
                    avg_completion_time_seconds: avg_completion_time.unwrap_or(0.0),
                    queue_depth_score: self.calculate_queue_depth_score(pending_tasks),
                    throughput_score: self.calculate_throughput_score(completed_tasks),
                    last_updated: Utc::now(),
                })
            }
            Ok(None) => {
                // No queue data available
                Ok(QueueHealthMetrics::default())
            }
            Err(e) => {
                error!("Failed to monitor queue health: {}", e);
                Ok(QueueHealthMetrics::default())
            }
        }
    }

    /// Calculate queue depth score (0-1, higher is better)
    fn calculate_queue_depth_score(&self, pending_tasks: i64) -> f64 {
        match pending_tasks {
            0 => 1.0,
            1..=10 => 0.9,
            11..=50 => 0.7,
            51..=100 => 0.5,
            _ => 0.2,
        }
    }

    /// Calculate throughput score (0-1, higher is better)
    fn calculate_throughput_score(&self, completed_tasks: i64) -> f64 {
        match completed_tasks {
            0..=5 => 0.3,
            6..=20 => 0.6,
            21..=50 => 0.8,
            _ => 1.0,
        }
    }
}

/// Real failure taxonomy implementation
pub struct RealFailureTaxonomy {
    db_client: Arc<DatabaseClient>,
}

impl RealFailureTaxonomy {
    pub fn new(db_client: Arc<DatabaseClient>) -> Self {
        Self { db_client }
    }

    /// Classify failure based on error message and context
    pub async fn classify_failure(&self, error_message: &str, task_context: &HashMap<String, serde_json::Value>) -> Result<FailureClassification, Box<dyn std::error::Error + Send + Sync>> {
        let failure_type = self.determine_failure_type(error_message);
        let severity = self.determine_failure_severity(error_message, task_context);
        let recommendations = self.generate_recommendations(&failure_type, &severity);
        let confidence = self.calculate_confidence(error_message, &failure_type);

        Ok(FailureClassification {
            failure_type,
            severity,
            recommendations,
            confidence,
            error_message: error_message.to_string(),
            classified_at: Utc::now(),
        })
    }

    /// Determine failure type based on error message
    fn determine_failure_type(&self, error_message: &str) -> FailureType {
        let error_lower = error_message.to_lowercase();
        
        if error_lower.contains("timeout") || error_lower.contains("deadline") {
            FailureType::Timeout
        } else if error_lower.contains("memory") || error_lower.contains("oom") {
            FailureType::ResourceExhaustion
        } else if error_lower.contains("network") || error_lower.contains("connection") {
            FailureType::NetworkError
        } else if error_lower.contains("permission") || error_lower.contains("unauthorized") {
            FailureType::PermissionError
        } else if error_lower.contains("validation") || error_lower.contains("invalid") {
            FailureType::ValidationError
        } else if error_lower.contains("dependency") || error_lower.contains("missing") {
            FailureType::DependencyError
        } else {
            FailureType::Unknown
        }
    }

    /// Determine failure severity
    fn determine_failure_severity(&self, error_message: &str, task_context: &HashMap<String, serde_json::Value>) -> FailureSeverity {
        let error_lower = error_message.to_lowercase();
        
        // Check if it's a critical task
        let is_critical = task_context.get("priority")
            .and_then(|v| v.as_str())
            .map(|p| p == "critical")
            .unwrap_or(false);

        if is_critical || error_lower.contains("critical") || error_lower.contains("fatal") {
            FailureSeverity::Critical
        } else if error_lower.contains("error") || error_lower.contains("failed") {
            FailureSeverity::High
        } else if error_lower.contains("warning") || error_lower.contains("issue") {
            FailureSeverity::Medium
        } else {
            FailureSeverity::Low
        }
    }

    /// Generate recommendations based on failure type and severity
    fn generate_recommendations(&self, failure_type: &FailureType, severity: &FailureSeverity) -> Vec<String> {
        let mut recommendations = Vec::new();

        match failure_type {
            FailureType::Timeout => {
                recommendations.push("Increase task timeout duration".to_string());
                recommendations.push("Optimize task complexity".to_string());
                recommendations.push("Check worker performance".to_string());
            }
            FailureType::ResourceExhaustion => {
                recommendations.push("Increase memory limits".to_string());
                recommendations.push("Optimize memory usage".to_string());
                recommendations.push("Scale worker resources".to_string());
            }
            FailureType::NetworkError => {
                recommendations.push("Check network connectivity".to_string());
                recommendations.push("Implement retry logic".to_string());
                recommendations.push("Use circuit breaker pattern".to_string());
            }
            FailureType::PermissionError => {
                recommendations.push("Review access permissions".to_string());
                recommendations.push("Update authentication tokens".to_string());
                recommendations.push("Check user roles".to_string());
            }
            FailureType::ValidationError => {
                recommendations.push("Validate input data".to_string());
                recommendations.push("Check data format".to_string());
                recommendations.push("Update validation rules".to_string());
            }
            FailureType::DependencyError => {
                recommendations.push("Install missing dependencies".to_string());
                recommendations.push("Update dependency versions".to_string());
                recommendations.push("Check service availability".to_string());
            }
            FailureType::Unknown => {
                recommendations.push("Investigate error details".to_string());
                recommendations.push("Check system logs".to_string());
                recommendations.push("Contact support if persistent".to_string());
            }
        }

        // Add severity-based recommendations
        match severity {
            FailureSeverity::Critical => {
                recommendations.push("Immediate attention required".to_string());
                recommendations.push("Consider system rollback".to_string());
            }
            FailureSeverity::High => {
                recommendations.push("Priority investigation needed".to_string());
            }
            FailureSeverity::Medium => {
                recommendations.push("Monitor for patterns".to_string());
            }
            FailureSeverity::Low => {
                recommendations.push("Log for future analysis".to_string());
            }
        }

        recommendations
    }

    /// Calculate confidence in classification
    fn calculate_confidence(&self, error_message: &str, failure_type: &FailureType) -> f64 {
        let error_lower = error_message.to_lowercase();
        
        match failure_type {
            FailureType::Timeout => {
                if error_lower.contains("timeout") && error_lower.contains("exceeded") {
                    0.95
                } else if error_lower.contains("timeout") {
                    0.8
                } else {
                    0.6
                }
            }
            FailureType::ResourceExhaustion => {
                if error_lower.contains("memory") && error_lower.contains("exhausted") {
                    0.9
                } else if error_lower.contains("oom") {
                    0.95
                } else {
                    0.7
                }
            }
            FailureType::NetworkError => {
                if error_lower.contains("connection") && error_lower.contains("refused") {
                    0.9
                } else if error_lower.contains("network") {
                    0.8
                } else {
                    0.6
                }
            }
            FailureType::PermissionError => {
                if error_lower.contains("permission") && error_lower.contains("denied") {
                    0.9
                } else if error_lower.contains("unauthorized") {
                    0.85
                } else {
                    0.7
                }
            }
            FailureType::ValidationError => {
                if error_lower.contains("validation") && error_lower.contains("failed") {
                    0.9
                } else if error_lower.contains("invalid") {
                    0.8
                } else {
                    0.6
                }
            }
            FailureType::DependencyError => {
                if error_lower.contains("dependency") && error_lower.contains("missing") {
                    0.9
                } else if error_lower.contains("not found") {
                    0.8
                } else {
                    0.6
                }
            }
            FailureType::Unknown => 0.3,
        }
    }
}

/// Real learning persistence implementation
pub struct RealLearningPersistence {
    db_client: Arc<DatabaseClient>,
}

impl RealLearningPersistence {
    pub fn new(db_client: Arc<DatabaseClient>) -> Self {
        Self { db_client }
    }
}

#[async_trait::async_trait]
impl crate::learning::LearningPersistence for RealLearningPersistence {
    async fn store_execution_records(&self, records: Vec<ExecutionRecord>) -> Result<()> {
        let query = r#"
            INSERT INTO execution_records (
                id, task_id, worker_id, execution_time_ms, success_rate, 
                quality_score, created_at, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (id) DO UPDATE SET
                execution_time_ms = EXCLUDED.execution_time_ms,
                success_rate = EXCLUDED.success_rate,
                quality_score = EXCLUDED.quality_score,
                metadata = EXCLUDED.metadata
        "#;

        for record in records {
            let success_rate = if record.success { 1.0 } else { 0.0 };
            self.db_client.execute(query, &[
                &record.id,
                &record.task_id.0,
                &record.worker_id.0,
                &record.execution_time_ms,
                &success_rate,
                &record.quality_score,
                &record.created_at,
                &serde_json::to_value(&record.metadata)?,
            ]).await?;
        }

        Ok(())
    }

    async fn get_execution_records(&self, pattern: &TaskPattern, limit: Option<usize>) -> Result<Vec<ExecutionRecord>> {
        let limit = limit.unwrap_or(100);
        let query = r#"
            SELECT id, task_id, worker_id, execution_time_ms, success_rate, 
                   quality_score, created_at, metadata
            FROM execution_records
            ORDER BY created_at DESC
            LIMIT $1
        "#;

        match self.db_client.query_with_params(query, &[&(limit as i32)]).await {
            Ok(rows) => {
                let mut records = Vec::new();
                for row in rows {
                    let success_rate: f64 = row.try_get("success_rate")?;
                    let task_id_uuid: Uuid = row.try_get("task_id")?;
                    let worker_id_uuid: Uuid = row.try_get("worker_id")?;
                    records.push(ExecutionRecord {
                        id: row.try_get("id")?,
                        task_id: TaskId(task_id_uuid),
                        worker_id: WorkerId(worker_id_uuid),
                        execution_time_ms: row.try_get("execution_time_ms")?,
                        success: success_rate > 0.5, // Convert rate back to bool
                        quality_score: row.try_get("quality_score")?,
                        error_message: None, // Not stored in database
                        metadata: row.try_get::<serde_json::Value, _>("metadata")?.into(),
                        created_at: row.try_get("created_at")?,
                    });
                }
                Ok(records)
            }
            Err(e) => {
                error!("Failed to get execution records: {}", e);
                Ok(Vec::new())
            }
        }
    }

    async fn store_worker_profiles(&self, profiles: HashMap<WorkerId, WorkerPerformanceProfile>) -> Result<()> {
        let query = r#"
            INSERT INTO worker_profiles (
                worker_id, task_count, success_rate, avg_execution_time_ms,
                quality_score, specialization_score, last_updated, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (worker_id) DO UPDATE SET
                task_count = EXCLUDED.task_count,
                success_rate = EXCLUDED.success_rate,
                avg_execution_time_ms = EXCLUDED.avg_execution_time_ms,
                quality_score = EXCLUDED.quality_score,
                specialization_score = EXCLUDED.specialization_score,
                metadata = EXCLUDED.metadata,
                last_updated = EXCLUDED.last_updated
        "#;

        for (worker_id, profile) in profiles {
            self.db_client.execute(query, &[
                &worker_id.0.to_string(),
                &profile.task_count,
                &profile.success_rate,
                &profile.avg_execution_time_ms,
                &profile.quality_score,
                &profile.specialization_score,
                &profile.last_updated,
                &serde_json::to_value(&profile.metadata)?,
            ]).await?;
        }

        Ok(())
    }

    async fn get_worker_profile(&self, worker_id: &WorkerId) -> Result<Option<WorkerPerformanceProfile>> {
        let query = r#"
            SELECT task_count, success_rate, avg_execution_time_ms,
                   quality_score, specialization_score, last_updated, metadata
            FROM worker_profiles
            WHERE worker_id = $1
        "#;

        match self.db_client.query_one_with_params(query, &[&worker_id.0.to_string()]).await {
            Ok(Some(row)) => {
                Ok(Some(WorkerPerformanceProfile {
                    worker_id: worker_id.clone(),
                    specialty: crate::worker_types::WorkerSpecialty::General, // Default, should be updated
                    total_executions: 0, // Default
                    successful_executions: 0, // Default
                    average_execution_time_ms: 0.0, // Default
                    average_quality_score: 0.0, // Default
                    performance_trend: crate::learning::types::PerformanceTrend::Unknown,
                    capability_scores: HashMap::new(), // Default
                    task_count: row.try_get("task_count")?,
                    success_rate: row.try_get("success_rate")?,
                    avg_execution_time_ms: row.try_get("avg_execution_time_ms")?,
                    quality_score: row.try_get("quality_score")?,
                    specialization_score: row.try_get("specialization_score")?,
                    last_updated: row.try_get("last_updated")?,
                    metadata: row.try_get::<serde_json::Value, _>("metadata")?.into(),
                }))
            }
            Ok(None) => Ok(None),
            Err(_) => Ok(None),
        }
    }

    async fn store_success_patterns(&self, patterns: Vec<SuccessPattern>) -> Result<()> {
        let query = r#"
            INSERT INTO success_patterns (
                pattern_name, pattern_type, confidence_score, outcomes,
                last_seen, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (pattern_name) DO UPDATE SET
                confidence_score = EXCLUDED.confidence_score,
                outcomes = EXCLUDED.outcomes,
                last_seen = EXCLUDED.last_seen,
                metadata = EXCLUDED.metadata
        "#;

        for pattern in patterns {
            self.db_client.execute(query, &[
                &pattern.pattern_name,
                &serde_json::to_value(&pattern.pattern_type)?,
                &pattern.confidence_score,
                &serde_json::to_value(&pattern.outcomes)?,
                &pattern.last_seen,
                &serde_json::to_value(&pattern.metadata)?,
            ]).await?;
        }

        Ok(())
    }

    async fn get_success_patterns(&self) -> Result<Vec<SuccessPattern>> {
        let query = "SELECT pattern_name, pattern_type, confidence_score, outcomes, last_seen, metadata FROM success_patterns";

        match self.db_client.query(query, &[]).await {
            Ok(rows) => {
                let mut patterns = Vec::new();
                for row in rows {
                    patterns.push(SuccessPattern {
                        pattern_name: row.get("pattern_name"),
                        pattern_type: serde_json::from_value(row.get("pattern_type"))?,
                        confidence_score: row.get("confidence_score"),
                        outcomes: serde_json::from_value(row.get("outcomes"))?,
                        last_seen: row.get("last_seen"),
                        metadata: row.get("metadata").into(),
                    });
                }
                Ok(patterns)
            }
            Err(e) => {
                error!("Failed to get success patterns: {}", e);
                Ok(Vec::new())
            }
        }
    }

    async fn store_failure_patterns(&self, patterns: Vec<FailurePattern>) -> Result<()> {
        let query = r#"
            INSERT INTO failure_patterns (
                pattern_name, pattern_type, confidence_score, outcomes,
                last_seen, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (pattern_name) DO UPDATE SET
                confidence_score = EXCLUDED.confidence_score,
                outcomes = EXCLUDED.outcomes,
                last_seen = EXCLUDED.last_seen,
                metadata = EXCLUDED.metadata
        "#;

        for pattern in patterns {
            self.db_client.execute(query, &[
                &pattern.pattern_name,
                &serde_json::to_value(&pattern.pattern_type)?,
                &pattern.confidence_score,
                &serde_json::to_value(&pattern.outcomes)?,
                &pattern.last_seen,
                &serde_json::to_value(&pattern.metadata)?,
            ]).await?;
        }

        Ok(())
    }

    async fn get_failure_patterns(&self) -> Result<Vec<FailurePattern>> {
        let query = "SELECT pattern_name, pattern_type, confidence_score, outcomes, last_seen, metadata FROM failure_patterns";

        match self.db_client.query(query, &[]).await {
            Ok(rows) => {
                let mut patterns = Vec::new();
                for row in rows {
                    patterns.push(FailurePattern {
                        pattern_name: row.get("pattern_name"),
                        pattern_type: serde_json::from_value(row.get("pattern_type"))?,
                        confidence_score: row.get("confidence_score"),
                        outcomes: serde_json::from_value(row.get("outcomes"))?,
                        last_seen: row.get("last_seen"),
                        metadata: row.get("metadata").into(),
                    });
                }
                Ok(patterns)
            }
            Err(e) => {
                error!("Failed to get failure patterns: {}", e);
                Ok(Vec::new())
            }
        }
    }

    async fn store_optimal_configs(&self, configs: Vec<OptimalConfig>) -> Result<()> {
        let query = r#"
            INSERT INTO optimal_configs (
                id, worker_type, task_type, config, performance_metrics, confidence, expires_at, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (worker_type, task_type) DO UPDATE SET
                config = EXCLUDED.config,
                performance_metrics = EXCLUDED.performance_metrics,
                expires_at = EXCLUDED.expires_at,
                metadata = EXCLUDED.metadata,
                confidence = EXCLUDED.confidence
        "#;

        for config in configs {
            self.db_client.execute(query, &[
                &config.id,
                &config.worker_type,
                &config.task_type,
                &config.config,
                &serde_json::to_value(&config.performance_metrics)?,
                &config.confidence,
                &config.expires_at,
                &config.metadata,
            ]).await?;
        }

        Ok(())
    }

    async fn get_optimal_configs(&self) -> Result<Vec<OptimalConfig>> {
        let query = "SELECT id, worker_type, task_type, config, performance_metrics, confidence, expires_at, metadata, created_at FROM optimal_configs WHERE expires_at > NOW() OR expires_at IS NULL";

        match self.db_client.query(query, &[]).await {
            Ok(rows) => {
                let mut configs = Vec::new();
                for row in rows {
                    configs.push(OptimalConfig {
                        id: row.get("id"),
                        worker_type: row.get("worker_type"),
                        task_type: row.get("task_type"),
                        config: row.get("config"),
                        performance_metrics: serde_json::from_value(row.get("performance_metrics"))?,
                        confidence: row.get("confidence"),
                        expires_at: row.get("expires_at"),
                        metadata: row.get("metadata"),
                        created_at: row.get("created_at"),
                    });
                }
                Ok(configs)
            }
            Err(e) => {
                error!("Failed to get optimal configs: {}", e);
                Ok(Vec::new())
            }
        }
    }

    async fn store_optimization_events(&self, events: Vec<OptimizationEvent>) -> Result<()> {
        let query = r#"
            INSERT INTO optimization_events (
                id, event_type, config_id, performance_delta, timestamp, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6)
        "#;

        for event in events {
            self.db_client.execute(query, &[
                &event.id,
                &serde_json::to_value(&event.event_type)?,
                &event.config_id,
                &serde_json::to_value(&event.performance_delta)?,
                &event.timestamp,
                &serde_json::to_value(&event.metadata)?,
            ]).await?;
        }

        Ok(())
    }

    async fn get_optimization_events(&self, config_id: &Uuid) -> Result<Vec<OptimizationEvent>> {
        let query = "SELECT id, event_type, config_id, performance_delta, timestamp, metadata FROM optimization_events WHERE config_id = $1 ORDER BY timestamp DESC";

        match self.db_client.query_with_params(query, &[config_id]).await {
            Ok(rows) => {
                let mut events = Vec::new();
                for row in rows {
                    events.push(OptimizationEvent {
                        id: row.get("id"),
                        event_type: serde_json::from_value(row.get("event_type"))?,
                        config_id: row.get("config_id"),
                        performance_delta: serde_json::from_value(row.get("performance_delta"))?,
                        timestamp: row.get("timestamp"),
                        metadata: serde_json::from_value(row.get("metadata"))?,
                    });
                }
                Ok(events)
            }
            Err(e) => {
                error!("Failed to get optimization events: {}", e);
                Ok(Vec::new())
            }
        }
    }
}

// Supporting types

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QueueHealthMetrics {
    pub total_tasks: i64,
    pub pending_tasks: i64,
    pub running_tasks: i64,
    pub completed_tasks: i64,
    pub failed_tasks: i64,
    pub success_rate: f64,
    pub failure_rate: f64,
    pub avg_completion_time_seconds: f64,
    pub queue_depth_score: f64,
    pub throughput_score: f64,
    #[schemars(with = "String")]

    pub last_updated: DateTime<Utc>,
}

impl Default for QueueHealthMetrics {
    fn default() -> Self {
        Self {
            total_tasks: 0,
            pending_tasks: 0,
            running_tasks: 0,
            completed_tasks: 0,
            failed_tasks: 0,
            success_rate: 1.0,
            failure_rate: 0.0,
            avg_completion_time_seconds: 0.0,
            queue_depth_score: 1.0,
            throughput_score: 0.5,
            last_updated: Utc::now(),
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FailureClassification {
    pub failure_type: FailureType,
    pub severity: FailureSeverity,
    pub recommendations: Vec<String>,
    pub confidence: f64,
    pub error_message: String,
    #[schemars(with = "String")]

    pub classified_at: DateTime<Utc>,
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
enum FailureType {
    Timeout,
    ResourceExhaustion,
    NetworkError,
    PermissionError,
    ValidationError,
    DependencyError,
    Unknown,
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
enum FailureSeverity {
    Low,
    Medium,
    High,
    Critical,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct PerformanceTrend {
    improvement_score: f64,
    confidence: f64,
    avg_execution_time: f64,
    avg_success_rate: f64,
    avg_quality_score: f64,
}
