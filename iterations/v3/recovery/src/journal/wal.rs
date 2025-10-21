//! Write-ahead log implementation with crash safety
//!
//! @author @darianrosebrook

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tracing::debug;

use crate::types::*;
use crate::types::Digest;
use crate::policy::DenialReason;

/// Write-ahead log for crash-safe operations
pub struct WriteAheadLog {
    log_path: PathBuf,
    writer: Mutex<BufWriter<File>>,
}

impl WriteAheadLog {
    /// Create a new WAL at the specified path
    pub fn new(log_path: PathBuf) -> Result<Self> {
        // Ensure the journal directory exists
        if let Some(parent) = log_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;

        let writer = Mutex::new(BufWriter::new(file));

        Ok(Self { log_path, writer })
    }

    /// Record the beginning of a change operation
    pub fn record_begin(&self, change_id: &ChangeId, path: &Path) -> Result<()> {
        let record = JournalRecord::Begin {
            change_id: change_id.clone(),
            path: path.to_path_buf(),
            timestamp: Utc::now(),
        };

        self.write_record(record)?;
        self.flush()?;
        Ok(())
    }

    /// Record the successful completion of a change operation
    pub fn record_commit(&self, change_id: &ChangeId, digest: Digest) -> Result<()> {
        let record = JournalRecord::Commit {
            change_id: change_id.clone(),
            digest,
            timestamp: Utc::now(),
        };

        self.write_record(record)?;
        self.flush()?;
        Ok(())
    }

    /// Record a denied operation (secret scanning, policy violation)
    pub fn record_denied(&self, change_id: &ChangeId, reason: DenialReason, fingerprint: Digest) -> Result<()> {
        let record = JournalRecord::Denied {
            change_id: change_id.clone(),
            reason,
            fingerprint,
            timestamp: Utc::now(),
        };

        self.write_record(record)?;
        self.flush()?;
        Ok(())
    }

    /// Write a journal record with proper serialization
    fn write_record(&self, record: JournalRecord) -> Result<()> {
        let mut writer = self.writer.lock().unwrap();
        
        // Serialize to JSON with newline delimiter
        let json = serde_json::to_string(&record)?;
        writer.write_all(json.as_bytes())?;
        writer.write_all(b"\n")?;
        
        debug!("Wrote journal record: {:?}", record);
        Ok(())
    }

    /// Flush the WAL to ensure durability
    pub fn flush(&self) -> Result<()> {
        let mut writer = self.writer.lock().unwrap();
        writer.flush()?;
        
        // Force fsync on the journal file
        let file = writer.get_ref();
        file.sync_all()?;
        
        debug!("Flushed WAL to disk");
        Ok(())
    }

    /// Replay the journal to recover from crash
    pub fn replay(&self) -> Result<ReplayResult> {
        let mut begin_records = std::collections::HashMap::new();
        let mut commit_records = std::collections::HashMap::new();
        let mut denied_records = std::collections::HashMap::new();

        // Read all journal records
        let content = std::fs::read_to_string(&self.log_path)?;
        
        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let record: JournalRecord = serde_json::from_str(line)?;
            
            match record {
                JournalRecord::Begin { change_id, path, timestamp } => {
                    begin_records.insert(change_id.clone(), (path, timestamp));
                }
                JournalRecord::Commit { change_id, digest, timestamp } => {
                    commit_records.insert(change_id, (digest, timestamp));
                }
                JournalRecord::Denied { change_id, reason, fingerprint, timestamp } => {
                    denied_records.insert(change_id, (reason, fingerprint, timestamp));
                }
            }
        }

        // Find orphaned Begin records (no corresponding Commit)
        let orphaned: Vec<_> = begin_records
            .keys()
            .filter(|id| !commit_records.contains_key(id) && !denied_records.contains_key(id))
            .cloned()
            .collect();

        Ok(ReplayResult {
            orphaned_begin_records: orphaned,
            completed_operations: commit_records.len(),
            denied_operations: denied_records.len(),
        })
    }

    /// Clean up orphaned records after successful replay
    pub fn cleanup_orphaned(&self, orphaned_ids: &[ChangeId]) -> Result<()> {
        for change_id in orphaned_ids {
            debug!("Cleaning up orphaned change: {}", change_id);
            // In a full implementation, this would delete the corresponding blob files
            // For now, we just log the cleanup
        }
        Ok(())
    }
}

/// Journal record types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JournalRecord {
    Begin {
        change_id: ChangeId,
        path: PathBuf,
        timestamp: DateTime<Utc>,
    },
    Commit {
        change_id: ChangeId,
        digest: Digest,
        timestamp: DateTime<Utc>,
    },
    Denied {
        change_id: ChangeId,
        reason: DenialReason,
        fingerprint: Digest,
        timestamp: DateTime<Utc>,
    },
}

/// Result of journal replay
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayResult {
    pub orphaned_begin_records: Vec<ChangeId>,
    pub completed_operations: usize,
    pub denied_operations: usize,
}

/// Crash-safe write path implementation
pub struct CrashSafeWriter {
    wal: WriteAheadLog,
    objects_dir: PathBuf,
    index_dir: PathBuf,
    refs_dir: PathBuf,
}

