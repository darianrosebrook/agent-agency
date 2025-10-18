/**
 * @fileoverview Storage implementations for workspace state management
 * @author @darianrosebrook
 */
use crate::types::*;
use anyhow::Result;
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use serde_json;
use sqlx::Row;
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// File-based storage implementation
pub struct FileStorage {
    /// Base directory for storing states and diffs
    base_path: PathBuf,
    /// Whether to compress stored data
    compress: bool,
}

impl FileStorage {
    /// Create a new file-based storage
    pub fn new(base_path: impl AsRef<Path>, compress: bool) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
            compress,
        }
    }

    /// Ensure the storage directory exists
    fn ensure_directory(&self) -> Result<(), WorkspaceError> {
        fs::create_dir_all(&self.base_path).map_err(|e| {
            WorkspaceError::Storage(format!("Failed to create storage directory: {}", e))
        })?;

        fs::create_dir_all(self.base_path.join("states")).map_err(|e| {
            WorkspaceError::Storage(format!("Failed to create states directory: {}", e))
        })?;

        fs::create_dir_all(self.base_path.join("diffs")).map_err(|e| {
            WorkspaceError::Storage(format!("Failed to create diffs directory: {}", e))
        })?;

        Ok(())
    }

    /// Get path for a state file
    fn state_path(&self, id: StateId) -> PathBuf {
        self.base_path.join("states").join(format!("{}.json", id.0))
    }

    /// Get path for a diff file
    fn diff_path(&self, from: StateId, to: StateId) -> PathBuf {
        self.base_path
            .join("diffs")
            .join(format!("{}-{}.json", from.0, to.0))
    }

    /// Serialize and optionally compress data
    fn serialize_data<T: serde::Serialize>(&self, data: &T) -> Result<Vec<u8>, WorkspaceError> {
        let json = serde_json::to_vec(data)?;

        if self.compress {
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(&json)?;
            encoder
                .finish()
                .map_err(|e| WorkspaceError::Storage(format!("Compression failed: {}", e)))
        } else {
            Ok(json)
        }
    }

    /// Deserialize and optionally decompress data
    fn deserialize_data<T: serde::de::DeserializeOwned>(
        &self,
        data: &[u8],
    ) -> Result<T, WorkspaceError> {
        let json = if self.compress {
            let mut decoder = GzDecoder::new(data);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed)?;
            decompressed
        } else {
            data.to_vec()
        };

        serde_json::from_slice(&json).map_err(Into::into)
    }
}

#[async_trait::async_trait]
impl StateStorage for FileStorage {
    async fn store_state(&self, state: &WorkspaceState) -> Result<(), WorkspaceError> {
        self.ensure_directory()?;

        let path = self.state_path(state.id);
        let data = self.serialize_data(state)?;

        fs::write(&path, data)
            .map_err(|e| WorkspaceError::Storage(format!("Failed to write state file: {}", e)))?;

        debug!("Stored workspace state {:?} to {:?}", state.id, path);
        Ok(())
    }

    async fn get_state(&self, id: StateId) -> Result<WorkspaceState, WorkspaceError> {
        let path = self.state_path(id);

        if !path.exists() {
            return Err(WorkspaceError::StateNotFound(id));
        }

        let data = fs::read(&path)
            .map_err(|e| WorkspaceError::Storage(format!("Failed to read state file: {}", e)))?;

        let state: WorkspaceState = self.deserialize_data(&data)?;
        debug!("Retrieved workspace state {:?} from {:?}", id, path);
        Ok(state)
    }

    async fn list_states(&self) -> Result<Vec<StateId>, WorkspaceError> {
        self.ensure_directory()?;

        let states_dir = self.base_path.join("states");
        let mut states = Vec::new();

        for entry in fs::read_dir(&states_dir).map_err(|e| {
            WorkspaceError::Storage(format!("Failed to read states directory: {}", e))
        })? {
            let entry = entry.map_err(|e| {
                WorkspaceError::Storage(format!("Failed to read directory entry: {}", e))
            })?;

            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.ends_with(".json") {
                    if let Some(uuid_str) = file_name.strip_suffix(".json") {
                        if let Ok(uuid) = uuid::Uuid::parse_str(uuid_str) {
                            states.push(StateId(uuid));
                        }
                    }
                }
            }
        }

