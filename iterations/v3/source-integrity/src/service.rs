//! Main source integrity verification service
//!
//! @author @darianrosebrook

use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::time::Instant;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{
    hasher::ContentHasher,
    storage::{PostgresSourceIntegrityStorage, SourceIntegrityStorage},
    tampering_detector::{TamperingDetectionResult, TamperingDetector},
    types::*,
};

/// Main service for source integrity verification
pub struct SourceIntegrityService {
    storage: Box<dyn SourceIntegrityStorage>,
    hasher: ContentHasher,
    tampering_detector: TamperingDetector,
    config: SourceIntegrityConfig,
}

impl SourceIntegrityService {
    /// Create a new source integrity service
    pub fn new(storage: Box<dyn SourceIntegrityStorage>, config: SourceIntegrityConfig) -> Self {
        let hasher = ContentHasher::new(config.default_hash_algorithm.clone());
        let tampering_detector = TamperingDetector::new();

        Self {
            storage,
            hasher,
            tampering_detector,
            config,
        }
    }

    /// Create a new service with PostgreSQL storage
    pub fn with_postgres_storage(pool: sqlx::PgPool, config: SourceIntegrityConfig) -> Self {
        let storage = Box::new(PostgresSourceIntegrityStorage::new(pool));
        Self::new(storage, config)
    }

