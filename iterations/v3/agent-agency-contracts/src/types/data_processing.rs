//! Data Processing Types - DTOs for data processing operations
//!
//! Defines the data transfer objects used by the data processing service.
//! These types enable clean communication between data processing consumers and providers.
//!
//! @author @darianrosebrook

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// Supported data formats for processing
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DataFormat {
    /// Plain text documents
    Text,
    /// PDF documents
    Pdf,
    /// Image files (JPEG, PNG, WebP, etc.)
    Image,
    /// Video files
    Video,
    /// Audio files
    Audio,
    /// Structured data (JSON, CSV, XML)
    Structured,
    /// Binary data
    Binary,
    /// Archive files (ZIP, TAR, etc.)
    Archive,
    /// Code files
    Code,
    /// Other formats
    Other(String),
}

/// Processing context for data operations
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProcessingContext {
    /// Unique processing request ID
    #[schemars(with = "String")]
    pub request_id: uuid::Uuid,
    /// Source of the data
    pub source: String,
    /// Format of the input data
    pub format: DataFormat,
    /// Processing priority
    pub priority: ProcessingPriority,
    /// Additional metadata
    #[serde(skip_serializing_if = "std::collections::HashMap::is_empty", default)]
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// Processing priority levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProcessingPriority {
    Low,
    Normal,
    High,
    Urgent,
}

/// Processed data result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProcessedData {
    /// Unique ID for the processed data
    #[schemars(with = "String")]
    pub id: uuid::Uuid,
    /// Original source identifier
    pub source_id: String,
    /// Format of the processed data
    pub format: DataFormat,
    /// Processed content (text, metadata, etc.)
    pub content: ProcessingContent,
    /// Processing metadata
    #[serde(skip_serializing_if = "std::collections::HashMap::is_empty", default)]
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
    /// Processing timestamp
    #[schemars(with = "String")]
    pub processed_at: chrono::DateTime<chrono::Utc>,
    /// Processing duration in milliseconds
    pub processing_time_ms: u64,
}

/// Content types that can be extracted
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ProcessingContent {
    /// Plain text content
    Text(String),
    /// Structured data (JSON)
    Structured(serde_json::Value),
    /// Binary data (base64 encoded)
    Binary(String),
    /// Multi-modal content (text + metadata)
    MultiModal {
        text: String,
        #[serde(skip_serializing_if = "std::collections::HashMap::is_empty", default)]
        metadata: std::collections::HashMap<String, serde_json::Value>,
    },
}

/// Processing statistics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProcessingStats {
    /// Total requests processed
    pub total_processed: u64,
    /// Successful processing count
    pub successful: u64,
    /// Failed processing count
    pub failed: u64,
    /// Average processing time in milliseconds
    pub average_processing_time_ms: f64,
    /// Current queue size
    pub queue_size: u64,
    /// Processing success rate (0.0 to 1.0)
    pub success_rate: f64,
}

/// File operation result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FileOperationResult {
    /// Operation success
    pub success: bool,
    /// File path operated on
    pub path: String,
    /// Operation result data
    pub result: Option<serde_json::Value>,
    /// Error message if failed
    pub error: Option<String>,
}

/// File operation types
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum FileOperation {
    /// Read file content
    Read { path: String },
    /// Write file content
    Write { path: String, content: Vec<u8> },
    /// List directory contents
    List { path: String },
    /// Check if path exists
    Exists { path: String },
    /// Get file metadata
    Metadata { path: String },
    /// Delete file or directory
    Delete { path: String },
    /// Create directory
    CreateDir { path: String },
    /// Copy file or directory
    Copy { from: String, to: String },
    /// Move/rename file or directory
    Move { from: String, to: String },
}

/// Validation result for data processing - uses string issues with recommendations
pub type ValidationResult = super::validation::ValidationResult<String>;
