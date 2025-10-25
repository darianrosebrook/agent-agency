//! Routing and judge selection logic
//!
//! This module provides routing recommendations and judge selection
//! algorithms based on learning signals and performance analysis.

use anyhow::Result;
use std::collections::HashMap;

use crate::types::JudgeId;
use super::types::*;
use super::analyzer::*;

/// Judge performance analysis for routing decisions
#[derive(Debug, Clone)]
pub struct JudgePerformanceAnalysis {
    pub recommended_judges: Vec<JudgeRecommendation>,
    pub confidence: f32,
    pub performance_scores: HashMap<String, f32>,
    pub specialization_match: f32,
}

/// Resource requirement analysis for allocation
#[derive(Debug, Clone)]
pub struct ResourceRequirementAnalysis {
    pub optimal_allocation: Option<ResourceAllocation>,
    pub alternative_allocations: Vec<ResourceAllocation>,
    pub resource_efficiency_score: f32,
    pub cost_estimate: Option<f32>,
}

/// Task priority levels for resource allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Resource allocation recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    pub judge_id: String,
    pub cpu_cores: u32,
    pub memory_gb: u32,
    pub memory_mb: u32,
    pub estimated_duration_ms: u64,
    pub priority: TaskPriority,
    pub accelerator_preference: Option<AcceleratorPreference>,
}

/// Accelerator preference for compute resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AcceleratorPreference {
    CPU,
    GPU,
    TPU,
    None,
}

/// Judge recommendation with performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeRecommendation {
    pub judge_id: String,
    pub specialization_score: f32,
    pub performance_score: f32,
    pub reliability_score: f32,
    pub estimated_latency_ms: u64,
    pub resource_efficiency: f32,
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

/// Routing engine for intelligent judge and resource selection
pub struct RoutingEngine {
    analyzer: LearningSignalAnalyzer,
}

impl RoutingEngine {
    /// Create a new routing engine
    pub fn new(analyzer: LearningSignalAnalyzer) -> Self {
        Self { analyzer }
    }

    /// Generate routing recommendation for a task
    pub async fn generate_routing_recommendation(
        &self,
        task_spec: &crate::types::TaskSpec,
    ) -> Result<RoutingRecommendation> {
        self.analyzer.analyze_for_routing(task_spec).await
    }

    /// Select optimal judge for a task based on performance history
    pub async fn select_optimal_judge(
        &self,
        task_spec: &crate::types::TaskSpec,
        available_judges: &[JudgeId],
    ) -> Result<JudgeRecommendation> {
        let recommendation = self.generate_routing_recommendation(task_spec).await?;

        // Find the best judge from available ones
        let mut best_judge = None;
        let mut best_score = 0.0;

        for judge_rec in &recommendation.recommended_judges {
            if available_judges.iter().any(|id| id.to_string() == judge_rec.judge_id) {
                let score = judge_rec.performance_score * judge_rec.reliability_score;
                if score > best_score {
                    best_score = score;
                    best_judge = Some(judge_rec.clone());
                }
            }
        }

        best_judge.ok_or_else(|| anyhow::anyhow!("No suitable judge found from available judges"))
    }

    /// Calculate resource requirements for a task
    pub async fn calculate_resource_requirements(
        &self,
        task_spec: &crate::types::TaskSpec,
    ) -> Result<ResourceRequirementAnalysis> {
        self.analyzer.analyze_resource_requirements(task_spec).await
    }

    /// Get judge performance metrics for monitoring
    pub async fn get_judge_performance_metrics(
        &self,
        judge_id: &JudgeId,
        time_window: TimeWindow,
    ) -> Result<JudgePerformanceMetrics> {
        // This would query the storage for judge performance data
        // Simplified implementation for now
        Ok(JudgePerformanceMetrics {
            judge_id: uuid::Uuid::new_v4(),
            execution_time_ms: 1000,
            quality_score: 0.85,
            caws_compliance: 0.9,
            claim_accuracy: Some(0.88),
            resource_efficiency: 0.8,
        })
    }
}

/// Judge performance metrics for evaluation
#[derive(Debug, Clone)]
pub struct JudgePerformanceMetrics {
    pub judge_id: uuid::Uuid,
    pub execution_time_ms: u64,
    pub quality_score: f32,
    pub caws_compliance: f32,
    pub claim_accuracy: Option<f32>,
    pub resource_efficiency: f32,
}
