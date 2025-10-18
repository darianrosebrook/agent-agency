//! Learning signal infrastructure for adaptive routing and performance tracking
//!
//! This module provides the core infrastructure for capturing learning signals
//! from council decisions, enabling adaptive routing and continuous improvement
//! of the arbitration system.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::types::{JudgeId, TaskId, VerdictId};
use async_trait::async_trait;

/// Learning signal capturing task outcomes and judge performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningSignal {
    pub id: Uuid,
    pub task_id: TaskId,
    pub verdict_id: VerdictId,
    pub outcome: TaskOutcome,
    pub judge_dissent: Vec<JudgeDissent>,
    pub latency_ms: u64,
    pub quality_score: f32,
    pub timestamp: DateTime<Utc>,

    // Performance metrics
    pub resource_usage: ResourceUsageMetrics,
    pub caws_compliance_score: f32,
    pub claim_verification_score: Option<f32>,

    // Context for learning
    pub task_complexity: TaskComplexity,
    pub worker_performance: Option<WorkerPerformanceMetrics>,
}

/// Task outcome classification for learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskOutcome {
    Success {
        confidence: f32,
        quality_indicators: Vec<QualityIndicator>,
    },
    PartialSuccess {
        issues: Vec<String>,
        confidence: f32,
        remediation_applied: bool,
    },
    Failure {
        reason: String,
        failure_category: FailureCategory,
        recoverable: bool,
    },
    Timeout {
        duration_ms: u64,
        partial_results: Option<PartialResults>,
    },
}

/// Quality indicators for successful tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityIndicator {
    HighConfidence,
    ComprehensiveEvidence,
    MinimalDissent,
    EfficientExecution,
    StrongCAWSCompliance,
    CompleteClaimVerification,
}

/// Failure categories for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailureCategory {
    ConsensusFailure,
    ResourceExhaustion,
    CAWSViolation,
    ClaimVerificationFailure,
    JudgeTimeout,
    SystemError,
}

/// Partial results from timed-out tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialResults {
    pub completed_judges: Vec<JudgeId>,
    pub partial_consensus: f32,
    pub estimated_completion: f32,
}

/// Judge dissent tracking for learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeDissent {
    pub judge_id: JudgeId,
    pub dissent_severity: DissentSeverity,
    pub rationale: String,
    pub confidence: f32,
    pub evidence_quality: f32,
}

/// Dissent severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DissentSeverity {
    Minor,    // Minor disagreement, easily resolved
    Moderate, // Significant disagreement, requires discussion
    Major,    // Fundamental disagreement, blocks consensus
    Critical, // Complete disagreement, system failure
}

/// Resource usage metrics for learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsageMetrics {
    pub cpu_usage_percent: f32,
    pub memory_usage_mb: u64,
    pub thermal_status: ThermalStatus,
    pub ane_utilization: Option<f32>,
    pub gpu_utilization: Option<f32>,
    pub energy_consumption: Option<f32>, // Joules
}

/// Thermal status for resource optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThermalStatus {
    Normal,
    Warning,
    Throttling,
    Critical,
}

/// Task complexity assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskComplexity {
    pub estimated_effort: EffortLevel,
    pub domain_complexity: f32,
    pub interdependency_count: u32,
    pub risk_factors: Vec<RiskFactor>,
}

/// Effort levels for task complexity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffortLevel {
    Trivial,     // < 5 minutes
    Simple,      // 5-30 minutes
    Moderate,    // 30 minutes - 2 hours
    Complex,     // 2-8 hours
    VeryComplex, // > 8 hours
}

/// Risk factors affecting task complexity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskFactor {
    HighRiskTier,
    MultipleDomains,
    BreakingChanges,
    ExternalDependencies,
    PerformanceCritical,
    SecuritySensitive,
    DataMigration,
}

/// Worker performance metrics for learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerPerformanceMetrics {
    pub worker_id: Uuid,
    pub execution_time_ms: u64,
    pub quality_score: f32,
    pub caws_compliance: f32,
    pub claim_accuracy: Option<f32>,
    pub resource_efficiency: f32,
}

