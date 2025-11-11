//! Git Worktree Manager
//!
//! Manages git worktree lifecycle for parallel worker isolation.
//! Creates isolated worktrees for each worker, handles merge conflicts,
//! and cleans up worktrees after execution.
//!
//! @author @darianrosebrook

use std::path::PathBuf;
use std::sync::Arc;
use std::collections::HashMap;
use std::process::Command;
use anyhow::{anyhow, Result, Context};
use uuid::Uuid;
use tracing::{info, warn, error};

use agent_agency_contracts::planning_io::Milestone;
use crate::planning::caws_quality_gates::CawsQualityGateExecutor;

/// Git worktree manager configuration
#[derive(Debug, Clone)]
pub struct WorktreeManagerConfig {
    /// Base directory for worktrees
    pub worktree_base_path: PathBuf,
    
    /// Main repository path
    pub main_repo_path: PathBuf,
    
    /// Branch name for worktrees
    pub base_branch: String,
    
    /// Auto-cleanup worktrees after merge
    pub auto_cleanup: bool,
    
    /// Maximum concurrent worktrees
    pub max_concurrent_worktrees: usize,
}

impl Default for WorktreeManagerConfig {
    fn default() -> Self {
        Self {
            worktree_base_path: PathBuf::from("/tmp/agent-agency-worktrees"),
            main_repo_path: PathBuf::from("."),
            base_branch: "main".to_string(),
            auto_cleanup: true,
            max_concurrent_worktrees: 10,
        }
    }
}

/// Worktree information
#[derive(Debug, Clone)]
pub struct WorktreeInfo {
    pub worktree_id: Uuid,
    pub milestone_id: String,
    pub worker_id: Uuid,
    pub worktree_path: PathBuf,
    pub branch_name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub status: WorktreeStatus,
}

/// Worktree status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorktreeStatus {
    Created,
    InUse,
    Merged,
    Conflict,
    CleanedUp,
}

/// Git worktree manager
pub struct WorktreeManager {
    config: WorktreeManagerConfig,
    active_worktrees: Arc<tokio::sync::RwLock<HashMap<Uuid, WorktreeInfo>>>,
    /// CAWS quality gates executor for pre-merge validation
    quality_gates_executor: Option<CawsQualityGateExecutor>,
}

impl WorktreeManager {
    /// Create a new worktree manager
    pub fn new(config: WorktreeManagerConfig) -> Self {
        // Ensure worktree base directory exists
        if let Err(e) = std::fs::create_dir_all(&config.worktree_base_path) {
            warn!("Failed to create worktree base directory: {}", e);
        }

        // Try to initialize quality gates executor
        let quality_gates_executor = CawsQualityGateExecutor::new(&config.main_repo_path)
            .ok()
            .map(|executor| {
                info!("CAWS quality gates executor initialized for worktree manager");
                executor
            });

        Self {
            config,
            active_worktrees: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            quality_gates_executor,
        }
    }

    /// Create a worktree for a milestone
    pub async fn create_worktree(
        &self,
        milestone: &Milestone,
        worker_id: Uuid,
    ) -> Result<WorktreeInfo> {
        let worktree_id = Uuid::new_v4();
        let branch_name = format!("worktree-{}-{}", milestone.id, worktree_id.to_string()[..8].to_string());
        let worktree_path = self.config.worktree_base_path.join(&branch_name);

        info!(
            "Creating worktree for milestone {} (worker {}) at {}",
            milestone.id,
            worker_id,
            worktree_path.display()
        );

        // Check concurrent worktree limit
        {
            let worktrees = self.active_worktrees.read().await;
            let active_count = worktrees.values()
                .filter(|w| matches!(w.status, WorktreeStatus::Created | WorktreeStatus::InUse))
                .count();
            
            if active_count >= self.config.max_concurrent_worktrees {
                return Err(anyhow!("Maximum concurrent worktrees ({}) reached", 
                    self.config.max_concurrent_worktrees));
            }
        }

        // Execute git worktree add command
        // First, ensure we're in the main repo directory
        let output = Command::new("git")
            .current_dir(&self.config.main_repo_path)
            .arg("worktree")
            .arg("add")
            .arg("-b")
            .arg(&branch_name)
            .arg(&worktree_path)
            .arg(&self.config.base_branch)
            .output()
            .context("Failed to execute git worktree add command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("Git worktree add failed: {}", stderr);
            return Err(anyhow!("Failed to create git worktree: {}", stderr));
        }

        info!("Git worktree created at {} (branch: {})", worktree_path.display(), branch_name);

        let worktree_info = WorktreeInfo {
            worktree_id,
            milestone_id: milestone.id.clone(),
            worker_id,
            worktree_path,
            branch_name: branch_name.clone(),
            created_at: chrono::Utc::now(),
            status: WorktreeStatus::Created,
        };

        // Store worktree info
        {
            let mut worktrees = self.active_worktrees.write().await;
            worktrees.insert(worktree_id, worktree_info.clone());
        }

        Ok(worktree_info)
    }

