//! Merkle tree implementation for file metadata
//!
//! @author @darianrosebrook

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use crate::types::{Digest, StreamingHasher};

use crate::types::*;

/// Tree entry for file metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeEntry {
    pub name: String,
    pub mode: FileMode,
    pub digest: Digest,   // Blob or symlink target
}

impl TreeEntry {
    /// Create a new tree entry
    pub fn new(name: String, mode: FileMode, digest: Digest) -> Self {
        Self { name, mode, digest }
    }

    /// Create a regular file entry
    pub fn file(name: String, digest: Digest) -> Self {
        Self::new(name, FileMode::Regular, digest)
    }

    /// Create an executable file entry
    pub fn executable(name: String, digest: Digest) -> Self {
        Self::new(name, FileMode::Executable, digest)
    }

    /// Create a symlink entry
    pub fn symlink(name: String, target_digest: Digest) -> Self {
        Self::new(name, FileMode::Symlink, target_digest)
    }

    /// Get the POSIX mode bits
    pub fn posix_mode(&self) -> u32 {
        self.mode.to_posix()
    }

    /// Check if this is a regular file
    pub fn is_file(&self) -> bool {
        matches!(self.mode, FileMode::Regular | FileMode::Executable)
    }

    /// Check if this is a symlink
    pub fn is_symlink(&self) -> bool {
        matches!(self.mode, FileMode::Symlink)
    }
}

/// Merkle tree for file metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileTree {
    pub entries: Vec<TreeEntry>,
    pub digest: Digest,
}

impl FileTree {
    /// Create an empty tree
    pub fn empty() -> Self {
        let zero_digest = Digest::from_bytes([0u8; 32]);
        Self {
            entries: Vec::new(),
            digest: zero_digest,
        }
    }

    /// Create a tree from entries
    pub fn from_entries(entries: Vec<TreeEntry>) -> Self {
        let digest = Self::calculate_tree_digest(&entries);
        Self { entries, digest }
    }

    /// Add an entry to the tree
    pub fn add_entry(&mut self, entry: TreeEntry) {
        // Remove any existing entry with the same name
        self.entries.retain(|e| e.name != entry.name);
        self.entries.push(entry);
        
        // Sort entries by name for deterministic ordering
        self.entries.sort_by(|a, b| a.name.cmp(&b.name));
        
        // Recalculate digest
        self.digest = Self::calculate_tree_digest(&self.entries);
    }

    /// Remove an entry by name
    pub fn remove_entry(&mut self, name: &str) -> Option<TreeEntry> {
        let mut removed = None;
        self.entries.retain(|entry| {
            if entry.name == name {
                removed = Some(entry.clone());
                false
            } else {
                true
            }
        });
        
        if removed.is_some() {
            self.digest = Self::calculate_tree_digest(&self.entries);
        }
        
        removed
    }

    /// Get an entry by name
    pub fn get_entry(&self, name: &str) -> Option<&TreeEntry> {
        self.entries.iter().find(|entry| entry.name == name)
    }

    /// Get all entries
    pub fn entries(&self) -> &[TreeEntry] {
        &self.entries
    }

    /// Get the tree digest
    pub fn digest(&self) -> Digest {
        self.digest
    }

    /// Check if the tree is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get the number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Calculate the tree digest from entries
    fn calculate_tree_digest(entries: &[TreeEntry]) -> Digest {
        if entries.is_empty() {
            return Digest::from_bytes([0u8; 32]);
        }

        let mut hasher = StreamingHasher::new();
        
        for entry in entries {
            // Hash the entry: name + mode + digest
            hasher.update(entry.name.as_bytes());
            hasher.update(&entry.posix_mode().to_le_bytes());
            hasher.update(entry.digest.as_bytes());
        }
        
        hasher.finalize()
    }

    /// Create a tree from a directory path
    pub fn from_directory(path: &PathBuf) -> Result<Self> {
        let mut entries = Vec::new();
        
        if !path.exists() {
            return Ok(Self::empty());
        }
        
        let dir_entries = std::fs::read_dir(path)?;
        
        for entry in dir_entries {
            let entry = entry?;
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            
            let metadata = entry.metadata()?;
            let file_type = metadata.file_type();
            
            if file_type.is_file() {
                // Calculate digest for file content
                let content = std::fs::read(&path)?;
                let digest = Self::calculate_content_digest(&content);
                
                // Check if file is executable
                let mode = if Self::is_executable(&path)? {
                    FileMode::Executable
                } else {
                    FileMode::Regular
                };
                
                entries.push(TreeEntry::new(name, mode, digest));
            } else if file_type.is_symlink() {
                // For symlinks, store the target as a blob
                let target = std::fs::read_link(&path)?;
                let target_str = target.to_string_lossy().to_string();
                let target_digest = Self::calculate_content_digest(target_str.as_bytes());
                
                entries.push(TreeEntry::symlink(name, target_digest));
            }
        }
        
        Ok(Self::from_entries(entries))
    }

