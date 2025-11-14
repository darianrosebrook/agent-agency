//! Common infrastructure for evidence collectors
//!
//! This module provides the shared foundation for all evidence collectors,
//! including the `EvidenceCollector` trait with default methods and the
//! `CollectorCtx` for execution context and configuration.

use crate::evidence::evidence_types::EvidenceCollectorConfig;
use crate::extraction_types::{AtomicClaim, Evidence, ProcessingContext};
use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tracing::{error, info, instrument, warn, Span};

/// Execution context for evidence collection
///
/// Provides common infrastructure like tracing, deadlines, configuration,
/// and metrics collection for all evidence collectors.
#[derive(Debug, Clone)]
pub struct CollectorCtx {
    /// Configuration for the collector
    pub config: EvidenceCollectorConfig,
    /// Processing context from the pipeline
    pub processing_context: ProcessingContext,
    /// Start time for timeout enforcement
    pub start_time: Instant,
    /// Deadline for collection completion
    pub deadline: Instant,
    /// Tracing span for this collection operation
    pub span: Span,
    /// Metrics sink for performance tracking
    pub metrics: CollectorMetrics,
}

impl CollectorCtx {
    /// Create a new collector context
    pub fn new(config: EvidenceCollectorConfig, processing_context: ProcessingContext) -> Self {
        let start_time = Instant::now();
        // Use a default timeout since config doesn't have timeout_seconds
        let deadline = start_time + Duration::from_secs(300); // 5 minutes default

        Self {
            config,
            processing_context,
            start_time,
            deadline,
            span: tracing::info_span!("evidence_collection"),
            metrics: CollectorMetrics::default(),
        }
    }

    /// Create a new collector context with a custom timeout (for testing)
    #[cfg(test)]
    pub fn with_timeout(
        config: EvidenceCollectorConfig,
        processing_context: ProcessingContext,
        timeout: Duration,
    ) -> Self {
        let start_time = Instant::now();
        let deadline = start_time + timeout;

        Self {
            config,
            processing_context,
            start_time,
            deadline,
            span: tracing::info_span!("evidence_collection"),
            metrics: CollectorMetrics::default(),
        }
    }

    /// Check if collection should timeout
    pub fn should_timeout(&self) -> bool {
        Instant::now() >= self.deadline
    }

    /// Get remaining time before timeout
    pub fn remaining_time(&self) -> Duration {
        self.deadline.saturating_duration_since(Instant::now())
    }

    /// Record a collection operation in metrics
    pub fn record_operation(&mut self, operation: &str, duration: Duration, success: bool) {
        self.metrics.record_operation(operation, duration, success);
    }
}

/// Metrics collected during evidence collection
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct CollectorMetrics {
    pub total_operations: usize,
    pub successful_operations: usize,
    pub failed_operations: usize,
    pub total_duration_ms: u64,
    pub average_duration_ms: f64,
}

impl CollectorMetrics {
    fn record_operation(&mut self, _operation: &str, duration: Duration, success: bool) {
        self.total_operations += 1;
        self.total_duration_ms += duration.as_millis() as u64;

        if success {
            self.successful_operations += 1;
        } else {
            self.failed_operations += 1;
        }

        self.average_duration_ms = self.total_duration_ms as f64 / self.total_operations as f64;
    }
}

/// Trait for evidence collectors with default implementations
///
/// This trait provides a standardized interface for all evidence collectors
/// with sensible defaults for common operations like validation, tracing,
/// and error handling.
#[async_trait::async_trait]
pub trait EvidenceCollector: Send + Sync {
    /// The type of input this collector processes
    type Input: Send + Sync;
    /// The type of output this collector produces
    type Output;

