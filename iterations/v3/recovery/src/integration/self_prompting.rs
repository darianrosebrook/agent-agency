use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::types::{Digest, ChangeId, ChangeSource, ConflictClass, SessionMeta, SessionRef, Commit};
use crate::cas::{ConcurrencyManager, ConcurrencyResult, ConflictInfo, ConflictResolution};
use crate::merkle::{Commit as MerkleCommit, FileTree as MerkleTree, AuthorInfo};
use crate::journal::WriteAheadLog;
use crate::policy::{CawsPolicy, PolicyEnforcer};

/// Integration between recovery system and self-prompting loop
pub struct SelfPromptingRecovery {
    /// Concurrency manager for handling conflicts
    concurrency_manager: ConcurrencyManager,
    /// Policy enforcer for CAWS compliance
    policy_enforcer: PolicyEnforcer,
    /// Write-ahead log for crash safety
    wal: WriteAheadLog,
    /// Current session
    current_session: Option<SessionRef>,
    /// Session metadata
    session_metadata: HashMap<String, SessionMeta>,
    /// Configuration
    config: RecoveryConfig,
}

/// Recovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryConfig {
    /// Enable automatic checkpointing
    pub auto_checkpoint: bool,
    /// Checkpoint frequency (iterations)
    pub checkpoint_frequency: u32,
    /// Enable conflict resolution
    pub enable_conflict_resolution: bool,
    /// Default conflict resolution strategy
    pub default_conflict_resolution: ConflictResolution,
    /// Enable recovery logging
    pub enable_logging: bool,
    /// Recovery directory
    pub recovery_dir: PathBuf,
    /// Session timeout (seconds)
    pub session_timeout: u64,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            auto_checkpoint: true,
            checkpoint_frequency: 10, // Checkpoint every 10 iterations
            enable_conflict_resolution: true,
            default_conflict_resolution: ConflictResolution::AutoMerge,
            enable_logging: true,
            recovery_dir: PathBuf::from(".recovery"),
            session_timeout: 3600, // 1 hour
        }
    }
}

impl SelfPromptingRecovery {
    /// Create a new self-prompting recovery integration
    pub fn new(config: RecoveryConfig) -> Result<Self> {
        let concurrency_manager = ConcurrencyManager::new();
        let policy_enforcer = PolicyEnforcer::new(CawsPolicy::new());
        let wal = WriteAheadLog::new(config.recovery_dir.join("journal.wal"))?;
        
        Ok(Self {
            concurrency_manager,
            policy_enforcer,
            wal,
            current_session: None,
            session_metadata: HashMap::new(),
            config,
        })
    }

    /// Start a new recovery session
    pub fn start_session(&mut self, session_meta: SessionMeta) -> Result<SessionRef> {
        let session_id = self.generate_session_id();
        let session_ref = SessionRef {
            id: session_id.clone(),
            meta: session_meta.clone(),
            created_at: chrono::Utc::now(),
        };

        // Check if session creation is allowed
        match self.policy_enforcer.check_session_creation(&session_id)? {
            crate::policy::SessionCheckResult::Allowed => {
                // TODO: Add session tracking to concurrency manager
                // For now, we'll track sessions separately

                // Store session metadata
                self.session_metadata.insert(session_id.clone(), session_meta);
                self.current_session = Some(session_ref.clone());

                if self.config.enable_logging {
                    println!("Started recovery session: {}", session_id);
                }

                Ok(session_ref)
            }
            crate::policy::SessionCheckResult::Rejected(reason) => {
                Err(anyhow!("Session creation rejected: {:?}", reason))
            }
        }
    }

    /// End the current recovery session
    pub fn end_session(&mut self) -> Result<()> {
        if let Some(session_ref) = self.current_session.take() {
            // Check if session deletion is allowed
            match self.policy_enforcer.check_session_deletion(&session_ref.id)? {
                crate::policy::SessionDeletionCheckResult::Allowed => {
                    // Remove session from concurrency manager
                    // TODO: Remove session from concurrency manager
                    
                    if self.config.enable_logging {
                        println!("Ended recovery session: {}", session_ref.id);
                    }
                    
                    Ok(())
                }
                crate::policy::SessionDeletionCheckResult::Rejected(reason) => {
                    Err(anyhow!("Session deletion rejected: {:?}", reason))
                }
            }
        } else {
            Err(anyhow!("No active session to end"))
        }
    }

