//! Research agent types and data structures

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Research query types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QueryType {
    /// General knowledge search
    Knowledge,
    /// Code-specific research
    Code,
    /// Documentation search
    Documentation,
    /// API reference lookup
    ApiReference,
    /// Error troubleshooting
    Troubleshooting,
    /// Best practices research
    BestPractices,
    /// Technical research
    Technical,
}

/// Research priority levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ResearchPriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

/// Search context for multimodal knowledge seeking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchContext {
    pub project_scope: Option<String>,
}

/// Configuration update for research agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationUpdate {
    pub field: String,
    pub value: serde_json::Value,
}

/// Knowledge query for research
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeQuery {
    pub id: Uuid,
    pub query: String,
    pub query_type: QueryType,
    pub max_results: Option<usize>,
    pub context: Option<String>,
    pub filters: HashMap<String, serde_json::Value>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Search result from various sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub url: Option<String>,
    pub source: String,
    pub relevance_score: f64,
    pub credibility_score: f64,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Knowledge source types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KnowledgeSource {
    /// Web page content
    WebPage(String),
    /// Documentation
    Documentation(String),
    /// Code repository
    CodeRepository(String),
    /// API documentation
    ApiDocumentation(String),
    /// Forum or community post
    CommunityPost(String),
    /// Academic paper
    AcademicPaper(String),
    /// Internal knowledge base
    InternalKnowledgeBase(String),
}

impl std::fmt::Display for KnowledgeSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KnowledgeSource::WebPage(url) => write!(f, "WebPage: {}", url),
            KnowledgeSource::Documentation(doc) => write!(f, "Documentation: {}", doc),
            KnowledgeSource::CodeRepository(repo) => write!(f, "CodeRepository: {}", repo),
            KnowledgeSource::ApiDocumentation(api) => write!(f, "ApiDocumentation: {}", api),
            KnowledgeSource::CommunityPost(post) => write!(f, "CommunityPost: {}", post),
            KnowledgeSource::AcademicPaper(paper) => write!(f, "AcademicPaper: {}", paper),
            KnowledgeSource::InternalKnowledgeBase(kb) => {
                write!(f, "InternalKnowledgeBase: {}", kb)
            }
        }
    }
}

/// Research query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchQuery {
    pub id: Uuid,
    pub query: String,
    pub query_type: QueryType,
    pub priority: ResearchPriority,
    pub context: Option<String>,
    pub max_results: Option<u32>,
    pub sources: Vec<KnowledgeSource>,
    pub created_at: DateTime<Utc>,
    pub deadline: Option<DateTime<Utc>>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Research result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchResult {
    pub query_id: Uuid,
    pub source: KnowledgeSource,
    pub title: String,
    pub content: String,
    pub summary: Option<String>,
    pub relevance_score: f32,
    pub confidence_score: f32,
    pub extracted_at: DateTime<Utc>,
    pub url: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Synthesized context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesizedContext {
    pub id: Uuid,
    pub query_id: Uuid,
    pub context_summary: String,
    pub key_findings: Vec<String>,
    pub supporting_evidence: Vec<ResearchResult>,
    pub confidence_score: f32,
    pub synthesized_at: DateTime<Utc>,
    pub sources: Vec<KnowledgeSource>,
    pub cross_references: Vec<CrossReference>,
}

/// Cross-reference between knowledge sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossReference {
    pub source_id: Uuid,
    pub target_id: Uuid,
    pub relationship: CrossReferenceType,
    pub strength: f32,
    pub context: String,
}

/// Types of cross-references
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrossReferenceType {
    /// Source supports target
    Supports,
    /// Source contradicts target
    Contradicts,
    /// Source builds upon target
    BuildsUpon,
    /// Source references target
    References,
    /// Source is similar to target
    Similar,
    /// Source is related to target
    Related,
}

/// Vector embedding for semantic search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorEmbedding {
    pub id: Uuid,
    pub content_id: Uuid,
    pub vector: Vec<f32>,
    pub model: String,
    pub dimension: u32,
    pub created_at: DateTime<Utc>,
}