    /// Verify source integrity with comprehensive analysis
    ///
    /// # Arguments
    /// * `source_id` - Unique identifier for the source
    /// * `source_type` - Type of source being verified
    /// * `content` - The content to verify
    /// * `verification_type` - Type of verification being performed
    /// * `metadata` - Additional metadata for analysis
    ///
    /// # Returns
    /// * `Result<IntegrityVerificationResult>` - Comprehensive verification result
    pub async fn verify_source_integrity(
        &self,
        source_id: &str,
        source_type: &SourceType,
        content: &str,
        verification_type: &VerificationType,
        metadata: &HashMap<String, serde_json::Value>,
    ) -> Result<IntegrityVerificationResult> {
        let start_time = Instant::now();
        debug!(
            "Starting source integrity verification for source: {}",
            source_id
        );

        // Calculate content hash
        let (calculated_hash, hash_duration_ms) =
            self.hasher.calculate_hash_with_timing(content)?;
        debug!(
            "Content hash calculated: {} (took {}ms)",
            calculated_hash, hash_duration_ms
        );

        // Get existing record if it exists
        let existing_record = self
            .storage
            .get_record_by_source(source_id, source_type)
            .await?;

        let stored_hash = existing_record.as_ref().map(|r| r.content_hash.clone());
        let stored_size = existing_record.as_ref().map(|r| r.content_size);

        // Perform tampering detection
        let tampering_result = if self.config.tampering_detection_enabled {
            self.tampering_detector
                .detect_tampering(
                    content,
                    stored_hash.as_deref().unwrap_or(""),
                    stored_size,
                    source_type,
                    metadata,
                )
                .await?
        } else {
            TamperingDetectionResult {
                indicators: Vec::new(),
                confidence_score: 0.0,
                analysis_details: HashMap::new(),
                detection_time_ms: 0,
            }
        };

        // Determine if content is verified
        let hash_match = stored_hash
            .as_ref()
            .map_or(true, |hash| hash == &calculated_hash);
        let tampering_detected = !tampering_result.indicators.is_empty();
        let verified = hash_match && !tampering_detected;

        // Determine integrity status
        let integrity_status = if verified {
            IntegrityStatus::Verified
        } else if tampering_detected {
            IntegrityStatus::Tampered
        } else {
            IntegrityStatus::Unknown
        };

        // Update or create source integrity record
        let record_id = if let Some(mut existing_record) = existing_record {
            // Update existing record
            existing_record.content_hash = calculated_hash.clone();
            existing_record.content_size = content.len() as i64;
            existing_record.integrity_status = integrity_status.clone();
            existing_record.tampering_indicators = tampering_result.indicators.clone();
            existing_record.last_verified_at = Some(Utc::now());
            existing_record.verification_count += 1;

            // Merge verification metadata
            for (key, value) in metadata {
                existing_record
                    .verification_metadata
                    .insert(key.clone(), value.clone());
            }

            self.storage.update_record(&existing_record).await?;
            existing_record.id
        } else {
            // Create new record
            let new_record = CreateSourceIntegrityRecord {
                source_id: source_id.to_string(),
                source_type: source_type.clone(),
                content_hash: calculated_hash.clone(),
                content_size: content.len() as i64,
                hash_algorithm: self.config.default_hash_algorithm.clone(),
                integrity_status: integrity_status.clone(),
                tampering_indicators: tampering_result.indicators.clone(),
                verification_metadata: metadata.clone(),
            };

            self.storage.store_record(&new_record).await?
        };

        // Store verification attempt
        let verification_result_type = if verified {
            VerificationResult::Passed
        } else if tampering_detected {
            VerificationResult::Failed
        } else {
            VerificationResult::Warning
        };

        let verification_duration_ms = start_time.elapsed().as_millis() as i32;

        let verification_record = CreateSourceIntegrityVerification {
            source_integrity_id: record_id,
            verification_type: verification_type.clone(),
            verification_result: verification_result_type,
            calculated_hash: calculated_hash.clone(),
            stored_hash: stored_hash.as_ref().map(|s| s.clone()).unwrap_or_default(),
            hash_match,
            tampering_detected,
            verification_details: tampering_result.analysis_details.clone(),
            verified_by: Some("source_integrity_service".to_string()),
            verification_duration_ms: Some(verification_duration_ms),
        };

        self.storage
            .store_verification(&verification_record)
            .await?;

        // Create alert if tampering detected and alerts are enabled
        if tampering_detected && self.config.alert_on_tampering {
            self.create_tampering_alert(&record_id, &tampering_result)
                .await?;
        }

        let total_duration_ms = start_time.elapsed().as_millis() as i32;

        info!(
            "Source integrity verification completed for {}: verified={}, tampering_detected={}, duration={}ms",
            source_id, verified, tampering_detected, total_duration_ms
        );

        Ok(IntegrityVerificationResult {
            verified,
            tampering_detected,
            calculated_hash,
            stored_hash: stored_hash.as_ref().map(|s| s.clone()),
            integrity_status,
            tampering_indicators: tampering_result.indicators,
            verification_timestamp: Utc::now(),
            verification_duration_ms: Some(total_duration_ms),
            verification_details: tampering_result.analysis_details,
        })
    }

    /// Get source integrity record
    ///
    /// # Arguments
    /// * `source_id` - Unique identifier for the source
    /// * `source_type` - Type of source
    ///
    /// # Returns
    /// * `Result<Option<SourceIntegrityRecord>>` - The integrity record if found
    pub async fn get_source_record(
        &self,
        source_id: &str,
        source_type: &SourceType,
    ) -> Result<Option<SourceIntegrityRecord>> {
        self.storage
            .get_record_by_source(source_id, source_type)
            .await
    }

    /// Get verification history for a source
    ///
    /// # Arguments
    /// * `source_id` - Unique identifier for the source
    /// * `source_type` - Type of source
    /// * `limit` - Maximum number of verifications to return
    ///
    /// # Returns
    /// * `Result<Vec<SourceIntegrityVerification>>` - Verification history
    pub async fn get_verification_history(
        &self,
        source_id: &str,
        source_type: &SourceType,
        limit: Option<i32>,
    ) -> Result<Vec<SourceIntegrityVerification>> {
        if let Some(record) = self.get_source_record(source_id, source_type).await? {
            self.storage
                .get_verification_history(&record.id, limit)
                .await
        } else {
            Ok(Vec::new())
        }
    }

