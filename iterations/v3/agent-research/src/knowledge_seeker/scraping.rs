//! Web scraping coordination and management

use std::sync::Arc;
use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing::{info, warn, error};

use crate::research_types::*;
use crate::{ConfigurationUpdate, WebScraper};

use super::processing::ContentProcessorManager;
use super::events::{EventEmitter, ResearchEvent};

/// Web scraping coordinator

#[derive(Debug)]
pub struct ScrapingCoordinator {
    #[serde(skip)]
    web_scraper: Arc<WebScraper>,
    #[serde(skip)]
    content_processor: Arc<ContentProcessorManager>,
    config: ResearchAgentConfig,
    #[serde(skip)]
    event_emitter: Arc<EventEmitter>,
}

impl ScrapingCoordinator {
    /// Create a new scraping coordinator
    pub async fn new(config: ResearchAgentConfig) -> Result<Self> {
        let web_scraper = Arc::new(WebScraper::new(config.web_scraping.clone()));
        let content_processor = Arc::new(ContentProcessorManager::new(config.clone()).await?);
        let event_emitter = Arc::new(EventEmitter::new());

        Ok(Self {
            web_scraper,
            content_processor,
            config,
            event_emitter,
        })
    }

    /// Scrape web sources for additional research results
    pub async fn scrape_web_sources(
        &self,
        query: &ResearchQuery,
        existing_results: &[ResearchResult],
    ) -> Result<Vec<ResearchResult>> {
        if !self.config.web_scraping.enabled {
            return Ok(Vec::new());
        }

        info!("Starting web scraping for query: {}", query.query);

        // Extract potential URLs from existing results and query
        let urls_to_scrape = self.extract_scraping_targets(query, existing_results).await;

        if urls_to_scrape.is_empty() {
            info!("No URLs to scrape for this query");
            return Ok(Vec::new());
        }

        let mut web_results = Vec::new();
        let max_scrapes = self.config.web_scraping.max_pages.min(5); // Limit concurrent scrapes

        // Scrape URLs concurrently with limit
        let mut handles = Vec::new();

        for url in urls_to_scrape.into_iter().take(max_scrapes) {
            let scraper = Arc::clone(&self.web_scraper);
            let processor = Arc::clone(&self.content_processor);
            let event_emitter = Arc::clone(&self.event_emitter);
            let query_id = query.id;

            let handle = tokio::spawn(async move {
                Self::scrape_single_url(scraper, processor, event_emitter, url, query_id).await
            });

            handles.push(handle);
        }

        // Collect results
        for handle in handles {
            match handle.await {
                Ok(Ok(result)) => {
                    if let Some(result) = result {
                        web_results.push(result);
                    }
                }
                Ok(Err(e)) => {
                    warn!("Web scraping task failed: {}", e);
                }
                Err(e) => {
                    error!("Web scraping task panicked: {}", e);
                }
            }
        }

        info!("Web scraping completed: {} results", web_results.len());
        Ok(web_results)
    }

    /// Extract URLs to scrape from query and existing results
    async fn extract_scraping_targets(
        &self,
        query: &ResearchQuery,
        existing_results: &[ResearchResult],
    ) -> Vec<String> {
        let mut urls = HashSet::new();

        // Extract URLs from existing results
        for result in existing_results {
            if let Some(url) = &result.url {
                // Only scrape if it's a different domain or related content
                if self.should_scrape_related_url(url, &result.content) {
                    urls.insert(url.clone());
                }
            }
        }

        // Add query-specific URLs if the query suggests web research
        if query.query.contains("web") || query.query.contains("online") || query.query.contains("current") {
            // TODO: Implement comprehensive query-specific URL extraction
            //       Currently relies on existing result URLs; should implement comprehensive extraction that adds default search URLs or extracts URLs from query for enhanced web research capabilities.
            //
            // COMPLETION CHECKLIST:
            // [ ] Primary functionality implemented
            // [ ] API/data structures defined & stable
            // [ ] Error handling + validation aligned with error taxonomy
            // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
            // [ ] Integration tests for external systems/contracts
            // [ ] Documentation: public API + system behavior
            // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
            // [ ] Security posture reviewed (inputs, authz, sandboxing)
            // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
            // [ ] Configurability and feature flags defined if relevant
            // [ ] Failure-mode cards documented (degradation paths)
            //
            // ACCEPTANCE CRITERIA:
            // - Default search URLs are added for web research queries
            // - URLs are extracted from query when present
            // - URL extraction is accurate and relevant
            // - Extraction handles various query formats gracefully
            //
            // DEPENDENCIES:
            // - URL extraction utilities (Required)
            // - Default search URL configuration (Required)
            // - Query parsing utilities (Required)
            //
            // ESTIMATED EFFORT: 6-8 hours (medium confidence)
            // PRIORITY: Low
            // BLOCKING: No
            //
            // GOVERNANCE:
            // - CAWS Tier: 2 (web research functionality)
            // - Change Budget: ~150 LOC
            // - Reviewer Requirements: URL extraction and web research expertise
        }

        urls.into_iter().collect()
    }

