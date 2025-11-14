//! Data Processing Service Adapter
//!
//! Adapts the real agent-data-processing service to implement the contracts::DataProcessingService trait.
//! This adapter enables dependency injection and breaks the direct dependency from orchestration to data processing.
//!
//! @author @darianrosebrook

#[cfg(feature = "data-processing")]
use async_trait::async_trait;
#[cfg(feature = "data-processing")]
use std::sync::Arc;

#[cfg(feature = "data-processing")]
use agent_agency_contracts::{
    errors::DataProcessingResult,
    types::data_processing::{
        DataFormat, FileOperation, FileOperationResult, ProcessedData, ProcessingContent,
        ProcessingContext, ProcessingPriority, ProcessingStats, ValidationResult,
    },
    DataProcessingService,
};

/// Adapter that wraps agent-data-processing service to implement contracts::DataProcessingService
#[cfg(feature = "data-processing")]
pub struct DataProcessingServiceAdapter {
    /// The underlying data processing service implementation
    data_processor: Arc<dyn agent_data_processing::DataProcessor>,
}

#[cfg(feature = "data-processing")]
impl DataProcessingServiceAdapter {
    /// Create a new data processing service adapter
    pub fn new(data_processor: Arc<dyn agent_data_processing::DataProcessor>) -> Self {
        Self { data_processor }
    }

    /// Convert contracts ProcessingContext to agent-data-processing types
    fn to_internal_context(
        &self,
        context: &ProcessingContext,
    ) -> agent_data_processing::ProcessingContext {
        agent_data_processing::ProcessingContext {
            request_id: context.request_id,
            source: context.source.clone(),
            format: self.to_internal_format(context.format.clone()),
            priority: self.to_internal_priority(context.priority.clone()),
            metadata: context.metadata.clone(),
        }
    }

    /// Convert contracts DataFormat to agent-data-processing types
    fn to_internal_format(&self, format: DataFormat) -> agent_data_processing::DataFormat {
        match format {
            DataFormat::Text => agent_data_processing::DataFormat::Text,
            DataFormat::Pdf => agent_data_processing::DataFormat::Pdf,
            DataFormat::Image => agent_data_processing::DataFormat::Image,
            DataFormat::Video => agent_data_processing::DataFormat::Video,
            DataFormat::Audio => agent_data_processing::DataFormat::Audio,
            DataFormat::Structured => agent_data_processing::DataFormat::Structured,
            DataFormat::Binary => agent_data_processing::DataFormat::Binary,
            DataFormat::Archive => agent_data_processing::DataFormat::Archive,
            DataFormat::Code => agent_data_processing::DataFormat::Code,
            DataFormat::Other(s) => agent_data_processing::DataFormat::Other(s),
        }
    }

    /// Convert contracts ProcessingPriority to agent-data-processing types
    fn to_internal_priority(
        &self,
        priority: ProcessingPriority,
    ) -> agent_data_processing::ProcessingPriority {
        match priority {
            ProcessingPriority::Low => agent_data_processing::ProcessingPriority::Low,
            ProcessingPriority::Normal => agent_data_processing::ProcessingPriority::Normal,
            ProcessingPriority::High => agent_data_processing::ProcessingPriority::High,
            ProcessingPriority::Urgent => agent_data_processing::ProcessingPriority::Urgent,
        }
    }

    /// Convert agent-data-processing ProcessedData to contracts types
    fn from_internal_data(&self, data: agent_data_processing::ProcessedData) -> ProcessedData {
        ProcessedData {
            id: data.id,
            source_id: data.source_id,
            format: self.from_internal_format(data.format),
            content: self.from_internal_content(data.content),
            metadata: data.metadata,
            processed_at: data.processed_at,
            processing_time_ms: data.processing_time_ms,
        }
    }

