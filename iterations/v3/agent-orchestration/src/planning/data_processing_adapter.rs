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
    ports::data_processing::{
        DataFormat, FileOperation, FileOperationResult, ProcessedData, ProcessingContent,
        ProcessingContext, ProcessingPriority, ProcessingStats, ValidationResult,
    },
    DataProcessingService,
};

/// Adapter that wraps agent-data-processing DataPipeline to implement contracts::DataProcessingService
#[cfg(feature = "data-processing")]
pub struct DataProcessingServiceAdapter {
    /// The underlying data processing pipeline implementation
    pipeline: Arc<agent_data_processing::DataPipeline>,
}

#[cfg(feature = "data-processing")]
impl DataProcessingServiceAdapter {
    /// Create a new data processing service adapter
    pub fn new(pipeline: Arc<agent_data_processing::DataPipeline>) -> Self {
        Self { pipeline }
    }

    /// Convert contracts ProcessingContext to agent-data-processing DataInput
    fn context_to_data_input(&self, context: &ProcessingContext) -> agent_data_processing::DataInput {
        use agent_data_processing::{ContentType, DataContent, DataSource, FileSource, ProcessingContext as InternalContext, ProcessingId, ProcessingPriority as InternalPriority};
        use chrono::Utc;
        use std::collections::HashMap;
        use std::path::PathBuf;

        // Convert DataFormat to ContentType
        let content_type = match context.format {
            DataFormat::Text => ContentType::Text,
            DataFormat::Pdf => ContentType::Pdf,
            DataFormat::Image => ContentType::Image,
            DataFormat::Video => ContentType::Video,
            DataFormat::Audio => ContentType::Audio,
            DataFormat::Structured => ContentType::Structured,
            DataFormat::Binary => ContentType::Binary,
            DataFormat::Archive => ContentType::Binary, // Archive not in ContentType, use Binary
            DataFormat::Code => ContentType::Code,
            DataFormat::Other(_) => ContentType::Unknown,
        };

        // Convert ProcessingPriority to internal ProcessingPriority
        let priority = match context.priority {
            ProcessingPriority::Low => InternalPriority::Low,
            ProcessingPriority::Normal => InternalPriority::Normal,
            ProcessingPriority::High => InternalPriority::High,
            ProcessingPriority::Urgent => InternalPriority::Critical, // Urgent maps to Critical
        };

        // Create file source from context source (assuming it's a file path)
        let file_source = FileSource {
            path: PathBuf::from(&context.source),
            content_type: content_type.clone(),
            size_bytes: 0, // Will be determined when file is read
            last_modified: Utc::now(),
        };

        use uuid::Uuid;
        
        agent_data_processing::DataInput {
            id: ProcessingId(context.request_id),
            source: DataSource::File(file_source),
            content: DataContent::Text(String::new()), // Content will be loaded from source
            metadata: context.metadata.clone(),
            processing_context: InternalContext {
                request_id: context.request_id.to_string(),
                user_id: None,
                project_scope: None,
                priority,
                deadline: None,
                tags: vec![],
            },
        }
    }

    /// Convert agent-data-processing ProcessingOutput to contracts ProcessedData
    fn output_to_processed_data(&self, output: &agent_data_processing::ProcessingOutput, context: &ProcessingContext) -> ProcessedData {
        use chrono::Utc;

        // Extract text content from output
        let text_content = output.processed_content.text_content.clone().unwrap_or_default();

        // Convert to ProcessingContent
        let content = if let Some(structured) = &output.processed_content.structured_data {
            ProcessingContent::Structured(structured.clone())
        } else if !text_content.is_empty() {
            ProcessingContent::Text(text_content)
        } else {
            ProcessingContent::Text(String::new())
        };

        ProcessedData {
            id: output.id.0,
            source_id: context.source.clone(),
            format: context.format.clone(),
            content,
            metadata: output.extracted_metadata.clone(),
            processed_at: output.created_at,
            processing_time_ms: output.processing_stats.processing_time_ms,
        }
    }