    /// Record a file change during self-prompting iteration
    pub fn record_change(
        &mut self,
        path: &str,
        content: &[u8],
        change_source: ChangeSource,
    ) -> Result<ChangeResult> {
        let session_ref = self.current_session.as_ref()
            .ok_or_else(|| anyhow!("No active session"))?;

        // Compute content digest
        let digest = self.compute_content_digest(content);
        
        // Get precondition digest for optimistic concurrency
        let precondition = self.get_file_precondition(path)?;
        
        // Record change with concurrency control
        match self.concurrency_manager.record_change(
            path,
            digest,
            precondition,
            change_source,
            &session_ref.id,
            session_ref.meta.agent_id.as_deref(),
        )? {
            ConcurrencyResult::Success => {
                // Change recorded successfully
                if self.config.enable_logging {
                    println!("Recorded change for {} in session {}", path, session_ref.id);
                }
                
                Ok(ChangeResult::Success)
            }
            ConcurrencyResult::Conflict(conflict) => {
                // Handle conflict
                if self.config.enable_conflict_resolution {
                    self.handle_conflict(conflict)
                } else {
                    Ok(ChangeResult::Conflict(conflict))
                }
            }
            ConcurrencyResult::Rejected => {
                Ok(ChangeResult::Rejected)
            }
            ConcurrencyResult::Branched(branch_name) => {
                Ok(ChangeResult::Branched(branch_name))
            }
        }
    }

    /// Create a checkpoint
    pub fn create_checkpoint(&mut self, label: Option<String>) -> Result<CheckpointResult> {
        let session_ref = self.current_session.as_ref()
            .ok_or_else(|| anyhow!("No active session"))?;

        // Create commit from current state
        let commit = self.create_commit_from_session(session_ref, label)?;
        
        // Write to WAL
        self.wal.record_commit(&ChangeId(commit.id.to_string()), commit.tree())?;

        if self.config.enable_logging {
            println!("Created checkpoint: {} for session {}", commit.id, session_ref.id);
        }

        Ok(CheckpointResult {
            commit_id: commit.id.to_string(),
            timestamp: chrono::Utc::now().timestamp() as u64,
            session_id: session_ref.id.clone(),
        })
    }

    /// Auto-checkpoint if needed
    pub fn auto_checkpoint_if_needed(&mut self, iteration: u32) -> Result<Option<CheckpointResult>> {
        if !self.config.auto_checkpoint {
            return Ok(None);
        }

        if iteration % self.config.checkpoint_frequency == 0 {
            Ok(Some(self.create_checkpoint(Some(format!("auto-checkpoint-{}", iteration)))?))
        } else {
            Ok(None)
        }
    }

    /// Handle a conflict
    fn handle_conflict(&mut self, conflict: ConflictInfo) -> Result<ChangeResult> {
        match self.config.default_conflict_resolution {
            ConflictResolution::AutoMerge => {
                // Attempt automatic merge
                self.auto_merge_conflict(&conflict)
            }
            ConflictResolution::Manual => {
                // Return conflict for manual resolution
                Ok(ChangeResult::Conflict(conflict))
            }
            ConflictResolution::Reject => {
                // Reject the change
                Ok(ChangeResult::Rejected)
            }
            ConflictResolution::Branch => {
                // Create a branch
                self.create_conflict_branch(&conflict)
            }
            ConflictResolution::UseNewer => {
                // Use the newer change
                self.use_newer_change(&conflict)
            }
            ConflictResolution::UseOlder => {
                // Use the older change
                self.use_older_change(&conflict)
            }
        }
    }

    /// Attempt automatic merge
    fn auto_merge_conflict(&mut self, conflict: &ConflictInfo) -> Result<ChangeResult> {
        // TODO: Implement automatic merge logic
        // For now, return conflict for manual resolution
        Ok(ChangeResult::Conflict(conflict.clone()))
    }

    /// Create a conflict branch
    fn create_conflict_branch(&mut self, conflict: &ConflictInfo) -> Result<ChangeResult> {
        let branch_name = format!("conflict-{}-{}", conflict.timestamp, conflict.conflicting_session);
        Ok(ChangeResult::Branched(branch_name))
    }

    /// Use the newer change
    fn use_newer_change(&mut self, conflict: &ConflictInfo) -> Result<ChangeResult> {
        // Use the current digest (newer)
        Ok(ChangeResult::Success)
    }

    /// Use the older change
    fn use_older_change(&mut self, conflict: &ConflictInfo) -> Result<ChangeResult> {
        // Use the base digest (older)
        Ok(ChangeResult::Success)
    }

    /// Compute content digest
    fn compute_content_digest(&self, content: &[u8]) -> Digest {
        use crate::types::StreamingHasher;
        let mut hasher = StreamingHasher::new();
        hasher.update(content);
        hasher.finalize()
    }

    /// Get file precondition for optimistic concurrency
    fn get_file_precondition(&self, path: &str) -> Result<Option<Digest>> {
        // TODO: Implement based on your file state tracking
        Ok(None)
    }

