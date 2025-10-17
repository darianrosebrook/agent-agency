//! SLA (Service Level Agreement) validation system
//!
//! Validates system performance against defined SLA targets from the working spec.

use crate::types::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// SLA validator for system performance metrics
pub struct SlaValidator {
    targets: SlaTargets,
}

impl SlaValidator {
    /// Create a new SLA validator with default targets based on working spec
    pub fn new() -> Self {
        let targets = SlaTargets {
            api_p95_ms: SlaDefinition {
                name: "API Response Time (P95)".to_string(),
                target: 1000.0, // 1000ms from working spec
                unit: "milliseconds".to_string(),
                higher_is_better: false, // Lower is better for latency
                tolerance_percent: 10.0, // 10% tolerance
            },
            council_consensus_ms: SlaDefinition {
                name: "Council Consensus Time".to_string(),
                target: 5000.0, // 5000ms from working spec
                unit: "milliseconds".to_string(),
                higher_is_better: false, // Lower is better for consensus time
                tolerance_percent: 15.0, // 15% tolerance
            },
            ane_utilization_percent: SlaDefinition {
                name: "Apple Silicon ANE Utilization".to_string(),
                target: 60.0, // 60% from working spec
                unit: "percentage".to_string(),
                higher_is_better: true, // Higher is better for utilization
                tolerance_percent: 5.0, // 5% tolerance
            },
            memory_usage_gb: SlaDefinition {
                name: "Memory Usage".to_string(),
                target: 50.0, // 50GB from working spec
                unit: "gigabytes".to_string(),
                higher_is_better: false, // Lower is better for memory usage
                tolerance_percent: 10.0, // 10% tolerance
            },
        };

        Self { targets }
    }

    /// Create SLA validator with custom targets
    pub fn with_targets(targets: SlaTargets) -> Self {
        Self { targets }
    }

    /// Validate a single metric against its SLA
    pub fn validate_metric(
        &self,
        sla_def: &SlaDefinition,
        actual_value: f64,
    ) -> SlaValidationResult {
        let passed = if sla_def.higher_is_better {
            // For metrics where higher is better (e.g., utilization)
            actual_value >= sla_def.target
        } else {
            // For metrics where lower is better (e.g., latency, memory usage)
            actual_value <= sla_def.target
        };

        let deviation_percent = if sla_def.higher_is_better {
            // Calculate how much below target we are
            if actual_value < sla_def.target {
                ((sla_def.target - actual_value) / sla_def.target) * 100.0
            } else {
                0.0 // Meeting or exceeding target
            }
        } else {
            // Calculate how much above target we are
            if actual_value > sla_def.target {
                ((actual_value - sla_def.target) / sla_def.target) * 100.0
            } else {
                0.0 // Meeting or below target
            }
        };

        let severity = self.calculate_severity(&sla_def, deviation_percent);

        SlaValidationResult {
            sla: sla_def.clone(),
            actual_value,
            passed,
            deviation_percent,
            severity,
        }
    }

    /// Validate all SLA metrics against provided measurements
    pub fn validate_all(&self, measurements: &HashMap<String, f64>) -> SlaValidationReport {
        let mut sla_results = Vec::new();

        // API Response Time
        if let Some(&api_p95) = measurements.get("api_p95_ms") {
            sla_results.push(self.validate_metric(&self.targets.api_p95_ms, api_p95));
        }

        // Council Consensus Time
        if let Some(&consensus_time) = measurements.get("council_consensus_ms") {
            sla_results
                .push(self.validate_metric(&self.targets.council_consensus_ms, consensus_time));
        }

        // ANE Utilization
        if let Some(&ane_utilization) = measurements.get("ane_utilization_percent") {
            sla_results
                .push(self.validate_metric(&self.targets.ane_utilization_percent, ane_utilization));
        }

        // Memory Usage
        if let Some(&memory_usage) = measurements.get("memory_usage_gb") {
            sla_results.push(self.validate_metric(&self.targets.memory_usage_gb, memory_usage));
        }

        let overall_compliant = sla_results.iter().all(|result| result.passed);
        let passed_count = sla_results.iter().filter(|result| result.passed).count();
        let failed_count = sla_results.len() - passed_count;
        let critical_violations = sla_results
            .iter()
            .filter(|result| {
                matches!(
                    result.severity,
                    SlaViolationSeverity::Critical | SlaViolationSeverity::Catastrophic
                )
            })
            .count();

        let average_deviation_percent = if sla_results.is_empty() {
            0.0
        } else {
            sla_results
                .iter()
                .map(|result| result.deviation_percent)
                .sum::<f64>()
                / sla_results.len() as f64
        };

        let worst_violation = sla_results
            .iter()
            .filter(|result| !result.passed)
            .max_by(|a, b| {
                a.deviation_percent
                    .partial_cmp(&b.deviation_percent)
                    .unwrap()
            })
            .cloned();

        let summary = SlaSummary {
            passed_count,
            failed_count,
            critical_violations,
            average_deviation_percent,
            worst_violation,
        };

        SlaValidationReport {
            timestamp: Utc::now(),
            overall_compliant,
            sla_results,
            summary,
        }
    }

