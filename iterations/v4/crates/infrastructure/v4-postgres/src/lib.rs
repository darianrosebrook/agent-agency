//! V4 PostgreSQL Adapter
//!
//! PostgreSQL storage with pgvector for embeddings and similarity search.
//!
//! ## Overview
//!
//! This crate provides:
//! - **Connection Pool**: Managed PostgreSQL connections with health checks
//! - **Event Sourcing**: Append-only event storage with content hashing
//! - **Vector Embeddings**: pgvector integration for similarity search
//! - **Workspace Chunking**: File chunking for embedding and retrieval
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use v4_postgres::{PostgresConfig, ConnectionPool, PostgresRepository};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create connection pool
//!     let config = PostgresConfig::local_dev();
//!     let pool = ConnectionPool::new(config).await?;
//!
//!     // Run migrations (creates tables + pgvector extension)
//!     pool.run_migrations().await?;
//!
//!     // Create repository
//!     let repo = PostgresRepository::new(pool.clone());
//!
//!     // Use the DatabasePort trait
//!     use v4_types::ports::DatabasePort;
//!     // repo.store_event(&event).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Vector Search
//!
//! ```rust,ignore
//! use v4_postgres::{EmbeddingRepository, ChunkRepository, Chunker};
//!
//! // Chunk a file
//! let chunker = Chunker::default();
//! let chunks = chunker.chunk_content("src/main.rs", &content, Some("rust"));
//!
//! // Store chunks
//! let chunk_repo = ChunkRepository::new(pool.clone());
//! for chunk in &chunks {
//!     chunk_repo.store_chunk(chunk).await?;
//! }
//!
//! // Store embeddings (after calling embedding model)
//! let embedding_repo = EmbeddingRepository::new(pool.clone());
//! embedding_repo.store_embedding(
//!     "chunk",
//!     &chunk_id.to_string(),
//!     &content,
//!     &embedding_vector,
//!     "text-embedding-3-small"
//! ).await?;
//!
//! // Find similar content
//! let results = embedding_repo.find_similar(&query_embedding, 10, 0.7).await?;
//! ```
//!
//! ## Database Schema
//!
//! ```text
//! ┌──────────────────┐     ┌──────────────────┐
//! │     events       │     │      tasks       │
//! ├──────────────────┤     ├──────────────────┤
//! │ id               │     │ id               │
//! │ sequence_number  │     │ title            │
//! │ event_type       │     │ description      │
//! │ aggregate_type   │     │ status           │
//! │ aggregate_id     │     │ council_verdict  │
//! │ payload (JSONB)  │     │ execution_time   │
//! │ content_hash     │     └──────────────────┘
//! └──────────────────┘
//!
//! ┌──────────────────┐     ┌──────────────────┐
//! │   embeddings     │────▶│     chunks       │
//! ├──────────────────┤     ├──────────────────┤
//! │ id               │     │ id               │
//! │ source_type      │     │ file_path        │
//! │ source_id        │     │ chunk_index      │
//! │ embedding (vec)  │     │ content          │
//! │ model            │     │ start_line       │
//! │ content_hash     │     │ end_line         │
//! └──────────────────┘     │ embedding_id ────┤
//!                          └──────────────────┘
//! ```
//!
//! ## Environment Variables
//!
//! | Variable | Default | Description |
//! |----------|---------|-------------|
//! | POSTGRES_HOST | localhost | Database host |
//! | POSTGRES_PORT | 5432 | Database port |
//! | POSTGRES_DB | agent_agency_v4 | Database name |
//! | POSTGRES_USER | postgres | Username |
//! | POSTGRES_PASSWORD | | Password |
//! | POSTGRES_SSL_MODE | prefer | SSL mode (disable/prefer/require) |
//! | POSTGRES_MAX_CONNECTIONS | 10 | Max pool connections |

pub mod chunks;
pub mod config;
pub mod embeddings;
pub mod error;
pub mod pool;
pub mod repository;

// Re-export main types
pub use chunks::{Chunk, ChunkRecord, ChunkRepository, Chunker};
pub use config::{ConfigError, PostgresConfig, SslMode};
pub use embeddings::{EmbeddingRecord, EmbeddingRepository, SimilarityResult};
pub use error::PostgresError;
pub use pool::{ConnectionPool, PoolStats};
pub use repository::{PostgresRepository, StoredEvent, TaskRecord};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exports() {
        // Verify main types are exported
        let _config = PostgresConfig::default();
        let _chunker = Chunker::default();
    }

    #[test]
    fn test_config_local_dev() {
        let config = PostgresConfig::local_dev();
        assert_eq!(config.database, "agent_agency_v4_dev");
        assert_eq!(config.ssl_mode, SslMode::Disable);
    }

    #[test]
    fn test_chunker_integration() {
        let chunker = Chunker::new(500, 50);
        let content = r#"
fn main() {
    println!("Hello, world!");
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#;

        let chunks = chunker.chunk_content("src/main.rs", content, Some("rust"));

        assert!(!chunks.is_empty());
        assert_eq!(chunks[0].file_path, "src/main.rs");
        assert_eq!(chunks[0].language, Some("rust".to_string()));
    }
}