    /// Create commit from session
    fn create_commit_from_session(&self, session_ref: &SessionRef, label: Option<String>) -> Result<MerkleCommit> {
        // TODO: Implement commit creation from session state
        // This would involve creating a Merkle tree from the current file state
        let tree = MerkleTree::empty(); // Placeholder
        let commit = MerkleCommit::new(
            Some(Digest::from_bytes([0; 32])), // Parent commit
            tree.digest(),
            session_ref.id.clone(),
            AuthorInfo {
                name: session_ref.meta.agent_id.clone().unwrap_or_default(),
                email: "agent@system".to_string(),
                agent_id: Some(session_ref.meta.agent_id.clone().unwrap_or_default()),
            },
            label,
            crate::types::ChangeStats::default(),
        );
        Ok(commit)
    }

    /// Generate session ID
    fn generate_session_id(&self) -> String {
        use uuid::Uuid;
        format!("session-{}", Uuid::new_v4())
    }

    /// Get current timestamp
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// Get recovery statistics
    pub fn get_stats(&self) -> RecoveryStats {
        RecoveryStats {
            active_session: self.current_session.is_some(),
            total_sessions: self.session_metadata.len(),
            conflicts_resolved: 0, // TODO: Track conflicts
            checkpoints_created: 0, // TODO: Track checkpoints
            last_checkpoint: None,
        }
    }

    /// Get current session
    pub fn get_current_session(&self) -> Option<&SessionRef> {
        self.current_session.as_ref()
    }

    /// Get session metadata
    pub fn get_session_metadata(&self, session_id: &str) -> Option<&SessionMeta> {
        self.session_metadata.get(session_id)
    }
}

/// Result of a change operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeResult {
    /// Change was successful
    Success,
    /// Conflict detected
    Conflict(ConflictInfo),
    /// Change was rejected
    Rejected,
    /// Branch was created
    Branched(String),
}

/// Result of a checkpoint operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointResult {
    /// Commit ID
    pub commit_id: String,
    /// Timestamp
    pub timestamp: u64,
    /// Session ID
    pub session_id: String,
}

/// Recovery statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryStats {
    /// Whether there's an active session
    pub active_session: bool,
    /// Total number of sessions
    pub total_sessions: usize,
    /// Number of conflicts resolved
    pub conflicts_resolved: usize,
    /// Number of checkpoints created
    pub checkpoints_created: usize,
    /// Last checkpoint timestamp
    pub last_checkpoint: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recovery_integration() {
        let config = RecoveryConfig::default();
        let recovery = SelfPromptingRecovery::new(config).unwrap();
        
        assert!(!recovery.get_current_session().is_some());
        assert_eq!(recovery.get_stats().total_sessions, 0);
    }

    #[test]
    fn test_session_management() {
        let config = RecoveryConfig::default();
        let mut recovery = SelfPromptingRecovery::new(config).unwrap();
        
        let session_meta = SessionMeta {
            agent_id: Some("agent1".to_string()),
            iteration: 1,
            labels: vec!["test".to_string()],
            protected: false,
        };
        
        let session_ref = recovery.start_session(session_meta).unwrap();
        assert_eq!(session_ref.agent_id, Some("agent1".to_string()));
        assert_eq!(session_ref.iteration, 1);
        
        recovery.end_session().unwrap();
        assert!(!recovery.get_current_session().is_some());
    }

    #[test]
    fn test_change_recording() {
        let config = RecoveryConfig::default();
        let mut recovery = SelfPromptingRecovery::new(config).unwrap();
        
        let session_meta = SessionMeta {
            agent_id: Some("agent1".to_string()),
            iteration: 1,
            labels: vec!["test".to_string()],
            protected: false,
        };
        
        recovery.start_session(session_meta).unwrap();
        
        let content = b"Hello, world!";
        let change_source = ChangeSource::AgentIteration {
            iteration: 1,
            agent_id: "agent1".to_string(),
        };
        
        let result = recovery.record_change("test.txt", content, change_source).unwrap();
        assert!(matches!(result, ChangeResult::Success));
    }

    #[test]
    fn test_checkpoint_creation() {
        let config = RecoveryConfig::default();
        let mut recovery = SelfPromptingRecovery::new(config).unwrap();
        
        let session_meta = SessionMeta {
            agent_id: Some("agent1".to_string()),
            iteration: 1,
            labels: vec!["test".to_string()],
            protected: false,
        };
        
        recovery.start_session(session_meta).unwrap();
        
        let result = recovery.create_checkpoint(Some("test-checkpoint".to_string())).unwrap();
        assert_eq!(result.session_id, recovery.get_current_session().unwrap().session_id);
    }
}