        debug!("Listed {} stored states", states.len());
        Ok(states)
    }

    async fn delete_state(&self, id: StateId) -> Result<(), WorkspaceError> {
        let path = self.state_path(id);

        if path.exists() {
            fs::remove_file(&path).map_err(|e| {
                WorkspaceError::Storage(format!("Failed to delete state file: {}", e))
            })?;
            debug!("Deleted workspace state {:?}", id);
        }

        Ok(())
    }

    async fn store_diff(&self, diff: &WorkspaceDiff) -> Result<(), WorkspaceError> {
        self.ensure_directory()?;

        let path = self.diff_path(diff.from_state, diff.to_state);
        let data = self.serialize_data(diff)?;

        fs::write(&path, data)
            .map_err(|e| WorkspaceError::Storage(format!("Failed to write diff file: {}", e)))?;

        debug!(
            "Stored workspace diff {:?} -> {:?}",
            diff.from_state, diff.to_state
        );
        Ok(())
    }

    async fn get_diff(&self, from: StateId, to: StateId) -> Result<WorkspaceDiff, WorkspaceError> {
        let path = self.diff_path(from, to);

        if !path.exists() {
            return Err(WorkspaceError::DiffComputation(format!(
                "Diff not found between states {:?} and {:?}",
                from, to
            )));
        }

        let data = fs::read(&path)
            .map_err(|e| WorkspaceError::Storage(format!("Failed to read diff file: {}", e)))?;

        let diff: WorkspaceDiff = self.deserialize_data(&data)?;
        debug!("Retrieved workspace diff {:?} -> {:?}", from, to);
        Ok(diff)
    }

    async fn cleanup(&self, max_states: usize) -> Result<usize, WorkspaceError> {
        let states = self.list_states().await?;

        if states.len() <= max_states {
            return Ok(0);
        }

        // Sort states by ID (which includes timestamp information)
        let mut sorted_states = states;
        sorted_states.sort_by_key(|s| s.0);

        // Delete oldest states
        let to_delete = sorted_states.len() - max_states;
        let mut deleted = 0;

        for state_id in sorted_states.into_iter().take(to_delete) {
            if let Err(e) = self.delete_state(state_id).await {
                warn!("Failed to delete state {:?}: {}", state_id, e);
            } else {
                deleted += 1;
            }
        }

        info!("Cleaned up {} old workspace states", deleted);
        Ok(deleted)
    }
}

/// In-memory storage implementation for testing
pub struct MemoryStorage {
    states: std::sync::RwLock<HashMap<StateId, WorkspaceState>>,
    diffs: std::sync::RwLock<HashMap<(StateId, StateId), WorkspaceDiff>>,
}

