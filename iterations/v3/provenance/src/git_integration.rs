//! Git integration for provenance tracking
//!
//! Provides integration with git repositories for linking provenance records
//! to git commits via CAWS-VERDICT-ID trailers.

use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use git2::{Commit, Repository, Signature};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Mutex;
use uuid::Uuid;

use crate::types::ProvenanceRecord;

/// Git commit information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub timestamp: DateTime<Utc>,
    pub trailer: String,
}

/// Git integration trait
#[async_trait]
pub trait GitIntegration: Send + Sync {
    /// Add a git trailer to a commit
    async fn add_trailer_to_commit(&self, commit_hash: &str, trailer: &str) -> Result<String>;

    /// Create a new commit with provenance trailer
    async fn create_provenance_commit(
        &self,
        message: &str,
        provenance_record: &ProvenanceRecord,
    ) -> Result<String>;

    /// Verify git trailer exists
    async fn verify_trailer(&self, commit_hash: &str, trailer: &str) -> Result<bool>;

    /// Get commit information by trailer
    async fn get_commit_by_trailer(&self, trailer: &str) -> Result<Option<CommitInfo>>;

    /// List commits with provenance trailers
    async fn list_provenance_commits(&self) -> Result<Vec<CommitInfo>>;
}

/// Git trailer manager implementation
pub struct GitTrailerManager {
    repository: Mutex<Repository>,
    branch: String,
    auto_commit: bool,
    commit_message_template: String,
}

impl GitTrailerManager {
    /// Create a new git trailer manager
    pub fn new<P: AsRef<Path>>(
        repo_path: P,
        branch: String,
        auto_commit: bool,
        commit_message_template: String,
    ) -> Result<Self> {
        let repository = Repository::open(repo_path).context("Failed to open git repository")?;

        Ok(Self {
            repository: Mutex::new(repository),
            branch,
            auto_commit,
            commit_message_template,
        })
    }

    /// Generate commit message from template
    fn generate_commit_message(&self, provenance_record: &ProvenanceRecord) -> String {
        self.commit_message_template
            .replace("{verdict_id}", &provenance_record.verdict_id.to_string())
            .replace("{decision}", &provenance_record.decision.decision_type())
            .replace(
                "{consensus_score}",
                &provenance_record.consensus_score.to_string(),
            )
            .replace("{timestamp}", &provenance_record.timestamp.to_rfc3339())
    }

    /// Create signature for commits
    fn create_signature(&self) -> Result<Signature> {
        let repo = self.repository.lock().unwrap();
        let config = repo.config()?;
        let name = config
            .get_string("user.name")
            .unwrap_or_else(|_| "Agent Agency V3".to_string());
        let email = config
            .get_string("user.email")
            .unwrap_or_else(|_| "agent-agency@localhost".to_string());

        Signature::now(&name, &email).context("Failed to create git signature")
    }

    /// Get current branch reference (simplified for now)
    fn get_branch_ref(&self) -> Result<()> {
        let refname = format!("refs/heads/{}", self.branch);
        let _repo = self.repository.lock().unwrap();
        // TODO: Implement proper reference handling without lifetime issues
        Ok(())
    }

    /// Get the current HEAD commit (simplified for now)
    fn get_head_commit(&self) -> Result<()> {
        // TODO: Implement proper commit handling without lifetime issues
        Ok(())
    }
}

