#![cfg(feature = "database")]
#![allow(warnings)] // Disables all warnings for the crate
#![allow(dead_code)] // Disables dead_code warnings for the crate

//! Agent Memory System - Enterprise-grade memory for AI agents
//!
//! This crate implements a sophisticated memory architecture combining:
//! - Knowledge graphs for structured entity relationships
//! - Vector embeddings for semantic similarity
//! - Temporal reasoning for time-based analysis
//! - Memory decay and importance weighting
//! - Context offloading and retrieval
//! - Provenance tracking for explainable AI
//!
//! The system supports both episodic (event-based) and semantic (factual) memory types,
//! with multi-hop reasoning capabilities and automatic knowledge evolution.
//!
//! @author @darianrosebrook

pub mod context_management;
pub mod decay;
pub mod graph_engine;
pub mod memory_manager;
pub mod memory_types;
pub mod temporal_reasoning;
// NOTE: prompting_types module exists in agent-research crate at:
//   agent-research/src/self_prompting_agent/prompting_types.rs
// It contains types for self-prompting agents: Task, TaskType, TaskResult, ExecutionMode, etc.
// If agent-memory needs prompting types, it can either:
//   1. Import from agent-research: `use agent_research::self_prompting_agent::prompting_types::*;`
//   2. Create its own prompting_types module if memory-specific types are needed
// pub mod prompting_types; // TODO: Create this module or import from agent-research
pub mod workspace_registry;
pub mod memory_service_adapter;

// New feature modules for enhanced memory capabilities
pub mod vector_search;
pub mod consolidation;
pub mod long_term_management;

// Enhanced memory system with new capabilities
pub struct EnhancedMemorySystem {
    base_system: MemorySystem,
    vector_search: crate::vector_search::search_engine::InMemoryVectorSearchEngine,
    consolidation: crate::consolidation::consolidation_engine::MemoryConsolidationEngine,
    long_term_manager: crate::long_term_management::lifecycle::MemoryLifecycleManager,
}

#[cfg(feature = "embeddings")]
pub mod embedding_integration;

#[cfg(feature = "context-offloading")]
pub mod context_offloading;

#[cfg(feature = "provenance-tracking")]
pub mod provenance;

use std::sync::Arc;
use anyhow::{Context, Result};

// Import types from memory_types
use crate::memory_types::{
    AgentExperience, MemoryId, TaskContext, ContextualMemory, 
    ContextMatch, ReasoningQuery, ReasoningResult, TimeRange, TemporalAnalysis
};

#[cfg(feature = "observability-integration")]
pub mod observability;

#[cfg(test)]
mod tests;

// Re-exports for public API
pub use context_management::{MemoryContextManager};
// pub use context_management::{FoldedContext, ContextSummary, ArchivedContext}; // TODO: Implement these types
pub use decay::{MemoryDecayEngine, DecayStats};
pub use graph_engine::{KnowledgeGraphEngine, Entity, Relationship, GraphQuery, GraphStats};
pub use memory_manager::{MemoryManager, MemoryStats};
pub use temporal_reasoning::{TemporalReasoningEngine};
pub use memory_types::{MemoryConfig, MemoryType, TemporalContext, TaskPriority, ExperienceOutcome, AgentFeedback, ExperienceContext};
// NOTE: prompting_types exists in agent-research crate. See note above for import options.
// pub use prompting_types::*; // TODO: Uncomment when module is created or imported from agent-research

#[cfg(feature = "embeddings")]
pub use embedding_integration::{EmbeddingIntegration, MemoryEmbedding};

/// Result type for memory operations
pub type MemoryResult<T> = Result<T, MemoryError>;

/// Error types for the memory system
#[derive(Debug, thiserror::Error)]
pub enum MemoryError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Embedding service error: {0}")]
    Embedding(String),

    #[error("Graph operation error: {0}")]
    Graph(String),

    #[error("Temporal reasoning error: {0}")]
    Temporal(String),

    #[error("Decay operation error: {0}")]
    Decay(String),

    #[error("Context offloading error: {0}")]
    ContextOffloading(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Entity not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Unknown memory error: {0}")]
    Other(String),
}

