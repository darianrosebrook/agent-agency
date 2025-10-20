/**
 * @fileoverview Core workspace state manager implementation
 * @author @darianrosebrook
 */
use crate::types::*;
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tracing::{debug, info, warn};

/// Main workspace state manager
pub struct WorkspaceStateManager {
    /// Configuration for the manager
    config: WorkspaceConfig,
    /// Storage backend for states and diffs
    storage: Box<dyn StateStorage>,
    /// Current workspace root path
    workspace_root: PathBuf,
}

impl WorkspaceStateManager {
    /// Create a new workspace state manager
    pub fn new(
        workspace_root: impl AsRef<Path>,
        config: WorkspaceConfig,
        storage: Box<dyn StateStorage>,
    ) -> Self {
        Self {
            config,
            storage,
            workspace_root: workspace_root.as_ref().to_path_buf(),
        }
    }

    /// Capture the current state of the workspace
    pub async fn capture_state(&self) -> Result<WorkspaceResult<StateId>, WorkspaceError> {
        let start_time = Instant::now();
        let mut warnings = Vec::new();

        info!(
            "Starting workspace state capture for {:?}",
            self.workspace_root
        );

        // Validate workspace path
        if !self.workspace_root.exists() {
            return Err(WorkspaceError::InvalidWorkspacePath(
                self.workspace_root.clone(),
            ));
        }

        if !self.workspace_root.is_dir() {
            return Err(WorkspaceError::InvalidWorkspacePath(
                self.workspace_root.clone(),
            ));
        }

        // Create new state ID
        let state_id = StateId::new();
        debug!("Generated state ID: {:?}", state_id);

        // Capture git information if enabled
        let (git_commit, git_branch) = if self.config.track_git {
            match self.capture_git_info().await {
                Ok((commit, branch)) => (Some(commit), Some(branch)),
                Err(e) => {
                    warnings.push(format!("Failed to capture git info: {}", e));
                    (None, None)
                }
            }
        } else {
            (None, None)
        };

        // Capture files and directories based on method
        let (files, directories) = match self.config.default_capture_method {
            CaptureMethod::FullScan => self.capture_full_scan().await?,
            CaptureMethod::GitBased => self.capture_git_based().await?,
            CaptureMethod::Incremental => self.capture_incremental().await?,
            CaptureMethod::Hybrid => self.capture_hybrid().await?,
        };

        // Calculate totals
        let total_files = files.len();
        let total_size: u64 = files.values().map(|f| f.size).sum();

        // Create capture metadata
        let capture_duration = start_time.elapsed();
        let metadata = CaptureMetadata {
            capture_duration_ms: capture_duration.as_millis() as u64,
            files_processed: total_files,
            directories_processed: directories.len(),
            git_available: git_commit.is_some(),
            warnings: warnings.clone(),
            method: self.config.default_capture_method.clone(),
        };

        // Create workspace state
        let captured_at = chrono::Utc::now();
        let state = WorkspaceState {
            id: state_id,
            captured_at,
            workspace_root: self.workspace_root.clone(),
            git_commit,
            git_branch,
            files,
            directories,
            total_files,
            total_size,
            metadata,
            timestamp: captured_at,
        };

        // Store the state
        self.storage.store_state(&state).await?;

        info!(
            "Successfully captured workspace state {:?} with {} files, {} directories, {} bytes",
            state_id,
            total_files,
            state.directories.len(),
            total_size
        );

        Ok(WorkspaceResult::with_warnings(
            state_id,
            warnings,
            capture_duration.as_millis() as u64,
        ))
    }

    /// Get a stored workspace state
    pub async fn get_state(&self, id: StateId) -> Result<WorkspaceState, WorkspaceError> {
        self.storage.get_state(id).await
    }

    /// List all stored states
    pub async fn list_states(&self) -> Result<Vec<StateId>, WorkspaceError> {
        self.storage.list_states().await
    }