    /// Get the collector's name for logging and identification
    fn name(&self) -> &'static str {
        "evidence-collector"
    }

    /// Get the collector's configuration
    fn config(&self) -> &EvidenceCollectorConfig;

    /// Pre-flight validation before collection
    ///
    /// Default implementation checks basic configuration validity.
    /// Override for collector-specific validation.
    fn preflight(
        &self,
        ctx: &CollectorCtx,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        if ctx.config.min_relevance_threshold < 0.0 || ctx.config.min_relevance_threshold > 1.0 {
            return Err(
                anyhow::anyhow!("min_relevance_threshold must be between 0.0 and 1.0").into(),
            );
        }

        if ctx.config.min_credibility_threshold < 0.0 || ctx.config.min_credibility_threshold > 1.0
        {
            return Err(
                anyhow::anyhow!("min_credibility_threshold must be between 0.0 and 1.0").into(),
            );
        }

        Ok(())
    }

    /// Core evidence collection logic
    ///
    /// This method must be implemented by each collector to provide
    /// their specific evidence collection logic.
    async fn collect(
        &self,
        input: &Self::Input,
        ctx: &CollectorCtx,
    ) -> Result<Self::Output, Box<dyn std::error::Error + Send + Sync + 'static>>;

    /// Main collection entry point with common infrastructure
    ///
    /// This provides the standard collection flow:
    /// 1. Pre-flight validation
    /// 2. Tracing and timing
    /// 3. Core collection logic
    /// 4. Metrics recording
    async fn run(
        &self,
        input: &Self::Input,
        ctx: &CollectorCtx,
    ) -> Result<Self::Output, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let span = tracing::info_span!("evidence_collection", collector = %self.name());
        let _enter = span.enter();
        let start = Instant::now();

        // Pre-flight checks
        self.preflight(ctx)?;

        // Check timeout before starting
        if ctx.should_timeout() {
            warn!("Collection timed out before starting");
            return Err(anyhow::anyhow!("Collection timeout").into());
        }

        info!("Starting evidence collection");

        // Core collection with error handling
        let result = self.collect(input, ctx).await;

        // Record metrics
        let duration = start.elapsed();
        let success = result.is_ok();

        if let Ok(ref output) = result {
            info!("Collection completed successfully in {:?}", duration);
            // Additional validation could go here
        } else if let Err(ref e) = result {
            error!("Collection failed after {:?}: {}", duration, e);
        }

        // Note: We can't mutate ctx.metrics here since it's &CollectorCtx
        // In practice, collectors would have their own metrics or use a different approach

        result
    }
}

/// Common helper functions for evidence collectors
pub mod helpers {
    use super::*;

    /// Validate that evidence meets minimum quality thresholds
    pub fn validate_evidence_quality(evidence: &[Evidence], threshold: f64) -> Result<(), String> {
        let low_quality_count = evidence.iter().filter(|e| e.confidence < threshold).count();

        if low_quality_count > evidence.len() / 2 {
            return Err(format!(
                "Too many low-quality evidence items: {} out of {} below threshold {:.2}",
                low_quality_count,
                evidence.len(),
                threshold
            ));
        }

        Ok(())
    }

    /// Create a standardized evidence item with common fields
    pub fn create_evidence_base(
        claim_id: uuid::Uuid,
        evidence_type: crate::extraction_types::EvidenceType,
        content: String,
        confidence: f64,
        relevance: f64,
    ) -> Evidence {
        Evidence {
            id: uuid::Uuid::new_v4(),
            claim_id,
            evidence_type,
            content,
            source: crate::extraction_types::EvidenceSource::CodeSearch {
                location: "unknown".to_string(),
                authority: "evidence-collector".to_string(),
                freshness: chrono::Utc::now(),
            },
            confidence,
            relevance,
            timestamp: chrono::Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extraction_types::ProcessingContext;

    #[test]
    fn test_collector_ctx_timeout() {
        let config = EvidenceCollectorConfig::default();
        let ctx = CollectorCtx::with_timeout(
            config,
            ProcessingContext::default(),
            Duration::from_secs(1),
        );

        assert!(!ctx.should_timeout());
        std::thread::sleep(std::time::Duration::from_secs(2));
        assert!(ctx.should_timeout());
    }

    #[test]
    fn test_metrics_recording() {
        let mut metrics = CollectorMetrics::default();

        // Simulate some operations
        metrics.record_operation("test1", Duration::from_millis(100), true);
        metrics.record_operation("test2", Duration::from_millis(200), false);

        assert_eq!(metrics.total_operations, 2);
        assert_eq!(metrics.successful_operations, 1);
        assert_eq!(metrics.failed_operations, 1);
        assert_eq!(metrics.total_duration_ms, 300);
    }
}
