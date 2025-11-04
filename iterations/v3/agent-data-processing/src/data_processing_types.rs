//! Core types for the data processing pipeline
//!
//! These types are shared across all pipeline stages and provide
//! the data contracts between ingestion, enrichment, indexing, knowledge, and operations.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use system_configuration::geometry::BoundingBox;
#[cfg(feature = "memory-integration")]
use agent_memory::graph_engine::{Relationship, RelationshipType};

// Stub definitions for when memory integration is not available
#[cfg(not(feature = "memory-integration"))]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Relationship {
    pub id: String,
    pub source_entity: String,
    pub target_entity: String,
    pub relationship_type: RelationshipType,
    pub confidence: f64,
    pub evidence: Vec<String>,
}

#[cfg(not(feature = "memory-integration"))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum RelationshipType {
    WorksFor,
    LocatedIn,
    PartOf,
    Created,
    Owns,
    RelatedTo,
    Other(String),
}

#[cfg(not(feature = "memory-integration"))]
impl std::fmt::Display for RelationshipType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RelationshipType::WorksFor => write!(f, "works_for"),
            RelationshipType::LocatedIn => write!(f, "located_in"),
            RelationshipType::PartOf => write!(f, "part_of"),
            RelationshipType::Created => write!(f, "created"),
            RelationshipType::Owns => write!(f, "owns"),
            RelationshipType::RelatedTo => write!(f, "related_to"),
            RelationshipType::Other(s) => write!(f, "{}", s),
        }
    }
}

/// Basic data block for processing
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Block {
    pub id: ProcessingId,
    pub content_type: ContentType,
    pub data: BlockData,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Data contained in a block
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum BlockData {
    Text(String),
    Binary(Vec<u8>),
    Structured(serde_json::Value),
}

/// Enriched block with additional processing results
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnrichedBlock {
    pub block: Block,
    pub enriched_content: EnrichedContent,
    pub processing_metadata: ProcessingMetadata,
}

/// Enriched content from processing stages
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnrichedContent {
    pub entities: Vec<ExtractedEntity>,
    pub visual_elements: Vec<VisualElement>,
    pub audio_transcript: Option<String>,
    pub topics: Vec<ExtractedTopic>,
    pub embeddings: Option<Vec<f32>>,
}

/// Extracted entity from content
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExtractedEntity {
    pub id: String,
    pub name: String,
    pub entity_type: String,
    pub confidence: f32,
    pub positions: Vec<TextPosition>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Visual element detected in content
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VisualElement {
    pub element_type: VisualElementType,
    pub position: BoundingBox,
    pub confidence: f32,
    pub text_content: Option<String>,
    pub description: Option<String>,
}

/// Type of visual element
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum VisualElementType {
    Image,
    Text,
    Diagram,
    Chart,
    Table,
}

/// Extracted topic from content
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExtractedTopic {
    pub name: String,
    pub confidence: f32,
    pub keywords: Vec<String>,
}

/// Text position information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TextPosition {
    pub start: usize,
    pub end: usize,
    pub page: Option<u32>,
}

/// Unique identifier for data processing operations
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct ProcessingId(#[schemars(with = "String")] pub Uuid);

impl ProcessingId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ProcessingId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<ProcessingId> for Uuid {
    fn from(processing_id: ProcessingId) -> Self {
        processing_id.0
    }
}

impl std::fmt::Display for ProcessingId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Raw input data for processing
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DataInput {
    pub id: ProcessingId,
    pub source: DataSource,
    pub content: DataContent,
    pub metadata: HashMap<String, serde_json::Value>,
    pub processing_context: ProcessingContext,
}

/// Source of the data being processed
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum DataSource {
    File(FileSource),
    Url(UrlSource),
    Stream(StreamSource),
    Database(DatabaseSource),
    Api(ApiSource),
}

/// File-based data source
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FileSource {
    pub path: PathBuf,
    pub content_type: ContentType,
    pub size_bytes: u64,
    #[schemars(with = "String")]
    pub last_modified: DateTime<Utc>,
}

/// URL-based data source
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UrlSource {
    pub url: String,
    pub content_type: Option<ContentType>,
    pub headers: HashMap<String, String>,
}

/// Streaming data source
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StreamSource {
    pub stream_id: String,
    pub content_type: ContentType,
}

/// Database-backed data source
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DatabaseSource {
    pub table: String,
    pub record_id: String,
    pub fields: Vec<String>,
}

/// API-based data source
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ApiSource {
    pub endpoint: String,
    pub method: String,
    pub parameters: HashMap<String, String>,
}

/// Content types supported by the pipeline
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum ContentType {
    Text,
    Pdf,
    Image,
    Video,
    Audio,
    Html,
    Json,
    Xml,
    Binary,
    Markdown,
    Code,
    Structured,
    Document,
    Unknown,
}

impl std::fmt::Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ContentType {
    pub fn from_mime_type(mime: &str) -> Self {
        match mime {
            "text/plain" | "text/markdown" => ContentType::Text,
            "application/pdf" => ContentType::Pdf,
            m if m.starts_with("image/") => ContentType::Image,
            m if m.starts_with("video/") => ContentType::Video,
            m if m.starts_with("audio/") => ContentType::Audio,
            "text/html" => ContentType::Html,
            "application/json" => ContentType::Json,
            "application/xml" | "text/xml" => ContentType::Xml,
            "text/x-markdown" => ContentType::Markdown,
            m if m.contains("javascript") || m.contains("typescript") || m.contains("rust") => ContentType::Code,
            _ => ContentType::Unknown,
        }
    }

    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "txt" | "md" => ContentType::Text,
            "pdf" => ContentType::Pdf,
            "jpg" | "jpeg" | "png" | "gif" | "webp" | "svg" => ContentType::Image,
            "mp4" | "avi" | "mov" | "mkv" => ContentType::Video,
            "mp3" | "wav" | "flac" | "aac" => ContentType::Audio,
            "html" | "htm" => ContentType::Html,
            "json" => ContentType::Json,
            "xml" => ContentType::Xml,
            "rs" | "js" | "ts" | "py" | "java" | "cpp" | "c" => ContentType::Code,
            _ => ContentType::Unknown,
        }
    }
}