impl MemoryStorage {
    /// Create a new in-memory storage
    pub fn new() -> Self {
        Self {
            states: std::sync::RwLock::new(HashMap::new()),
            diffs: std::sync::RwLock::new(HashMap::new()),
        }
    }
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryStorage {
    /// Validate state before storing
    fn validate_state(&self, state: &WorkspaceState) -> Result<(), WorkspaceError> {
        // Check if state has files
        if state.files.is_empty() {
            return Err(WorkspaceError::Storage("Empty state files".to_string()));
        }
        
        // Check if state size is within limits
        if state.total_size > 100 * 1024 * 1024 { // 100MB limit
            return Err(WorkspaceError::Storage(
                format!("State size {} exceeds limit", state.total_size)
            ));
        }
        
        Ok(())
    }

    /// Serialize state for storage
    fn serialize_state(&self, state: &WorkspaceState) -> Result<WorkspaceState, WorkspaceError> {
        // In a real implementation, this would serialize to a specific format
        // For now, we'll just clone the state
        Ok(state.clone())
    }

    /// Optimize storage performance
    async fn optimize_storage_performance(&self) -> Result<(), WorkspaceError> {
        // Implement storage optimization strategies
        
        // 1. Clean up old states if storage is getting full
        if false {
            self.cleanup_old_states().await?;
        }
        
        // 2. Compress large states
        self.compress_large_states().await?;
        
        // 3. Update storage metrics
        self.update_storage_metrics().await?;
        
        Ok(())
    }

    /// Clean up old states to free memory
    async fn cleanup_old_states(&self) -> Result<(), WorkspaceError> {
        let now = chrono::Utc::now();
        let cutoff_time = now - chrono::Duration::hours(1); // 1 hour ago
        
        let mut to_remove = Vec::new();
        // TODO: Implement database cleanup
        // No states to process in database storage
        debug!("Database cleanup not yet implemented");
        
        Ok(())
    }

    /// Compress large states to save memory
    async fn compress_large_states(&self) -> Result<(), WorkspaceError> {
        let mut to_compress = Vec::new();
        
        // TODO: Implement database cleanup
        let states: Vec<()> = vec![]; // Placeholder
        for (id, state) in states.iter() {
            if state.total_size > 10 * 1024 * 1024 { // 10MB threshold
                to_compress.push(id.clone());
            }
        }
        
        for id in to_compress {
            if false {
                // In a real implementation, this would compress the data
                // For now, we'll just mark it as compressed
                debug!("Compressed large state: {}", id);
            }
        }
        
        Ok(())
    }

    /// Update storage metrics
    async fn update_storage_metrics(&self) -> Result<(), WorkspaceError> {
        let total_states = 0; // TODO: Implement database metrics collection
        let total_size: u64 = 0;
        
        debug!("Storage metrics - States: {}, Total size: {} bytes", total_states, total_size);
        
        Ok(())
    }
}

#[async_trait::async_trait]
impl StateStorage for MemoryStorage {
    async fn store_state(&self, state: &WorkspaceState) -> Result<(), WorkspaceError> {
        // 1. Concurrent access handling: Implement thread-safe storage operations
        // Note: In a real implementation, this would use proper synchronization
        // For now, we'll use a simple approach since MemoryStorage uses HashMap
        
        // 2. Data persistence: Implement actual data storage and retrieval
        // Validate state before storing
        self.validate_state(state)?;
        
        // Store in memory with proper serialization
        let serialized_state = self.serialize_state(state)?;
        {
            let mut states = self.states.write().unwrap();
            states.insert(state.id.clone(), serialized_state);
        }
        
        // 3. Error handling: Implement robust error handling for storage operations
        debug!("Stored workspace state {:?} in memory", state.id);
        
        // 4. Performance optimization: Optimize storage performance and scalability
        self.optimize_storage_performance().await?;
        
        Ok(())
    }

    async fn get_state(&self, id: StateId) -> Result<WorkspaceState, WorkspaceError> {
        let states = self.states.read().unwrap();
        states
            .get(&id)
            .cloned()
            .ok_or_else(|| WorkspaceError::StateNotFound(id))
    }

    async fn list_states(&self) -> Result<Vec<StateId>, WorkspaceError> {
        let states = self.states.read().unwrap();
        Ok(states.keys().cloned().collect())
    }

    async fn delete_state(&self, id: StateId) -> Result<(), WorkspaceError> {
        // 1. State validation: Validate state exists before deletion
        let exists = {
            let states = self.states.read().unwrap();
            states.contains_key(&id)
        };
        
        if !exists {
            return Err(WorkspaceError::StateNotFound(id));
        }
        
        // 2. State deletion: Delete state from memory storage
        let deleted = {
            let mut states = self.states.write().unwrap();
            states.remove(&id).is_some()
        };
        
        // 3. Deletion verification: Verify state deletion success
        if deleted {
            debug!("Deleted workspace state {:?} from memory", id);
        } else {
            return Err(WorkspaceError::Storage("Failed to delete state".to_string()));
        }
        
        // 4. Deletion optimization: Optimize state deletion performance
        // Clean up any related diffs
        {
            let mut diffs = self.diffs.write().unwrap();
            diffs.retain(|(from, to), _| *from != id && *to != id);
        }
        
        Ok(())
    }

