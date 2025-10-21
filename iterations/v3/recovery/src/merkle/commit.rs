//! Commit implementation for Merkle trees
//!
//! @author @darianrosebrook

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::types::{Digest, StreamingHasher};

use crate::types::*;
// use super::tree::FileTree; // Unused

/// Commit object with Merkle tree root
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Commit {
    pub id: Digest,
    pub parent: Option<Digest>,
    pub tree: Digest,                   // Merkle root (trees are authoritative)
    pub session_id: String,
    pub caws_verdict_id: Option<String>,
    pub message: Option<String>,
    pub stats: ChangeStats,             // Observability
    pub timestamp: DateTime<Utc>,
    pub author: AuthorInfo,
}

/// Author information for commits
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorInfo {
    pub name: String,
    pub email: String,
    pub agent_id: Option<String>,
}

impl AuthorInfo {
    /// Create author info for an agent
    pub fn agent(name: String, email: String, agent_id: String) -> Self {
        Self {
            name,
            email,
            agent_id: Some(agent_id),
        }
    }

    /// Create author info for a human user
    pub fn human(name: String, email: String) -> Self {
        Self {
            name,
            email,
            agent_id: None,
        }
    }
}

impl Commit {
    /// Create a new commit
    pub fn new(
        parent: Option<Digest>,
        tree: Digest,
        session_id: String,
        author: AuthorInfo,
        message: Option<String>,
        stats: ChangeStats,
    ) -> Self {
        let timestamp = Utc::now();
        let id = Self::calculate_commit_id(parent, tree, session_id.clone(), author.clone(), timestamp);
        
        Self {
            id,
            parent,
            tree,
            session_id,
            caws_verdict_id: None,
            message,
            stats,
            timestamp,
            author,
        }
    }

    /// Create a commit with CAWS verdict
    pub fn with_verdict(
        mut self,
        verdict_id: String,
    ) -> Self {
        self.caws_verdict_id = Some(verdict_id);
        self
    }

    /// Calculate the commit ID from its components
    fn calculate_commit_id(
        parent: Option<Digest>,
        tree: Digest,
        session_id: String,
        author: AuthorInfo,
        timestamp: DateTime<Utc>,
    ) -> Digest {
        let mut hasher = StreamingHasher::new();
        
        // Hash parent commit (or zero if root)
        if let Some(parent) = parent {
            hasher.update(parent.as_bytes());
        } else {
            hasher.update(&[0u8; 32]);
        }
        
        // Hash tree root
        hasher.update(tree.as_bytes());
        
        // Hash session ID
        hasher.update(session_id.as_bytes());
        
        // Hash author info
        hasher.update(author.name.as_bytes());
        hasher.update(author.email.as_bytes());
        if let Some(agent_id) = &author.agent_id {
            hasher.update(agent_id.as_bytes());
        }
        
        // Hash timestamp
        hasher.update(&timestamp.timestamp().to_le_bytes());
        
        hasher.finalize()
    }

    /// Get the commit ID
    pub fn id(&self) -> Digest {
        self.id
    }

    /// Get the parent commit ID
    pub fn parent(&self) -> Option<Digest> {
        self.parent
    }

    /// Get the tree root digest
    pub fn tree(&self) -> Digest {
        self.tree
    }

    /// Check if this is a root commit (no parent)
    pub fn is_root(&self) -> bool {
        self.parent.is_none()
    }

    /// Get the session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Get the author information
    pub fn author(&self) -> &AuthorInfo {
        &self.author
    }

    /// Get the commit message
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    /// Get the change statistics
    pub fn stats(&self) -> &ChangeStats {
        &self.stats
    }

    /// Get the timestamp
    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    /// Check if this commit has a CAWS verdict
    pub fn has_verdict(&self) -> bool {
        self.caws_verdict_id.is_some()
    }

    /// Get the CAWS verdict ID
    pub fn verdict_id(&self) -> Option<&str> {
        self.caws_verdict_id.as_deref()
    }
}

/// Commit builder for creating commits
pub struct CommitBuilder {
    parent: Option<Digest>,
    tree: Option<Digest>,
    session_id: Option<String>,
    author: Option<AuthorInfo>,
    message: Option<String>,
    stats: Option<ChangeStats>,
}

impl CommitBuilder {
    /// Create a new commit builder
    pub fn new() -> Self {
        Self {
            parent: None,
            tree: None,
            session_id: None,
            author: None,
            message: None,
            stats: None,
        }
    }

    /// Set the parent commit
    pub fn parent(mut self, parent: Digest) -> Self {
        self.parent = Some(parent);
        self
    }

    /// Set the tree root
    pub fn tree(mut self, tree: Digest) -> Self {
        self.tree = Some(tree);
        self
    }

    /// Set the session ID
    pub fn session_id(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }

    /// Set the author
    pub fn author(mut self, author: AuthorInfo) -> Self {
        self.author = Some(author);
        self
    }

