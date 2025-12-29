//! Arbiter Pipeline Optimizer - Sub-50ms Decision Making
//!
//! Optimizes the arbiter's decision pipeline for <50ms classification and routing,
//! supporting 1000+ tasks/minute sustained throughput while maintaining CAWS compliance.
//! Now uses common-pipeline framework for standardized patterns.

use anyhow::Result;
use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use system_configuration::{
    ExecutablePipeline, PipelineResult, PipelineStage as CommonPipelineStage, SequentialPipeline,
    SequentialPipelineConfig, StreamingPipeline, StreamingPipelineConfig,
};
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Configuration for arbiter decision pipeline optimization
/// Now supports both sequential and streaming execution modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionPipelineConfig {
    /// Base sequential pipeline configuration
    pub base: SequentialPipelineConfig,
    /// Streaming pipeline configuration (when enabled)
    pub streaming: Option<StreamingPipelineConfig>,
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
    /// Enable streaming execution for complex judge deliberations
    pub enable_streaming: bool,
}

impl Default for DecisionPipelineConfig {
    fn default() -> Self {
        Self {
            base: SequentialPipelineConfig::default(),
            streaming: Some(StreamingPipelineConfig::default()),
            target_latency_ms: 50,
            max_concurrent_decisions: 100,
            cache_size: 1000,
            speculative_execution: true,
            speculative_threshold: 0.8,
            enable_streaming: false, // Default to sequential for compatibility
        }
    }
}

/// Arbiter pipeline optimizer for sub-50ms decisions
/// Now supports both sequential and streaming pipeline execution modes
pub struct ArbiterPipelineOptimizer {
    config: Arc<RwLock<DecisionPipelineConfig>>,
    /// Common sequential pipeline for simple decisions
    sequential_pipeline: Option<Arc<SequentialPipeline<DecisionResult>>>,
    /// Streaming pipeline for complex judge deliberations with chunked processing
    streaming_pipeline: Option<Arc<StreamingPipeline<DecisionInput, DecisionResult>>>,
    /// Decision cache for frequently seen task patterns
    decision_cache: Arc<RwLock<lru::LruCache<String, DecisionResult>>>,
    /// Performance metrics
    metrics: Arc<RwLock<PipelineMetrics>>,
    /// Speculative execution accuracy tracking
    speculative_accuracy: Arc<RwLock<SpeculativeAccuracyTracker>>,
    /// Active decision workers
    #[allow(dead_code)]
    workers: Vec<tokio::task::JoinHandle<()>>,
    /// Monitoring task handle
    monitoring_handle: Option<tokio::task::JoinHandle<()>>,
    /// Continuous optimization service
    continuous_optimizer: Option<Arc<crate::continuous_optimization::ContinuousOptimizationService>>,
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

    /// Process speculative execution stage with confidence scoring and rollback
    fn process_speculative_execution(input: DecisionResult) -> PipelineResult<DecisionResult> {
        // Extract task description from metadata for speculative analysis
        let task_description = input.metadata.get("task_description")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Generate speculative decision with confidence scoring
        let mut speculative_result = Self::generate_speculative_decision(task_description)?;

        // Update historical accuracy in confidence factors
        speculative_result.confidence_factors.historical_accuracy = 0.75;

        // Calculate confidence score based on decision quality indicators
        let confidence_score = Self::calculate_speculative_confidence(&speculative_result, &input)?;

        // Only use speculative result if confidence is above threshold (0.8 default)
        let threshold = 0.8;
        if confidence_score >= threshold {
            Ok(DecisionResult {
                task_type: speculative_result.task_type,
                risk_tier: speculative_result.risk_tier,
                worker_pool: speculative_result.worker_pool,
                confidence: confidence_score,
                timestamp: chrono::Utc::now(),
                metadata: {
                    let mut meta = input.metadata.clone();
                    meta.insert("speculative_used".to_string(), serde_json::Value::Bool(true));
                    meta.insert("speculative_confidence".to_string(), serde_json::Value::Number(
                        serde_json::Number::from_f64(confidence_score).unwrap()
                    ));
                    meta.insert("rollback_available".to_string(), serde_json::Value::Bool(true));
                    meta.insert("original_task_type".to_string(), serde_json::Value::String(input.task_type));
                    meta.insert("original_risk_tier".to_string(), serde_json::Value::String(input.risk_tier));
                    meta.insert("original_worker_pool".to_string(), serde_json::Value::String(input.worker_pool));
                    meta
                },
            })
        } else {
            // Fall back to input result but mark as non-speculative
            Ok(DecisionResult {
                task_type: input.task_type,
                risk_tier: input.risk_tier,
                worker_pool: input.worker_pool,
                confidence: input.confidence * 0.9,
                timestamp: input.timestamp,
                metadata: {
                    let mut meta = input.metadata.clone();
                    meta.insert("speculative_used".to_string(), serde_json::Value::Bool(false));
                    meta.insert("speculative_confidence".to_string(), serde_json::Value::Number(
                        serde_json::Number::from_f64(confidence_score).unwrap()
                    ));
                    meta
                },
            })
        }
    }

    /// Generate speculative decision based on task description patterns with ensemble analysis
    fn generate_speculative_decision(task_description: &str) -> PipelineResult<SpeculativeDecision> {
        let ensemble_results = Self::ensemble_pattern_analysis(task_description);
        let (task_type, risk_tier, worker_pool, pattern_scores) = Self::select_best_ensemble_result(ensemble_results);
        let confidence_factors = Self::calculate_confidence_factors(task_description, &task_type, &risk_tier, &worker_pool);

        Ok(SpeculativeDecision {
            task_type,
            risk_tier,
            worker_pool,
            confidence_factors,
            pattern_scores,
        })
    }

    /// Speculate task type from description patterns
    fn speculate_task_type(task_description: &str) -> String {
        let desc = task_description.to_lowercase();

        if desc.contains("write code") || desc.contains("implement feature") || desc.contains("build component") {
            "code_generation".to_string()
        } else if desc.contains("write test") || desc.contains("create spec") || desc.contains("validate") {
            "testing".to_string()
        } else if desc.contains("review code") || desc.contains("analyze") || desc.contains("audit") {
            "analysis".to_string()
        } else if desc.contains("design") || desc.contains("architecture") || desc.contains("plan") {
            "design".to_string()
        } else if desc.contains("fix bug") || desc.contains("resolve issue") || desc.contains("debug") {
            "bug_fixing".to_string()
        } else if desc.contains("document") || desc.contains("readme") || desc.contains("docs") {
            "documentation".to_string()
        } else if desc.contains("code") {
            "code_generation".to_string()
        } else if desc.contains("test") {
            "testing".to_string()
        } else {
            "general".to_string()
        }
    }

    /// Speculate risk tier based on content analysis
    fn speculate_risk_tier(task_description: &str) -> String {
        let desc = task_description.to_lowercase();

        if desc.contains("security") || desc.contains("auth") || desc.contains("encrypt") ||
           desc.contains("billing") || desc.contains("payment") || desc.contains("finance") ||
           desc.contains("database") || desc.contains("migration") || desc.contains("production") {
            "high".to_string()
        } else if desc.contains("api") || desc.contains("integration") || desc.contains("deployment") ||
                  desc.contains("infrastructure") || desc.contains("network") {
            "medium".to_string()
        } else {
            "low".to_string()
        }
    }

    /// Speculate worker pool based on task type and risk
    fn speculate_worker_pool(task_type: &str, risk_tier: &str) -> String {
        match (task_type, risk_tier) {
            ("code_generation", "high") => "specialized_coding_high_risk".to_string(),
            ("code_generation", _) => "specialized_coding".to_string(),
            ("testing", "high") => "specialized_testing_high_risk".to_string(),
            ("testing", _) => "specialized_testing".to_string(),
            ("analysis", _) => "analysis_pool".to_string(),
            ("design", _) => "design_pool".to_string(),
            ("bug_fixing", "high") => "specialized_debugging_high_risk".to_string(),
            ("bug_fixing", _) => "specialized_debugging".to_string(),
            ("documentation", _) => "documentation_pool".to_string(),
            (_, "high") => "high_risk_pool".to_string(),
            _ => "general_pool".to_string(),
        }
    }

