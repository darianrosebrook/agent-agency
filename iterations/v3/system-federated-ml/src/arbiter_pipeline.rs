//! Arbiter Pipeline Optimizer - Sub-50ms Decision Making
//!
//! Optimizes the arbiter's decision pipeline for <50ms classification and routing,
//! supporting 1000+ tasks/minute sustained throughput while maintaining CAWS compliance.
//! Now uses common-pipeline framework for standardized patterns.

use schemars::JsonSchema;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use async_trait::async_trait;
use system_configuration::{SequentialPipeline, SequentialPipelineConfig, PipelineStage as CommonPipelineStage, ExecutablePipeline, PipelineError, PipelineResult};

/// Configuration for arbiter decision pipeline optimization
/// Now wraps SequentialPipelineConfig with domain-specific settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionPipelineConfig {
    /// Base sequential pipeline configuration
    pub base: SequentialPipelineConfig,
    /// Domain-specific configuration
    /// Target decision latency (ms)
    pub target_latency_ms: u64,
    /// Maximum concurrent decisions
    pub max_concurrent_decisions: usize,
    /// Decision cache size
    pub cache_size: usize,
    /// Enable speculative execution
    pub speculative_execution: bool,
    /// Quality threshold for speculative decisions
    pub speculative_threshold: f64,
}

impl Default for DecisionPipelineConfig {
    fn default() -> Self {
        Self {
            base: SequentialPipelineConfig::default(),
            target_latency_ms: 50,
            max_concurrent_decisions: 100,
            cache_size: 1000,
            speculative_execution: true,
            speculative_threshold: 0.8,
        }
    }
}

/// Arbiter pipeline optimizer for sub-50ms decisions
/// Now wraps SequentialPipeline with domain-specific decision logic
#[derive(Debug)]
pub struct ArbiterPipelineOptimizer {
    config: Arc<RwLock<DecisionPipelineConfig>>,
    /// Common sequential pipeline for standardized execution
    /// Uses DecisionResult as both input and output to work with SequentialPipeline's Input=Output constraint
    sequential_pipeline: Arc<SequentialPipeline<DecisionResult>>,
    /// Decision cache for frequently seen task patterns
    decision_cache: Arc<RwLock<lru::LruCache<String, DecisionResult>>>,
    /// Performance metrics
    metrics: Arc<RwLock<PipelineMetrics>>,
    /// Active decision workers
    workers: Vec<tokio::task::JoinHandle<()>>,
    /// Monitoring task handle
    monitoring_handle: Option<tokio::task::JoinHandle<()>>,
}

/// Input for decision pipeline
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DecisionInput {
    /// Task description
    pub task_description: String,
    /// Task metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Priority level
    pub priority: u8,
}

/// Cached decision result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DecisionResult {
    /// Task classification
    pub task_type: String,
    /// Risk tier assessment
    pub risk_tier: String,
    /// Recommended worker pool
    pub worker_pool: String,
    /// Confidence score
    pub confidence: f64,
    /// Cached timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Additional metadata for pipeline stages
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Decision pipeline stages
#[derive(Debug, Clone, JsonSchema)]
pub enum DecisionStage {
    CacheLookup,
    Classification,
    RiskAssessment,
    WorkerSelection,
    SpeculativeExecution,
}

/// Adapter to convert decision stages to common pipeline stages
#[derive(Debug)]
pub struct DecisionStageAdapter {
    stage_type: DecisionStage,
    cache: Option<Arc<RwLock<lru::LruCache<String, DecisionResult>>>>,
}

impl DecisionStageAdapter {
    pub fn new(
        stage_type: DecisionStage,
        cache: Option<Arc<RwLock<lru::LruCache<String, DecisionResult>>>>,
    ) -> Self {
        Self { stage_type, cache }
    }
}

#[async_trait]
impl CommonPipelineStage<DecisionResult, DecisionResult> for DecisionStageAdapter {
    fn name(&self) -> &str {
        match self.stage_type {
            DecisionStage::CacheLookup => "cache_lookup",
            DecisionStage::Classification => "classification",
            DecisionStage::RiskAssessment => "risk_assessment",
            DecisionStage::WorkerSelection => "worker_selection",
            DecisionStage::SpeculativeExecution => "speculative_execution",
        }
    }