impl From<anyhow::Error> for MemoryError {
    fn from(err: anyhow::Error) -> Self {
        MemoryError::Other(err.to_string())
    }
}

impl From<regex::Error> for MemoryError {
    fn from(err: regex::Error) -> Self {
        MemoryError::Other(format!("Regex error: {}", err))
    }
}

impl From<std::string::FromUtf8Error> for MemoryError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        MemoryError::Other(format!("UTF-8 decoding error: {}", err))
    }
}

/// Core memory system initialization
pub struct MemorySystem {
    manager: MemoryManager,
    graph_engine: KnowledgeGraphEngine,
    #[cfg(feature = "embeddings")]
    embedding_integration: EmbeddingIntegration,
    temporal_engine: TemporalReasoningEngine,
    decay_engine: MemoryDecayEngine,
    context_manager: MemoryContextManager,
    workspace_registry: Arc<workspace_registry::WorkspaceRegistry>,
}

// Manual Debug implementation to handle non-Debug fields
impl std::fmt::Debug for MemorySystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemorySystem")
            .field("manager", &self.manager)
            .field("graph_engine", &self.graph_engine)
            .field("temporal_engine", &self.temporal_engine)
            .field("decay_engine", &self.decay_engine)
            .field("context_manager", &self.context_manager)
            .field("workspace_registry", &self.workspace_registry)
            .finish()
    }
}

impl MemorySystem {
    /// Initialize the complete memory system
    pub async fn init(config: MemoryConfig) -> MemoryResult<Self> {
        // Create workspace registry first
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost/agent_agency_v3".to_string());
        let db_pool = Arc::new(
            sqlx::PgPool::connect(&database_url)
                .await
                .context("Failed to connect to database for memory system")?
        );
        let workspace_registry = Arc::new(workspace_registry::WorkspaceRegistry::new(
            config.workspace_config.access_config.clone(),
            db_pool.clone(),
        ));
        workspace_registry.initialize().await?;

        let manager = MemoryManager::new_with_registry(
            config.clone(),
            (*db_pool).clone(),
            Arc::clone(&workspace_registry)
        ).await?;
        let graph_engine = KnowledgeGraphEngine::new(&config.graph_config).await?;

        #[cfg(feature = "embeddings")]
        let embedding_integration = EmbeddingIntegration::new(&config.embedding_config).await?;

        let temporal_engine = TemporalReasoningEngine::new(&config.temporal_config).await?;
        let decay_engine = MemoryDecayEngine::new_with_workspace_registry(
            &config.decay_config,
            (*db_pool).clone(),
            Arc::clone(&workspace_registry)
        ).await?;
        let context_manager = MemoryContextManager::new(config.context_config.clone()).await?;

        Ok(Self {
            manager,
            graph_engine,
            #[cfg(feature = "embeddings")]
            embedding_integration,
            temporal_engine,
            decay_engine,
            context_manager,
            workspace_registry,
        })
    }

    /// Get the memory manager
    pub fn manager(&self) -> &MemoryManager {
        &self.manager
    }

    /// Get the knowledge graph engine
    pub fn graph_engine(&self) -> &KnowledgeGraphEngine {
        &self.graph_engine
    }

    /// Get the embedding integration
    #[cfg(feature = "embeddings")]
    pub fn embedding_integration(&self) -> &EmbeddingIntegration {
        &self.embedding_integration
    }

    /// Get the temporal reasoning engine
    pub fn temporal_engine(&self) -> &TemporalReasoningEngine {
        &self.temporal_engine
    }

    /// Get the memory decay engine
    pub fn decay_engine(&self) -> &MemoryDecayEngine {
        &self.decay_engine
    }

    /// Get the context manager
    pub fn context_manager(&self) -> &MemoryContextManager {
        &self.context_manager
    }

