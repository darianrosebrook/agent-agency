//! Query orchestration and execution coordination

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::time::{timeout, Duration};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::research_types::*;
use anyhow::{Context, Result};

use super::core::KnowledgeSeeker;
use super::events::EventEmitter;
use super::knowledge_metrics::MetricsCollector;
use super::processing::ContentProcessorManager;
use super::scraping::ScrapingCoordinator;
use super::search::SearchCoordinator;
use super::synthesis::ContextSynthesizer;
use super::ResearchEvent;

/// Query orchestrator for coordinating research execution

#[derive(Debug)]
pub struct QueryOrchestrator {
    config: ResearchAgentConfig,
    search_coordinator: Arc<SearchCoordinator>,
    scraping_coordinator: Arc<ScrapingCoordinator>,
    context_synthesizer: Arc<ContextSynthesizer>,
    content_processor: Arc<ContentProcessorManager>,
    metrics_collector: Arc<MetricsCollector>,
    event_emitter: Arc<EventEmitter>,
}

impl QueryOrchestrator {
    /// Create a new query orchestrator
    pub async fn new(config: ResearchAgentConfig) -> Result<Self> {
        Ok(Self {
            config,
            search_coordinator: Arc::new(SearchCoordinator::new(Default::default()).await?),
            scraping_coordinator: Arc::new(ScrapingCoordinator::new(Default::default()).await?),
            context_synthesizer: Arc::new(ContextSynthesizer::new(Default::default()).await?),
            content_processor: Arc::new(ContentProcessorManager::new(Default::default()).await?),
            metrics_collector: Arc::new(MetricsCollector::new().await?),
            event_emitter: Arc::new(EventEmitter::new()),
        })
    }

    /// Execute a research query with full orchestration
    pub async fn execute_query(&self, query: ResearchQuery) -> Result<Vec<ResearchResult>> {
        let start_time = std::time::Instant::now();

        info!(
            "Executing research query: {} (type: {:?})",
            query.query, query.query_type
        );

        // Emit query started event
        self.event_emitter
            .emit(ResearchEvent::QueryStarted(query.id))
            .await;

        // Set timeout for the entire query
        let timeout_duration = Duration::from_secs(self.config.web_scraping.timeout_seconds as u64);
        let result = timeout(timeout_duration, self.execute_query_internal(query.clone())).await;

        let results = match result {
            Ok(Ok(results)) => {
                info!(
                    "Research query completed successfully: {} results",
                    results.len()
                );
                self.event_emitter
                    .emit(ResearchEvent::QueryCompleted(query.id, results.len()))
                    .await;
                results
            }
            Ok(Err(e)) => {
                error!("Research query failed: {}", e);
                self.event_emitter
                    .emit(ResearchEvent::QueryFailed(query.id, e.to_string()))
                    .await;
                return Err(e);
            }
            Err(_) => {
                let error_msg = format!(
                    "Query timed out after {} seconds",
                    timeout_duration.as_secs()
                );
                error!("{}", error_msg);
                self.event_emitter
                    .emit(ResearchEvent::QueryFailed(query.id, error_msg.clone()))
                    .await;
                return Err(anyhow::anyhow!(error_msg));
            }
        };

        // Record metrics
        let duration_ms = start_time.elapsed().as_millis() as u64;
        self.metrics_collector
            .record_query_execution(duration_ms, results.len() as u64, true)
            .await;

        Ok(results)
    }

    /// Internal query execution logic
    async fn execute_query_internal(&self, query: ResearchQuery) -> Result<Vec<ResearchResult>> {
        let mut all_results = Vec::new();

        // V2 Integration: Enhanced hybrid search combining vector and keyword search
        info!("Using V2-enhanced hybrid search for improved research");

        // Perform vector search first
        let vector_results = self.search_coordinator.vector_search(&query).await?;
        all_results.extend(vector_results);

        // V2 Integration: Add keyword-based search for hybrid approach
        if matches!(
            query.query_type,
            QueryType::Knowledge | QueryType::Code | QueryType::Documentation
        ) {
            let keyword_results = self.search_coordinator.keyword_search(&query).await?;
            all_results.extend(keyword_results);
        }

        // If web scraping is enabled and we have web sources, scrape additional content
        if self.config.web_scraping.enabled && self.should_scrape_web(&query) {
            let web_results = self
                .scraping_coordinator
                .scrape_web_sources(&query, &all_results)
                .await?;
            all_results.extend(web_results);
        }

        // Process and rank results
        let processed_results = self.process_and_rank_results(all_results, &query).await?;

        // Limit results if specified
        let max_results = query
            .max_results
            .unwrap_or(self.config.vector_search.max_results) as usize;
        let final_results = processed_results.into_iter().take(max_results).collect();

        Ok(final_results)
    }