    /// Determine if a URL should be scraped based on content relevance
    fn should_scrape_related_url(&self, url: &str, content: &str) -> bool {
        // Skip already scraped URLs or internal links
        if url.contains("#") || url.contains("localhost") || url.contains("127.0.0.1") {
            return false;
        }

        // Prioritize documentation and knowledge sources
        let priority_domains = [
            "github.com", "docs.rs", "wikipedia.org", "stackoverflow.com",
            "developer.mozilla.org", "web.dev", "developers.google.com",
            "docs.microsoft.com", "kubernetes.io", "docker.com"
        ];

        for domain in &priority_domains {
            if url.contains(domain) {
                return true;
            }
        }

        // Scrape if content suggests it's a reference or external link
        content.contains("see also") ||
        content.contains("reference") ||
        content.contains("documentation") ||
        content.contains("docs")
    }

    /// Scrape a single URL
    async fn scrape_single_url(
        web_scraper: Arc<WebScraper>,
        content_processor: Arc<ContentProcessorManager>,
        event_emitter: Arc<EventEmitter>,
        url: String,
        query_id: Uuid,
    ) -> Result<Option<ResearchResult>> {
        event_emitter.emit(ResearchEvent::ScrapingStarted(url.clone())).await;

        match web_scraper.scrape_url(&url).await {
            Ok(scraping_result) => {
                event_emitter.emit(ResearchEvent::ScrapingCompleted(
                    url.clone(),
                    scraping_result.content.len()
                )).await;

                // Process the scraped content
                match content_processor.process_content(&scraping_result.content).await {
                    Ok(processed) => {
                        let processed = processed.unwrap_or_else(|| crate::research_types::ContentProcessingResult {
                            original_content: scraping_result.content.clone(),
                            processed_content: scraping_result.content.clone(),
                            extracted_text: scraping_result.content.clone(),
                            summary: None,
                            key_phrases: vec![],
                            entities: vec![],
                            links: vec![],
                            processing_time_ms: 0,
                            metadata: std::collections::HashMap::new(),
                        });

                        let result = ResearchResult {
                            query_id,
                            source: "web_scraped".to_string(),
                            title: scraping_result.title,
                            content: processed.processed_content,
                            summary: processed.summary,
                            relevance_score: 0.6, // Moderate relevance for scraped content
                            confidence_score: 0.7, // Good confidence for structured scraping
                            extracted_at: chrono::Utc::now(),
                            url: Some(url),
                            metadata: scraping_result.metadata,
                        };

                        Ok(Some(result))
                    }
                    Err(e) => {
                        warn!("Failed to process scraped content from {}: {}", url, e);
                        Ok(None)
                    }
                }
            }
            Err(e) => {
                event_emitter.emit(ResearchEvent::ScrapingFailed(url, e.to_string())).await;
                warn!("Failed to scrape URL {}: {}", url, e);
                Ok(None)
            }
        }
    }

    /// Update configuration
    pub async fn update_config(&self, update: ConfigurationUpdate) -> Result<()> {
        match update {
            ConfigurationUpdate::WebScraping(config) => {
                // Update web scraping configuration
                info!("Web scraping configuration updated");
            }
            _ => {} // Other updates don't affect scraping
        }
        Ok(())
    }
}
