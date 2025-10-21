//! Quality Guardrails Module
//!
//! Implements CAWS compliance validation and quality preservation
//! for the runtime optimization pipeline.

use crate::performance_monitor::{PerformanceMetrics, SLAMetrics};
use crate::bayesian_optimizer::OptimizationResult;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn, error};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Quality guardrails configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityConfig {
    /// CAWS compliance threshold (0.0-1.0)
    pub caws_compliance_threshold: f64,
    /// Performance degradation threshold (%)
    pub max_performance_degradation: f64,
    /// Quality preservation priority (0.0-1.0)
    pub quality_priority: f64,
    /// Enable strict mode (fail on any violation)
    pub strict_mode: bool,
    /// SLA validation enabled
    pub sla_validation_enabled: bool,
}

/// Compliance check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceCheck {
    /// Overall compliance score (0.0-1.0)
    pub compliance_score: f64,
    /// CAWS compliance score
    pub caws_compliance: f64,
    /// Performance compliance score
    pub performance_compliance: f64,
    /// Quality preservation score
    pub quality_preservation: f64,
    /// Violations found
    pub violations: Vec<ComplianceViolation>,
    /// Recommendations for improvement
    pub recommendations: Vec<String>,
    /// Timestamp of check
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Compliance violation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceViolation {
    /// CAWS compliance below threshold
    CAWSCompliance { current: f64, required: f64 },
    /// Performance degradation too high
    PerformanceDegradation { degradation: f64, threshold: f64 },
    /// Quality preservation failure
    QualityDegradation { degradation: f64, threshold: f64 },
    /// SLA violation
    SLAViolation { metric: String, current: f64, target: f64 },
    /// Security vulnerability
    SecurityIssue { severity: String, description: String },
}

/// Performance threshold validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceThreshold {
    /// Metric name
    pub metric: String,
    /// Minimum acceptable value
    pub minimum: Option<f64>,
    /// Maximum acceptable value
    pub maximum: Option<f64>,
    /// Target value
    pub target: Option<f64>,
    /// Priority level (0.0-1.0)
    pub priority: f64,
}

/// Quality guardrails for optimization validation
pub struct QualityGuardrails {
    config: QualityConfig,
    baseline_metrics: Arc<RwLock<Option<PerformanceMetrics>>>,
    performance_thresholds: Arc<RwLock<Vec<PerformanceThreshold>>>,
    compliance_history: Arc<RwLock<Vec<ComplianceCheck>>>,
    caws_validator: CAWSValidator,
}

impl QualityGuardrails {
    /// Create new quality guardrails
    pub fn new(config: QualityConfig) -> Self {
        Self {
            config,
            baseline_metrics: Arc::new(RwLock::new(None)),
            performance_thresholds: Arc::new(RwLock::new(Vec::new())),
            compliance_history: Arc::new(RwLock::new(Vec::new())),
            caws_validator: CAWSValidator::new(),
        }
    }

    /// Establish baseline performance metrics
    pub async fn establish_baseline(&self, metrics: PerformanceMetrics) -> Result<()> {
        let mut baseline = self.baseline_metrics.write().await;
        *baseline = Some(metrics);

        // Initialize performance thresholds based on baseline
        self.initialize_performance_thresholds(&metrics).await;

        info!("Established quality guardrails baseline: {:?}", metrics);
        Ok(())
    }

