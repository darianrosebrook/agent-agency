//! Cleanup hooks for index maintenance on file removal

use async_trait::async_trait;
use std::path::PathBuf;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use parking_lot::RwLock;
use crate::data_processing_types::ProcessingId;
use tracing::{info, warn};

/// Registry mapping file paths to their ProcessingIds for cleanup
#[derive(Debug, Clone)]
pub struct PathRegistry {
    path_to_ids: Arc<RwLock<HashMap<PathBuf, HashSet<ProcessingId>>>>,
}

impl PathRegistry {
    pub fn new() -> Self {
        Self {
            path_to_ids: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a path-to-id mapping
    pub fn register(&self, path: PathBuf, id: ProcessingId) {
        self.path_to_ids.write().entry(path).or_insert_with(HashSet::new).insert(id);
    }

    /// Get all ProcessingIds for a given path
    pub fn get_ids_for_path(&self, path: &PathBuf) -> HashSet<ProcessingId> {
        self.path_to_ids.read().get(path).cloned().unwrap_or_default()
    }

    /// Remove all mappings for a path
    pub fn purge_path(&self, path: &PathBuf) -> HashSet<ProcessingId> {
        self.path_to_ids.write().remove(path).unwrap_or_default()
    }
}

impl Default for PathRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for index cleanup operations when files are removed
#[async_trait]
pub trait IndexCleanup: Send + Sync {
    /// Purge all entries related to the given path from the index
    async fn purge_path(&self, path: &PathBuf);
}

/// No-op cleanup implementation for when no cleanup is needed
pub struct NoOpCleanup;

#[async_trait]
impl IndexCleanup for NoOpCleanup {
    async fn purge_path(&self, _path: &PathBuf) {
        // No-op
    }
}

/// Real cleanup implementation that removes indexed content
pub struct IndexCleanupHandler {
    path_registry: Arc<PathRegistry>,
    indexing_stage: Arc<crate::indexing::DefaultIndexingStage>,
}

impl IndexCleanupHandler {
    pub fn new(path_registry: Arc<PathRegistry>, indexing_stage: Arc<crate::indexing::DefaultIndexingStage>) -> Self {
        Self {
            path_registry,
            indexing_stage,
        }
    }
}

#[async_trait]
impl IndexCleanup for IndexCleanupHandler {
    async fn purge_path(&self, path: &PathBuf) {
        info!("Purging indexes for removed file: {:?}", path);
        
        let ids_to_remove = self.path_registry.purge_path(path);
        
        if ids_to_remove.is_empty() {
            warn!("No indexed content found for path: {:?}", path);
            return;
        }

        info!("Removing {} indexed entries for path: {:?}", ids_to_remove.len(), path);
        
        // TODO: Implement actual purge methods in DefaultIndexingStage
        // For now, log what would be removed
        for id in &ids_to_remove {
            // Would call: self.indexing_stage.purge_by_id(id).await;
            info!("Would purge ProcessingId: {}", id);
        }
    }
}
