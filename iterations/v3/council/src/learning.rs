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
    Minor,      // Minor disagreement, easily resolved
    Moderate,   // Significant disagreement, requires discussion
    Major,      // Fundamental disagreement, blocks consensus
    Critical,   // Complete disagreement, system failure
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
    Trivial,    // < 5 minutes
    Simple,     // 5-30 minutes
    Moderate,   // 30 minutes - 2 hours
    Complex,    // 2-8 hours
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
    /// Create a new learning signal analyzer
    pub fn new(storage: Box<dyn LearningSignalStorage>) -> Self {
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
        
        // Generate routing recommendation
        Ok(RoutingRecommendation {
            recommended_judges: judge_performance.recommended_judges,
            resource_allocation: resource_analysis.optimal_allocation,
            estimated_complexity: resource_analysis.estimated_complexity,
            confidence: self.calculate_recommendation_confidence(&similar_signals),
            rationale: self.generate_rationale(&judge_performance, &resource_analysis),
        })
    }
    
    /// Get learning signals for similar tasks
    async fn get_similar_task_signals(
        &self,
        task_spec: &crate::types::TaskSpec,
    ) -> Result<Vec<LearningSignal>> {
        // Implementation would query storage for similar tasks based on:
        // - Risk tier
        // - Domain overlap
        // - Complexity indicators
        // - Historical performance
        todo!("Implement similar task signal retrieval")
    }
    
    /// Analyze judge performance for task type
    async fn analyze_judge_performance(
        &self,
        task_spec: &crate::types::TaskSpec,
    ) -> Result<JudgePerformanceAnalysis> {
        // Implementation would analyze historical judge performance
        todo!("Implement judge performance analysis")
    }
    
    /// Analyze resource requirements
    async fn analyze_resource_requirements(
        &self,
        task_spec: &crate::types::TaskSpec,
    ) -> Result<ResourceRequirementAnalysis> {
        // Implementation would analyze resource needs based on:
        // - Task complexity
        // - Historical resource usage
        // - Current system state
        todo!("Implement resource requirement analysis")
    }
    
    /// Calculate recommendation confidence
    fn calculate_recommendation_confidence(&self, signals: &[LearningSignal]) -> f32 {
        if signals.is_empty() {
            return 0.5; // Default confidence with no data
        }
        
        let success_rate: f32 = signals.iter()
            .map(|s| match s.outcome {
                TaskOutcome::Success { .. } => 1.0,
                TaskOutcome::PartialSuccess { .. } => 0.7,
                _ => 0.0,
            })
            .sum::<f32>() / signals.len() as f32;
        
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
            judge_analysis.summary,
            resource_analysis.summary
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