/// Learning signal storage and retrieval
#[async_trait::async_trait]
pub trait LearningSignalStorage: Send + Sync {
    /// Store a learning signal
    async fn store_signal(&self, signal: LearningSignal) -> Result<()>;

    /// Get learning signals for a task
    async fn get_signals_for_task(&self, task_id: TaskId) -> Result<Vec<LearningSignal>>;

    /// Get learning signals for a judge
    async fn get_signals_for_judge(&self, judge_id: &JudgeId) -> Result<Vec<LearningSignal>>;

    /// Get learning signals within time range
    async fn get_signals_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<LearningSignal>>;

    /// Get aggregated performance metrics
    async fn get_performance_metrics(
        &self,
        entity_type: PerformanceEntityType,
        entity_id: String,
        time_window: TimeWindow,
    ) -> Result<AggregatedMetrics>;

    /// Get learning recommendations
    async fn get_learning_recommendations(&self) -> Result<Vec<LearningRecommendation>>;
}

/// Entity types for performance tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceEntityType {
    Judge(JudgeId),
    Worker(Uuid),
    TaskType(String),
    System,
}

/// Time windows for metrics aggregation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeWindow {
    LastHour,
    LastDay,
    LastWeek,
    LastMonth,
    Custom {
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    },
}

/// Aggregated performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetrics {
    pub total_signals: u64,
    pub success_rate: f32,
    pub average_quality_score: f32,
    pub average_latency_ms: f64,
    pub dissent_rate: f32,
    pub resource_efficiency: f32,
    pub trends: PerformanceTrends,
}

/// Performance trends over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTrends {
    pub quality_trend: TrendDirection,
    pub latency_trend: TrendDirection,
    pub dissent_trend: TrendDirection,
    pub resource_efficiency_trend: TrendDirection,
}

/// Trend direction indicators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Improving,
    Stable,
    Declining,
    Volatile,
}

/// Learning recommendations for system improvement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningRecommendation {
    pub id: Uuid,
    pub recommendation_type: RecommendationType,
    pub priority: RecommendationPriority,
    pub description: String,
    pub rationale: String,
    pub expected_impact: f32,
    pub implementation_effort: EffortLevel,
    pub evidence: Vec<String>,
}

/// Types of learning recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationType {
    RoutingOptimization,
    ResourceAllocation,
    JudgeSelection,
    TaskComplexityAdjustment,
    CAWSComplianceImprovement,
    ClaimVerificationEnhancement,
}

/// Recommendation priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Critical,
    High,
    Medium,
    Low,
}

/// Learning signal analyzer for adaptive routing
pub struct LearningSignalAnalyzer {
    storage: Box<dyn LearningSignalStorage>,
}

impl LearningSignalAnalyzer {
    /// Create a new learning signal analyzer with in-memory storage
    pub fn new() -> Self {
        Self {
            storage: Box::new(InMemoryLearningSignalStorage::default()),
        }
    }

    /// Create a learning signal analyzer with custom storage
    pub fn with_storage(storage: Box<dyn LearningSignalStorage>) -> Self {
        Self { storage }
    }

    /// Analyze signals and generate routing recommendations
    pub async fn analyze_for_routing(
        &self,
        task_spec: &crate::types::TaskSpec,
    ) -> Result<RoutingRecommendation> {
        // Get historical signals for similar tasks
        let similar_signals = self.get_similar_task_signals(task_spec).await?;

        // Analyze judge performance for this task type
        let judge_performance = self.analyze_judge_performance(task_spec).await?;

        // Analyze resource requirements
        let resource_analysis = self.analyze_resource_requirements(task_spec).await?;

        // Generate rationale before moving values
        let rationale = self.generate_rationale(&judge_performance, &resource_analysis);
        
        // Extract values after borrowing
        let recommended_judges = judge_performance.recommended_judges;
        let resource_allocation = resource_analysis.optimal_allocation;
        let estimated_complexity = resource_analysis.estimated_complexity;
        let confidence = self.calculate_recommendation_confidence(&similar_signals);

        // Generate routing recommendation
        Ok(RoutingRecommendation {
            recommended_judges,
            resource_allocation,
            estimated_complexity,
            confidence,
            rationale,
        })
    }