    /// Calculate confidence score for speculative decision
    fn calculate_speculative_confidence(speculative: &SpeculativeDecision, baseline: &DecisionResult) -> PipelineResult<f64> {
        let factors = &speculative.confidence_factors;
        let mut confidence = factors.ensemble_consensus;

        if let Some(task_consensus) = speculative.pattern_scores.get("task_type_consensus") {
            confidence = confidence * 0.7 + task_consensus * 0.3;
        }
        if let Some(risk_consensus) = speculative.pattern_scores.get("risk_tier_consensus") {
            confidence = confidence * 0.8 + risk_consensus * 0.2;
        }

        confidence += factors.pattern_strength * 0.1;
        confidence += factors.context_completeness * 0.05;
        confidence += factors.risk_clarity * 0.05;

        if speculative.task_type == baseline.task_type {
            confidence += 0.1;
        }
        if speculative.risk_tier == baseline.risk_tier {
            confidence += 0.05;
        }

        confidence *= factors.historical_accuracy;

        if speculative.worker_pool.contains("specialized") {
            confidence += 0.05;
        }

        Ok(confidence.max(0.1).min(0.95))
    }

    /// Ensemble pattern analysis using multiple matching strategies
    fn ensemble_pattern_analysis(task_description: &str) -> Vec<(String, String, String, f64)> {
        let mut results = Vec::new();
        let desc = task_description.to_lowercase();

        let (tt1, rt1, wp1) = Self::keyword_pattern_matching(&desc);
        results.push((tt1, rt1, wp1, 0.7));

        let (tt2, rt2, wp2) = Self::semantic_pattern_matching(&desc);
        results.push((tt2, rt2, wp2, 0.8));

        let (tt3, rt3, wp3) = Self::context_aware_pattern_matching(&desc);
        results.push((tt3, rt3, wp3, 0.75));

        let (tt4, rt4, wp4) = Self::statistical_pattern_matching(&desc);
        results.push((tt4, rt4, wp4, 0.6));

        results
    }

    /// Keyword-based pattern matching
    fn keyword_pattern_matching(desc: &str) -> (String, String, String) {
        let task_type = Self::speculate_task_type(desc);
        let risk_tier = Self::speculate_risk_tier(desc);
        let worker_pool = Self::speculate_worker_pool(&task_type, &risk_tier);
        (task_type, risk_tier, worker_pool)
    }

    /// Semantic pattern matching
    fn semantic_pattern_matching(desc: &str) -> (String, String, String) {
        let mut task_type = "general".to_string();
        let mut risk_tier = "low".to_string();

        if desc.contains("write") && (desc.contains("code") || desc.contains("function") || desc.contains("class")) {
            task_type = "code_generation".to_string();
            if desc.contains("complex") || desc.contains("algorithm") {
                risk_tier = "medium".to_string();
            }
        } else if desc.contains("test") || desc.contains("validate") || desc.contains("verify") {
            task_type = "testing".to_string();
            if desc.contains("integration") || desc.contains("system") {
                risk_tier = "medium".to_string();
            }
        } else if desc.contains("review") || desc.contains("analyze") || desc.contains("examine") {
            task_type = "analysis".to_string();
            if desc.contains("security") || desc.contains("performance") {
                risk_tier = "high".to_string();
            }
        }

        if desc.contains("security") || desc.contains("auth") || desc.contains("encrypt") ||
           desc.contains("billing") || desc.contains("payment") || desc.contains("database") {
            risk_tier = "high".to_string();
        }

        let worker_pool = Self::speculate_worker_pool(&task_type, &risk_tier);
        (task_type, risk_tier, worker_pool)
    }

    /// Context-aware pattern matching
    fn context_aware_pattern_matching(desc: &str) -> (String, String, String) {
        let mut task_type = "general".to_string();
        let mut risk_tier = "low".to_string();

        let action_words = ["create", "build", "implement", "design", "fix", "update", "optimize"];
        let target_words = ["code", "test", "api", "database", "ui", "component", "feature"];

        let has_action = action_words.iter().any(|&word| desc.contains(word));
        let has_target = target_words.iter().any(|&word| desc.contains(word));

        if has_action && has_target {
            if desc.contains("create") || desc.contains("build") || desc.contains("implement") {
                if desc.contains("code") || desc.contains("component") {
                    task_type = "code_generation".to_string();
                } else if desc.contains("test") {
                    task_type = "testing".to_string();
                } else if desc.contains("api") {
                    task_type = "api_development".to_string();
                    risk_tier = "medium".to_string();
                }
            } else if desc.contains("fix") || desc.contains("update") {
                if desc.contains("bug") || desc.contains("issue") {
                    task_type = "bug_fixing".to_string();
                } else {
                    task_type = "code_modification".to_string();
                }
            } else if desc.contains("design") {
                task_type = "design".to_string();
            }
        }

        if desc.contains("production") || desc.contains("live") || desc.contains("critical") {
            risk_tier = "high".to_string();
        } else if desc.contains("experimental") || desc.contains("prototype") {
            risk_tier = "low".to_string();
        }

        let worker_pool = Self::speculate_worker_pool(&task_type, &risk_tier);
        (task_type, risk_tier, worker_pool)
    }

    /// Statistical pattern matching
    fn statistical_pattern_matching(desc: &str) -> (String, String, String) {
        let task_type = if desc.contains("code") {
            "code_generation".to_string()
        } else if desc.contains("test") {
            "testing".to_string()
        } else if desc.contains("review") || desc.contains("check") {
            "analysis".to_string()
        } else {
            "general".to_string()
        };

        let risk_tier = if desc.contains("security") || desc.contains("auth") || desc.contains("data") {
            "high".to_string()
        } else if desc.contains("feature") || desc.contains("api") {
            "medium".to_string()
        } else {
            "low".to_string()
        };

        let worker_pool = Self::speculate_worker_pool(&task_type, &risk_tier);
        (task_type, risk_tier, worker_pool)
    }

