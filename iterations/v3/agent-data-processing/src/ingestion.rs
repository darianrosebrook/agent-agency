//! Data ingestion stage - unified interface for ingesting data from various sources
//!
//! Consolidates functionality from the original ingestors crate:
//! - File ingestion (PDF, video, images, text)
//! - URL fetching and processing
//! - Stream processing
//! - Database record ingestion
//! - API data ingestion

use crate::types::*;
use crate::{DataProcessingResult, DataProcessingError};
use async_trait::async_trait;
use std::path::Path;

/// Result from ingestion operations
pub type IngestionResult = DataProcessingResult<ProcessingOutput>;

/// Stage for data ingestion operations
#[async_trait]
pub trait IngestionStage: Send + Sync {
    /// Get the name of this ingestion stage
    fn name(&self) -> &'static str;

    /// Check if this stage can handle the given data source
    fn can_ingest(&self, source: &DataSource) -> bool;

    /// Ingest data from the given input
    async fn ingest(&self, input: DataInput) -> IngestionResult;

    /// Get supported content types
    fn supported_content_types(&self) -> &[ContentType];
}

/// Default implementation combining all ingestion capabilities
pub struct DefaultIngestionStage {
    file_ingestor: FileIngestor,
    url_ingestor: UrlIngestor,
    stream_ingestor: StreamIngestor,
    database_ingestor: DatabaseIngestor,
    api_ingestor: ApiIngestor,
}

impl DefaultIngestionStage {
    /// Create a new default ingestion stage
    pub async fn new() -> DataProcessingResult<Self> {
        Ok(Self {
            file_ingestor: FileIngestor::new(),
            url_ingestor: UrlIngestor::new(),
            stream_ingestor: StreamIngestor::new(),
            database_ingestor: DatabaseIngestor::new().await?,
            api_ingestor: ApiIngestor::new(),
        })
    }
}

#[async_trait]
impl IngestionStage for DefaultIngestionStage {
    fn name(&self) -> &'static str {
        "default_ingestion"
    }

    fn can_ingest(&self, source: &DataSource) -> bool {
        match source {
            DataSource::File(_) => self.file_ingestor.can_ingest(source),
            DataSource::Url(_) => self.url_ingestor.can_ingest(source),
            DataSource::Stream(_) => self.stream_ingestor.can_ingest(source),
            DataSource::Database(_) => self.database_ingestor.can_ingest(source),
            DataSource::Api(_) => self.api_ingestor.can_ingest(source),
        }
    }

    async fn ingest(&self, input: DataInput) -> IngestionResult {
        match &input.source {
            DataSource::File(_) => self.file_ingestor.ingest(input).await,
            DataSource::Url(_) => self.url_ingestor.ingest(input).await,
            DataSource::Stream(_) => self.stream_ingestor.ingest(input).await,
            DataSource::Database(_) => self.database_ingestor.ingest(input).await,
            DataSource::Api(_) => self.api_ingestor.ingest(input).await,
        }
    }

    fn supported_content_types(&self) -> &[ContentType] {
        &[
            ContentType::Text,
            ContentType::Pdf,
            ContentType::Image,
            ContentType::Video,
            ContentType::Audio,
            ContentType::Html,
            ContentType::Json,
            ContentType::Xml,
            ContentType::Binary,
            ContentType::Markdown,
            ContentType::Code,
        ]
    }
}

#[async_trait]
impl crate::pipeline::PipelineStage for DefaultIngestionStage {
    fn name(&self) -> &'static str {
        "ingestion"
    }

    async fn process(&self, input: DataInput) -> DataProcessingResult<ProcessingOutput> {
        self.ingest(input).await
    }
}

/// File-based data ingestion
pub struct FileIngestor;

impl FileIngestor {
    pub fn new() -> Self {
        Self
    }

    pub fn can_ingest(&self, source: &DataSource) -> bool {
        matches!(source, DataSource::File(_))
    }

