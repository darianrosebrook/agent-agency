/// Performance-based router for model hot-swapping
///
/// Routes requests to model instances based on real-time performance metrics
/// and load balancing requirements.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Performance-based router for intelligent model selection
pub struct PerformanceRouter {
    /// Performance metrics for each model
    performance_metrics: Arc<RwLock<HashMap<String, ModelMetrics>>>,
    /// Routing policies
    routing_policies: Arc<RwLock<Vec<RoutingPolicy>>>,
    /// Performance thresholds
    thresholds: PerformanceThresholds,
    /// Routing history for analytics
    routing_history: Arc<RwLock<Vec<RoutingDecision>>>,
}

/// Performance metrics for a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetrics {
    /// Model identifier
    pub model_id: String,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f32,
    /// 95th percentile response time
    pub p95_response_time_ms: f32,
    /// Throughput in requests per second
    pub throughput_rps: f32,
    /// Error rate (0.0-1.0)
    pub error_rate: f32,
    /// CPU utilization percentage
    pub cpu_utilization: f32,
    /// Memory utilization percentage
    pub memory_utilization: f32,
    /// Model accuracy score (0.0-1.0)
    pub accuracy_score: f32,
    /// Last updated timestamp
    pub last_updated: chrono::DateTime<chrono::Utc>,
    /// Sample count for metrics
    pub sample_count: usize,
}

/// Routing policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingPolicy {
    /// Policy name
    pub name: String,
    /// Policy priority (higher = more important)
    pub priority: u8,
    /// Policy condition
    pub condition: RoutingCondition,
    /// Action to take when condition matches
    pub action: RoutingAction,
    /// Policy enabled status
    pub enabled: bool,
}

/// Routing condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoutingCondition {
    /// Route based on response time threshold
    ResponseTimeBelow { threshold_ms: f32 },
    ResponseTimeAbove { threshold_ms: f32 },
    /// Route based on error rate threshold
    ErrorRateBelow { threshold: f32 },
    ErrorRateAbove { threshold: f32 },
    /// Route based on throughput capacity
    ThroughputAbove { threshold_rps: f32 },
    /// Route based on resource utilization
    CpuUtilizationBelow { threshold_percent: f32 },
    MemoryUtilizationBelow { threshold_percent: f32 },
    /// Route based on accuracy requirements
    AccuracyAbove { threshold: f32 },
    /// Custom condition (placeholder)
    Custom { condition_name: String },
}

/// Routing action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoutingAction {
    /// Route to specific model
    RouteToModel { model_id: String },
    /// Route to fastest available model
    RouteToFastest,
    /// Route to most accurate model
    RouteToMostAccurate,
    /// Route to least loaded model
    RouteToLeastLoaded,
    /// Reject request
    RejectRequest { reason: String },
}

/// Performance thresholds for routing decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceThresholds {
    /// Maximum acceptable response time in ms
    pub max_response_time_ms: f32,
    /// Maximum acceptable error rate
    pub max_error_rate: f32,
    /// Minimum required accuracy
    pub min_accuracy: f32,
    /// Maximum CPU utilization before routing away
    pub max_cpu_utilization: f32,
    /// Maximum memory utilization before routing away
    pub max_memory_utilization: f32,
}

/// Routing decision record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    /// Request identifier
    pub request_id: String,
    /// Selected model
    pub selected_model: String,
    /// Routing reason
    pub reason: String,
    /// Confidence in decision (0.0-1.0)
    pub confidence: f32,
    /// Timestamp of decision
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Request context
    pub context: RequestContext,
}

/// Request context for routing decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestContext {
    /// Required accuracy level
    pub accuracy_requirement: Option<f32>,
    /// Maximum acceptable latency
    pub max_latency_ms: Option<f32>,
    /// Priority level
    pub priority: u8,
    /// Request size/complexity hint
    pub complexity_hint: RequestComplexity,
}

/// Request complexity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestComplexity {
    Low,
    Medium,
    High,
    Critical,
}

