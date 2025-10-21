use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::types::{Digest, ChangeSource, ConflictClass};
use crate::types::Digest as SourceDigest;

/// Optimistic concurrency control for file changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcurrencyControl {
    /// Precondition digest for optimistic concurrency
    pub precondition: Option<Digest>,
    /// Change source information
    pub source: ChangeSource,
    /// Timestamp when change was made
    pub timestamp: u64,
    /// Session ID for tracking
    pub session_id: String,
    /// Agent ID (if applicable)
    pub agent_id: Option<String>,
}

/// Conflict information when precondition fails
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictInfo {
    /// Conflict class
    pub class: ConflictClass,
    /// Base digest (what was expected)
    pub base_digest: Digest,
    /// Current digest (what actually exists)
    pub current_digest: Digest,
    /// Conflict timestamp
    pub timestamp: u64,
    /// Conflicting session ID
    pub conflicting_session: String,
    /// Conflict resolution strategy
    pub resolution_strategy: ConflictResolution,
}

/// Conflict resolution strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolution {
    /// Automatically resolve using merge
    AutoMerge,
    /// Require manual resolution
    Manual,
    /// Reject the change
    Reject,
    /// Create a branch
    Branch,
    /// Use the newer change
    UseNewer,
    /// Use the older change
    UseOlder,
}

/// Concurrency manager for handling optimistic concurrency
pub struct ConcurrencyManager {
    /// Current file states (path -> digest)
    file_states: HashMap<String, Digest>,
    /// Pending changes (path -> change info)
    pending_changes: HashMap<String, ConcurrencyControl>,
    /// Conflict history
    conflict_history: Vec<ConflictInfo>,
    /// Configuration
    config: ConcurrencyConfig,
}

/// Concurrency configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcurrencyConfig {
    /// Maximum number of pending changes
    pub max_pending_changes: usize,
    /// Conflict resolution timeout (seconds)
    pub conflict_timeout: u64,
    /// Default conflict resolution strategy
    pub default_resolution: ConflictResolution,
    /// Enable automatic conflict resolution
    pub auto_resolve: bool,
    /// Enable conflict logging
    pub log_conflicts: bool,
}

impl Default for ConcurrencyConfig {
    fn default() -> Self {
        Self {
            max_pending_changes: 1000,
            conflict_timeout: 300, // 5 minutes
            default_resolution: ConflictResolution::Manual,
            auto_resolve: false,
            log_conflicts: true,
        }
    }
}

impl ConcurrencyManager {
    /// Create a new concurrency manager
    pub fn new() -> Self {
        Self {
            file_states: HashMap::new(),
            pending_changes: HashMap::new(),
            conflict_history: Vec::new(),
            config: ConcurrencyConfig::default(),
        }
    }

    /// Create a new concurrency manager with custom configuration
    pub fn with_config(config: ConcurrencyConfig) -> Self {
        Self {
            file_states: HashMap::new(),
            pending_changes: HashMap::new(),
            conflict_history: Vec::new(),
            config,
        }
    }

    /// Attempt to record a file change with optimistic concurrency
    pub fn record_change(
        &mut self,
        path: &str,
        new_digest: Digest,
        precondition: Option<Digest>,
        source: ChangeSource,
        session_id: &str,
        agent_id: Option<&str>,
    ) -> Result<ConcurrencyResult> {
        // Check if we have too many pending changes
        if self.pending_changes.len() >= self.config.max_pending_changes {
            return Err(anyhow!("Too many pending changes"));
        }

        // Check precondition
        let current_digest = self.file_states.get(path).copied();
        if let Some(precond) = precondition {
            if current_digest != Some(precond) {
                // Precondition failed - conflict detected
                let conflict = ConflictInfo {
                    class: self.classify_conflict(&source, current_digest),
                    base_digest: precond,
                    current_digest: current_digest.unwrap_or_else(|| Digest::from_bytes(&[])),
                    timestamp: self.current_timestamp(),
                    conflicting_session: session_id.to_string(),
                    resolution_strategy: self.config.default_resolution.clone(),
                };

                if self.config.log_conflicts {
                    self.conflict_history.push(conflict.clone());
                }

                return Ok(ConcurrencyResult::Conflict(conflict));
            }
        }

        // Record the change
        let control = ConcurrencyControl {
            precondition,
            source,
            timestamp: self.current_timestamp(),
            session_id: session_id.to_string(),
            agent_id: agent_id.map(|s| s.to_string()),
        };

        self.pending_changes.insert(path.to_string(), control);
        self.file_states.insert(path.to_string(), new_digest);

        Ok(ConcurrencyResult::Success)
    }