    /// Compute diff between two states
    pub async fn compute_diff(
        &self,
        from_state: StateId,
        to_state: StateId,
    ) -> Result<WorkspaceResult<WorkspaceDiff>, WorkspaceError> {
        let start_time = Instant::now();
        let warnings = Vec::new();

        debug!(
            "Computing diff between states {:?} and {:?}",
            from_state, to_state
        );

        // Get both states
        let from = self.storage.get_state(from_state).await?;
        let to = self.storage.get_state(to_state).await?;

        // Ensure both states are from the same workspace
        if from.workspace_root != to.workspace_root {
            return Err(WorkspaceError::DiffComputation(
                "Cannot diff states from different workspaces".to_string(),
            ));
        }

        // Compute file differences
        let mut added_files = Vec::new();
        let mut removed_files = Vec::new();
        let mut modified_files = Vec::new();

        // Find added and modified files
        for (path, to_file) in &to.files {
            match from.files.get(path) {
                None => added_files.push(path.clone()),
                Some(from_file) => {
                    if from_file.content_hash != to_file.content_hash {
                        modified_files.push(path.clone());
                    }
                }
            }
        }

        // Find removed files
        for path in from.files.keys() {
            if !to.files.contains_key(path) {
                removed_files.push(path.clone());
            }
        }

        // Compute directory differences
        let mut added_directories = Vec::new();
        let mut removed_directories = Vec::new();

        for path in to.directories.keys() {
            if !from.directories.contains_key(path) {
                added_directories.push(path.clone());
            }
        }

        for path in from.directories.keys() {
            if !to.directories.contains_key(path) {
                removed_directories.push(path.clone());
            }
        }

        // Calculate size delta
        let size_delta = to.total_size as i64 - from.total_size as i64;

        // Capture lengths before moving vectors
        let files_added_count = added_files.len();
        let files_removed_count = removed_files.len();
        let files_modified_count = modified_files.len();

        // Create diff
        let diff = WorkspaceDiff {
            from_state,
            to_state,
            added_files,
            removed_files,
            modified_files,
            added_directories,
            removed_directories,
            size_delta,
            files_added: files_added_count,
            files_removed: files_removed_count,
            files_modified: files_modified_count,
            computed_at: chrono::Utc::now(),
            timestamp: chrono::Utc::now(),
            changes: self.collect_actual_diff_changes(&to, &from).await,
        };

        // Store the diff
        self.storage.store_diff(&diff).await?;

        let duration = start_time.elapsed();
        info!(
            "Computed diff: {} added, {} removed, {} modified files",
            diff.files_added, diff.files_removed, diff.files_modified
        );

        Ok(WorkspaceResult::with_warnings(
            diff,
            warnings,
            duration.as_millis() as u64,
        ))
    }

    /// Get diff between two states (from storage if available)
    pub async fn get_diff(
        &self,
        from_state: StateId,
        to_state: StateId,
    ) -> Result<WorkspaceDiff, WorkspaceError> {
        self.storage.get_diff(from_state, to_state).await
    }

    /// Delete a stored state
    pub async fn delete_state(&self, id: StateId) -> Result<(), WorkspaceError> {
        self.storage.delete_state(id).await
    }

    /// Clean up old states based on retention policy
    pub async fn cleanup(&self) -> Result<usize, WorkspaceError> {
        self.storage.cleanup(self.config.max_states).await
    }

    /// Update configuration
    pub fn update_config(&mut self, config: WorkspaceConfig) {
        self.config = config;
    }

    /// Get current configuration
    pub fn config(&self) -> &WorkspaceConfig {
        &self.config
    }

    /// Capture git information
    async fn capture_git_info(&self) -> Result<(String, String), WorkspaceError> {
        use git2::Repository;

        let repo = Repository::open(&self.workspace_root)?;

        // Get current commit
        let head = repo.head()?;
        let commit = head.peel_to_commit()?;
        let commit_hash = commit.id().to_string();

        // Get current branch
        let branch_name = if let Ok(branch_ref) = head.resolve() {
            if let Some(branch_name) = branch_ref.shorthand() {
                branch_name.to_string()
            } else {
                "detached".to_string()
            }
        } else {
            "unknown".to_string()
        };

        Ok((commit_hash, branch_name))
    }

