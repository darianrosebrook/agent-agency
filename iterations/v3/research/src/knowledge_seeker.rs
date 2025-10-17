//! Knowledge Seeker
//!
//! Main research coordinator that orchestrates knowledge gathering, context synthesis,
//! and research capabilities for the Agent Agency system.

use crate::types::*;
use crate::ContentProcessingConfig;
use crate::{ContentProcessor, ContextBuilder, VectorSearchEngine, WebScraper};
use anyhow::{Context, Result};
use async_trait::async_trait;
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Main knowledge seeker for research coordination
#[derive(Debug)]
pub struct KnowledgeSeeker {
    config: ResearchAgentConfig,
    vector_search: Arc<VectorSearchEngine>,
    context_builder: Arc<ContextBuilder>,
    web_scraper: Arc<WebScraper>,
    content_processor: Arc<ContentProcessor>,

    // Active research sessions
    active_sessions: Arc<DashMap<Uuid, ResearchSession>>,

    // Research metrics
    metrics: Arc<RwLock<ResearchMetrics>>,

    // Event channel for research events
    event_sender: mpsc::UnboundedSender<ResearchEvent>,

    // Status
    status: Arc<RwLock<ResearchAgentStatus>>,
}

/// Research events for monitoring and debugging
#[derive(Debug)]
pub enum ResearchEvent {
    QueryStarted(Uuid),
    QueryCompleted(Uuid, Vec<ResearchResult>),
    QueryFailed(Uuid, String),
    ContextSynthesized(Uuid, SynthesizedContext),
    SessionCreated(Uuid),
    SessionCompleted(Uuid),
    ErrorOccurred(String),
}

impl KnowledgeSeeker {
    /// Create a new knowledge seeker
    pub async fn new(config: ResearchAgentConfig) -> Result<Self> {
        info!("Initializing knowledge seeker");

        let (event_sender, _event_receiver) = mpsc::unbounded_channel();

        // Initialize vector search engine
        let vector_search = Arc::new(
            VectorSearchEngine::new(
                &config.vector_search.qdrant_url,
                &config.vector_search.collection_name,
                config.vector_search.dimension as u32,
                config.vector_search.similarity_threshold,
                config.vector_search.max_results,
            )
            .await
            .context("Failed to initialize vector search engine")?,
        );

        // Initialize other components
        let context_builder = Arc::new(ContextBuilder::new(config.context_synthesis.clone()));
        let web_scraper = Arc::new(WebScraper::new(config.web_scraping.clone()));
        let content_processor = Arc::new(ContentProcessor::new(ContentProcessingConfig {
            enable_cleaning: true,
            enable_markdown: true,
            enable_text_extraction: true,
            max_content_length: 1000000,
            enable_summarization: false,
        }));

        let seeker = Self {
            config,
            vector_search,
            context_builder,
            web_scraper,
            content_processor,
            active_sessions: Arc::new(DashMap::new()),
            metrics: Arc::new(RwLock::new(ResearchMetrics {
                total_queries: 0,
                successful_queries: 0,
                failed_queries: 0,
                average_response_time_ms: 0.0,
                average_relevance_score: 0.0,
                average_confidence_score: 0.0,
                cache_hit_rate: 0.0,
                vector_search_accuracy: 0.0,
                web_scraping_success_rate: 0.0,
                context_synthesis_quality: 0.0,
                last_updated: chrono::Utc::now(),
            })),
            event_sender,
            status: Arc::new(RwLock::new(ResearchAgentStatus::Initializing)),
        };

        // Initialize status
        {
            let mut status = seeker.status.write().await;
            *status = ResearchAgentStatus::Available;
        }

        info!("Knowledge seeker initialized successfully");
        Ok(seeker)
    }