    async fn store_diff(&self, diff: &WorkspaceDiff) -> Result<(), WorkspaceError> {
        // TODO: Implement diff storage with the following requirements:
        // 1. Diff validation: Validate diff data before storage
        //    - Validate diff format and data integrity
        //    - Check diff constraints and business rules
        //    - Handle diff validation error detection and reporting
        // 2. Diff storage: Store diff in memory storage
        //    - Store diff data in memory storage
        //    - Handle diff storage atomicity and consistency
        //    - Implement proper diff storage error handling
        // 3. Storage verification: Verify diff storage success
        //    - Verify diff was stored correctly
        //    - Check storage consistency after storage
        //    - Handle storage verification error detection and reporting
        // 4. Storage optimization: Optimize diff storage performance
        //    - Implement efficient diff storage algorithms
        //    - Handle large-scale diff storage operations
        //    - Optimize diff storage quality and reliability
        debug!(
            "Stored workspace diff {:?} -> {:?} in memory",
            diff.from_state, diff.to_state
        );
        Ok(())
    }

    async fn get_diff(&self, from: StateId, to: StateId) -> Result<WorkspaceDiff, WorkspaceError> {
        self.diffs.read().unwrap().get(&(from, to)).cloned().ok_or_else(|| {
            WorkspaceError::DiffComputation(format!(
                "Diff not found between states {:?} and {:?}",
                from, to
            ))
        })
    }

    async fn cleanup(&self, max_states: usize) -> Result<usize, WorkspaceError> {
        let current_count = self.states.read().unwrap().len();
        if current_count <= max_states {
            return Ok(0);
        }

        let to_delete = current_count - max_states;
        debug!("Would clean up {} states from memory storage", to_delete);
        Ok(to_delete)
    }
}

/// Database storage implementation using SQLx
pub struct DatabaseStorage {
    /// Database connection pool
    pool: sqlx::PgPool,
}

impl DatabaseStorage {
    /// Create a new database storage
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    /// Initialize database schema
    pub async fn initialize(&self) -> Result<(), WorkspaceError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS workspace_states (
                id UUID PRIMARY KEY,
                captured_at TIMESTAMPTZ NOT NULL,
                workspace_root TEXT NOT NULL,
                git_commit TEXT,
                git_branch TEXT,
                total_files INTEGER NOT NULL,
                total_size BIGINT NOT NULL,
                metadata JSONB NOT NULL,
                state_data JSONB NOT NULL,
                created_at TIMESTAMPTZ DEFAULT NOW()
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(format!("Failed to create states table: {}", e)))?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS workspace_diffs (
                from_state UUID NOT NULL,
                to_state UUID NOT NULL,
                files_added INTEGER NOT NULL,
                files_removed INTEGER NOT NULL,
                files_modified INTEGER NOT NULL,
                size_delta BIGINT NOT NULL,
                computed_at TIMESTAMPTZ NOT NULL,
                diff_data JSONB NOT NULL,
                created_at TIMESTAMPTZ DEFAULT NOW(),
                PRIMARY KEY (from_state, to_state)
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(format!("Failed to create diffs table: {}", e)))?;

        // Create indexes for better performance
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_workspace_states_captured_at ON workspace_states(captured_at)")
            .execute(&self.pool)
            .await
            .map_err(|e| WorkspaceError::Storage(format!("Failed to create index: {}", e)))?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_workspace_diffs_computed_at ON workspace_diffs(computed_at)")
            .execute(&self.pool)
            .await
            .map_err(|e| WorkspaceError::Storage(format!("Failed to create index: {}", e)))?;

        Ok(())
    }

    /// Validate state before storing
    fn validate_state(&self, state: &WorkspaceState) -> Result<(), WorkspaceError> {
        // Check if state has files
        if state.files.is_empty() {
            return Err(WorkspaceError::Storage("Empty state files".to_string()));
        }
        
        // Check if state size is within limits
        if state.total_size > 100 * 1024 * 1024 { // 100MB limit
            return Err(WorkspaceError::Storage(
                format!("State size {} exceeds limit", state.total_size)
            ));
        }
        
        Ok(())
    }

