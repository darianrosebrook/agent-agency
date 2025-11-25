#![cfg(feature = "database")]
//! Workspace Registry
//!
//! Manages workspace discovery, access controls, and permissions for cross-workspace memory access.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::memory_types::{WorkspaceAccess, WorkspaceAccessConfig, WorkspaceEntry};
use crate::{MemoryError, MemoryResult};
use sqlx::{PgPool, Row};

/// Workspace registry for managing workspace access controls
#[derive(Debug)]
pub struct WorkspaceRegistry {
    /// Configuration for workspace access
    config: WorkspaceAccessConfig,
    /// Registry of known workspaces
    workspaces: Arc<RwLock<HashMap<String, WorkspaceEntry>>>,
    /// Database pool for persistence
    db_pool: Arc<PgPool>,
}

impl WorkspaceRegistry {
    /// Create a new workspace registry
    pub fn new(config: WorkspaceAccessConfig, db_pool: Arc<PgPool>) -> Self {
        Self {
            config,
            workspaces: Arc::new(RwLock::new(HashMap::new())),
            db_pool,
        }
    }

    /// Initialize the workspace registry
    pub async fn initialize(&self) -> MemoryResult<()> {
        // Load persisted workspaces from database
        self.load_workspaces().await?;

        // Initialize default workspaces
        self.initialize_default_workspaces().await?;

        // Auto-discover workspaces
        self.discover_workspaces().await?;

        Ok(())
    }

    /// Register a workspace with the given access level
    pub async fn register_workspace(
        &self,
        path: &Path,
        access: WorkspaceAccess,
    ) -> MemoryResult<String> {
        let path_str = path.to_string_lossy().to_string();
        let workspace_id = self.generate_workspace_id(path);

        let entry = WorkspaceEntry {
            id: workspace_id.clone(),
            name: path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "Unknown".to_string()),
            path: path.to_path_buf(),
            access,
            created_at: Utc::now(),
            discovered_at: Utc::now(),
            last_accessed: Utc::now(),
            access_count: 0,
            is_default: self.is_default_workspace(path),
        };

        // Check if workspace count limit exceeded
        {
            let workspaces = self.workspaces.read().await;
            if workspaces.len() >= self.config.max_workspaces {
                return Err(MemoryError::Validation(format!(
                    "Maximum workspace limit ({}) exceeded",
                    self.config.max_workspaces
                )));
            }
        }

        // Store in registry
        {
            let mut workspaces = self.workspaces.write().await;
            workspaces.insert(workspace_id.clone(), entry);
        }

        // Persist to database
        self.persist_workspace(&workspace_id).await?;

