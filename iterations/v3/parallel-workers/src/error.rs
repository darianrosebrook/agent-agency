//! Error types for the parallel worker system

use std::path::PathBuf;
use thiserror::Error;
use crate::types::{TaskId, SubTaskId, WorkerId, WorkerSpecialty};

/// Main error type for the parallel worker system
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum ParallelError {
    #[error("Task decomposition failed: {message}")]
    Decomposition { message: String, source: Option<Box<dyn std::error::Error + Send + Sync>> },

    #[error("Worker coordination failed: {message}")]
    Coordination { message: String, source: Option<Box<dyn std::error::Error + Send + Sync>> },

    #[error("Worker execution failed: {message}")]
    WorkerExecution { worker_id: WorkerId, message: String, source: Option<Box<dyn std::error::Error + Send + Sync>> },

    #[error("Worker error: {0}")]
    Worker(WorkerError),

    #[error("Progress tracking failed: {message}")]
    ProgressTracking { message: String, source: Option<Box<dyn std::error::Error + Send + Sync>> },

    #[error("Quality validation failed: {message}")]
    Validation { message: String, source: Option<Box<dyn std::error::Error + Send + Sync>> },

    #[error("Communication failed: {message}")]
    Communication { message: String, source: Option<Box<dyn std::error::Error + Send + Sync>> },

    #[error("Resource exhaustion: {resource_type} - available: {available}, required: {required}")]
    ResourceExhaustion { resource_type: String, available: u64, required: u64 },

    #[error("Timeout exceeded: {duration_secs} seconds")]
    Timeout { duration_secs: u64 },

    #[error("Task failed with {failed_workers} failed workers out of {total_workers} total")]
    TaskFailure { task_id: TaskId, failed_workers: usize, total_workers: usize },

    #[error("Invalid configuration: {message}")]
    Configuration { message: String },

    #[error("IO error: {message}")]
    Io { message: String, source: std::io::Error },

    #[error("Serialization error: {message}")]
    Serialization { message: String, source: serde_json::Error },

    #[error("Unknown error: {message}")]
    Unknown { message: String },
}

/// Error type for decomposition operations
#[derive(Error, Debug)]
pub enum DecompositionError {
    #[error("Pattern recognition failed: {message}")]
    PatternRecognition { message: String },

    #[error("Dependency analysis failed: {message}")]
    DependencyAnalysis { message: String },

    #[error("Complexity scoring failed: {message}")]
    ComplexityScoring { message: String },

    #[error("Invalid task structure: {message}")]
    InvalidTask { message: String },

    #[error("File analysis failed for {path}: {message}")]
    FileAnalysis { path: PathBuf, message: String },

    #[error("Unsupported task type: {task_type}")]
    UnsupportedTaskType { task_type: String },

    #[error("Circular dependency detected involving subtasks: {subtask_ids:?}")]
    CircularDependency { subtask_ids: Vec<SubTaskId> },
}

