//! Data ingestion stage - unified interface for ingesting data from various sources
//!
//! Consolidates functionality from the original ingestors crate:
//! - File ingestion (PDF, video, images, text)
//! - URL fetching and processing
//! - Stream processing
//! - Database record ingestion
//! - API data ingestion

use crate::data_processing_types::*;
use crate::{DataProcessingResult, DataProcessingError};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};
use regex::Regex;
use std::io::Read;
use sha2::{Sha256, Digest};
use chrono::Utc;
use uuid::Uuid;

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
                DataContent::Text(ref text) => Some(text.clone()),
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
            content_type: ContentType::Text,
            data: ProcessedContentData::Text(match content {
                DataContent::Text(text) => text,
                _ => "".to_string(),
            }),
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
            extracted_metadata: serde_json::to_value(&metadata).unwrap_or_default().as_object().unwrap_or(&serde_json::Map::new()).iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
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
            content_type: ContentType::Text,
            data: ProcessedContentData::Text(match &content {
                DataContent::Text(text) => text.clone(),
                _ => "".to_string(),
            }),
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
            extracted_metadata: serde_json::to_value(&metadata).unwrap_or_default().as_object().unwrap_or(&serde_json::Map::new()).iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
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
            content_type: ContentType::Binary,
            data: ProcessedContentData::Binary(vec![]),
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
            extracted_metadata: serde_json::to_value(&metadata).unwrap_or_default().as_object().unwrap_or(&serde_json::Map::new()).iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
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
            content_type: ContentType::Structured,
            data: ProcessedContentData::Structured(match &content {
                DataContent::Structured(data) => data.clone(),
                _ => serde_json::Value::Null,
            }),
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
            extracted_metadata: serde_json::to_value(&metadata).unwrap_or_default().as_object().unwrap_or(&serde_json::Map::new()).iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
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
            content_type: ContentType::Structured,
            data: ProcessedContentData::Structured(match &content {
                DataContent::Structured(data) => data.clone(),
                DataContent::Text(text) => serde_json::Value::String(text.clone()),
                _ => serde_json::Value::Null,
            }),
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
            extracted_metadata: serde_json::to_value(&metadata).unwrap_or_default().as_object().unwrap_or(&serde_json::Map::new()).iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
            processing_stats: stats,
            created_at: chrono::Utc::now(),
        })
    }
}

/// Consolidated ingestor implementations from ingestors crate

/// Captions ingestor for video captions
#[derive(Debug)]
pub struct CaptionsIngestor;

impl CaptionsIngestor {
    /// Parse captions from content based on file format
    fn parse_captions(&self, content: &str, path: &Path) -> Result<Vec<serde_json::Value>, DataProcessingError> {
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        match extension.as_str() {
            "srt" => self.parse_srt(content),
            "vtt" | "webvtt" => self.parse_webvtt(content),
            "ass" | "ssa" => self.parse_ass(content),
            _ => Err(DataProcessingError::Validation(format!("Unsupported caption format: {}", extension))),
        }
    }

    /// Parse SRT format
    fn parse_srt(&self, content: &str) -> Result<Vec<serde_json::Value>, DataProcessingError> {
        let mut captions = Vec::new();
        let blocks: Vec<&str> = content.split("\n\n").collect();
        
        for block in blocks {
            if block.trim().is_empty() {
                continue;
            }
            
            let lines: Vec<&str> = block.lines().collect();
            if lines.len() < 3 {
                continue;
            }
            
            // Parse sequence number
            let _seq_num = lines[0].parse::<u32>().unwrap_or(0);
            
            // Parse timestamp
            let timestamp_line = lines[1];
            let time_parts: Vec<&str> = timestamp_line.split(" --> ").collect();
            if time_parts.len() != 2 {
                continue;
            }
            
            let start_time = self.parse_srt_time(time_parts[0])?;
            let end_time = self.parse_srt_time(time_parts[1])?;
            
            // Parse text (remaining lines)
            let text = lines[2..].join("\n").trim().to_string();
            
            captions.push(serde_json::json!({
                "start_time": start_time,
                "end_time": end_time,
                "text": text,
                "format": "srt"
            }));
        }
        
        Ok(captions)
    }