        Ok(workspace_id)
    }

    /// Get workspace entry by ID
    pub async fn get_workspace(&self, workspace_id: &str) -> MemoryResult<Option<WorkspaceEntry>> {
        let workspaces = self.workspaces.read().await;
        Ok(workspaces.get(workspace_id).cloned())
    }

    /// Record workspace access
    pub async fn record_access(&self, workspace_id: &str) -> MemoryResult<()> {
        let mut workspaces = self.workspaces.write().await;
        if let Some(entry) = workspaces.get_mut(workspace_id) {
            entry.last_accessed = Utc::now();
            entry.access_count += 1;
            self.persist_workspace(workspace_id).await?;
        }
        Ok(())
    }

    /// Check if access to a workspace is allowed
    pub async fn check_access(&self, workspace_id: &str) -> MemoryResult<bool> {
        if let Some(entry) = self.get_workspace(workspace_id).await? {
            match entry.access {
                WorkspaceAccess::Enabled => Ok(true),
                WorkspaceAccess::Disabled => Ok(false),
                WorkspaceAccess::ReadOnly => Ok(true), // Allow read access
                WorkspaceAccess::Blocked => Ok(false),
            }
        } else {
            // Unknown workspace - check if it's in a default or blocked path
            let workspaces = self.workspaces.read().await;
            for (_, entry) in workspaces.iter() {
                if entry.path.to_string_lossy().contains(workspace_id) {
                    return Box::pin(self.check_access(&entry.id)).await;
                }
            }
            Ok(false) // Unknown workspace defaults to blocked
        }
    }

    /// Get all accessible workspaces for the current context
    pub async fn get_accessible_workspaces(&self) -> MemoryResult<Vec<WorkspaceEntry>> {
        let workspaces = self.workspaces.read().await;
        let accessible = workspaces
            .values()
            .filter(|entry| {
                matches!(
                    entry.access,
                    WorkspaceAccess::Enabled | WorkspaceAccess::ReadOnly
                )
            })
            .cloned()
            .collect();
        Ok(accessible)
    }

    /// Discover workspaces in configured paths
    async fn discover_workspaces(&self) -> MemoryResult<()> {
        for base_path_str in &self.config.discovery_paths {
            let base_path = std::path::Path::new(base_path_str);
            if !base_path.exists() {
                continue;
            }

            // Find git repositories (common workspace indicator)
            self.discover_git_repos(base_path).await?;
        }
        Ok(())
    }

    /// Discover git repositories as workspaces
    async fn discover_git_repos(&self, base_path: &Path) -> MemoryResult<()> {
        // Simple directory traversal to find .git folders
        self.traverse_and_register(base_path, 3).await // Max depth 3
    }

    /// Traverse directories and register workspaces
    async fn traverse_and_register(&self, path: &Path, max_depth: usize) -> MemoryResult<()> {
        if max_depth == 0 {
            return Ok(());
        }

        if let Ok(entries) = tokio::fs::read_dir(path).await {
            let mut entries = entries;
            while let Ok(Some(entry)) = entries.next_entry().await {
                let entry_path = entry.path();

                if entry_path.is_dir() {
                    // Check if this is a workspace (has .git or specific markers)
                    if self.is_workspace_candidate(&entry_path) {
                        // Check if already registered
                        let path_str = entry_path.to_string_lossy().to_string();
                        let workspace_id = self.generate_workspace_id(&entry_path);

                        if self.get_workspace(&workspace_id).await?.is_none() {
                            let access = if self.is_blocked_workspace(&entry_path) {
                                WorkspaceAccess::Blocked
                            } else {
                                self.config.default_access.clone()
                            };

                            self.register_workspace(&entry_path, access).await?;
                        }
                    } else {
                        // Recurse into subdirectories
                        Box::pin(self.traverse_and_register(&entry_path, max_depth - 1)).await?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Check if a path is a workspace candidate
    fn is_workspace_candidate(&self, path: &Path) -> bool {
        // Check for common workspace indicators
        path.join(".git").exists()
            || path.join("Cargo.toml").exists()
            || path.join("package.json").exists()
            || path.join("pyproject.toml").exists()
    }

    /// Check if a workspace is in the blocked list
    fn is_blocked_workspace(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy().to_string();
        self.config
            .blocked_workspaces
            .iter()
            .any(|blocked| path_str.starts_with(blocked))
    }

    /// Check if a workspace is a default workspace
    fn is_default_workspace(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy().to_string();
        self.config
            .default_workspaces
            .iter()
            .any(|default| path_str.starts_with(default))
    }

    /// Initialize default workspaces
    async fn initialize_default_workspaces(&self) -> MemoryResult<()> {
        for default_path in &self.config.default_workspaces {
            let path = PathBuf::from(default_path);
            if path.exists() {
                self.register_workspace(&path, WorkspaceAccess::Enabled)
                    .await?;
            }
        }
        Ok(())
    }

    /// Generate a unique workspace ID from path
    fn generate_workspace_id(&self, path: &Path) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        path.hash(&mut hasher);
        format!("ws_{:x}", hasher.finish())
    }

    /// Load workspaces from database
    async fn load_workspaces(&self) -> MemoryResult<()> {
        let rows = sqlx::query(
            r#"
            SELECT workspace_id, name, path, access, created_at, last_accessed, access_count, discovered_at, is_default
            FROM workspace_registry
            ORDER BY last_accessed DESC
            "#
        )
        .fetch_all(&*self.db_pool)
        .await
        .map_err(MemoryError::Database)?;

        let mut workspaces = self.workspaces.write().await;
        for row in rows {
            let workspace_id: String =
                row.try_get("workspace_id").map_err(MemoryError::Database)?;
            let name: String = row.try_get("name").map_err(MemoryError::Database)?;
            let created_at: DateTime<Utc> =
                row.try_get("created_at").map_err(MemoryError::Database)?;
            let last_accessed: Option<DateTime<Utc>> = row.try_get("last_accessed").ok();
            let path: String = row.try_get("path").map_err(MemoryError::Database)?;
            let access: String = row.try_get("access").map_err(MemoryError::Database)?;
            let access_count: i64 = row.try_get("access_count").map_err(MemoryError::Database)?;
            let discovered_at: DateTime<Utc> = row
                .try_get("discovered_at")
                .map_err(MemoryError::Database)?;
            let is_default: bool = row.try_get("is_default").map_err(MemoryError::Database)?;

            let entry = WorkspaceEntry {
                id: workspace_id.clone(),
                name,
                path: PathBuf::from(path),
                access: match access.as_str() {
                    "enabled" => crate::memory_types::WorkspaceAccess::Enabled,
                    "disabled" => crate::memory_types::WorkspaceAccess::Disabled,
                    "readonly" => crate::memory_types::WorkspaceAccess::ReadOnly,
                    "blocked" => crate::memory_types::WorkspaceAccess::Blocked,
                    _ => crate::memory_types::WorkspaceAccess::Enabled, // default
                },
                created_at,
                last_accessed: last_accessed.unwrap_or(created_at), // Use created_at if no last_accessed
                access_count: access_count as u64,
                discovered_at,
                is_default,
            };

            workspaces.insert(workspace_id, entry);
        }

        tracing::info!("Loaded {} workspaces from database", workspaces.len());
        Ok(())
    }

    /// Persist workspace to database
    async fn persist_workspace(&self, workspace_id: &str) -> MemoryResult<()> {
        let workspaces = self.workspaces.read().await;

        if let Some(entry) = workspaces.get(workspace_id) {
            sqlx::query(
                r#"
                INSERT INTO workspace_registry (
                    workspace_id, name, path, access, created_at, last_accessed, access_count, discovered_at, is_default
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                ON CONFLICT (workspace_id) DO UPDATE SET
                    name = EXCLUDED.name,
                    path = EXCLUDED.path,
                    access = EXCLUDED.access,
                    last_accessed = EXCLUDED.last_accessed,
                    access_count = EXCLUDED.access_count,
                    is_default = EXCLUDED.is_default
                "#
            )
            .bind(&entry.id)
            .bind(&entry.name)
            .bind(entry.path.to_string_lossy().as_ref())
            .bind(match entry.access {
                crate::memory_types::WorkspaceAccess::Enabled => "enabled",
                crate::memory_types::WorkspaceAccess::Disabled => "disabled",
                crate::memory_types::WorkspaceAccess::ReadOnly => "readonly",
                crate::memory_types::WorkspaceAccess::Blocked => "blocked",
            })
            .bind(entry.created_at)
            .bind(entry.last_accessed)
            .bind(entry.access_count as i64)
            .bind(entry.discovered_at)
            .bind(entry.is_default)
            .execute(&*self.db_pool)
            .await
            .map_err(MemoryError::Database)?;

            tracing::debug!("Persisted workspace {} to database", workspace_id);
        }

        Ok(())
    }

    /// Get workspaces that haven't been accessed since the cutoff date
    pub async fn get_unused_workspaces(
        &self,
        cutoff_date: chrono::DateTime<chrono::Utc>,
    ) -> MemoryResult<Vec<WorkspaceEntry>> {
        let workspaces = self.workspaces.read().await;
        let unused = workspaces
            .values()
            .filter(|entry| entry.last_accessed < cutoff_date)
            .cloned()
            .collect();
        Ok(unused)
    }

    /// Update workspace access level
    pub async fn update_workspace_access(
        &self,
        workspace_id: &str,
        access: WorkspaceAccess,
    ) -> MemoryResult<()> {
        let mut workspaces = self.workspaces.write().await;
        if let Some(entry) = workspaces.get_mut(workspace_id) {
            entry.access = access;
            entry.last_accessed = chrono::Utc::now();
            Ok(())
        } else {
            Err(crate::MemoryError::NotFound(format!(
                "Workspace {} not found",
                workspace_id
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_workspace_registration() {
        let temp_dir = TempDir::new().unwrap();
        let config = WorkspaceAccessConfig::default();
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost/agent_agency_v3".to_string());
        let db_pool = match PgPool::connect(&database_url).await {
            Ok(pool) => Arc::new(pool),
            Err(e) => {
                eprintln!("Skipping test: Database not available: {}", e);
                return;
            }
        };
        let registry = WorkspaceRegistry::new(config, db_pool);

        // Test registering a workspace
        let workspace_id = registry
            .register_workspace(temp_dir.path(), WorkspaceAccess::Enabled)
            .await
            .unwrap();

        // Verify it's registered
        let entry = registry
            .get_workspace(&workspace_id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(entry.access, WorkspaceAccess::Enabled);
        // is_default is only true if the path is in default_workspaces config
        // For a temp directory test, this will be false unless we configure it
        let is_in_default_workspaces = registry.config.default_workspaces.iter().any(|d| temp_dir.path().to_string_lossy().starts_with(d));
        assert_eq!(entry.is_default, is_in_default_workspaces, "is_default should match whether path is in default_workspaces config");
    }

    #[tokio::test]
    async fn test_access_control() {
        let temp_dir = TempDir::new().unwrap();
        let config = WorkspaceAccessConfig::default();
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost/agent_agency_v3".to_string());
        let db_pool = match PgPool::connect(&database_url).await {
            Ok(pool) => Arc::new(pool),
            Err(e) => {
                eprintln!("Skipping test: Database not available: {}", e);
                return;
            }
        };
        let registry = WorkspaceRegistry::new(config, db_pool);

        // Register blocked workspace
        let blocked_path = temp_dir.path().join("blocked");
        std::fs::create_dir(&blocked_path).unwrap();
        let workspace_id = registry
            .register_workspace(&blocked_path, WorkspaceAccess::Blocked)
            .await
            .unwrap();

        // Check access is denied
        assert!(!registry.check_access(&workspace_id).await.unwrap());
    }
}
