//! Knowledge Seeker - Modular Research Coordinator
//!
//! Main research coordinator that orchestrates knowledge gathering, context synthesis,
//! and research capabilities for the Agent Agency system.
//!
//! This module has been decomposed into focused sub-modules for better maintainability:
//! - core: Main KnowledgeSeeker struct and configuration
//! - orchestration: Query execution coordination
//! - search: Vector and keyword search
//! - scraping: Web scraping management
//! - synthesis: Context synthesis and summarization
//! - processing: Content processing
//! - database: Database integration
//! - metrics: Metrics collection
//! - sessions: Session management
//! - events: Event emission
//! - index: Inverted index for search

pub mod knowledge_seeker;

// Re-export the main components for backward compatibility
pub use knowledge_seeker::{KnowledgeSeeker, ResearchEvent, InvertedIndex, Posting, SearchResult};

// Re-export ResearchAgent trait implementation
pub use knowledge_seeker::core::KnowledgeSeeker as ModularKnowledgeSeeker;
