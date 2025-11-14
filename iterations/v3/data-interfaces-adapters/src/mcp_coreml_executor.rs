//! CoreML Ingestion Executor Implementation for MCP Tools
//!
//! Implements CoreMLIngestionExecutor trait using agent-data-processing enrichers.
//! This allows CoreML tools to be registered with the MCP server without exposing
//! them via the REST API (observer's API).
//!
//! @author @darianrosebrook

use agent_data_processing::data_processing_types::{
    ContentType, DataContent, DataInput, DataSource, FileSource, ProcessingContext,
    ProcessingPriority,
};
use agent_data_processing::enrichment::{
    AsrEnricher, EnrichmentCircuitBreakerConfig, VisionEnricher, VisualCaptioningEnricher,
};
use agent_data_processing::ingestion::{IngestionStage, UnifiedIngestor};
use agent_mcp::tools::CoreMLIngestionExecutor;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// Real CoreML ingestion executor implementation
pub struct RealCoreMLIngestionExecutor {
    asr_enricher: AsrEnricher,
    vision_enricher: VisionEnricher,
    visual_captioning_enricher: VisualCaptioningEnricher,
}

impl RealCoreMLIngestionExecutor {
    pub fn new(whisper_model_path: Option<PathBuf>, _yolo_model_path: Option<PathBuf>) -> Self {
        let circuit_breaker_config = EnrichmentCircuitBreakerConfig::default();
        
        let asr_enricher = AsrEnricher::new(circuit_breaker_config.clone());
        // Note: CoreML feature integration pending - whisper model path handling
        // will be implemented when coreml feature is added to Cargo.toml
        let _ = whisper_model_path; // Suppress unused variable warning
        
        Self {
            asr_enricher,
            vision_enricher: VisionEnricher::new(circuit_breaker_config.clone()),
            visual_captioning_enricher: VisualCaptioningEnricher::new(circuit_breaker_config),
        }
    }
}

#[async_trait::async_trait]
impl CoreMLIngestionExecutor for RealCoreMLIngestionExecutor {
    async fn transcribe_audio(
        &self,
        file_path: &str,
        content_type: Option<&str>,
    ) -> Result<serde_json::Value, String> {
        let audio_data = tokio::fs::read(file_path)
            .await
            .map_err(|e| format!("Failed to read audio file: {}", e))?;

        let content_type = content_type.unwrap_or("audio/wav");

        let result = self
            .asr_enricher
            .enrich_audio(&audio_data, content_type)
            .await
            .map_err(|e| format!("ASR enrichment failed: {}", e))?;

        Ok(serde_json::json!({
            "transcription": result.processed_content.audio_transcript.unwrap_or_default(),
            "confidence": result.extracted_metadata.get("confidence")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            "language": result.extracted_metadata.get("language")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            "duration": result.extracted_metadata.get("duration")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
        }))
    }

    async fn detect_objects(
        &self,
        file_path: &str,
        content_type: Option<&str>,
    ) -> Result<serde_json::Value, String> {
        let image_data = tokio::fs::read(file_path)
            .await
            .map_err(|e| format!("Failed to read image file: {}", e))?;

        let content_type = content_type.unwrap_or("image/jpeg");

        let result = self
            .visual_captioning_enricher
            .enrich_visual(&image_data, content_type)
            .await
            .map_err(|e| format!("Object detection failed: {}", e))?;

        let objects = result
            .extracted_metadata
            .get("objects")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        Ok(serde_json::json!({
            "objects": objects,
            "caption": result.extracted_metadata.get("caption")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            "confidence": result.extracted_metadata.get("confidence")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
        }))
    }