    /// Convert contracts DataFormat to agent-data-processing ContentType
    fn format_to_content_type(&self, format: DataFormat) -> agent_data_processing::ContentType {
        use agent_data_processing::ContentType;
        match format {
            DataFormat::Text => ContentType::Text,
            DataFormat::Pdf => ContentType::Pdf,
            DataFormat::Image => ContentType::Image,
            DataFormat::Video => ContentType::Video,
            DataFormat::Audio => ContentType::Audio,
            DataFormat::Structured => ContentType::Structured,
            DataFormat::Binary => ContentType::Binary,
            DataFormat::Archive => ContentType::Binary,
            DataFormat::Code => ContentType::Code,
            DataFormat::Other(_) => ContentType::Unknown,
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
        let data_input = self.context_to_data_input(&context);
        let output = self
            .pipeline
            .process(data_input)
            .await
            .map_err(
                |e| agent_agency_contracts::errors::DataProcessingError::ProcessingFailed {
                    operation: "process_data".to_string(),
                    reason: e.to_string(),
                },
            )?;

        Ok(self.output_to_processed_data(&output, &context))
    }

    async fn batch_process(
        &self,
        contexts: Vec<ProcessingContext>,
    ) -> DataProcessingResult<Vec<Result<ProcessedData, String>>> {
        // Process each context individually through the pipeline
        let mut results = Vec::new();
        for context in contexts {
            match self.process_data(context.clone()).await {
                Ok(processed_data) => results.push(Ok(processed_data)),
                Err(e) => results.push(Err(e.to_string())),
            }
        }
        Ok(results)
    }

    async fn validate_data(
        &self,
        context: &ProcessingContext,
    ) -> DataProcessingResult<ValidationResult> {
        // Validate by attempting to process - if it succeeds, data is valid
        let data_input = self.context_to_data_input(context);
        
        // Try to process to validate
        match self.pipeline.process(data_input).await {
            Ok(output) => {
                // Data is valid if processing succeeded
                let issues = if output.processing_stats.errors_encountered.is_empty() {
                    vec![]
                } else {
                    output.processing_stats.errors_encountered
                };
                
                Ok(ValidationResult {
                    is_valid: issues.is_empty(),
                    score: if issues.is_empty() { 1.0 } else { 0.5 },
                    issues,
                    warnings: vec![],
                    recommendations: vec![],
                })
            }
            Err(e) => {
                Ok(ValidationResult {
                    is_valid: false,
                    score: 0.0,
                    issues: vec![e.to_string()],
                    warnings: vec![],
                    recommendations: vec!["Check data format and source availability".to_string()],
                })
            }
        }
    }

    async fn supported_formats(&self) -> Vec<DataFormat> {
        // Return all supported formats based on ContentType mapping
        vec![
            DataFormat::Text,
            DataFormat::Pdf,
            DataFormat::Image,
            DataFormat::Video,
            DataFormat::Audio,
            DataFormat::Structured,
            DataFormat::Binary,
            DataFormat::Archive,
            DataFormat::Code,
        ]
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
            .pipeline
            .get_stats()
            .await
            .map_err(|e| {
                agent_agency_contracts::errors::DataProcessingError::ServiceUnavailable {
                    service: "data-processing".to_string(),
                }
            })?;

        // Convert PipelineStats to ProcessingStats
        let success_rate = if stats.total_processed > 0 {
            1.0 - stats.error_rate
        } else {
            1.0
        };

        Ok(ProcessingStats {
            total_processed: stats.total_processed,
            successful: stats.total_processed.saturating_sub((stats.total_processed as f64 * stats.error_rate) as u64),
            failed: (stats.total_processed as f64 * stats.error_rate) as u64,
            average_processing_time_ms: stats.avg_processing_time_ms,
            queue_size: stats.queue_depth,
            success_rate,
        })
    }

    async fn extract_text(&self, data: &[u8], format: DataFormat) -> DataProcessingResult<String> {
        use agent_data_processing::{ContentType, DataContent, DataInput, DataSource, FileSource, ProcessingContext as InternalContext, ProcessingId, ProcessingPriority as InternalPriority};
        use chrono::Utc;
        use std::path::PathBuf;
        use uuid::Uuid;

        // Create a temporary DataInput from the raw data
        let content_type = self.format_to_content_type(format);
        
        let data_input = DataInput {
            id: ProcessingId(Uuid::new_v4()),
            source: DataSource::File(FileSource {
                path: PathBuf::from("<in-memory>"),
                content_type,
                size_bytes: data.len() as u64,
                last_modified: Utc::now(),
            }),
            content: DataContent::Binary(data.to_vec()),
            metadata: std::collections::HashMap::new(),
            processing_context: InternalContext {
                request_id: Uuid::new_v4().to_string(),
                user_id: None,
                project_scope: None,
                priority: InternalPriority::Normal,
                deadline: None,
                tags: vec![],
            },
        };

        // Process through pipeline to extract text
        match self.pipeline.process(data_input).await {
            Ok(output) => {
                // Extract text from output
                let text = output.processed_content.text_content
                    .unwrap_or_else(|| {
                        match &output.processed_content.data {
                            agent_data_processing::ProcessedContentData::Text(t) => t.clone(),
                            agent_data_processing::ProcessedContentData::Binary(b) => {
                                String::from_utf8_lossy(b).to_string()
                            }
                            agent_data_processing::ProcessedContentData::Structured(s) => {
                                serde_json::to_string(s).unwrap_or_default()
                            }
                        }
                    });
                Ok(text)
            }
            Err(e) => Err(agent_agency_contracts::errors::DataProcessingError::ProcessingFailed {
                operation: "extract_text".to_string(),
                reason: e.to_string(),
            }),
        }
    }

    async fn generate_embedding(&self, text: &str) -> DataProcessingResult<Vec<f32>> {
        // PLACEHOLDER: Embedding generation not directly available in DataPipeline
        // This requires integration with agent-memory or a separate embedding service
        // For now, return an error indicating this feature requires additional dependencies
        Err(agent_agency_contracts::errors::DataProcessingError::ProcessingFailed {
            operation: "generate_embedding".to_string(),
            reason: "Embedding generation requires agent-memory integration. Use DataProcessingSystem with memory-integration feature instead.".to_string(),
        })
    }
}