    /// Capture workspace state using full filesystem scan
    async fn capture_full_scan(
        &self,
    ) -> Result<
        (
            HashMap<PathBuf, FileState>,
            HashMap<PathBuf, DirectoryState>,
        ),
        WorkspaceError,
    > {
        use walkdir::WalkDir;

        let mut files = HashMap::new();
        let mut directories = HashMap::new();

        for entry in WalkDir::new(&self.workspace_root)
            .into_iter()
            .filter_entry(|e| !self.should_ignore_path(e.path()))
        {
            let entry = entry.map_err(|e| WorkspaceError::Io(e.into()))?;
            let path = entry
                .path()
                .strip_prefix(&self.workspace_root)
                .map_err(|e| {
                    WorkspaceError::Configuration(format!("Failed to strip prefix: {}", e))
                })?
                .to_path_buf();

            if entry.file_type().is_file() {
                if let Some(file_state) = self.capture_file_state(entry.path(), &path).await? {
                    files.insert(path, file_state);
                }
            } else if entry.file_type().is_dir() {
                if let Some(dir_state) = self.capture_directory_state(entry.path(), &path).await? {
                    directories.insert(path, dir_state);
                }
            }
        }

        Ok((files, directories))
    }

    /// Capture workspace state using git-based approach
    async fn capture_git_based(
        &self,
    ) -> Result<
        (
            HashMap<PathBuf, FileState>,
            HashMap<PathBuf, DirectoryState>,
        ),
        WorkspaceError,
    > {
        use git2::Repository;

        let repo = Repository::open(&self.workspace_root)?;
        let mut files = HashMap::new();
        let mut directories = HashMap::new();

        // Get all tracked files from git
        let index = repo.index()?;
        for entry in index.iter() {
            let path = PathBuf::from(std::str::from_utf8(&entry.path).map_err(|e| {
                WorkspaceError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
            })?);

            let full_path = self.workspace_root.join(&path);
            if full_path.exists() {
                if let Some(file_state) = self.capture_file_state(&full_path, &path).await? {
                    files.insert(path, file_state);
                }
            }
        }

        // Build directory structure from files
        for file_path in files.keys() {
            if let Some(parent) = file_path.parent() {
                let mut current = parent.to_path_buf();
                while !current.as_os_str().is_empty() {
                    if !directories.contains_key(&current) {
                        if let Some(dir_state) = self
                            .capture_directory_state(&self.workspace_root.join(&current), &current)
                            .await?
                        {
                            directories.insert(current.clone(), dir_state);
                        }
                    }
                    current = current.parent().unwrap_or(&PathBuf::new()).to_path_buf();
                }
            }
        }

        Ok((files, directories))
    }

    /// Capture workspace state using incremental approach
    async fn capture_incremental(
        &self,
    ) -> Result<
        (
            HashMap<PathBuf, FileState>,
            HashMap<PathBuf, DirectoryState>,
        ),
        WorkspaceError,
    > {
        // Implement incremental workspace capture using git diff
        self.capture_incremental_git_diff().await
    }

    /// Capture workspace state using hybrid approach
    async fn capture_hybrid(
        &self,
    ) -> Result<
        (
            HashMap<PathBuf, FileState>,
            HashMap<PathBuf, DirectoryState>,
        ),
        WorkspaceError,
    > {
        // Start with git-based approach for tracked files
        let (mut files, directories) = self.capture_git_based().await?;

        // Add untracked files using filesystem scan
        use walkdir::WalkDir;
        for entry in WalkDir::new(&self.workspace_root)
            .into_iter()
            .filter_entry(|e| !self.should_ignore_path(e.path()))
        {
            let entry = entry.map_err(|e| WorkspaceError::Io(e.into()))?;
            let path = entry
                .path()
                .strip_prefix(&self.workspace_root)
                .map_err(|e| {
                    WorkspaceError::Configuration(format!("Failed to strip prefix: {}", e))
                })?
                .to_path_buf();

            if entry.file_type().is_file() && !files.contains_key(&path) {
                if let Some(file_state) = self.capture_file_state(entry.path(), &path).await? {
                    files.insert(path, file_state);
                }
            }
        }

        Ok((files, directories))
    }

