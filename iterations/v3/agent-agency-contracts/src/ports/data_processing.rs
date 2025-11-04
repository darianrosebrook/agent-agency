//! Data Processing Service Port
//!
//! Defines the interface for data processing operations including ingestion,
//! enrichment, indexing, and file operations. This port enables dependency injection
//! and testing for data processing services.
//!
//! @author @darianrosebrook

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::errors::DataProcessingResult;

/// Supported data formats for processing
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, JsonSchema)]
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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, JsonSchema)]
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
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// Processing priority levels
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProcessingPriority {
    Low,
    Normal,
    High,
    Urgent,
}

/// Processed data result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, JsonSchema)]
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
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
    /// Processing timestamp
    #[schemars(with = "String")]
    pub processed_at: chrono::DateTime<chrono::Utc>,
    /// Processing duration in milliseconds
    pub processing_time_ms: u64,
}

/// Content types that can be extracted
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, JsonSchema)]
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
        metadata: std::collections::HashMap<String, serde_json::Value>,
    },
}

/// Processing statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, JsonSchema)]
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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, JsonSchema)]
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

/// Core data processing service interface
/// Implementations provide data ingestion, processing, and management capabilities
#[async_trait::async_trait]
pub trait DataProcessingService: Send + Sync {
    /// Process data from a given source
    ///
    /// # Arguments
    /// * `context` - Processing context with source and format information
    ///
    /// # Returns
    /// Processed data result or error if processing fails
    async fn process_data(&self, context: ProcessingContext) -> DataProcessingResult<ProcessedData>;

    /// Batch process multiple data sources
    ///
    /// # Arguments
    /// * `contexts` - Vector of processing contexts
    ///
    /// # Returns
    /// Vector of processing results (may contain errors for individual items)
    async fn batch_process(&self, contexts: Vec<ProcessingContext>) -> DataProcessingResult<Vec<Result<ProcessedData, String>>>;

    /// Validate data format and content
    ///
    /// # Arguments
    /// * `context` - Processing context to validate
    ///
    /// # Returns
    /// Validation result indicating if data can be processed
    async fn validate_data(&self, context: &ProcessingContext) -> DataProcessingResult<ValidationResult>;

    /// Get supported data formats
    ///
    /// # Returns
    /// Vector of supported data formats
    async fn supported_formats(&self) -> Vec<DataFormat>;

    /// Perform file system operations (read, write, list, etc.)
    ///
    /// # Arguments
    /// * `operation` - The file operation to perform
    ///
    /// # Returns
    /// File operation result
    async fn file_operation(&self, operation: FileOperation) -> DataProcessingResult<FileOperationResult>;

    /// Get processing statistics
    ///
    /// # Returns
    /// Current processing statistics
    async fn get_processing_stats(&self) -> DataProcessingResult<ProcessingStats>;

    /// Extract text content from various formats
    ///
    /// # Arguments
    /// * `data` - Raw data to extract text from
    /// * `format` - Format of the input data
    ///
    /// # Returns
    /// Extracted text content
    async fn extract_text(&self, data: &[u8], format: DataFormat) -> DataProcessingResult<String>;

    /// Generate embeddings for text content
    ///
    /// # Arguments
    /// * `text` - Text content to embed
    ///
    /// # Returns
    /// Vector embedding for the text
    async fn generate_embedding(&self, text: &str) -> DataProcessingResult<Vec<f32>>;
}

/// File operation types
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, JsonSchema)]
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

/// Validation result for data processing
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ValidationResult {
    /// Whether the data is valid for processing
    pub is_valid: bool,
    /// Validation score (0.0 to 1.0)
    pub score: f64,
    /// Validation issues found
    pub issues: Vec<String>,
    /// Validation warnings
    pub warnings: Vec<String>,
    /// Recommended actions
    pub recommendations: Vec<String>,
}
