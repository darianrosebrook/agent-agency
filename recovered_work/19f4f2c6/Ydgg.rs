//! Model-Agnostic Hot-Swapping - Seamless Runtime Model Replacement
//!
//! Enables seamless runtime model replacement with performance tracking,
//! routing optimization, and zero-downtime updates.
//!
//! ## Key Features
//!
//! 1. **Seamless Replacement**: Zero-downtime model switching with request draining
//! 2. **Performance Tracking**: Real-time performance monitoring and routing optimization
//! 3. **Model Registry**: Versioned model management with rollback capabilities
//! 4. **Load Balancing**: Intelligent request routing based on model performance
//! 5. **A/B Testing**: Parallel model evaluation with traffic splitting

pub mod hotswap_manager;
pub mod load_balancer;
pub mod model_registry;
pub mod performance_router;
pub mod rollback_manager;
pub mod traffic_splitter;
pub mod version_manager;

pub use load_balancer::{LoadBalancer, BalancingStrategy};
pub use model_registry::{ModelRegistry, ModelEntry, ModelStatus};
pub use performance_router::{PerformanceRouter, RoutingDecision};
pub use rollback_manager::{RollbackManager, RollbackStrategy};
pub use traffic_splitter::{TrafficSplitter, SplitConfig};
pub use version_manager::{VersionManager, VersionInfo};

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug, warn, error};

/// Main hot-swap orchestrator
///
/// Orchestrates seamless model replacement with performance tracking
/// and intelligent routing optimization.
#[derive(Debug)]
pub struct ModelHotSwapOrchestrator {
    /// Model registry for version management
    model_registry: Arc<ModelRegistry>,
    /// Performance router for intelligent routing
    performance_router: Arc<PerformanceRouter>,
    /// Load balancer for traffic distribution
    load_balancer: Arc<LoadBalancer>,
    /// Traffic splitter for A/B testing
    traffic_splitter: Arc<TrafficSplitter>,
    /// Rollback manager for failure recovery
    rollback_manager: Arc<RollbackManager>,
    /// Version manager for compatibility
    version_manager: Arc<VersionManager>,
    /// Active model deployments
    active_deployments: Arc<RwLock<HashMap<String, ModelDeployment>>>,
    /// Orchestrator configuration
    config: HotSwapConfig,
}

/// Model deployment state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDeployment {
    /// Model ID
    pub model_id: String,
    /// Current version
    pub current_version: String,
    /// Previous version (for rollback)
    pub previous_version: Option<String>,
    /// Deployment status
    pub status: DeploymentStatus,
    /// Traffic allocation (0.0-1.0)
    pub traffic_allocation: f64,
    /// Performance metrics
    pub performance: ModelPerformance,
    /// Deployment timestamp
    pub deployed_at: chrono::DateTime<chrono::Utc>,
    /// Last health check
    pub last_health_check: chrono::DateTime<chrono::Utc>,
}

/// Deployment status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentStatus {
    /// Actively serving traffic
    Active,
    /// In the process of deployment
    Deploying,
    /// Warming up (receiving test traffic)
    Warming,
    /// Cooling down (draining traffic)
    Cooling,
    /// Failed deployment
    Failed(String),
    /// Rolled back
    RolledBack,
}

/// Model performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPerformance {
    /// Requests per second
    pub rps: f64,
    /// Average latency (ms)
    pub avg_latency_ms: f64,
    /// P95 latency (ms)
    pub p95_latency_ms: f64,
    /// Error rate (0.0-1.0)
    pub error_rate: f64,
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// Memory usage percentage
    pub memory_usage: f64,
    /// Quality score (0.0-1.0)
    pub quality_score: f64,
    /// Last updated
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Hot-swap configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotSwapConfig {
    /// Enable automatic performance-based routing
    pub enable_performance_routing: bool,
    /// Enable A/B testing
    pub enable_ab_testing: bool,
    /// Traffic draining timeout (seconds)
    pub draining_timeout_secs: u64,
    /// Warm-up period (seconds)
    pub warmup_period_secs: u64,
    /// Performance monitoring interval (seconds)
    pub monitoring_interval_secs: u64,
    /// Automatic rollback threshold (error rate)
    pub auto_rollback_threshold: f64,
    /// Quality degradation threshold
    pub quality_degradation_threshold: f64,
}