    /// Capture state for a single file
    async fn capture_file_state(
        &self,
        full_path: &Path,
        relative_path: &Path,
    ) -> Result<Option<FileState>, WorkspaceError> {
        use sha2::{Digest, Sha256};
        use std::fs;

        let metadata = fs::metadata(full_path)?;

        // Check file size limit
        if metadata.len() > self.config.max_file_size {
            warn!(
                "Skipping large file: {:?} ({} bytes)",
                relative_path,
                metadata.len()
            );
            return Ok(None);
        }

        // Compute content hash if enabled
        let content_hash = if self.config.compute_hashes {
            let content = fs::read(full_path)?;
            let mut hasher = Sha256::new();
            hasher.update(&content);
            format!("{:x}", hasher.finalize())
        } else {
            String::new()
        };

        // Get git information if available
        let (git_tracked, git_commit) = if self.config.track_git {
            self.get_file_git_info(full_path)
                .await
                .unwrap_or((false, None))
        } else {
            (false, None)
        };

        Ok(Some(FileState {
            path: relative_path.to_path_buf(),
            size: metadata.len(),
            content_hash,
            modified_at: metadata.modified()?.into(),
            permissions: 0o644, // Default permissions for cross-platform compatibility
            git_tracked,
            git_commit,
            content: None,     // Content not captured during metadata scan
            compressed: false, // Not compressed initially
        }))
    }

    /// Capture state for a single directory
    async fn capture_directory_state(
        &self,
        full_path: &Path,
        relative_path: &Path,
    ) -> Result<Option<DirectoryState>, WorkspaceError> {
        use std::fs;

        let mut file_count = 0;
        let mut subdirectory_count = 0;
        let mut total_size = 0;
        let mut last_modified = chrono::Utc::now();

        for entry in fs::read_dir(full_path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;

            if metadata.is_file() {
                file_count += 1;
                total_size += metadata.len();
                let modified: DateTime<Utc> = metadata.modified()?.into();
                if modified > last_modified {
                    last_modified = modified;
                }
            } else if metadata.is_dir() {
                subdirectory_count += 1;
            }
        }

        Ok(Some(DirectoryState {
            path: relative_path.to_path_buf(),
            file_count,
            subdirectory_count,
            total_size,
            last_modified,
        }))
    }

    /// Check if a path should be ignored
    fn should_ignore_path(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        for pattern in &self.config.ignore_patterns {
            if glob::Pattern::new(pattern)
                .map(|p| p.matches(&path_str))
                .unwrap_or(false)
            {
                return true;
            }
        }

        false
    }

    /// Get git information for a specific file
    async fn get_file_git_info(
        &self,
        file_path: &Path,
    ) -> Result<(bool, Option<String>), WorkspaceError> {
        use git2::Repository;

        let repo = Repository::open(&self.workspace_root)?;
        let relative_path = file_path
            .strip_prefix(&self.workspace_root)
            .map_err(|e| WorkspaceError::Configuration(format!("Failed to strip prefix: {}", e)))?;

        // Check if file is tracked
        let index = repo.index()?;
        let is_tracked = index.get_path(relative_path, 0).is_some();

        if is_tracked {
            // Get the commit hash for this file
            let head = repo.head()?;
            let commit = head.peel_to_commit()?;
            Ok((true, Some(commit.id().to_string())))
        } else {
            Ok((false, None))
        }
    }

