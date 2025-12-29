//! Source integrity service implementation
//!
//! This module provides the main service for source integrity verification.

use crate::{
    hasher::ContentHasher,
    integrity_types::*,
    storage_new::{NewSourceIntegrityStorage, PostgresSourceIntegrityStorage},
    tampering_detector::{TamperingDetectionResult, TamperingDetector},
};
use anyhow::Result;
use uuid::Uuid;

/// Main service for source integrity verification
pub struct SourceIntegrityService {
    storage: Box<dyn NewSourceIntegrityStorage>,
    hasher: ContentHasher,
    tampering_detector: TamperingDetector,
    config: SourceIntegrityConfig,
}

impl SourceIntegrityService {
    /// Create a new source integrity service
    pub fn new(storage: Box<dyn NewSourceIntegrityStorage>, config: SourceIntegrityConfig) -> Self {
        let hasher = ContentHasher::new(config.default_hash_algorithm.clone());
        let tampering_detector = TamperingDetector::new();

        Self {
            storage,
            hasher,
            tampering_detector,
            config,
        }
    }

    /// Verify source integrity
    pub async fn verify_integrity(
        &self,
        source_id: &str,
        source_type: SourceType,
        content: &[u8],
        metadata: Option<serde_json::Value>,
    ) -> Result<IntegrityVerificationResult> {
        // Compute cryptographic hash of content
        let start_time = std::time::Instant::now();
        // Convert content to string for hashing (ContentHasher expects &str)
        let content_str = String::from_utf8_lossy(content);
        let content_hash = self.hasher.calculate_hash(&content_str)?;
        let hash_duration = start_time.elapsed().as_millis() as u64;
        
        // Retrieve stored hash from database for source_id
        let stored_records = self.storage.get_records_by_source(source_id).await?;
        
        let (stored_hash, integrity_status, tampering_indicators, tampering_detected) = if let Some(latest_record) = stored_records.iter().max_by_key(|r| r.created_at) {
            // Compare computed hash with stored hash
            let hashes_match = latest_record.content_hash == content_hash;
            
            // Verify hash algorithm matches stored algorithm
            let algorithm_matches = latest_record.hash_algorithm == self.config.default_hash_algorithm;
            
            if hashes_match && algorithm_matches {
                (Some(latest_record.content_hash.clone()), IntegrityStatus::Verified, vec![], false)
            } else {
                // Hash mismatch or algorithm mismatch - potential tampering
                let mut indicators = vec![];
                if !hashes_match {
                    indicators.push(TamperingIndicator::HashMismatch);
                }
                if !algorithm_matches {
                    indicators.push(TamperingIndicator::MetadataInconsistency);
                }
                
                // Use tampering detector for additional analysis
                let content_str = String::from_utf8_lossy(content);
                let metadata_map = metadata
                    .as_ref()
                    .and_then(|m| m.as_object())
                    .map(|obj| {
                        obj.iter()
                            .map(|(k, v)| (k.clone(), v.clone()))
                            .collect()
                    })
                    .unwrap_or_default();
                
                let tampering_result = self.tampering_detector.detect_tampering(
                    &content_str,
                    &latest_record.content_hash,
                    Some(latest_record.content_size),
                    &source_type,
                    &metadata_map,
                ).await?;
                
                // Merge tampering indicators from detector
                let mut all_indicators = indicators;
                all_indicators.extend(tampering_result.indicators);
                
                // If we have indicators, we detected tampering
                let tampering_detected = !all_indicators.is_empty();
                let status = if tampering_detected {
                    IntegrityStatus::Tampered
                } else {
                    IntegrityStatus::Unknown
                };
                
                (Some(latest_record.content_hash.clone()), status, all_indicators, tampering_detected)
            }
        } else {
            // First-time verification - no stored hash to compare
            (None, IntegrityStatus::Verified, vec![], false)
        };
        
        let content_size = content.len() as i64; // Convert to i64

        // Create integrity record
        let integrity_record = CreateSourceIntegrityRecord {
            source_id: source_id.to_string(),
            source_type,
            content_hash: content_hash.clone(),
            content_size,
            hash_algorithm: self.config.default_hash_algorithm.clone(),
            integrity_status: IntegrityStatus::Verified,
            tampering_indicators: tampering_indicators.clone(),
            verification_metadata: metadata
                .map(|m| {
                    if let serde_json::Value::Object(map) = m {
                        map.into_iter().collect()
                    } else {
                        std::collections::HashMap::new()
                    }
                })
                .unwrap_or_default(),
        };

        // Store the record
        let record_id = self.storage.store_record(&integrity_record).await?;

        // Create result
        Ok(IntegrityVerificationResult {
            verified: integrity_status == IntegrityStatus::Verified,
            tampering_detected,
            calculated_hash: content_hash.clone(),
            stored_hash,
            integrity_status,
            tampering_indicators,
            verification_timestamp: chrono::Utc::now(),
            verification_duration_ms: Some(hash_duration as i32),
            verification_details: {
                let mut details = std::collections::HashMap::new();
                details.insert("hash_duration_ms".to_string(), serde_json::Value::Number(serde_json::Number::from(hash_duration)));
                details.insert("hash_algorithm".to_string(), serde_json::Value::String(self.config.default_hash_algorithm.to_string()));
                details
            },
        })
    }

    /// Get integrity record by ID
    pub async fn get_record(&self, id: &Uuid) -> Result<Option<SourceIntegrityRecord>> {
        self.storage.get_record(id).await
    }

    /// List all integrity records
    pub async fn list_records(&self) -> Result<Vec<SourceIntegrityRecord>> {
        self.storage.list_records().await
    }

    /// Get records by source
    pub async fn get_records_by_source(
        &self,
        source_id: &str,
    ) -> Result<Vec<SourceIntegrityRecord>> {
        self.storage.get_records_by_source(source_id).await
    }

    /// Get records by status
    pub async fn get_records_by_status(
        &self,
        status: IntegrityStatus,
    ) -> Result<Vec<SourceIntegrityRecord>> {
        self.storage.get_records_by_status(status).await
    }

    /// Delete a record
    pub async fn delete_record(&self, id: &Uuid) -> Result<()> {
        self.storage.delete_record(id).await
    }

    /// Get integrity statistics
    pub async fn get_statistics(&self) -> Result<()> {
        self.storage.get_integrity_stats().await
    }

    /// Cleanup old records
    pub async fn cleanup_old_records(
        &self,
        older_than: chrono::DateTime<chrono::Utc>,
    ) -> Result<usize> {
        self.storage.cleanup_old_records(older_than).await
    }
}