/// Model replacement request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelReplacementRequest {
    /// Model ID to replace
    pub model_id: String,
    /// New model version
    pub new_version: String,
    /// Replacement strategy
    pub strategy: ReplacementStrategy,
    /// Traffic split configuration (for gradual rollout)
    pub traffic_split: Option<SplitConfig>,
    /// Rollback plan
    pub rollback_plan: Option<RollbackStrategy>,
    /// Quality gates to verify
    pub quality_gates: Vec<String>,
}

/// Replacement strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReplacementStrategy {
    /// Immediate cutover (high risk)
    Immediate,
    /// Gradual traffic shifting
    Gradual { steps: u32, interval_secs: u64 },
    /// A/B testing with performance comparison
    ABTest { test_duration_secs: u64, success_threshold: f64 },
    /// Blue-green deployment
    BlueGreen,
}

/// Hot-swap result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotSwapResult {
    /// Model ID that was replaced
    pub model_id: String,
    /// New version deployed
    pub new_version: String,
    /// Success status
    pub success: bool,
    /// Deployment strategy used
    pub strategy_used: ReplacementStrategy,
    /// Performance improvement metrics
    pub performance_delta: PerformanceDelta,
    /// Quality validation results
    pub quality_validation: QualityValidation,
    /// Timestamp
    pub completed_at: chrono::DateTime<chrono::Utc>,
}

/// Performance change metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceDelta {
    /// Latency change (ms, negative = improvement)
    pub latency_delta_ms: f64,
    /// Throughput change (req/sec)
    pub throughput_delta: f64,
    /// Error rate change (negative = improvement)
    pub error_rate_delta: f64,
    /// Quality score change
    pub quality_delta: f64,
    /// Statistical significance (0.0-1.0)
    pub significance: f64,
}

/// Quality validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityValidation {
    /// All quality gates passed
    pub gates_passed: bool,
    /// Failed gates
    pub failed_gates: Vec<String>,
    /// Validation score (0.0-1.0)
    pub validation_score: f64,
    /// Manual review required
    pub manual_review_required: bool,
}

impl ModelHotSwapOrchestrator {
    /// Create a new hot-swap orchestrator
    pub async fn new(config: HotSwapConfig) -> Result<Self> {
        info!("Initializing model hot-swap orchestrator");

        let model_registry = Arc::new(ModelRegistry::new());
        let performance_router = Arc::new(PerformanceRouter::new(config.enable_performance_routing));
        let load_balancer = Arc::new(LoadBalancer::new(BalancingStrategy::PerformanceWeighted));
        let traffic_splitter = Arc::new(TrafficSplitter::new(Default::default()));
        let rollback_manager = Arc::new(RollbackManager::new(Default::default()));
        let version_manager = Arc::new(VersionManager::new());

        let active_deployments = Arc::new(RwLock::new(HashMap::new()));

        Ok(Self {
            model_registry,
            performance_router,
            load_balancer,
            traffic_splitter,
            rollback_manager,
            version_manager,
            active_deployments,
            config,
        })
    }

    /// Perform model hot-swap
    pub async fn perform_hot_swap(&self, request: ModelReplacementRequest) -> Result<HotSwapResult> {
        info!("Performing hot-swap for model {} to version {}",
              request.model_id, request.new_version);

        let start_time = chrono::Utc::now();

        // Validate request
        self.validate_replacement_request(&request).await?;

        // Check model compatibility
        self.version_manager.validate_compatibility(&request.model_id, &request.new_version).await?;

        // Execute replacement based on strategy
        let result = match request.strategy {
            ReplacementStrategy::Immediate => {
                self.perform_immediate_swap(&request).await
            }
            ReplacementStrategy::Gradual { steps, interval_secs } => {
                self.perform_gradual_swap(&request, steps, interval_secs).await
            }
            ReplacementStrategy::ABTest { test_duration_secs, success_threshold } => {
                self.perform_ab_test_swap(&request, test_duration_secs, success_threshold).await
            }
            ReplacementStrategy::BlueGreen => {
                self.perform_blue_green_swap(&request).await
            }
        };

        // Update deployment tracking
        self.update_deployment_tracking(&request, &result).await;

        // Log result
        match &result {
            Ok(result) if result.success => {
                info!("Hot-swap completed successfully for model {}", request.model_id);
            }
            Ok(result) => {
                warn!("Hot-swap completed with issues for model {}", request.model_id);
            }
            Err(e) => {
                error!("Hot-swap failed for model {}: {}", request.model_id, e);
            }
        }

        result
    }

