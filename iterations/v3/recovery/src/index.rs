//! SQLite index implementation for recovery system
//!
//! @author @darianrosebrook

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json;
use sqlx::{SqlitePool, Row};
use std::path::Path;
use tracing::{debug, error, info, warn};

use crate::types::*;
use crate::types::Digest;
use crate::merkle::AuthorInfo;

/// SQLite index for recovery metadata
pub struct RecoveryIndex {
    pool: SqlitePool,
}

impl RecoveryIndex {
    /// Create a new recovery index
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePool::connect(database_url).await?;
        let index = Self { pool };
        index.initialize_schema().await?;
        Ok(index)
    }

    /// Initialize the database schema
    async fn initialize_schema(&self) -> Result<()> {
        // Create file_versions table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS file_versions (
                path TEXT NOT NULL,
                commit_id BLOB NOT NULL,
                digest BLOB NOT NULL,
                mode INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                PRIMARY KEY (path, commit_id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create commits table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS commits (
                id BLOB PRIMARY KEY,
                parent BLOB,
                tree BLOB NOT NULL,
                session_id TEXT NOT NULL,
                caws_verdict_id TEXT,
                message TEXT,
                stats_json TEXT,
                timestamp INTEGER NOT NULL,
                author_name TEXT NOT NULL,
                author_email TEXT NOT NULL,
                author_agent_id TEXT
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes for performance
        sqlx::query("CREATE INDEX IF NOT EXISTS ix_file_latest ON file_versions(path, commit_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS ix_commits_session ON commits(session_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS ix_commits_timestamp ON commits(timestamp)")
            .execute(&self.pool)
            .await?;

        info!("Recovery index schema initialized");
        Ok(())
    }

    /// Record a file version
    pub async fn record_file_version(
        &self,
        path: &str,
        commit_id: Digest,
        digest: Digest,
        mode: FileMode,
    ) -> Result<()> {
        let now = Utc::now().timestamp();
        
        sqlx::query(
            "INSERT OR REPLACE INTO file_versions (path, commit_id, digest, mode, created_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(path)
        .bind(&commit_id.as_bytes()[..])
        .bind(&digest.as_bytes()[..])
        .bind(mode.to_posix() as i64)
        .bind(now)
        .execute(&self.pool)
        .await?;

        debug!("Recorded file version: {} -> {}", path, digest);
        Ok(())
    }

    /// Record a commit
    pub async fn record_commit(&self, commit: &Commit) -> Result<()> {
        let stats_json = serde_json::to_string(&commit.stats)?;
        let timestamp = commit.timestamp.timestamp();
        
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO commits 
            (id, parent, tree, session_id, caws_verdict_id, message, stats_json, timestamp, author_name, author_email, author_agent_id)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&commit.id.as_bytes()[..])
        .bind(commit.parent.map(|p| p.as_bytes().to_vec()))
        .bind(&commit.tree.as_bytes()[..])
        .bind(&commit.session_id)
        .bind(&commit.caws_verdict_id)
        .bind(&commit.message)
        .bind(&stats_json)
        .bind(timestamp)
        .bind(&commit.author.name)
        .bind(&commit.author.email)
        .bind(&commit.author.agent_id)
        .execute(&self.pool)
        .await?;

        debug!("Recorded commit: {}", commit.id);
        Ok(())
    }

    /// Get the latest version of a file
    pub async fn get_latest_file_version(&self, path: &str) -> Result<Option<FileVersion>> {
        let row = sqlx::query(
            "SELECT commit_id, digest, mode, created_at FROM file_versions WHERE path = ? ORDER BY created_at DESC LIMIT 1",
        )
        .bind(path)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let commit_id_bytes: Vec<u8> = row.get("commit_id");
            let digest_bytes: Vec<u8> = row.get("digest");
            let mode: i64 = row.get("mode");
            let created_at: i64 = row.get("created_at");

            let commit_id = Digest::from_bytes(
                commit_id_bytes.try_into()
                    .map_err(|_| anyhow::anyhow!("Invalid commit_id length"))?
            );
            let digest = Digest::from_bytes(
                digest_bytes.try_into()
                    .map_err(|_| anyhow::anyhow!("Invalid digest length"))?
            );
            let mode = FileMode::from_posix(mode as u32);
            let timestamp = DateTime::from_timestamp(created_at, 0)
                .ok_or_else(|| anyhow::anyhow!("Invalid timestamp"))?;

            Ok(Some(FileVersion {
                path: path.to_string(),
                commit_id,
                digest,
                mode,
                created_at: timestamp,
            }))
        } else {
            Ok(None)
        }
    }

    /// Get all file versions for a path
    pub async fn get_file_history(&self, path: &str) -> Result<Vec<FileVersion>> {
        let rows = sqlx::query(
            "SELECT commit_id, digest, mode, created_at FROM file_versions WHERE path = ? ORDER BY created_at ASC",
        )
        .bind(path)
        .fetch_all(&self.pool)
        .await?;

        let mut versions = Vec::new();
        for row in rows {
            let commit_id_bytes: Vec<u8> = row.get("commit_id");
            let digest_bytes: Vec<u8> = row.get("digest");
            let mode: i64 = row.get("mode");
            let created_at: i64 = row.get("created_at");

            let commit_id = Digest::from_bytes(
                commit_id_bytes.try_into()
                    .map_err(|_| anyhow::anyhow!("Invalid commit_id length"))?
            );
            let digest = Digest::from_bytes(
                digest_bytes.try_into()
                    .map_err(|_| anyhow::anyhow!("Invalid digest length"))?
            );
            let mode = FileMode::from_posix(mode as u32);
            let timestamp = DateTime::from_timestamp(created_at, 0)
                .ok_or_else(|| anyhow::anyhow!("Invalid timestamp"))?;

            versions.push(FileVersion {
                path: path.to_string(),
                commit_id,
                digest,
                mode,
                created_at: timestamp,
            });
        }

        Ok(versions)
    }

    /// Get a commit by ID
    pub async fn get_commit(&self, commit_id: Digest) -> Result<Option<Commit>> {
        let row = sqlx::query(
            "SELECT * FROM commits WHERE id = ?",
        )
        .bind(&commit_id.as_bytes()[..])
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let id_bytes: Vec<u8> = row.get("id");
            let parent_bytes: Option<Vec<u8>> = row.get("parent");
            let tree_bytes: Vec<u8> = row.get("tree");
            let session_id: String = row.get("session_id");
            let caws_verdict_id: Option<String> = row.get("caws_verdict_id");
            let message: Option<String> = row.get("message");
            let stats_json: String = row.get("stats_json");
            let timestamp: i64 = row.get("timestamp");
            let author_name: String = row.get("author_name");
            let author_email: String = row.get("author_email");
            let author_agent_id: Option<String> = row.get("author_agent_id");

            let id = Digest::from_bytes(
                id_bytes.try_into()
                    .map_err(|_| anyhow::anyhow!("Invalid id length"))?
            );
            let parent = if let Some(parent_bytes) = parent_bytes {
                Some(Digest::from_bytes(
                    parent_bytes.try_into()
                        .map_err(|_| anyhow::anyhow!("Invalid parent length"))?
                ))
            } else {
                None
            };
            let tree = Digest::from_bytes(
                tree_bytes.try_into()
                    .map_err(|_| anyhow::anyhow!("Invalid tree length"))?
            );
            let stats: ChangeStats = serde_json::from_str(&stats_json)?;
            let timestamp = DateTime::from_timestamp(timestamp, 0)
                .ok_or_else(|| anyhow::anyhow!("Invalid timestamp"))?;
            let author = AuthorInfo {
                name: author_name.clone(),
                email: author_email.clone(),
                agent_id: author_agent_id.clone(),
            };

            Ok(Some(Commit {
                id,
                parent,
                tree,
                session_id,
                caws_verdict_id,
                message,
                stats,
                timestamp,
                author: AuthorInfo {
                    name: author_name,
                    email: author_email,
                    agent_id: author_agent_id,
                },
            }))
        } else {
            Ok(None)
        }
    }

    /// Get commits for a session
    pub async fn get_session_commits(&self, session_id: &str) -> Result<Vec<Commit>> {
        let rows = sqlx::query(
            "SELECT * FROM commits WHERE session_id = ? ORDER BY timestamp ASC",
        )
        .bind(session_id)
        .fetch_all(&self.pool)
        .await?;

        let mut commits = Vec::new();
        for row in rows {
            if let Ok(commit) = self.row_to_commit(row) {
                commits.push(commit);
            }
        }

        Ok(commits)
    }

    /// Convert a database row to a Commit
    fn row_to_commit(&self, row: sqlx::sqlite::SqliteRow) -> Result<Commit> {
        let id_bytes: Vec<u8> = row.get("id");
        let parent_bytes: Option<Vec<u8>> = row.get("parent");
        let tree_bytes: Vec<u8> = row.get("tree");
        let session_id: String = row.get("session_id");
        let caws_verdict_id: Option<String> = row.get("caws_verdict_id");
        let message: Option<String> = row.get("message");
        let stats_json: String = row.get("stats_json");
        let timestamp: i64 = row.get("timestamp");
        let author_name: String = row.get("author_name");
        let author_email: String = row.get("author_email");
        let author_agent_id: Option<String> = row.get("author_agent_id");

        let id = Digest::from_bytes(
            id_bytes.try_into()
                .map_err(|_| anyhow::anyhow!("Invalid id length"))?
        );
        let parent = if let Some(parent_bytes) = parent_bytes {
            Some(Digest::from_bytes(
                parent_bytes.try_into()
                    .map_err(|_| anyhow::anyhow!("Invalid parent length"))?
            ))
        } else {
            None
        };
        let tree = Digest::from_bytes(
            tree_bytes.try_into()
                .map_err(|_| anyhow::anyhow!("Invalid tree length"))?
        );
        let stats: ChangeStats = serde_json::from_str(&stats_json)?;
        let timestamp = DateTime::from_timestamp(timestamp, 0)
            .ok_or_else(|| anyhow::anyhow!("Invalid timestamp"))?;
        let author = AuthorInfo {
            name: author_name,
            email: author_email,
            agent_id: author_agent_id,
        };

        Ok(Commit {
            id,
            parent,
            tree,
            session_id,
            caws_verdict_id,
            message,
            stats,
            timestamp,
            author,
        })
    }

    /// Rebuild the index from Merkle trees (fsck --reindex)
    pub async fn rebuild_from_trees(&self, refs_dir: &Path) -> Result<RebuildReport> {
        info!("Starting index rebuild from Merkle trees");
        
        let mut report = RebuildReport {
            commits_processed: 0,
            files_processed: 0,
            errors: Vec::new(),
        };

        // Clear existing data
        sqlx::query("DELETE FROM file_versions").execute(&self.pool).await?;
        sqlx::query("DELETE FROM commits").execute(&self.pool).await?;

        // Walk refs directory to find all commits
        if refs_dir.exists() {
            let entries = std::fs::read_dir(refs_dir)?;
            for entry in entries {
                let entry = entry?;
                if entry.path().is_file() {
                    if let Ok(commit_id_hex) = std::fs::read_to_string(&entry.path()) {
                        if let Ok(commit_id) = Digest::from_hex(commit_id_hex.trim()) {
                            match self.rebuild_commit_from_tree(&commit_id, refs_dir).await {
                                Ok(()) => {
                                    report.commits_processed += 1;
                                }
                                Err(e) => {
                                    report.errors.push(format!("Failed to rebuild commit {}: {}", commit_id, e));
                                }
                            }
                        }
                    }
                }
            }
        }

        info!("Index rebuild completed: {} commits, {} files, {} errors", 
              report.commits_processed, report.files_processed, report.errors.len());

        Ok(report)
    }

    /// Rebuild a single commit from its tree
    async fn rebuild_commit_from_tree(&self, commit_id: &Digest, refs_dir: &Path) -> Result<()> {
        // In a full implementation, this would:
        // 1. Read the commit object from the objects store
        // 2. Read the tree object
        // 3. Walk the tree to find all files
        // 4. Record each file version
        // 5. Record the commit
        
        debug!("Rebuilding commit from tree: {}", commit_id);
        Ok(())
    }

    /// Get index statistics
    pub async fn get_stats(&self) -> Result<IndexStats> {
        let file_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM file_versions")
            .fetch_one(&self.pool)
            .await?;
        
        let commit_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM commits")
            .fetch_one(&self.pool)
            .await?;

        let session_count: (i64,) = sqlx::query_as("SELECT COUNT(DISTINCT session_id) FROM commits")
            .fetch_one(&self.pool)
            .await?;

        Ok(IndexStats {
            file_versions: file_count.0 as u64,
            commits: commit_count.0 as u64,
            sessions: session_count.0 as u64,
        })
    }

    /// Close the database connection
    pub async fn close(self) -> Result<()> {
        self.pool.close().await;
        Ok(())
    }
}

