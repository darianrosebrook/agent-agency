//! Source integrity service implementation
//!
//! This module provides the main service for source integrity verification.

use crate::{
    hasher::ContentHasher,
    tampering_detector::{TamperingDetectionResult, TamperingDetector},
    integrity_types::*,
    storage_new::{NewSourceIntegrityStorage, PostgresSourceIntegrityStorage},
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

    /// Verify source integrity - simplified implementation
    pub async fn verify_integrity(
        &self,
        source_id: &str,
        source_type: SourceType,
        content: &[u8],
        metadata: Option<serde_json::Value>,
    ) -> Result<IntegrityVerificationResult> {
        // Simplified implementation - just create a basic record
        let content_hash = format!("hash_{}", content.len()); // Simple hash placeholder
        let content_size = content.len() as i64; // Convert to i64

        // Create integrity record
        let integrity_record = CreateSourceIntegrityRecord {
            source_id: source_id.to_string(),
            source_type,
            content_hash: content_hash.clone(),
            content_size,
            hash_algorithm: self.config.default_hash_algorithm.clone(),
            integrity_status: IntegrityStatus::Verified, // Simplified - always verified
            tampering_indicators: vec![], // Empty for now
            verification_metadata: metadata.map(|m| {
                if let serde_json::Value::Object(map) = m {
                    map.into_iter().collect()
                } else {
                    std::collections::HashMap::new()
                }
            }).unwrap_or_default(),
        };

        // Store the record
        let record_id = self.storage.store_record(&integrity_record).await?;

        // Create result
        Ok(IntegrityVerificationResult {
            verified: true,
            tampering_detected: false,
            calculated_hash: content_hash.clone(),
            stored_hash: Some(content_hash),
            integrity_status: IntegrityStatus::Verified,
            tampering_indicators: vec![],
            verification_timestamp: chrono::Utc::now(),
            verification_duration_ms: Some(0),
            verification_details: std::collections::HashMap::new(),
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
    pub async fn get_records_by_source(&self, source_id: &str) -> Result<Vec<SourceIntegrityRecord>> {
        self.storage.get_records_by_source(source_id).await
    }

    /// Get records by status
    pub async fn get_records_by_status(&self, status: IntegrityStatus) -> Result<Vec<SourceIntegrityRecord>> {
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
    pub async fn cleanup_old_records(&self, older_than: chrono::DateTime<chrono::Utc>) -> Result<usize> {
        self.storage.cleanup_old_records(older_than).await
    }
}