// Temporarily disable async trait implementation due to thread safety issues
// TODO: Implement proper thread-safe git integration
/*
#[async_trait]
impl GitIntegration for GitTrailerManager {
    async fn add_trailer_to_commit(
        &self,
        commit_hash: &str,
        trailer: &str,
    ) -> Result<String> {
        // This would typically involve:
        // 1. Finding the commit
        // 2. Creating a new commit with the trailer added to the message
        // 3. Updating the branch reference

        let commit = self.repository.find_commit(
            git2::Oid::from_str(commit_hash)
                .context("Invalid commit hash")?
        )?;

        // Get the current commit message
        let mut message = commit.message()
            .context("Commit has no message")?
            .to_string();

        // Add the trailer if not already present
        if !message.contains(trailer) {
            message.push_str(&format!("\n\n{}", trailer));
        }

        // Create new commit with trailer
        let signature = self.create_signature()?;
        let tree = commit.tree()?;

        let new_commit_id = self.repository.commit(
            Some(&format!("refs/heads/{}", self.branch)),
            &signature,
            &signature,
            &message,
            &tree,
            &[&commit],
        )?;

        Ok(new_commit_id.to_string())
    }

    async fn create_provenance_commit(
        &self,
        message: &str,
        provenance_record: &ProvenanceRecord,
    ) -> Result<String> {
        if !self.auto_commit {
            return Err(anyhow::anyhow!("Auto-commit is disabled"));
        }

        let signature = self.create_signature()?;
        let head_commit = self.get_head_commit()?;
        let tree = head_commit.tree()?;

        // Generate commit message with trailer
        let commit_message = format!(
            "{}\n\n{}",
            message,
            provenance_record.git_trailer
        );

        let new_commit_id = self.repository.commit(
            Some(&format!("refs/heads/{}", self.branch)),
            &signature,
            &signature,
            &commit_message,
            &tree,
            &[&head_commit],
        )?;

        Ok(new_commit_id.to_string())
    }

    async fn verify_trailer(&self, commit_hash: &str, trailer: &str) -> Result<bool> {
        let commit = self.repository.find_commit(
            git2::Oid::from_str(commit_hash)
                .context("Invalid commit hash")?
        )?;

        let message = commit.message()
            .context("Commit has no message")?;

        Ok(message.contains(trailer))
    }

    async fn get_commit_by_trailer(&self, trailer: &str) -> Result<Option<CommitInfo>> {
        let mut revwalk = self.repository.revwalk()?;
        revwalk.push_head()?;

        for commit_id in revwalk {
            let commit_id = commit_id?;
            let commit = self.repository.find_commit(commit_id)?;

            if let Some(message) = commit.message() {
                if message.contains(trailer) {
                    return Ok(Some(CommitInfo {
                        hash: commit_id.to_string(),
                        message: message.to_string(),
                        author: commit.author().name().unwrap_or("Unknown").to_string(),
                        timestamp: DateTime::from_timestamp(
                            commit.time().seconds(),
                            0,
                        ).unwrap_or_else(Utc::now),
                        trailer: trailer.to_string(),
                    }));
                }
            }
        }

        Ok(None)
    }

    async fn list_provenance_commits(&self) -> Result<Vec<CommitInfo>> {
        let mut commits = Vec::new();
        let mut revwalk = self.repository.revwalk()?;
        revwalk.push_head()?;

        for commit_id in revwalk {
            let commit_id = commit_id?;
            let commit = self.repository.find_commit(commit_id)?;

            if let Some(message) = commit.message() {
                if message.contains("CAWS-VERDICT-ID:") {
                    if let Some(trailer_start) = message.find("CAWS-VERDICT-ID:") {
                        let trailer_line = &message[trailer_start..];
                        let trailer = trailer_line.lines().next().unwrap_or("").to_string();

                        commits.push(CommitInfo {
                            hash: commit_id.to_string(),
                            message: message.to_string(),
                            author: commit.author().name().unwrap_or("Unknown").to_string(),
                            timestamp: DateTime::from_timestamp(
                                commit.time().seconds(),
                                0,
                            ).unwrap_or_else(Utc::now),
                            trailer,
                        });
                    }
                }
            }
        }

        Ok(commits)
    }
}


/// Git repository status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryStatus {
    pub is_clean: bool,
    pub current_branch: String,
    pub last_commit: Option<CommitInfo>,
    pub uncommitted_changes: Vec<String>,
    pub provenance_commits_count: u32,
}

/// Git integration utilities
pub struct GitUtils;

impl GitUtils {
    /// Check if a directory is a git repository
    pub fn is_git_repository<P: AsRef<Path>>(path: P) -> bool {
        Repository::open(path).is_ok()
    }

    /// Initialize a new git repository
    pub fn init_repository<P: AsRef<Path>>(path: P) -> Result<Repository> {
        Repository::init(path)
            .context("Failed to initialize git repository")
    }

    /// Get repository status
    pub fn get_repository_status(repo: &Repository) -> Result<RepositoryStatus> {
        let head = repo.head()?;
        let current_branch = head.shorthand().unwrap_or("HEAD").to_string();

        let mut status_options = git2::StatusOptions::new();
        status_options.include_untracked(true);
        status_options.include_ignored(false);

        let statuses = repo.statuses(Some(&mut status_options))?;
        let is_clean = statuses.is_empty();

        let mut uncommitted_changes = Vec::new();
        for entry in statuses.iter() {
            if let Some(path) = entry.path() {
                uncommitted_changes.push(path.to_string());
            }
        }

        let last_commit = if let Ok(commit) = repo.head()?.peel_to_commit() {
            Some(CommitInfo {
                hash: commit.id().to_string(),
                message: commit.message().unwrap_or("").to_string(),
                author: commit.author().name().unwrap_or("Unknown").to_string(),
                timestamp: DateTime::from_timestamp(
                    commit.time().seconds(),
                    0,
                ).unwrap_or_else(Utc::now),
                trailer: String::new(),
            })
        } else {
            None
        };

        // Count provenance commits
        let provenance_commits_count = Self::count_provenance_commits(repo)?;

        Ok(RepositoryStatus {
            is_clean,
            current_branch,
            last_commit,
            uncommitted_changes,
            provenance_commits_count,
        })
    }

    /// Count commits with provenance trailers
    fn count_provenance_commits(repo: &Repository) -> Result<u32> {
        let mut count = 0;
        let mut revwalk = repo.revwalk()?;
        revwalk.push_head()?;

        for commit_id in revwalk {
            let commit_id = commit_id?;
            let commit = repo.find_commit(commit_id)?;

            if let Some(message) = commit.message() {
                if message.contains("CAWS-VERDICT-ID:") {
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    /// Extract verdict ID from git trailer
    pub fn extract_verdict_id_from_trailer(trailer: &str) -> Result<Uuid> {
        if let Some(start) = trailer.find("CAWS-VERDICT-ID:") {
            let verdict_part = &trailer[start + 16..]; // Length of "CAWS-VERDICT-ID:"
            let verdict_id = verdict_part.trim();

            Uuid::parse_str(verdict_id)
                .context("Invalid verdict ID in git trailer")
        } else {
            Err(anyhow::anyhow!("No CAWS-VERDICT-ID trailer found"))
        }
    }

    /// Create git trailer from verdict ID
    pub fn create_trailer_from_verdict_id(verdict_id: Uuid) -> String {
        format!("CAWS-VERDICT-ID: {}", verdict_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_git_utils_trailer_creation_and_extraction() {
        let verdict_id = Uuid::new_v4();
        let trailer = GitUtils::create_trailer_from_verdict_id(verdict_id);

        assert!(trailer.contains("CAWS-VERDICT-ID:"));
        assert!(trailer.contains(&verdict_id.to_string()));

        let extracted_id = GitUtils::extract_verdict_id_from_trailer(&trailer).unwrap();
        assert_eq!(extracted_id, verdict_id);
    }

    #[test]
    fn test_git_utils_trailer_extraction_invalid() {
        let result = GitUtils::extract_verdict_id_from_trailer("Some other text");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_git_trailer_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        // Initialize a git repository
        let _repo = GitUtils::init_repository(repo_path).unwrap();

        // Create trailer manager
        let manager = GitTrailerManager::new(
            repo_path,
            "main".to_string(),
            true,
            "Test commit: {verdict_id}".to_string(),
        ).unwrap();

        // Test commit message generation
        let provenance_record = create_test_provenance_record();
        let message = manager.generate_commit_message(&provenance_record);

        assert!(message.contains(&provenance_record.verdict_id.to_string()));
    }

    fn create_test_provenance_record() -> ProvenanceRecord {
        use crate::types::*;
        use std::collections::HashMap;

        ProvenanceRecord {
            id: Uuid::new_v4(),
            verdict_id: Uuid::new_v4(),
            task_id: Uuid::new_v4(),
            decision: VerdictDecision::Accept {
                confidence: 0.9,
                summary: "Test verdict".to_string(),
            },
            consensus_score: 0.85,
            judge_verdicts: HashMap::new(),
            caws_compliance: CawsComplianceProvenance {
                is_compliant: true,
                compliance_score: 0.95,
                violations: vec![],
                waivers_used: vec![],
                budget_adherence: BudgetAdherence {
                    max_files: 10,
                    actual_files: 8,
                    max_loc: 1000,
                    actual_loc: 750,
                    max_time_minutes: Some(60),
                    actual_time_minutes: Some(45),
                    within_budget: true,
                },
            },
            claim_verification: None,
            git_commit_hash: None,
            git_trailer: "CAWS-VERDICT-ID: test".to_string(),
            signature: String::new(),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }
}
*/