    /// Commit a pending change
    pub fn commit_change(&mut self, path: &str) -> Result<()> {
        if let Some(control) = self.pending_changes.remove(path) {
            // Change is now committed
            Ok(())
        } else {
            Err(anyhow!("No pending change for path: {}", path))
        }
    }

    /// Rollback a pending change
    pub fn rollback_change(&mut self, path: &str) -> Result<()> {
        if let Some(control) = self.pending_changes.remove(path) {
            // Restore previous state if we have a precondition
            if let Some(precond) = control.precondition {
                self.file_states.insert(path.to_string(), precond);
            } else {
                self.file_states.remove(path);
            }
            Ok(())
        } else {
            Err(anyhow!("No pending change for path: {}", path))
        }
    }

    /// Resolve a conflict
    pub fn resolve_conflict(
        &mut self,
        path: &str,
        conflict: &ConflictInfo,
        resolution: ConflictResolution,
    ) -> Result<ConcurrencyResult> {
        match resolution {
            ConflictResolution::AutoMerge => {
                // Attempt automatic merge
                self.auto_merge_conflict(path, conflict)
            }
            ConflictResolution::Manual => {
                // Return conflict for manual resolution
                Ok(ConcurrencyResult::Conflict(conflict.clone()))
            }
            ConflictResolution::Reject => {
                // Reject the change
                Ok(ConcurrencyResult::Rejected)
            }
            ConflictResolution::Branch => {
                // Create a branch
                self.create_branch(path, conflict)
            }
            ConflictResolution::UseNewer => {
                // Use the newer change
                self.use_newer_change(path, conflict)
            }
            ConflictResolution::UseOlder => {
                // Use the older change
                self.use_older_change(path, conflict)
            }
        }
    }

    /// Get current file state
    pub fn get_file_state(&self, path: &str) -> Option<&Digest> {
        self.file_states.get(path)
    }

    /// Get pending changes
    pub fn get_pending_changes(&self) -> &HashMap<String, ConcurrencyControl> {
        &self.pending_changes
    }

    /// Get conflict history
    pub fn get_conflict_history(&self) -> &[ConflictInfo] {
        &self.conflict_history
    }

    /// Clear conflict history
    pub fn clear_conflict_history(&mut self) {
        self.conflict_history.clear();
    }

    /// Get concurrency statistics
    pub fn get_stats(&self) -> ConcurrencyStats {
        ConcurrencyStats {
            total_files: self.file_states.len(),
            pending_changes: self.pending_changes.len(),
            total_conflicts: self.conflict_history.len(),
            recent_conflicts: self.conflict_history
                .iter()
                .filter(|c| self.current_timestamp() - c.timestamp < 3600) // Last hour
                .count(),
        }
    }

    /// Classify a conflict based on source and current state
    fn classify_conflict(&self, source: &ChangeSource, current_digest: Option<Digest>) -> ConflictClass {
        match source {
            ChangeSource::AgentIteration { .. } => {
                if current_digest.is_some() {
                    ConflictClass::AgentVsAgent
                } else {
                    ConflictClass::AgentVsSystem
                }
            }
            ChangeSource::HumanEdit { .. } => {
                if current_digest.is_some() {
                    ConflictClass::HumanVsAgent
                } else {
                    ConflictClass::HumanVsSystem
                }
            }
            ChangeSource::SystemRecovery { .. } => ConflictClass::SystemVsSystem,
            ChangeSource::CawsValidation { .. } => ConflictClass::ValidationVsSystem,
        }
    }