/// Error type for worker operations
#[derive(Error, Debug, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum WorkerError {
    #[error("Worker not found")]
    WorkerNotFound { worker_id: WorkerId },

    #[error("No specialized worker available for {specialty:?}")]
    NoSpecializedWorkerAvailable { specialty: WorkerSpecialty },

    #[error("Worker initialization failed: {message}")]
    Initialization { message: String },

    #[error("Worker execution timeout after {timeout_secs} seconds")]
    ExecutionTimeout { timeout_secs: u64 },

    #[error("Worker communication failed: {message}")]
    Communication { message: String },

    #[error("Worker resource limits exceeded: {resource_type}")]
    ResourceLimitsExceeded { resource_type: String },

    #[error("Worker specialty mismatch: expected {expected:?}, got {actual:?}")]
    SpecialtyMismatch { expected: WorkerSpecialty, actual: WorkerSpecialty },

    #[error("Worker isolation failed: {message}")]
    IsolationFailure { message: String },

    #[error("Worker cleanup failed: {message}")]
    CleanupFailure { message: String },

    #[error("Worker panic occurred: {message}")]
    WorkerPanic { message: String },

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("Execution failed: {message}")]
    ExecutionFailed { worker_id: WorkerId, message: String },

    #[error("I/O error: {message}")]
    Io {
        message: String,
        #[serde(skip)]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl Clone for WorkerError {
    fn clone(&self) -> Self {
        match self {
            WorkerError::WorkerNotFound { worker_id } => WorkerError::WorkerNotFound {
                worker_id: worker_id.clone(),
            },
            WorkerError::NoSpecializedWorkerAvailable { specialty } => WorkerError::NoSpecializedWorkerAvailable {
                specialty: specialty.clone(),
            },
            WorkerError::Initialization { message } => WorkerError::Initialization {
                message: message.clone(),
            },
            WorkerError::SpecialtyMismatch { expected, actual } => WorkerError::SpecialtyMismatch {
                expected: expected.clone(),
                actual: actual.clone(),
            },
            WorkerError::IsolationFailure { message } => WorkerError::IsolationFailure {
                message: message.clone(),
            },
            WorkerError::CleanupFailure { message } => WorkerError::CleanupFailure {
                message: message.clone(),
            },
            WorkerError::WorkerPanic { message } => WorkerError::WorkerPanic {
                message: message.clone(),
            },
            WorkerError::NotImplemented(msg) => WorkerError::NotImplemented(msg.clone()),
            WorkerError::ExecutionFailed { worker_id, message } => WorkerError::ExecutionFailed {
                worker_id: worker_id.clone(),
                message: message.clone(),
            },
            WorkerError::Io { message, .. } => WorkerError::Io {
                message: message.clone(),
                source: None, // Skip cloning the source to avoid Box<dyn Error> Clone issues
            },
            WorkerError::ExecutionTimeout { timeout_secs } => WorkerError::ExecutionTimeout {
                timeout_secs: *timeout_secs,
            },
            WorkerError::Communication { message } => WorkerError::Communication {
                message: message.clone(),
            },
            WorkerError::ResourceLimitsExceeded { resource_type } => WorkerError::ResourceLimitsExceeded {
                resource_type: resource_type.clone(),
            },
        }
    }
}

/// Error type for progress tracking operations
#[derive(Error, Debug)]
pub enum ProgressError {
    #[error("Progress update failed: {message}")]
    UpdateFailed { worker_id: WorkerId, message: String },

    #[error("Progress aggregation failed: {message}")]
    AggregationFailed { message: String },

    #[error("Progress synthesis failed: {message}")]
    SynthesisFailed { message: String },

    #[error("Invalid progress state: {message}")]
    InvalidState { message: String },

    #[error("Progress persistence failed: {message}")]
    PersistenceFailed { message: String },
}

/// Error type for quality validation operations
#[derive(Error, Debug, Clone)]
pub enum ValidationError {
    #[error("Validation gate '{gate_name}' failed: {message}")]
    GateFailed { gate_name: String, message: String },

    #[error("Validation timeout after {timeout_secs} seconds")]
    ValidationTimeout { timeout_secs: u64 },

    #[error("Quality threshold not met: required {required}, got {actual}")]
    ThresholdNotMet { required: f32, actual: f32 },

    #[error("Validation context invalid: {message}")]
    InvalidContext { message: String },

    #[error("External validation tool failed: {tool_name} - {message}")]
    ExternalToolFailure { tool_name: String, message: String },
}

/// Error type for communication operations
#[derive(Error, Debug)]
pub enum CommunicationError {
    #[error("Channel send failed: {message}")]
    ChannelSendFailed { message: String },

    #[error("Channel receive failed: {message}")]
    ChannelReceiveFailed { message: String },

    #[error("Message serialization failed: {message}")]
    SerializationFailed { message: String },

    #[error("Message deserialization failed: {message}")]
    DeserializationFailed { message: String },

    #[error("Channel buffer overflow")]
    BufferOverflow,

