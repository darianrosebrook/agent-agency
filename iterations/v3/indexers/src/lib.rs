//! Multimodal indexers for V3 RAG system
//!
//! Provides:
//! - BM25 full-text search indexing
//! - HNSW approximate nearest neighbor search
//! - Database persistence with connection pooling
//! - Job scheduler with concurrency governance

pub mod bm25_indexer;
pub mod database;
pub mod hnsw_indexer;
pub mod job_scheduler;
pub mod types;

pub use bm25_indexer::Bm25Indexer;
pub use database::{DatabasePool, VectorStore};
pub use hnsw_indexer::HnswIndexer;
pub use job_scheduler::{IngestionJob, JobScheduler, JobType};
pub use types::*;