    /// Validate benchmark results against SLA targets
    pub fn validate_benchmark_results(&self, results: &[BenchmarkResult]) -> SlaValidationReport {
        let mut measurements = HashMap::new();

        // Extract performance metrics from benchmark results
        for result in results {
            // Use speed metric as proxy for API response time
            measurements.insert("api_p95_ms".to_string(), result.metrics.speed);

            // Use efficiency as proxy for resource utilization
            measurements.insert(
                "ane_utilization_percent".to_string(),
                result.metrics.efficiency * 100.0,
            );

            // Use compliance as proxy for system health
            if result.benchmark_type == BenchmarkType::ComplianceBenchmark {
                measurements.insert(
                    "council_consensus_ms".to_string(),
                    result.metrics.compliance * 1000.0,
                );
            }
        }

        self.validate_all(&measurements)
    }

    /// Get current SLA targets
    pub fn get_targets(&self) -> &SlaTargets {
        &self.targets
    }

    /// Calculate severity of SLA violation based on deviation
    fn calculate_severity(
        &self,
        sla_def: &SlaDefinition,
        deviation_percent: f64,
    ) -> SlaViolationSeverity {
        if deviation_percent == 0.0 {
            return SlaViolationSeverity::Minor; // No violation
        }

        // Critical thresholds based on tolerance
        let critical_threshold = sla_def.tolerance_percent * 3.0; // 3x tolerance
        let catastrophic_threshold = sla_def.tolerance_percent * 5.0; // 5x tolerance

        if deviation_percent >= catastrophic_threshold {
            SlaViolationSeverity::Catastrophic
        } else if deviation_percent >= critical_threshold {
            SlaViolationSeverity::Critical
        } else if deviation_percent >= sla_def.tolerance_percent {
            SlaViolationSeverity::Moderate
        } else {
            SlaViolationSeverity::Minor
        }
    }
}