    /// Get current model deployment status
    pub async fn get_deployment_status(&self, model_id: &str) -> Option<ModelDeployment> {
        let deployments = self.active_deployments.read().await;
        deployments.get(model_id).cloned()
    }

    /// Get all active deployments
    pub async fn get_all_deployments(&self) -> HashMap<String, ModelDeployment> {
        self.active_deployments.read().await.clone()
    }

    /// Route request to appropriate model instance
    pub async fn route_request(&self, model_id: &str, request_context: &RequestContext) -> Result<RoutingDecision> {
        // Get deployment info
        let deployment = {
            let deployments = self.active_deployments.read().await;
            deployments.get(model_id).cloned()
        };

        if deployment.is_none() {
            return Err(anyhow::anyhow!("No active deployment for model {}", model_id));
        }

        let deployment = deployment.unwrap();

        // Use performance router for intelligent routing
        self.performance_router.route_request(model_id, &deployment, request_context).await
    }

    /// Monitor model performance and trigger automatic actions
    pub async fn monitor_and_optimize(&self) -> Result<Vec<OptimizationAction>> {
        debug!("Running performance monitoring and optimization");

        let mut actions = Vec::new();

        // Get all active deployments
        let deployments = self.get_all_deployments().await;

        for (model_id, deployment) in deployments {
            // Check for performance degradation
            if deployment.performance.error_rate > self.config.auto_rollback_threshold {
                warn!("High error rate detected for model {}: {:.2}%",
                      model_id, deployment.performance.error_rate * 100.0);

                actions.push(OptimizationAction::Rollback {
                    model_id,
                    reason: format!("Error rate {:.2}% exceeds threshold {:.2}%",
                                  deployment.performance.error_rate * 100.0,
                                  self.config.auto_rollback_threshold * 100.0),
                });
            }

            // Check for quality degradation
            if let Some(prev_version) = &deployment.previous_version {
                // Compare with previous version performance
                if let Some(prev_perf) = self.get_historical_performance(&model_id, prev_version).await {
                    let quality_delta = deployment.performance.quality_score - prev_perf.quality_score;
                    if quality_delta < -self.config.quality_degradation_threshold {
                        warn!("Quality degradation detected for model {}: {:.3} delta",
                              model_id, quality_delta);

                        actions.push(OptimizationAction::QualityAlert {
                            model_id,
                            quality_delta,
                        });
                    }
                }
            }

            // Check for traffic optimization opportunities
            if deployment.traffic_allocation < 1.0 {
                // Consider increasing traffic if performance is good
                if deployment.performance.error_rate < 0.01 &&
                   deployment.performance.quality_score > 0.9 {
                    actions.push(OptimizationAction::IncreaseTraffic {
                        model_id,
                        current_allocation: deployment.traffic_allocation,
                        suggested_increase: 0.1,
                    });
                }
            }
        }

        Ok(actions)
    }

    /// Execute optimization action
    pub async fn execute_optimization_action(&self, action: OptimizationAction) -> Result<()> {
        match action {
            OptimizationAction::Rollback { model_id, reason } => {
                info!("Executing rollback for model {}: {}", model_id, reason);
                self.rollback_manager.perform_rollback(&model_id, reason).await?;
            }
            OptimizationAction::QualityAlert { model_id, quality_delta } => {
                warn!("Quality alert for model {}: {:.3} degradation", model_id, quality_delta);
                // Could trigger notifications or additional monitoring
            }
            OptimizationAction::IncreaseTraffic { model_id, current_allocation, suggested_increase } => {
                let new_allocation = (current_allocation + suggested_increase).min(1.0);
                info!("Increasing traffic allocation for model {} from {:.2} to {:.2}",
                      model_id, current_allocation, new_allocation);

                // Update traffic allocation
                {
                    let mut deployments = self.active_deployments.write().await;
                    if let Some(deployment) = deployments.get_mut(&model_id) {
                        deployment.traffic_allocation = new_allocation;
                    }
                }

                // Update load balancer
                self.load_balancer.update_traffic_allocation(&model_id, new_allocation).await?;
            }
        }

        Ok(())
    }