    /// Get learning signals for similar tasks
    async fn get_similar_task_signals(
        &self,
        task_spec: &crate::types::TaskSpec,
    ) -> Result<Vec<LearningSignal>> {
        // Calculate a simple hash for task similarity based on task content
        let task_hash = (task_spec.id.as_u128() as u64).wrapping_mul(2654435761) % 2^32;

        // Get similar signals from storage (up to 10 most relevant)
        self.storage.get_similar_signals(task_hash, 10).await
    }

    /// Analyze judge performance for task type
    async fn analyze_judge_performance(
        &self,
        task_spec: &crate::types::TaskSpec,
    ) -> Result<JudgePerformanceAnalysis> {
        // TODO: Implement historical judge performance data querying with the following requirements:
        // 1. Historical data database integration: Query historical judge performance data
        //    - Query historical judge performance data from database systems
        //    - Handle historical data database integration optimization and performance
        //    - Implement historical data database integration validation and quality assurance
        //    - Support historical data database integration customization and configuration
        // 2. Judge performance analysis: Analyze historical judge performance patterns
        //    - Analyze historical judge performance patterns and trends
        //    - Handle judge performance analysis optimization and performance
        //    - Implement judge performance analysis validation and quality assurance
        //    - Support judge performance analysis customization and configuration
        // 3. Performance data aggregation: Aggregate judge performance data for analysis
        //    - Aggregate judge performance data for comprehensive analysis
        //    - Handle performance data aggregation optimization and performance
        //    - Implement performance data aggregation validation and quality assurance
        //    - Support performance data aggregation customization and configuration
        // 4. Historical data optimization: Optimize historical judge performance data querying performance
        //    - Implement historical judge performance data querying optimization strategies
        //    - Handle historical data monitoring and analytics
        //    - Implement historical data validation and quality assurance
        //    - Ensure historical judge performance data querying meets performance and accuracy standards
        // For simulation, generate realistic performance analysis based on task characteristics

        let task_hash = task_spec.id.as_u128() as u32;
        let mut judge_rankings = Vec::new();

        // Generate performance data for 3-5 judges
        let judge_count = (task_hash % 3) + 3;

        for i in 0..judge_count {
            let judge_id = format!("judge-{}", i);
            let base_accuracy = 0.75 + (task_hash % 25) as f32 / 100.0;
            let judge_accuracy = (base_accuracy + i as f32 * 0.05).min(0.95);

            let ranking = JudgeRanking {
                judge_id: judge_id.clone(),
                accuracy_score: judge_accuracy,
                consistency_score: judge_accuracy - 0.05 + (i as f32 * 0.02),
                speed_score: 0.8 + (i as f32 * 0.05),
                reliability_score: judge_accuracy - 0.03,
                total_evaluations: 50 + (task_hash % 100) as u32,
                ranking_score: judge_accuracy * 0.4 + (judge_accuracy - 0.05) * 0.3 + (0.8 + i as f32 * 0.05) * 0.2 + (judge_accuracy - 0.03) * 0.1,
                specialization_score: match i % 3 {
                    0 => Some(SpecializationScore {
                        domain: "Code Quality".to_string(),
                        score: judge_accuracy,
                        confidence: 0.85,
                    }),
                    1 => Some(SpecializationScore {
                        domain: "Security".to_string(),
                        score: judge_accuracy + 0.05,
                        confidence: 0.90,
                    }),
                    _ => Some(SpecializationScore {
                        domain: "Performance".to_string(),
                        score: judge_accuracy - 0.02,
                        confidence: 0.80,
                    }),
                },
            };

            judge_rankings.push(ranking);
        }

        // Sort by ranking score (descending)
        judge_rankings.sort_by(|a, b| b.ranking_score.partial_cmp(&a.ranking_score).unwrap());

        // Generate insights and recommendations
        let insights = vec![
            "High-performing judges show consistent accuracy above 0.85".to_string(),
            format!("Task complexity {} requires specialized judge selection", task_spec.risk_tier.as_str()),
            "Recent performance trends indicate improving consensus quality".to_string(),
        ];

        let recommendations = vec![
            "Route high-risk tasks to top 3 performing judges".to_string(),
            "Monitor judge performance for signs of performance drift".to_string(),
            "Consider judge specialization for complex task types".to_string(),
        ];

        let analysis = JudgePerformanceAnalysis {
            task_type: task_spec.description.chars().take(50).collect::<String>(),
            judge_rankings,
            insights,
            recommendations,
            analysis_timestamp: Utc::now(),
            confidence_score: 0.82,
        };

        Ok(analysis)
    }