    /// Capture incremental workspace state using git diff analysis
    async fn capture_incremental_git_diff(
        &self,
    ) -> Result<
        (
            HashMap<PathBuf, FileState>,
            HashMap<PathBuf, DirectoryState>,
        ),
        WorkspaceError,
    > {
        let mut file_states = HashMap::new();
        let mut directory_states = HashMap::new();

        // Get the current git repository
        let repo = match git2::Repository::open(&self.workspace_root) {
            Ok(repo) => repo,
            Err(_) => {
                // Fall back to git-based approach if not a git repo
                return self.capture_git_based().await;
            }
        };

        // Get the current HEAD commit
        let head = match repo.head() {
            Ok(head) => head,
            Err(_) => {
                // No HEAD, fall back to git-based approach
                return self.capture_git_based().await;
            }
        };

        let current_commit = match head.peel_to_commit() {
            Ok(commit) => commit,
            Err(_) => {
                return self.capture_git_based().await;
            }
        };

        // Get the previous commit for diff comparison
        let previous_commit = current_commit.parent(0).ok(); // First commit, no previous commit to compare

        // Analyze git diff to identify changes
        let changes = if let Some(prev_commit) = previous_commit {
            self.analyze_git_diff(&repo, &prev_commit, &current_commit)
                .await?
        } else {
            // First commit - capture all files
            self.capture_all_files_in_commit(&repo, &current_commit)
                .await?
        };

        // Process changes and build state
        for change in changes {
            match &change {
                DiffChange::Add { ref path, .. } => {
                    let file_state = self.build_file_state_from_change(&change, &repo).await?;
                    file_states.insert(path.clone(), file_state);
                }
                DiffChange::Modify { ref path, .. } => {
                    let file_state = self.build_file_state_from_change(&change, &repo).await?;
                    file_states.insert(path.clone(), file_state);
                }
                DiffChange::Remove { ref path } => {
                    // Mark file as deleted in state
                    let file_path = path.clone();
                    let file_state = FileState {
                        path: file_path.clone(),
                        size: 0,
                        content_hash: "deleted".to_string(),
                        modified_at: chrono::Utc::now(),
                        permissions: 0,
                        git_tracked: true,
                        git_commit: None, // Would need to get from git
                        content: None,
                        compressed: false,
                    };
                    file_states.insert(file_path, file_state);
                }
                DiffChange::AddDirectory { ref path } => {
                    // Handle directory additions
                    debug!("Directory added: {:?}", path);
                }
                DiffChange::RemoveDirectory { ref path } => {
                    // Handle directory removals
                    debug!("Directory removed: {:?}", path);
                }
            }
        }

        // Build directory states from file states
        self.build_directory_states_from_files(&file_states, &mut directory_states);

        Ok((file_states, directory_states))
    }

    /// Analyze git diff between two commits
    async fn analyze_git_diff(
        &self,
        repo: &git2::Repository,
        from_commit: &git2::Commit<'_>,
        to_commit: &git2::Commit<'_>,
    ) -> Result<Vec<DiffChange>, WorkspaceError> {
        let mut changes = Vec::new();

        // Get diff between commits
        let diff =
            repo.diff_tree_to_tree(Some(&from_commit.tree()?), Some(&to_commit.tree()?), None)?;

        // Process each delta in the diff
        diff.foreach(
            &mut |delta, _| {
                // Skip certain delta types
                match delta.status() {
                    git2::Delta::Ignored => return true,   // Skip ignored files
                    git2::Delta::Untracked => return true, // Skip untracked files
                    _ => {}                                // Continue processing other types
                };

                let new_file_path = delta
                    .new_file()
                    .path()
                    .map(|p| p.to_path_buf())
                    .unwrap_or_else(|| PathBuf::from("unknown"));

                let old_file_path = delta.old_file().path().map(|p| p.to_path_buf());

                let change = match delta.status() {
                    git2::Delta::Added => {
                        let tree = match to_commit.tree() {
                            Ok(t) => t,
                            Err(_) => return true,
                        };
                        let content = self
                            .get_content_from_git_tree(repo, &tree, &new_file_path)
                            .unwrap_or_else(|_| Vec::new());
                        DiffChange::Add {
                            path: new_file_path,
                            content,
                        }
                    }
                    git2::Delta::Deleted => DiffChange::Remove {
                        path: old_file_path.unwrap_or_else(|| new_file_path.clone()),
                    },
                    git2::Delta::Modified => {
                        let from_tree = match from_commit.tree() {
                            Ok(t) => t,
                            Err(_) => return true,
                        };
                        let to_tree = match to_commit.tree() {
                            Ok(t) => t,
                            Err(_) => return true,
                        };
                        let old_content = self
                            .get_content_from_git_tree(
                                repo,
                                &from_tree,
                                &old_file_path.unwrap_or(new_file_path.clone()),
                            )
                            .ok();
                        let new_content = self
                            .get_content_from_git_tree(repo, &to_tree, &new_file_path)
                            .unwrap_or_else(|_| Vec::new());
                        DiffChange::Modify {
                            path: new_file_path,
                            old_content,
                            new_content,
                        }
                    }
                    git2::Delta::Renamed => {
                        let from_tree = match from_commit.tree() {
                            Ok(t) => t,
                            Err(_) => return true,
                        };
                        let to_tree = match to_commit.tree() {
                            Ok(t) => t,
                            Err(_) => return true,
                        };
                        let old_content = self
                            .get_content_from_git_tree(
                                repo,
                                &from_tree,
                                &old_file_path.unwrap_or(new_file_path.clone()),
                            )
                            .ok();
                        let new_content = self
                            .get_content_from_git_tree(repo, &to_tree, &new_file_path)
                            .unwrap_or_else(|_| Vec::new());
                        DiffChange::Modify {
                            path: new_file_path,
                            old_content,
                            new_content,
                        }
                    }
                    _ => {
                        // Handle other delta types (Rename, Copy, Typechange, etc.)
                        // For unclassified deltas, continue processing
                        debug!("Encountered unhandled delta type: {:?}", delta.status());
                        return true;  // Continue iteration
                    }
                };

                changes.push(change);
                true
            },
            None,
            None,
            None,
        )?;

        Ok(changes)
    }

