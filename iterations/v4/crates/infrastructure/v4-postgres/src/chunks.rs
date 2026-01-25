//! File Chunking for Workspace Indexing
//!
//! Splits source files into chunks for embedding and retrieval.

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use sqlx::Row;

use crate::error::PostgresError;
use crate::pool::ConnectionPool;

/// Chunk repository for file content
pub struct ChunkRepository {
    pool: ConnectionPool,
}

impl ChunkRepository {
    /// Create a new chunk repository
    pub fn new(pool: ConnectionPool) -> Self {
        Self { pool }
    }

    /// Store a chunk
    pub async fn store_chunk(&self, chunk: &Chunk) -> Result<uuid::Uuid, PostgresError> {
        let content_hash = Self::hash_content(&chunk.content);
        let metadata = chunk
            .metadata
            .as_ref()
            .map(|m| serde_json::to_value(m).ok())
            .flatten();

        let row = sqlx::query(
            r#"
            INSERT INTO chunks (
                file_path, chunk_index, content, content_hash,
                start_line, end_line, language, embedding_id, metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (file_path, chunk_index) DO UPDATE SET
                content = EXCLUDED.content,
                content_hash = EXCLUDED.content_hash,
                start_line = EXCLUDED.start_line,
                end_line = EXCLUDED.end_line,
                language = EXCLUDED.language,
                embedding_id = EXCLUDED.embedding_id,
                metadata = EXCLUDED.metadata
            RETURNING id
            "#,
        )
        .bind(&chunk.file_path)
        .bind(chunk.chunk_index as i32)
        .bind(&chunk.content)
        .bind(&content_hash)
        .bind(chunk.start_line.map(|l| l as i32))
        .bind(chunk.end_line.map(|l| l as i32))
        .bind(&chunk.language)
        .bind(chunk.embedding_id)
        .bind(&metadata)
        .fetch_one(self.pool.pool())
        .await?;

        Ok(row.get("id"))
    }

    /// Store multiple chunks in a transaction
    pub async fn store_chunks(&self, chunks: &[Chunk]) -> Result<Vec<uuid::Uuid>, PostgresError> {
        let mut ids = Vec::with_capacity(chunks.len());

        for chunk in chunks {
            let id = self.store_chunk(chunk).await?;
            ids.push(id);
        }

        Ok(ids)
    }

    /// Get chunks for a file
    pub async fn get_chunks_for_file(&self, file_path: &str) -> Result<Vec<ChunkRecord>, PostgresError> {
        let rows = sqlx::query(
            r#"
            SELECT id, file_path, chunk_index, content, content_hash,
                   start_line, end_line, language, embedding_id, metadata, created_at
            FROM chunks
            WHERE file_path = $1
            ORDER BY chunk_index ASC
            "#,
        )
        .bind(file_path)
        .fetch_all(self.pool.pool())
        .await?;

        let mut chunks = Vec::with_capacity(rows.len());
        for row in rows {
            chunks.push(ChunkRecord {
                id: row.get("id"),
                file_path: row.get("file_path"),
                chunk_index: row.get::<i32, _>("chunk_index") as usize,
                content: row.get("content"),
                content_hash: row.get("content_hash"),
                start_line: row.get::<Option<i32>, _>("start_line").map(|l| l as usize),
                end_line: row.get::<Option<i32>, _>("end_line").map(|l| l as usize),
                language: row.get("language"),
                embedding_id: row.get("embedding_id"),
                metadata: row.get("metadata"),
                created_at: row.get("created_at"),
            });
        }

        Ok(chunks)
    }

    /// Get chunk by ID
    pub async fn get_chunk(&self, id: uuid::Uuid) -> Result<Option<ChunkRecord>, PostgresError> {
        let row = sqlx::query(
            r#"
            SELECT id, file_path, chunk_index, content, content_hash,
                   start_line, end_line, language, embedding_id, metadata, created_at
            FROM chunks
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(self.pool.pool())
        .await?;

        Ok(row.map(|row| ChunkRecord {
            id: row.get("id"),
            file_path: row.get("file_path"),
            chunk_index: row.get::<i32, _>("chunk_index") as usize,
            content: row.get("content"),
            content_hash: row.get("content_hash"),
            start_line: row.get::<Option<i32>, _>("start_line").map(|l| l as usize),
            end_line: row.get::<Option<i32>, _>("end_line").map(|l| l as usize),
            language: row.get("language"),
            embedding_id: row.get("embedding_id"),
            metadata: row.get("metadata"),
            created_at: row.get("created_at"),
        }))
    }

    /// Delete chunks for a file
    pub async fn delete_chunks_for_file(&self, file_path: &str) -> Result<u64, PostgresError> {
        let result = sqlx::query("DELETE FROM chunks WHERE file_path = $1")
            .bind(file_path)
            .execute(self.pool.pool())
            .await?;

        Ok(result.rows_affected())
    }

    /// Link chunk to embedding
    pub async fn link_embedding(
        &self,
        chunk_id: uuid::Uuid,
        embedding_id: uuid::Uuid,
    ) -> Result<(), PostgresError> {
        sqlx::query("UPDATE chunks SET embedding_id = $1 WHERE id = $2")
            .bind(embedding_id)
            .bind(chunk_id)
            .execute(self.pool.pool())
            .await?;

        Ok(())
    }

    /// Get chunks without embeddings
    pub async fn get_unembedded_chunks(&self, limit: i32) -> Result<Vec<ChunkRecord>, PostgresError> {
        let rows = sqlx::query(
            r#"
            SELECT id, file_path, chunk_index, content, content_hash,
                   start_line, end_line, language, embedding_id, metadata, created_at
            FROM chunks
            WHERE embedding_id IS NULL
            ORDER BY created_at ASC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(self.pool.pool())
        .await?;

        let mut chunks = Vec::with_capacity(rows.len());
        for row in rows {
            chunks.push(ChunkRecord {
                id: row.get("id"),
                file_path: row.get("file_path"),
                chunk_index: row.get::<i32, _>("chunk_index") as usize,
                content: row.get("content"),
                content_hash: row.get("content_hash"),
                start_line: row.get::<Option<i32>, _>("start_line").map(|l| l as usize),
                end_line: row.get::<Option<i32>, _>("end_line").map(|l| l as usize),
                language: row.get("language"),
                embedding_id: row.get("embedding_id"),
                metadata: row.get("metadata"),
                created_at: row.get("created_at"),
            });
        }

        Ok(chunks)
    }

    /// Count total chunks
    pub async fn count(&self) -> Result<i64, PostgresError> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM chunks")
            .fetch_one(self.pool.pool())
            .await?;

        Ok(row.get("count"))
    }

    /// Count chunks by language
    pub async fn count_by_language(&self, language: &str) -> Result<i64, PostgresError> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM chunks WHERE language = $1")
            .bind(language)
            .fetch_one(self.pool.pool())
            .await?;

        Ok(row.get("count"))
    }

    fn hash_content(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        hex::encode(hasher.finalize())
    }
}

/// A chunk of file content to be embedded
#[derive(Debug, Clone)]
pub struct Chunk {
    /// File path
    pub file_path: String,
    /// Chunk index within the file
    pub chunk_index: usize,
    /// Chunk content
    pub content: String,
    /// Starting line number (1-indexed)
    pub start_line: Option<usize>,
    /// Ending line number (1-indexed)
    pub end_line: Option<usize>,
    /// Programming language
    pub language: Option<String>,
    /// Associated embedding ID (if already embedded)
    pub embedding_id: Option<uuid::Uuid>,
    /// Additional metadata
    pub metadata: Option<serde_json::Value>,
}

impl Chunk {
    /// Create a new chunk
    pub fn new(file_path: impl Into<String>, chunk_index: usize, content: impl Into<String>) -> Self {
        Self {
            file_path: file_path.into(),
            chunk_index,
            content: content.into(),
            start_line: None,
            end_line: None,
            language: None,
            embedding_id: None,
            metadata: None,
        }
    }