impl PerformanceRouter {
    /// Create a new performance router with default policies
    pub fn new() -> Self {
        let thresholds = PerformanceThresholds {
            max_response_time_ms: 1000.0,
            max_error_rate: 0.05,
            min_accuracy: 0.8,
            max_cpu_utilization: 80.0,
            max_memory_utilization: 85.0,
        };

        let default_policies = vec![
            RoutingPolicy {
                name: "latency_critical".to_string(),
                priority: 10,
                condition: RoutingCondition::ResponseTimeBelow { threshold_ms: 100.0 },
                action: RoutingAction::RouteToFastest,
                enabled: true,
            },
            RoutingPolicy {
                name: "high_accuracy_required".to_string(),
                priority: 9,
                condition: RoutingCondition::AccuracyAbove { threshold: 0.95 },
                action: RoutingAction::RouteToMostAccurate,
                enabled: true,
            },
            RoutingPolicy {
                name: "overload_protection".to_string(),
                priority: 8,
                condition: RoutingCondition::CpuUtilizationBelow { threshold_percent: 80.0 },
                action: RoutingAction::RouteToLeastLoaded,
                enabled: true,
            },
            RoutingPolicy {
                name: "fallback".to_string(),
                priority: 1,
                condition: RoutingCondition::ErrorRateBelow { threshold: 0.1 },
                action: RoutingAction::RouteToLeastLoaded,
                enabled: true,
            },
        ];

        Self {
            performance_metrics: Arc::new(RwLock::new(HashMap::new())),
            routing_policies: Arc::new(RwLock::new(default_policies)),
            thresholds,
            routing_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Route a request based on performance metrics and policies
    pub async fn route_request(&self, request: &RoutingRequest) -> Result<RoutingDecision> {
        let metrics = self.performance_metrics.read().await;
        let policies = self.routing_policies.read().await;

        // Evaluate policies in priority order
        for policy in policies.iter().filter(|p| p.enabled).collect::<Vec<_>>() {
            if self.evaluate_condition(&policy.condition, &metrics, request).await {
                let action_result = self.execute_action(&policy.action, &metrics, request).await?;

                let decision = RoutingDecision {
                    request_id: request.id.clone(),
                    selected_model: action_result.model_id,
                    reason: format!("Policy '{}': {:?}", policy.name, policy.condition),
                    confidence: action_result.confidence,
                    timestamp: chrono::Utc::now(),
                    context: request.context.clone(),
                };

                // Record decision
                let mut history = self.routing_history.write().await;
                history.push(decision.clone());

                return Ok(decision);
            }
        }

        // Default fallback routing
        self.fallback_routing(&metrics, request).await
    }

    /// Update performance metrics for a model
    pub async fn update_metrics(&self, model_id: &str, new_metrics: ModelMetrics) -> Result<()> {
        let mut metrics = self.performance_metrics.write().await;
        metrics.insert(model_id.to_string(), new_metrics);
        debug!("Updated metrics for model {}", model_id);
        Ok(())
    }

    /// Add a new routing policy
    pub async fn add_policy(&self, policy: RoutingPolicy) -> Result<()> {
        let mut policies = self.routing_policies.write().await;
        policies.push(policy);
        // Sort by priority (highest first)
        policies.sort_by(|a, b| b.priority.cmp(&a.priority));
        Ok(())
    }

    /// Remove a routing policy
    pub async fn remove_policy(&self, policy_name: &str) -> Result<()> {
        let mut policies = self.routing_policies.write().await;
        policies.retain(|p| p.name != policy_name);
        Ok(())
    }

    /// Evaluate a routing condition
    async fn evaluate_condition(
        &self,
        condition: &RoutingCondition,
        metrics: &HashMap<String, ModelMetrics>,
        request: &RoutingRequest,
    ) -> bool {
        match condition {
            RoutingCondition::ResponseTimeBelow { threshold_ms } => {
                self.check_response_time_condition(metrics, *threshold_ms, false)
            }
            RoutingCondition::ResponseTimeAbove { threshold_ms } => {
                self.check_response_time_condition(metrics, *threshold_ms, true)
            }
            RoutingCondition::ErrorRateBelow { threshold } => {
                self.check_error_rate_condition(metrics, *threshold, false)
            }
            RoutingCondition::ErrorRateAbove { threshold } => {
                self.check_error_rate_condition(metrics, *threshold, true)
            }
            RoutingCondition::ThroughputAbove { threshold_rps } => {
                self.check_throughput_condition(metrics, *threshold_rps)
            }
            RoutingCondition::CpuUtilizationBelow { threshold_percent } => {
                self.check_cpu_condition(metrics, *threshold_percent, false)
            }
            RoutingCondition::MemoryUtilizationBelow { threshold_percent } => {
                self.check_memory_condition(metrics, *threshold_percent, false)
            }
            RoutingCondition::AccuracyAbove { threshold } => {
                self.check_accuracy_condition(metrics, *threshold, request)
            }
            RoutingCondition::Custom { condition_name } => {
                // Custom conditions would be implemented here
                warn!("Custom condition '{}' not implemented", condition_name);
                false
            }
        }
    }

    /// Execute a routing action
    async fn execute_action(
        &self,
        action: &RoutingAction,
        metrics: &HashMap<String, ModelMetrics>,
        request: &RoutingRequest,
    ) -> Result<ActionResult> {
        match action {
            RoutingAction::RouteToModel { model_id } => {
                if metrics.contains_key(model_id) {
                    Ok(ActionResult {
                        model_id: model_id.clone(),
                        confidence: 0.9,
                    })
                } else {
                    Err(anyhow::anyhow!("Model {} not found in metrics", model_id))
                }
            }
            RoutingAction::RouteToFastest => {
                self.find_fastest_model(metrics)
            }
            RoutingAction::RouteToMostAccurate => {
                self.find_most_accurate_model(metrics, request)
            }
            RoutingAction::RouteToLeastLoaded => {
                self.find_least_loaded_model(metrics)
            }
            RoutingAction::RejectRequest { reason } => {
                Err(anyhow::anyhow!("Request rejected: {}", reason))
            }
        }
    }

    /// Fallback routing when no policies match
    async fn fallback_routing(
        &self,
        metrics: &HashMap<String, ModelMetrics>,
        request: &RoutingRequest,
    ) -> Result<RoutingDecision> {
        // Find the best available model based on overall performance
        let best_model = self.find_best_overall_model(metrics)?;

        Ok(RoutingDecision {
            request_id: request.id.clone(),
            selected_model: best_model,
            reason: "Fallback routing - best available model".to_string(),
            confidence: 0.7,
            timestamp: chrono::Utc::now(),
            context: request.context.clone(),
        })
    }

    // Helper methods for condition checking
    fn check_response_time_condition(&self, metrics: &HashMap<String, ModelMetrics>, threshold: f32, above: bool) -> bool {
        metrics.values().any(|m| {
            if above {
                m.avg_response_time_ms > threshold
            } else {
                m.avg_response_time_ms < threshold
            }
        })
    }

    fn check_error_rate_condition(&self, metrics: &HashMap<String, ModelMetrics>, threshold: f32, above: bool) -> bool {
        metrics.values().any(|m| {
            if above {
                m.error_rate > threshold
            } else {
                m.error_rate < threshold
            }
        })
    }

    fn check_throughput_condition(&self, metrics: &HashMap<String, ModelMetrics>, threshold: f32) -> bool {
        metrics.values().any(|m| m.throughput_rps > threshold)
    }

    fn check_cpu_condition(&self, metrics: &HashMap<String, ModelMetrics>, threshold: f32, above: bool) -> bool {
        metrics.values().any(|m| {
            if above {
                m.cpu_utilization > threshold
            } else {
                m.cpu_utilization < threshold
            }
        })
    }

    fn check_memory_condition(&self, metrics: &HashMap<String, ModelMetrics>, threshold: f32, above: bool) -> bool {
        metrics.values().any(|m| {
            if above {
                m.memory_utilization > threshold
            } else {
                m.memory_utilization < threshold
            }
        })
    }

    fn check_accuracy_condition(&self, metrics: &HashMap<String, ModelMetrics>, threshold: f32, request: &RoutingRequest) -> bool {
        if let Some(required_accuracy) = request.context.accuracy_requirement {
            metrics.values().any(|m| m.accuracy_score >= required_accuracy.max(threshold))
        } else {
            metrics.values().any(|m| m.accuracy_score >= threshold)
        }
    }

    // Helper methods for finding best models
    fn find_fastest_model(&self, metrics: &HashMap<String, ModelMetrics>) -> Result<ActionResult> {
        let fastest = metrics.values()
            .min_by(|a, b| a.avg_response_time_ms.partial_cmp(&b.avg_response_time_ms).unwrap())
            .ok_or_else(|| anyhow::anyhow!("No models available"))?;

        Ok(ActionResult {
            model_id: fastest.model_id.clone(),
            confidence: 0.85,
        })
    }

    fn find_most_accurate_model(&self, metrics: &HashMap<String, ModelMetrics>, request: &RoutingRequest) -> Result<ActionResult> {
        let min_accuracy = request.context.accuracy_requirement.unwrap_or(self.thresholds.min_accuracy);

        let most_accurate = metrics.values()
            .filter(|m| m.accuracy_score >= min_accuracy)
            .max_by(|a, b| a.accuracy_score.partial_cmp(&b.accuracy_score).unwrap())
            .ok_or_else(|| anyhow::anyhow!("No sufficiently accurate models available"))?;

        Ok(ActionResult {
            model_id: most_accurate.model_id.clone(),
            confidence: 0.9,
        })
    }

    fn find_least_loaded_model(&self, metrics: &HashMap<String, ModelMetrics>) -> Result<ActionResult> {
        let least_loaded = metrics.values()
            .min_by(|a, b| {
                let a_load = (a.cpu_utilization + a.memory_utilization) / 2.0;
                let b_load = (b.cpu_utilization + b.memory_utilization) / 2.0;
                a_load.partial_cmp(&b_load).unwrap()
            })
            .ok_or_else(|| anyhow::anyhow!("No models available"))?;

        Ok(ActionResult {
            model_id: least_loaded.model_id.clone(),
            confidence: 0.8,
        })
    }

    fn find_best_overall_model(&self, metrics: &HashMap<String, ModelMetrics>) -> Result<String> {
        // Simple scoring function combining multiple metrics
        let best = metrics.values()
            .max_by(|a, b| {
                let a_score = self.calculate_overall_score(a);
                let b_score = self.calculate_overall_score(b);
                a_score.partial_cmp(&b_score).unwrap()
            })
            .ok_or_else(|| anyhow::anyhow!("No models available"))?;

        Ok(best.model_id.clone())
    }

    fn calculate_overall_score(&self, metrics: &ModelMetrics) -> f32 {
        // Normalize and combine metrics into a single score
        let latency_score = 1.0 - (metrics.avg_response_time_ms / self.thresholds.max_response_time_ms).min(1.0);
        let accuracy_score = metrics.accuracy_score;
        let reliability_score = 1.0 - metrics.error_rate;
        let resource_score = 1.0 - ((metrics.cpu_utilization + metrics.memory_utilization) / 200.0).min(1.0);

        // Weighted combination
        (latency_score * 0.3) + (accuracy_score * 0.3) + (reliability_score * 0.2) + (resource_score * 0.2)
    }

    /// Get routing statistics
    pub async fn get_statistics(&self) -> RouterStatistics {
        let history = self.routing_history.read().await;
        let metrics = self.performance_metrics.read().await;

        let total_decisions = history.len();
        let avg_confidence = if total_decisions > 0 {
            history.iter().map(|d| d.confidence).sum::<f32>() / total_decisions as f32
        } else {
            0.0
        };

        RouterStatistics {
            total_decisions,
            average_confidence: avg_confidence,
            active_models: metrics.len(),
            policies_active: self.routing_policies.read().await.iter().filter(|p| p.enabled).count(),
        }
    }
}

/// Action execution result
#[derive(Debug)]
struct ActionResult {
    model_id: String,
    confidence: f32,
}

/// Routing request
#[derive(Debug)]
pub struct RoutingRequest {
    pub id: String,
    pub context: RequestContext,
}

/// Router statistics
#[derive(Debug)]
pub struct RouterStatistics {
    pub total_decisions: usize,
    pub average_confidence: f32,
    pub active_models: usize,
    pub policies_active: usize,
}