    pub async fn ingest(&self, input: DataInput) -> IngestionResult {
        let file_source = match &input.source {
            DataSource::File(fs) => fs,
            _ => return Err(DataProcessingError::Validation("Expected file source".to_string())),
        };

        // Read file content based on type
        let content = self.read_file_content(&file_source.path, &file_source.content_type).await?;

        // Extract basic metadata
        let mut metadata = input.metadata.clone();
        metadata.insert("file_size".to_string(), file_source.size_bytes.into());
        metadata.insert("last_modified".to_string(),
            serde_json::to_value(file_source.last_modified).unwrap_or(serde_json::Value::Null));

        // Create processed content
        let processed_content = ProcessedContent {
            text_content: match content {
                DataContent::Text(text) => Some(text),
                DataContent::Binary(_) => None, // Would need OCR/extraction
                DataContent::Structured(_) => None,
                DataContent::File(_) => None,
            },
            structured_data: None,
            embeddings: None,
            entities: vec![], // Would be extracted in enrichment stage
            relationships: vec![],
            visual_elements: vec![],
            audio_transcript: None,
        };

        let stats = ProcessingStats {
            processing_time_ms: 100, // Placeholder
            bytes_processed: file_source.size_bytes,
            entities_extracted: 0,
            relationships_found: 0,
            embeddings_generated: 0,
            errors_encountered: vec![],
        };

        Ok(ProcessingOutput {
                    id: input.id.clone(),
            original_input: input,
            processed_content,
            extracted_metadata: metadata,
            processing_stats: stats,
            created_at: chrono::Utc::now(),
        })
    }

    async fn read_file_content(&self, path: &Path, content_type: &ContentType) -> DataProcessingResult<DataContent> {
        match content_type {
            ContentType::Text | ContentType::Markdown | ContentType::Code => {
                let text = tokio::fs::read_to_string(path).await
                    .map_err(|e| DataProcessingError::Io(e))?;
                Ok(DataContent::Text(text))
            }
            ContentType::Json => {
                let text = tokio::fs::read_to_string(path).await
                    .map_err(|e| DataProcessingError::Io(e))?;
                let value: serde_json::Value = serde_json::from_str(&text)
                    .map_err(|e| DataProcessingError::Serialization(e))?;
                Ok(DataContent::Structured(value))
            }
            ContentType::Binary | ContentType::Image | ContentType::Video | ContentType::Audio | ContentType::Pdf => {
                let data = tokio::fs::read(path).await
                    .map_err(|e| DataProcessingError::Io(e))?;
                Ok(DataContent::Binary(data))
            }
            _ => {
                let data = tokio::fs::read(path).await
                    .map_err(|e| DataProcessingError::Io(e))?;
                Ok(DataContent::Binary(data))
            }
        }
    }
}

/// URL-based data ingestion
pub struct UrlIngestor {
    client: reqwest::Client,
}

impl UrlIngestor {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub fn can_ingest(&self, source: &DataSource) -> bool {
        matches!(source, DataSource::Url(_))
    }

    pub async fn ingest(&self, input: DataInput) -> IngestionResult {
        let url_source = match &input.source {
            DataSource::Url(us) => us,
            _ => return Err(DataProcessingError::Validation("Expected URL source".to_string())),
        };

        // Build request with headers
        let mut request = self.client.get(&url_source.url);
        for (key, value) in &url_source.headers {
            request = request.header(key, value);
        }

        // Make request
        let response = request.send().await
            .map_err(|e| DataProcessingError::Http(e.to_string()))?;

        if !response.status().is_success() {
            return Err(DataProcessingError::Http(format!("HTTP {}: {}",
                response.status(), response.status().canonical_reason().unwrap_or("Unknown"))));
        }

        // Get content type from response
        let content_type = response.headers()
            .get("content-type")
            .and_then(|ct| ct.to_str().ok())
            .and_then(|ct| Some(ContentType::from_mime_type(ct)))
            .unwrap_or(ContentType::Unknown);

        // Read response body
        let bytes = response.bytes().await
            .map_err(|e| DataProcessingError::Http(e.to_string()))?;

        let content = match content_type {
            ContentType::Json => {
                let text = String::from_utf8_lossy(&bytes);
                match serde_json::from_str(&text) {
                    Ok(value) => DataContent::Structured(value),
                    Err(_) => DataContent::Text(text.to_string()),
                }
            }
            ContentType::Text | ContentType::Html | ContentType::Xml | ContentType::Markdown => {
                DataContent::Text(String::from_utf8_lossy(&bytes).to_string())
            }
            _ => DataContent::Binary(bytes.to_vec()),
        };

        let mut metadata = input.metadata.clone();
        metadata.insert("url".to_string(), url_source.url.clone().into());
        metadata.insert("response_content_type".to_string(), format!("{:?}", content_type).into());
        metadata.insert("response_size".to_string(), bytes.len().into());

        let processed_content = ProcessedContent {
            text_content: match &content {
                DataContent::Text(text) => Some(text.clone()),
                _ => None,
            },
            structured_data: match &content {
                DataContent::Structured(data) => Some(data.clone()),
                _ => None,
            },
            embeddings: None,
            entities: vec![],
            relationships: vec![],
            visual_elements: vec![],
            audio_transcript: None,
        };

        let stats = ProcessingStats {
            processing_time_ms: 200, // Placeholder
            bytes_processed: bytes.len() as u64,
            entities_extracted: 0,
            relationships_found: 0,
            embeddings_generated: 0,
            errors_encountered: vec![],
        };

        Ok(ProcessingOutput {
                    id: input.id.clone(),
            original_input: input,
            processed_content,
            extracted_metadata: metadata,
            processing_stats: stats,
            created_at: chrono::Utc::now(),
        })
    }
}