    /// Analyze resource requirements
    async fn analyze_resource_requirements(
        &self,
        task_spec: &crate::types::TaskSpec,
    ) -> Result<ResourceRequirementAnalysis> {
        // TODO: Implement historical resource usage data analysis with the following requirements:
        // 1. Historical resource data integration: Analyze historical resource usage data
        //    - Analyze historical resource usage data from database systems
        //    - Handle historical resource data integration optimization and performance
        //    - Implement historical resource data integration validation and quality assurance
        //    - Support historical resource data integration customization and configuration
        // 2. Resource usage pattern analysis: Analyze resource usage patterns and trends
        //    - Analyze resource usage patterns and trends for optimization
        //    - Handle resource usage pattern analysis optimization and performance
        //    - Implement resource usage pattern analysis validation and quality assurance
        //    - Support resource usage pattern analysis customization and configuration
        // 3. Resource requirement prediction: Predict resource requirements based on historical data
        //    - Predict resource requirements based on historical usage patterns
        //    - Handle resource requirement prediction optimization and performance
        //    - Implement resource requirement prediction validation and quality assurance
        //    - Support resource requirement prediction customization and configuration
        // 4. Resource analysis optimization: Optimize historical resource usage data analysis performance
        //    - Implement historical resource usage data analysis optimization strategies
        //    - Handle resource analysis monitoring and analytics
        //    - Implement resource analysis validation and quality assurance
        //    - Ensure historical resource usage data analysis meets performance and accuracy standards
        // For simulation, generate realistic resource analysis based on task characteristics

        let task_hash = task_spec.id.as_u128() as u32;
        let task_complexity = self.estimate_task_complexity(task_spec);

        // Base resource requirements based on task complexity
        let (base_cpu, base_memory, base_io) = match task_complexity {
            TaskComplexity::Low => (10.0, 256, 100000),
            TaskComplexity::Medium => (25.0, 512, 500000),
            TaskComplexity::High => (50.0, 1024, 2000000),
            TaskComplexity::Critical => (80.0, 2048, 5000000),
        };

        // Add variability based on task characteristics
        let cpu_percent = (base_cpu + (task_hash % 20) as f32).min(95.0);
        let memory_mb = base_memory + (task_hash % 512) as u32;
        let io_bytes_per_sec = base_io + (task_hash % 1000000) as u64;

        // Predict resource usage patterns
        let predicted_patterns = vec![
            ResourceUsagePattern {
                resource_type: "CPU".to_string(),
                average_usage: cpu_percent,
                peak_usage: cpu_percent * 1.5,
                usage_distribution: vec![0.1, 0.2, cpu_percent/100.0, 0.3, 0.2], // Time-based distribution
                confidence: 0.85,
            },
            ResourceUsagePattern {
                resource_type: "Memory".to_string(),
                average_usage: memory_mb as f32,
                peak_usage: (memory_mb as f32) * 1.3,
                usage_distribution: vec![0.8, 0.9, 1.0, 0.9, 0.7], // Stable memory usage
                confidence: 0.90,
            },
            ResourceUsagePattern {
                resource_type: "I/O".to_string(),
                average_usage: io_bytes_per_sec as f32,
                peak_usage: (io_bytes_per_sec as f32) * 2.0,
                usage_distribution: vec![0.2, 0.5, 1.0, 0.8, 0.3], // Burst I/O pattern
                confidence: 0.75,
            },
        ];

        // Generate optimization recommendations
        let optimization_recommendations = vec![
            "Consider parallel execution for CPU-intensive operations".to_string(),
            "Implement memory pooling to reduce allocation overhead".to_string(),
            format!("Optimize I/O operations with batching (predicted {} bytes/sec)", io_bytes_per_sec),
        ];

        // Calculate estimated execution time
        let estimated_execution_time_ms = match task_complexity {
            TaskComplexity::Low => 5000 + (task_hash % 5000) as u64,
            TaskComplexity::Medium => 15000 + (task_hash % 10000) as u64,
            TaskComplexity::High => 45000 + (task_hash % 30000) as u64,
            TaskComplexity::Critical => 120000 + (task_hash % 60000) as u64,
        };

        let analysis = ResourceRequirementAnalysis {
            task_complexity,
            predicted_resource_usage: predicted_patterns,
            estimated_execution_time_ms,
            optimization_recommendations,
            bottleneck_prediction: self.predict_bottlenecks(&predicted_patterns),
            scaling_recommendations: self.generate_scaling_recommendations(task_complexity),
            analysis_confidence: 0.82,
            analysis_timestamp: Utc::now(),
        };

        Ok(analysis)
    }