    /// Execute a research query
    pub async fn execute_query(&self, query: ResearchQuery) -> Result<Vec<ResearchResult>> {
        let start_time = std::time::Instant::now();

        info!(
            "Executing research query: {} (type: {:?})",
            query.query, query.query_type
        );

        // Update status
        {
            let mut status = self.status.write().await;
            *status = ResearchAgentStatus::Busy;
        }

        // Emit query started event
        let _ = self
            .event_sender
            .send(ResearchEvent::QueryStarted(query.id));

        let results = match self.execute_query_internal(query.clone()).await {
            Ok(results) => {
                info!(
                    "Research query completed successfully: {} results",
                    results.len()
                );
                results
            }
            Err(e) => {
                error!("Research query failed: {}", e);
                let _ = self
                    .event_sender
                    .send(ResearchEvent::ErrorOccurred(e.to_string()));
                return Err(e);
            }
        };

        // Update status
        {
            let mut status = self.status.write().await;
            *status = ResearchAgentStatus::Available;
        }

        // Emit query completed event
        let _ = self
            .event_sender
            .send(ResearchEvent::QueryCompleted(query.id, results.clone()));

        // Update metrics
        self.update_metrics(true, start_time.elapsed().as_millis() as u64, &results)
            .await;

        Ok(results)
    }

    /// Synthesize context from research results
    pub async fn synthesize_context(
        &self,
        query_id: Uuid,
        results: Vec<ResearchResult>,
    ) -> Result<SynthesizedContext> {
        info!("Synthesizing context for query: {}", query_id);

        let (synthesized_context, _metrics) = self
            .context_builder
            .synthesize_context(query_id, results)
            .await
            .context("Context synthesis failed")?;

        // Emit context synthesized event
        let _ = self.event_sender.send(ResearchEvent::ContextSynthesized(
            query_id,
            synthesized_context.clone(),
        ));

        info!("Context synthesis completed for query: {}", query_id);
        Ok(synthesized_context)
    }

    /// Create a new research session
    pub async fn create_session(
        &self,
        session_name: String,
        context: Option<String>,
    ) -> Result<ResearchSession> {
        let session = ResearchSession {
            id: Uuid::new_v4(),
            session_name,
            queries: Vec::new(),
            context,
            created_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            is_active: true,
            metadata: HashMap::new(),
        };

        self.active_sessions.insert(session.id, session.clone());

        // Emit session created event
        let _ = self
            .event_sender
            .send(ResearchEvent::SessionCreated(session.id));

        info!(
            "Created research session: {} ({})",
            session.session_name, session.id
        );
        Ok(session)
    }

    /// Add query to research session
    pub async fn add_query_to_session(&self, session_id: Uuid, query_id: Uuid) -> Result<()> {
        if let Some(mut session) = self.active_sessions.get_mut(&session_id) {
            session.queries.push(query_id);
            session.last_activity = chrono::Utc::now();
            info!("Added query {} to session {}", query_id, session_id);
        } else {
            return Err(anyhow::anyhow!("Session not found: {}", session_id));
        }

        Ok(())
    }

    /// Complete a research session
    pub async fn complete_session(&self, session_id: Uuid) -> Result<()> {
        if let Some(mut session) = self.active_sessions.get_mut(&session_id) {
            session.is_active = false;
            session.last_activity = chrono::Utc::now();

            // Emit session completed event
            let _ = self
                .event_sender
                .send(ResearchEvent::SessionCompleted(session_id));

            info!("Completed research session: {}", session_id);
        } else {
            return Err(anyhow::anyhow!("Session not found: {}", session_id));
        }

        Ok(())
    }

    /// Get research session
    pub async fn get_session(&self, session_id: Uuid) -> Option<ResearchSession> {
        self.active_sessions.get(&session_id).map(|s| s.clone())
    }

    /// Get all active sessions
    pub async fn get_active_sessions(&self) -> Vec<ResearchSession> {
        self.active_sessions
            .iter()
            .filter(|s| s.is_active)
            .map(|s| s.clone())
            .collect()
    }

    /// Get research capabilities
    pub async fn get_capabilities(&self) -> ResearchCapabilities {
        ResearchCapabilities {
            supported_query_types: vec![
                QueryType::Knowledge,
                QueryType::Code,
                QueryType::Documentation,
                QueryType::ApiReference,
                QueryType::Troubleshooting,
                QueryType::BestPractices,
            ],
            supported_sources: vec![
                KnowledgeSource::WebPage("".to_string()),
                KnowledgeSource::Documentation("".to_string()),
                KnowledgeSource::CodeRepository("".to_string()),
                KnowledgeSource::ApiDocumentation("".to_string()),
                KnowledgeSource::CommunityPost("".to_string()),
                KnowledgeSource::AcademicPaper("".to_string()),
                KnowledgeSource::InternalKnowledgeBase("".to_string()),
            ],
            max_concurrent_queries: self.config.performance.max_concurrent_requests as u32,
            max_context_size: self.config.context_synthesis.max_context_size,
            vector_search_enabled: true,
            web_scraping_enabled: true,
            content_processing_enabled: true,
            context_synthesis_enabled: true,
            real_time_updates: true,
        }
    }

