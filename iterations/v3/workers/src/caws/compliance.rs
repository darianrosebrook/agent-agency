//! CAWS compliance and validation types
//!
//! This module handles CAWS waivers, validation results, and compliance checking.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// CAWS waiver for exceptional circumstances
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CawsWaiver {
    pub id: String,
    pub reason: String,
    pub justification: String,
    pub time_bounded: bool,
    pub expires_at: Option<DateTime<Utc>>,
}

/// CAWS validation result
#[derive(Debug, Clone)]
pub struct CawsValidationResult {
    pub is_compliant: bool,
    pub compliance_score: f32,
    pub violations: Vec<CawsViolation>,
    pub warnings: Vec<String>,
    pub suggestions: Vec<String>,
    pub validated_at: DateTime<Utc>,
}

/// CAWS violation
#[derive(Debug, Clone)]
pub struct CawsViolation {
    pub id: String,
    pub rule_id: String,
    pub severity: ViolationSeverity,
    pub message: String,
    pub file_path: String,
    pub line: u32,
    pub column: u32,
    pub suggestion: Option<String>,
    pub constitutional_reference: Option<String>,
}

/// Violation severity levels
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl CawsWaiver {
    /// Create a new waiver
    pub fn new(id: String, reason: String, justification: String) -> Self {
        Self {
            id,
            reason,
            justification,
            time_bounded: false,
            expires_at: None,
        }
    }

    /// Check if waiver is still valid
    pub fn is_valid(&self) -> bool {
        if !self.time_bounded {
            return true;
        }
        
        if let Some(expires_at) = self.expires_at {
            Utc::now() < expires_at
        } else {
            true
        }
    }
}

impl CawsValidationResult {
    /// Create a new validation result
    pub fn new() -> Self {
        Self {
            is_compliant: true,
            compliance_score: 1.0,
            violations: vec![],
            warnings: vec![],
            suggestions: vec![],
            validated_at: Utc::now(),
        }
    }

    /// Calculate compliance score based on violations
    pub fn calculate_score(&mut self) {
        if self.violations.is_empty() {
            self.compliance_score = 1.0;
            return;
        }

        let critical_count = self.violations.iter()
            .filter(|v| matches!(v.severity, ViolationSeverity::Critical))
            .count();
        let high_count = self.violations.iter()
            .filter(|v| matches!(v.severity, ViolationSeverity::High))
            .count();
        let medium_count = self.violations.iter()
            .filter(|v| matches!(v.severity, ViolationSeverity::Medium))
            .count();

        // Weighted scoring: Critical = -0.4, High = -0.2, Medium = -0.1
        let penalty = (critical_count as f32 * 0.4) + (high_count as f32 * 0.2) + (medium_count as f32 * 0.1);
        self.compliance_score = (1.0 - penalty).max(0.0);
        self.is_compliant = self.compliance_score >= 0.8;
    }
}