    async fn extract_text_from_image(
        &self,
        file_path: &str,
        content_type: Option<&str>,
    ) -> Result<serde_json::Value, String> {
        let image_data = tokio::fs::read(file_path)
            .await
            .map_err(|e| format!("Failed to read image file: {}", e))?;

        let content_type = content_type.unwrap_or("image/jpeg");

        let result = self
            .vision_enricher
            .enrich_image(&image_data, content_type)
            .await
            .map_err(|e| format!("OCR extraction failed: {}", e))?;

        Ok(serde_json::json!({
            "text": result.processed_content.text_content.unwrap_or_default(),
            "bounding_boxes": result.extracted_metadata.get("bounding_boxes")
                .cloned()
                .unwrap_or_default(),
            "confidence": result.extracted_metadata.get("confidence")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
        }))
    }

    async fn process_video(&self, file_path: &str) -> Result<serde_json::Value, String> {
        let ingestor = UnifiedIngestor::new();

        let file_metadata = tokio::fs::metadata(file_path)
            .await
            .map_err(|e| format!("Failed to read file metadata: {}", e))?;

        let data_input = DataInput {
            id: agent_data_processing::data_processing_types::ProcessingId::new(),
            source: DataSource::File(FileSource {
                path: PathBuf::from(file_path),
                content_type: ContentType::Video,
                size_bytes: file_metadata.len(),
                last_modified: file_metadata
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| {
                        chrono::DateTime::from_timestamp(d.as_secs() as i64, 0)
                            .unwrap_or_else(|| chrono::Utc::now())
                    })
                    .unwrap_or_else(|| chrono::Utc::now()),
            }),
            content: DataContent::Binary(
                tokio::fs::read(file_path)
                    .await
                    .map_err(|e| format!("Failed to read video file: {}", e))?,
            ),
            metadata: HashMap::new(),
            processing_context: ProcessingContext {
                request_id: uuid::Uuid::new_v4().to_string(),
                user_id: None,
                project_scope: None,
                priority: ProcessingPriority::Normal,
                deadline: None,
                tags: vec![],
            },
        };

        let result = ingestor
            .ingest(data_input)
            .await
            .map_err(|e| format!("Video processing failed: {}", e))?;

        Ok(serde_json::json!({
            "metadata": result.extracted_metadata,
            "audio_transcript": result.processed_content.audio_transcript,
            "visual_elements": result.processed_content.visual_elements,
            "duration": result.extracted_metadata.get("duration_seconds")
                .and_then(|v| v.as_f64()),
            "resolution": result.extracted_metadata.get("resolution")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        }))
    }
}

/// Helper function to create and configure CoreML executor
pub fn create_coreml_executor(
    whisper_model_path: Option<PathBuf>,
    yolo_model_path: Option<PathBuf>,
) -> Arc<dyn CoreMLIngestionExecutor> {
    Arc::new(RealCoreMLIngestionExecutor::new(
        whisper_model_path,
        yolo_model_path,
    ))
}

/// Helper function to wire up CoreML executor with MCP server
///
/// This should be called when initializing the MCP server to enable
/// CoreML-powered ingestion tools. The tools will be available via MCP
/// protocol but NOT exposed via the REST API (observer's API).
///
/// Example usage:
/// ```rust,ignore
/// use data_interfaces_adapters::mcp_coreml_executor;
/// use std::env;
/// use std::path::PathBuf;
///
/// // Get model paths from environment
/// let whisper_path = env::var("COREML_WHISPER_MODEL_PATH")
///     .ok()
///     .map(PathBuf::from);
/// let yolo_path = env::var("COREML_YOLO_MODEL_PATH")
///     .ok()
///     .map(PathBuf::from);
///
/// // Create executor
/// let executor = mcp_coreml_executor::create_coreml_executor(whisper_path, yolo_path);
///
/// // Wire up with MCP server
/// mcp_server.set_coreml_executor(executor);
/// ```
pub fn wire_coreml_executor_to_mcp_server(
    mcp_server: &agent_mcp::MCPServer,
    whisper_model_path: Option<PathBuf>,
    yolo_model_path: Option<PathBuf>,
) {
    let executor = create_coreml_executor(whisper_model_path, yolo_model_path);
    mcp_server.set_coreml_executor(executor);
}