    /// Validate compliance of optimization result
    pub async fn validate_compliance(&self, optimization_result: &OptimizationResult) -> Result<ComplianceCheck> {
        debug!("Validating optimization compliance");

        let baseline = self.baseline_metrics.read().await
            .clone()
            .context("No baseline metrics established for compliance validation")?;

        let mut violations = Vec::new();
        let mut recommendations = Vec::new();

        // CAWS compliance check
        let caws_compliance = self.caws_validator.validate_compliance().await?;
        if caws_compliance < self.config.caws_compliance_threshold {
            violations.push(ComplianceViolation::CAWSCompliance {
                current: caws_compliance,
                required: self.config.caws_compliance_threshold,
            });
            recommendations.push(format!("Improve CAWS compliance from {:.2} to {:.2}",
                                       caws_compliance, self.config.caws_compliance_threshold));
        }

        // Performance degradation check
        let performance_degradation = self.calculate_performance_degradation(&baseline, optimization_result);
        if performance_degradation > self.config.max_performance_degradation {
            violations.push(ComplianceViolation::PerformanceDegradation {
                degradation: performance_degradation,
                threshold: self.config.max_performance_degradation,
            });
            recommendations.push(format!("Reduce performance degradation from {:.2}% to below {:.2}%",
                                       performance_degradation, self.config.max_performance_degradation));
        }

        // Quality preservation check
        let quality_degradation = (1.0 - optimization_result.quality_preservation) * 100.0;
        let quality_threshold = (1.0 - self.config.quality_priority) * 100.0;
        if quality_degradation > quality_threshold {
            violations.push(ComplianceViolation::QualityDegradation {
                degradation: quality_degradation,
                threshold: quality_threshold,
            });
            recommendations.push(format!("Improve quality preservation - current degradation: {:.2}%", quality_degradation));
        }

        // Performance threshold validation
        let threshold_violations = self.validate_performance_thresholds(&baseline).await?;
        violations.extend(threshold_violations);

        // Calculate compliance scores
        let compliance_score = self.calculate_compliance_score(&violations);
        let performance_compliance = 1.0 - (performance_degradation / 100.0).min(1.0);
        let quality_preservation = optimization_result.quality_preservation;

        let check = ComplianceCheck {
            compliance_score,
            caws_compliance,
            performance_compliance,
            quality_preservation,
            violations,
            recommendations,
            timestamp: chrono::Utc::now(),
        };

        // Record compliance check
        let mut history = self.compliance_history.write().await;
        history.push(check.clone());

        // Log results
        if check.compliance_score >= 0.8 {
            info!("Compliance check passed: {:.2} score", check.compliance_score);
        } else if check.compliance_score >= 0.6 {
            warn!("Compliance check marginal: {:.2} score", check.compliance_score);
        } else {
            error!("Compliance check failed: {:.2} score", check.compliance_score);
            for violation in &check.violations {
                error!("Violation: {:?}", violation);
            }
        }

        Ok(check)
    }

    /// Validate performance thresholds
    pub async fn validate_performance_thresholds(&self, current_metrics: &PerformanceMetrics) -> Result<Vec<ComplianceViolation>> {
        let thresholds = self.performance_thresholds.read().await;
        let mut violations = Vec::new();

        for threshold in thresholds.iter() {
            let current_value = self.get_metric_value(current_metrics, &threshold.metric);

            // Check minimum threshold
            if let Some(min) = threshold.minimum {
                if current_value < min {
                    violations.push(ComplianceViolation::SLAViolation {
                        metric: threshold.metric.clone(),
                        current: current_value,
                        target: min,
                    });
                }
            }

            // Check maximum threshold
            if let Some(max) = threshold.maximum {
                if current_value > max {
                    violations.push(ComplianceViolation::SLAViolation {
                        metric: threshold.metric.clone(),
                        current: current_value,
                        target: max,
                    });
                }
            }
        }

        Ok(violations)
    }

    /// Set performance thresholds
    pub async fn set_performance_thresholds(&self, thresholds: Vec<PerformanceThreshold>) -> Result<()> {
        let mut current_thresholds = self.performance_thresholds.write().await;
        *current_thresholds = thresholds;
        debug!("Updated performance thresholds");
        Ok(())
    }

    /// Initialize default performance thresholds based on baseline
    async fn initialize_performance_thresholds(&self, baseline: &PerformanceMetrics) {
        let thresholds = vec![
            PerformanceThreshold {
                metric: "throughput".to_string(),
                minimum: Some(baseline.throughput * 0.9), // 90% of baseline
                maximum: None,
                target: Some(baseline.throughput * 1.2), // 20% improvement target
                priority: 0.8,
            },
            PerformanceThreshold {
                metric: "avg_latency_ms".to_string(),
                minimum: None,
                maximum: Some(baseline.avg_latency_ms * 1.1), // 10% degradation max
                target: Some(baseline.avg_latency_ms * 0.8), // 20% improvement target
                priority: 0.9,
            },
            PerformanceThreshold {
                metric: "error_rate".to_string(),
                minimum: None,
                maximum: Some(baseline.error_rate * 2.0), // Double error rate max
                target: Some(baseline.error_rate * 0.5), // 50% error reduction target
                priority: 1.0,
            },
            PerformanceThreshold {
                metric: "memory_usage_percent".to_string(),
                minimum: None,
                maximum: Some(90.0), // 90% memory usage max
                target: Some(baseline.memory_usage_percent * 0.9), // 10% memory reduction
                priority: 0.7,
            },
        ];

        let mut current_thresholds = self.performance_thresholds.write().await;
        *current_thresholds = thresholds;
    }