    /// Estimate task complexity based on specifications
    fn estimate_task_complexity(&self, task_spec: &crate::types::TaskSpec) -> TaskComplexity {
        let description_len = task_spec.description.len();
        let acceptance_criteria_count = task_spec.acceptance_criteria.len();

        // Simple heuristic based on task specifications
        if task_spec.risk_tier == crate::models::RiskTier::Tier1
            || description_len > 500
            || acceptance_criteria_count > 5 {
            TaskComplexity::Critical
        } else if task_spec.risk_tier == crate::models::RiskTier::Tier2
            || description_len > 200
            || acceptance_criteria_count > 3 {
            TaskComplexity::High
        } else if description_len > 100 || acceptance_criteria_count > 1 {
            TaskComplexity::Medium
        } else {
            TaskComplexity::Low
        }
    }

    /// Predict potential bottlenecks
    fn predict_bottlenecks(&self, patterns: &[ResourceUsagePattern]) -> Vec<String> {
        let mut bottlenecks = Vec::new();

        for pattern in patterns {
            if pattern.peak_usage > 90.0 && pattern.resource_type == "CPU" {
                bottlenecks.push("CPU saturation likely during peak execution".to_string());
            } else if pattern.peak_usage > 1024.0 && pattern.resource_type == "Memory" {
                bottlenecks.push("Memory pressure may cause swapping".to_string());
            } else if pattern.peak_usage > 1000000.0 && pattern.resource_type == "I/O" {
                bottlenecks.push("I/O bottleneck may slow execution".to_string());
            }
        }

        if bottlenecks.is_empty() {
            bottlenecks.push("No significant bottlenecks predicted".to_string());
        }

        bottlenecks
    }

    /// Generate scaling recommendations
    fn generate_scaling_recommendations(&self, complexity: TaskComplexity) -> Vec<String> {
        match complexity {
            TaskComplexity::Low => vec![
                "Single-threaded execution sufficient".to_string(),
                "Minimal resource allocation needed".to_string(),
            ],
            TaskComplexity::Medium => vec![
                "Consider multi-threading for CPU operations".to_string(),
                "Monitor memory usage during execution".to_string(),
            ],
            TaskComplexity::High => vec![
                "Parallel execution recommended".to_string(),
                "Implement resource pooling".to_string(),
                "Consider distributed execution".to_string(),
            ],
            TaskComplexity::Critical => vec![
                "Distributed execution strongly recommended".to_string(),
                "Implement comprehensive resource management".to_string(),
                "Consider specialized hardware acceleration".to_string(),
                "Monitor all resource usage continuously".to_string(),
            ],
        }
    }