    /// Set the commit message
    pub fn message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }

    /// Set the change statistics
    pub fn stats(mut self, stats: ChangeStats) -> Self {
        self.stats = Some(stats);
        self
    }

    /// Build the commit
    pub fn build(self) -> Result<Commit> {
        let parent = self.parent;
        let tree = self.tree.ok_or_else(|| anyhow::anyhow!("Tree root is required"))?;
        let session_id = self.session_id.ok_or_else(|| anyhow::anyhow!("Session ID is required"))?;
        let author = self.author.ok_or_else(|| anyhow::anyhow!("Author is required"))?;
        let message = self.message;
        let stats = self.stats.unwrap_or_else(|| ChangeStats {
            files_added: 0,
            files_changed: 0,
            files_deleted: 0,
            bytes_added: 0,
            bytes_changed: 0,
            dedupe_ratio: 0.0,
        });

        Ok(Commit::new(parent, tree, session_id, author, message, stats))
    }
}

impl Default for CommitBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Commit chain for tracking history
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommitChain {
    pub commits: Vec<Commit>,
}

impl CommitChain {
    /// Create an empty commit chain
    pub fn new() -> Self {
        Self {
            commits: Vec::new(),
        }
    }

    /// Add a commit to the chain
    pub fn add_commit(&mut self, commit: Commit) {
        self.commits.push(commit);
    }

    /// Get the latest commit
    pub fn latest(&self) -> Option<&Commit> {
        self.commits.last()
    }

    /// Get the root commit (first in chain)
    pub fn root(&self) -> Option<&Commit> {
        self.commits.first()
    }

    /// Get all commits in the chain
    pub fn commits(&self) -> &[Commit] {
        &self.commits
    }

    /// Get the length of the chain
    pub fn len(&self) -> usize {
        self.commits.len()
    }

    /// Check if the chain is empty
    pub fn is_empty(&self) -> bool {
        self.commits.is_empty()
    }

    /// Get commits in reverse chronological order (newest first)
    pub fn reverse_chronological(&self) -> Vec<&Commit> {
        let mut commits: Vec<&Commit> = self.commits.iter().collect();
        commits.reverse();
        commits
    }

    /// Find a commit by ID
    pub fn find_commit(&self, id: Digest) -> Option<&Commit> {
        self.commits.iter().find(|commit| commit.id == id)
    }

    /// Get the commit history for a specific file
    pub fn file_history(&self, path: &str) -> Vec<&Commit> {
        self.commits
            .iter()
            .filter(|commit| {
                // In a full implementation, this would check if the commit
                // modified the specific file by examining the tree
                true // Simplified for now
            })
            .collect()
    }
}

impl Default for CommitChain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commit_creation() {
        let tree_digest = Digest::from_bytes([1u8; 32]);
        let session_id = "session-123".to_string();
        let author = AuthorInfo::agent(
            "Test Agent".to_string(),
            "agent@test.com".to_string(),
            "agent-123".to_string(),
        );
        let stats = ChangeStats {
            files_added: 1,
            files_changed: 0,
            files_deleted: 0,
            bytes_added: 100,
            bytes_changed: 0,
            dedupe_ratio: 0.5,
        };

        let commit = Commit::new(
            None,
            tree_digest,
            session_id,
            author,
            Some("Initial commit".to_string()),
            stats,
        );

        assert!(commit.is_root());
        assert_eq!(commit.tree(), tree_digest);
        assert!(commit.message().is_some());
    }

    #[test]
    fn test_commit_builder() {
        let tree_digest = Digest::from_bytes([1u8; 32]);
        let session_id = "session-123".to_string();
        let author = AuthorInfo::human(
            "Test User".to_string(),
            "user@test.com".to_string(),
        );

        let commit = CommitBuilder::new()
            .tree(tree_digest)
            .session_id(session_id)
            .author(author)
            .message("Test commit".to_string())
            .build()
            .unwrap();

        assert!(commit.is_root());
        assert_eq!(commit.tree(), tree_digest);
    }

    #[test]
    fn test_commit_chain() {
        let mut chain = CommitChain::new();
        assert!(chain.is_empty());

        let tree_digest = Digest::from_bytes([1u8; 32]);
        let session_id = "session-123".to_string();
        let author = AuthorInfo::agent(
            "Test Agent".to_string(),
            "agent@test.com".to_string(),
            "agent-123".to_string(),
        );

        let commit = Commit::new(
            None,
            tree_digest,
            session_id,
            author,
            None,
            ChangeStats::default(),
        );

        chain.add_commit(commit);
        assert_eq!(chain.len(), 1);
        assert!(chain.latest().is_some());
        assert!(chain.root().is_some());
    }
}

impl Default for ChangeStats {
    fn default() -> Self {
        Self {
            files_added: 0,
            files_changed: 0,
            files_deleted: 0,
            bytes_added: 0,
            bytes_changed: 0,
            dedupe_ratio: 0.0,
        }
    }
}
