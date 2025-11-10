//! Core knowledge seeker functionality and configuration

use schemars::JsonSchema;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use dashmap::DashMap;

use crate::research_types::*;
use crate::ContentProcessor;
use crate::ContextBuilder;
use crate::VectorSearchEngine;
use crate::WebScraper;
use crate::MultimodalContext;
use data_infrastructure::DatabaseClient;

use super::orchestration::QueryOrchestrator;
use super::search::SearchCoordinator;
use super::scraping::ScrapingCoordinator;
use super::synthesis::ContextSynthesizer;
use super::processing::ContentProcessorManager;
use super::database::DatabaseManager;
use super::knowledge_metrics::MetricsCollector;
use super::sessions::SessionManager;
use super::events::{EventEmitter, ResearchEvent};

/// Main knowledge seeker for research coordination

#[derive(Debug, Serialize, Deserialize) ]
pub struct KnowledgeSeeker {
    config: ResearchAgentConfig,

    // Core component coordinators
    orchestrator: Arc<QueryOrchestrator>,
    search_coordinator: Arc<SearchCoordinator>,
    scraping_coordinator: Arc<ScrapingCoordinator>,
    context_synthesizer: Arc<ContextSynthesizer>,
    content_processor: Arc<ContentProcessorManager>,
    database_manager: Arc<DatabaseManager>,
    metrics_collector: Arc<MetricsCollector>,
    session_manager: Arc<SessionManager>,
    event_emitter: Arc<EventEmitter>,

    // Status tracking
    status: Arc<RwLock<ResearchAgentStatus>>,
}

/// Research events for monitoring and debugging
#[derive(Debug, Clone, Serialize, Deserialize) ]
pub enum ResearchEvent {
    QueryStarted(Uuid),
    QueryCompleted(Uuid, usize), // query_id, result_count
    QueryFailed(Uuid, String),   // query_id, error_message
    ScrapingStarted(String),     // url
    ScrapingCompleted(String, usize), // url, content_length
    ScrapingFailed(String, String),   // url, error_message
    ContextSynthesisStarted(Uuid),    // query_id
    ContextSynthesisCompleted(Uuid),  // query_id
    SessionCreated(Uuid),       // session_id
    SessionCompleted(Uuid),     // session_id
    ErrorOccurred(String),      // error_message
    ConfigurationUpdated,
    ComponentHealthCheck(String, bool), // component_name, healthy
}