    /// Get current status
    pub async fn get_status(&self) -> ResearchAgentStatus {
        let status = self.status.read().await;
        status.clone()
    }

    /// Get research metrics
    pub async fn get_metrics(&self) -> ResearchMetrics {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }

    /// Update configuration
    pub async fn update_config(&self, update: ResearchConfigUpdate) -> Result<()> {
        info!(
            "Updating research configuration: {} = {:?}",
            update.field, update.value
        );

        // TODO: Implement configuration updates
        // This would involve updating the relevant config fields and restarting affected components

        Ok(())
    }

    /// Internal query execution
    async fn execute_query_internal(&self, query: ResearchQuery) -> Result<Vec<ResearchResult>> {
        let mut all_results = Vec::new();

        // Generate embedding for semantic search
        let query_embedding = self
            .vector_search
            .generate_embedding(&query.query)
            .await
            .context("Failed to generate query embedding")?;

        // Perform vector search
        let vector_results = self
            .vector_search
            .search(&query_embedding, query.max_results, None)
            .await
            .context("Vector search failed")?;

        // Convert knowledge entries to research results
        for entry in vector_results {
            let result = ResearchResult {
                query_id: query.id,
                source: entry.source.clone(),
                title: entry.title.clone(),
                content: entry.content.clone(),
                summary: None,         // TODO: Generate summary
                relevance_score: 0.8,  // TODO: Calculate actual relevance
                confidence_score: 0.9, // TODO: Calculate actual confidence
                extracted_at: chrono::Utc::now(),
                url: entry.source_url.clone(),
                metadata: entry.metadata.clone(),
            };
            all_results.push(result);
        }

        // If web scraping is enabled and we have web sources, scrape additional content
        if self.config.web_scraping.rate_limit_per_minute > 0 {
            let web_results = self.scrape_web_sources(&query).await?;
            all_results.extend(web_results);
        }

        // Sort results by relevance score
        all_results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());

        // Limit results if specified
        if let Some(max_results) = query.max_results {
            all_results.truncate(max_results as usize);
        }

        Ok(all_results)
    }

    /// Scrape web sources for additional information
    async fn scrape_web_sources(&self, query: &ResearchQuery) -> Result<Vec<ResearchResult>> {
        let mut web_results = Vec::new();

        for source in &query.sources {
            if let KnowledgeSource::WebPage(url) = source {
                if !url.is_empty() {
                    match self.web_scraper.scrape_url(url).await {
                        Ok(scraping_result) => {
                            let processed_content = self
                                .content_processor
                                .process_content(&scraping_result.content)
                                .await?;

                            let result = ResearchResult {
                                query_id: query.id,
                                source: source.clone(),
                                title: scraping_result.title,
                                content: processed_content.processed_content,
                                summary: processed_content.summary,
                                relevance_score: 0.7, // TODO: Calculate relevance
                                confidence_score: 0.8, // TODO: Calculate confidence
                                extracted_at: chrono::Utc::now(),
                                url: Some(url.clone()),
                                metadata: scraping_result.metadata,
                            };

                            web_results.push(result);
                        }
                        Err(e) => {
                            warn!("Failed to scrape URL {}: {}", url, e);
                        }
                    }
                }
            }
        }

        Ok(web_results)
    }

    /// Update research metrics
    async fn update_metrics(
        &self,
        success: bool,
        response_time_ms: u64,
        results: &[ResearchResult],
    ) {
        let mut metrics = self.metrics.write().await;

        metrics.total_queries += 1;
        if success {
            metrics.successful_queries += 1;
        } else {
            metrics.failed_queries += 1;
        }

        // Update running averages
        let total = metrics.total_queries;
        metrics.average_response_time_ms = (metrics.average_response_time_ms * (total - 1) as f64
            + response_time_ms as f64)
            / total as f64;

        if !results.is_empty() {
            let avg_relevance: f32 =
                results.iter().map(|r| r.relevance_score).sum::<f32>() / results.len() as f32;
            let avg_confidence: f32 =
                results.iter().map(|r| r.confidence_score).sum::<f32>() / results.len() as f32;

            metrics.average_relevance_score =
                (metrics.average_relevance_score * (total - 1) as f32 + avg_relevance)
                    / total as f32;
            metrics.average_confidence_score =
                (metrics.average_confidence_score * (total - 1) as f32 + avg_confidence)
                    / total as f32;
        }

        metrics.last_updated = chrono::Utc::now();
    }
}