    /// Attempt automatic merge
    fn auto_merge_conflict(&mut self, path: &str, conflict: &ConflictInfo) -> Result<ConcurrencyResult> {
        // For now, just return the conflict for manual resolution
        // In a real implementation, you'd implement merge logic here
        Ok(ConcurrencyResult::Conflict(conflict.clone()))
    }

    /// Create a branch for conflict resolution
    fn create_branch(&mut self, path: &str, conflict: &ConflictInfo) -> Result<ConcurrencyResult> {
        // Create a new branch with the conflicting change
        let branch_name = format!("conflict-{}-{}", path.replace('/', "-"), conflict.timestamp);
        // In a real implementation, you'd create the branch here
        Ok(ConcurrencyResult::Branched(branch_name))
    }

    /// Use the newer change
    fn use_newer_change(&mut self, path: &str, conflict: &ConflictInfo) -> Result<ConcurrencyResult> {
        // Use the current digest (newer)
        self.file_states.insert(path.to_string(), conflict.current_digest);
        Ok(ConcurrencyResult::Success)
    }

    /// Use the older change
    fn use_older_change(&mut self, path: &str, conflict: &ConflictInfo) -> Result<ConcurrencyResult> {
        // Use the base digest (older)
        self.file_states.insert(path.to_string(), conflict.base_digest);
        Ok(ConcurrencyResult::Success)
    }

    /// Get current timestamp
    fn current_timestamp(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

/// Result of a concurrency operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConcurrencyResult {
    /// Operation succeeded
    Success,
    /// Conflict detected
    Conflict(ConflictInfo),
    /// Change was rejected
    Rejected,
    /// Branch was created
    Branched(String),
}

/// Concurrency statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcurrencyStats {
    /// Total number of tracked files
    pub total_files: usize,
    /// Number of pending changes
    pub pending_changes: usize,
    /// Total number of conflicts
    pub total_conflicts: usize,
    /// Number of recent conflicts (last hour)
    pub recent_conflicts: usize,
}

/// Conflict detector for identifying potential conflicts
pub struct ConflictDetector {
    /// Conflict patterns
    patterns: Vec<ConflictPattern>,
}

/// Conflict pattern for detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictPattern {
    /// Pattern name
    pub name: String,
    /// File pattern to match
    pub file_pattern: String,
    /// Conflict class
    pub class: ConflictClass,
    /// Resolution strategy
    pub resolution: ConflictResolution,
}

impl ConflictDetector {
    /// Create a new conflict detector
    pub fn new() -> Self {
        Self {
            patterns: Vec::new(),
        }
    }

    /// Add a conflict pattern
    pub fn add_pattern(&mut self, pattern: ConflictPattern) {
        self.patterns.push(pattern);
    }

    /// Detect potential conflicts
    pub fn detect_conflicts(&self, path: &str, source: &ChangeSource) -> Vec<ConflictPattern> {
        self.patterns
            .iter()
            .filter(|pattern| {
                // Simple pattern matching - in practice, you'd use a proper glob library
                self.matches_pattern(path, &pattern.file_pattern)
            })
            .cloned()
            .collect()
    }