/// Knowledge base entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEntry {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub source: KnowledgeSource,
    pub source_url: Option<String>,
    pub content_type: ContentType,
    pub language: Option<String>,
    pub tags: Vec<String>,
    pub embedding: Option<VectorEmbedding>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub access_count: u64,
    pub last_accessed: Option<DateTime<Utc>>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Content types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentType {
    /// Plain text content
    Text,
    /// Markdown content
    Markdown,
    /// HTML content
    Html,
    /// Code content
    Code,
    /// Documentation
    Documentation,
    /// API specification
    ApiSpec,
    /// Tutorial or guide
    Tutorial,
    /// Reference material
    Reference,
    /// Error message or log
    Error,
}

/// Web scraping result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebScrapingResult {
    pub url: String,
    pub title: String,
    pub content: String,
    pub content_type: ContentType,
    pub scraped_at: DateTime<Utc>,
    pub status_code: u16,
    pub content_size: usize,
    pub processing_time_ms: u64,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Content processing result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentProcessingResult {
    pub original_content: String,
    pub processed_content: String,
    pub extracted_text: String,
    pub summary: Option<String>,
    pub key_phrases: Vec<String>,
    pub entities: Vec<String>,
    pub links: Vec<String>,
    pub processing_time_ms: u64,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Research performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchMetrics {
    pub total_queries: u64,
    pub successful_queries: u64,
    pub failed_queries: u64,
    pub average_response_time_ms: f64,
    pub average_relevance_score: f32,
    pub average_confidence_score: f32,
    pub cache_hit_rate: f32,
    pub vector_search_accuracy: f32,
    pub web_scraping_success_rate: f32,
    pub context_synthesis_quality: f32,
    pub fuzzy_match_adjustments: u64,
    pub last_updated: DateTime<Utc>,
}

/// Research agent status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResearchAgentStatus {
    /// Agent is available for queries
    Available,
    /// Agent is processing a query
    Busy,
    /// Agent is in maintenance mode
    Maintenance,
    /// Agent has encountered an error
    Error(String),
    /// Agent is initializing
    Initializing,
}

/// Research configuration update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchConfigUpdate {
    pub field: String,
    pub value: serde_json::Value,
    pub updated_at: DateTime<Utc>,
}

/// Research session for tracking related queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchSession {
    pub id: Uuid,
    pub session_name: String,
    pub queries: Vec<Uuid>,
    pub context: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub is_active: bool,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Research agent capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchCapabilities {
    pub supported_query_types: Vec<QueryType>,
    pub supported_sources: Vec<KnowledgeSource>,
    pub max_concurrent_queries: u32,
    pub max_context_size: usize,
    pub vector_search_enabled: bool,
    pub web_scraping_enabled: bool,
    pub content_processing_enabled: bool,
    pub context_synthesis_enabled: bool,
    pub real_time_updates: bool,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub max_concurrent_requests: usize,
    pub request_timeout_ms: u64,
}

/// Research agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchAgentConfig {
    pub vector_search: VectorSearchConfig,
    pub web_scraping: WebScrapingConfig,
    pub context_synthesis: ContextSynthesisConfig,
    pub performance: PerformanceConfig,
    pub fuzzy_matching: FuzzyMatchingConfig,
}

/// Vector search configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorSearchConfig {
    pub enabled: bool,
    pub qdrant_url: String,
    pub collection_name: String,
    pub model: String,
    pub dimension: u32,
    pub similarity_threshold: f32,
    pub max_results: u32,
    pub batch_size: u32,
}

/// Web scraping configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebScrapingConfig {
    pub enabled: bool,
    pub max_depth: u32,
    pub max_pages: u32,
    pub timeout_ms: u64,
    pub timeout_seconds: u64,
    pub user_agent: String,
    pub respect_robots_txt: bool,
    pub allowed_domains: Vec<String>,
    pub rate_limit_per_minute: u32,
}

/// Context synthesis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSynthesisConfig {
    pub enabled: bool,
    pub similarity_threshold: f32,
    pub max_cross_references: usize,
    pub max_context_size: usize,
    pub synthesis_timeout_ms: u64,
}

/// Fuzzy matching configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuzzyMatchingConfig {
    pub enabled: bool,
    pub similarity_threshold: f32,
    pub boost_per_match: f32,
    pub coverage_boost: f32,
    pub max_total_boost: f32,
}