    /// Get alerts for a source
    ///
    /// # Arguments
    /// * `source_id` - Unique identifier for the source
    /// * `source_type` - Type of source
    /// * `limit` - Maximum number of alerts to return
    ///
    /// # Returns
    /// * `Result<Vec<SourceIntegrityAlert>>` - Source alerts
    pub async fn get_source_alerts(
        &self,
        source_id: &str,
        source_type: &SourceType,
        limit: Option<i32>,
    ) -> Result<Vec<SourceIntegrityAlert>> {
        if let Some(record) = self.get_source_record(source_id, source_type).await? {
            self.storage.get_alerts(&record.id, limit).await
        } else {
            Ok(Vec::new())
        }
    }

    /// Get source integrity statistics
    ///
    /// # Arguments
    /// * `time_range_start` - Start of time range for statistics
    /// * `time_range_end` - End of time range for statistics
    ///
    /// # Returns
    /// * `Result<SourceIntegrityStats>` - Integrity statistics
    pub async fn get_statistics(
        &self,
        time_range_start: Option<chrono::DateTime<Utc>>,
        time_range_end: Option<chrono::DateTime<Utc>>,
    ) -> Result<SourceIntegrityStats> {
        self.storage
            .get_statistics(time_range_start, time_range_end)
            .await
    }