/// Stream-based data ingestion
pub struct StreamIngestor;

impl StreamIngestor {
    pub fn new() -> Self {
        Self
    }

    pub fn can_ingest(&self, source: &DataSource) -> bool {
        matches!(source, DataSource::Stream(_))
    }

    pub async fn ingest(&self, input: DataInput) -> IngestionResult {
        let stream_source = match &input.source {
            DataSource::Stream(ss) => ss,
            _ => return Err(DataProcessingError::Validation("Expected stream source".to_string())),
        };

        // For streams, we expect the content to already be provided
        let _content = match &input.content {
            DataContent::Binary(_data) => {
                // Read from stream
                let buffer = Vec::new();
                // Note: This is a simplified implementation
                // In practice, you'd need proper async stream handling
                DataContent::Binary(buffer)
            }
            _ => return Err(DataProcessingError::Validation("Stream input must contain stream content".to_string())),
        };

        let mut metadata = input.metadata.clone();
        metadata.insert("stream_id".to_string(), stream_source.stream_id.clone().into());

        let processed_content = ProcessedContent {
            text_content: None, // Would need format detection
            structured_data: None,
            embeddings: None,
            entities: vec![],
            relationships: vec![],
            visual_elements: vec![],
            audio_transcript: None,
        };

        let stats = ProcessingStats {
            processing_time_ms: 50, // Placeholder
            bytes_processed: 0, // Would track actual bytes read
            entities_extracted: 0,
            relationships_found: 0,
            embeddings_generated: 0,
            errors_encountered: vec![],
        };

        Ok(ProcessingOutput {
                    id: input.id.clone(),
            original_input: input,
            processed_content,
            extracted_metadata: metadata,
            processing_stats: stats,
            created_at: chrono::Utc::now(),
        })
    }
}

/// Database-backed data ingestion
pub struct DatabaseIngestor {
    // Would hold database connection pool
}

impl DatabaseIngestor {
    pub async fn new() -> DataProcessingResult<Self> {
        // Initialize database connection
        Ok(Self {})
    }

    pub fn can_ingest(&self, source: &DataSource) -> bool {
        matches!(source, DataSource::Database(_))
    }

    pub async fn ingest(&self, input: DataInput) -> IngestionResult {
        let db_source = match &input.source {
            DataSource::Database(ds) => ds,
            _ => return Err(DataProcessingError::Validation("Expected database source".to_string())),
        };

        // Query database based on source configuration
        // This is a placeholder - actual implementation would use sqlx or similar
        let content = DataContent::Structured(serde_json::json!({
            "table": db_source.table,
            "record_id": db_source.record_id,
            "fields": db_source.fields
        }));

        let mut metadata = input.metadata.clone();
        metadata.insert("table".to_string(), db_source.table.clone().into());
        metadata.insert("record_id".to_string(), db_source.record_id.clone().into());

        let processed_content = ProcessedContent {
            text_content: None,
            structured_data: Some(match &content {
                DataContent::Structured(data) => data.clone(),
                _ => serde_json::Value::Null,
            }),
            embeddings: None,
            entities: vec![],
            relationships: vec![],
            visual_elements: vec![],
            audio_transcript: None,
        };

        let stats = ProcessingStats {
            processing_time_ms: 75, // Placeholder
            bytes_processed: 0, // Would track serialized size
            entities_extracted: 0,
            relationships_found: 0,
            embeddings_generated: 0,
            errors_encountered: vec![],
        };

        Ok(ProcessingOutput {
                    id: input.id.clone(),
            original_input: input,
            processed_content,
            extracted_metadata: metadata,
            processing_stats: stats,
            created_at: chrono::Utc::now(),
        })
    }
}

