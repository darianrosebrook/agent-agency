use anyhow::{anyhow, Result};
use fastcdc::v2020::{ChunkData, FastCDC};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::types::{Digest, StreamingHasher, ChunkRef};

/// Content-Defined Chunking configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkingConfig {
    /// Minimum chunk size in bytes
    pub min_size: usize,
    /// Average chunk size in bytes
    pub avg_size: usize,
    /// Maximum chunk size in bytes
    pub max_size: usize,
    /// Whether to use gear-based rolling hash
    pub use_gear_hash: bool,
    /// Gear hash polynomial (for gear-based hashing)
    pub gear_polynomial: u64,
    /// Gear hash mask (for gear-based hashing)
    pub gear_mask: u64,
}

impl Default for ChunkingConfig {
    fn default() -> Self {
        Self {
            min_size: 4 * 1024,     // 4KB
            avg_size: 16 * 1024,    // 16KB
            max_size: 64 * 1024,    // 64KB
            use_gear_hash: true,
            gear_polynomial: 0x9e3779b97f4a7c15, // Golden ratio
            gear_mask: 0x1fffffffffffffff,        // 60-bit mask
        }
    }
}

/// Chunk information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    /// Chunk digest
    pub digest: Digest,
    /// Chunk offset in the original content
    pub offset: usize,
    /// Chunk length
    pub length: usize,
    /// Chunk data
    pub data: Vec<u8>,
}

/// Chunk list for a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkList {
    /// List of chunks
    pub chunks: Vec<Chunk>,
    /// Total content length
    pub total_length: usize,
    /// File digest (hash of all chunk digests)
    pub file_digest: Digest,
}

/// CDC chunker for content-defined chunking
pub struct CdcChunker {
    config: ChunkingConfig,
    /// Cache for chunk digests
    chunk_cache: HashMap<Digest, Chunk>,
}

impl CdcChunker {
    /// Create a new CDC chunker with default configuration
    pub fn new() -> Self {
        Self {
            config: ChunkingConfig::default(),
            chunk_cache: HashMap::new(),
        }
    }

    /// Create a new CDC chunker with custom configuration
    pub fn with_config(config: ChunkingConfig) -> Self {
        Self {
            config,
            chunk_cache: HashMap::new(),
        }
    }

    /// Chunk content using CDC
    pub fn chunk_content(&mut self, content: &[u8]) -> Result<ChunkList> {
        if content.is_empty() {
            return Ok(ChunkList {
                chunks: Vec::new(),
                total_length: 0,
                file_digest: Digest::from_bytes([0; 32]),
            });
        }

        let mut chunks = Vec::new();
        let mut chunk_digests = Vec::new();

        // Use FastCDC for chunking
        let fastcdc = FastCDC::new(
            content,
            self.config.min_size.try_into().unwrap(),
            self.config.avg_size.try_into().unwrap(),
            self.config.max_size.try_into().unwrap(),
        );

        for chunk_data in fastcdc {
            let chunk = self.create_chunk(&chunk_data, content)?;
            chunk_digests.push(chunk.digest);
            chunks.push(chunk);
        }

        // Compute file digest from chunk digests
        let file_digest = self.compute_file_digest(&chunk_digests)?;

        Ok(ChunkList {
            chunks: chunks.into_iter().map(|chunk_ref| Chunk {
                digest: chunk_ref.digest,
                offset: chunk_ref.offset as usize,
                length: chunk_ref.length as usize,
                data: Vec::new(), // TODO: Store actual data if needed
            }).collect(),
            total_length: content.len(),
            file_digest,
        })
    }

    /// Create a chunk from chunk data
    fn create_chunk(&mut self, chunk_data: &fastcdc::v2020::Chunk, content: &[u8]) -> Result<ChunkRef> {
        let offset = chunk_data.offset as usize;
        let length = chunk_data.length as usize;
        let data = content[offset..offset + length].to_vec();

        // Compute chunk digest
        let digest = self.compute_chunk_digest(&data)?;

        let chunk_ref = ChunkRef {
            digest,
            offset: chunk_data.offset as u64,
            length: chunk_data.length as u32,
        };

        Ok(chunk_ref)
    }

    /// Compute digest for a chunk
    fn compute_chunk_digest(&self, data: &[u8]) -> Result<Digest> {
        let mut hasher = StreamingHasher::new();
        hasher.update(data);
        Ok(hasher.finalize())
    }

    /// Compute file digest from chunk digests
    fn compute_file_digest(&self, chunk_digests: &[Digest]) -> Result<Digest> {
        if chunk_digests.is_empty() {
            return Ok(Digest::from_bytes([0; 32]));
        }

        let mut hasher = StreamingHasher::new();
        for digest in chunk_digests {
            hasher.update(digest.as_bytes());
        }
        Ok(hasher.finalize())
    }