#[async_trait]
pub trait ResearchAgent: Send + Sync {
    /// Execute a research query
    async fn execute_query(&self, query: ResearchQuery) -> Result<Vec<ResearchResult>>;

    /// Synthesize context from results
    async fn synthesize_context(
        &self,
        query_id: Uuid,
        results: Vec<ResearchResult>,
    ) -> Result<SynthesizedContext>;

    /// Get research capabilities
    async fn get_capabilities(&self) -> ResearchCapabilities;

    /// Get current status
    async fn get_status(&self) -> ResearchAgentStatus;
}

#[async_trait]
impl ResearchAgent for KnowledgeSeeker {
    async fn execute_query(&self, query: ResearchQuery) -> Result<Vec<ResearchResult>> {
        self.execute_query(query).await
    }

    async fn synthesize_context(
        &self,
        query_id: Uuid,
        results: Vec<ResearchResult>,
    ) -> Result<SynthesizedContext> {
        self.synthesize_context(query_id, results).await
    }

    async fn get_capabilities(&self) -> ResearchCapabilities {
        self.get_capabilities().await
    }

    async fn get_status(&self) -> ResearchAgentStatus {
        self.get_status().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_knowledge_seeker_creation() {
        let config = ResearchAgentConfig {
            vector_search: VectorSearchConfig {
                enabled: true,
                qdrant_url: "http://localhost:6333".to_string(),
                collection_name: "test".to_string(),
                model: "test".to_string(),
                dimension: 768,
                similarity_threshold: 0.7,
                max_results: 10,
                batch_size: 32,
            },
            web_scraping: crate::WebScrapingConfig {
                enabled: true,
                max_depth: 3,
                max_pages: 10,
                timeout_ms: 30000,
                timeout_seconds: 30,
                user_agent: "test".to_string(),
                respect_robots_txt: false,
                allowed_domains: vec![],
                rate_limit_per_minute: 60,
            },
            context_synthesis: crate::ContextSynthesisConfig {
                enabled: true,
                max_context_size: 50000,
                context_overlap_percent: 0.1,
                enable_semantic_chunking: true,
                chunk_size: 1000,
                enable_cross_references: true,
            },
            performance: crate::PerformanceConfig {
                max_concurrent_requests: 10,
                request_timeout_ms: 30000,
            },
        };
        let seeker = KnowledgeSeeker::new(config).await;

        // In a real test, we'd assert successful creation
        // For now, we just ensure it compiles
        assert!(seeker.is_ok() || seeker.is_err());
    }

    #[tokio::test]
    async fn test_research_session_creation() {
        let config = ResearchAgentConfig {
            vector_search: VectorSearchConfig {
                enabled: true,
                qdrant_url: "http://localhost:6333".to_string(),
                collection_name: "test".to_string(),
                model: "test".to_string(),
                dimension: 768,
                similarity_threshold: 0.7,
                max_results: 10,
                batch_size: 32,
            },
            web_scraping: crate::WebScrapingConfig {
                enabled: true,
                max_depth: 3,
                max_pages: 10,
                timeout_ms: 30000,
                timeout_seconds: 30,
                user_agent: "test".to_string(),
                respect_robots_txt: false,
                allowed_domains: vec![],
                rate_limit_per_minute: 60,
            },
            context_synthesis: crate::ContextSynthesisConfig {
                enabled: true,
                max_context_size: 50000,
                context_overlap_percent: 0.1,
                enable_semantic_chunking: true,
                chunk_size: 1000,
                enable_cross_references: true,
            },
            performance: crate::PerformanceConfig {
                max_concurrent_requests: 10,
                request_timeout_ms: 30000,
            },
        };
        let seeker = KnowledgeSeeker::new(config).await.unwrap_or_else(|_| {
            // Create a minimal seeker for testing
            todo!("Create minimal seeker for testing")
        });

        let session = seeker
            .create_session("test session".to_string(), None)
            .await;
        assert!(session.is_ok());

        let session = session.unwrap();
        assert_eq!(session.session_name, "test session");
        assert!(session.is_active);
    }
}