    /// Set line range
    pub fn with_lines(mut self, start: usize, end: usize) -> Self {
        self.start_line = Some(start);
        self.end_line = Some(end);
        self
    }

    /// Set language
    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Stored chunk record
#[derive(Debug, Clone)]
pub struct ChunkRecord {
    /// Chunk ID
    pub id: uuid::Uuid,
    /// File path
    pub file_path: String,
    /// Chunk index
    pub chunk_index: usize,
    /// Content
    pub content: String,
    /// Content hash
    pub content_hash: String,
    /// Start line
    pub start_line: Option<usize>,
    /// End line
    pub end_line: Option<usize>,
    /// Language
    pub language: Option<String>,
    /// Embedding ID
    pub embedding_id: Option<uuid::Uuid>,
    /// Metadata
    pub metadata: Option<serde_json::Value>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

/// Utility to split content into chunks
pub struct Chunker {
    /// Maximum characters per chunk
    pub max_chars: usize,
    /// Overlap between chunks (in characters)
    pub overlap: usize,
}

impl Default for Chunker {
    fn default() -> Self {
        Self {
            max_chars: 1000,
            overlap: 100,
        }
    }
}

impl Chunker {
    /// Create a new chunker
    pub fn new(max_chars: usize, overlap: usize) -> Self {
        Self { max_chars, overlap }
    }

