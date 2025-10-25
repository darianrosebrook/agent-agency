//! Error handling types for intelligent testing

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Error handling result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorHandlingResult {
    pub error_detection: ErrorDetection,
    pub error_classification: ErrorClassification,
    pub error_recovery: ErrorRecovery,
    pub error_reporting: ErrorReporting,
    pub error_handling_effectiveness: f64,
}

/// Error recovery result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorRecoveryResult {
    pub automatic_recovery: AutomaticRecovery,
    pub fallback_strategies: Vec<FallbackStrategy>,
    pub manual_intervention: ManualIntervention,
    pub recovery_verification: RecoveryVerification,
}

/// Error classification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorClassificationResult {
    pub error_types: Vec<ErrorType>,
    pub error_severity: Vec<ErrorSeverity>,
    pub error_impact: Vec<ErrorImpact>,
    pub error_priority: Vec<ErrorPriority>,
    pub classification_confidence: f64,
}

/// Syntactic error in NLP processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntacticError {
    pub error_id: Uuid,
    pub error_type: String,
    pub error_description: String,
    pub error_position: usize,
}

/// Error detection component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetection {
    pub detection_id: Uuid,
    pub detected_errors: Vec<DetectedError>,
    pub detection_confidence: f64,
}

/// Detected error instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedError {
    pub error_id: Uuid,
    pub error_type: String,
    pub error_description: String,
    pub error_severity: String,
}

/// Error classification component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorClassification {
    pub classification_id: Uuid,
    pub error_categories: Vec<String>,
    pub classification_confidence: f64,
}

/// Error recovery component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorRecovery {
    pub recovery_id: Uuid,
    pub recovery_attempts: Vec<RecoveryAttempt>,
    pub recovery_success_rate: f64,
}

/// Recovery attempt record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryAttempt {
    pub attempt_id: Uuid,
    pub attempt_timestamp: DateTime<Utc>,
    pub recovery_strategy: String,
    pub success: bool,
    pub duration_ms: u64,
}

/// Error reporting component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorReporting {
    pub report_id: Uuid,
    pub error_summary: String,
    pub error_details: Vec<String>,
    pub reporting_timestamp: DateTime<Utc>,
}

/// Error type classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorType {
    pub type_id: Uuid,
    pub type_name: String,
    pub type_description: String,
}

/// Error severity level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorSeverity {
    pub severity_id: Uuid,
    pub severity_level: String,
    pub severity_description: String,
}

/// Error impact assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorImpact {
    pub impact_id: Uuid,
    pub impact_level: String,
    pub impact_description: String,
}

/// Error priority ranking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPriority {
    pub priority_id: Uuid,
    pub priority_level: String,
    pub priority_description: String,
}

/// Automatic recovery component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomaticRecovery {
    pub recovery_attempts: Vec<RecoveryAttempt>,
    pub success_rate: f64,
    pub average_recovery_time_ms: u64,
}

/// Fallback strategy specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackStrategy {
    pub strategy_id: Uuid,
    pub strategy_name: String,
    pub strategy_description: String,
    pub effectiveness_score: f64,
}

/// Manual intervention record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualIntervention {
    pub intervention_id: Uuid,
    pub intervention_timestamp: DateTime<Utc>,
    pub intervention_type: String,
    pub intervention_description: String,
    pub resolution_time_ms: u64,
}

/// Recovery verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryVerification {
    pub verification_id: Uuid,
    pub verification_timestamp: DateTime<Utc>,
    pub verification_result: bool,
    pub verification_details: String,
}