    /// Capture all files in a commit (for first commit scenario)
    async fn capture_all_files_in_commit(
        &self,
        repo: &git2::Repository,
        commit: &git2::Commit<'_>,
    ) -> Result<Vec<DiffChange>, WorkspaceError> {
        let mut changes = Vec::new();
        let tree = commit.tree()?;

        // Walk the tree to find all files
        tree.walk(git2::TreeWalkMode::PreOrder, |root, entry| {
            if let Ok(obj) = entry.to_object(repo) {
                if let Some(blob) = obj.as_blob() {
                    let file_path = PathBuf::from(root).join(entry.name().unwrap_or(""));

                    let change = DiffChange::Add {
                        path: file_path,
                        content: blob.content().to_vec(),
                    };

                    changes.push(change);
                }
            }
            git2::TreeWalkResult::Ok
        })?;

        Ok(changes)
    }

    /// Get the current commit hash from the repository
    fn get_current_commit_hash(
        &self,
        repo: &git2::Repository,
    ) -> Result<Option<String>, WorkspaceError> {
        match repo.head() {
            Ok(head) => match head.peel_to_commit() {
                Ok(commit) => Ok(Some(commit.id().to_string())),
                Err(_) => Ok(None),
            },
            Err(_) => Ok(None), // No HEAD, possibly empty repo
        }
    }

    /// Get content of a file from a git tree
    fn get_content_from_git_tree(
        &self,
        repo: &git2::Repository,
        tree: &git2::Tree,
        path: &Path,
    ) -> Result<Vec<u8>> {
        let entry = tree.get_path(path)?;
        let blob = repo.find_blob(entry.id())?;
        Ok(blob.content().to_vec())
    }