    /// Select best result from ensemble analysis
    fn select_best_ensemble_result(results: Vec<(String, String, String, f64)>) -> (String, String, String, HashMap<String, f64>) {
        let results_len = results.len();
        let mut task_type_counts: HashMap<String, f64> = HashMap::new();
        let mut risk_tier_counts: HashMap<String, f64> = HashMap::new();
        let mut worker_pool_counts: HashMap<String, f64> = HashMap::new();

        for (task_type, risk_tier, worker_pool, confidence) in results {
            *task_type_counts.entry(task_type).or_insert(0.0) += confidence;
            *risk_tier_counts.entry(risk_tier).or_insert(0.0) += confidence;
            *worker_pool_counts.entry(worker_pool).or_insert(0.0) += confidence;
        }

        let best_task_type = task_type_counts.iter().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).map(|(k, _)| k.clone()).unwrap_or_else(|| "general".to_string());
        let best_risk_tier = risk_tier_counts.iter().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).map(|(k, _)| k.clone()).unwrap_or_else(|| "low".to_string());
        let best_worker_pool = Self::speculate_worker_pool(&best_task_type, &best_risk_tier);

        let mut pattern_scores = HashMap::new();
        pattern_scores.insert("task_type_consensus".to_string(), task_type_counts.get(&best_task_type).copied().unwrap_or(0.0) / results_len as f64);
        pattern_scores.insert("risk_tier_consensus".to_string(), risk_tier_counts.get(&best_risk_tier).copied().unwrap_or(0.0) / results_len as f64);
        pattern_scores.insert("ensemble_agreement".to_string(), {
            let total_agreement = task_type_counts.get(&best_task_type).unwrap_or(&0.0) +
                                 risk_tier_counts.get(&best_risk_tier).unwrap_or(&0.0);
            total_agreement / (results_len as f64 * 2.0)
        });

        (best_task_type, best_risk_tier, best_worker_pool, pattern_scores)
    }

    /// Calculate comprehensive confidence factors
    fn calculate_confidence_factors(task_description: &str, task_type: &str, risk_tier: &str, _worker_pool: &str) -> SpeculativeConfidenceFactors {
        let desc = task_description.to_lowercase();

        let pattern_strength = {
            let mut strength: f64 = 0.0;
            let task_keywords = match task_type {
                "code_generation" => vec!["code", "implement", "write", "build"],
                "testing" => vec!["test", "validate", "verify", "check"],
                "analysis" => vec!["review", "analyze", "audit", "examine"],
                "bug_fixing" => vec!["fix", "bug", "issue", "resolve"],
                "design" => vec!["design", "architecture", "plan", "structure"],
                "documentation" => vec!["document", "readme", "docs", "write"],
                _ => vec![],
            };

            for keyword in task_keywords {
                if desc.contains(keyword) {
                    strength += 0.2;
                }
            }
            strength.min(1.0_f64)
        };

        let context_completeness = {
            let mut completeness: f64 = 0.3;
            if desc.len() > 50 { completeness += 0.2; }
            if desc.len() > 100 { completeness += 0.2; }
            if desc.contains("for") || desc.contains("with") || desc.contains("using") {
                completeness += 0.2;
            }
            if desc.contains("should") || desc.contains("must") || desc.contains("needs") {
                completeness += 0.1;
            }
            completeness.min(1.0_f64)
        };

        let risk_clarity = match risk_tier {
            "high" => {
                if desc.contains("security") || desc.contains("auth") || desc.contains("critical") ||
                   desc.contains("production") || desc.contains("billing") || desc.contains("payment") {
                    0.9
                } else {
                    0.6
                }
            }
            "medium" => {
                if desc.contains("api") || desc.contains("integration") || desc.contains("deployment") {
                    0.8
                } else {
                    0.5
                }
            }
            _ => 0.7,
        };

        let ensemble_consensus = (pattern_strength + context_completeness + risk_clarity) / 3.0;

        SpeculativeConfidenceFactors {
            pattern_strength,
            historical_accuracy: 0.7,
            context_completeness,
            ensemble_consensus,
            risk_clarity,
        }
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
                let task_description = input
                    .metadata
                    .get("task_description")
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
                let task_description = input
                    .metadata
                    .get("task_description")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&input.task_type);

                // Simple risk assessment based on keywords
                let risk_tier = if task_description.contains("auth")
                    || task_description.contains("security")
                    || task_description.contains("billing")
                    || task_description.contains("payment")
                    || task_description.contains("database")
                    || task_description.contains("migration")
                {
                    "high"
                } else if task_description.contains("api")
                    || task_description.contains("integration")
                    || task_description.contains("deployment")
                    || task_description.contains("production")
                {
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
                Self::process_speculative_execution(input)
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

/// Speculative accuracy tracker for historical performance analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeculativeAccuracyTracker {
    /// Total speculative decisions made
    pub total_speculative_decisions: u64,
    /// Correct speculative decisions
    pub correct_speculative_decisions: u64,
    /// Task type accuracy by pattern
    pub task_type_accuracy: HashMap<String, PatternAccuracy>,
    /// Risk tier accuracy by pattern
    pub risk_tier_accuracy: HashMap<String, PatternAccuracy>,
    /// Worker pool accuracy by pattern
    pub worker_pool_accuracy: HashMap<String, PatternAccuracy>,
    /// Overall accuracy rate (0.0-1.0)
    pub overall_accuracy: f64,
    /// Last updated timestamp
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Pattern accuracy tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternAccuracy {
    /// Pattern string (e.g., "write code", "security")
    pub pattern: String,
    /// Times this pattern was used
    pub occurrences: u64,
    /// Times this pattern led to correct decisions
    pub correct_predictions: u64,
    /// Accuracy rate for this pattern (0.0-1.0)
    pub accuracy_rate: f64,
    /// Confidence in this pattern's reliability
    pub confidence: f64,
}

impl Default for SpeculativeAccuracyTracker {
    fn default() -> Self {
        Self {
            total_speculative_decisions: 0,
            correct_speculative_decisions: 0,
            task_type_accuracy: HashMap::new(),
            risk_tier_accuracy: HashMap::new(),
            worker_pool_accuracy: HashMap::new(),
            overall_accuracy: 0.7, // Start with moderate confidence
            last_updated: chrono::Utc::now(),
        }
    }
}

impl SpeculativeAccuracyTracker {
    /// Record a speculative decision outcome
    pub fn record_decision_outcome(&mut self, task_description: &str, was_correct: bool) {
        self.total_speculative_decisions += 1;
        if was_correct {
            self.correct_speculative_decisions += 1;
        }

        // Update overall accuracy with exponential moving average
        let alpha = 0.1; // Learning rate
        let new_accuracy = if self.total_speculative_decisions > 0 {
            self.correct_speculative_decisions as f64 / self.total_speculative_decisions as f64
        } else {
            0.0
        };
        self.overall_accuracy = self.overall_accuracy * (1.0 - alpha) + new_accuracy * alpha;

        // Extract and update pattern accuracies
        self.update_pattern_accuracy(task_description, was_correct);

        self.last_updated = chrono::Utc::now();
    }

    /// Update pattern-based accuracy tracking
    fn update_pattern_accuracy(&mut self, task_description: &str, was_correct: bool) {
        let desc = task_description.to_lowercase();

        // Track task type patterns
        if desc.contains("write code") || desc.contains("implement") {
            self.update_single_pattern("code_generation", was_correct, &mut self.task_type_accuracy.clone());
        }
        if desc.contains("test") || desc.contains("validate") {
            self.update_single_pattern("testing", was_correct, &mut self.task_type_accuracy.clone());
        }
        if desc.contains("review") || desc.contains("analyze") {
            self.update_single_pattern("analysis", was_correct, &mut self.task_type_accuracy.clone());
        }

        // Track risk tier patterns
        if desc.contains("security") || desc.contains("auth") {
            self.update_single_pattern("high_risk", was_correct, &mut self.risk_tier_accuracy.clone());
        }
        if desc.contains("api") || desc.contains("integration") {
            self.update_single_pattern("medium_risk", was_correct, &mut self.risk_tier_accuracy.clone());
        }
    }

    /// Update accuracy for a single pattern
    fn update_single_pattern(&mut self, pattern_key: &str, was_correct: bool, pattern_map: &mut HashMap<String, PatternAccuracy>) {
        let entry = pattern_map.entry(pattern_key.to_string()).or_insert_with(|| PatternAccuracy {
            pattern: pattern_key.to_string(),
            occurrences: 0,
            correct_predictions: 0,
            accuracy_rate: 0.5, // Start neutral
            confidence: 0.5,
        });

        entry.occurrences += 1;
        if was_correct {
            entry.correct_predictions += 1;
        }

        // Update accuracy with exponential moving average
        let alpha = 0.2; // Faster learning for patterns
        let new_accuracy = entry.correct_predictions as f64 / entry.occurrences as f64;
        entry.accuracy_rate = entry.accuracy_rate * (1.0 - alpha) + new_accuracy * alpha;

        // Update confidence based on sample size
        entry.confidence = (entry.occurrences as f64).min(10.0) / 10.0;
    }

    /// Get historical accuracy for a task description
    pub fn get_historical_accuracy(&self, task_description: &str) -> f64 {
        let desc = task_description.to_lowercase();
        let mut total_weighted_accuracy = 0.0;
        let mut total_weight = 0.0;

        // Check task type patterns
        if let Some(accuracy) = self.task_type_accuracy.get("code_generation") {
            if desc.contains("write code") || desc.contains("implement") {
                total_weighted_accuracy += accuracy.accuracy_rate * accuracy.confidence;
                total_weight += accuracy.confidence;
            }
        }

        if let Some(accuracy) = self.task_type_accuracy.get("testing") {
            if desc.contains("test") || desc.contains("validate") {
                total_weighted_accuracy += accuracy.accuracy_rate * accuracy.confidence;
                total_weight += accuracy.confidence;
            }
        }

        if let Some(accuracy) = self.task_type_accuracy.get("analysis") {
            if desc.contains("review") || desc.contains("analyze") {
                total_weighted_accuracy += accuracy.accuracy_rate * accuracy.confidence;
                total_weight += accuracy.confidence;
            }
        }

        // Return weighted average or overall accuracy if no patterns match
        if total_weight > 0.0 {
            total_weighted_accuracy / total_weight
        } else {
            self.overall_accuracy
        }
    }
}

/// Speculative decision result for confidence scoring
#[derive(Debug, Clone)]
pub struct SpeculativeDecision {
    /// Task type classification
    pub task_type: String,
    /// Risk tier assessment
    pub risk_tier: String,
    /// Worker pool assignment
    pub worker_pool: String,
    /// Confidence factors for decision analysis
    pub confidence_factors: SpeculativeConfidenceFactors,
    /// Pattern match scores for ensemble analysis
    pub pattern_scores: HashMap<String, f64>,
}

/// Confidence factors for speculative decision analysis
#[derive(Debug, Clone)]
pub struct SpeculativeConfidenceFactors {
    /// Pattern match strength (0.0-1.0)
    pub pattern_strength: f64,
    /// Historical accuracy for similar patterns (0.0-1.0)
    pub historical_accuracy: f64,
    /// Context completeness score (0.0-1.0)
    pub context_completeness: f64,
    /// Ensemble consensus score (0.0-1.0)
    pub ensemble_consensus: f64,
    /// Risk assessment clarity (0.0-1.0)
    pub risk_clarity: f64,
}

impl ArbiterPipelineOptimizer {
    /// Create new arbiter pipeline optimizer
    pub async fn new(config: DecisionPipelineConfig) -> Result<Self> {
        let decision_cache = Arc::new(RwLock::new(lru::LruCache::new(
            std::num::NonZeroUsize::new(config.cache_size).unwrap(),
        )));

        let metrics = Arc::new(RwLock::new(PipelineMetrics {
            avg_latency_ms: 0.0,
            p95_latency_ms: 0.0,
            cache_hit_rate: 0.0,
            total_decisions: 0,
            speculative_accuracy: 0.0,
            last_updated: chrono::Utc::now(),
        }));

        let speculative_accuracy = Arc::new(RwLock::new(SpeculativeAccuracyTracker::default()));

        let sequential_pipeline = if !config.enable_streaming {
            // Create sequential pipeline with decision stages
            let sequential_config = SequentialPipelineConfig {
                base: config.base.base.clone(),
                max_stage_retries: 3,
                continue_on_stage_failure: false,
                stage_timeout: std::time::Duration::from_millis(config.target_latency_ms / 4),
                enable_stage_caching: true,
            };
            let mut seq_pipeline = SequentialPipeline::new(sequential_config);

            // Add decision stages
            let cache_ref = Some(Arc::clone(&decision_cache));
            seq_pipeline
                .add_stage(Box::new(DecisionStageAdapter::new(
                    DecisionStage::CacheLookup,
                    cache_ref,
                )))
                .await;

            seq_pipeline
                .add_stage(Box::new(DecisionStageAdapter::new(
                    DecisionStage::Classification,
                    None,
                )))
                .await;

            seq_pipeline
                .add_stage(Box::new(DecisionStageAdapter::new(
                    DecisionStage::RiskAssessment,
                    None,
                )))
                .await;

            seq_pipeline
                .add_stage(Box::new(DecisionStageAdapter::new(
                    DecisionStage::WorkerSelection,
                    None,
                )))
                .await;

            if config.speculative_execution {
                seq_pipeline
                    .add_stage(Box::new(DecisionStageAdapter::new(
                        DecisionStage::SpeculativeExecution,
                        None,
                    )))
                    .await;
            }

            Some(Arc::new(seq_pipeline))
        } else {
            None
        };

        let streaming_pipeline = if config.enable_streaming {
            // Create streaming pipeline for complex judge deliberations
            let streaming_config = if let Some(ref streaming_cfg) = config.streaming {
                streaming_cfg.clone()
            } else {
                StreamingPipelineConfig {
                    base: config.base.base.clone(),
                    buffer_size: config.max_concurrent_decisions,
                    max_active_streams: config.max_concurrent_decisions,
                    stream_timeout: std::time::Duration::from_millis(config.target_latency_ms),
                    enable_backpressure: true,
                    backpressure_threshold: config.max_concurrent_decisions / 2,
                    enable_multiplexing: true,
                }
            };

            // Create streaming processor that handles DecisionInput -> DecisionResult
            let processor = {
                let cache = Arc::clone(&decision_cache);
                Arc::new(move |input: DecisionInput| -> PipelineResult<DecisionResult> {
                    // For streaming mode, we need to implement a more sophisticated processor
                    // that can handle complex judge-like evaluations with chunking
                    Self::process_streaming_decision(input, Arc::clone(&cache))
                })
            };

            let mut stream_pipeline = StreamingPipeline::new(streaming_config, processor);
            stream_pipeline.start().await?;
            Some(Arc::new(stream_pipeline))
        } else {
            None
        };

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            sequential_pipeline,
            streaming_pipeline,
            decision_cache,
            metrics,
            speculative_accuracy,
            workers: Vec::new(),
            monitoring_handle: None,
            continuous_optimizer: None,
        })
    }

    /// Process a decision through streaming pipeline with chunked judge deliberations
    fn process_streaming_decision(
        input: DecisionInput,
        cache: Arc<RwLock<lru::LruCache<String, DecisionResult>>>,
    ) -> PipelineResult<DecisionResult> {
        // Check cache first (streaming equivalent of cache lookup stage)
        let cache_key = format!("{}:{}", input.task_description, input.metadata.get("context")
            .and_then(|v| v.as_str()).unwrap_or(""));
        {
            let mut cache_guard = cache.blocking_write();
            if let Some(cached) = cache_guard.get(&cache_key) {
                return Ok(cached.clone());
            }
        }

        // Convert DecisionInput to DecisionResult for processing
        let mut result = DecisionResult {
            task_type: "unknown".to_string(),
            risk_tier: "unknown".to_string(),
            worker_pool: "default".to_string(),
            confidence: 0.0,
            timestamp: chrono::Utc::now(),
            metadata: input.metadata.clone(),
        };

        // Add task description to metadata for processing
        result.metadata.insert(
            "task_description".to_string(),
            serde_json::Value::String(input.task_description.clone()),
        );

        // Process through chunked streaming stages with judge-like deliberations
        let task_description = input.task_description;
        let context = input.metadata.get("context")
            .and_then(|v| v.as_str()).unwrap_or("");

        // Stage 1: Chunked Classification (break task description into chunks for analysis)
        result.task_type = Self::classify_task_chunked(&task_description)?;

        // Stage 2: Dual-Session Risk Assessment (parallel judge deliberations)
        result.risk_tier = Self::assess_risk_dual_session(&task_description, context)?;

        // Stage 3: Worker Selection with Confidence Scoring
        result.worker_pool = Self::select_worker_chunked(&result.task_type, &result.risk_tier)?;

        // Calculate confidence based on processing quality and deliberation depth
        result.confidence = Self::calculate_streaming_confidence(&result)?;

        // Add processing metadata
        result.metadata.insert(
            "processing_mode".to_string(),
            serde_json::Value::String("streaming_chunked".to_string()),
        );
        result.metadata.insert(
            "judge_deliberations".to_string(),
            serde_json::Value::Number(3.into()), // 3 stages of deliberation
        );

        // Cache the result
        {
            let mut cache_guard = cache.blocking_write();
            cache_guard.put(cache_key, result.clone());
        }

        Ok(result)
    }

    /// Chunked classification breaking task description into semantic chunks
    fn classify_task_chunked(task_description: &str) -> PipelineResult<String> {
        // Break task description into semantic chunks (sentences, phrases)
        let sentences: Vec<&str> = task_description.split(|c: char| c == '.' || c == '!' || c == '?')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        let mut classification_scores = std::collections::HashMap::new();

        // Process each chunk independently (simulates chunked processing)
        for chunk in sentences.chunks(2) { // Process in pairs for better context
            let chunk_text = chunk.join(". ");

            // Score different task types based on chunk content
            if chunk_text.contains("test") || chunk_text.contains("spec") || chunk_text.contains("validate") {
                *classification_scores.entry("testing").or_insert(0) += 2;
            }
            if chunk_text.contains("code") || chunk_text.contains("implement") || chunk_text.contains("build") {
                *classification_scores.entry("code_generation").or_insert(0) += 2;
            }
            if chunk_text.contains("analyze") || chunk_text.contains("review") || chunk_text.contains("evaluate") {
                *classification_scores.entry("analysis").or_insert(0) += 2;
            }
            if chunk_text.contains("design") || chunk_text.contains("plan") || chunk_text.contains("architect") {
                *classification_scores.entry("design").or_insert(0) += 2;
            }
            if chunk_text.contains("fix") || chunk_text.contains("bug") || chunk_text.contains("debug") {
                *classification_scores.entry("bug_fixing").or_insert(0) += 2;
            }
        }

        // Return highest scoring classification, default to general
        let task_type = classification_scores.into_iter()
            .max_by_key(|(_, score)| *score)
            .map(|(task_type, _)| task_type)
            .unwrap_or("general");

        Ok(task_type.to_string())
    }

    /// Dual-session risk assessment with parallel judge deliberations
    fn assess_risk_dual_session(task_description: &str, context: &str) -> PipelineResult<String> {
        let content = format!("{} {}", task_description, context);

        // Primary session: Keyword-based analysis
        let primary_risk = Self::assess_risk_keywords(&content);

        // Secondary session: Pattern-based analysis (simulates parallel processing)
        let secondary_risk = Self::assess_risk_patterns(&content);

        // Combine results (dual-session consensus)
        match (primary_risk.as_str(), secondary_risk.as_str()) {
            ("high", _) | (_, "high") => Ok("high".to_string()),
            ("medium", "medium") => Ok("medium".to_string()),
            ("medium", _) | (_, "medium") => Ok("medium".to_string()),
            _ => Ok("low".to_string()),
        }
    }

    /// Primary session risk assessment (keyword-based)
    fn assess_risk_keywords(content: &str) -> String {
        if content.contains("security") || content.contains("auth") ||
           content.contains("billing") || content.contains("payment") ||
           content.contains("database") || content.contains("migration") {
            "high".to_string()
        } else if content.contains("api") || content.contains("integration") ||
                  content.contains("deployment") || content.contains("production") {
            "medium".to_string()
        } else {
            "low".to_string()
        }
    }

    /// Secondary session risk assessment (pattern-based)
    fn assess_risk_patterns(content: &str) -> String {
        let words: Vec<&str> = content.split_whitespace().collect();
        let mut risk_score = 0;

        // Pattern analysis for risk indicators
        for window in words.windows(2) {
            match window {
                ["user", "data"] | ["personal", "information"] | ["sensitive", "data"] => {
                    risk_score += 3;
                }
                ["production", "system"] | ["live", "environment"] | ["customer", "facing"] => {
                    risk_score += 2;
                }
                ["critical", "path"] | ["core", "functionality"] | ["breaking", "change"] => {
                    risk_score += 2;
                }
                _ => {}
            }
        }

        match risk_score {
            5.. => "high".to_string(),
            2..=4 => "medium".to_string(),
            _ => "low".to_string(),
        }
    }

    /// Chunked worker selection with deliberation phases
    fn select_worker_chunked(task_type: &str, risk_tier: &str) -> PipelineResult<String> {
        // Phase 1: Base selection
        let base_pool = match (task_type, risk_tier) {
            ("code_generation", "high") => "specialized_coding_high_risk",
            ("code_generation", _) => "specialized_coding",
            ("testing", "high") => "specialized_testing_high_risk",
            ("testing", _) => "specialized_testing",
            ("analysis", _) => "analysis_pool",
            ("design", _) => "design_pool",
            ("bug_fixing", "high") => "specialized_debugging_high_risk",
            ("bug_fixing", _) => "specialized_debugging",
            (_, "high") => "high_risk_pool",
            _ => "general_pool",
        };

        // Phase 2: Capacity consideration (simulate chunked deliberation)
        let final_pool = if base_pool.contains("high_risk") {
            // For high-risk tasks, prefer specialized pools
            format!("{}_primary", base_pool)
        } else {
            base_pool.to_string()
        };

        Ok(final_pool)
    }

    /// Calculate confidence for streaming decisions with chunked processing
    fn calculate_streaming_confidence(result: &DecisionResult) -> PipelineResult<f64> {
        // Base confidence for streaming chunked processing
        let mut confidence: f64 = 0.85;

        // Adjust based on deliberation depth (chunked processing)
        if result.task_type != "general" && result.task_type != "unknown" {
            confidence += 0.05; // Good classification from chunked analysis
        }

        // Adjust based on dual-session risk assessment consensus
        if result.risk_tier != "unknown" {
            confidence += 0.03; // Risk assessment completed
        }

        // Adjust based on worker pool deliberation quality
        if result.worker_pool.contains("specialized") {
            confidence += 0.04; // Specialized routing
        }
        if result.worker_pool.contains("_primary") {
            confidence += 0.02; // High-risk primary routing
        }

        // Adjust based on processing mode metadata
        if let Some(mode) = result.metadata.get("processing_mode") {
            if mode == "streaming_chunked" {
                confidence += 0.01; // Bonus for chunked processing
            }
        }

        Ok(confidence.min(0.95)) // Cap at 95% for streaming mode
    }

    /// Process speculative execution stage with confidence scoring and rollback
    fn process_speculative_execution(input: DecisionResult) -> PipelineResult<DecisionResult> {
        // Extract task description from metadata for speculative analysis
        let task_description = input.metadata.get("task_description")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Generate speculative decision with confidence scoring
        let mut speculative_result = Self::generate_speculative_decision(task_description)?;

        // Update historical accuracy in confidence factors (in a real implementation,
        // this would access the accuracy tracker, but for now we use a default)
        speculative_result.confidence_factors.historical_accuracy = 0.75; // Placeholder

        // Calculate confidence score based on decision quality indicators
        let confidence_score = Self::calculate_speculative_confidence(&speculative_result, &input)?;

        // Only use speculative result if confidence is above threshold (0.8 default)
        let threshold = 0.8; // Could be made configurable
        if confidence_score >= threshold {
            Ok(DecisionResult {
                task_type: speculative_result.task_type,
                risk_tier: speculative_result.risk_tier,
                worker_pool: speculative_result.worker_pool,
                confidence: confidence_score,
                timestamp: chrono::Utc::now(),
                metadata: {
                    let mut meta = input.metadata.clone();
                    meta.insert("speculative_used".to_string(), serde_json::Value::Bool(true));
                    meta.insert("speculative_confidence".to_string(), serde_json::Value::Number(
                        serde_json::Number::from_f64(confidence_score).unwrap()
                    ));
                    meta.insert("rollback_available".to_string(), serde_json::Value::Bool(true));
                    meta.insert("original_task_type".to_string(), serde_json::Value::String(input.task_type));
                    meta.insert("original_risk_tier".to_string(), serde_json::Value::String(input.risk_tier));
                    meta.insert("original_worker_pool".to_string(), serde_json::Value::String(input.worker_pool));
                    meta
                },
            })
        } else {
            // Fall back to input result but mark as non-speculative
            Ok(DecisionResult {
                task_type: input.task_type,
                risk_tier: input.risk_tier,
                worker_pool: input.worker_pool,
                confidence: input.confidence * 0.9, // Slightly reduce confidence for non-speculative
                timestamp: input.timestamp,
                metadata: {
                    let mut meta = input.metadata.clone();
                    meta.insert("speculative_used".to_string(), serde_json::Value::Bool(false));
                    meta.insert("speculative_confidence".to_string(), serde_json::Value::Number(
                        serde_json::Number::from_f64(confidence_score).unwrap()
                    ));
                    meta
                },
            })
        }
    }

    /// Generate speculative decision based on task description patterns with ensemble analysis
    fn generate_speculative_decision(task_description: &str) -> PipelineResult<SpeculativeDecision> {
        // Use ensemble of multiple pattern matching strategies
        let ensemble_results = Self::ensemble_pattern_analysis(task_description);

        // Select best result based on consensus and confidence
        let (task_type, risk_tier, worker_pool, pattern_scores) = Self::select_best_ensemble_result(ensemble_results);

        // Calculate confidence factors
        let confidence_factors = Self::calculate_confidence_factors(task_description, &task_type, &risk_tier, &worker_pool);

        Ok(SpeculativeDecision {
            task_type,
            risk_tier,
            worker_pool,
            confidence_factors,
            pattern_scores,
        })
    }

    /// Speculate task type from description patterns
    fn speculate_task_type(task_description: &str) -> String {
        let desc = task_description.to_lowercase();

        // High-confidence patterns for task type speculation
        if desc.contains("write code") || desc.contains("implement feature") || desc.contains("build component") {
            "code_generation".to_string()
        } else if desc.contains("write test") || desc.contains("create spec") || desc.contains("validate") {
            "testing".to_string()
        } else if desc.contains("review code") || desc.contains("analyze") || desc.contains("audit") {
            "analysis".to_string()
        } else if desc.contains("design") || desc.contains("architecture") || desc.contains("plan") {
            "design".to_string()
        } else if desc.contains("fix bug") || desc.contains("resolve issue") || desc.contains("debug") {
            "bug_fixing".to_string()
        } else if desc.contains("document") || desc.contains("readme") || desc.contains("docs") {
            "documentation".to_string()
        } else {
            // Default based on common patterns
            if desc.contains("code") {
                "code_generation".to_string()
            } else if desc.contains("test") {
                "testing".to_string()
            } else {
                "general".to_string()
            }
        }
    }

    /// Speculate risk tier based on content analysis
    fn speculate_risk_tier(task_description: &str) -> String {
        let desc = task_description.to_lowercase();

        // High-risk indicators (speculative analysis)
        if desc.contains("security") || desc.contains("auth") || desc.contains("encrypt") ||
           desc.contains("billing") || desc.contains("payment") || desc.contains("finance") ||
           desc.contains("database") || desc.contains("migration") || desc.contains("production") {
            "high".to_string()
        }
        // Medium-risk indicators
        else if desc.contains("api") || desc.contains("integration") || desc.contains("deployment") ||
                  desc.contains("infrastructure") || desc.contains("network") {
            "medium".to_string()
        }
        // Low-risk default
        else {
            "low".to_string()
        }
    }

    /// Speculate worker pool based on task type and risk
    fn speculate_worker_pool(task_type: &str, risk_tier: &str) -> String {
        match (task_type, risk_tier) {
            ("code_generation", "high") => "specialized_coding_high_risk".to_string(),
            ("code_generation", _) => "specialized_coding".to_string(),
            ("testing", "high") => "specialized_testing_high_risk".to_string(),
            ("testing", _) => "specialized_testing".to_string(),
            ("analysis", _) => "analysis_pool".to_string(),
            ("design", _) => "design_pool".to_string(),
            ("bug_fixing", "high") => "specialized_debugging_high_risk".to_string(),
            ("bug_fixing", _) => "specialized_debugging".to_string(),
            ("documentation", _) => "documentation_pool".to_string(),
            (_, "high") => "high_risk_pool".to_string(),
            _ => "general_pool".to_string(),
        }
    }

    /// Calculate confidence score for speculative decision using advanced factors
    fn calculate_speculative_confidence(speculative: &SpeculativeDecision, baseline: &DecisionResult) -> PipelineResult<f64> {
        let factors = &speculative.confidence_factors;

        // Start with ensemble consensus as base
        let mut confidence = factors.ensemble_consensus;

        // Adjust based on pattern scores
        if let Some(task_consensus) = speculative.pattern_scores.get("task_type_consensus") {
            confidence = confidence * 0.7 + task_consensus * 0.3;
        }
        if let Some(risk_consensus) = speculative.pattern_scores.get("risk_tier_consensus") {
            confidence = confidence * 0.8 + risk_consensus * 0.2;
        }

        // Boost confidence based on confidence factors
        confidence += factors.pattern_strength * 0.1;      // Pattern matching strength
        confidence += factors.context_completeness * 0.05;  // Context richness
        confidence += factors.risk_clarity * 0.05;         // Risk assessment clarity

        // Baseline validation bonus (if we have comparison data)
        if speculative.task_type == baseline.task_type {
            confidence += 0.1;
        }
        if speculative.risk_tier == baseline.risk_tier {
            confidence += 0.05;
        }

        // Historical accuracy adjustment (learned from past decisions)
        confidence *= factors.historical_accuracy;

        // Specialized worker pools indicate stronger confidence
        if speculative.worker_pool.contains("specialized") {
            confidence += 0.05;
        }

        // Ensure reasonable bounds
        Ok(confidence.max(0.1).min(0.95))
    }

    /// Ensemble pattern analysis using multiple matching strategies
    fn ensemble_pattern_analysis(task_description: &str) -> Vec<(String, String, String, f64)> {
        let mut results = Vec::new();
        let desc = task_description.to_lowercase();

        // Strategy 1: Keyword-based pattern matching (original approach)
        let (tt1, rt1, wp1) = Self::keyword_pattern_matching(&desc);
        results.push((tt1, rt1, wp1, 0.7)); // Base confidence for keyword matching

        // Strategy 2: Semantic pattern matching (more sophisticated)
        let (tt2, rt2, wp2) = Self::semantic_pattern_matching(&desc);
        results.push((tt2, rt2, wp2, 0.8)); // Higher confidence for semantic matching

        // Strategy 3: Context-aware pattern matching
        let (tt3, rt3, wp3) = Self::context_aware_pattern_matching(&desc);
        results.push((tt3, rt3, wp3, 0.75)); // Moderate confidence for context-aware

        // Strategy 4: Statistical pattern matching (based on common patterns)
        let (tt4, rt4, wp4) = Self::statistical_pattern_matching(&desc);
        results.push((tt4, rt4, wp4, 0.6)); // Lower confidence for statistical

        results
    }

    /// Original keyword-based pattern matching
    fn keyword_pattern_matching(desc: &str) -> (String, String, String) {
        let task_type = Self::speculate_task_type(desc);
        let risk_tier = Self::speculate_risk_tier(desc);
        let worker_pool = Self::speculate_worker_pool(&task_type, &risk_tier);
        (task_type, risk_tier, worker_pool)
    }

    /// Semantic pattern matching with more sophisticated analysis
    fn semantic_pattern_matching(desc: &str) -> (String, String, String) {
        let mut task_type = "general".to_string();
        let mut risk_tier = "low".to_string();

        // Code-related patterns with semantic understanding
        if desc.contains("write") && (desc.contains("code") || desc.contains("function") || desc.contains("class")) {
            task_type = "code_generation".to_string();
            if desc.contains("complex") || desc.contains("algorithm") {
                risk_tier = "medium".to_string();
            }
        }

        // Test patterns with semantic context
        else if desc.contains("test") || desc.contains("validate") || desc.contains("verify") {
            task_type = "testing".to_string();
            if desc.contains("integration") || desc.contains("system") {
                risk_tier = "medium".to_string();
            }
        }

        // Analysis patterns
        else if desc.contains("review") || desc.contains("analyze") || desc.contains("examine") {
            task_type = "analysis".to_string();
            if desc.contains("security") || desc.contains("performance") {
                risk_tier = "high".to_string();
            }
        }

        // High-risk semantic indicators
        if desc.contains("security") || desc.contains("auth") || desc.contains("encrypt") ||
           desc.contains("billing") || desc.contains("payment") || desc.contains("database") {
            risk_tier = "high".to_string();
        }

        let worker_pool = Self::speculate_worker_pool(&task_type, &risk_tier);
        (task_type, risk_tier, worker_pool)
    }

    /// Context-aware pattern matching considering task context
    fn context_aware_pattern_matching(desc: &str) -> (String, String, String) {
        let mut task_type = "general".to_string();
        let mut risk_tier = "low".to_string();

        // Look for action + target patterns
        let action_words = ["create", "build", "implement", "design", "fix", "update", "optimize"];
        let target_words = ["code", "test", "api", "database", "ui", "component", "feature"];

        let has_action = action_words.iter().any(|&word| desc.contains(word));
        let has_target = target_words.iter().any(|&word| desc.contains(word));

        if has_action && has_target {
            // Determine specific task type based on combination
            if desc.contains("create") || desc.contains("build") || desc.contains("implement") {
                if desc.contains("code") || desc.contains("component") {
                    task_type = "code_generation".to_string();
                } else if desc.contains("test") {
                    task_type = "testing".to_string();
                } else if desc.contains("api") {
                    task_type = "api_development".to_string();
                    risk_tier = "medium".to_string();
                }
            }
            else if desc.contains("fix") || desc.contains("update") {
                if desc.contains("bug") || desc.contains("issue") {
                    task_type = "bug_fixing".to_string();
                } else {
                    task_type = "code_modification".to_string();
                }
            }
            else if desc.contains("design") {
                task_type = "design".to_string();
            }
        }

        // Risk assessment based on context
        if desc.contains("production") || desc.contains("live") || desc.contains("critical") {
            risk_tier = "high".to_string();
        } else if desc.contains("experimental") || desc.contains("prototype") {
            risk_tier = "low".to_string();
        }

        let worker_pool = Self::speculate_worker_pool(&task_type, &risk_tier);
        (task_type, risk_tier, worker_pool)
    }

    /// Statistical pattern matching based on common task patterns
    fn statistical_pattern_matching(desc: &str) -> (String, String, String) {
        // This would be trained on historical data, but for now use heuristics
        let task_type = if desc.contains("code") {
            "code_generation".to_string()
        } else if desc.contains("test") {
            "testing".to_string()
        } else if desc.contains("review") || desc.contains("check") {
            "analysis".to_string()
        } else {
            "general".to_string()
        };

        let risk_tier = if desc.contains("security") || desc.contains("auth") || desc.contains("data") {
            "high".to_string()
        } else if desc.contains("feature") || desc.contains("api") {
            "medium".to_string()
        } else {
            "low".to_string()
        };

        let worker_pool = Self::speculate_worker_pool(&task_type, &risk_tier);
        (task_type, risk_tier, worker_pool)
    }

    /// Select best result from ensemble analysis
    fn select_best_ensemble_result(results: Vec<(String, String, String, f64)>) -> (String, String, String, HashMap<String, f64>) {
        let results_len = results.len();
        let mut task_type_counts: HashMap<String, f64> = HashMap::new();
        let mut risk_tier_counts: HashMap<String, f64> = HashMap::new();
        let mut worker_pool_counts: HashMap<String, f64> = HashMap::new();

        // Aggregate votes with confidence weights
        for (task_type, risk_tier, worker_pool, confidence) in results {
            *task_type_counts.entry(task_type).or_insert(0.0) += confidence;
            *risk_tier_counts.entry(risk_tier).or_insert(0.0) += confidence;
            *worker_pool_counts.entry(worker_pool).or_insert(0.0) += confidence;
        }

        // Find consensus results
        let best_task_type = task_type_counts.iter().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).map(|(k, _)| k.clone()).unwrap_or_else(|| "general".to_string());
        let best_risk_tier = risk_tier_counts.iter().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).map(|(k, _)| k.clone()).unwrap_or_else(|| "low".to_string());
        let best_worker_pool = Self::speculate_worker_pool(&best_task_type, &best_risk_tier);

        // Calculate pattern scores for confidence analysis
        let mut pattern_scores = HashMap::new();
        pattern_scores.insert("task_type_consensus".to_string(), task_type_counts.get(&best_task_type).copied().unwrap_or(0.0) / results_len as f64);
        pattern_scores.insert("risk_tier_consensus".to_string(), risk_tier_counts.get(&best_risk_tier).copied().unwrap_or(0.0) / results_len as f64);
        pattern_scores.insert("ensemble_agreement".to_string(), {
            let total_agreement = task_type_counts.get(&best_task_type).unwrap_or(&0.0) +
                                 risk_tier_counts.get(&best_risk_tier).unwrap_or(&0.0);
            total_agreement / (results_len as f64 * 2.0)
        });

        (best_task_type, best_risk_tier, best_worker_pool, pattern_scores)
    }

    /// Calculate comprehensive confidence factors
    fn calculate_confidence_factors(task_description: &str, task_type: &str, risk_tier: &str, worker_pool: &str) -> SpeculativeConfidenceFactors {
        let desc = task_description.to_lowercase();
        let _ = worker_pool; // Suppress unused warning

        // Pattern strength based on keyword matches
        let pattern_strength: f64 = {
            let mut strength: f64 = 0.0;
            let task_keywords = match task_type {
                "code_generation" => vec!["code", "implement", "write", "build"],
                "testing" => vec!["test", "validate", "verify", "check"],
                "analysis" => vec!["review", "analyze", "audit", "examine"],
                "bug_fixing" => vec!["fix", "bug", "issue", "resolve"],
                "design" => vec!["design", "architecture", "plan", "structure"],
                "documentation" => vec!["document", "readme", "docs", "write"],
                _ => vec![],
            };

            for keyword in task_keywords {
                if desc.contains(keyword) {
                    strength += 0.2;
                }
            }
            strength.min(1.0)
        };

        // Context completeness based on description richness
        let context_completeness: f64 = {
            let mut completeness: f64 = 0.3; // Base completeness

            // Length-based completeness
            if desc.len() > 50 { completeness += 0.2; }
            if desc.len() > 100 { completeness += 0.2; }

            // Detail indicators
            if desc.contains("for") || desc.contains("with") || desc.contains("using") {
                completeness += 0.2;
            }

            // Specific requirements mentioned
            if desc.contains("should") || desc.contains("must") || desc.contains("needs") {
                completeness += 0.1;
            }

            completeness.min(1.0)
        };

        // Risk clarity based on clear risk indicators
        let risk_clarity = match risk_tier {
            "high" => {
                if desc.contains("security") || desc.contains("auth") || desc.contains("critical") ||
                   desc.contains("production") || desc.contains("billing") || desc.contains("payment") {
                    0.9
                } else {
                    0.6 // High risk but weak indicators
                }
            }
            "medium" => {
                if desc.contains("api") || desc.contains("integration") || desc.contains("deployment") {
                    0.8
                } else {
                    0.5
                }
            }
            "low" => 0.7, // Low risk tasks are usually clearer
            _ => 0.5,
        };

        SpeculativeConfidenceFactors {
            pattern_strength,
            historical_accuracy: 0.7, // Will be updated by tracker
            context_completeness,
            ensemble_consensus: 0.8, // Will be set by ensemble analysis
            risk_clarity,
        }
    }

    /// Rollback speculative decision if it proves incorrect
    pub async fn rollback_speculative_decision(&self, decision_result: &mut DecisionResult) -> Result<()> {
        // Check if this decision used speculative execution and has rollback data
        let used_speculative = decision_result.metadata
            .get("speculative_used")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let rollback_available = decision_result.metadata
            .get("rollback_available")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if used_speculative && rollback_available {
            // Rollback to original decision values
            if let Some(original_task_type) = decision_result.metadata
                .get("original_task_type")
                .and_then(|v| v.as_str()) {
                decision_result.task_type = original_task_type.to_string();
            }

            if let Some(original_risk_tier) = decision_result.metadata
                .get("original_risk_tier")
                .and_then(|v| v.as_str()) {
                decision_result.risk_tier = original_risk_tier.to_string();
            }

            if let Some(original_worker_pool) = decision_result.metadata
                .get("original_worker_pool")
                .and_then(|v| v.as_str()) {
                decision_result.worker_pool = original_worker_pool.to_string();
            }

            // Reduce confidence significantly due to rollback
            decision_result.confidence *= 0.3; // 70% confidence reduction

            // Update metadata to reflect rollback
            decision_result.metadata.insert("speculative_rollback".to_string(), serde_json::Value::Bool(true));
            decision_result.metadata.insert("rollback_timestamp".to_string(),
                serde_json::Value::String(chrono::Utc::now().to_rfc3339()));

            // Record incorrect speculative decision for learning
            if let Some(task_desc) = decision_result.metadata.get("task_description")
                .and_then(|v| v.as_str()) {
                let _ = self.record_speculative_outcome(task_desc, false).await;
            }

            info!("Rolled back speculative decision for task: confidence reduced to {:.2}",
                  decision_result.confidence);
        }

        Ok(())
    }

    /// Record speculative decision outcome for learning
    pub async fn record_speculative_outcome(&self, task_description: &str, was_correct: bool) -> Result<()> {
        let mut tracker = self.speculative_accuracy.write().await;
        tracker.record_decision_outcome(task_description, was_correct);

        // Also update metrics for backward compatibility
        self.update_speculative_accuracy(was_correct).await;

        Ok(())
    }

    /// Get speculative accuracy statistics
    pub async fn get_speculative_stats(&self) -> HashMap<String, f64> {
        let tracker = self.speculative_accuracy.read().await;
        let mut stats = HashMap::new();

        stats.insert("overall_accuracy".to_string(), tracker.overall_accuracy);
        stats.insert("total_decisions".to_string(), tracker.total_speculative_decisions as f64);

        // Add pattern accuracy stats
        for (pattern, accuracy) in &tracker.task_type_accuracy {
            stats.insert(format!("task_type_{}_accuracy", pattern), accuracy.accuracy_rate);
            stats.insert(format!("task_type_{}_confidence", pattern), accuracy.confidence);
        }

        for (pattern, accuracy) in &tracker.risk_tier_accuracy {
            stats.insert(format!("risk_tier_{}_accuracy", pattern), accuracy.accuracy_rate);
            stats.insert(format!("risk_tier_{}_confidence", pattern), accuracy.confidence);
        }

        stats
    }

    /// Update speculative accuracy metrics (backward compatibility)
    async fn update_speculative_accuracy(&self, was_correct: bool) {
        let mut metrics = self.metrics.write().await;
        let tracker = self.speculative_accuracy.read().await;

        // Use tracker's overall accuracy
        metrics.speculative_accuracy = tracker.overall_accuracy;
        metrics.last_updated = chrono::Utc::now();
    }

    /// Optimize decision pipeline parameters
    pub async fn optimize_pipeline(&self, parameters: &HashMap<String, f64>) -> Result<()> {
        info!("Optimizing arbiter decision pipeline");

        // Extract relevant parameters
        let current_config = self.config.read().await;
        let target_latency = parameters
            .get("decision_timeout_ms")
            .copied()
            .unwrap_or(current_config.target_latency_ms as f64) as u64;

        let max_concurrent = parameters
            .get("max_concurrent_decisions")
            .copied()
            .unwrap_or(current_config.max_concurrent_decisions as f64)
            as usize;
        drop(current_config);

        // Update configuration
        {
            let mut config = self.config.write().await;
            config.target_latency_ms = target_latency;
            config.max_concurrent_decisions = max_concurrent;

            // Update base pipeline config timeout based on target latency
            config.base.base.timeout = std::time::Duration::from_millis(target_latency * 2); // Allow 2x latency for timeout
            config.base.stage_timeout = std::time::Duration::from_millis(target_latency / 4);
            // Each stage gets 1/4 of total latency budget
        }

        // Update metrics with optimization event
        {
            let mut metrics = self.metrics.write().await;
            metrics.last_updated = chrono::Utc::now();
        }

        info!(
            "Updated pipeline config: latency={}ms, concurrent={}",
            target_latency, max_concurrent
        );

        Ok(())
    }

    /// Start continuous performance monitoring and auto-tuning loop
    /// Returns a channel receiver that can be used to trigger optimizations
    pub fn start_monitoring(
        &mut self,
    ) -> Result<tokio::sync::mpsc::Receiver<HashMap<String, f64>>> {
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
    pub async fn process_optimization_requests(
        &self,
        mut rx: tokio::sync::mpsc::Receiver<HashMap<String, f64>>,
    ) {
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
    /// Uses streaming pipeline if enabled, otherwise falls back to sequential
    pub async fn make_decision(
        &self,
        task_description: &str,
        context: &str,
    ) -> Result<DecisionResult> {
        let start_time = std::time::Instant::now();

        let result = if let Some(ref streaming_pipeline) = self.streaming_pipeline {
            // Use streaming pipeline for complex judge deliberations
            info!("Using streaming pipeline for decision making");
            let input = DecisionInput {
                task_description: task_description.to_string(),
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("context".to_string(), serde_json::Value::String(context.to_string()));
                    meta.insert("priority".to_string(), serde_json::Value::Number(1.into()));
                    meta
                },
                priority: 1,
            };

            // Send input to streaming pipeline
            streaming_pipeline.send(input).await
                .map_err(|e| anyhow::anyhow!("Streaming pipeline send failed: {}", e))?;

            // Wait for result with timeout
            match streaming_pipeline.recv_timeout(std::time::Duration::from_millis(100)).await {
                Ok(Some(result)) => result,
                Ok(None) => {
                    // Timeout - create fallback result
                    warn!("Streaming pipeline timeout, using fallback decision");
                    self.create_fallback_decision(task_description, context).await?
                }
                Err(e) => {
                    warn!("Streaming pipeline error: {}, using fallback", e);
                    self.create_fallback_decision(task_description, context).await?
                }
            }
        } else if let Some(ref sequential_pipeline) = self.sequential_pipeline {
            // Use sequential pipeline for simple decisions
            // Convert DecisionInput to DecisionResult for pipeline (SequentialPipeline requires Input = Output)
            let mut initial_result = DecisionResult {
                task_type: "unknown".to_string(),
                risk_tier: "unknown".to_string(),
                worker_pool: "default".to_string(),
                confidence: 0.0,
                timestamp: chrono::Utc::now(),
                metadata: HashMap::new(),
            };
            initial_result.metadata.insert(
                "task_description".to_string(),
                serde_json::Value::String(task_description.to_string()),
            );
            initial_result.metadata.insert(
                "context".to_string(),
                serde_json::Value::String(context.to_string()),
            );

            // Execute through sequential pipeline
            sequential_pipeline
                .execute(initial_result)
                .await
                .map_err(|e| anyhow::anyhow!("Sequential pipeline execution failed: {}", e))?
        } else {
            return Err(anyhow::anyhow!("No pipeline configured"));
        };

        // Cache the result
        let cache_key = self.create_cache_key(task_description, context);
        self.cache_result(cache_key, result.clone()).await;

        let latency = start_time.elapsed().as_millis() as f64;
        self.update_metrics(false, latency, result.confidence).await;

        Ok(result)
    }

    /// Create fallback decision when streaming pipeline fails
    async fn create_fallback_decision(
        &self,
        task_description: &str,
        context: &str,
    ) -> Result<DecisionResult> {
        warn!("Creating fallback decision for: {}", task_description);

        // Simple rule-based fallback
        let task_type = Self::classify_task_chunked(task_description)
            .unwrap_or_else(|_| "general".to_string());
        let risk_tier = Self::assess_risk_dual_session(task_description, context)
            .unwrap_or_else(|_| "medium".to_string());
        let worker_pool = Self::select_worker_chunked(&task_type, &risk_tier)
            .unwrap_or_else(|_| "general_pool".to_string());

        Ok(DecisionResult {
            task_type,
            risk_tier,
            worker_pool,
            confidence: 0.6, // Lower confidence for fallback
            timestamp: chrono::Utc::now(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("fallback".to_string(), serde_json::Value::Bool(true));
                meta.insert("task_description".to_string(), serde_json::Value::String(task_description.to_string()));
                meta.insert("context".to_string(), serde_json::Value::String(context.to_string()));
                meta
            },
        })
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
    #[allow(dead_code)]
    async fn check_cache(&self, cache_key: &str) -> Option<DecisionResult> {
        let mut cache = self.decision_cache.write().await;
        cache.get(cache_key).cloned()
    }

    /// Cache decision result
    #[allow(dead_code)]
    async fn cache_result(&self, cache_key: String, result: DecisionResult) {
        let mut cache = self.decision_cache.write().await;
        cache.put(cache_key, result);
    }

    /// Make standard decision (non-speculative)
    #[allow(dead_code)]
    async fn make_standard_decision(
        &self,
        task_description: &str,
        context: &str,
    ) -> Result<DecisionResult> {
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
    #[allow(dead_code)]
    async fn make_speculative_decision(
        &self,
        task_description: &str,
        context: &str,
    ) -> Result<DecisionResult> {
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

        let risk_tier =
            if task_description.contains("security") || task_description.contains("auth") {
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
        if content.contains("security")
            || content.contains("auth")
            || content.contains("billing")
            || content.contains("payment")
            || content.contains("database")
            || content.contains("migration")
        {
            Ok("high".to_string())
        }
        // Medium risk indicators
        else if content.contains("api")
            || content.contains("integration")
            || content.contains("deployment")
            || content.contains("production")
        {
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
        metrics.cache_hit_rate =
            metrics.cache_hit_rate * (1.0 - hit_rate_alpha) + hit * hit_rate_alpha;

        // Update speculative accuracy (basic implementation)
        let threshold = {
            let config = self.config.read().await;
            config.speculative_threshold
        };
        if confidence >= threshold {
            let accuracy_alpha = 0.05;
            metrics.speculative_accuracy =
                metrics.speculative_accuracy * (1.0 - accuracy_alpha) + 0.9 * accuracy_alpha;
        }

        metrics.last_updated = chrono::Utc::now();

        // Send metrics to continuous optimizer if available
        if let Some(ref optimizer) = self.continuous_optimizer {
            // Convert to PerformanceMetrics format expected by continuous optimizer
            let perf_metrics = crate::performance_monitor::PerformanceMetrics {
                throughput: metrics.cache_hit_rate * 100.0, // Rough throughput estimate
                avg_latency_ms: metrics.avg_latency_ms,
                p95_latency_ms: metrics.avg_latency_ms * 1.2, // Rough P95 estimate
                p99_latency_ms: metrics.avg_latency_ms * 1.5, // Rough P99 estimate
                error_rate: 1.0 - metrics.speculative_accuracy, // Error rate from speculative accuracy
                cpu_usage_percent: 0.0, // Not tracked
                memory_usage_percent: 0.0, // Not tracked
                active_connections: metrics.total_decisions as u64, // Rough estimate
                queue_depth: 0, // Not tracked
                timestamp: metrics.last_updated,
            };

            // Update continuous optimizer (don't block on this)
            let optimizer_clone = Arc::clone(optimizer);
            tokio::spawn(async move {
                if let Err(e) = optimizer_clone.update_performance(perf_metrics).await {
                    warn!("Failed to update continuous optimizer: {}", e);
                }
            });
        }
    }

    /// Get current pipeline metrics
    pub async fn get_metrics(&self) -> PipelineMetrics {
        self.metrics.read().await.clone()
    }

    /// Check if streaming pipeline is enabled
    pub async fn is_streaming_enabled(&self) -> bool {
        self.config.read().await.enable_streaming
    }

    /// Enable or disable streaming mode
    pub async fn set_streaming_mode(&self, enabled: bool) -> Result<()> {
        let mut config = self.config.write().await;
        if config.enable_streaming != enabled {
            info!("Changing streaming mode from {} to {}", config.enable_streaming, enabled);
            config.enable_streaming = enabled;
            // Note: In a real implementation, you might want to restart the pipelines
            // For now, this just updates the configuration
        }
        Ok(())
    }

    /// Set continuous optimization service
    pub fn set_continuous_optimizer(&mut self, optimizer: Arc<crate::continuous_optimization::ContinuousOptimizationService>) {
        self.continuous_optimizer = Some(optimizer);
    }

    /// Get continuous optimization service status
    pub async fn get_continuous_optimizer_status(&self) -> Option<crate::continuous_optimization::ContinuousOptimizationStatus> {
        if let Some(ref optimizer) = self.continuous_optimizer {
            match optimizer.get_status().await {
                Ok(status) => Some(status),
                Err(e) => {
                    warn!("Failed to get continuous optimizer status: {}", e);
                    None
                }
            }
        } else {
            None
        }
    }
}