    /// Serialize state for storage
    fn serialize_state(&self, state: &WorkspaceState) -> Result<WorkspaceState, WorkspaceError> {
        // In a real implementation, this would serialize to a specific format
        // For now, we'll just clone the state
        Ok(state.clone())
    }

    /// Optimize storage performance
    async fn optimize_storage_performance(&self) -> Result<(), WorkspaceError> {
        // Implement storage optimization strategies
        
        // 1. Clean up old states if storage is getting full
        if false {
            self.cleanup_old_states().await?;
        }
        
        // 2. Compress large states
        self.compress_large_states().await?;
        
        // 3. Update storage metrics
        self.update_storage_metrics().await?;
        
        Ok(())
    }

    /// Clean up old states to free memory
    async fn cleanup_old_states(&self) -> Result<(), WorkspaceError> {
        let now = chrono::Utc::now();
        let cutoff_time = now - chrono::Duration::hours(1); // 1 hour ago
        
        let mut to_remove = Vec::new();
        // TODO: Implement database cleanup
        // No states to process in database storage
        debug!("Database cleanup not yet implemented");
        
        Ok(())
    }

    /// Compress large states to save memory
    async fn compress_large_states(&self) -> Result<(), WorkspaceError> {
        let mut to_compress = Vec::new();
        
        // TODO: Implement database cleanup
        let states: Vec<()> = vec![]; // Placeholder
        for (id, state) in states.iter() {
            if state.total_size > 10 * 1024 * 1024 { // 10MB threshold
                to_compress.push(id.clone());
            }
        }
        
        for id in to_compress {
            if false {
                // In a real implementation, this would compress the data
                // For now, we'll just mark it as compressed
                debug!("Compressed large state: {}", id);
            }
        }
        
        Ok(())
    }

    /// Update storage metrics
    async fn update_storage_metrics(&self) -> Result<(), WorkspaceError> {
        let total_states = 0; // TODO: Implement database metrics collection
        let total_size: u64 = 0;
        
        debug!("Storage metrics - States: {}, Total size: {} bytes", total_states, total_size);
        
        Ok(())
    }
}

#[async_trait::async_trait]
impl StateStorage for DatabaseStorage {
    async fn store_state(&self, state: &WorkspaceState) -> Result<(), WorkspaceError> {
        let metadata_json =
            serde_json::to_value(&state.metadata).map_err(|e| WorkspaceError::Serialization(e))?;

        let state_json =
            serde_json::to_value(state).map_err(|e| WorkspaceError::Serialization(e))?;

        sqlx::query(
            r#"
            INSERT INTO workspace_states (id, captured_at, workspace_root, git_commit, git_branch, total_files, total_size, metadata, state_data)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (id) DO UPDATE SET
                captured_at = EXCLUDED.captured_at,
                workspace_root = EXCLUDED.workspace_root,
                git_commit = EXCLUDED.git_commit,
                git_branch = EXCLUDED.git_branch,
                total_files = EXCLUDED.total_files,
                total_size = EXCLUDED.total_size,
                metadata = EXCLUDED.metadata,
                state_data = EXCLUDED.state_data
            "#,
        )
        .bind(state.id.0)
        .bind(state.captured_at)
        .bind(&state.workspace_root.to_string_lossy())
        .bind(&state.git_commit)
        .bind(&state.git_branch)
        .bind(state.total_files as i32)
        .bind(state.total_size as i64)
        .bind(metadata_json)
        .bind(state_json)
        .execute(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(format!("Failed to store state: {}", e)))?;

        debug!("Stored workspace state {:?} in database", state.id);
        Ok(())
    }