    /// Convert agent-data-processing DataFormat to contracts types
    fn from_internal_format(&self, format: agent_data_processing::DataFormat) -> DataFormat {
        match format {
            agent_data_processing::DataFormat::Text => DataFormat::Text,
            agent_data_processing::DataFormat::Pdf => DataFormat::Pdf,
            agent_data_processing::DataFormat::Image => DataFormat::Image,
            agent_data_processing::DataFormat::Video => DataFormat::Video,
            agent_data_processing::DataFormat::Audio => DataFormat::Audio,
            agent_data_processing::DataFormat::Structured => DataFormat::Structured,
            agent_data_processing::DataFormat::Binary => DataFormat::Binary,
            agent_data_processing::DataFormat::Archive => DataFormat::Archive,
            agent_data_processing::DataFormat::Code => DataFormat::Code,
            agent_data_processing::DataFormat::Other(s) => DataFormat::Other(s),
        }
    }

    /// Convert agent-data-processing ProcessingContent to contracts types
    fn from_internal_content(
        &self,
        content: agent_data_processing::ProcessingContent,
    ) -> ProcessingContent {
        match content {
            agent_data_processing::ProcessingContent::Text(s) => ProcessingContent::Text(s),
            agent_data_processing::ProcessingContent::Structured(v) => {
                ProcessingContent::Structured(v)
            }
            agent_data_processing::ProcessingContent::Binary(s) => ProcessingContent::Binary(s),
            agent_data_processing::ProcessingContent::MultiModal { text, metadata } => {
                ProcessingContent::MultiModal { text, metadata }
            }
        }
    }
}

#[cfg(feature = "data-processing")]
#[async_trait]
impl DataProcessingService for DataProcessingServiceAdapter {
    async fn process_data(
        &self,
        context: ProcessingContext,
    ) -> DataProcessingResult<ProcessedData> {
        let internal_context = self.to_internal_context(&context);
        let result = self
            .data_processor
            .process_data(internal_context)
            .await
            .map_err(
                |e| agent_agency_contracts::errors::DataProcessingError::ProcessingFailed {
                    operation: "process_data".to_string(),
                    reason: e.to_string(),
                },
            )?;

        Ok(self.from_internal_data(result))
    }

    async fn batch_process(
        &self,
        contexts: Vec<ProcessingContext>,
    ) -> DataProcessingResult<Vec<Result<ProcessedData, String>>> {
        let internal_contexts: Vec<_> = contexts
            .iter()
            .map(|c| self.to_internal_context(c))
            .collect();

        let results = self
            .data_processor
            .batch_process(internal_contexts)
            .await
            .map_err(
                |e| agent_agency_contracts::errors::DataProcessingError::ProcessingFailed {
                    operation: "batch_process".to_string(),
                    reason: e.to_string(),
                },
            )?;

        Ok(results
            .into_iter()
            .map(|r| {
                r.map(|d| self.from_internal_data(d))
                    .map_err(|e| e.to_string())
            })
            .collect())
    }

    async fn validate_data(
        &self,
        context: &ProcessingContext,
    ) -> DataProcessingResult<ValidationResult> {
        let internal_context = self.to_internal_context(context);
        let result = self
            .data_processor
            .validate_data(&internal_context)
            .await
            .map_err(
                |e| agent_agency_contracts::errors::DataProcessingError::ValidationFailed {
                    reason: e.to_string(),
                },
            )?;

        Ok(ValidationResult {
            is_valid: result.is_valid,
            score: result.score,
            issues: result.issues,
            warnings: result.warnings,
            recommendations: result.recommendations,
        })
    }

    async fn supported_formats(&self) -> Vec<DataFormat> {
        self.data_processor
            .supported_formats()
            .await
            .into_iter()
            .map(|f| self.from_internal_format(f))
            .collect()
    }