/// File version record
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileVersion {
    pub path: String,
    pub commit_id: Digest,
    pub digest: Digest,
    pub mode: FileMode,
    pub created_at: DateTime<Utc>,
}

/// Index rebuild report
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RebuildReport {
    pub commits_processed: u64,
    pub files_processed: u64,
    pub errors: Vec<String>,
}

/// Index statistics
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexStats {
    pub file_versions: u64,
    pub commits: u64,
    pub sessions: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_index_creation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let database_url = format!("sqlite:{}", db_path.display());
        
        let index = RecoveryIndex::new(&database_url).await.unwrap();
        let stats = index.get_stats().await.unwrap();
        
        assert_eq!(stats.file_versions, 0);
        assert_eq!(stats.commits, 0);
        assert_eq!(stats.sessions, 0);
    }

    #[tokio::test]
    async fn test_file_version_recording() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let database_url = format!("sqlite:{}", db_path.display());
        
        let index = RecoveryIndex::new(&database_url).await.unwrap();
        
        let path = "test.txt";
        let commit_id = Digest::from_bytes([1u8; 32]);
        let digest = Digest::from_bytes([2u8; 32]);
        let mode = FileMode::Regular;
        
        index.record_file_version(path, commit_id, digest, mode).await.unwrap();
        
        let version = index.get_latest_file_version(path).await.unwrap().unwrap();
        assert_eq!(version.path, path);
        assert_eq!(version.commit_id, commit_id);
        assert_eq!(version.digest, digest);
        assert_eq!(version.mode, mode);
    }
}