    /// Check if a path matches a pattern
    fn matches_pattern(&self, path: &str, pattern: &str) -> bool {
        if pattern.contains('*') {
            let pattern_parts: Vec<&str> = pattern.split('*').collect();
            if pattern_parts.len() == 2 {
                let prefix = pattern_parts[0];
                let suffix = pattern_parts[1];
                path.starts_with(prefix) && path.ends_with(suffix)
            } else {
                false
            }
        } else {
            path == pattern
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimistic_concurrency_success() {
        let mut manager = ConcurrencyManager::new();
        let digest = Digest::from_bytes(&[1, 2, 3, 4]);
        
        let result = manager.record_change(
            "test.txt",
            digest,
            None, // No precondition
            ChangeSource::AgentIteration {
                iteration: 1,
                agent_id: "agent1".to_string(),
            },
            "session1",
            Some("agent1"),
        ).unwrap();
        
        assert!(matches!(result, ConcurrencyResult::Success));
        assert!(manager.get_file_state("test.txt").is_some());
    }

    #[test]
    fn test_optimistic_concurrency_conflict() {
        let mut manager = ConcurrencyManager::new();
        let digest1 = Digest::from_bytes(&[1, 2, 3, 4]);
        let digest2 = Digest::from_bytes(&[5, 6, 7, 8]);
        
        // First change succeeds
        let result1 = manager.record_change(
            "test.txt",
            digest1,
            None,
            ChangeSource::AgentIteration {
                iteration: 1,
                agent_id: "agent1".to_string(),
            },
            "session1",
            Some("agent1"),
        ).unwrap();
        assert!(matches!(result1, ConcurrencyResult::Success));
        
        // Second change with precondition fails
        let result2 = manager.record_change(
            "test.txt",
            digest2,
            Some(digest1), // Expecting digest1
            ChangeSource::AgentIteration {
                iteration: 2,
                agent_id: "agent2".to_string(),
            },
            "session2",
            Some("agent2"),
        ).unwrap();
        
        assert!(matches!(result2, ConcurrencyResult::Conflict(_)));
    }

    #[test]
    fn test_conflict_classification() {
        let mut manager = ConcurrencyManager::new();
        let digest = Digest::from_bytes(&[1, 2, 3, 4]);
        
        // Set up a file state
        manager.file_states.insert("test.txt".to_string(), digest);
        
        // Test different conflict types
        let agent_source = ChangeSource::AgentIteration {
            iteration: 1,
            agent_id: "agent1".to_string(),
        };
        
        let human_source = ChangeSource::HumanEdit {
            user_id: "user1".to_string(),
        };
        
        // Agent vs agent conflict
        let result = manager.record_change(
            "test.txt",
            Digest::from_bytes(&[5, 6, 7, 8]),
            Some(digest),
            agent_source,
            "session1",
            Some("agent1"),
        ).unwrap();
        
        if let ConcurrencyResult::Conflict(conflict) = result {
            assert_eq!(conflict.class, ConflictClass::AgentVsAgent);
        } else {
            panic!("Expected conflict");
        }
    }

    #[test]
    fn test_concurrency_stats() {
        let mut manager = ConcurrencyManager::new();
        
        // Add some changes
        manager.record_change(
            "test1.txt",
            Digest::from_bytes(&[1, 2, 3, 4]),
            None,
            ChangeSource::AgentIteration {
                iteration: 1,
                agent_id: "agent1".to_string(),
            },
            "session1",
            Some("agent1"),
        ).unwrap();
        
        manager.record_change(
            "test2.txt",
            Digest::from_bytes(&[5, 6, 7, 8]),
            None,
            ChangeSource::HumanEdit {
                user_id: "user1".to_string(),
            },
            "session2",
            None,
        ).unwrap();
        
        let stats = manager.get_stats();
        assert_eq!(stats.total_files, 2);
        assert_eq!(stats.pending_changes, 2);
    }

    #[test]
    fn test_conflict_detector() {
        let mut detector = ConflictDetector::new();
        
        // Add a pattern for configuration files
        detector.add_pattern(ConflictPattern {
            name: "Config files".to_string(),
            file_pattern: "*.config".to_string(),
            class: ConflictClass::HumanVsAgent,
            resolution: ConflictResolution::Manual,
        });
        
        let conflicts = detector.detect_conflicts(
            "app.config",
            &ChangeSource::AgentIteration {
                iteration: 1,
                agent_id: "agent1".to_string(),
            },
        );
        
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].name, "Config files");
    }
}
