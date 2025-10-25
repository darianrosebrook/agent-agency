//! Learning signal analysis and pattern recognition
//!
//! This module provides analysis capabilities for learning signals,
//! including judge performance analysis, task complexity estimation,
//! and routing recommendations.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::types::{JudgeId, TaskId};
use agent_agency_database::DatabaseClient;
use super::types::*;
use super::storage::{LearningSignalStorage, HistoricalResourceData};

/// Learning signal analyzer for adaptive routing
#[derive(Debug)]
pub struct LearningSignalAnalyzer {
    storage: Box<dyn LearningSignalStorage>,
    db_client: Option<DatabaseClient>,
}

/// Judge performance patterns for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgePerformancePatterns {
    pub total_judges: usize,
    pub average_accuracy: f32,
    pub average_consistency: f32,
    pub performance_trends: Vec<String>,
    pub recommendations: Vec<String>,
    pub consistency_patterns: Vec<String>,
    pub accuracy_trends: Vec<f32>,
    pub specialization_areas: Vec<String>,
    pub improvement_opportunities: Vec<String>,
}

/// Aggregated judge performance data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedJudgeData {
    pub total_judges: usize,
    pub total_tasks: u64,
    pub average_accuracy: f32,
    pub specialization_score: f32,
    pub reliability_index: f32,
    pub recent_performance_trend: String,
    pub performance_distribution: String,
    pub quality_metrics: Vec<String>,
    pub total_evaluations: u32,
}

impl Default for LearningSignalAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl LearningSignalAnalyzer {
    /// Create a new learning signal analyzer with in-memory storage
    pub fn new() -> Self {
        Self {
            storage: Box::new(super::storage::InMemoryLearningSignalStorage::default()),
            db_client: None,
        }
    }

    /// Create a learning signal analyzer with custom storage
    pub fn with_storage(storage: Box<dyn LearningSignalStorage>) -> Self {
        Self {
            storage,
            db_client: None,
        }
    }

    /// Create a learning signal analyzer with database client
    pub fn with_database_client(db_client: DatabaseClient) -> Self {
        Self {
            storage: Box::new(super::storage::InMemoryLearningSignalStorage::default()),
            db_client: Some(db_client),
        }
    }

    /// Create a learning signal analyzer with both custom storage and database client
    pub fn with_storage_and_database(
        storage: Box<dyn LearningSignalStorage>,
        db_client: DatabaseClient,
    ) -> Self {
        Self {
            storage,
            db_client: Some(db_client),
        }
    }

    /// Estimate task complexity using multi-factor analysis
    pub fn estimate_task_complexity(&self, task_spec: &crate::types::TaskSpec) -> TaskComplexity {
        let mut complexity_score = 0.0;

        // Risk tier factor
        complexity_score += match task_spec.risk_tier {
            crate::types::RiskTier::Tier1 => 0.8,
            crate::types::RiskTier::Tier2 => 0.5,
            crate::types::RiskTier::Tier3 => 0.2,
        };

        // Description length factor
        let desc_length = task_spec.description.len() as f32;
        complexity_score += (desc_length / 1000.0).min(0.3);

        // Title complexity factor
        let title_complexity = task_spec.title.split_whitespace().count() as f32;
        complexity_score += (title_complexity / 20.0).min(0.2);

        // Complexity indicators
        let indicators = Self::count_complexity_indicators(&task_spec.description);
        complexity_score += (indicators as f32 / 10.0).min(0.4);

        // Determine complexity level
        if complexity_score >= 0.8 {
            TaskComplexity::VeryComplex
        } else if complexity_score >= 0.6 {
            TaskComplexity::Complex
        } else if complexity_score >= 0.4 {
            TaskComplexity::Moderate
        } else {
            TaskComplexity::Simple
        }
    }

    /// Count complexity indicators in description
    fn count_complexity_indicators(description: &str) -> u32 {
        let indicators = [
            "complex", "difficult", "challenging", "critical", "urgent",
            "breaking", "migration", "security", "performance", "optimization",
            "refactor", "restructure", "redesign", "rewrite", "overhaul"
        ];

        indicators.iter()
            .map(|indicator| description.to_lowercase().matches(indicator).count() as u32)
            .sum()
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
        let resource_allocation = resource_analysis.optimal_allocation.unwrap_or_else(|| ResourceAllocation {
            judge_id: recommended_judges.first().map(|j| j.judge_id.clone()).unwrap_or_else(|| "default".to_string()),
            cpu_cores: 2,
            memory_gb: 4,
            memory_mb: 4096,
            estimated_duration_ms: 30000,
            priority: TaskPriority::Medium,
            accelerator_preference: None,
        });

        let estimated_complexity = self.estimate_task_complexity(task_spec);
        let confidence = judge_performance.confidence;

        Ok(RoutingRecommendation {
            recommended_judges,
            resource_allocation,
            estimated_complexity,
            confidence,
            rationale,
        })
    }

    /// Get similar task signals for analysis
    async fn get_similar_task_signals(&self, task_spec: &crate::types::TaskSpec) -> Result<Vec<LearningSignal>> {
        // This would implement similarity matching - simplified for now
        Ok(vec![])
    }

    /// Analyze judge performance for task type
    async fn analyze_judge_performance(&self, task_spec: &crate::types::TaskSpec) -> Result<JudgePerformanceAnalysis> {
        // Simplified implementation
        Ok(JudgePerformanceAnalysis {
            recommended_judges: vec![],
            confidence: 0.8,
            performance_scores: HashMap::new(),
            specialization_match: 0.7,
        })
    }

    /// Analyze resource requirements
    async fn analyze_resource_requirements(&self, task_spec: &crate::types::TaskSpec) -> Result<ResourceRequirementAnalysis> {
        // Simplified implementation
        Ok(ResourceRequirementAnalysis {
            optimal_allocation: Some(ResourceAllocation {
                judge_id: "default".to_string(),
                cpu_cores: 2,
                memory_gb: 4,
                memory_mb: 4096,
                estimated_duration_ms: 30000,
                priority: TaskPriority::Medium,
                accelerator_preference: None,
            }),
            alternative_allocations: vec![],
            resource_efficiency_score: 0.8,
            cost_estimate: None,
        })
    }

    /// Generate rationale for routing decision
    fn generate_rationale(&self, judge_performance: &JudgePerformanceAnalysis, resource_analysis: &ResourceRequirementAnalysis) -> String {
        format!(
            "Routing decision based on judge performance confidence ({:.2}) and resource efficiency ({:.2})",
            judge_performance.confidence, resource_analysis.resource_efficiency_score
        )
    }
}

/// Judge performance analysis results
#[derive(Debug, Clone)]
pub struct JudgePerformanceAnalysis {
    pub recommended_judges: Vec<JudgeRecommendation>,
    pub confidence: f32,
    pub performance_scores: HashMap<String, f32>,
    pub specialization_match: f32,
}

/// Resource requirement analysis results
#[derive(Debug, Clone)]
pub struct ResourceRequirementAnalysis {
    pub optimal_allocation: Option<ResourceAllocation>,
    pub alternative_allocations: Vec<ResourceAllocation>,
    pub resource_efficiency_score: f32,
    pub cost_estimate: Option<f32>,
}

/// Task priority levels
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

/// Accelerator preference for resource allocation
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
