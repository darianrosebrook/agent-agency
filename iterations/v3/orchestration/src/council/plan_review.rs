use std::sync::Arc;
use anyhow::Result;
use crate::planning::types::WorkingSpec;
use crate::quality::{QualityReport, SatisficingResult};

/// Plan review request to council
#[derive(Debug, Clone)]
pub struct PlanReviewRequest {
    pub task_id: String,
    pub working_spec: WorkingSpec,
    pub quality_report: Option<QualityReport>,
    pub satisficing_analysis: Option<SatisficingResult>,
    pub iteration: usize,
    pub context: String,
}

/// Plan review verdict from council
#[derive(Debug, Clone)]
pub struct PlanReviewVerdict {
    pub plan_id: String,
    pub approved: bool,
    pub feedback: String,
    pub quality_score: f64,
}

/// Plan review service trait
#[async_trait::async_trait]
pub trait PlanReviewService: Send + Sync {
    async fn review_plan(&self, plan_id: &str) -> Result<PlanReviewVerdict>;
}