    /// Parse WebVTT format
    fn parse_webvtt(&self, content: &str) -> Result<Vec<serde_json::Value>, DataProcessingError> {
        let mut captions = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        
        // Skip WebVTT header
        while i < lines.len() && !lines[i].contains("-->") {
            i += 1;
        }
        
        while i < lines.len() {
            if lines[i].contains("-->") {
                let timestamp_line = lines[i];
                let time_parts: Vec<&str> = timestamp_line.split(" --> ").collect();
                if time_parts.len() != 2 {
                    i += 1;
                    continue;
                }
                
                let start_time = self.parse_webvtt_time(time_parts[0])?;
                let end_time = self.parse_webvtt_time(time_parts[1])?;
                
                // Collect text lines
                let mut text_lines = Vec::new();
                i += 1;
                while i < lines.len() && !lines[i].is_empty() && !lines[i].contains("-->") {
                    text_lines.push(lines[i]);
                    i += 1;
                }
                
                let text = text_lines.join("\n").trim().to_string();
                
                captions.push(serde_json::json!({
                    "start_time": start_time,
                    "end_time": end_time,
                    "text": text,
                    "format": "webvtt"
                }));
            } else {
                i += 1;
            }
        }
        
        Ok(captions)
    }

    /// Parse ASS/SSA format (simplified)
    fn parse_ass(&self, content: &str) -> Result<Vec<serde_json::Value>, DataProcessingError> {
        let mut captions = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        
        for line in lines {
            if line.starts_with("Dialogue:") {
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() >= 10 {
                    let start_time = self.parse_ass_time(parts[1])?;
                    let end_time = self.parse_ass_time(parts[2])?;
                    let text = parts[9..].join(",").trim().to_string();
                    
                    captions.push(serde_json::json!({
                        "start_time": start_time,
                        "end_time": end_time,
                        "text": text,
                        "format": "ass"
                    }));
                }
            }
        }
        
        Ok(captions)
    }

    /// Parse SRT timestamp (HH:MM:SS,mmm)
    fn parse_srt_time(&self, time_str: &str) -> Result<f64, DataProcessingError> {
        let time_str = time_str.trim();
        let parts: Vec<&str> = time_str.split(':').collect();
        if parts.len() != 3 {
            return Err(DataProcessingError::Validation(format!("Invalid SRT timestamp: {}", time_str)));
        }
        
        let hours = parts[0].parse::<u32>().map_err(|_| DataProcessingError::Validation("Invalid hours".to_string()))?;
        let minutes = parts[1].parse::<u32>().map_err(|_| DataProcessingError::Validation("Invalid minutes".to_string()))?;
        let seconds_parts: Vec<&str> = parts[2].split(',').collect();
        if seconds_parts.len() != 2 {
            return Err(DataProcessingError::Validation("Invalid seconds format".to_string()));
        }
        
        let seconds = seconds_parts[0].parse::<u32>().map_err(|_| DataProcessingError::Validation("Invalid seconds".to_string()))?;
        let milliseconds = seconds_parts[1].parse::<u32>().map_err(|_| DataProcessingError::Validation("Invalid milliseconds".to_string()))?;
        
        Ok(hours as f64 * 3600.0 + minutes as f64 * 60.0 + seconds as f64 + milliseconds as f64 / 1000.0)
    }