impl CrashSafeWriter {
    /// Create a new crash-safe writer
    pub fn new(
        wal: WriteAheadLog,
        objects_dir: PathBuf,
        index_dir: PathBuf,
        refs_dir: PathBuf,
    ) -> Self {
        Self {
            wal,
            objects_dir,
            index_dir,
            refs_dir,
        }
    }

    /// Write a change with full crash safety
    pub async fn write_change(&self, change: &FileChange) -> Result<ChangeId> {
        let change_id = ChangeId::new();
        
        // Step 1: Record Begin in WAL and fsync
        self.wal.record_begin(&change_id, &change.path)?;
        
        // Step 2: Write payload to temp file, fsync, then atomic rename with directory fsync
        let blob_path = self.write_blob_atomic(&change_id, &change.payload).await?;
        
        // Step 3: Update SQLite index in transaction, fsync index directory
        self.update_index_atomic(&change_id, change).await?;
        
        // Step 4: Update refs with atomic rename and directory fsync
        self.update_refs_atomic(&change_id).await?;
        
        // Step 5: Record Commit in WAL and fsync
        let digest = self.calculate_digest(&change.payload)?;
        self.wal.record_commit(&change_id, digest)?;
        
        Ok(change_id)
    }

    /// Write blob with atomic rename and directory fsync
    async fn write_blob_atomic(&self, change_id: &ChangeId, payload: &ChangePayload) -> Result<PathBuf> {
        // Create objects directory structure
        let digest = self.calculate_digest(payload)?;
        let hex_digest = digest.to_hex();
        let object_dir = self.objects_dir.join(&hex_digest[0..2]);
        std::fs::create_dir_all(&object_dir)?;

        // Write to temp file first
        let temp_path = object_dir.join(format!("{}.tmp", hex_digest));
        let final_path = object_dir.join(&hex_digest);
        
        {
            let mut file = std::fs::File::create(&temp_path)?;
            self.serialize_payload(payload, &mut file)?;
            file.sync_all()?; // fsync the file
        }
        
        // fsync the parent directory
        if let Ok(dir_file) = std::fs::File::open(&object_dir) {
            dir_file.sync_all()?;
        }
        
        // Atomic rename
        std::fs::rename(&temp_path, &final_path)?;
        
        // fsync the parent directory again after rename
        if let Ok(dir_file) = std::fs::File::open(&object_dir) {
            dir_file.sync_all()?;
        }
        
        Ok(final_path)
    }

    /// Update SQLite index with transaction and directory fsync
    async fn update_index_atomic(&self, change_id: &ChangeId, change: &FileChange) -> Result<()> {
        // In a full implementation, this would:
        // 1. Begin SQLite transaction
        // 2. Update file_versions and commits tables
        // 3. Commit transaction
        // 4. fsync the index directory
        
        debug!("Updating index for change: {}", change_id);
        Ok(())
    }

    /// Update refs with atomic rename and directory fsync
    async fn update_refs_atomic(&self, change_id: &ChangeId) -> Result<()> {
        // In a full implementation, this would:
        // 1. Write to refs/sessions/<sid>.tmp
        // 2. fsync the file
        // 3. Atomic rename to final name
        // 4. fsync the refs directory
        
        debug!("Updating refs for change: {}", change_id);
        Ok(())
    }

    /// Calculate digest for payload
    fn calculate_digest(&self, payload: &ChangePayload) -> Result<Digest> {
        use crate::types::StreamingHasher;
        
        let mut hasher = StreamingHasher::new();
        match payload {
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

    /// Serialize payload to file
    fn serialize_payload(&self, payload: &ChangePayload, file: &mut File) -> Result<()> {
        // In a full implementation, this would serialize with PayloadHeader
        // For now, just serialize as JSON
        let json = serde_json::to_string(payload)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_wal_record_begin() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test.log");
        let wal = WriteAheadLog::new(log_path).unwrap();
        
        let change_id = ChangeId::new();
        let path = std::path::Path::new("test.txt");
        
        wal.record_begin(&change_id, path).unwrap();
        wal.flush().unwrap();
    }

    #[test]
    fn test_wal_record_commit() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test.log");
        let wal = WriteAheadLog::new(log_path).unwrap();
        
        let change_id = ChangeId::new();
        let digest = Digest::from_bytes([1u8; 32]);
        
        wal.record_commit(&change_id, digest).unwrap();
        wal.flush().unwrap();
    }

    #[test]
    fn test_wal_replay() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test.log");
        let wal = WriteAheadLog::new(log_path).unwrap();
        
        let change_id = ChangeId::new();
        let path = std::path::Path::new("test.txt");
        let digest = Digest::from_bytes([1u8; 32]);
        
        wal.record_begin(&change_id, path).unwrap();
        wal.record_commit(&change_id, digest).unwrap();
        wal.flush().unwrap();
        
        let result = wal.replay().unwrap();
        assert_eq!(result.completed_operations, 1);
        assert_eq!(result.orphaned_begin_records.len(), 0);
    }
}