    async fn file_operation(
        &self,
        operation: FileOperation,
    ) -> DataProcessingResult<FileOperationResult> {
        use std::path::Path;
        use tokio::fs;
        use tracing::{debug, warn};

        match operation {
            FileOperation::Read { path } => {
                debug!("Reading file: {}", path);
                match fs::read(&path).await {
                    Ok(content) => Ok(FileOperationResult {
                        success: true,
                        path: path.clone(),
                        result: Some(serde_json::json!({
                            "content": String::from_utf8_lossy(&content),
                            "size_bytes": content.len(),
                        })),
                        error: None,
                    }),
                    Err(e) => {
                        warn!("Failed to read file {}: {}", path, e);
                        Ok(FileOperationResult {
                            success: false,
                            path,
                            result: None,
                            error: Some(format!("Failed to read file: {}", e)),
                        })
                    }
                }
            }
            FileOperation::Write { path, content } => {
                debug!("Writing file: {} ({} bytes)", path, content.len());
                match fs::write(&path, &content).await {
                    Ok(_) => Ok(FileOperationResult {
                        success: true,
                        path: path.clone(),
                        result: Some(serde_json::json!({
                            "bytes_written": content.len(),
                        })),
                        error: None,
                    }),
                    Err(e) => {
                        warn!("Failed to write file {}: {}", path, e);
                        Ok(FileOperationResult {
                            success: false,
                            path,
                            result: None,
                            error: Some(format!("Failed to write file: {}", e)),
                        })
                    }
                }
            }
            FileOperation::List { path } => {
                debug!("Listing directory: {}", path);
                match fs::read_dir(&path).await {
                    Ok(mut entries) => {
                        let mut file_list = Vec::new();
                        while let Ok(Some(entry)) = entries.next_entry().await {
                            if let Ok(metadata) = entry.metadata().await {
                                file_list.push(serde_json::json!({
                                    "name": entry.file_name().to_string_lossy(),
                                    "path": entry.path().to_string_lossy(),
                                    "is_file": metadata.is_file(),
                                    "is_dir": metadata.is_dir(),
                                    "size": if metadata.is_file() { Some(metadata.len()) } else { None },
                                }));
                            }
                        }
                        Ok(FileOperationResult {
                            success: true,
                            path: path.clone(),
                            result: Some(serde_json::json!({
                                "entries": file_list,
                                "count": file_list.len(),
                            })),
                            error: None,
                        })
                    }
                    Err(e) => {
                        warn!("Failed to list directory {}: {}", path, e);
                        Ok(FileOperationResult {
                            success: false,
                            path,
                            result: None,
                            error: Some(format!("Failed to list directory: {}", e)),
                        })
                    }
                }
            }
            FileOperation::Exists { path } => {
                debug!("Checking if path exists: {}", path);
                let exists = Path::new(&path).exists();
                Ok(FileOperationResult {
                    success: true,
                    path: path.clone(),
                    result: Some(serde_json::json!({
                        "exists": exists,
                    })),
                    error: None,
                })
            }
            FileOperation::Metadata { path } => {
                debug!("Getting metadata for: {}", path);
                match fs::metadata(&path).await {
                    Ok(metadata) => Ok(FileOperationResult {
                        success: true,
                        path: path.clone(),
                        result: Some(serde_json::json!({
                            "is_file": metadata.is_file(),
                            "is_dir": metadata.is_dir(),
                            "size": metadata.len(),
                            "modified": metadata.modified()
                                .ok()
                                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                                .map(|d| d.as_secs()),
                        })),
                        error: None,
                    }),
                    Err(e) => {
                        warn!("Failed to get metadata for {}: {}", path, e);
                        Ok(FileOperationResult {
                            success: false,
                            path,
                            result: None,
                            error: Some(format!("Failed to get metadata: {}", e)),
                        })
                    }
                }
            }
            FileOperation::Delete { path } => {
                debug!("Deleting: {}", path);
                let path_obj = Path::new(&path);
                let result = if path_obj.is_dir() {
                    fs::remove_dir_all(&path).await
                } else {
                    fs::remove_file(&path).await
                };

                match result {
                    Ok(_) => Ok(FileOperationResult {
                        success: true,
                        path: path.clone(),
                        result: Some(serde_json::json!({
                            "deleted": true,
                        })),
                        error: None,
                    }),
                    Err(e) => {
                        warn!("Failed to delete {}: {}", path, e);
                        Ok(FileOperationResult {
                            success: false,
                            path,
                            result: None,
                            error: Some(format!("Failed to delete: {}", e)),
                        })
                    }
                }
            }
            FileOperation::CreateDir { path } => {
                debug!("Creating directory: {}", path);
                match fs::create_dir_all(&path).await {
                    Ok(_) => Ok(FileOperationResult {
                        success: true,
                        path: path.clone(),
                        result: Some(serde_json::json!({
                            "created": true,
                        })),
                        error: None,
                    }),
                    Err(e) => {
                        warn!("Failed to create directory {}: {}", path, e);
                        Ok(FileOperationResult {
                            success: false,
                            path,
                            result: None,
                            error: Some(format!("Failed to create directory: {}", e)),
                        })
                    }
                }
            }
            FileOperation::Copy { from, to } => {
                debug!("Copying {} to {}", from, to);
                match fs::copy(&from, &to).await {
                    Ok(bytes_copied) => Ok(FileOperationResult {
                        success: true,
                        path: to.clone(),
                        result: Some(serde_json::json!({
                            "from": from,
                            "bytes_copied": bytes_copied,
                        })),
                        error: None,
                    }),
                    Err(e) => {
                        warn!("Failed to copy {} to {}: {}", from, to, e);
                        Ok(FileOperationResult {
                            success: false,
                            path: to,
                            result: None,
                            error: Some(format!("Failed to copy file: {}", e)),
                        })
                    }
                }
            }
            FileOperation::Move { from, to } => {
                debug!("Moving {} to {}", from, to);
                match fs::rename(&from, &to).await {
                    Ok(_) => Ok(FileOperationResult {
                        success: true,
                        path: to.clone(),
                        result: Some(serde_json::json!({
                            "from": from,
                            "moved": true,
                        })),
                        error: None,
                    }),
                    Err(e) => {
                        warn!("Failed to move {} to {}: {}", from, to, e);
                        Ok(FileOperationResult {
                            success: false,
                            path: to,
                            result: None,
                            error: Some(format!("Failed to move file: {}", e)),
                        })
                    }
                }
            }
        }
    }