    /// Delete a source integrity record
    ///
    /// # Arguments
    /// * `source_id` - Unique identifier for the source
    /// * `source_type` - Type of source
    ///
    /// # Returns
    /// * `Result<bool>` - True if record was deleted, false if not found
    pub async fn delete_source_record(
        &self,
        source_id: &str,
        source_type: &SourceType,
    ) -> Result<bool> {
        if let Some(record) = self.get_source_record(source_id, source_type).await? {
            self.storage.delete_record(&record.id).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Create a tampering alert
    async fn create_tampering_alert(
        &self,
        record_id: &Uuid,
        tampering_result: &TamperingDetectionResult,
    ) -> Result<Uuid> {
        let alert_type = if tampering_result
            .indicators
            .contains(&TamperingIndicator::HashMismatch)
        {
            AlertType::HashMismatch
        } else {
            AlertType::TamperingDetected
        };

        let severity = if tampering_result.confidence_score > 0.8 {
            AlertSeverity::Critical
        } else if tampering_result.confidence_score > 0.6 {
            AlertSeverity::High
        } else if tampering_result.confidence_score > 0.4 {
            AlertSeverity::Medium
        } else {
            AlertSeverity::Low
        };

        let alert_message = format!(
            "Tampering detected with confidence {:.2}: {}",
            tampering_result.confidence_score,
            tampering_result
                .indicators
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );

        let alert_data = HashMap::from([
            (
                "confidence_score".to_string(),
                serde_json::Value::Number(
                    serde_json::Number::from_f64(tampering_result.confidence_score).unwrap(),
                ),
            ),
            (
                "detection_time_ms".to_string(),
                serde_json::Value::Number(serde_json::Number::from(
                    tampering_result.detection_time_ms as u64,
                )),
            ),
            (
                "analysis_details".to_string(),
                serde_json::to_value(&tampering_result.analysis_details)?,
            ),
        ]);

        let alert = CreateSourceIntegrityAlert {
            source_integrity_id: *record_id,
            alert_type,
            severity,
            alert_message,
            alert_data,
        };

        let alert_id = self.storage.store_alert(&alert).await?;

        warn!(
            "Tampering alert created: {} for record {}",
            alert_id, record_id
        );

        Ok(alert_id)
    }

    /// Get the service configuration
    pub fn config(&self) -> &SourceIntegrityConfig {
        &self.config
    }

    /// Update the service configuration
    pub fn update_config(&mut self, config: SourceIntegrityConfig) {
        self.config = config;
        self.hasher = ContentHasher::new(self.config.default_hash_algorithm.clone());
    }
}

/// Trait for services that need source integrity verification
#[async_trait]
pub trait SourceIntegrityVerifiable {
    /// Verify the integrity of this source
    async fn verify_integrity(
        &self,
        service: &SourceIntegrityService,
    ) -> Result<IntegrityVerificationResult>;

    /// Get the source identifier
    fn source_id(&self) -> &str;

    /// Get the source type
    fn source_type(&self) -> SourceType;

    /// Get the source content
    fn source_content(&self) -> &str;

    /// Get additional metadata
    fn metadata(&self) -> HashMap<String, serde_json::Value>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // Mock storage for testing
    struct MockStorage;

    #[async_trait]
    impl SourceIntegrityStorage for MockStorage {
        async fn store_record(&self, _record: &CreateSourceIntegrityRecord) -> Result<Uuid> {
            Ok(Uuid::new_v4())
        }

        async fn get_record(&self, _id: &Uuid) -> Result<Option<SourceIntegrityRecord>> {
            Ok(None)
        }

        async fn get_record_by_source(
            &self,
            _source_id: &str,
            _source_type: &SourceType,
        ) -> Result<Option<SourceIntegrityRecord>> {
            Ok(None)
        }

        async fn update_record(&self, _record: &SourceIntegrityRecord) -> Result<()> {
            Ok(())
        }

        async fn store_verification(
            &self,
            _verification: &CreateSourceIntegrityVerification,
        ) -> Result<Uuid> {
            Ok(Uuid::new_v4())
        }

        async fn store_alert(&self, _alert: &CreateSourceIntegrityAlert) -> Result<Uuid> {
            Ok(Uuid::new_v4())
        }

        async fn get_verification_history(
            &self,
            _source_integrity_id: &Uuid,
            _limit: Option<i32>,
        ) -> Result<Vec<SourceIntegrityVerification>> {
            Ok(Vec::new())
        }

        async fn get_alerts(
            &self,
            _source_integrity_id: &Uuid,
            _limit: Option<i32>,
        ) -> Result<Vec<SourceIntegrityAlert>> {
            Ok(Vec::new())
        }

        async fn get_statistics(
            &self,
            _time_range_start: Option<DateTime<Utc>>,
            _time_range_end: Option<DateTime<Utc>>,
        ) -> Result<SourceIntegrityStats> {
            Ok(SourceIntegrityStats {
                total_sources: 0,
                verified_sources: 0,
                tampered_sources: 0,
                unknown_sources: 0,
                pending_sources: 0,
                total_verifications: 0,
                avg_verification_count: 0.0,
                last_verification: None,
                verification_success_rate: 0.0,
                avg_verification_duration_ms: 0.0,
            })
        }

        async fn delete_record(&self, _id: &Uuid) -> Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_verify_source_integrity_new_source() {
        let storage = Box::new(MockStorage);
        let config = SourceIntegrityConfig::default();
        let service = SourceIntegrityService::new(storage, config);

        let result = service
            .verify_source_integrity(
                "test_source",
                &SourceType::Content,
                "test content",
                &VerificationType::Initial,
                &HashMap::new(),
            )
            .await
            .unwrap();

        assert!(result.verified);
        assert!(!result.tampering_detected);
        assert_eq!(result.integrity_status, IntegrityStatus::Verified);
    }

    #[tokio::test]
    async fn test_verify_source_integrity_with_tampering() {
        let storage = Box::new(MockStorage);
        let config = SourceIntegrityConfig::default();
        let service = SourceIntegrityService::new(storage, config);

        let result = service
            .verify_source_integrity(
                "test_source",
                &SourceType::Content,
                "test content <!-- TAMPERED -->",
                &VerificationType::OnAccess,
                &HashMap::new(),
            )
            .await
            .unwrap();

        // Should detect tampering due to suspicious pattern
        assert!(!result.verified);
        assert!(result.tampering_detected);
        assert_eq!(result.integrity_status, IntegrityStatus::Tampered);
    }
}