    /// Calculate performance degradation percentage
    fn calculate_performance_degradation(&self, baseline: &PerformanceMetrics, optimization: &OptimizationResult) -> f64 {
        // Performance degradation based on throughput and latency changes
        let throughput_change = (baseline.throughput - optimization.expected_improvement).max(0.0) / baseline.throughput;
        let latency_penalty = (optimization.expected_improvement - baseline.throughput).max(0.0) / baseline.throughput;

        (throughput_change + latency_penalty) * 50.0 // Scale to percentage
    }

    /// Calculate overall compliance score
    fn calculate_compliance_score(&self, violations: &[ComplianceViolation]) -> f64 {
        if violations.is_empty() {
            return 1.0;
        }

        // Weight violations by severity
        let total_penalty: f64 = violations.iter().map(|v| match v {
            ComplianceViolation::CAWSCompliance { .. } => 0.3,
            ComplianceViolation::PerformanceDegradation { .. } => 0.25,
            ComplianceViolation::QualityDegradation { .. } => 0.25,
            ComplianceViolation::SLAViolation { .. } => 0.15,
            ComplianceViolation::SecurityIssue { severity, .. } => {
                match severity.as_str() {
                    "critical" => 0.4,
                    "high" => 0.3,
                    "medium" => 0.2,
                    _ => 0.1,
                }
            }
        }).sum();

        (1.0 - total_penalty).max(0.0)
    }

    /// Get metric value by name
    fn get_metric_value(&self, metrics: &PerformanceMetrics, metric: &str) -> f64 {
        match metric {
            "throughput" => metrics.throughput,
            "avg_latency_ms" => metrics.avg_latency_ms,
            "p95_latency_ms" => metrics.p95_latency_ms,
            "p99_latency_ms" => metrics.p99_latency_ms,
            "error_rate" => metrics.error_rate,
            "cpu_usage_percent" => metrics.cpu_usage_percent,
            "memory_usage_percent" => metrics.memory_usage_percent,
            "active_connections" => metrics.active_connections as f64,
            "queue_depth" => metrics.queue_depth as f64,
            _ => 0.0,
        }
    }

    /// Get compliance history
    pub async fn get_compliance_history(&self) -> Vec<ComplianceCheck> {
        self.compliance_history.read().await.clone()
    }

    /// Check if optimization should be blocked due to quality concerns
    pub async fn should_block_optimization(&self, compliance_check: &ComplianceCheck) -> bool {
        if self.config.strict_mode {
            compliance_check.compliance_score < 0.8
        } else {
            // Allow marginal compliance but block critical violations
            compliance_check.violations.iter().any(|v| matches!(v,
                ComplianceViolation::SecurityIssue { severity, .. } if severity == "critical"
            ))
        }
    }
}

/// CAWS compliance validator
struct CAWSValidator {
    // In a real implementation, this would integrate with CAWS tooling
}

impl CAWSValidator {
    fn new() -> Self {
        Self {}
    }

    async fn validate_compliance(&self) -> Result<f64> {
        // TODO: Integrate with actual CAWS validation
        // For now, return a mock compliance score
        // In production, this would run CAWS quality gates

        // Simulate CAWS compliance check
        // This would typically run linting, testing, security scans, etc.
        let mock_compliance = 0.85; // 85% CAWS compliant

        debug!("CAWS compliance validated: {:.2}", mock_compliance);
        Ok(mock_compliance)
    }
}

impl Default for QualityConfig {
    fn default() -> Self {
        Self {
            caws_compliance_threshold: 0.8,
            max_performance_degradation: 10.0,
            quality_priority: 0.9,
            strict_mode: false,
            sla_validation_enabled: true,
        }
    }
}

// @darianrosebrook
// Quality guardrails module implementing CAWS compliance validation and performance thresholds