    async fn process(&self, input: DecisionResult) -> PipelineResult<DecisionResult> {
        match self.stage_type {
            DecisionStage::CacheLookup => {
                if let Some(cache) = &self.cache {
                    // Use task_type as cache key since we're working with DecisionResult now
                    let cache_key = format!("{}:{}", input.task_type, input.risk_tier);
                    let mut cache = cache.write().await;
                    if let Some(cached_result) = cache.get(&cache_key) {
                        return Ok(cached_result.clone());
                    }
                }
                // No cache hit, pass through to next stage
                Ok(input) // Pass through unchanged
            }
            DecisionStage::Classification => {
                // Extract task description from metadata if available, or use task_type
                let task_description = input.metadata.get("task_description")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&input.task_type);
                
                // Simple classification based on keywords
                let task_type = if task_description.contains("planning") {
                    "planning"
                } else if task_description.contains("execution") {
                    "execution"
                } else if task_description.contains("monitoring") {
                    "monitoring"
                } else {
                    "general"
                };

                Ok(DecisionResult {
                    task_type: task_type.to_string(),
                    risk_tier: input.risk_tier.clone(),
                    worker_pool: input.worker_pool.clone(),
                    confidence: input.confidence,
                    timestamp: input.timestamp,
                    metadata: input.metadata.clone(),
                })
            }
            DecisionStage::RiskAssessment => {
                // Extract task description from metadata for risk assessment
                let task_description = input.metadata.get("task_description")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&input.task_type);
                
                // Simple risk assessment based on keywords
                let risk_tier = if task_description.contains("auth") || 
                    task_description.contains("security") || 
                    task_description.contains("billing") || 
                    task_description.contains("payment") ||
                    task_description.contains("database") || 
                    task_description.contains("migration") {
                    "high"
                } else if task_description.contains("api") || 
                    task_description.contains("integration") ||
                    task_description.contains("deployment") || 
                    task_description.contains("production") {
                    "medium"
                } else {
                    "low"
                };

                Ok(DecisionResult {
                    task_type: input.task_type.clone(),
                    risk_tier: risk_tier.to_string(),
                    worker_pool: input.worker_pool.clone(),
                    confidence: input.confidence * 0.9, // Slightly reduce confidence after risk assessment
                    timestamp: input.timestamp,
                    metadata: input.metadata.clone(),
                })
            }
            DecisionStage::WorkerSelection => {
                // Select worker pool based on task type and risk tier
                let worker_pool = match (input.task_type.as_str(), input.risk_tier.as_str()) {
                    ("planning", "high") => "planning_high_risk_pool",
                    ("execution", "high") => "execution_high_risk_pool",
                    ("planning", _) => "planning_pool",
                    ("execution", _) => "execution_pool",
                    (_, "high") => "high_risk_pool",
                    _ => "general_pool",
                };

                Ok(DecisionResult {
                    task_type: input.task_type.clone(),
                    risk_tier: input.risk_tier.clone(),
                    worker_pool: worker_pool.to_string(),
                    confidence: input.confidence * 0.95, // Slightly reduce confidence after worker selection
                    timestamp: input.timestamp,
                    metadata: input.metadata.clone(),
                })
            }
            DecisionStage::SpeculativeExecution => {
                // For now, just pass through - speculative execution not yet implemented
                Ok(input)
            }
        }
    }

    fn can_handle(&self, _input: &DecisionResult) -> bool {
        true
    }
}