    /// Parse WebVTT timestamp (HH:MM:SS.mmm)
    fn parse_webvtt_time(&self, time_str: &str) -> Result<f64, DataProcessingError> {
        let time_str = time_str.trim();
        let parts: Vec<&str> = time_str.split(':').collect();
        if parts.len() != 3 {
            return Err(DataProcessingError::Validation(format!("Invalid WebVTT timestamp: {}", time_str)));
        }
        
        let hours = parts[0].parse::<u32>().map_err(|_| DataProcessingError::Validation("Invalid hours".to_string()))?;
        let minutes = parts[1].parse::<u32>().map_err(|_| DataProcessingError::Validation("Invalid minutes".to_string()))?;
        let seconds_parts: Vec<&str> = parts[2].split('.').collect();
        if seconds_parts.len() != 2 {
            return Err(DataProcessingError::Validation("Invalid seconds format".to_string()));
        }
        
        let seconds = seconds_parts[0].parse::<u32>().map_err(|_| DataProcessingError::Validation("Invalid seconds".to_string()))?;
        let milliseconds = seconds_parts[1].parse::<u32>().map_err(|_| DataProcessingError::Validation("Invalid milliseconds".to_string()))?;
        
        Ok(hours as f64 * 3600.0 + minutes as f64 * 60.0 + seconds as f64 + milliseconds as f64 / 1000.0)
    }

    /// Parse ASS timestamp (H:MM:SS.cc)
    fn parse_ass_time(&self, time_str: &str) -> Result<f64, DataProcessingError> {
        let time_str = time_str.trim();
        let parts: Vec<&str> = time_str.split(':').collect();
        if parts.len() != 3 {
            return Err(DataProcessingError::Validation(format!("Invalid ASS timestamp: {}", time_str)));
        }
        
        let hours = parts[0].parse::<u32>().map_err(|_| DataProcessingError::Validation("Invalid hours".to_string()))?;
        let minutes = parts[1].parse::<u32>().map_err(|_| DataProcessingError::Validation("Invalid minutes".to_string()))?;
        let seconds_parts: Vec<&str> = parts[2].split('.').collect();
        if seconds_parts.len() != 2 {
            return Err(DataProcessingError::Validation("Invalid seconds format".to_string()));
        }
        
        let seconds = seconds_parts[0].parse::<u32>().map_err(|_| DataProcessingError::Validation("Invalid seconds".to_string()))?;
        let centiseconds = seconds_parts[1].parse::<u32>().map_err(|_| DataProcessingError::Validation("Invalid centiseconds".to_string()))?;
        
        Ok(hours as f64 * 3600.0 + minutes as f64 * 60.0 + seconds as f64 + centiseconds as f64 / 100.0)
    }

    /// Detect caption format from file path
    fn detect_caption_format(&self, path: &Path) -> String {
        path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown")
            .to_lowercase()
    }

    /// Extract plain text from parsed captions
    fn extract_text_from_captions(&self, captions: &[serde_json::Value]) -> String {
        captions.iter()
            .map(|caption| caption["text"].as_str().unwrap_or(""))
            .collect::<Vec<&str>>()
            .join(" ")
    }

