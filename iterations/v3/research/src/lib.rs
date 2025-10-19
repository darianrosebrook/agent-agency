//! Agent Agency V3 - Research Agent
//!
//! Provides intelligent knowledge gathering, context synthesis, and research capabilities
//! for the Agent Agency system with vector search integration and web scraping.

pub mod confidence_manager;
pub mod content_processor;
pub mod context_builder;
pub mod embeddings;
// pub mod enhanced_knowledge_seeker; // Temporarily disabled due to compilation issues
pub mod information_processor;
pub mod knowledge_seeker;
pub mod multimodal_retriever;
pub mod multimodal_context_provider;
pub mod types;
pub mod vector_search;
pub mod web_scraper;

pub use confidence_manager::{ConfidenceManager, ConfidenceManagerConfig, IConfidenceManager};
pub use content_processor::ContentProcessor;
pub use context_builder::{ContextBuilder, CrossReferenceDetector};
// pub use enhanced_knowledge_seeker::{
//     EnhancedKnowledgeSeeker, EnhancedKnowledgeSeekerConfig, IEnhancedKnowledgeSeeker,
// }; // Temporarily commented due to compilation issues
pub use information_processor::{
    IInformationProcessor, InformationProcessor, InformationProcessorConfig,
};
pub use knowledge_seeker::KnowledgeSeeker;
pub use multimodal_retriever::{
    MultimodalQuery, MultimodalRetriever, MultimodalRetrieverConfig,
};
pub use multimodal_context_provider::{
    MultimodalContextProvider, 
    MultimodalContext, 
    EvidenceItem, 
    Citation,
    ContextBudget,
};
pub use types::*;
pub use vector_search::VectorSearchEngine;
pub use web_scraper::WebScraper;

/// Research agent configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResearchConfig {
    /// Vector database configuration
    pub vector_db: VectorDbConfig,
    /// Web scraping configuration
    pub web_scraping: WebScrapingConfig,
    /// Content processing configuration
    pub content_processing: ContentProcessingConfig,
    /// Context synthesis configuration
    pub context_synthesis: ContextSynthesisConfig,
    /// Performance settings
    pub performance: PerformanceConfig,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VectorDbConfig {
    /// Qdrant database URL
    pub qdrant_url: String,
    /// Collection name for knowledge base
    pub collection_name: String,
    /// Vector dimension size
    pub vector_size: u32,
    /// Similarity threshold for search
    pub similarity_threshold: f32,
    /// Maximum results per search
    pub max_results: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WebScrapingConfig {
    /// User agent for requests
    pub user_agent: String,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// Maximum content size to scrape (bytes)
    pub max_content_size: usize,
    /// Rate limiting (requests per minute)
    pub rate_limit_per_minute: u32,
    /// Allowed domains (empty = all domains)
    pub allowed_domains: Vec<String>,
    /// Blocked domains
    pub blocked_domains: Vec<String>,
    /// Search engines to use
    pub search_engines: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContentProcessingConfig {
    /// Enable content cleaning
    pub enable_cleaning: bool,
    /// Enable markdown conversion
    pub enable_markdown: bool,
    /// Enable text extraction
    pub enable_text_extraction: bool,
    /// Maximum content length
    pub max_content_length: usize,
    /// Enable content summarization
    pub enable_summarization: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContextSynthesisConfig {
    /// Maximum context window size
    pub max_context_size: usize,
    /// Context overlap percentage
    pub context_overlap_percent: f32,
    /// Enable semantic chunking
    pub enable_semantic_chunking: bool,
    /// Chunk size for processing
    pub chunk_size: usize,
    /// Enable cross-reference detection
    pub enable_cross_references: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PerformanceConfig {
    /// Maximum concurrent requests
    pub max_concurrent_requests: u32,
    /// Request timeout in milliseconds
    pub request_timeout_ms: u64,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
    /// Enable request caching
    pub enable_caching: bool,
    /// Enable performance monitoring
    pub enable_monitoring: bool,
}

impl Default for ResearchConfig {
    fn default() -> Self {
        Self {
            vector_db: VectorDbConfig {
                qdrant_url: "http://localhost:6333".to_string(),
                collection_name: "knowledge_base".to_string(),
                vector_size: 1536, // OpenAI embedding size
                similarity_threshold: 0.7,
                max_results: 10,
            },
            web_scraping: WebScrapingConfig {
                user_agent: "Agent-Agency-Research/1.0".to_string(),
                timeout_seconds: 30,
                max_content_size: 1024 * 1024, // 1MB
                rate_limit_per_minute: 60,
                allowed_domains: vec![],
                blocked_domains: vec![],
                search_engines: vec!["google".to_string(), "bing".to_string()],
            },
            content_processing: ContentProcessingConfig {
                enable_cleaning: true,
                enable_markdown: true,
                enable_text_extraction: true,
                max_content_length: 100_000,
                enable_summarization: true,
            },
            context_synthesis: ContextSynthesisConfig {
                max_context_size: 50_000,
                context_overlap_percent: 0.1,
                enable_semantic_chunking: true,
                chunk_size: 1000,
                enable_cross_references: true,
            },
            performance: PerformanceConfig {
                max_concurrent_requests: 10,
                request_timeout_ms: 30000, // 30 seconds
                cache_ttl_seconds: 3600,   // 1 hour
                enable_caching: true,
                enable_monitoring: true,
            },
        }
    }
}