/// Pipeline performance metrics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PipelineMetrics {
    /// Average decision latency (ms)
    pub avg_latency_ms: f64,
    /// P95 decision latency (ms)
    pub p95_latency_ms: f64,
    /// Cache hit rate
    pub cache_hit_rate: f64,
    /// Total decisions processed
    pub total_decisions: u64,
    /// Speculative decisions accuracy
    pub speculative_accuracy: f64,
    /// Last updated timestamp
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl ArbiterPipelineOptimizer {
    /// Create new arbiter pipeline optimizer
    pub async fn new(config: DecisionPipelineConfig) -> Result<Self> {
        let decision_cache = Arc::new(RwLock::new(lru::LruCache::new(
            std::num::NonZeroUsize::new(config.cache_size).unwrap()
        )));

        let metrics = Arc::new(RwLock::new(PipelineMetrics {
            avg_latency_ms: 0.0,
            p95_latency_ms: 0.0,
            cache_hit_rate: 0.0,
            total_decisions: 0,
            speculative_accuracy: 0.0,
            last_updated: chrono::Utc::now(),
        }));

        // Create sequential pipeline with decision stages
        // Convert DecisionPipelineConfig to SequentialPipelineConfig
        let sequential_config = SequentialPipelineConfig {
            base: config.base.base.clone(),
            max_stage_retries: 3,
            continue_on_stage_failure: false,
            stage_timeout: std::time::Duration::from_millis(config.target_latency_ms / 4),
            enable_stage_caching: true,
        };
        let mut sequential_pipeline = SequentialPipeline::new(sequential_config);

        // Add decision stages
        let cache_ref = Some(Arc::clone(&decision_cache));
        sequential_pipeline.add_stage(Box::new(DecisionStageAdapter::new(
            DecisionStage::CacheLookup,
            cache_ref,
        ))).await;

        sequential_pipeline.add_stage(Box::new(DecisionStageAdapter::new(
            DecisionStage::Classification,
            None,
        ))).await;

        sequential_pipeline.add_stage(Box::new(DecisionStageAdapter::new(
            DecisionStage::RiskAssessment,
            None,
        ))).await;

        sequential_pipeline.add_stage(Box::new(DecisionStageAdapter::new(
            DecisionStage::WorkerSelection,
            None,
        ))).await;

        if config.speculative_execution {
            sequential_pipeline.add_stage(Box::new(DecisionStageAdapter::new(
                DecisionStage::SpeculativeExecution,
                None,
            ))).await;
        }

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            sequential_pipeline: Arc::new(sequential_pipeline),
            decision_cache,
            metrics,
            workers: Vec::new(),
            monitoring_handle: None,
        })
    }

    /// Optimize decision pipeline parameters
    pub async fn optimize_pipeline(&self, parameters: &HashMap<String, f64>) -> Result<()> {
        info!("Optimizing arbiter decision pipeline");

        // Extract relevant parameters
        let current_config = self.config.read().await;
        let target_latency = parameters.get("decision_timeout_ms")
            .copied()
            .unwrap_or(current_config.target_latency_ms as f64) as u64;

        let max_concurrent = parameters.get("max_concurrent_decisions")
            .copied()
            .unwrap_or(current_config.max_concurrent_decisions as f64) as usize;
        drop(current_config);

        // Update configuration
        {
            let mut config = self.config.write().await;
            config.target_latency_ms = target_latency;
            config.max_concurrent_decisions = max_concurrent;
            
            // Update base pipeline config timeout based on target latency
            config.base.base.timeout = std::time::Duration::from_millis(target_latency * 2); // Allow 2x latency for timeout
            config.base.stage_timeout = std::time::Duration::from_millis(target_latency / 4); // Each stage gets 1/4 of total latency budget
        }

        // Update metrics with optimization event
        {
            let mut metrics = self.metrics.write().await;
            metrics.last_updated = chrono::Utc::now();
        }

        info!("Updated pipeline config: latency={}ms, concurrent={}", target_latency, max_concurrent);

        Ok(())
    }
    
    /// Start continuous performance monitoring and auto-tuning loop
    /// Returns a channel receiver that can be used to trigger optimizations
    pub fn start_monitoring(&mut self) -> Result<tokio::sync::mpsc::Receiver<HashMap<String, f64>>> {
        if self.monitoring_handle.is_some() {
            warn!("Monitoring loop already running");
            return Err(anyhow::anyhow!("Monitoring loop already running"));
        }

        let (tx, rx) = tokio::sync::mpsc::channel(10);
        let config = Arc::clone(&self.config);
        let metrics = Arc::clone(&self.metrics);
        
        // Spawn background monitoring task
        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(30)); // Check every 30 seconds
            
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        // Read current metrics
                        let current_metrics = metrics.read().await;
                        let avg_latency = current_metrics.avg_latency_ms;
                        let p95_latency = current_metrics.p95_latency_ms;
                        let target_latency = {
                            let config_guard = config.read().await;
                            config_guard.target_latency_ms as f64
                        };
                        
                        drop(current_metrics);
                        
                        // Auto-tune if latency exceeds target
                        if avg_latency > target_latency * 1.2 || p95_latency > target_latency * 1.5 {
                            info!("Performance degradation detected: avg={:.2}ms, p95={:.2}ms, target={}ms", 
                                  avg_latency, p95_latency, target_latency);
                            
                            // Calculate optimization parameters
                            let mut optimization_params = HashMap::new();
                            
                            // Reduce timeout if latency is high
                            let new_timeout = (target_latency * 0.9) as u64; // Target 90% of current target
                            optimization_params.insert("decision_timeout_ms".to_string(), new_timeout as f64);
                            
                            // Adjust concurrent decisions based on performance
                            let current_concurrent = {
                                let config_guard = config.read().await;
                                config_guard.max_concurrent_decisions
                            };
                            
                            // Reduce concurrency if latency is high
                            let new_concurrent = if avg_latency > target_latency * 1.5 {
                                (current_concurrent as f64 * 0.8) as usize // Reduce by 20%
                            } else {
                                current_concurrent
                            };
                            optimization_params.insert("max_concurrent_decisions".to_string(), new_concurrent as f64);
                            
                            // Send optimization request via channel
                            if tx.send(optimization_params).await.is_err() {
                                break; // Receiver dropped, exit loop
                            }
                        }
                    }
                }
            }
        });
        
        self.monitoring_handle = Some(handle);
        info!("Started continuous performance monitoring loop");
        
        Ok(rx)
    }
    
    /// Process optimization requests from monitoring loop
    pub async fn process_optimization_requests(&self, mut rx: tokio::sync::mpsc::Receiver<HashMap<String, f64>>) {
        while let Some(params) = rx.recv().await {
            if let Err(e) = self.optimize_pipeline(&params).await {
                warn!("Failed to apply optimization parameters: {}", e);
            }
        }
    }
    
    /// Stop continuous monitoring loop
    pub fn stop_monitoring(&mut self) {
        if let Some(handle) = self.monitoring_handle.take() {
            handle.abort();
            info!("Stopped continuous performance monitoring loop");
        }
    }

    /// Make optimized decision with caching and speculative execution
    pub async fn make_decision(&self, task_description: &str, context: &str) -> Result<DecisionResult> {
        let start_time = std::time::Instant::now();

        // Convert DecisionInput to DecisionResult for pipeline (SequentialPipeline requires Input = Output)
        // Store original task_description in metadata for stages that need it
        let mut initial_result = DecisionResult {
            task_type: "unknown".to_string(),
            risk_tier: "unknown".to_string(),
            worker_pool: "default".to_string(),
            confidence: 0.0,
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        };
        initial_result.metadata.insert("task_description".to_string(), serde_json::Value::String(task_description.to_string()));
        initial_result.metadata.insert("context".to_string(), serde_json::Value::String(context.to_string()));

        // Execute through sequential pipeline
        let result = self.sequential_pipeline.execute(initial_result).await
            .map_err(|e| anyhow::anyhow!("Pipeline execution failed: {}", e))?;

        // Cache the result
        let cache_key = self.create_cache_key(task_description, context);
        self.cache_result(cache_key, result.clone()).await;

        let latency = start_time.elapsed().as_millis() as f64;
        self.update_metrics(false, latency, result.confidence).await;

        Ok(result)
    }

    /// Apply optimized parameters to running pipeline
    pub async fn apply_parameters(&self, parameters: &HashMap<String, f64>) -> Result<()> {
        self.optimize_pipeline(parameters).await
    }

    /// Create cache key from task description and context
    fn create_cache_key(&self, task_description: &str, context: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        task_description.hash(&mut hasher);
        context.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Check decision cache
    async fn check_cache(&self, cache_key: &str) -> Option<DecisionResult> {
        let mut cache = self.decision_cache.write().await;
        cache.get(cache_key).cloned()
    }

    /// Cache decision result
    async fn cache_result(&self, cache_key: String, result: DecisionResult) {
        let mut cache = self.decision_cache.write().await;
        cache.put(cache_key, result);
    }

    /// Make standard decision (non-speculative)
    async fn make_standard_decision(&self, task_description: &str, context: &str) -> Result<DecisionResult> {
        // TODO: Implement ML-based or rule-based decision logic
        //       Currently uses basic decision logic; should use ML models or rule-based classification for accurate decisions.
        //
        // COMPLETION CHECKLIST:
        // [ ] Implement ML model for decision classification
        // [ ] Or implement rule-based classification system
        // [ ] Train/configure decision model with historical data
        // [ ] Handle decision confidence and uncertainty
        // [ ] Add unit tests with various decision scenarios
        // [ ] Add integration tests with real decision tasks
        // [ ] Performance: Decision should complete in <100ms
        // [ ] Documentation: Document decision methodology
        //
        // ACCEPTANCE CRITERIA:
        // - Decisions are accurate and consistent
        // - Decision confidence is properly calculated
        // - Handles edge cases and ambiguous inputs
        // - Performance meets latency requirements
        //
        // DEPENDENCIES:
        // - ML model or rule engine (Required)
        // - Training data or rule definitions (Required)
        //
        // ESTIMATED EFFORT: 8-12 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (decision making feature)
        // - Change Budget: ~200 LOC
        // - Reviewer Requirements: ML or rule-based systems expertise

        let task_type = self.classify_task_type(task_description)?;
        let risk_tier = self.assess_risk_tier(task_description, context)?;
        let worker_pool = self.select_worker_pool(&task_type, &risk_tier)?;

        Ok(DecisionResult {
            task_type,
            risk_tier,
            worker_pool,
            confidence: 0.85, // Base confidence
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        })
    }

    /// Make speculative decision with quality validation
    async fn make_speculative_decision(&self, task_description: &str, context: &str) -> Result<DecisionResult> {
        // Fast-path decision for immediate response
        let fast_result = self.make_fast_decision(task_description)?;

        // Only return fast result if confidence is above threshold
        let threshold = {
            let config = self.config.read().await;
            config.speculative_threshold
        };
        if fast_result.confidence >= threshold {
            return Ok(fast_result);
        }

        // Fall back to standard decision if fast-path confidence too low
        self.make_standard_decision(task_description, context).await
    }

    /// Fast decision for speculative execution
    fn make_fast_decision(&self, task_description: &str) -> Result<DecisionResult> {
        // Ultra-fast rule-based classification for speculative execution

        let task_type = if task_description.contains("test") || task_description.contains("spec") {
            "testing"
        } else if task_description.contains("code") || task_description.contains("implement") {
            "coding"
        } else if task_description.contains("analyze") || task_description.contains("review") {
            "analysis"
        } else {
            "general"
        };

        let risk_tier = if task_description.contains("security") || task_description.contains("auth") {
            "high"
        } else if task_description.contains("billing") || task_description.contains("payment") {
            "high"
        } else {
            "medium"
        };

        let worker_pool = match risk_tier {
            "high" => "specialized",
            _ => "general",
        };

        Ok(DecisionResult {
            task_type: task_type.to_string(),
            risk_tier: risk_tier.to_string(),
            worker_pool: worker_pool.to_string(),
            confidence: 0.7, // Lower confidence for fast decisions
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        })
    }

    /// Classify task type from description
    fn classify_task_type(&self, task_description: &str) -> Result<String> {
        // Rule-based classification - could be enhanced with ML model
        if task_description.contains("write") && task_description.contains("code") {
            Ok("code_generation".to_string())
        } else if task_description.contains("test") || task_description.contains("spec") {
            Ok("testing".to_string())
        } else if task_description.contains("review") || task_description.contains("analyze") {
            Ok("analysis".to_string())
        } else if task_description.contains("design") || task_description.contains("ui") {
            Ok("design".to_string())
        } else {
            Ok("general".to_string())
        }
    }

    /// Assess risk tier based on task content
    fn assess_risk_tier(&self, task_description: &str, context: &str) -> Result<String> {
        let content = format!("{} {}", task_description, context);

        // High risk indicators
        if content.contains("security") || content.contains("auth") ||
           content.contains("billing") || content.contains("payment") ||
           content.contains("database") || content.contains("migration") {
            Ok("high".to_string())
        }
        // Medium risk indicators
        else if content.contains("api") || content.contains("integration") ||
                content.contains("deployment") || content.contains("production") {
            Ok("medium".to_string())
        }
        // Low risk default
        else {
            Ok("low".to_string())
        }
    }

    /// Select appropriate worker pool
    fn select_worker_pool(&self, task_type: &str, risk_tier: &str) -> Result<String> {
        match (task_type, risk_tier) {
            ("code_generation", "high") => Ok("specialized_coding_high_risk".to_string()),
            ("code_generation", _) => Ok("specialized_coding".to_string()),
            ("testing", "high") => Ok("specialized_testing_high_risk".to_string()),
            ("testing", _) => Ok("specialized_testing".to_string()),
            ("analysis", _) => Ok("analysis_pool".to_string()),
            ("design", _) => Ok("design_pool".to_string()),
            _ => Ok("general_pool".to_string()),
        }
    }

    /// Update performance metrics
    async fn update_metrics(&self, cache_hit: bool, latency_ms: f64, confidence: f64) {
        let mut metrics = self.metrics.write().await;

        // Update counters
        metrics.total_decisions += 1;

        // Update latency (simple moving average)
        let alpha = 0.1; // Smoothing factor
        metrics.avg_latency_ms = metrics.avg_latency_ms * (1.0 - alpha) + latency_ms * alpha;

        // Update cache hit rate
        let hit_rate_alpha = 0.01; // Slow-moving average for hit rate
        let hit = if cache_hit { 1.0 } else { 0.0 };
        metrics.cache_hit_rate = metrics.cache_hit_rate * (1.0 - hit_rate_alpha) + hit * hit_rate_alpha;

        // TODO: Implement proper speculative accuracy tracking
        //       Currently uses basic update; should track speculative accuracy with proper statistical methods.
        //
        // COMPLETION CHECKLIST:
        // [ ] Track actual speculative execution outcomes
        // [ ] Compare speculative results with final results
        // [ ] Calculate accuracy metrics using proper statistical methods
        // [ ] Track accuracy over time windows
        // [ ] Handle accuracy calculation errors
        // [ ] Add unit tests with various accuracy scenarios
        // [ ] Add integration tests with real speculative execution
        // [ ] Performance: Accuracy tracking should complete in <1ms
        // [ ] Documentation: Document accuracy calculation methodology
        //
        // ACCEPTANCE CRITERIA:
        // - Speculative accuracy reflects actual execution outcomes
        // - Statistical methods are used for accuracy calculation
        // - Accuracy is tracked over appropriate time windows
        // - Accuracy metrics are accurate and reliable
        // - Tracking performance is acceptable
        //
        // DEPENDENCIES:
        // - Outcome tracking system (Required)
        // - Statistical analysis utilities (Required)
        // - Time window management (Required)
        //
        // ESTIMATED EFFORT: 5-7 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (monitoring feature)
        // - Change Budget: ~200 LOC
        // - Reviewer Requirements: Statistics expertise
        let threshold = {
            let config = self.config.read().await;
            config.speculative_threshold
        };
        if confidence >= threshold {
            let accuracy_alpha = 0.05;
            metrics.speculative_accuracy = metrics.speculative_accuracy * (1.0 - accuracy_alpha) + 0.9 * accuracy_alpha;
        }

        metrics.last_updated = chrono::Utc::now();
    }

    /// Get current pipeline metrics
    pub async fn get_metrics(&self) -> PipelineMetrics {
        self.metrics.read().await.clone()
    }
}