impl KnowledgeSeeker {
    /// Create a new knowledge seeker with database pool integration
    pub async fn new(config: ResearchAgentConfig, database_pool: Arc<DatabaseClient>) -> Result<Self> {
        info!("Initializing knowledge seeker with database pool integration");

        // Initialize event emitter
        let event_emitter = Arc::new(EventEmitter::new());

        // Initialize core components
        let orchestrator = Arc::new(QueryOrchestrator::new(config.clone()).await?);
        let search_coordinator = Arc::new(SearchCoordinator::new(config.clone()).await?);
        let scraping_coordinator = Arc::new(ScrapingCoordinator::new(config.clone()).await?);
        let context_synthesizer = Arc::new(ContextSynthesizer::new(config.clone()).await?);
        let content_processor = Arc::new(ContentProcessorManager::new(config.clone()).await?);
        let database_manager = Arc::new(DatabaseManager::new(database_pool).await?);
        let metrics_collector = Arc::new(MetricsCollector::new().await?);
        let session_manager = Arc::new(SessionManager::new().await?);

        let seeker = Self {
            config,
            orchestrator,
            search_coordinator,
            scraping_coordinator,
            context_synthesizer,
            content_processor,
            database_manager,
            metrics_collector,
            session_manager,
            event_emitter,
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

    /// Get current status
    pub async fn get_status(&self) -> ResearchAgentStatus {
        *self.status.read().await
    }

    /// Get capabilities
    pub async fn get_capabilities(&self) -> ResearchCapabilities {
        ResearchCapabilities {
            supported_query_types: vec![
                QueryType::Knowledge,
                QueryType::Code,
                QueryType::Documentation,
                QueryType::Research,
            ],
            max_results: self.config.vector_search.max_results,
            supports_web_scraping: self.config.web_scraping.enabled,
            supports_context_synthesis: true,
            supported_content_types: vec![
                ContentType::Text,
                ContentType::Html,
                ContentType::Markdown,
                ContentType::Code,
            ],
        }
    }

    /// Get metrics
    pub async fn get_metrics(&self) -> ResearchMetrics {
        self.metrics_collector.get_metrics().await
    }

    /// Update configuration
    pub async fn update_config(&mut self, update: ConfigurationUpdate) -> Result<()> {
        self.validate_configuration_update(&update)?;

        // Update local config
        self.apply_configuration_update(&update).await?;

        // Update component configs
        self.orchestrator.update_config(update.clone()).await?;
        self.search_coordinator.update_config(update.clone()).await?;
        self.scraping_coordinator.update_config(update.clone()).await?;
        self.context_synthesizer.update_config(update.clone()).await?;
        self.content_processor.update_config(update.clone()).await?;

        // Emit configuration update event
        self.event_emitter.emit(ResearchEvent::ConfigurationUpdated).await;

        info!("Configuration updated successfully");
        Ok(())
    }

    /// Validate configuration update
    fn validate_configuration_update(&self, update: &ConfigurationUpdate) -> Result<()> {
        match update {
            ConfigurationUpdate::VectorSearch(config) => {
                if config.dimension == 0 {
                    return Err(anyhow::anyhow!("Vector dimension must be greater than 0"));
                }
                if config.similarity_threshold < 0.0 || config.similarity_threshold > 1.0 {
                    return Err(anyhow::anyhow!("Similarity threshold must be between 0.0 and 1.0"));
                }
            }
            ConfigurationUpdate::WebScraping(config) => {
                if config.max_depth == 0 {
                    return Err(anyhow::anyhow!("Max depth must be greater than 0"));
                }
                if config.max_pages == 0 {
                    return Err(anyhow::anyhow!("Max pages must be greater than 0"));
                }
            }
            ConfigurationUpdate::ContextSynthesis(config) => {
                if config.max_context_length == 0 {
                    return Err(anyhow::anyhow!("Max context length must be greater than 0"));
                }
            }
        }
        Ok(())
    }

    /// Apply configuration update
    async fn apply_configuration_update(&mut self, update: &ConfigurationUpdate) -> Result<()> {
        match update {
            ConfigurationUpdate::VectorSearch(config) => {
                self.config.vector_search = config.clone();
            }
            ConfigurationUpdate::WebScraping(config) => {
                self.config.web_scraping = config.clone();
            }
            ConfigurationUpdate::ContextSynthesis(config) => {
                self.config.context_synthesis = config.clone();
            }
        }
        Ok(())
    }

    /// Get access to internal components for orchestration
    pub fn orchestrator(&self) -> Arc<QueryOrchestrator> {
        Arc::clone(&self.orchestrator)
    }

    pub fn search_coordinator(&self) -> Arc<SearchCoordinator> {
        Arc::clone(&self.search_coordinator)
    }

    pub fn scraping_coordinator(&self) -> Arc<ScrapingCoordinator> {
        Arc::clone(&self.scraping_coordinator)
    }

    pub fn context_synthesizer(&self) -> Arc<ContextSynthesizer> {
        Arc::clone(&self.context_synthesizer)
    }

    pub fn content_processor(&self) -> Arc<ContentProcessorManager> {
        Arc::clone(&self.content_processor)
    }

    pub fn database_manager(&self) -> Arc<DatabaseManager> {
        Arc::clone(&self.database_manager)
    }

    pub fn metrics_collector(&self) -> Arc<MetricsCollector> {
        Arc::clone(&self.metrics_collector)
    }

    pub fn session_manager(&self) -> Arc<SessionManager> {
        Arc::clone(&self.session_manager)
    }

    pub fn event_emitter(&self) -> Arc<EventEmitter> {
        Arc::clone(&self.event_emitter)
    }
}
