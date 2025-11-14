//! Modular knowledge seeker components
//!
//! This module provides decomposed components for the knowledge seeker,
//! organized by responsibility for better maintainability and separation of concerns.

pub mod core;
pub mod database;
pub mod events;
pub mod index;
pub mod knowledge_metrics; // Renamed from metrics to avoid conflict
pub mod orchestration;
pub mod processing;
pub mod scraping;
pub mod search;
pub mod sessions;
pub mod synthesis;

// Re-export the main KnowledgeSeeker from core
pub use core::KnowledgeSeeker;

// Re-export key types for backward compatibility
pub use core::ResearchEvent;
pub use database::DatabaseManager;
pub use events::EventEmitter;
pub use index::{InvertedIndex, Posting, SearchResult};
pub use knowledge_metrics::MetricsCollector;
pub use orchestration::QueryOrchestrator;
pub use processing::ContentProcessorManager;
pub use scraping::ScrapingCoordinator;
pub use search::SearchCoordinator;
pub use sessions::SessionManager;
pub use synthesis::ContextSynthesizer;