    /// Split content into chunks
    pub fn chunk_content(&self, file_path: &str, content: &str, language: Option<&str>) -> Vec<Chunk> {
        let lines: Vec<&str> = content.lines().collect();
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut chunk_start_line = 1;
        let mut current_line = 1;
        let mut chunk_index = 0;

        for line in &lines {
            let line_with_newline = format!("{}\n", line);

            if current_chunk.len() + line_with_newline.len() > self.max_chars && !current_chunk.is_empty() {
                // Save current chunk
                let mut chunk = Chunk::new(file_path, chunk_index, current_chunk.trim_end())
                    .with_lines(chunk_start_line, current_line - 1);

                if let Some(lang) = language {
                    chunk = chunk.with_language(lang);
                }

                chunks.push(chunk);
                chunk_index += 1;

                // Start new chunk with overlap
                let overlap_start = if current_line > 3 { current_line - 3 } else { chunk_start_line };
                current_chunk = lines[(overlap_start - 1)..(current_line - 1)]
                    .iter()
                    .map(|l| format!("{}\n", l))
                    .collect();
                chunk_start_line = overlap_start;
            }

            current_chunk.push_str(&line_with_newline);
            current_line += 1;
        }

        // Don't forget the last chunk
        if !current_chunk.is_empty() {
            let mut chunk = Chunk::new(file_path, chunk_index, current_chunk.trim_end())
                .with_lines(chunk_start_line, lines.len());

            if let Some(lang) = language {
                chunk = chunk.with_language(lang);
            }

            chunks.push(chunk);
        }

        chunks
    }

    /// Detect language from file extension
    pub fn detect_language(file_path: &str) -> Option<String> {
        let ext = file_path.rsplit('.').next()?;
        let lang = match ext.to_lowercase().as_str() {
            "rs" => "rust",
            "py" => "python",
            "js" => "javascript",
            "ts" => "typescript",
            "tsx" => "typescript",
            "jsx" => "javascript",
            "go" => "go",
            "java" => "java",
            "c" => "c",
            "cpp" | "cc" | "cxx" => "cpp",
            "h" | "hpp" => "c",
            "rb" => "ruby",
            "php" => "php",
            "swift" => "swift",
            "kt" => "kotlin",
            "scala" => "scala",
            "sql" => "sql",
            "md" => "markdown",
            "json" => "json",
            "yaml" | "yml" => "yaml",
            "toml" => "toml",
            "html" => "html",
            "css" => "css",
            "scss" | "sass" => "scss",
            "sh" | "bash" => "bash",
            _ => return None,
        };
        Some(lang.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_creation() {
        let chunk = Chunk::new("src/main.rs", 0, "fn main() {}")
            .with_lines(1, 3)
            .with_language("rust");

        assert_eq!(chunk.file_path, "src/main.rs");
        assert_eq!(chunk.chunk_index, 0);
        assert_eq!(chunk.start_line, Some(1));
        assert_eq!(chunk.language, Some("rust".to_string()));
    }

    #[test]
    fn test_chunker_simple() {
        let chunker = Chunker::new(50, 10);
        let content = "line 1\nline 2\nline 3\nline 4\nline 5";
        let chunks = chunker.chunk_content("test.rs", content, Some("rust"));

        assert!(!chunks.is_empty());
        assert_eq!(chunks[0].language, Some("rust".to_string()));
    }

    #[test]
    fn test_chunker_large_content() {
        let chunker = Chunker::new(100, 20);
        let content = (0..50).map(|i| format!("line {}", i)).collect::<Vec<_>>().join("\n");
        let chunks = chunker.chunk_content("test.py", &content, Some("python"));

        assert!(chunks.len() > 1);
        for (i, chunk) in chunks.iter().enumerate() {
            assert_eq!(chunk.chunk_index, i);
        }
    }

    #[test]
    fn test_detect_language() {
        assert_eq!(Chunker::detect_language("main.rs"), Some("rust".to_string()));
        assert_eq!(Chunker::detect_language("app.py"), Some("python".to_string()));
        assert_eq!(Chunker::detect_language("index.tsx"), Some("typescript".to_string()));
        assert_eq!(Chunker::detect_language("unknown.xyz"), None);
    }
}
