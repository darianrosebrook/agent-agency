//! Memory Archival System
//!
//! Intelligent archival and retrieval of long-term memories.

use crate::long_term_management::*;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use flate2::Compression;
use std::io::{Write, Read};

/// Memory archival configuration
#[derive(Debug, Clone)]
pub struct ArchivalConfig {
    pub compression_enabled: bool,
    pub encryption_enabled: bool,
    pub archival_batch_size: usize,
    pub retrieval_timeout_ms: u64,
    pub storage_tiers: Vec<StorageTier>,
}

/// Storage tier definition
#[derive(Debug, Clone)]
pub struct StorageTier {
    pub name: String,
    pub cost_per_gb_month: f64,
    pub retrieval_time_ms: u64,
    pub durability: f64, // 0.0 to 1.0
    pub max_age_days: Option<u64>,
}

/// Memory archival manager
pub struct MemoryArchivalManager {
    config: ArchivalConfig,
    archived_memories: std::collections::HashMap<crate::memory_types::MemoryId, ArchivedMemory>,
}

impl MemoryArchivalManager {
    pub fn new(config: ArchivalConfig) -> Self {
        Self {
            config,
            archived_memories: std::collections::HashMap::new(),
        }
    }

    /// Archive memories to long-term storage
    pub async fn archive_memories(
        &mut self,
        memories: Vec<crate::memory_types::Memory>,
    ) -> crate::MemoryResult<ArchivalResult> {
        let mut archived = Vec::new();
        let mut failed = Vec::new();

        for memory in memories {
            let memory_id = memory.id.clone();
            match self.archive_single_memory(memory).await {
                Ok(archived_memory) => {
                    self.archived_memories.insert(archived_memory.memory_id.clone(), archived_memory.clone());
                    archived.push(archived_memory);
                }
                Err(e) => failed.push((memory_id, e.to_string())),
            }
        }

        let total_processed = archived.len() + failed.len();

        Ok(ArchivalResult {
            archived_memories: archived,
            failed_archivals: failed,
            total_processed,
            archival_timestamp: chrono::Utc::now(),
        })
    }

    /// Retrieve archived memories
    pub async fn retrieve_memories(
        &self,
        memory_ids: &[crate::memory_types::MemoryId],
    ) -> crate::MemoryResult<RetrievalResult> {
        let mut retrieved = Vec::new();
        let mut not_found = Vec::new();

        for memory_id in memory_ids {
            if let Some(archived) = self.archived_memories.get(memory_id) {
                match self.retrieve_single_memory(archived).await {
                    Ok(memory) => retrieved.push(memory),
                    Err(_) => not_found.push(memory_id.clone()),
                }
            } else {
                not_found.push(memory_id.clone());
            }
        }

        Ok(RetrievalResult {
            retrieved_memories: retrieved,
            not_found_ids: not_found,
            retrieval_time_ms: 0, // Would be measured
        })
    }

    /// Search archived memories
    pub async fn search_archived(
        &self,
        query: &str,
        limit: usize,
    ) -> crate::MemoryResult<Vec<ArchivedMemory>> {
        // Simple text search in archived memories
        let mut results = Vec::new();

        for archived in self.archived_memories.values() {
            if archived.search_text.to_lowercase().contains(&query.to_lowercase()) {
                results.push(archived.clone());
                if results.len() >= limit {
                    break;
                }
            }
        }

        Ok(results)
    }