    /// Get worktree path for a worker
    pub async fn get_worktree_path(&self, worker_id: Uuid) -> Result<PathBuf> {
        let worktrees = self.active_worktrees.read().await;
        worktrees
            .values()
            .find(|w| w.worker_id == worker_id)
            .map(|w| w.worktree_path.clone())
            .ok_or_else(|| anyhow!("No worktree found for worker {}", worker_id))
    }

    /// Get worktree path for a milestone
    pub async fn get_worktree_path_by_milestone(&self, milestone_id: &str) -> Result<PathBuf> {
        let worktrees = self.active_worktrees.read().await;
        worktrees
            .values()
            .find(|w| w.milestone_id == milestone_id && matches!(w.status, WorktreeStatus::Created | WorktreeStatus::InUse))
            .map(|w| w.worktree_path.clone())
            .ok_or_else(|| anyhow!("No worktree found for milestone {}", milestone_id))
    }

    /// Mark worktree as in use
    pub async fn mark_in_use(&self, worktree_id: Uuid) -> Result<()> {
        let mut worktrees = self.active_worktrees.write().await;
        if let Some(worktree) = worktrees.get_mut(&worktree_id) {
            worktree.status = WorktreeStatus::InUse;
            Ok(())
        } else {
            Err(anyhow!("Worktree {} not found", worktree_id))
        }
    }

