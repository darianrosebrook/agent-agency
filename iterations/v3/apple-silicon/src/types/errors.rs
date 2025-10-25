//! Error types for Apple Silicon operations

use serde::{Deserialize, Serialize};

/// Core ML specific errors for Apple Silicon operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoreMLError {
    /// Invalid input data or parameters provided to Core ML
    InvalidInput(String),
    /// Unsupported format or feature requested
    UnsupportedFormat(String),
    /// Unsupported data type for the operation
    UnsupportedDataType(String),
    /// Unsupported output type requested
    UnsupportedOutputType(String),
    /// Memory allocation or management error
    MemoryError(String),
    /// System-level error preventing operation
    SystemError(String),
    /// Parsing or serialization error
    ParsingError(String),
    /// Validation error for input parameters
    ValidationError(String),
}

impl std::fmt::Display for CoreMLError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoreMLError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            CoreMLError::UnsupportedFormat(msg) => write!(f, "Unsupported format: {}", msg),
            CoreMLError::UnsupportedDataType(msg) => write!(f, "Unsupported data type: {}", msg),
            CoreMLError::UnsupportedOutputType(msg) => write!(f, "Unsupported output type: {}", msg),
            CoreMLError::MemoryError(msg) => write!(f, "Memory error: {}", msg),
            CoreMLError::SystemError(msg) => write!(f, "System error: {}", msg),
            CoreMLError::ParsingError(msg) => write!(f, "Parsing error: {}", msg),
            CoreMLError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl std::error::Error for CoreMLError {}

/// Apple Neural Engine specific errors
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ANEError {
    /// ANE is not available on this device
    NotAvailable(String),
    /// ANE operation failed
    OperationFailed(String),
    /// ANE model compilation failed
    CompilationFailed(String),
    /// ANE performance requirements not met
    PerformanceError(String),
    /// ANE memory constraints exceeded
    MemoryConstraints(String),
}

impl std::fmt::Display for ANEError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ANEError::NotAvailable(msg) => write!(f, "ANE not available: {}", msg),
            ANEError::OperationFailed(msg) => write!(f, "ANE operation failed: {}", msg),
            ANEError::CompilationFailed(msg) => write!(f, "ANE compilation failed: {}", msg),
            ANEError::PerformanceError(msg) => write!(f, "ANE performance error: {}", msg),
            ANEError::MemoryConstraints(msg) => write!(f, "ANE memory constraints: {}", msg),
        }
    }
}

impl std::error::Error for ANEError {}

/// Metal GPU specific errors
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetalError {
    /// Metal framework not available
    NotAvailable(String),
    /// GPU memory allocation failed
    MemoryAllocationFailed(String),
    /// Shader compilation failed
    ShaderCompilationFailed(String),
    /// Command buffer execution failed
    CommandBufferFailed(String),
    /// GPU device lost or unavailable
    DeviceLost(String),
}

impl std::fmt::Display for MetalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetalError::NotAvailable(msg) => write!(f, "Metal not available: {}", msg),
            MetalError::MemoryAllocationFailed(msg) => write!(f, "Metal memory allocation failed: {}", msg),
            MetalError::ShaderCompilationFailed(msg) => write!(f, "Metal shader compilation failed: {}", msg),
            MetalError::CommandBufferFailed(msg) => write!(f, "Metal command buffer failed: {}", msg),
            MetalError::DeviceLost(msg) => write!(f, "Metal device lost: {}", msg),
        }
    }
}

impl std::error::Error for MetalError {}