impl Default for SlaValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a human-readable SLA validation report
pub fn format_sla_report(report: &SlaValidationReport) -> String {
    let mut output = String::new();

    output.push_str("🚀 SLA Validation Report\n");
    output.push_str("=======================\n\n");

    output.push_str(&format!(
        "📊 Validation Time: {}\n",
        report.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
    ));
    output.push_str(&format!(
        "✅ Overall Status: {}\n",
        if report.overall_compliant {
            "COMPLIANT"
        } else {
            "NON-COMPLIANT"
        }
    ));
    output.push_str("\n");

    output.push_str("📈 Summary:\n");
    output.push_str(&format!(
        "  • Passed: {}/{}\n",
        report.summary.passed_count,
        report.summary.passed_count + report.summary.failed_count
    ));
    output.push_str(&format!("  • Failed: {}\n", report.summary.failed_count));
    output.push_str(&format!(
        "  • Critical Violations: {}\n",
        report.summary.critical_violations
    ));
    output.push_str(&format!(
        "  • Average Deviation: {:.1}%\n",
        report.summary.average_deviation_percent
    ));
    output.push_str("\n");

    output.push_str("📋 Individual SLA Results:\n");
    for result in &report.sla_results {
        let status_icon = if result.passed { "✅" } else { "❌" };
        let severity_icon = match result.severity {
            SlaViolationSeverity::Minor => "🟡",
            SlaViolationSeverity::Moderate => "🟠",
            SlaViolationSeverity::Critical => "🔴",
            SlaViolationSeverity::Catastrophic => "💀",
        };

        output.push_str(&format!(
            "{} {} {}: {:.2}{} (target: {:.2}{})\n",
            status_icon,
            severity_icon,
            result.sla.name,
            result.actual_value,
            result.sla.unit,
            result.sla.target,
            result.sla.unit
        ));

        if !result.passed {
            output.push_str(&format!(
                "   Deviation: {:.1}% {}\n",
                result.deviation_percent,
                if result.deviation_percent > 0.0 {
                    "above target"
                } else {
                    "below target"
                }
            ));
        }
    }

    if let Some(worst) = &report.summary.worst_violation {
        output.push_str("\n");
        output.push_str("⚠️  Most Critical Violation:\n");
        output.push_str(&format!(
            "  • {}: {:.1}% deviation\n",
            worst.sla.name, worst.deviation_percent
        ));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sla_validator_creation() {
        let validator = SlaValidator::new();
        assert_eq!(validator.targets.api_p95_ms.target, 1000.0);
        assert_eq!(validator.targets.council_consensus_ms.target, 5000.0);
        assert_eq!(validator.targets.ane_utilization_percent.target, 60.0);
        assert_eq!(validator.targets.memory_usage_gb.target, 50.0);
    }

    #[test]
    fn test_metric_validation_passing() {
        let validator = SlaValidator::new();

        // Test API latency within target (lower is better)
        let result = validator.validate_metric(&validator.targets.api_p95_ms, 800.0);
        assert!(result.passed);
        assert_eq!(result.deviation_percent, 0.0);
        assert_eq!(result.severity, SlaViolationSeverity::Minor);

        // Test ANE utilization above target (higher is better)
        let result = validator.validate_metric(&validator.targets.ane_utilization_percent, 70.0);
        assert!(result.passed);
        assert_eq!(result.deviation_percent, 0.0);
    }

    #[test]
    fn test_metric_validation_failing() {
        let validator = SlaValidator::new();

        // Test API latency exceeding target (lower is better)
        let result = validator.validate_metric(&validator.targets.api_p95_ms, 1200.0);
        assert!(!result.passed);
        assert_eq!(result.deviation_percent, 20.0); // 20% over target

        // Test ANE utilization below target (higher is better)
        let result = validator.validate_metric(&validator.targets.ane_utilization_percent, 50.0);
        assert!(!result.passed);
        assert_eq!(result.deviation_percent, 16.67); // 16.67% below target
    }

    #[test]
    fn test_severity_calculation() {
        let validator = SlaValidator::new();

        // Minor violation (within tolerance)
        let result = validator.validate_metric(&validator.targets.api_p95_ms, 1050.0); // 5% over
        assert_eq!(result.severity, SlaViolationSeverity::Minor);

        // Moderate violation (exceeds tolerance)
        let result = validator.validate_metric(&validator.targets.api_p95_ms, 1100.0); // 10% over
        assert_eq!(result.severity, SlaViolationSeverity::Moderate);

        // Critical violation (3x tolerance)
        let result = validator.validate_metric(&validator.targets.api_p95_ms, 1300.0); // 30% over
        assert_eq!(result.severity, SlaViolationSeverity::Critical);

        // Catastrophic violation (5x tolerance)
        let result = validator.validate_metric(&validator.targets.api_p95_ms, 1500.0); // 50% over
        assert_eq!(result.severity, SlaViolationSeverity::Catastrophic);
    }

    #[test]
    fn test_sla_report_formatting() {
        let validator = SlaValidator::new();
        let mut measurements = HashMap::new();
        measurements.insert("api_p95_ms".to_string(), 800.0); // Good
        measurements.insert("council_consensus_ms".to_string(), 6000.0); // Bad

        let report = validator.validate_all(&measurements);
        let formatted = format_sla_report(&report);

        assert!(formatted.contains("SLA Validation Report"));
        assert!(formatted.contains("Overall Status"));
        assert!(formatted.contains("API Response Time"));
        assert!(formatted.contains("Council Consensus Time"));
    }
}
