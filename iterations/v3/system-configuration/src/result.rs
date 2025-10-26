//! Common result abstractions for validation, testing, and operation outcomes

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Common trait for all result types that have success/failure status
pub trait ResultStatus {
    fn is_success(&self) -> bool;
    fn is_failure(&self) -> bool {
        !self.is_success()
    }
}

/// Common trait for results that can be scored
pub trait ScoredResult {
    fn score(&self) -> f32;
    fn score_normalized(&self) -> f32 {
        self.score().clamp(0.0, 1.0)
    }
}

/// Common trait for results that track timing
pub trait TimedResult {
    fn timestamp(&self) -> DateTime<Utc>;
    fn duration_ms(&self) -> Option<u64>;
}

/// Generic validation result that can be used across domains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonValidationResult<T = String> {
    pub is_valid: bool,
    pub score: Option<f32>,
    pub errors: Vec<T>,
    pub warnings: Vec<T>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}

impl<T> ResultStatus for CommonValidationResult<T> {
    fn is_success(&self) -> bool {
        self.is_valid
    }
}

impl<T> ScoredResult for CommonValidationResult<T> {
    fn score(&self) -> f32 {
        self.score.unwrap_or(if self.is_valid { 1.0 } else { 0.0 })
    }
}

impl<T> TimedResult for CommonValidationResult<T> {
    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn duration_ms(&self) -> Option<u64> {
        None // Could be added as a field if needed
    }
}

/// Generic test result abstraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonTestResult<T = String> {
    pub passed: bool,
    pub score: Option<f32>,
    pub errors: Vec<T>,
    pub warnings: Vec<T>,
    pub execution_time_ms: u64,
    pub metadata: HashMap<String, serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}

impl<T> ResultStatus for CommonTestResult<T> {
    fn is_success(&self) -> bool {
        self.passed
    }
}

impl<T> ScoredResult for CommonTestResult<T> {
    fn score(&self) -> f32 {
        self.score.unwrap_or(if self.passed { 1.0 } else { 0.0 })
    }
}

impl<T> TimedResult for CommonTestResult<T> {
    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn duration_ms(&self) -> Option<u64> {
        Some(self.execution_time_ms)
    }
}

/// Operation result for tracking outcomes of operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResult<T = serde_json::Value, E = String> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<E>,
    pub execution_time_ms: u64,
    pub metadata: HashMap<String, serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}

impl<T, E> ResultStatus for OperationResult<T, E> {
    fn is_success(&self) -> bool {
        self.success
    }
}

impl<T, E> TimedResult for OperationResult<T, E> {
    fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    fn duration_ms(&self) -> Option<u64> {
        Some(self.execution_time_ms)
    }
}