    /// Reconstruct content from chunks
    pub fn reconstruct_content(&self, chunk_list: &ChunkList) -> Result<Vec<u8>> {
        let mut content = Vec::with_capacity(chunk_list.total_length);
        
        // Sort chunks by offset
        let mut sorted_chunks = chunk_list.chunks.clone();
        sorted_chunks.sort_by_key(|chunk| chunk.offset);

        // Verify chunks are contiguous
        let mut expected_offset = 0;
        for chunk in &sorted_chunks {
            if chunk.offset != expected_offset {
                return Err(anyhow!(
                    "Chunk at offset {} is not contiguous with previous chunk ending at {}",
                    chunk.offset,
                    expected_offset
                ));
            }
            expected_offset = chunk.offset + chunk.length;
        }

        // Reconstruct content
        for chunk in &sorted_chunks {
            content.extend_from_slice(&chunk.data);
        }

        // Verify file digest
        let reconstructed_digest = self.compute_file_digest(&chunk_list.chunks.iter().map(|c| c.digest).collect::<Vec<_>>())?;
        if reconstructed_digest != chunk_list.file_digest {
            return Err(anyhow!("File digest mismatch during reconstruction"));
        }

        Ok(content)
    }

    /// Get chunk by digest
    pub fn get_chunk(&self, digest: &Digest) -> Option<&Chunk> {
        self.chunk_cache.get(digest)
    }

    /// Check if chunk exists
    pub fn has_chunk(&self, digest: &Digest) -> bool {
        self.chunk_cache.contains_key(digest)
    }

    /// Get chunk statistics
    pub fn get_chunk_stats(&self, chunk_list: &ChunkList) -> ChunkStats {
        let chunk_sizes: Vec<usize> = chunk_list.chunks.iter().map(|c| c.length).collect();
        let total_chunks = chunk_list.chunks.len();
        
        let min_size = chunk_sizes.iter().min().copied().unwrap_or(0);
        let max_size = chunk_sizes.iter().max().copied().unwrap_or(0);
        let avg_size = if total_chunks > 0 {
            chunk_sizes.iter().sum::<usize>() / total_chunks
        } else {
            0
        };

        ChunkStats {
            total_chunks,
            min_size,
            max_size,
            avg_size,
            total_size: chunk_list.total_length,
        }
    }

    /// Get the current configuration
    pub fn config(&self) -> &ChunkingConfig {
        &self.config
    }

    /// Update the configuration
    pub fn update_config(&mut self, config: ChunkingConfig) {
        self.config = config;
        self.chunk_cache.clear();
    }
}

/// Chunk statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkStats {
    /// Total number of chunks
    pub total_chunks: usize,
    /// Minimum chunk size
    pub min_size: usize,
    /// Maximum chunk size
    pub max_size: usize,
    /// Average chunk size
    pub avg_size: usize,
    /// Total content size
    pub total_size: usize,
}

/// Gear-based rolling hash for CDC
pub struct GearHash {
    /// Current hash value
    hash: u64,
    /// Polynomial for the hash
    polynomial: u64,
    /// Mask for the hash
    mask: u64,
    /// Window size
    window_size: usize,
    /// Rolling window
    window: Vec<u8>,
    /// Window index
    window_index: usize,
}

impl GearHash {
    /// Create a new gear hash
    pub fn new(polynomial: u64, mask: u64, window_size: usize) -> Self {
        Self {
            hash: 0,
            polynomial,
            mask,
            window_size,
            window: vec![0; window_size],
            window_index: 0,
        }
    }

    /// Update hash with a byte
    pub fn update(&mut self, byte: u8) {
        // Remove old byte from hash
        let old_byte = self.window[self.window_index];
        self.hash = self.hash.wrapping_sub(old_byte as u64);

        // Add new byte to hash
        self.hash = self.hash.wrapping_add(byte as u64);
        self.hash = self.hash.wrapping_mul(self.polynomial);
        self.hash &= self.mask;

        // Update window
        self.window[self.window_index] = byte;
        self.window_index = (self.window_index + 1) % self.window_size;
    }

    /// Get current hash value
    pub fn hash(&self) -> u64 {
        self.hash
    }

    /// Reset the hash
    pub fn reset(&mut self) {
        self.hash = 0;
        self.window.fill(0);
        self.window_index = 0;
    }
}

/// Chunk deduplication store
pub struct ChunkStore {
    /// Storage for chunks
    chunks: HashMap<Digest, Chunk>,
    /// Statistics
    stats: ChunkStats,
}