/// The actual content data
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum DataContent {
    Text(String),
    Binary(Vec<u8>),
    Structured(serde_json::Value),
    File(PathBuf),
}

/// Processing context and metadata
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProcessingContext {
    pub request_id: String,
    pub user_id: Option<String>,
    pub project_scope: Option<String>,
    pub priority: ProcessingPriority,
    #[schemars(with = "Option<String>")]
    pub deadline: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
}

/// Processing priority levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ProcessingPriority {
    Low,
    Normal,
    High,
    Critical,
}

impl Default for ProcessingPriority {
    fn default() -> Self {
        ProcessingPriority::Normal
    }
}

/// Output from data processing pipeline
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProcessingOutput {
    pub id: ProcessingId,
    pub original_input: DataInput,
    pub processed_content: ProcessedContent,
    pub extracted_metadata: HashMap<String, serde_json::Value>,
    pub processing_stats: ProcessingStats,
    #[schemars(with = "String")]
    pub created_at: DateTime<Utc>,
}

/// Raw processed content data
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum ProcessedContentData {
    Text(String),
    Binary(Vec<u8>),
    Structured(serde_json::Value),
}

/// Processed content with multiple representations
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProcessedContent {
    pub data: ProcessedContentData,
    pub content_type: ContentType,
    pub text_content: Option<String>,
    pub structured_data: Option<serde_json::Value>,
    pub embeddings: Option<Vec<f32>>,
    pub entities: Vec<Entity>,
    pub relationships: Vec<Relationship>,
    pub visual_elements: Vec<VisualElement>,
    pub audio_transcript: Option<String>,
}