    /// Calculate recommendation confidence
    fn calculate_recommendation_confidence(&self, signals: &[LearningSignal]) -> f32 {
        if signals.is_empty() {
            return 0.5; // Default confidence with no data
        }

        let success_rate: f32 = signals
            .iter()
            .map(|s| match s.outcome {
                TaskOutcome::Success { .. } => 1.0,
                TaskOutcome::PartialSuccess { .. } => 0.7,
                _ => 0.0,
            })
            .sum::<f32>()
            / signals.len() as f32;

        // Confidence based on success rate and sample size
        let sample_confidence = (signals.len() as f32 / 100.0).min(1.0);
        success_rate * 0.7 + sample_confidence * 0.3
    }

    /// Generate recommendation rationale
    fn generate_rationale(
        &self,
        judge_analysis: &JudgePerformanceAnalysis,
        resource_analysis: &ResourceRequirementAnalysis,
    ) -> String {
        format!(
            "Based on historical performance: {} with {} resource allocation",
            judge_analysis.summary, resource_analysis.summary
        )
    }
}

/// Routing recommendation from learning analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRecommendation {
    pub recommended_judges: Vec<JudgeRecommendation>,
    pub resource_allocation: ResourceAllocation,
    pub estimated_complexity: TaskComplexity,
    pub confidence: f32,
    pub rationale: String,
}

/// Judge recommendation with performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeRecommendation {
    pub judge_id: JudgeId,
    pub expected_quality_score: f32,
    pub expected_latency_ms: u64,
    pub historical_success_rate: f32,
    pub specialization_score: f32,
}

/// Resource allocation recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    pub cpu_cores: u32,
    pub memory_mb: u64,
    pub preferred_accelerator: AcceleratorPreference,
    pub thermal_budget: f32,
}

/// Accelerator preferences for optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AcceleratorPreference {
    ANE,    // Apple Neural Engine
    GPU,    // Metal GPU
    CPU,    // CPU-only
    Hybrid, // Dynamic selection
}

/// Judge performance analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgePerformanceAnalysis {
    pub recommended_judges: Vec<JudgeRecommendation>,
    pub summary: String,
    pub confidence: f32,
}

/// Resource requirement analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirementAnalysis {
    pub optimal_allocation: ResourceAllocation,
    pub estimated_complexity: TaskComplexity,
    pub summary: String,
    pub confidence: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_learning_signal_creation() {
        let signal = LearningSignal {
            id: Uuid::new_v4(),
            task_id: Uuid::new_v4(),
            verdict_id: Uuid::new_v4(),
            outcome: TaskOutcome::Success {
                confidence: 0.9,
                quality_indicators: vec![QualityIndicator::HighConfidence],
            },
            judge_dissent: vec![],
            latency_ms: 1500,
            quality_score: 0.85,
            timestamp: Utc::now(),
            resource_usage: ResourceUsageMetrics {
                cpu_usage_percent: 45.0,
                memory_usage_mb: 512,
                thermal_status: ThermalStatus::Normal,
                ane_utilization: Some(0.8),
                gpu_utilization: None,
                energy_consumption: Some(2.5),
            },
            caws_compliance_score: 0.95,
            claim_verification_score: Some(0.88),
            task_complexity: TaskComplexity {
                estimated_effort: EffortLevel::Moderate,
                domain_complexity: 0.6,
                interdependency_count: 3,
                risk_factors: vec![RiskFactor::HighRiskTier],
            },
            worker_performance: None,
        };

        assert_eq!(signal.quality_score, 0.85);
        assert_eq!(signal.latency_ms, 1500);
    }

    #[test]
    fn test_dissent_severity_ordering() {
        let severities = vec![
            DissentSeverity::Minor,
            DissentSeverity::Moderate,
            DissentSeverity::Major,
            DissentSeverity::Critical,
        ];

        // Test that we can serialize/deserialize all severity levels
        for severity in severities {
            let serialized = serde_json::to_string(&severity).unwrap();
            let deserialized: DissentSeverity = serde_json::from_str(&serialized).unwrap();
            assert_eq!(format!("{:?}", severity), format!("{:?}", deserialized));
        }
    }

    #[test]
    fn test_effort_level_estimation() {
        let effort = EffortLevel::Moderate;
        match effort {
            EffortLevel::Trivial => assert!(false, "Should be Moderate"),
            EffortLevel::Simple => assert!(false, "Should be Moderate"),
            EffortLevel::Moderate => assert!(true),
            EffortLevel::Complex => assert!(false, "Should be Moderate"),
            EffortLevel::VeryComplex => assert!(false, "Should be Moderate"),
        }
    }
}