impl ChunkStore {
    /// Create a new chunk store
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            stats: ChunkStats {
                total_chunks: 0,
                min_size: 0,
                max_size: 0,
                avg_size: 0,
                total_size: 0,
            },
        }
    }

    /// Store a chunk
    pub fn store_chunk(&mut self, chunk: Chunk) -> Digest {
        let digest = chunk.digest;
        self.chunks.insert(digest, chunk);
        self.update_stats();
        digest
    }

    /// Get a chunk by digest
    pub fn get_chunk(&self, digest: &Digest) -> Option<&Chunk> {
        self.chunks.get(digest)
    }

    /// Check if chunk exists
    pub fn has_chunk(&self, digest: &Digest) -> bool {
        self.chunks.contains_key(digest)
    }

    /// Remove a chunk
    pub fn remove_chunk(&mut self, digest: &Digest) -> Option<Chunk> {
        let chunk = self.chunks.remove(digest);
        if chunk.is_some() {
            self.update_stats();
        }
        chunk
    }

    /// Get all chunks
    pub fn list_chunks(&self) -> Vec<&Chunk> {
        self.chunks.values().collect()
    }

    /// Get chunk statistics
    pub fn stats(&self) -> &ChunkStats {
        &self.stats
    }

    /// Update statistics
    fn update_stats(&mut self) {
        if self.chunks.is_empty() {
            self.stats = ChunkStats {
                total_chunks: 0,
                min_size: 0,
                max_size: 0,
                avg_size: 0,
                total_size: 0,
            };
            return;
        }

        let chunk_sizes: Vec<usize> = self.chunks.values().map(|c| c.length).collect();
        let total_chunks = self.chunks.len();
        
        let min_size = chunk_sizes.iter().min().copied().unwrap_or(0);
        let max_size = chunk_sizes.iter().max().copied().unwrap_or(0);
        let avg_size = if total_chunks > 0 {
            chunk_sizes.iter().sum::<usize>() / total_chunks
        } else {
            0
        };
        let total_size = chunk_sizes.iter().sum();

        self.stats = ChunkStats {
            total_chunks,
            min_size,
            max_size,
            avg_size,
            total_size,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cdc_chunking() {
        let mut chunker = CdcChunker::new();
        let content = b"Hello, world! This is a test content for chunking.";
        
        let chunk_list = chunker.chunk_content(content).unwrap();
        
        assert!(!chunk_list.chunks.is_empty());
        assert_eq!(chunk_list.total_length, content.len());
        
        // Verify reconstruction
        let reconstructed = chunker.reconstruct_content(&chunk_list).unwrap();
        assert_eq!(reconstructed, content);
    }

    #[test]
    fn test_chunk_deduplication() {
        let mut chunker = CdcChunker::new();
        let content1 = b"Hello, world! This is a test.";
        let content2 = b"Hello, world! This is another test.";
        
        let chunk_list1 = chunker.chunk_content(content1).unwrap();
        let chunk_list2 = chunker.chunk_content(content2).unwrap();
        
        // Should have some shared chunks
        let chunks1: std::collections::HashSet<Digest> = chunk_list1.chunks.iter().map(|c| c.digest).collect();
        let chunks2: std::collections::HashSet<Digest> = chunk_list2.chunks.iter().map(|c| c.digest).collect();
        
        let intersection: std::collections::HashSet<_> = chunks1.intersection(&chunks2).collect();
        assert!(!intersection.is_empty());
    }

    #[test]
    fn test_gear_hash() {
        let mut gear_hash = GearHash::new(0x9e3779b97f4a7c15, 0x1fffffffffffffff, 4);
        
        // Test rolling hash
        for &byte in b"Hello" {
            gear_hash.update(byte);
        }
        
        let hash1 = gear_hash.hash();
        
        // Reset and hash again
        gear_hash.reset();
        for &byte in b"Hello" {
            gear_hash.update(byte);
        }
        
        let hash2 = gear_hash.hash();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_chunk_store() {
        let mut store = ChunkStore::new();
        
        let chunk = Chunk {
            digest: Digest::from_bytes(&[1, 2, 3, 4]),
            offset: 0,
            length: 10,
            data: b"Hello, world!".to_vec(),
        };
        
        let digest = store.store_chunk(chunk.clone());
        assert!(store.has_chunk(&digest));
        assert_eq!(store.get_chunk(&digest).unwrap().length, 10);
        
        let stats = store.stats();
        assert_eq!(stats.total_chunks, 1);
        assert_eq!(stats.total_size, 10);
    }

    #[test]
    fn test_empty_content() {
        let mut chunker = CdcChunker::new();
        let content = b"";
        
        let chunk_list = chunker.chunk_content(content).unwrap();
        assert!(chunk_list.chunks.is_empty());
        assert_eq!(chunk_list.total_length, 0);
    }

    #[test]
    fn test_small_content() {
        let mut chunker = CdcChunker::new();
        let content = b"Hi";
        
        let chunk_list = chunker.chunk_content(content).unwrap();
        assert_eq!(chunk_list.chunks.len(), 1);
        assert_eq!(chunk_list.total_length, content.len());
    }
}