    /// Check if a file is executable
    fn is_executable(path: &PathBuf) -> Result<bool> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = std::fs::metadata(path)?;
            let permissions = metadata.permissions();
            Ok(permissions.mode() & 0o111 != 0)
        }
        
        #[cfg(not(unix))]
        {
            // On non-Unix systems, check file extension or shebang
            if let Some(extension) = path.extension() {
                let ext = extension.to_string_lossy().to_lowercase();
                Ok(matches!(ext.as_str(), "exe" | "bat" | "cmd" | "sh" | "py" | "rb" | "pl" | "ps1"))
            } else {
                // Check for shebang in first line
                if let Ok(content) = std::fs::read_to_string(path) {
                    Ok(content.starts_with("#!"))
                } else {
                    Ok(false)
                }
            }
        }
    }

    /// Calculate digest for content
    fn calculate_content_digest(content: &[u8]) -> Digest {
        let mut hasher = StreamingHasher::new();
        hasher.update(content);
        hasher.finalize()
    }

    /// Apply changes to create a new tree
    pub fn apply_changes(&self, changes: &[FileChange]) -> Result<Self> {
        let mut new_entries = self.entries.clone();
        
        for change in changes {
            match &change.payload {
                ChangePayload::Full(_) | ChangePayload::UnifiedDiff { .. } | ChangePayload::ChunkMap(_) => {
                    // Calculate new digest for the change
                    let new_digest = self.calculate_change_digest(change)?;
                    
                    // Update or add the entry
                    let entry = TreeEntry::new(
                        change.path.to_string_lossy().to_string(),
                        change.mode,
                        new_digest,
                    );
                    
                    // Remove existing entry with same name
                    new_entries.retain(|e| e.name != entry.name);
                    new_entries.push(entry);
                }
            }
        }
        
        // Sort entries by name for deterministic ordering
        new_entries.sort_by(|a, b| a.name.cmp(&b.name));
        
        Ok(Self::from_entries(new_entries))
    }

    /// Calculate digest for a change
    fn calculate_change_digest(&self, change: &FileChange) -> Result<Digest> {
        let mut hasher = StreamingHasher::new();
        
        match &change.payload {
            ChangePayload::Full(data) => {
                hasher.update(data);
            }
            ChangePayload::UnifiedDiff { hunks, .. } => {
                for hunk in hunks {
                    for line in &hunk.lines {
                        hasher.update(line.as_bytes());
                    }
                }
            }
            ChangePayload::ChunkMap(chunk_list) => {
                for chunk in &chunk_list.chunks {
                    hasher.update(chunk.digest.as_bytes());
                }
            }
        }
        
        Ok(hasher.finalize())
    }

    /// Get a diff between two trees
    pub fn diff(&self, other: &Self) -> TreeDiff {
        let mut added = Vec::new();
        let mut modified = Vec::new();
        let mut removed = Vec::new();
        
        // Create maps for efficient lookup
        let self_map: HashMap<String, &TreeEntry> = self.entries.iter()
            .map(|e| (e.name.clone(), e))
            .collect();
        let other_map: HashMap<String, &TreeEntry> = other.entries.iter()
            .map(|e| (e.name.clone(), e))
            .collect();
        
        // Find added and modified entries
        for entry in &other.entries {
            if let Some(self_entry) = self_map.get(&entry.name) {
                if self_entry.digest != entry.digest {
                    modified.push(entry.clone());
                }
            } else {
                added.push(entry.clone());
            }
        }
        
        // Find removed entries
        for entry in &self.entries {
            if !other_map.contains_key(&entry.name) {
                removed.push(entry.clone());
            }
        }
        
        TreeDiff {
            added,
            modified,
            removed,
        }
    }
}

/// Tree diff result
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeDiff {
    pub added: Vec<TreeEntry>,
    pub modified: Vec<TreeEntry>,
    pub removed: Vec<TreeEntry>,
}

impl TreeDiff {
    /// Check if there are any changes
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.modified.is_empty() && self.removed.is_empty()
    }

    /// Get the total number of changes
    pub fn len(&self) -> usize {
        self.added.len() + self.modified.len() + self.removed.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_empty_tree() {
        let tree = FileTree::empty();
        assert!(tree.is_empty());
        assert_eq!(tree.len(), 0);
    }

    #[test]
    fn test_tree_with_entries() {
        let digest1 = Digest::from_bytes([1u8; 32]);
        let digest2 = Digest::from_bytes([2u8; 32]);
        
        let entries = vec![
            TreeEntry::file("file1.txt".to_string(), digest1),
            TreeEntry::executable("script.sh".to_string(), digest2),
        ];
        
        let tree = FileTree::from_entries(entries);
        assert_eq!(tree.len(), 2);
        assert!(!tree.is_empty());
    }

    #[test]
    fn test_tree_add_entry() {
        let mut tree = FileTree::empty();
        let digest = Digest::from_bytes([1u8; 32]);
        let entry = TreeEntry::file("test.txt".to_string(), digest);
        
        tree.add_entry(entry);
        assert_eq!(tree.len(), 1);
    }

    #[test]
    fn test_tree_remove_entry() {
        let digest = Digest::from_bytes([1u8; 32]);
        let entry = TreeEntry::file("test.txt".to_string(), digest);
        let mut tree = FileTree::from_entries(vec![entry.clone()]);
        
        let removed = tree.remove_entry("test.txt");
        assert!(removed.is_some());
        assert!(tree.is_empty());
    }

    #[test]
    fn test_tree_diff() {
        let digest1 = Digest::from_bytes([1u8; 32]);
        let digest2 = Digest::from_bytes([2u8; 32]);
        let digest3 = Digest::from_bytes([3u8; 32]);
        
        let tree1 = FileTree::from_entries(vec![
            TreeEntry::file("file1.txt".to_string(), digest1),
            TreeEntry::file("file2.txt".to_string(), digest2),
        ]);
        
        let tree2 = FileTree::from_entries(vec![
            TreeEntry::file("file1.txt".to_string(), digest1), // unchanged
            TreeEntry::file("file2.txt".to_string(), digest3), // modified
            TreeEntry::file("file3.txt".to_string(), digest2),  // added
        ]);
        
        let diff = tree1.diff(&tree2);
        assert_eq!(diff.added.len(), 1);
        assert_eq!(diff.modified.len(), 1);
        assert_eq!(diff.removed.len(), 0);
    }
}