    /// Synthesize context from research results
    pub async fn synthesize_context(
        &self,
        query_id: Uuid,
        results: Vec<ResearchResult>,
    ) -> Result<SynthesizedContext> {
        self.event_emitter
            .emit(ResearchEvent::ContextSynthesisStarted(query_id))
            .await;

        let context = self
            .context_synthesizer
            .synthesize(query_id, results)
            .await?;

        self.event_emitter
            .emit(ResearchEvent::ContextSynthesisCompleted(query_id))
            .await;

        Ok(context)
    }

    /// Determine if web scraping should be performed for this query
    fn should_scrape_web(&self, query: &ResearchQuery) -> bool {
        // Scrape for research queries or when we have few local results
        matches!(
            query.query_type,
            QueryType::Knowledge | QueryType::Technical
        ) || query.query.contains("web")
            || query.query.contains("online")
            || query.query.contains("current")
    }

    /// Process and rank research results
    async fn process_and_rank_results(
        &self,
        results: Vec<ResearchResult>,
        query: &ResearchQuery,
    ) -> Result<Vec<ResearchResult>> {
        let mut processed_results = Vec::new();

        for mut result in results {
            // Process content if needed
            if let Some(processed) = self
                .content_processor
                .process_content(&result.content)
                .await?
            {
                result.content = processed.processed_content;
                if let Some(summary) = processed.summary {
                    result.summary = Some(summary);
                }
            }

            // Calculate final relevance and confidence scores
            result.relevance_score = self.calculate_relevance_score(&result, query)?;
            result.confidence_score = self.calculate_confidence_score(&result, query)?;

            processed_results.push(result);
        }

        // Sort by relevance score (highest first)
        processed_results.sort_by(|a, b| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(processed_results)
    }

    /// Calculate relevance score for a result
    fn calculate_relevance_score(
        &self,
        result: &ResearchResult,
        query: &ResearchQuery,
    ) -> Result<f32> {
        // Simple relevance calculation based on content match
        let query_lower = query.query.to_lowercase();
        let content_lower = result.content.to_lowercase();
        let title_lower = result.title.to_lowercase();

        let mut score = 0.0;

        // Exact matches in title are highly relevant
        if title_lower.contains(&query_lower) {
            score += 0.8;
        }

        // Keyword matches in content
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();
        let mut word_matches = 0;

        for word in &query_words {
            if content_lower.contains(word) {
                word_matches += 1;
            }
        }

        if !query_words.is_empty() {
            let word_match_ratio = word_matches as f32 / query_words.len() as f32;
            score += word_match_ratio * 0.5;
        }

        // Boost score for recent content
        let days_old = (Utc::now() - result.extracted_at).num_days();
        if days_old < 30 {
            score += 0.1;
        }

        Ok(score.min(1.0))
    }

    /// Calculate confidence score for a result
    fn calculate_confidence_score(
        &self,
        result: &ResearchResult,
        query: &ResearchQuery,
    ) -> Result<f32> {
        let mut confidence: f32 = 0.5; // Base confidence

        // Higher confidence for structured content
        if result.content.contains("```") || result.content.contains("# ") {
            confidence += 0.2;
        }

        // Higher confidence for official sources
        if let Some(url) = &result.url {
            if url.contains("github.com")
                || url.contains("docs.rs")
                || url.contains("wikipedia.org")
            {
                confidence += 0.2;
            }
        }

        // Lower confidence for very short content
        if result.content.len() < 100 {
            confidence -= 0.2;
        }

        // Adjust based on result source credibility
        confidence += match &result.source {
            KnowledgeSource::WebPage(_) => 0.1,
            KnowledgeSource::InternalKnowledgeBase(_) => 0.2,
            KnowledgeSource::Documentation(_) => 0.15,
            KnowledgeSource::CodeRepository(_) => 0.15,
            KnowledgeSource::ApiDocumentation(_) => 0.15,
            KnowledgeSource::CommunityPost(_) => 0.1,
            KnowledgeSource::AcademicPaper(_) => 0.2,
        };

        Ok(confidence.min(1.0_f32).max(0.0_f32))
    }

    /// Update configuration
    pub async fn update_config(&self, _update: ConfigurationUpdate) -> Result<()> {
        // Configuration updates would be applied to individual components
        Ok(())
    }
}