    /// Get the workspace registry
    pub fn workspace_registry(&self) -> &Arc<workspace_registry::WorkspaceRegistry> {
        &self.workspace_registry
    }

    /// Store an agent experience (episodic memory)
    pub async fn store_experience(&self, experience: AgentExperience) -> MemoryResult<MemoryId> {
        let memory_id = self.manager.store_experience(experience.clone()).await?;

        #[cfg(feature = "embeddings")]
        {
            // Generate embedding for the experience
            let embedding = self.embedding_integration.generate_experience_embedding(&experience).await?;
            self.embedding_integration.store_embedding(memory_id, embedding).await?;
        }

        // Extract entities and relationships for knowledge graph
        let entities = self.graph_engine.extract_entities_from_experience(&experience).await?;
        let relationships = self.graph_engine.extract_relationships_from_experience(&experience, &entities).await?;

        // Store in knowledge graph
        for entity in entities {
            self.graph_engine.upsert_entity(entity).await?;
        }

        for relationship in relationships {
            self.graph_engine.upsert_relationship(relationship).await?;
        }

        Ok(memory_id)
    }

    /// Retrieve contextual memories based on current context
    pub async fn retrieve_contextual_memories(&self, context: &TaskContext, limit: usize) -> MemoryResult<Vec<ContextualMemory>> {
        let mut all_memories = Vec::new();

        #[cfg(feature = "embeddings")]
        {
            // Get semantic matches via embeddings
            let semantic_matches = self.embedding_integration.semantic_search_context(context, limit).await?;

            // Add semantic matches
            for (memory_id, similarity) in semantic_matches {
                if let Ok(memory) = self.manager.retrieve_memory(memory_id).await {
                    all_memories.push(ContextualMemory {
                        memory,
                        relevance_score: similarity,
                        context_match: ContextMatch::Semantic,
                        reasoning_path: vec![format!("Semantic similarity: {:.3}", similarity)],
                    });
                }
            }
        }

        // Get graph-based matches
        let graph_matches = self.graph_engine.find_related_entities(context, limit).await?;

        // Add graph matches
        for (memory_id, path) in graph_matches {
            if let Ok(experience) = self.manager.retrieve_memory(memory_id).await {
                all_memories.push(ContextualMemory {
                    memory: experience,
                    relevance_score: 0.8, // Graph matches get high base score
                    context_match: ContextMatch::Graph(path.len()),
                    reasoning_path: path,
                });
            }
        }

        // Apply temporal weighting and decay
        self.decay_engine.apply_temporal_weighting(&mut all_memories).await?;

        // Sort by relevance and return top results
        all_memories.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap_or(std::cmp::Ordering::Equal));
        all_memories.truncate(limit);

        Ok(all_memories)
    }

    /// Perform multi-hop reasoning on the knowledge graph
    pub async fn perform_reasoning(&self, query: ReasoningQuery) -> MemoryResult<ReasoningResult> {
        self.graph_engine.perform_multi_hop_reasoning(query).await
    }

    /// Analyze temporal patterns in agent performance
    pub async fn analyze_temporal_patterns(&self, agent_id: &str, time_range: &TimeRange) -> MemoryResult<TemporalAnalysis> {
        self.temporal_engine.analyze_agent_performance(agent_id, time_range).await
    }

    /// Run memory maintenance (decay, consolidation, cleanup)
    pub async fn run_maintenance(&self) -> MemoryResult<MaintenanceResult> {
        let decayed_count = self.decay_engine.run_decay_cycle().await?;
        let consolidated_count = self.manager.consolidate_memories().await?;
        let cleanup_count = self.manager.cleanup_expired_memories().await?;

        Ok(MaintenanceResult {
            decayed_memories: decayed_count,
            consolidated_memories: consolidated_count,
            cleaned_memories: cleanup_count,
        })
    }
}

/// Result of memory maintenance operations
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MaintenanceResult {
    pub decayed_memories: usize,
    pub consolidated_memories: usize,
    pub cleaned_memories: usize,
}