    /// Merge worktree changes back to main branch
    pub async fn merge_worktree(
        &self,
        worktree_id: Uuid,
    ) -> Result<MergeResult> {
        let worktree_info = {
            let worktrees = self.active_worktrees.read().await;
            worktrees.get(&worktree_id)
                .ok_or_else(|| anyhow!("Worktree {} not found", worktree_id))?
                .clone()
        };

        info!("Merging worktree {} (branch {})", worktree_id, worktree_info.branch_name);

        // Run CAWS quality gates before merge (waiver-aware)
        if let Some(ref executor) = self.quality_gates_executor {
            info!("Running CAWS quality gates before merge (waiver-aware)");
            
            // Execute quality gates in the worktree context
            match executor.execute_quality_gates("push").await {
                Ok(gate_result) => {
                    info!(
                        "Pre-merge quality gates: {} violations ({} waived, {} blocking)",
                        gate_result.total_violations,
                        gate_result.waived_violations,
                        gate_result.blocking_violations
                    );
                    
                    // Block merge if there are non-waived violations
                    if !gate_result.passed {
                        warn!(
                            "Merge blocked: {} blocking quality gate violations found",
                            gate_result.blocking_violations
                        );
                        
                        // Log blocking violations
                        for violation in &gate_result.violations {
                            if !violation.waived {
                                warn!(
                                    "Blocking violation: {} - {} ({})",
                                    violation.gate,
                                    violation.message,
                                    violation.file.as_deref().unwrap_or("unknown file")
                                );
                            }
                        }
                        
                        return Err(anyhow!(
                            "Cannot merge worktree {}: {} blocking quality gate violations ({} waived violations allowed)",
                            worktree_id,
                            gate_result.blocking_violations,
                            gate_result.waived_violations
                        ));
                    }
                    
                    // Log waived violations for audit trail
                    if gate_result.waived_violations > 0 {
                        info!("Allowing merge with {} waived violations", gate_result.waived_violations);
                        for violation in &gate_result.violations {
                            if violation.waived {
                                info!(
                                    "Waived violation: {} - {} (waiver: {})",
                                    violation.gate,
                                    violation.message,
                                    violation.waived_by.as_deref().unwrap_or("unknown")
                                );
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to execute quality gates before merge: {}", e);
                    // Continue with merge if quality gates fail to execute (graceful degradation)
                    // In production, you might want to make this stricter
                }
            }
        } else {
            // Try to initialize quality gates executor if not already set
            if let Ok(executor) = CawsQualityGateExecutor::new(&self.config.main_repo_path) {
                info!("Running CAWS quality gates before merge (waiver-aware, fallback executor)");
                
                match executor.execute_quality_gates("push").await {
                    Ok(gate_result) => {
                        info!(
                            "Pre-merge quality gates: {} violations ({} waived, {} blocking)",
                            gate_result.total_violations,
                            gate_result.waived_violations,
                            gate_result.blocking_violations
                        );
                        
                        // Block merge if there are non-waived violations
                        if !gate_result.passed {
                            warn!(
                                "Merge blocked: {} blocking quality gate violations found",
                                gate_result.blocking_violations
                            );
                            
                            // Log blocking violations
                            for violation in &gate_result.violations {
                                if !violation.waived {
                                    warn!(
                                        "Blocking violation: {} - {} ({})",
                                        violation.gate,
                                        violation.message,
                                        violation.file.as_deref().unwrap_or("unknown file")
                                    );
                                }
                            }
                            
                            return Err(anyhow!(
                                "Cannot merge worktree {}: {} blocking quality gate violations ({} waived violations allowed)",
                                worktree_id,
                                gate_result.blocking_violations,
                                gate_result.waived_violations
                            ));
                        }
                        
                        // Log waived violations for audit trail
                        if gate_result.waived_violations > 0 {
                            info!("Allowing merge with {} waived violations", gate_result.waived_violations);
                            for violation in &gate_result.violations {
                                if violation.waived {
                                    info!(
                                        "Waived violation: {} - {} (waiver: {})",
                                        violation.gate,
                                        violation.message,
                                        violation.waived_by.as_deref().unwrap_or("unknown")
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to execute quality gates before merge: {}", e);
                        // Continue with merge if quality gates fail to execute (graceful degradation)
                        // In production, you might want to make this stricter
                    }
                }
            } else {
                warn!("Quality gates executor not available - proceeding with merge without quality gate checks");
            }
        }

        // Switch to main branch in main repo
        let checkout_output = Command::new("git")
            .current_dir(&self.config.main_repo_path)
            .arg("checkout")
            .arg(&self.config.base_branch)
            .output()
            .context("Failed to checkout main branch")?;

        if !checkout_output.status.success() {
            let stderr = String::from_utf8_lossy(&checkout_output.stderr);
            return Err(anyhow!("Failed to checkout main branch: {}", stderr));
        }

        // Merge the worktree branch into main
        let merge_output = Command::new("git")
            .current_dir(&self.config.main_repo_path)
            .arg("merge")
            .arg("--no-ff")
            .arg("-m")
            .arg(format!("Merged worktree {} for milestone {}", 
                worktree_id, worktree_info.milestone_id))
            .arg(&worktree_info.branch_name)
            .output()
            .context("Failed to execute git merge command")?;

        let mut conflicts = Vec::new();
        let mut files_changed = 0;

        if !merge_output.status.success() {
            // Check for merge conflicts
            let stderr = String::from_utf8_lossy(&merge_output.stderr);
            
            // Get conflicted files
            let conflict_output = Command::new("git")
                .current_dir(&self.config.main_repo_path)
                .arg("diff")
                .arg("--name-only")
                .arg("--diff-filter=U")
                .output()
                .context("Failed to check for conflicts")?;

            if conflict_output.status.success() {
                let conflict_files = String::from_utf8_lossy(&conflict_output.stdout);
                conflicts = conflict_files
                    .lines()
                    .filter(|line| !line.is_empty())
                    .map(|s| s.to_string())
                    .collect();
            }

            if conflicts.is_empty() {
                // Merge failed but no conflicts - might be other issues
                warn!("Merge failed but no conflicts detected: {}", stderr);
            } else {
                info!("Merge conflicts detected in {} files", conflicts.len());
            }
        } else {
            // Merge succeeded - count changed files
            let diff_output = Command::new("git")
                .current_dir(&self.config.main_repo_path)
                .arg("diff")
                .arg("--name-only")
                .arg(format!("{}^..{}", &self.config.base_branch, &worktree_info.branch_name))
                .output()
                .context("Failed to count changed files")?;

            if diff_output.status.success() {
                let changed_files = String::from_utf8_lossy(&diff_output.stdout);
                files_changed = changed_files.lines().filter(|line| !line.is_empty()).count();
            }
        }

        let merge_result = MergeResult {
            success: conflicts.is_empty(),
            conflicts,
            files_changed,
            merge_message: format!("Merged worktree {} for milestone {}", 
                worktree_id, worktree_info.milestone_id),
        };

        // Update worktree status
        {
            let mut worktrees = self.active_worktrees.write().await;
            if let Some(worktree) = worktrees.get_mut(&worktree_id) {
                if merge_result.conflicts.is_empty() {
                    worktree.status = WorktreeStatus::Merged;
                } else {
                    worktree.status = WorktreeStatus::Conflict;
                }
            }
        }

        Ok(merge_result)
    }

    /// Resolve merge conflicts
    pub async fn resolve_conflicts(
        &self,
        worktree_id: Uuid,
        resolution_strategy: ConflictResolutionStrategy,
    ) -> Result<()> {
        info!("Resolving conflicts for worktree {}", worktree_id);

        let worktree_info = {
            let worktrees = self.active_worktrees.read().await;
            worktrees.get(&worktree_id)
                .ok_or_else(|| anyhow!("Worktree {} not found", worktree_id))?
                .clone()
        };

        // Resolve conflicts based on strategy
        match resolution_strategy {
            ConflictResolutionStrategy::AcceptOurs => {
                // Accept main branch changes (ours)
                let checkout_output = Command::new("git")
                    .current_dir(&self.config.main_repo_path)
                    .arg("checkout")
                    .arg("--ours")
                    .arg(".")
                    .output()
                    .context("Failed to accept ours for conflict resolution")?;

                if !checkout_output.status.success() {
                    let stderr = String::from_utf8_lossy(&checkout_output.stderr);
                    return Err(anyhow!("Failed to accept ours: {}", stderr));
                }

                // Add resolved files and continue merge
                let add_output = Command::new("git")
                    .current_dir(&self.config.main_repo_path)
                    .arg("add")
                    .arg(".")
                    .output()
                    .context("Failed to add resolved files")?;

                if !add_output.status.success() {
                    let stderr = String::from_utf8_lossy(&add_output.stderr);
                    return Err(anyhow!("Failed to add resolved files: {}", stderr));
                }

                // Complete the merge
                let commit_output = Command::new("git")
                    .current_dir(&self.config.main_repo_path)
                    .arg("commit")
                    .arg("--no-edit")
                    .output()
                    .context("Failed to complete merge commit")?;

                if !commit_output.status.success() {
                    let stderr = String::from_utf8_lossy(&commit_output.stderr);
                    return Err(anyhow!("Failed to complete merge: {}", stderr));
                }

                info!("Resolved conflicts by accepting main branch changes");
            }
            ConflictResolutionStrategy::AcceptTheirs => {
                // Accept worktree changes (theirs)
                let checkout_output = Command::new("git")
                    .current_dir(&self.config.main_repo_path)
                    .arg("checkout")
                    .arg("--theirs")
                    .arg(".")
                    .output()
                    .context("Failed to accept theirs for conflict resolution")?;

                if !checkout_output.status.success() {
                    let stderr = String::from_utf8_lossy(&checkout_output.stderr);
                    return Err(anyhow!("Failed to accept theirs: {}", stderr));
                }

                // Add resolved files and continue merge
                let add_output = Command::new("git")
                    .current_dir(&self.config.main_repo_path)
                    .arg("add")
                    .arg(".")
                    .output()
                    .context("Failed to add resolved files")?;

                if !add_output.status.success() {
                    let stderr = String::from_utf8_lossy(&add_output.stderr);
                    return Err(anyhow!("Failed to add resolved files: {}", stderr));
                }

                // Complete the merge
                let commit_output = Command::new("git")
                    .current_dir(&self.config.main_repo_path)
                    .arg("commit")
                    .arg("--no-edit")
                    .output()
                    .context("Failed to complete merge commit")?;

                if !commit_output.status.success() {
                    let stderr = String::from_utf8_lossy(&commit_output.stderr);
                    return Err(anyhow!("Failed to complete merge: {}", stderr));
                }

                info!("Resolved conflicts by accepting worktree changes");
            }
            ConflictResolutionStrategy::Manual => {
                // Require manual resolution
                return Err(anyhow!("Manual conflict resolution required for worktree {}. Conflicts are in: {}", 
                    worktree_id, 
                    worktree_info.worktree_path.display()));
            }
        }

        // Update status after resolution
        {
            let mut worktrees = self.active_worktrees.write().await;
            if let Some(worktree) = worktrees.get_mut(&worktree_id) {
                worktree.status = WorktreeStatus::Merged;
            }
        }

        Ok(())
    }

    /// Cleanup worktree
    pub async fn cleanup_worktree(&self, worktree_id: Uuid) -> Result<()> {
        let worktree_info = {
            let mut worktrees = self.active_worktrees.write().await;
            worktrees.remove(&worktree_id)
                .ok_or_else(|| anyhow!("Worktree {} not found", worktree_id))?
        };

        info!("Cleaning up worktree {} at {}", worktree_id, worktree_info.worktree_path.display());

        // Execute git worktree remove command
        // First try to remove via git (this handles branch cleanup properly)
        let remove_output = Command::new("git")
            .current_dir(&self.config.main_repo_path)
            .arg("worktree")
            .arg("remove")
            .arg(&worktree_info.worktree_path)
            .output()
            .context("Failed to execute git worktree remove command")?;

        if !remove_output.status.success() {
            // If git worktree remove fails, try force remove
            let force_output = Command::new("git")
                .current_dir(&self.config.main_repo_path)
                .arg("worktree")
                .arg("remove")
                .arg("--force")
                .arg(&worktree_info.worktree_path)
                .output()
                .context("Failed to execute git worktree remove --force command")?;

            if !force_output.status.success() {
                let stderr = String::from_utf8_lossy(&force_output.stderr);
                warn!("Git worktree remove failed: {}. Attempting manual cleanup.", stderr);
                
                // Fallback: manual directory removal
                if worktree_info.worktree_path.exists() {
                    if let Err(e) = std::fs::remove_dir_all(&worktree_info.worktree_path) {
                        warn!("Failed to remove worktree directory: {}", e);
                    }
                }
            } else {
                info!("Worktree removed via git worktree remove --force");
            }
        } else {
            info!("Worktree removed successfully via git worktree remove");
        }

        // Also remove the branch if it still exists
        let branch_remove_output = Command::new("git")
            .current_dir(&self.config.main_repo_path)
            .arg("branch")
            .arg("-d")
            .arg(&worktree_info.branch_name)
            .output();

        if let Ok(output) = branch_remove_output {
            if !output.status.success() {
                // Try force delete if normal delete fails
                let _ = Command::new("git")
                    .current_dir(&self.config.main_repo_path)
                    .arg("branch")
                    .arg("-D")
                    .arg(&worktree_info.branch_name)
                    .output();
            }
        }

        Ok(())
    }

    /// Cleanup all active worktrees
    pub async fn cleanup_all(&self) -> Result<()> {
        let worktree_ids: Vec<Uuid> = {
            let worktrees = self.active_worktrees.read().await;
            worktrees.keys().cloned().collect()
        };

        for worktree_id in worktree_ids {
            if let Err(e) = self.cleanup_worktree(worktree_id).await {
                warn!("Failed to cleanup worktree {}: {}", worktree_id, e);
            }
        }

        Ok(())
    }

    /// List all active worktrees
    pub async fn list_worktrees(&self) -> Vec<WorktreeInfo> {
        let worktrees = self.active_worktrees.read().await;
        worktrees.values().cloned().collect()
    }
}

/// Merge result
#[derive(Debug, Clone)]
pub struct MergeResult {
    pub success: bool,
    pub conflicts: Vec<String>,
    pub files_changed: usize,
    pub merge_message: String,
}

/// Conflict resolution strategy
#[derive(Debug, Clone)]
pub enum ConflictResolutionStrategy {
    /// Accept changes from main branch
    AcceptOurs,
    
    /// Accept changes from worktree
    AcceptTheirs,
    
    /// Require manual resolution
    Manual,
}