    async fn get_processing_stats(&self) -> DataProcessingResult<ProcessingStats> {
        let stats = self
            .data_processor
            .get_processing_stats()
            .await
            .map_err(|e| {
                agent_agency_contracts::errors::DataProcessingError::ServiceUnavailable {
                    service: "data-processing".to_string(),
                }
            })?;

        Ok(ProcessingStats {
            total_processed: stats.total_processed,
            successful: stats.successful,
            failed: stats.failed,
            average_processing_time_ms: stats.average_processing_time_ms,
            queue_size: stats.queue_size,
            success_rate: stats.success_rate,
        })
    }

    async fn extract_text(&self, data: &[u8], format: DataFormat) -> DataProcessingResult<String> {
        let internal_format = self.to_internal_format(format);
        self.data_processor
            .extract_text(data, internal_format)
            .await
            .map_err(
                |e| agent_agency_contracts::errors::DataProcessingError::ProcessingFailed {
                    operation: "extract_text".to_string(),
                    reason: e.to_string(),
                },
            )
    }

    async fn generate_embedding(&self, text: &str) -> DataProcessingResult<Vec<f32>> {
        self.data_processor
            .generate_embedding(text)
            .await
            .map_err(
                |e| agent_agency_contracts::errors::DataProcessingError::ProcessingFailed {
                    operation: "generate_embedding".to_string(),
                    reason: e.to_string(),
                },
            )
    }
}