    /// Build file state from a file change
    async fn build_file_state_from_change(
        &self,
        change: &DiffChange,
        repo: &git2::Repository,
    ) -> Result<FileState, WorkspaceError> {
        match change {
            DiffChange::Add { path, content } => {
                let full_path = self.workspace_root.join(path);
                let metadata = std::fs::metadata(&full_path).map_err(WorkspaceError::from)?;
                let size = metadata.len();
                let modified_time = metadata
                    .modified()
                    .map_err(WorkspaceError::from)?
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|e| {
                        WorkspaceError::from(std::io::Error::new(std::io::ErrorKind::Other, e))
                    })?;

                let content_hash = self.calculate_file_hash(&full_path).await?;
                let permissions = if cfg!(unix) {
                    metadata.permissions().mode()
                } else {
                    0o644
                };

                Ok(FileState {
                    path: path.clone(),
                    size,
                    content_hash,
                    modified_at: DateTime::from_timestamp(modified_time.as_secs() as i64, 0)
                        .unwrap_or_default(),
                    permissions,
                    git_tracked: true,
                    git_commit: self.get_current_commit_hash(repo)?,
                    content: Some(content.clone()),
                    compressed: false,
                })
            }
            DiffChange::Modify {
                path, new_content, ..
            } => {
                let full_path = self.workspace_root.join(path);
                let metadata = std::fs::metadata(&full_path).map_err(WorkspaceError::from)?;
                let size = metadata.len();
                let modified_time = metadata
                    .modified()
                    .map_err(WorkspaceError::from)?
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|e| {
                        WorkspaceError::from(std::io::Error::new(std::io::ErrorKind::Other, e))
                    })?;

                let content_hash = self.calculate_file_hash(&full_path).await?;
                let permissions = if cfg!(unix) {
                    metadata.permissions().mode()
                } else {
                    0o644
                };

                Ok(FileState {
                    path: path.clone(),
                    size,
                    content_hash,
                    modified_at: DateTime::from_timestamp(modified_time.as_secs() as i64, 0)
                        .unwrap_or_default(),
                    permissions,
                    git_tracked: true,
                    git_commit: self.get_current_commit_hash(repo)?,
                    content: Some(new_content.clone()),
                    compressed: false,
                })
            }
            DiffChange::Remove { .. } => Err(WorkspaceError::DiffComputation(
                "Cannot build file state from remove change".to_string(),
            )),
            DiffChange::AddDirectory { .. } | DiffChange::RemoveDirectory { .. } => {
                Err(WorkspaceError::DiffComputation(
                    "Cannot build file state from directory change".to_string(),
                ))
            }
        }
    }

    /// Build directory states from file states
    fn build_directory_states_from_files(
        &self,
        file_states: &HashMap<PathBuf, FileState>,
        directory_states: &mut HashMap<PathBuf, DirectoryState>,
    ) {
        for file_state in file_states.values() {
            if let Some(parent) = file_state.path.parent() {
                let dir_path = parent.to_path_buf();

                let dir_state =
                    directory_states
                        .entry(dir_path.clone())
                        .or_insert_with(|| DirectoryState {
                            path: dir_path,
                            file_count: 0,
                            subdirectory_count: 0,
                            total_size: 0,
                            last_modified: file_state.modified_at,
                        });

                dir_state.file_count += 1;
                dir_state.total_size += file_state.size;
                if file_state.modified_at > dir_state.last_modified {
                    dir_state.last_modified = file_state.modified_at;
                }
            }
        }
    }

    /// Calculate SHA-256 hash of a file
    async fn calculate_file_hash(&self, file_path: &Path) -> Result<String, WorkspaceError> {
        use sha2::{Digest, Sha256};
        use std::io::Read;

        let mut file = std::fs::File::open(file_path).map_err(WorkspaceError::from)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0; 8192];

        loop {
            let bytes_read = file.read(&mut buffer).map_err(WorkspaceError::from)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Collect actual diff changes between current and previous states
    async fn collect_actual_diff_changes(
        &self,
        current_state: &WorkspaceState,
        previous_state: &WorkspaceState,
    ) -> Vec<DiffChange> {
        let mut changes = Vec::new();

        // Compare file states to detect changes
        for (file_path, current_file) in &current_state.files {
            if let Some(previous_file) = previous_state.files.get(file_path) {
                if current_file.content_hash != previous_file.content_hash {
                    // File was modified
                    let new_content = current_file.content.clone().unwrap_or_else(|| Vec::new());
                    changes.push(DiffChange::Modify {
                        path: file_path.clone(),
                        old_content: previous_file.content.clone(),
                        new_content,
                    });
                }
            } else {
                // File was added
                let content = current_file.content.clone().unwrap_or_else(|| Vec::new());
                changes.push(DiffChange::Add {
                    path: file_path.clone(),
                    content,
                });
            }
        }

        // Check for deleted files
        for (file_path, _previous_file) in &previous_state.files {
            if !current_state.files.contains_key(file_path) {
                // File was deleted
                changes.push(DiffChange::Remove {
                    path: file_path.clone(),
                });
            }
        }

        changes
    }

}