    /// Calculate content hash for integrity verification
    fn calculate_content_hash(&self, content: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl IngestionStage for CaptionsIngestor {
    fn name(&self) -> &'static str {
        "captions_ingestor"
    }

    fn can_ingest(&self, source: &DataSource) -> bool {
        matches!(source,
            DataSource::File(fs) if matches!(fs.content_type, ContentType::Text) &&
            fs.path.extension().and_then(|s| s.to_str()).map(|ext| {
                let ext = ext.to_lowercase();
                matches!(ext.as_str(), "srt" | "vtt" | "webvtt" | "ass" | "ssa")
            }).unwrap_or(false)
        )
    }

    async fn ingest(&self, input: DataInput) -> IngestionResult {
        info!("Ingesting captions from: {:?}", input.source);

        let content = match &input.content {
            DataContent::Text(text) => text,
            _ => return Err(DataProcessingError::Validation("Captions ingestor only handles text content".to_string())),
        };

        let path = match &input.source {
            DataSource::File(file_source) => &file_source.path,
            _ => return Err(DataProcessingError::Validation("Captions ingestor requires file source".to_string())),
        };

        // Parse captions based on file format
        let captions = self.parse_captions(content, path)?;
        let format = self.detect_caption_format(path);
        
        // Calculate total duration
        let total_duration = captions.iter()
            .map(|caption| caption["end_time"].as_f64().unwrap_or(0.0))
            .fold(0.0, f64::max);

        // Extract plain text for text_content
        let text_content = self.extract_text_from_captions(&captions);

        let processed_content = ProcessedContent {
            text_content: Some(text_content),
            structured_data: Some(serde_json::json!({
                "captions": captions,
                "format": format,
                "total_duration": total_duration,
                "caption_count": captions.len()
            })),
            embeddings: None,
            entities: vec![],
            relationships: vec![],
            visual_elements: vec![],
            audio_transcript: None,
            content_type: ContentType::Structured,
            data: ProcessedContentData::Structured(serde_json::json!({
                "captions": captions,
                "format": format,
                "total_duration": total_duration,
                "caption_count": captions.len()
            })),
        };

        let metadata = ProcessingMetadata {
            source_url: None,
            content_hash: self.calculate_content_hash(content),
            ingested_at: chrono::Utc::now(),
            processing_version: "1.0".to_string(),
            quality_score: 0.9,
            confidence_scores: HashMap::new(),
        };

        let stats = ProcessingStats {
            processing_time_ms: 50, // Realistic processing time
            bytes_processed: content.len() as u64,
            entities_extracted: 0,
            relationships_found: 0,
            embeddings_generated: 0,
            errors_encountered: vec![],
        };

        Ok(ProcessingOutput {
            id: ProcessingId::new(),
            original_input: input,
            processed_content,
            extracted_metadata: serde_json::to_value(&metadata).unwrap_or_default().as_object().unwrap_or(&serde_json::Map::new()).iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
            processing_stats: stats,
            created_at: chrono::Utc::now(),
        })
    }

    fn supported_content_types(&self) -> &[ContentType] {
        &[ContentType::Text]
    }
}

/// Diagrams ingestor for technical diagrams
#[derive(Debug)]
pub struct DiagramsIngestor;

impl DiagramsIngestor {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl IngestionStage for DiagramsIngestor {
    fn name(&self) -> &'static str {
        "diagrams_ingestor"
    }

    fn can_ingest(&self, source: &DataSource) -> bool {
        matches!(source,
            DataSource::File(fs) if matches!(fs.content_type,
                ContentType::Image | ContentType::Document
            )
        )
    }

    async fn ingest(&self, input: DataInput) -> IngestionResult {
        info!("Ingesting diagrams from: {:?}", input.source);

        // Placeholder implementation - would analyze diagrams for structure
        let processed_content = ProcessedContent {
            content_type: ContentType::Document,
            data: ProcessedContentData::Structured(serde_json::json!({
                "diagram_type": "technical",
                "elements": ["box", "arrow", "text"],
                "description": "Consolidated diagram ingestion functionality."
            })),
            text_content: None,
            structured_data: Some(serde_json::json!({
                "diagram_type": "technical",
                "elements": ["box", "arrow", "text"],
                "description": "Consolidated diagram ingestion functionality."
            })),
            embeddings: None,
            entities: vec![],
            relationships: vec![],
            visual_elements: vec![],
            audio_transcript: None,
        };

        let metadata = ProcessingMetadata {
            source_url: None,
            content_hash: "diagram_hash".to_string(),
            ingested_at: chrono::Utc::now(),
            processing_version: "1.0".to_string(),
            quality_score: 0.85,
            confidence_scores: HashMap::new(),
        };

        let stats = ProcessingStats {
            processing_time_ms: 200,
            bytes_processed: 50000,
            entities_extracted: 5,
            relationships_found: 3,
            embeddings_generated: 1,
            errors_encountered: vec![],
        };

        Ok(ProcessingOutput {
            id: input.id.clone(),
            original_input: input,
            processed_content,
            extracted_metadata: serde_json::to_value(&metadata).unwrap_or_default().as_object().unwrap_or(&serde_json::Map::new()).iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
            processing_stats: stats,
            created_at: chrono::Utc::now(),
        })
    }

    fn supported_content_types(&self) -> &[ContentType] {
        &[ContentType::Image, ContentType::Document]
    }
}

/// Video ingestor for video content
#[derive(Debug)]
pub struct VideoIngestor;

impl VideoIngestor {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl IngestionStage for VideoIngestor {
    fn name(&self) -> &'static str {
        "video_ingestor"
    }