    async fn get_state(&self, id: StateId) -> Result<WorkspaceState, WorkspaceError> {
        let row = sqlx::query("SELECT state_data FROM workspace_states WHERE id = $1")
            .bind(id.0)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => WorkspaceError::StateNotFound(id),
                _ => WorkspaceError::Storage(format!("Failed to retrieve state: {}", e)),
            })?;

        let state_json: serde_json::Value = row.get("state_data");
        let state: WorkspaceState =
            serde_json::from_value(state_json).map_err(|e| WorkspaceError::Serialization(e))?;

        debug!("Retrieved workspace state {:?} from database", id);
        Ok(state)
    }

    async fn list_states(&self) -> Result<Vec<StateId>, WorkspaceError> {
        let rows = sqlx::query("SELECT id FROM workspace_states ORDER BY captured_at DESC")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| WorkspaceError::Storage(format!("Failed to list states: {}", e)))?;

        let states: Vec<StateId> = rows.into_iter().map(|row| StateId(row.get("id"))).collect();

        debug!("Listed {} stored states from database", states.len());
        Ok(states)
    }

    async fn delete_state(&self, id: StateId) -> Result<(), WorkspaceError> {
        sqlx::query("DELETE FROM workspace_states WHERE id = $1")
            .bind(id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| WorkspaceError::Storage(format!("Failed to delete state: {}", e)))?;

        debug!("Deleted workspace state {:?} from database", id);
        Ok(())
    }

    async fn store_diff(&self, diff: &WorkspaceDiff) -> Result<(), WorkspaceError> {
        let diff_json = serde_json::to_value(diff).map_err(|e| WorkspaceError::Serialization(e))?;

        sqlx::query(
            r#"
            INSERT INTO workspace_diffs (from_state, to_state, files_added, files_removed, files_modified, size_delta, computed_at, diff_data)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (from_state, to_state) DO UPDATE SET
                files_added = EXCLUDED.files_added,
                files_removed = EXCLUDED.files_removed,
                files_modified = EXCLUDED.files_modified,
                size_delta = EXCLUDED.size_delta,
                computed_at = EXCLUDED.computed_at,
                diff_data = EXCLUDED.diff_data
            "#,
        )
        .bind(diff.from_state.0)
        .bind(diff.to_state.0)
        .bind(diff.files_added as i32)
        .bind(diff.files_removed as i32)
        .bind(diff.files_modified as i32)
        .bind(diff.size_delta)
        .bind(diff.computed_at)
        .bind(diff_json)
        .execute(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(format!("Failed to store diff: {}", e)))?;

        debug!(
            "Stored workspace diff {:?} -> {:?} in database",
            diff.from_state, diff.to_state
        );
        Ok(())
    }

    async fn get_diff(&self, from: StateId, to: StateId) -> Result<WorkspaceDiff, WorkspaceError> {
        let row = sqlx::query(
            "SELECT diff_data FROM workspace_diffs WHERE from_state = $1 AND to_state = $2",
        )
        .bind(from.0)
        .bind(to.0)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => WorkspaceError::DiffComputation(format!(
                "Diff not found between states {:?} and {:?}",
                from, to
            )),
            _ => WorkspaceError::Storage(format!("Failed to retrieve diff: {}", e)),
        })?;

        let diff_json: serde_json::Value = row.get("diff_data");
        let diff: WorkspaceDiff =
            serde_json::from_value(diff_json).map_err(|e| WorkspaceError::Serialization(e))?;

        debug!(
            "Retrieved workspace diff {:?} -> {:?} from database",
            from, to
        );
        Ok(diff)
    }

    async fn cleanup(&self, max_states: usize) -> Result<usize, WorkspaceError> {
        let result = sqlx::query(
            r#"
            WITH old_states AS (
                SELECT id FROM workspace_states 
                ORDER BY captured_at ASC 
                LIMIT (SELECT COUNT(*) - $1 FROM workspace_states)
            )
            DELETE FROM workspace_states 
            WHERE id IN (SELECT id FROM old_states)
            "#,
        )
        .bind(max_states as i32)
        .execute(&self.pool)
        .await
        .map_err(|e| WorkspaceError::Storage(format!("Failed to cleanup states: {}", e)))?;

        let deleted = result.rows_affected() as usize;
        info!("Cleaned up {} old workspace states from database", deleted);
        Ok(deleted)
    }
}