    #[error("Communication timeout after {timeout_secs} seconds")]
    CommunicationTimeout { timeout_secs: u64 },

    #[error("Invalid message format: {message}")]
    InvalidMessageFormat { message: String },
}

/// Error type for synthesis operations
#[derive(Error, Debug)]
pub enum SynthesisError {
    #[error("Result synthesis failed: {message}")]
    SynthesisFailed { message: String },

    #[error("Conflicting results detected: {conflict_description}")]
    ConflictingResults { conflict_description: String },

    #[error("Incomplete results: {completed}/{total} subtasks completed")]
    IncompleteResults { completed: usize, total: usize },

    #[error("Result validation failed: {message}")]
    ValidationFailed { message: String },

    #[error("Result aggregation failed: {message}")]
    AggregationFailed { message: String },
}

/// Result type aliases for cleaner code
pub type ParallelResult<T> = Result<T, ParallelError>;
pub type DecompositionResult<T> = Result<T, DecompositionError>;
pub type WorkerExecutionResult<T> = Result<T, WorkerError>;
pub type ProgressResult<T> = Result<T, ProgressError>;
pub type CommunicationResult<T> = Result<T, CommunicationError>;
pub type SynthesisResult<T> = Result<T, SynthesisError>;

// Conversion implementations
impl From<std::io::Error> for ParallelError {
    fn from(err: std::io::Error) -> Self {
        ParallelError::Io {
            message: err.to_string(),
            source: err,
        }
    }
}

impl From<serde_json::Error> for ParallelError {
    fn from(err: serde_json::Error) -> Self {
        ParallelError::Serialization {
            message: err.to_string(),
            source: err,
        }
    }
}

impl From<tokio::sync::mpsc::error::SendError<crate::types::WorkerMessage>> for ParallelError {
    fn from(err: tokio::sync::mpsc::error::SendError<crate::types::WorkerMessage>) -> Self {
        ParallelError::Communication {
            message: format!("Failed to send worker message: {}", err),
            source: None,
        }
    }
}

impl From<tokio::sync::mpsc::error::TryRecvError> for ParallelError {
    fn from(err: tokio::sync::mpsc::error::TryRecvError) -> Self {
        ParallelError::Communication {
            message: format!("Failed to receive worker message: {}", err),
            source: None,
        }
    }
}

impl From<ValidationError> for ParallelError {
    fn from(err: ValidationError) -> Self {
        ParallelError::Validation {
            message: format!("Validation error: {:?}", err),
            source: None,
        }
    }
}

impl From<SynthesisError> for ParallelError {
    fn from(err: SynthesisError) -> Self {
        ParallelError::ProgressTracking {
            message: format!("Synthesis error: {:?}", err),
            source: None,
        }
    }
}

impl From<DecompositionError> for ParallelError {
    fn from(err: DecompositionError) -> Self {
        ParallelError::Decomposition {
            message: format!("Decomposition error: {:?}", err),
            source: None,
        }
    }
}

impl From<ProgressError> for ParallelError {
    fn from(err: ProgressError) -> Self {
        ParallelError::ProgressTracking {
            message: format!("Progress error: {:?}", err),
            source: None,
        }
    }
}

impl From<WorkerError> for ParallelError {
    fn from(err: WorkerError) -> Self {
        // Use a generic error variant since WorkerExecution requires worker_id
        ParallelError::Coordination {
            message: format!("Worker error: {:?}", err),
            source: None,
        }
    }
}

// Worker error conversions
impl From<tokio::sync::mpsc::error::SendError<crate::types::WorkerMessage>> for WorkerError {
    fn from(err: tokio::sync::mpsc::error::SendError<crate::types::WorkerMessage>) -> Self {
        WorkerError::Communication {
            message: format!("Failed to send message: {}", err),
        }
    }
}

// Validation error conversions
impl From<std::io::Error> for ValidationError {
    fn from(err: std::io::Error) -> Self {
        ValidationError::ExternalToolFailure {
            tool_name: "io".to_string(),
            message: err.to_string(),
        }
    }
}