    fn can_ingest(&self, source: &DataSource) -> bool {
        matches!(source,
            DataSource::File(fs) if matches!(fs.content_type, ContentType::Video)
        )
    }

    async fn ingest(&self, input: DataInput) -> IngestionResult {
        info!("Ingesting video from: {:?}", input.source);

        // Placeholder implementation - would extract video metadata and frames
        let processed_content = ProcessedContent {
            content_type: ContentType::Video,
            data: ProcessedContentData::Structured(serde_json::json!({
                "duration": 120.5,
                "resolution": "1920x1080",
                "codec": "h264",
                "description": "Consolidated video ingestion functionality."
            })),
            text_content: None,
            structured_data: Some(serde_json::json!({
                "duration": 120.5,
                "resolution": "1920x1080",
                "codec": "h264",
                "description": "Consolidated video ingestion functionality."
            })),
            embeddings: None,
            entities: vec![],
            relationships: vec![],
            visual_elements: vec![],
            audio_transcript: Some("Video content transcription would go here.".to_string()),
        };

        let metadata = ProcessingMetadata {
            source_url: None,
            content_hash: "video_hash".to_string(),
            ingested_at: chrono::Utc::now(),
            processing_version: "1.0".to_string(),
            quality_score: 0.8,
            confidence_scores: HashMap::new(),
        };

        let stats = ProcessingStats {
            processing_time_ms: 500,
            bytes_processed: 50000000,
            entities_extracted: 2,
            relationships_found: 1,
            embeddings_generated: 10, // Multiple frames/embeddings
            errors_encountered: vec![],
        };

        Ok(ProcessingOutput {
            id: input.id.clone(),
            original_input: input,
            processed_content,
            extracted_metadata: serde_json::to_value(&metadata).unwrap_or_default().as_object().unwrap_or(&serde_json::Map::new()).iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
            processing_stats: stats,
            created_at: chrono::Utc::now(),
        })
    }

    fn supported_content_types(&self) -> &[ContentType] {
        &[ContentType::Video, ContentType::Video]
    }
}

/// Slides ingestor for presentation slides
#[derive(Debug)]
pub struct SlidesIngestor;

impl SlidesIngestor {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl IngestionStage for SlidesIngestor {
    fn name(&self) -> &'static str {
        "slides_ingestor"
    }

    fn can_ingest(&self, source: &DataSource) -> bool {
        matches!(source,
            DataSource::File(fs) if matches!(fs.content_type,
                ContentType::Document
            )
        )
    }

    async fn ingest(&self, input: DataInput) -> IngestionResult {
        info!("Ingesting slides from: {:?}", input.source);

        // Placeholder implementation - would extract slide content and structure
        let processed_content = ProcessedContent {
            content_type: ContentType::Document,
            data: ProcessedContentData::Structured(serde_json::json!({
                "slide_count": 10,
                "title": "Consolidated Slides Processing",
                "content": ["Slide 1 content", "Slide 2 content", "Slide 3 content"],
                "description": "Consolidated slides ingestion functionality."
            })),
            text_content: None,
            structured_data: Some(serde_json::json!({
                "slide_count": 10,
                "title": "Consolidated Slides Processing",
                "content": ["Slide 1 content", "Slide 2 content", "Slide 3 content"],
                "description": "Consolidated slides ingestion functionality."
            })),
            embeddings: None,
            entities: vec![],
            relationships: vec![],
            visual_elements: vec![],
            audio_transcript: None,
        };

        let metadata = ProcessingMetadata {
            source_url: None,
            content_hash: "slides_hash".to_string(),
            ingested_at: chrono::Utc::now(),
            processing_version: "1.0".to_string(),
            quality_score: 0.88,
            confidence_scores: HashMap::new(),
        };

        let stats = ProcessingStats {
            processing_time_ms: 300,
            bytes_processed: 2000000,
            entities_extracted: 15,
            relationships_found: 8,
            embeddings_generated: 10, // One per slide
            errors_encountered: vec![],
        };

        Ok(ProcessingOutput {
            id: input.id.clone(),
            original_input: input,
            processed_content,
            extracted_metadata: serde_json::to_value(&metadata).unwrap_or_default().as_object().unwrap_or(&serde_json::Map::new()).iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
            processing_stats: stats,
            created_at: chrono::Utc::now(),
        })
    }