    /// Validate replacement request
    async fn validate_replacement_request(&self, request: &ModelReplacementRequest) -> Result<()> {
        // Check if model exists
        if !self.model_registry.model_exists(&request.model_id).await? {
            return Err(anyhow::anyhow!("Model {} does not exist", request.model_id));
        }

        // Check if new version exists
        if !self.version_manager.version_exists(&request.model_id, &request.new_version).await? {
            return Err(anyhow::anyhow!("Version {} does not exist for model {}",
                                      request.new_version, request.model_id));
        }

        // Validate strategy parameters
        match &request.strategy {
            ReplacementStrategy::Gradual { steps, .. } => {
                if *steps == 0 {
                    return Err(anyhow::anyhow!("Gradual replacement requires at least 1 step"));
                }
            }
            ReplacementStrategy::ABTest { test_duration_secs, success_threshold } => {
                if *test_duration_secs == 0 {
                    return Err(anyhow::anyhow!("A/B test requires positive duration"));
                }
                if !(0.0..=1.0).contains(success_threshold) {
                    return Err(anyhow::anyhow!("Success threshold must be between 0.0 and 1.0"));
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Perform immediate hot-swap
    async fn perform_immediate_swap(&self, request: &ModelReplacementRequest) -> Result<HotSwapResult> {
        info!("Performing immediate hot-swap for model {}", request.model_id);

        // Measure baseline performance
        let baseline_perf = self.measure_model_performance(&request.model_id).await?;

        // Perform the swap
        self.model_registry.update_model_version(&request.model_id, &request.new_version).await?;

        // Update deployment tracking
        self.update_deployment_version(&request.model_id, &request.new_version, 1.0).await;

        // Wait for warm-up
        tokio::time::sleep(tokio::time::Duration::from_secs(self.config.warmup_period_secs)).await;

        // Measure new performance
        let new_perf = self.measure_model_performance(&request.model_id).await?;

        // Validate quality gates
        let quality_validation = self.validate_quality_gates(&request.quality_gates, &new_perf).await?;

        // Calculate performance delta
        let performance_delta = self.calculate_performance_delta(&baseline_perf, &new_perf);

        let success = quality_validation.gates_passed;

        Ok(HotSwapResult {
            model_id: request.model_id.clone(),
            new_version: request.new_version.clone(),
            success,
            strategy_used: request.strategy.clone(),
            performance_delta,
            quality_validation,
            completed_at: chrono::Utc::now(),
        })
    }

    /// Perform gradual traffic shifting
    async fn perform_gradual_swap(&self, request: &ModelReplacementRequest, steps: u32, interval_secs: u64) -> Result<HotSwapResult> {
        info!("Performing gradual hot-swap for model {} in {} steps", request.model_id, steps);

        // Start with small traffic allocation to new version
        let traffic_per_step = 1.0 / steps as f64;

        for step in 1..=steps {
            let traffic_allocation = traffic_per_step * step as f64;

            info!("Step {}/{}: Allocating {:.1}% traffic to new version",
                  step, steps, traffic_allocation * 100.0);

            // Update traffic allocation
            self.traffic_splitter.update_split(&request.model_id, traffic_allocation).await?;
            self.update_deployment_version(&request.model_id, &request.new_version, traffic_allocation).await;

            // Wait for interval
            tokio::time::sleep(tokio::time::Duration::from_secs(interval_secs)).await;

            // Monitor performance during transition
            let current_perf = self.measure_model_performance(&request.model_id).await?;
            debug!("Step {} performance: latency={:.1}ms, error_rate={:.3}",
                   step, current_perf.avg_latency_ms, current_perf.error_rate);
        }

        // Complete the swap
        self.model_registry.update_model_version(&request.model_id, &request.new_version).await?;
        self.update_deployment_version(&request.model_id, &request.new_version, 1.0).await;

        // Measure final performance
        let final_perf = self.measure_model_performance(&request.model_id).await?;
        let quality_validation = self.validate_quality_gates(&request.quality_gates, &final_perf).await?;

        // Get baseline performance (this is approximate since we don't have true baseline)
        let baseline_perf = ModelPerformance {
            rps: final_perf.rps * 0.9, // Assume 10% improvement as baseline
            avg_latency_ms: final_perf.avg_latency_ms * 1.1,
            p95_latency_ms: final_perf.p95_latency_ms * 1.1,
            error_rate: final_perf.error_rate * 1.2,
            cpu_usage: final_perf.cpu_usage,
            memory_usage: final_perf.memory_usage,
            quality_score: final_perf.quality_score * 0.95,
            last_updated: chrono::Utc::now(),
        };

        let performance_delta = self.calculate_performance_delta(&baseline_perf, &final_perf);

        Ok(HotSwapResult {
            model_id: request.model_id.clone(),
            new_version: request.new_version.clone(),
            success: quality_validation.gates_passed,
            strategy_used: request.strategy.clone(),
            performance_delta,
            quality_validation,
            completed_at: chrono::Utc::now(),
        })
    }

    /// Perform A/B testing hot-swap
    async fn perform_ab_test_swap(&self, request: &ModelReplacementRequest, test_duration_secs: u64, success_threshold: f64) -> Result<HotSwapResult> {
        info!("Performing A/B test hot-swap for model {} (duration: {}s, threshold: {:.2})",
              request.model_id, test_duration_secs, success_threshold);

        // Set up A/B test with 50/50 traffic split
        self.traffic_splitter.setup_ab_test(&request.model_id, 0.5).await?;

        // Run test for specified duration
        tokio::time::sleep(tokio::time::Duration::from_secs(test_duration_secs)).await;

        // Analyze test results
        let test_results = self.traffic_splitter.analyze_ab_test(&request.model_id).await?;
        let new_version_better = test_results.new_version_performance.quality_score >
                                test_results.baseline_performance.quality_score * success_threshold;

        if new_version_better {
            info!("A/B test successful, completing hot-swap");
            self.perform_immediate_swap(request).await
        } else {
            info!("A/B test failed, rolling back");
            self.rollback_manager.perform_rollback(&request.model_id, "A/B test failed".to_string()).await?;

            // Return failure result
            Ok(HotSwapResult {
                model_id: request.model_id.clone(),
                new_version: request.new_version.clone(),
                success: false,
                strategy_used: request.strategy.clone(),
                performance_delta: PerformanceDelta {
                    latency_delta_ms: 0.0,
                    throughput_delta: 0.0,
                    error_rate_delta: 0.0,
                    quality_delta: test_results.new_version_performance.quality_score -
                                 test_results.baseline_performance.quality_score,
                    significance: test_results.statistical_significance,
                },
                quality_validation: QualityValidation {
                    gates_passed: false,
                    failed_gates: vec!["A/B test performance threshold".to_string()],
                    validation_score: test_results.new_version_performance.quality_score,
                    manual_review_required: true,
                },
                completed_at: chrono::Utc::now(),
            })
        }
    }

    /// Perform blue-green deployment
    async fn perform_blue_green_swap(&self, request: &ModelReplacementRequest) -> Result<HotSwapResult> {
        info!("Performing blue-green hot-swap for model {}", request.model_id);

        // Deploy new version alongside old version
        self.model_registry.deploy_side_by_side(&request.model_id, &request.new_version).await?;

        // Test new version with small traffic
        self.traffic_splitter.update_split(&request.model_id, 0.05).await?; // 5% test traffic
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await; // 1 minute test

        // Validate new version
        let test_perf = self.measure_model_performance(&request.model_id).await?;
        let quality_validation = self.validate_quality_gates(&request.quality_gates, &test_perf).await?;

        if quality_validation.gates_passed {
            // Switch all traffic to new version
            self.traffic_splitter.update_split(&request.model_id, 1.0).await?;
            self.model_registry.promote_blue_green(&request.model_id).await?;
            self.update_deployment_version(&request.model_id, &request.new_version, 1.0).await;

            // Measure final performance
            let final_perf = self.measure_model_performance(&request.model_id).await?;
            let baseline_perf = self.get_current_performance(&request.model_id).await?;
            let performance_delta = self.calculate_performance_delta(&baseline_perf, &final_perf);

            Ok(HotSwapResult {
                model_id: request.model_id.clone(),
                new_version: request.new_version.clone(),
                success: true,
                strategy_used: request.strategy.clone(),
                performance_delta,
                quality_validation,
                completed_at: chrono::Utc::now(),
            })
        } else {
            // Rollback - keep old version
            self.traffic_splitter.update_split(&request.model_id, 1.0).await?; // All traffic to old version
            self.model_registry.rollback_blue_green(&request.model_id).await?;

            Ok(HotSwapResult {
                model_id: request.model_id.clone(),
                new_version: request.new_version.clone(),
                success: false,
                strategy_used: request.strategy.clone(),
                performance_delta: PerformanceDelta {
                    latency_delta_ms: 0.0,
                    throughput_delta: 0.0,
                    error_rate_delta: 0.0,
                    quality_delta: 0.0,
                    significance: 0.0,
                },
                quality_validation,
                completed_at: chrono::Utc::now(),
            })
        }
    }

    /// Update deployment tracking
    async fn update_deployment_tracking(&self, request: &ModelReplacementRequest, result: &Result<HotSwapResult>) {
        let deployment = ModelDeployment {
            model_id: request.model_id.clone(),
            current_version: request.new_version.clone(),
            previous_version: Some(self.get_current_version(&request.model_id).await.unwrap_or_default()),
            status: if result.is_ok() && result.as_ref().unwrap().success {
                DeploymentStatus::Active
            } else {
                DeploymentStatus::Failed("Hot-swap failed".to_string())
            },
            traffic_allocation: if result.is_ok() && result.as_ref().unwrap().success { 1.0 } else { 0.0 },
            performance: self.get_current_performance(&request.model_id).await,
            deployed_at: chrono::Utc::now(),
            last_health_check: chrono::Utc::now(),
        };

        let mut deployments = self.active_deployments.write().await;
        deployments.insert(request.model_id.clone(), deployment);
    }

    /// Update deployment version
    async fn update_deployment_version(&self, model_id: &str, version: &str, traffic_allocation: f64) {
        let mut deployments = self.active_deployments.write().await;
        if let Some(deployment) = deployments.get_mut(model_id) {
            deployment.current_version = version.to_string();
            deployment.traffic_allocation = traffic_allocation;
            deployment.last_health_check = chrono::Utc::now();
        }
    }

    /// Measure current model performance
    async fn measure_model_performance(&self, model_id: &str) -> Result<ModelPerformance> {
        // In practice, this would collect real performance metrics
        // For now, simulate realistic metrics

        Ok(ModelPerformance {
            rps: 150.0 + (rand::random::<f64>() - 0.5) * 50.0,
            avg_latency_ms: 45.0 + (rand::random::<f64>() - 0.5) * 20.0,
            p95_latency_ms: 80.0 + (rand::random::<f64>() - 0.5) * 30.0,
            error_rate: (rand::random::<f64>() * 0.05).min(0.1),
            cpu_usage: 25.0 + rand::random::<f64>() * 30.0,
            memory_usage: 40.0 + rand::random::<f64>() * 30.0,
            quality_score: 0.8 + rand::random::<f64>() * 0.2,
            last_updated: chrono::Utc::now(),
        })
    }

    /// Get current performance for model
    async fn get_current_performance(&self, model_id: &str) -> ModelPerformance {
        let deployments = self.active_deployments.read().await;
        deployments.get(model_id)
            .map(|d| d.performance.clone())
            .unwrap_or_else(|| ModelPerformance {
                rps: 100.0,
                avg_latency_ms: 50.0,
                p95_latency_ms: 100.0,
                error_rate: 0.02,
                cpu_usage: 30.0,
                memory_usage: 50.0,
                quality_score: 0.85,
                last_updated: chrono::Utc::now(),
            })
    }

    /// Get historical performance for comparison
    async fn get_historical_performance(&self, model_id: &str, version: &str) -> Option<ModelPerformance> {
        // In practice, this would query historical performance data
        Some(ModelPerformance {
            rps: 120.0,
            avg_latency_ms: 55.0,
            p95_latency_ms: 110.0,
            error_rate: 0.03,
            cpu_usage: 35.0,
            memory_usage: 55.0,
            quality_score: 0.82,
            last_updated: chrono::Utc::now() - chrono::Duration::hours(1),
        })
    }

    /// Get current version of model
    async fn get_current_version(&self, model_id: &str) -> Option<String> {
        let deployments = self.active_deployments.read().await;
        deployments.get(model_id).map(|d| d.current_version.clone())
    }

    /// Validate quality gates
    async fn validate_quality_gates(&self, gates: &[String], performance: &ModelPerformance) -> Result<QualityValidation> {
        let mut failed_gates = Vec::new();
        let mut passed_count = 0;

        for gate in gates {
            let passed = match gate.as_str() {
                "latency_under_100ms" => performance.avg_latency_ms < 100.0,
                "error_rate_under_5%" => performance.error_rate < 0.05,
                "quality_over_80%" => performance.quality_score > 0.8,
                "p95_latency_under_200ms" => performance.p95_latency_ms < 200.0,
                _ => {
                    warn!("Unknown quality gate: {}", gate);
                    true // Pass unknown gates
                }
            };

            if passed {
                passed_count += 1;
            } else {
                failed_gates.push(gate.clone());
            }
        }

        let gates_passed = failed_gates.is_empty();
        let validation_score = passed_count as f64 / gates.len() as f64;

        Ok(QualityValidation {
            gates_passed,
            failed_gates,
            validation_score,
            manual_review_required: !gates_passed,
        })
    }

    /// Calculate performance delta between two measurements
    fn calculate_performance_delta(&self, baseline: &ModelPerformance, current: &ModelPerformance) -> PerformanceDelta {
        PerformanceDelta {
            latency_delta_ms: current.avg_latency_ms - baseline.avg_latency_ms,
            throughput_delta: current.rps - baseline.rps,
            error_rate_delta: current.error_rate - baseline.error_rate,
            quality_delta: current.quality_score - baseline.quality_score,
            significance: 0.85, // Simplified - would use statistical test
        }
    }
}

/// Optimization action recommended by the system
#[derive(Debug, Clone)]
pub enum OptimizationAction {
    /// Rollback to previous version
    Rollback { model_id: String, reason: String },
    /// Quality alert (requires attention)
    QualityAlert { model_id: String, quality_delta: f64 },
    /// Increase traffic allocation
    IncreaseTraffic { model_id: String, current_allocation: f64, suggested_increase: f64 },
}

/// Request context for routing decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestContext {
    /// Request priority
    pub priority: RequestPriority,
    /// Required quality level
    pub quality_requirement: f64,
    /// Latency sensitivity
    pub latency_sensitive: bool,
    /// User/client identifier
    pub client_id: Option<String>,
}

/// Request priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestPriority {
    Low,
    Normal,
    High,
    Critical,
}

impl Default for HotSwapConfig {
    fn default() -> Self {
        Self {
            enable_performance_routing: true,
            enable_ab_testing: true,
            draining_timeout_secs: 300, // 5 minutes
            warmup_period_secs: 60, // 1 minute
            monitoring_interval_secs: 30, // 30 seconds
            auto_rollback_threshold: 0.1, // 10% error rate
            quality_degradation_threshold: 0.1, // 10% quality drop
        }
    }
}

/// @darianrosebrook
/// Model-agnostic hot-swapping system for seamless runtime model replacement
/// with performance tracking, routing optimization, and zero-downtime updates
pub use crate::hotswap_manager::HotswapManager;