/// Named entity extracted from content
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Entity {
    pub id: String,
    pub name: String,
    pub entity_type: EntityType,
    pub confidence: f64,
    pub positions: Vec<TextPosition>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Types of entities that can be extracted
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum EntityType {
    Person,
    Organization,
    Location,
    Date,
    Time,
    Money,
    Percentage,
    Product,
    Event,
    Other(String),
}



/// Query for retrieving processed data
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DataQuery {
    pub query_type: QueryType,
    pub text_query: Option<String>,
    pub semantic_vector: Option<Vec<f32>>,
    pub entity_filters: Vec<EntityFilter>,
    pub content_filters: Vec<ContentFilter>,
    pub limit: usize,
    pub context: ProcessingContext,
}

/// Types of queries supported
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum QueryType {
    TextSearch,
    SemanticSearch,
    EntitySearch,
    HybridSearch,
}

/// Filter for entity-based queries
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EntityFilter {
    pub entity_type: EntityType,
    pub entity_names: Vec<String>,
    pub min_confidence: f64,
}

/// Filter for content-based queries
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContentFilter {
    pub content_type: ContentType,
    pub date_range: Option<DateRange>,
    pub tags: Vec<String>,
}

/// Date range for filtering
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DateRange {
    #[schemars(with = "String")]
    pub start: DateTime<Utc>,
    #[schemars(with = "String")]
    pub end: DateTime<Utc>,
}

/// Retrieved data from queries
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RetrievedData {
    pub id: ProcessingId,
    pub content: ProcessedContent,
    pub relevance_score: f64,
    pub matched_entities: Vec<Entity>,
    pub source_metadata: HashMap<String, serde_json::Value>,
}

impl RetrievedData {
    /// Enhance retrieved data with contextual information
    #[cfg(feature = "memory-integration")]
    pub fn enhance_with_context(&mut self, _context_memories: &[agent_memory::ContextualMemory]) {
        #[cfg(feature = "memory-integration")]
        {
            // Enhance relevance score based on contextual memories
            // This would use agent-memory to boost relevance for contextually relevant results
            // Implementation details would depend on the specific memory integration
        }
    }
}

/// Processing metadata
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProcessingMetadata {
    pub source_url: Option<String>,
    pub content_hash: String,
    #[schemars(with = "String")]
    pub ingested_at: chrono::DateTime<chrono::Utc>,
    pub processing_version: String,
    pub quality_score: f64,
    pub confidence_scores: HashMap<String, f64>,
}

/// Processing statistics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProcessingStats {
    pub processing_time_ms: u64,
    pub bytes_processed: u64,
    pub entities_extracted: usize,
    pub relationships_found: usize,
    pub embeddings_generated: usize,
    pub errors_encountered: Vec<String>,
}

/// System-wide statistics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SystemStats {
    pub pipeline: PipelineStats,
    #[cfg(feature = "memory-integration")]
    pub memory: agent_memory::MemoryStats,
    #[cfg(feature = "workspace-integration")]
    pub workspace: crate::workspace_hooks::WorkspaceStats,
}

/// Pipeline-level statistics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PipelineStats {
    pub total_processed: u64,
    pub active_operations: usize,
    pub queue_depth: usize,
    pub error_rate: f64,
    pub avg_processing_time_ms: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_type_from_extension() {
        assert_eq!(ContentType::from_extension("rs"), ContentType::Code);
        assert_eq!(ContentType::from_extension("pdf"), ContentType::Pdf);
        assert_eq!(ContentType::from_extension("jpg"), ContentType::Image);
        assert_eq!(ContentType::from_extension("unknown"), ContentType::Unknown);
    }

    #[test]
    fn test_content_type_from_mime_type() {
        assert_eq!(ContentType::from_mime_type("text/plain"), ContentType::Text);
        assert_eq!(ContentType::from_mime_type("application/pdf"), ContentType::Pdf);
        assert_eq!(ContentType::from_mime_type("image/jpeg"), ContentType::Image);
        assert_eq!(ContentType::from_mime_type("unknown/type"), ContentType::Unknown);
    }

    #[test]
    fn test_processing_id_generation() {
        let id1 = ProcessingId::new();
        let id2 = ProcessingId::new();
        assert_ne!(id1, id2);
    }
}