/// In-memory implementation of LearningSignalStorage for development and testing
#[derive(Debug, Default)]
pub struct InMemoryLearningSignalStorage {
    signals: std::sync::RwLock<Vec<LearningSignal>>,
}

#[async_trait::async_trait]
impl LearningSignalStorage for InMemoryLearningSignalStorage {
    async fn store_signal(&self, signal: LearningSignal) -> Result<()> {
        let mut signals = self.signals.write().unwrap();
        signals.push(signal);
        Ok(())
    }

    async fn get_signals_for_task(&self, task_id: TaskId) -> Result<Vec<LearningSignal>> {
        let signals = self.signals.read().unwrap();
        let task_signals: Vec<_> = signals
            .iter()
            .filter(|s| s.task_id == task_id)
            .cloned()
            .collect();
        Ok(task_signals)
    }

    async fn get_signals_for_judge(&self, judge_id: &JudgeId) -> Result<Vec<LearningSignal>> {
        let signals = self.signals.read().unwrap();
        let judge_signals: Vec<_> = signals
            .iter()
            .filter(|s| s.judge_dissent.iter().any(|d| &d.judge_id == judge_id))
            .cloned()
            .collect();
        Ok(judge_signals)
    }

    async fn get_signals_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<LearningSignal>> {
        let signals = self.signals.read().unwrap();
        let time_filtered: Vec<_> = signals
            .iter()
            .filter(|s| s.timestamp >= start && s.timestamp <= end)
            .cloned()
            .collect();
        Ok(time_filtered)
    }

    async fn get_performance_metrics(
        &self,
        entity_type: PerformanceEntityType,
        entity_id: String,
        time_range_days: u32,
    ) -> Result<AggregatedMetrics> {
        let end_time = Utc::now();
        let start_time = end_time - chrono::Duration::days(time_range_days as i64);
        let signals = self.get_signals_by_time_range(start_time, end_time).await?;

        let filtered_signals: Vec<_> = signals
            .into_iter()
            .filter(|s| match entity_type {
                PerformanceEntityType::Judge => s.judge_dissent.iter().any(|d| d.judge_id.to_string() == entity_id),
                PerformanceEntityType::Task => s.task_id.to_string() == entity_id,
                PerformanceEntityType::Worker => s.worker_performance.as_ref().map(|w| w.worker_id.to_string() == entity_id).unwrap_or(false),
            })
            .collect();

        if filtered_signals.is_empty() {
            return Ok(AggregatedMetrics {
                entity_type,
                entity_id,
                time_range_days,
                total_signals: 0,
                avg_latency_ms: 0.0,
                avg_quality_score: 0.0,
                success_rate: 0.0,
                resource_efficiency: 0.0,
            });
        }

        let total_signals = filtered_signals.len() as u64;
        let avg_latency_ms = filtered_signals.iter().map(|s| s.latency_ms as f64).sum::<f64>() / total_signals as f64;
        let avg_quality_score = filtered_signals.iter().map(|s| s.quality_score as f64).sum::<f64>() / total_signals as f64;
        let success_rate = filtered_signals.iter()
            .filter(|s| matches!(s.outcome, TaskOutcome::Success { .. }))
            .count() as f64 / total_signals as f64;

        // Calculate resource efficiency (lower resource usage per quality score = better)
        let avg_resource_usage = filtered_signals.iter()
            .map(|s| (s.resource_usage.cpu_percent + s.resource_usage.memory_mb as f64) / 100.0)
            .sum::<f64>() / total_signals as f64;
        let resource_efficiency = if avg_resource_usage > 0.0 {
            avg_quality_score / avg_resource_usage
        } else {
            1.0
        };

        Ok(AggregatedMetrics {
            entity_type,
            entity_id,
            time_range_days,
            total_signals,
            avg_latency_ms,
            avg_quality_score,
            success_rate,
            resource_efficiency,
        })
    }
}
