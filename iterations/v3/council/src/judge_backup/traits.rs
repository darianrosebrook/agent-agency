//! Judge trait definitions and interfaces
//!
//! Core judge interface for evaluating working specifications
//! with standardized methods for review, health monitoring,
//! and specialization scoring.

use crate::error::CouncilResult;
use crate::judge_backup::types::{JudgeConfig, JudgeHealthMetrics, ReviewContext};
use crate::judge_backup::verdicts::JudgeVerdict;

/// Core judge trait for evaluating working specifications
#[async_trait::async_trait]
pub trait Judge: Send + Sync + std::fmt::Debug {
    /// Get the judge's configuration
    fn config(&self) -> &JudgeConfig;

    /// Review a working specification and return a verdict
    async fn review_spec(
        &self,
        context: &ReviewContext,
    ) -> CouncilResult<JudgeVerdict>;

    /// Get the judge's specialization score for a given context
    fn specialization_score(&self, context: &ReviewContext) -> f64;

    /// Check if the judge is available for review
    fn is_available(&self) -> bool;

    /// Get judge health metrics
    fn health_metrics(&self) -> JudgeHealthMetrics;
}