/// API-based data ingestion
pub struct ApiIngestor {
    client: reqwest::Client,
}

impl ApiIngestor {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub fn can_ingest(&self, source: &DataSource) -> bool {
        matches!(source, DataSource::Api(_))
    }

    pub async fn ingest(&self, input: DataInput) -> IngestionResult {
        let api_source = match &input.source {
            DataSource::Api(r#as) => r#as,
            _ => return Err(DataProcessingError::Validation("Expected API source".to_string())),
        };

        // Build API request
        let mut request = match api_source.method.as_str() {
            "GET" => self.client.get(&api_source.endpoint),
            "POST" => self.client.post(&api_source.endpoint),
            "PUT" => self.client.put(&api_source.endpoint),
            "DELETE" => self.client.delete(&api_source.endpoint),
            _ => return Err(DataProcessingError::Validation(format!("Unsupported HTTP method: {}", api_source.method))),
        };

        // Add parameters
        for (key, value) in &api_source.parameters {
            request = request.query(&[(key, value)]);
        }

        // Make request
        let response = request.send().await
            .map_err(|e| DataProcessingError::Http(e.to_string()))?;

        if !response.status().is_success() {
            return Err(DataProcessingError::Http(format!("HTTP {}: {}",
                response.status(), response.status().canonical_reason().unwrap_or("Unknown"))));
        }

        // Parse response
        let text = response.text().await
            .map_err(|e| DataProcessingError::Http(e.to_string()))?;

        let content = match serde_json::from_str(&text) {
            Ok(value) => DataContent::Structured(value),
            Err(_) => DataContent::Text(text.clone()),
        };

        let mut metadata = input.metadata.clone();
        metadata.insert("endpoint".to_string(), api_source.endpoint.clone().into());
        metadata.insert("method".to_string(), api_source.method.clone().into());

        let processed_content = ProcessedContent {
            text_content: match &content {
                DataContent::Text(text) => Some(text.clone()),
                _ => None,
            },
            structured_data: match &content {
                DataContent::Structured(data) => Some(data.clone()),
                _ => None,
            },
            embeddings: None,
            entities: vec![],
            relationships: vec![],
            visual_elements: vec![],
            audio_transcript: None,
        };

        let stats = ProcessingStats {
            processing_time_ms: 150, // Placeholder
            bytes_processed: text.len() as u64,
            entities_extracted: 0,
            relationships_found: 0,
            embeddings_generated: 0,
            errors_encountered: vec![],
        };

        Ok(ProcessingOutput {
                    id: input.id.clone(),
            original_input: input,
            processed_content,
            extracted_metadata: metadata,
            processing_stats: stats,
            created_at: chrono::Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::collections::HashMap;
    use std::io::Write;

    #[tokio::test]
    async fn test_file_ingestor_text() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "Hello, world!").unwrap();

        let input = DataInput {
            id: ProcessingId::new(),
            source: DataSource::File(FileSource {
                path: file_path.clone(),
                content_type: ContentType::Text,
                size_bytes: 13,
                last_modified: chrono::Utc::now(),
            }),
            content: DataContent::File(file_path),
            metadata: HashMap::new(),
            processing_context: ProcessingContext {
                request_id: "test".to_string(),
                user_id: None,
                project_scope: None,
                priority: ProcessingPriority::Normal,
                deadline: None,
                tags: vec![],
            },
        };

        let ingestor = FileIngestor::new();
        let result = ingestor.ingest(input).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.processed_content.text_content, Some("Hello, world!".to_string()));
    }

    #[test]
    fn test_default_ingestion_stage_creation() {
        let stage = tokio::runtime::Runtime::new().unwrap().block_on(async {
            DefaultIngestionStage::new().await
        });
        assert!(stage.is_ok());
    }
}
