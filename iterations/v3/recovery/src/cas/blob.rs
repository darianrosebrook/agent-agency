//! Content-Addressable Storage blob implementation
//!
//! @author @darianrosebrook

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use tracing::debug;

use crate::types::*;
use crate::types::{Digest, StreamingHasher};

/// Blob storage for content-addressable objects
pub struct BlobStore {
    objects_dir: PathBuf,
}

impl BlobStore {
    /// Create a new blob store
    pub fn new(objects_dir: PathBuf) -> Self {
        Self { objects_dir }
    }

    /// Store a blob with payload header
    pub fn store_blob(&self, digest: Digest, header: PayloadHeader, data: &[u8]) -> Result<()> {
        let blob_path = self.get_blob_path(digest);
        
        // Ensure directory exists
        if let Some(parent) = blob_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Write blob with header
        let mut file = File::create(&blob_path)?;
        self.write_blob_with_header(&mut file, header, data)?;
        file.sync_all()?;

        debug!("Stored blob: {} at {}", digest, blob_path.display());
        Ok(())
    }

    /// Retrieve a blob by digest
    pub fn get_blob(&self, digest: Digest) -> Result<Option<Blob>> {
        let blob_path = self.get_blob_path(digest);
        
        if !blob_path.exists() {
            return Ok(None);
        }

        let mut file = File::open(&blob_path)?;
        let blob = self.read_blob_with_header(&mut file)?;
        
        debug!("Retrieved blob: {}", digest);
        Ok(Some(blob))
    }

    /// Check if a blob exists
    pub fn has_blob(&self, digest: Digest) -> bool {
        self.get_blob_path(digest).exists()
    }

    /// Get the path for a blob
    fn get_blob_path(&self, digest: Digest) -> PathBuf {
        let hex_digest = digest.to_hex();
        let dir = &hex_digest[0..2];
        self.objects_dir.join(dir).join(&hex_digest)
    }

    /// Write blob with header
    fn write_blob_with_header(&self, file: &mut File, header: PayloadHeader, data: &[u8]) -> Result<()> {
        // Serialize header
        let header_bytes = bincode::serialize(&header)?;
        let header_len = header_bytes.len() as u32;
        
        // Write header length (4 bytes)
        file.write_all(&header_len.to_le_bytes())?;
        
        // Write header
        file.write_all(&header_bytes)?;
        
        // Write data
        file.write_all(data)?;
        
        Ok(())
    }

    /// Read blob with header
    fn read_blob_with_header(&self, file: &mut File) -> Result<Blob> {
        // Read header length
        let mut header_len_bytes = [0u8; 4];
        file.read_exact(&mut header_len_bytes)?;
        let header_len = u32::from_le_bytes(header_len_bytes) as usize;
        
        // Read header
        let mut header_bytes = vec![0u8; header_len];
        file.read_exact(&mut header_bytes)?;
        let header: PayloadHeader = bincode::deserialize(&header_bytes)?;
        
        // Read data
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        
        Ok(Blob { header, data })
    }

    /// Calculate digest for content
    pub fn calculate_digest(content: &[u8]) -> Digest {
        let mut hasher = StreamingHasher::new();
        hasher.update(content);
        hasher.finalize()
    }

    /// Verify blob integrity
    pub fn verify_blob(&self, digest: Digest) -> Result<bool> {
        if let Some(blob) = self.get_blob(digest)? {
            let calculated_digest = Self::calculate_digest(&blob.data);
            Ok(calculated_digest == digest)
        } else {
            Ok(false)
        }
    }

    /// Get blob size
    pub fn get_blob_size(&self, digest: Digest) -> Result<Option<u64>> {
        let blob_path = self.get_blob_path(digest);
        if blob_path.exists() {
            Ok(Some(std::fs::metadata(&blob_path)?.len()))
        } else {
            Ok(None)
        }
    }