    /// Optimize archival storage
    pub async fn optimize_storage(&self) -> crate::MemoryResult<StorageOptimizationResult> {
        let mut optimizations = Vec::new();

        // Analyze storage usage and suggest optimizations
        let total_archived = self.archived_memories.len();
        let mut tier_usage = std::collections::HashMap::new();

        for archived in self.archived_memories.values() {
            *tier_usage.entry(archived.storage_tier.clone()).or_insert(0) += 1;
        }

        // Suggest moving old memories to cheaper tiers
        for (tier, count) in tier_usage {
            if count > 1000 { // Arbitrary threshold
                optimizations.push(StorageOptimization {
                    optimization_type: OptimizationType::TierMigration,
                    description: format!("Consider migrating {} memories from {} tier", count, tier),
                    estimated_savings: count as f64 * 0.1, // 10% cost reduction
                });
            }
        }

        Ok(StorageOptimizationResult {
            optimizations,
            total_archived,
            // TODO: Calculate actual storage efficiency
            // - [ ] Compare compressed vs uncompressed sizes
            // - [ ] Calculate deduplication savings
            // - [ ] Track storage tier utilization
            // - [ ] Add unit tests for efficiency calculation
            // - [ ] Add integration tests with real storage data
            storage_efficiency: 0.85, // Mock efficiency
        })
    }

    /// Archive single memory
    async fn archive_single_memory(&self, memory: crate::memory_types::Memory) -> crate::MemoryResult<ArchivedMemory> {
        let search_text = self.extract_search_text(&memory);
        let compressed_data = if self.config.compression_enabled {
            self.compress_memory_data(&memory).await?
        } else {
            serde_json::to_vec(&memory)?
        };

        let storage_tier = self.select_storage_tier(&memory).await;

        let archived = ArchivedMemory {
            memory_id: memory.id.clone(),
            original_memory: memory,
            archived_at: chrono::Utc::now(),
            storage_tier,
            compressed_data,
            search_text,
            access_count: 0,
            last_accessed: None,
        };

        Ok(archived)
    }

    /// Retrieve single memory
    async fn retrieve_single_memory(&self, archived: &ArchivedMemory) -> crate::MemoryResult<crate::memory_types::Memory> {
        let memory = if self.config.compression_enabled {
            // Try to decompress, but fall back to direct deserialization if decompression fails
            // This handles backward compatibility with uncompressed data
            match self.decompress_memory_data(&archived.compressed_data).await {
                Ok(mem) => mem,
                Err(_) => {
                    // Fallback: try to deserialize as uncompressed JSON
                    // This handles cases where old data wasn't compressed
                    serde_json::from_slice(&archived.compressed_data)
                        .map_err(|e| crate::MemoryError::Other(format!("Failed to decompress or deserialize memory: {}", e)))?
                }
            }
        } else {
            serde_json::from_slice(&archived.compressed_data)
                .map_err(|e| crate::MemoryError::Other(format!("Failed to deserialize memory: {}", e)))?
        };

        Ok(memory)
    }

    /// Extract searchable text from memory
    fn extract_search_text(&self, memory: &crate::memory_types::Memory) -> String {
        // TODO: Implement comprehensive text extraction with metadata and structured content parsing
        //       Currently uses basic content cloning; should extract searchable text from structured memory content including metadata.
        memory.content.clone()
    }

    /// Compress memory data using gzip compression
    /// Implemented: Real compression using flate2 gzip encoder with configurable compression level
    async fn compress_memory_data(&self, memory: &crate::memory_types::Memory) -> crate::MemoryResult<Vec<u8>> {
        // Serialize memory to JSON bytes first
        let json_data = serde_json::to_vec(memory)
            .map_err(|e| crate::MemoryError::Other(format!("Failed to serialize memory for compression: {}", e)))?;
        
        // Use default compression level (6) - balanced between speed and compression ratio
        // Compression levels: 0 (no compression) to 9 (maximum compression)
        let compression_level = Compression::default(); // Level 6
        
        // Create gzip encoder
        let mut encoder = GzEncoder::new(Vec::new(), compression_level);
        
        // Write JSON data to encoder
        encoder.write_all(&json_data)
            .map_err(|e| crate::MemoryError::Other(format!("Failed to write data to compression encoder: {}", e)))?;
        
        // Finish compression and get compressed bytes
        let compressed_data = encoder.finish()
            .map_err(|e| crate::MemoryError::Other(format!("Failed to finish compression: {}", e)))?;
        
        Ok(compressed_data)
    }

