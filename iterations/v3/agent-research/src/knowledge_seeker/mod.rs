//! Modular knowledge seeker components
//!
//! This module provides decomposed components for the knowledge seeker,
//! organized by responsibility for better maintainability and separation of concerns.

pub mod core;
pub mod orchestration;
pub mod search;
pub mod scraping;
pub mod synthesis;
pub mod processing;
pub mod database;
pub mod knowledge_metrics; // Renamed from metrics to avoid conflict
pub mod sessions;
pub mod events;
pub mod index;

// Re-export the main KnowledgeSeeker from core
pub use core::KnowledgeSeeker;

// Re-export key types for backward compatibility
pub use core::ResearchEvent;
pub use index::{InvertedIndex, Posting, SearchResult};
pub use orchestration::QueryOrchestrator;
pub use search::SearchCoordinator;
pub use scraping::ScrapingCoordinator;
pub use synthesis::ContextSynthesizer;
pub use processing::ContentProcessorManager;
pub use database::DatabaseManager;
pub use knowledge_metrics::MetricsCollector;
pub use sessions::SessionManager;
pub use events::EventEmitter;