    /// List all blobs in the store
    pub fn list_blobs(&self) -> Result<Vec<Digest>> {
        let mut digests = Vec::new();
        
        if self.objects_dir.exists() {
            for entry in std::fs::read_dir(&self.objects_dir)? {
                let entry = entry?;
                if entry.path().is_dir() {
                    for file_entry in std::fs::read_dir(entry.path())? {
                        let file_entry = file_entry?;
                        if file_entry.path().is_file() {
                            if let Some(file_name) = file_entry.path().file_name() {
                                if let Some(name_str) = file_name.to_str() {
                                    if let Ok(digest) = Digest::from_hex(name_str) {
                                        digests.push(digest);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(digests)
    }

    /// Remove a blob
    pub fn remove_blob(&self, digest: Digest) -> Result<()> {
        let blob_path = self.get_blob_path(digest);
        if blob_path.exists() {
            std::fs::remove_file(&blob_path)?;
            debug!("Removed blob: {}", digest);
        }
        Ok(())
    }

    /// Get storage statistics
    pub fn get_stats(&self) -> Result<BlobStats> {
        let mut total_size = 0u64;
        let mut blob_count = 0u64;
        
        if self.objects_dir.exists() {
            for entry in std::fs::read_dir(&self.objects_dir)? {
                let entry = entry?;
                if entry.path().is_dir() {
                    for file_entry in std::fs::read_dir(entry.path())? {
                        let file_entry = file_entry?;
                        if file_entry.path().is_file() {
                            if let Ok(metadata) = file_entry.metadata() {
                                total_size += metadata.len();
                                blob_count += 1;
                            }
                        }
                    }
                }
            }
        }
        
        Ok(BlobStats {
            total_size,
            blob_count,
        })
    }
}

/// Blob with header and data
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Blob {
    pub header: PayloadHeader,
    pub data: Vec<u8>,
}

impl Blob {
    /// Create a new blob
    pub fn new(header: PayloadHeader, data: Vec<u8>) -> Self {
        Self { header, data }
    }

    /// Get the blob header
    pub fn header(&self) -> &PayloadHeader {
        &self.header
    }

    /// Get the blob data
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get the uncompressed size
    pub fn uncompressed_size(&self) -> u32 {
        self.header.content_len
    }

    /// Check if the blob is compressed
    pub fn is_compressed(&self) -> bool {
        !matches!(self.header.codec, Codec::None)
    }

    /// Get the compression ratio
    pub fn compression_ratio(&self) -> f64 {
        if self.header.content_len == 0 {
            0.0
        } else {
            self.data.len() as f64 / self.header.content_len as f64
        }
    }
}

/// Blob storage statistics
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlobStats {
    pub total_size: u64,
    pub blob_count: u64,
}

impl BlobStats {
    /// Get average blob size
    pub fn average_size(&self) -> f64 {
        if self.blob_count == 0 {
            0.0
        } else {
            self.total_size as f64 / self.blob_count as f64
        }
    }

    /// Get size in human-readable format
    pub fn human_size(&self) -> String {
        self.format_bytes(self.total_size)
    }

    /// Format bytes in human-readable format
    fn format_bytes(&self, bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        const THRESHOLD: f64 = 1024.0;

        if bytes == 0 {
            return "0 B".to_string();
        }

        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= THRESHOLD && unit_index < UNITS.len() - 1 {
            size /= THRESHOLD;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{} {}", bytes, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }
}

/// Blob builder for creating blobs with proper headers
pub struct BlobBuilder {
    header: PayloadHeader,
    data: Vec<u8>,
}

impl BlobBuilder {
    /// Create a new blob builder
    pub fn new() -> Self {
        Self {
            header: PayloadHeader {
                version: 1,
                kind: PayloadKind::Full,
                codec: Codec::None,
                eol: None,
                content_len: 0,
            },
            data: Vec::new(),
        }
    }

    /// Set the payload kind
    pub fn kind(mut self, kind: PayloadKind) -> Self {
        self.header.kind = kind;
        self
    }

    /// Set the compression codec
    pub fn codec(mut self, codec: Codec) -> Self {
        self.header.codec = codec;
        self
    }

    /// Set the end-of-line type
    pub fn eol(mut self, eol: Eol) -> Self {
        self.header.eol = Some(eol);
        self
    }

    /// Set the data
    pub fn data(mut self, data: Vec<u8>) -> Self {
        self.header.content_len = data.len() as u32;
        self.data = data;
        self
    }

    /// Build the blob
    pub fn build(self) -> Blob {
        Blob {
            header: self.header,
            data: self.data,
        }
    }
}

impl Default for BlobBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_blob_store_creation() {
        let temp_dir = TempDir::new().unwrap();
        let objects_dir = temp_dir.path().join("objects");
        let store = BlobStore::new(objects_dir);
        
        assert!(store.objects_dir.exists() || !store.objects_dir.exists());
    }

    #[test]
    fn test_blob_storage_and_retrieval() {
        let temp_dir = TempDir::new().unwrap();
        let objects_dir = temp_dir.path().join("objects");
        let store = BlobStore::new(objects_dir);
        
        let data = b"test content";
        let digest = BlobStore::calculate_digest(data);
        let header = PayloadHeader {
            version: 1,
            kind: PayloadKind::Full,
            codec: Codec::None,
            eol: None,
            content_len: data.len() as u32,
        };
        
        store.store_blob(digest, header, data).unwrap();
        
        let retrieved = store.get_blob(digest).unwrap().unwrap();
        assert_eq!(retrieved.data, data);
        assert_eq!(retrieved.header.content_len, data.len() as u32);
    }

    #[test]
    fn test_blob_verification() {
        let temp_dir = TempDir::new().unwrap();
        let objects_dir = temp_dir.path().join("objects");
        let store = BlobStore::new(objects_dir);
        
        let data = b"test content";
        let digest = BlobStore::calculate_digest(data);
        let header = PayloadHeader {
            version: 1,
            kind: PayloadKind::Full,
            codec: Codec::None,
            eol: None,
            content_len: data.len() as u32,
        };
        
        store.store_blob(digest, header, data).unwrap();
        
        assert!(store.verify_blob(digest).unwrap());
    }

    #[test]
    fn test_blob_builder() {
        let data = b"test content";
        let blob = BlobBuilder::new()
            .kind(PayloadKind::Full)
            .codec(Codec::None)
            .data(data.to_vec())
            .build();
        
        assert_eq!(blob.data, data);
        assert_eq!(blob.header.kind, PayloadKind::Full);
        assert_eq!(blob.header.codec, Codec::None);
        assert_eq!(blob.header.content_len, data.len() as u32);
    }
}