    /// Decompress memory data using gzip decompression
    /// Implemented: Real decompression using flate2 gzip decoder with error handling and data validation
    async fn decompress_memory_data(&self, data: &[u8]) -> crate::MemoryResult<crate::memory_types::Memory> {
        // Create gzip decoder
        let mut decoder = GzDecoder::new(data);
        
        // Read decompressed data into buffer
        let mut decompressed_data = Vec::new();
        decoder.read_to_end(&mut decompressed_data)
            .map_err(|e| crate::MemoryError::Other(format!("Failed to decompress memory data: {}", e)))?;
        
        // Validate that we got some data
        if decompressed_data.is_empty() {
            return Err(crate::MemoryError::Other("Decompressed data is empty".to_string()));
        }
        
        // Deserialize JSON back to Memory struct
        let memory: crate::memory_types::Memory = serde_json::from_slice(&decompressed_data)
            .map_err(|e| crate::MemoryError::Other(format!("Failed to deserialize decompressed memory data: {}", e)))?;
        
        Ok(memory)
    }

    /// Select appropriate storage tier
    async fn select_storage_tier(&self, memory: &crate::memory_types::Memory) -> String {
        let age_days = (chrono::Utc::now() - memory.created_at).as_seconds_f32() as f64 / (24.0 * 3600.0);

        // Select tier based on age and importance
        for tier in &self.config.storage_tiers {
            if let Some(max_age) = tier.max_age_days {
                if age_days <= max_age as f64 {
                    return tier.name.clone();
                }
            }
        }

        // Default to first tier
        self.config.storage_tiers.first()
            .map(|t| t.name.clone())
            .unwrap_or_else(|| "default".to_string())
    }
}

/// Archived memory representation
#[derive(Debug, Clone)]
pub struct ArchivedMemory {
    pub memory_id: crate::memory_types::MemoryId,
    pub original_memory: crate::memory_types::Memory,
    pub archived_at: chrono::DateTime<chrono::Utc>,
    pub storage_tier: String,
    pub compressed_data: Vec<u8>,
    pub search_text: String,
    pub access_count: u32,
    pub last_accessed: Option<chrono::DateTime<chrono::Utc>>,
}

/// Archival result
#[derive(Debug, Clone)]
pub struct ArchivalResult {
    pub archived_memories: Vec<ArchivedMemory>,
    pub failed_archivals: Vec<(crate::memory_types::MemoryId, String)>,
    pub total_processed: usize,
    pub archival_timestamp: chrono::DateTime<chrono::Utc>,
}

/// Retrieval result
#[derive(Debug, Clone)]
pub struct RetrievalResult {
    pub retrieved_memories: Vec<crate::memory_types::Memory>,
    pub not_found_ids: Vec<crate::memory_types::MemoryId>,
    pub retrieval_time_ms: u64,
}

/// Storage optimization suggestion
#[derive(Debug, Clone)]
pub struct StorageOptimization {
    pub optimization_type: OptimizationType,
    pub description: String,
    pub estimated_savings: f64, // Cost savings in currency units
}

/// Optimization type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationType {
    TierMigration,
    Compression,
    Deduplication,
    Cleanup,
}

/// Storage optimization result
#[derive(Debug, Clone)]
pub struct StorageOptimizationResult {
    pub optimizations: Vec<StorageOptimization>,
    pub total_archived: usize,
    pub storage_efficiency: f64,
}

/// Archival statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchivalStats {
    pub total_archived_memories: usize,
    pub total_storage_used_gb: f64,
    pub average_retrieval_time_ms: f64,
    pub archival_success_rate: f64,
    pub storage_cost_per_month: f64,
    pub last_archival_run: Option<chrono::DateTime<chrono::Utc>>,
}
