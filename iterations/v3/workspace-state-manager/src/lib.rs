pub mod manager;
pub mod rollback;
pub mod storage;
/**
 * @fileoverview Workspace State Manager - Repository state management with stable views, diffs, and rollback capabilities
 * @author @darianrosebrook
 */
pub mod types;

// Re-export main types and functionality
pub use manager::WorkspaceStateManager;
pub use rollback::{RollbackManager, RollbackResult, ViewMetadata, WorkspaceViewManager};
pub use storage::{DatabaseStorage, FileStorage, MemoryStorage};
pub use types::*;

use std::sync::Arc;

/// Create a new workspace state manager with file-based storage
pub fn create_file_manager(
    workspace_root: impl AsRef<std::path::Path>,
    config: WorkspaceConfig,
    storage_path: impl AsRef<std::path::Path>,
) -> WorkspaceStateManager {
    let storage = Box::new(FileStorage::new(storage_path, config.compress_states));
    WorkspaceStateManager::new(workspace_root, config, storage)
}

/// Create a new workspace state manager with in-memory storage (for testing)
pub fn create_memory_manager(
    workspace_root: impl AsRef<std::path::Path>,
    config: WorkspaceConfig,
) -> WorkspaceStateManager {
    let storage = Box::new(MemoryStorage::new());
    WorkspaceStateManager::new(workspace_root, config, storage)
}

/// Create a new workspace state manager with database storage
pub fn create_database_manager(
    workspace_root: impl AsRef<std::path::Path>,
    config: WorkspaceConfig,
    pool: sqlx::PgPool,
) -> WorkspaceStateManager {
    let storage = Box::new(DatabaseStorage::new(pool));
    WorkspaceStateManager::new(workspace_root, config, storage)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_workspace_state_capture() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_path = temp_dir.path();

        // Create some test files
        std::fs::create_dir_all(workspace_path.join("src")).unwrap();
        std::fs::write(workspace_path.join("README.md"), "# Test Project").unwrap();
        std::fs::write(workspace_path.join("src/main.rs"), "fn main() {}").unwrap();

        let config = WorkspaceConfig::default();
        let manager = create_memory_manager(workspace_path, config);

        // Capture state
        let result = manager.capture_state().await.unwrap();
        assert!(result.success);
        assert_eq!(result.data.0.to_string().len(), 36); // UUID length

        // Verify we can retrieve the state
        let state = manager.get_state(result.data).await.unwrap();
        assert_eq!(state.total_files, 2);
        assert!(state
            .files
            .contains_key(&std::path::PathBuf::from("README.md")));
        assert!(state
            .files
            .contains_key(&std::path::PathBuf::from("src/main.rs")));
    }

    #[tokio::test]
    async fn test_workspace_diff() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_path = temp_dir.path();

        // Create initial files
        std::fs::create_dir_all(workspace_path.join("src")).unwrap();
        std::fs::write(workspace_path.join("README.md"), "# Test Project").unwrap();
        std::fs::write(workspace_path.join("src/main.rs"), "fn main() {}").unwrap();

        let config = WorkspaceConfig::default();
        let manager = create_memory_manager(workspace_path, config);

        // Capture initial state
        let initial_result = manager.capture_state().await.unwrap();
        let initial_state = initial_result.data;

        // Modify files
        std::fs::write(workspace_path.join("README.md"), "# Updated Test Project").unwrap();
        std::fs::write(workspace_path.join("src/lib.rs"), "pub fn hello() {}").unwrap();

        // Capture modified state
        let modified_result = manager.capture_state().await.unwrap();
        let modified_state = modified_result.data;

        // Compute diff
        let diff_result = manager
            .compute_diff(initial_state, modified_state)
            .await
            .unwrap();
        let diff = diff_result.data;

        assert_eq!(diff.files_modified, 1); // README.md
        assert_eq!(diff.files_added, 1); // src/lib.rs
        assert_eq!(diff.files_removed, 0);
    }

    #[tokio::test]
    async fn test_workspace_view_creation() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_path = temp_dir.path();
        let views_dir = temp_dir.path().join("views");

        // Create test files
        std::fs::create_dir_all(workspace_path.join("src")).unwrap();
        std::fs::write(workspace_path.join("README.md"), "# Test Project").unwrap();

        let config = WorkspaceConfig::default();
        let manager = Arc::new(create_memory_manager(workspace_path, config));
        let view_manager = WorkspaceViewManager::new(Arc::clone(&manager), &views_dir);

        // Capture state
        let result = manager.capture_state().await.unwrap();
        let state_id = result.data;

        // Create view
        let view_result = view_manager
            .create_view(state_id, Some("test-view".to_string()))
            .await
            .unwrap();
        let view_path = view_result.data;

        // Verify view was created
        assert!(view_path.exists());
        assert!(view_path.join("README.md").exists());
        assert!(view_path.join(".workspace-view.json").exists());

        // Verify metadata
        let metadata = view_manager.get_view_metadata("test-view").await.unwrap();
        assert_eq!(metadata.name, "test-view");
        assert_eq!(metadata.state_id, state_id);
    }

    #[tokio::test]
    async fn test_rollback_operation() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_path = temp_dir.path();
        let backup_dir = temp_dir.path().join("backups");

        // Create initial files
        std::fs::create_dir_all(workspace_path.join("src")).unwrap();
        std::fs::write(workspace_path.join("README.md"), "# Test Project").unwrap();

        let config = WorkspaceConfig::default();
        let manager = Arc::new(create_memory_manager(workspace_path, config));
        let rollback_manager = RollbackManager::new(Arc::clone(&manager), &backup_dir);

        // Capture initial state
        let result = manager.capture_state().await.unwrap();
        let initial_state = result.data;

        // Modify files
        std::fs::write(workspace_path.join("README.md"), "# Modified Project").unwrap();
        std::fs::write(workspace_path.join("src/main.rs"), "fn main() {}").unwrap();

        // Perform rollback
        let rollback_result = rollback_manager
            .rollback_to_state(initial_state, false)
            .await
            .unwrap();

        assert!(rollback_result.success);
        assert_eq!(rollback_result.files_modified, 1); // README.md restored
        assert_eq!(rollback_result.files_removed, 1); // src/main.rs removed

        // Verify rollback worked
        let content = std::fs::read_to_string(workspace_path.join("README.md")).unwrap();
        assert_eq!(content, "# Test Project");
        assert!(!workspace_path.join("src/main.rs").exists());
    }
}