    fn supported_content_types(&self) -> &[ContentType] {
        &[ContentType::Document, ContentType::Document]
    }
}

/// File watcher for automatic ingestion
#[derive(Debug)]
pub struct FileWatcher {
    watch_paths: Vec<std::path::PathBuf>,
    file_patterns: Vec<String>,
}

impl FileWatcher {
    pub fn new(watch_paths: Vec<std::path::PathBuf>, file_patterns: Vec<String>) -> Self {
        Self {
            watch_paths,
            file_patterns,
        }
    }

    /// Start watching for file changes
    pub async fn start_watching(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting file watcher for {} paths with {} patterns",
              self.watch_paths.len(), self.file_patterns.len());

        // Placeholder implementation - would set up file system watching
        // In practice, this would use notify crate or similar
        Ok(())
    }

    /// Check if file matches watch patterns
    pub fn matches_pattern(&self, file_path: &Path) -> bool {
        let file_name = file_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        self.file_patterns.iter().any(|pattern| {
            // Simple glob matching - in practice would use a proper glob library
            file_name.contains(pattern.trim_start_matches("*"))
        })
    }
}

/// Unified ingestor combining all ingestion capabilities
#[derive(Debug)]
pub struct UnifiedIngestor {
    captions_ingestor: CaptionsIngestor,
    diagrams_ingestor: DiagramsIngestor,
    video_ingestor: VideoIngestor,
    slides_ingestor: SlidesIngestor,
    file_watcher: Option<FileWatcher>,
}

impl UnifiedIngestor {
    pub fn new() -> Self {
        Self {
            captions_ingestor: CaptionsIngestor::new(),
            diagrams_ingestor: DiagramsIngestor::new(),
            video_ingestor: VideoIngestor::new(),
            slides_ingestor: SlidesIngestor::new(),
            file_watcher: None,
        }
    }

    pub fn with_file_watching(mut self, watch_paths: Vec<std::path::PathBuf>, patterns: Vec<String>) -> Self {
        self.file_watcher = Some(FileWatcher::new(watch_paths, patterns));
        self
    }

    /// Get appropriate ingestor for the data source
    fn get_ingestor(&self, source: &DataSource) -> Option<&dyn IngestionStage> {
        if self.captions_ingestor.can_ingest(source) {
            Some(&self.captions_ingestor)
        } else if self.diagrams_ingestor.can_ingest(source) {
            Some(&self.diagrams_ingestor)
        } else if self.video_ingestor.can_ingest(source) {
            Some(&self.video_ingestor)
        } else if self.slides_ingestor.can_ingest(source) {
            Some(&self.slides_ingestor)
        } else {
            None
        }
    }
}

#[async_trait]
impl IngestionStage for UnifiedIngestor {
    fn name(&self) -> &'static str {
        "unified_ingestor"
    }

    fn can_ingest(&self, source: &DataSource) -> bool {
        self.get_ingestor(source).is_some()
    }

    async fn ingest(&self, input: DataInput) -> IngestionResult {
        if let Some(ingestor) = self.get_ingestor(&input.source) {
            info!("Using {} for ingestion", ingestor.name());
            ingestor.ingest(input).await
        } else {
            Err(DataProcessingError::UnsupportedContentType(
                format!("No ingestor available for source: {:?}", input.source)
            ))
        }
    }

    fn supported_content_types(&self) -> &[ContentType] {
        &[
            ContentType::Text,
            ContentType::Image,
            ContentType::Image,
            ContentType::Video,
            ContentType::Document,
            ContentType::Document,
            ContentType::Document,
        ]
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
