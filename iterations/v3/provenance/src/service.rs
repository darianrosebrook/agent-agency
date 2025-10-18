//! Provenance service implementation
//!
//! Main service for managing provenance records with git integration and signing

use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    git_integration::GitIntegration,
    signer::{SignerFactory, SignerTrait},
    types::{
        BudgetAdherence, CawsComplianceProvenance, ExportFormat, ExportMetadata, FilterOperator,
        FilterType, IntegrityCheckResult, IntegrityIssue, IntegrityIssueType, IntegritySeverity,
        ProvenanceChain, ProvenanceExport, ProvenanceFilter, ProvenanceQuery, ProvenanceRecord,
        ProvenanceStats, TimeRange, VerdictDecision,
    },
    ProvenanceConfig,
};

/// Storage trait for provenance records
#[async_trait]
pub trait ProvenanceStorage: Send + Sync {
    async fn store_record(&self, record: &ProvenanceRecord) -> Result<()>;
    async fn get_record(&self, id: &str) -> Result<Option<ProvenanceRecord>>;
    async fn query_records(&self, query: &ProvenanceQuery) -> Result<Vec<ProvenanceRecord>>;
    async fn update_record(&self, record: &ProvenanceRecord) -> Result<()>;
    async fn delete_record(&self, id: &str) -> Result<()>;
    async fn get_statistics(&self, time_range: Option<TimeRange>) -> Result<ProvenanceStats>;
}

/// Main provenance service
pub struct ProvenanceService {
    signer: Box<dyn SignerTrait>,
    git_integration: Option<Box<dyn GitIntegration>>,
    storage: Box<dyn ProvenanceStorage>,
    config: ProvenanceConfig,
}

impl ProvenanceService {
    /// Create a new provenance service
    pub fn new(
        signer: Box<dyn SignerTrait>,
        git_integration: Option<Box<dyn GitIntegration>>,
        storage: Box<dyn ProvenanceStorage>,
        config: ProvenanceConfig,
    ) -> Self {
        Self {
            signer,
            git_integration,
            storage,
            config,
        }
    }

    /// Create a new provenance service with default configuration
    pub fn with_defaults(
        storage: Box<dyn ProvenanceStorage>,
        config: ProvenanceConfig,
    ) -> Result<Self> {
        let signer = SignerFactory::create_signer(
            &config.signing.key_path,
            config.signing.key_id.clone(),
            match config.signing.algorithm {
                crate::SigningAlgorithm::RS256 => crate::signer::SigningAlgorithm::RS256,
                crate::SigningAlgorithm::ES256 => crate::signer::SigningAlgorithm::ES256,
                crate::SigningAlgorithm::EdDSA => crate::signer::SigningAlgorithm::EdDSA,
            },
        )?;

        let git_integration = if std::path::Path::new(&config.git.repository_path)
            .join(".git")
            .exists()
        {
            // TODO[provenance-git-bridge]: Re-enable GitIntegration once implementation satisfies:
            // 1. Git integration implementation: Implement proper GitIntegration trait
            //    - Complete GitIntegration trait implementation with all required methods
            //    - Handle Git operations with proper error handling and validation
            //    - Implement thread-safe Git operations and async support
            // 2. Git trailer management: Implement Git trailer management functionality
            //    - Handle Git trailer addition, modification, and removal
            //    - Implement proper Git trailer validation and formatting
            //    - Handle Git trailer synchronization and consistency
            // 3. Error handling: Implement robust error handling for Git operations
            //    - Handle Git-specific errors and exceptions
            //    - Provide meaningful error messages and recovery options
            //    - Implement proper error propagation and handling
            // 4. Performance optimization: Optimize Git operations for performance
            //    - Implement efficient Git operation caching and batching
            //    - Minimize Git repository access and operations
            //    - Handle large repositories and operations efficiently
            // Acceptance Criteria:
            // - Integration tests can run against a temp git repo, capture provenance commits with
            //   CAWS trailers, and verify trailer presence plus JWS hashes.
            // - Concurrent provenance writes are serialized without deadlocks or data loss.
            // - Misconfigured repos return typed errors before any commit attempt, preventing
            //   silent provenance gaps.
            // Some(Box::new(GitTrailerManager::new(
            //     &config.git.repository_path,
            //     config.git.branch.clone(),
            //     config.git.auto_commit,
            //     config.git.commit_message_template.clone(),
            // )?) as Box<dyn GitIntegration>)
            None
        } else {
            None
        };

        Ok(Self::new(signer, git_integration, storage, config))
    }

    /// Record a provenance entry with full integration
    pub async fn record_provenance(&self, record: ProvenanceRecord) -> Result<ProvenanceRecord> {
        // Sign the record
        let signature = self.signer.sign(&record).await?;
        let mut signed_record = record;
        signed_record.signature = signature;

        // Store in database
        self.storage.store_record(&signed_record).await?;

        // Integrate with git if available
        if let Some(ref git) = self.git_integration {
            if self.config.git.auto_commit {
                let commit_message = self.generate_commit_message(&signed_record);
                let commit_hash = git
                    .create_provenance_commit(&commit_message, &signed_record)
                    .await?;

                signed_record.git_commit_hash = Some(commit_hash);

                // Update the record with git commit hash
                self.storage.update_record(&signed_record).await?;
            }
        }

        Ok(signed_record)
    }

    /// Generate commit message for provenance record
    fn generate_commit_message(&self, record: &ProvenanceRecord) -> String {
        let decision_summary = record.decision_summary();
        let consensus_score = record.consensus_score;
        let compliance_score = record.caws_compliance.compliance_score;

        format!(
            "CAWS Verdict {}: {} (Consensus: {:.2}, Compliance: {:.2})",
            record.verdict_id, decision_summary, consensus_score, compliance_score
        )
    }

    /// Verify provenance record integrity
    pub async fn verify_integrity(
        &self,
        record: &ProvenanceRecord,
    ) -> Result<IntegrityCheckResult> {
        let mut issues = Vec::new();
        let mut is_valid = true;

        // Verify signature
        if !self.signer.verify(record, &record.signature).await? {
            issues.push(IntegrityIssue {
                record_id: record.id,
                issue_type: IntegrityIssueType::SignatureInvalid,
                description: "Digital signature verification failed".to_string(),
                severity: IntegritySeverity::Critical,
            });
            is_valid = false;
        }

        // Verify git integration if present
        if let Some(ref git) = self.git_integration {
            if let Some(commit_hash) = &record.git_commit_hash {
                if !git.verify_trailer(commit_hash, &record.git_trailer).await? {
                    issues.push(IntegrityIssue {
                        record_id: record.id,
                        issue_type: IntegrityIssueType::GitTrailerCorrupted,
                        description: "Git trailer verification failed".to_string(),
                        severity: IntegritySeverity::Major,
                    });
                    is_valid = false;
                }
            } else {
                issues.push(IntegrityIssue {
                    record_id: record.id,
                    issue_type: IntegrityIssueType::GitCommitMissing,
                    description: "Git commit hash is missing".to_string(),
                    severity: IntegritySeverity::Minor,
                });
            }
        }

        // Verify timestamp consistency
        let now = Utc::now();
        let time_diff = (now - record.timestamp).num_seconds();
        if time_diff.abs() > 3600 {
            // More than 1 hour difference
            issues.push(IntegrityIssue {
                record_id: record.id,
                issue_type: IntegrityIssueType::TimestampInconsistent,
                description: format!("Timestamp is {} seconds from now", time_diff),
                severity: IntegritySeverity::Warning,
            });
        }

        Ok(IntegrityCheckResult {
            is_valid,
            issues,
            checked_records: 1,
            checked_at: now,
        })
    }

    /// Get provenance statistics
    pub async fn get_statistics(&self, time_range: Option<TimeRange>) -> Result<ProvenanceStats> {
        self.storage.get_statistics(time_range).await
    }

    /// Export provenance data
    pub async fn export_data(
        &self,
        query: ProvenanceQuery,
        format: ExportFormat,
    ) -> Result<ProvenanceExport> {
        let records = self.storage.query_records(&query).await?;

        let export_id = Uuid::new_v4();
        let metadata = ExportMetadata {
            total_records: records.len() as u64,
            time_range: query.time_range.unwrap_or_else(|| TimeRange {
                start: chrono::DateTime::from_timestamp(0, 0).unwrap_or_else(Utc::now),
                end: Utc::now(),
            }),
            filters_applied: self.extract_filters_from_query(&query)?,
            export_reason: "Data export requested".to_string(),
            recipient: None,
        };

        Ok(ProvenanceExport {
            export_id,
            format,
            records,
            metadata,
            created_at: Utc::now(),
            created_by: "provenance-service".to_string(),
        })
    }

    /// Extract filters from provenance query parameters
    fn extract_filters_from_query(&self, query: &ProvenanceQuery) -> Result<Vec<ProvenanceFilter>> {
        let mut filters = Vec::new();

        // 1. Query parsing: Parse provenance query to extract applied filters
        self.extract_time_range_filters(query, &mut filters)?;
        self.extract_entity_filters(query, &mut filters)?;
        self.extract_activity_filters(query, &mut filters)?;
        self.extract_agent_filters(query, &mut filters)?;
        self.extract_custom_filters(query, &mut filters)?;

        // 2. Filter validation: Validate extracted filters for correctness
        self.validate_filters(&filters)?;

        // 3. Filter processing: Process filters for provenance data export
        self.optimize_filters(&mut filters)?;

        Ok(filters)
    }

    /// Extract time range filters from query
    fn extract_time_range_filters(
        &self,
        query: &ProvenanceQuery,
        filters: &mut Vec<ProvenanceFilter>,
    ) -> Result<()> {
        if let Some(time_range) = &query.time_range {
            filters.push(ProvenanceFilter {
                filter_type: FilterType::TimeRange,
                field: "timestamp".to_string(),
                operator: FilterOperator::Between,
                value: serde_json::json!({
                    "start": time_range.start,
                    "end": time_range.end
                }),
                description: format!(
                    "Time range: {} to {}",
                    time_range.start.format("%Y-%m-%d %H:%M:%S"),
                    time_range.end.format("%Y-%m-%d %H:%M:%S")
                ),
            });
        }
        Ok(())
    }

    /// Extract entity filters from query
    fn extract_entity_filters(
        &self,
        query: &ProvenanceQuery,
        filters: &mut Vec<ProvenanceFilter>,
    ) -> Result<()> {
        if let Some(entity_types) = &query.entity_types {
            if !entity_types.is_empty() {
                filters.push(ProvenanceFilter {
                    filter_type: FilterType::EntityType,
                    field: "entity_type".to_string(),
                    operator: FilterOperator::In,
                    value: serde_json::to_value(entity_types)?,
                    description: format!("Entity types: {}", entity_types.join(", ")),
                });
            }
        }

        if let Some(entity_ids) = &query.entity_ids {
            if !entity_ids.is_empty() {
                filters.push(ProvenanceFilter {
                    filter_type: FilterType::EntityId,
                    field: "entity_id".to_string(),
                    operator: FilterOperator::In,
                    value: serde_json::to_value(entity_ids)?,
                    description: format!("Entity IDs: {}", entity_ids.len()),
                });
            }
        }

        Ok(())
    }

    /// Extract activity filters from query
    fn extract_activity_filters(
        &self,
        query: &ProvenanceQuery,
        filters: &mut Vec<ProvenanceFilter>,
    ) -> Result<()> {
        if let Some(activity_types) = &query.activity_types {
            if !activity_types.is_empty() {
                filters.push(ProvenanceFilter {
                    filter_type: FilterType::ActivityType,
                    field: "activity_type".to_string(),
                    operator: FilterOperator::In,
                    value: serde_json::to_value(activity_types)?,
                    description: format!("Activity types: {}", activity_types.join(", ")),
                });
            }
        }

        if let Some(activity_ids) = &query.activity_ids {
            if !activity_ids.is_empty() {
                filters.push(ProvenanceFilter {
                    filter_type: FilterType::ActivityId,
                    field: "activity_id".to_string(),
                    operator: FilterOperator::In,
                    value: serde_json::to_value(activity_ids)?,
                    description: format!("Activity IDs: {}", activity_ids.len()),
                });
            }
        }

        Ok(())
    }

    /// Extract agent filters from query
    fn extract_agent_filters(
        &self,
        query: &ProvenanceQuery,
        filters: &mut Vec<ProvenanceFilter>,
    ) -> Result<()> {
        if let Some(agent_ids) = &query.agent_ids {
            if !agent_ids.is_empty() {
                filters.push(ProvenanceFilter {
                    filter_type: FilterType::AgentId,
                    field: "agent_id".to_string(),
                    operator: FilterOperator::In,
                    value: serde_json::to_value(agent_ids)?,
                    description: format!("Agent IDs: {}", agent_ids.len()),
                });
            }
        }

        if let Some(agent_types) = &query.agent_types {
            if !agent_types.is_empty() {
                filters.push(ProvenanceFilter {
                    filter_type: FilterType::AgentType,
                    field: "agent_type".to_string(),
                    operator: FilterOperator::In,
                    value: serde_json::to_value(agent_types)?,
                    description: format!("Agent types: {}", agent_types.join(", ")),
                });
            }
        }

        Ok(())
    }

    /// Extract custom filters from query
    fn extract_custom_filters(
        &self,
        query: &ProvenanceQuery,
        filters: &mut Vec<ProvenanceFilter>,
    ) -> Result<()> {
        if let Some(custom_filters) = &query.custom_filters {
            for (field, filter_value) in custom_filters {
                let filter = ProvenanceFilter {
                    filter_type: FilterType::Custom,
                    field: field.clone(),
                    operator: self.determine_operator(filter_value)?,
                    value: filter_value.clone(),
                    description: format!(
                        "Custom filter: {} {}",
                        field,
                        self.describe_filter_value(filter_value)?
                    ),
                };
                filters.push(filter);
            }
        }
        Ok(())
    }

    /// Determine filter operator from value type
    fn determine_operator(&self, value: &serde_json::Value) -> Result<FilterOperator> {
        match value {
            serde_json::Value::Array(_) => Ok(FilterOperator::In),
            serde_json::Value::Object(obj) => {
                if obj.contains_key("start") && obj.contains_key("end") {
                    Ok(FilterOperator::Between)
                } else if obj.contains_key("min") && obj.contains_key("max") {
                    Ok(FilterOperator::Between)
                } else {
                    Ok(FilterOperator::Equals)
                }
            }
            _ => Ok(FilterOperator::Equals),
        }
    }

    /// Describe filter value for documentation
    fn describe_filter_value(&self, value: &serde_json::Value) -> Result<String> {
        match value {
            serde_json::Value::String(s) => Ok(format!("= '{}'", s)),
            serde_json::Value::Number(n) => Ok(format!("= {}", n)),
            serde_json::Value::Bool(b) => Ok(format!("= {}", b)),
            serde_json::Value::Array(arr) => Ok(format!("in [{} values]", arr.len())),
            serde_json::Value::Object(obj) => {
                if obj.contains_key("start") && obj.contains_key("end") {
                    Ok("between range".to_string())
                } else if obj.contains_key("min") && obj.contains_key("max") {
                    Ok("between range".to_string())
                } else {
                    Ok("matches object".to_string())
                }
            }
            serde_json::Value::Null => Ok("is null".to_string()),
        }
    }

    /// Validate extracted filters for correctness
    fn validate_filters(&self, filters: &[ProvenanceFilter]) -> Result<()> {
        for filter in filters {
            // Verify filter syntax and parameter validity
            if filter.field.is_empty() {
                return Err(anyhow::anyhow!("Filter field cannot be empty"));
            }

            // Check filter compatibility and consistency
            match filter.filter_type {
                FilterType::TimeRange => {
                    if filter.field != "timestamp" {
                        return Err(anyhow::anyhow!(
                            "Time range filter must use 'timestamp' field"
                        ));
                    }
                }
                FilterType::EntityType => {
                    if filter.field != "entity_type" {
                        return Err(anyhow::anyhow!(
                            "Entity type filter must use 'entity_type' field"
                        ));
                    }
                }
                FilterType::ActivityType => {
                    if filter.field != "activity_type" {
                        return Err(anyhow::anyhow!(
                            "Activity type filter must use 'activity_type' field"
                        ));
                    }
                }
                FilterType::AgentType => {
                    if filter.field != "agent_type" {
                        return Err(anyhow::anyhow!(
                            "Agent type filter must use 'agent_type' field"
                        ));
                    }
                }
                _ => {} // Custom filters can use any field
            }

            // Validate operator compatibility
            match filter.operator {
                FilterOperator::In => {
                    if !filter.value.is_array() {
                        return Err(anyhow::anyhow!("IN operator requires array value"));
                    }
                }
                FilterOperator::Between => {
                    if !filter.value.is_object() {
                        return Err(anyhow::anyhow!("BETWEEN operator requires object value"));
                    }
                }
                _ => {} // Other operators are compatible with any value type
            }
        }
        Ok(())
    }

    /// Optimize filters for performance
    fn optimize_filters(&self, filters: &mut Vec<ProvenanceFilter>) -> Result<()> {
        // Sort filters by selectivity (most selective first)
        filters.sort_by(|a, b| {
            let a_selectivity = self.estimate_filter_selectivity(a);
            let b_selectivity = self.estimate_filter_selectivity(b);
            b_selectivity
                .partial_cmp(&a_selectivity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Remove redundant filters
        self.remove_redundant_filters(filters)?;

        Ok(())
    }

    /// Estimate filter selectivity (lower is more selective)
    fn estimate_filter_selectivity(&self, filter: &ProvenanceFilter) -> f64 {
        match filter.filter_type {
            FilterType::TimeRange => 0.1, // Time ranges are usually very selective
            FilterType::EntityId | FilterType::ActivityId | FilterType::AgentId => 0.05, // IDs are very selective
            FilterType::EntityType | FilterType::ActivityType | FilterType::AgentType => 0.3, // Types are moderately selective
            FilterType::Custom => 0.5, // Custom filters are less predictable
        }
    }

    /// Remove redundant filters
    fn remove_redundant_filters(&self, filters: &mut Vec<ProvenanceFilter>) -> Result<()> {
        let mut i = 0;
        while i < filters.len() {
            let mut j = i + 1;
            while j < filters.len() {
                if self.filters_are_redundant(&filters[i], &filters[j]) {
                    filters.remove(j);
                } else {
                    j += 1;
                }
            }
            i += 1;
        }
        Ok(())
    }

    /// Check if two filters are redundant
    fn filters_are_redundant(
        &self,
        filter1: &ProvenanceFilter,
        filter2: &ProvenanceFilter,
    ) -> bool {
        // Same field and operator
        if filter1.field == filter2.field && filter1.operator == filter2.operator {
            // Check if values are equivalent
            match (&filter1.value, &filter2.value) {
                (serde_json::Value::Array(arr1), serde_json::Value::Array(arr2)) => {
                    // If one array contains the other, they're redundant
                    arr1.len() == arr2.len() && arr1.iter().all(|v| arr2.contains(v))
                }
                (val1, val2) => val1 == val2,
            }
        } else {
            false
        }
    }

    /// Perform full integrity check on all records
    pub async fn perform_integrity_check(&self) -> Result<IntegrityCheckResult> {
        let mut all_issues = Vec::new();
        let mut checked_records = 0;
        let mut all_valid = true;

        // Get all records (in batches to avoid memory issues)
        let batch_size = 1000;
        let mut offset = 0;

        loop {
            let query = ProvenanceQuery {
                task_id: None,
                verdict_id: None,
                decision_type: None,
                time_range: None,
                judge_id: None,
                compliance_status: None,
                limit: Some(batch_size),
                offset: Some(offset),
            };

            let records = self.storage.query_records(&query).await?;
            if records.is_empty() {
                break;
            }

            for record in records {
                let integrity_result = self.verify_integrity(&record).await?;
                all_issues.extend(integrity_result.issues);
                checked_records += integrity_result.checked_records;

                if !integrity_result.is_valid {
                    all_valid = false;
                }
            }

            offset += batch_size;
        }

        Ok(IntegrityCheckResult {
            is_valid: all_valid,
            issues: all_issues,
            checked_records,
            checked_at: Utc::now(),
        })
    }

    /// Get provenance chain for a task
    pub async fn get_provenance_chain(&self, task_id: Uuid) -> Result<ProvenanceChain> {
        let query = ProvenanceQuery {
            task_id: Some(task_id),
            verdict_id: None,
            decision_type: None,
            time_range: None,
            judge_id: None,
            compliance_status: None,
            limit: None,
            offset: None,
        };

        let records = self.storage.query_records(&query).await?;

        // Sort by timestamp
        let mut sorted_records = records;
        sorted_records.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        // Verify chain integrity
        let mut integrity_verified = true;
        for record in &sorted_records {
            let integrity_result = self.verify_integrity(record).await?;
            if !integrity_result.is_valid {
                integrity_verified = false;
                break;
            }
        }

        // Capture values before moving sorted_records
        let chain_length = sorted_records.len() as u32;
        let created_at = sorted_records
            .first()
            .map(|r| r.timestamp)
            .unwrap_or_else(Utc::now);
        let last_updated = sorted_records
            .last()
            .map(|r| r.timestamp)
            .unwrap_or_else(Utc::now);

        Ok(ProvenanceChain {
            chain_id: Uuid::new_v4(),
            entries: sorted_records,
            integrity_verified,
            chain_length,
            created_at,
            last_updated,
        })
    }

    /// Lightweight generic event append for telemetry (e.g., ARM planning).
    /// NOTE: For Tier 1 scenarios, promote these to signed records.
    pub async fn append_event(&self, event_type: &str, payload: serde_json::Value) -> Result<()> {
        // Build a minimal ProvenanceRecord-like entry for storage
        let rec = ProvenanceRecord {
            id: Uuid::new_v4(),
            verdict_id: Uuid::new_v4(),
            task_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            caws_compliance: CawsComplianceProvenance {
                is_compliant: true,
                compliance_score: 0.0,
                violations: vec![],
                waivers_used: vec![],
                budget_adherence: BudgetAdherence {
                    max_files: 0,
                    actual_files: 0,
                    max_loc: 0,
                    actual_loc: 0,
                    max_time_minutes: None,
                    actual_time_minutes: None,
                    within_budget: true,
                },
            },
            consensus_score: 0.0,
            decision: VerdictDecision::Accept {
                confidence: 0.5,
                summary: "Telemetry event".to_string(),
            },
            judge_verdicts: HashMap::new(),
            claim_verification: None,
            git_trailer: "telemetry".to_string(),
            git_commit_hash: None,
            signature: "unsigned".to_string(),
            metadata: HashMap::from([
                ("event_type".into(), event_type.into()),
                ("payload".into(), payload),
            ]),
        };
        // Store without signing to keep it lightweight
        self.storage.store_record(&rec).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_provenance_service_creation() {
        let config = ProvenanceConfig::default();
        let storage = MockProvenanceStorage::new();

        let service = ProvenanceService::with_defaults(Box::new(storage), config).unwrap();

        // Service should be created successfully
        assert_eq!(service.config.signing.algorithm, SigningAlgorithm::EdDSA);
    }

    #[tokio::test]
    async fn test_provenance_record_integrity_verification() {
        let config = ProvenanceConfig::default();
        let storage = MockProvenanceStorage::new();

        let service = ProvenanceService::with_defaults(Box::new(storage), config).unwrap();

        let record = create_test_provenance_record();
        let signed_record = service.record_provenance(record).await.unwrap();

        let integrity_result = service.verify_integrity(&signed_record).await.unwrap();
        assert!(integrity_result.is_valid);
        assert_eq!(integrity_result.checked_records, 1);
    }

    fn create_test_provenance_record() -> ProvenanceRecord {
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

    // Mock storage implementation for testing
    struct MockProvenanceStorage {
        records: HashMap<Uuid, ProvenanceRecord>,
    }

    impl MockProvenanceStorage {
        fn new() -> Self {
            Self {
                records: HashMap::new(),
            }
        }
    }

    #[async_trait]
    impl ProvenanceStorage for MockProvenanceStorage {
        async fn store_record(&self, record: &ProvenanceRecord) -> Result<()> {
            // Mock implementation - in real implementation, this would store to database
            Ok(())
        }

        async fn update_record(&self, _record: &ProvenanceRecord) -> Result<()> {
            // Mock implementation
            Ok(())
        }

        async fn get_record(&self, _id: &str) -> Result<Option<ProvenanceRecord>> {
            // Mock implementation
            Ok(None)
        }

        async fn query_records(&self, _query: &ProvenanceQuery) -> Result<Vec<ProvenanceRecord>> {
            // Mock implementation
            Ok(vec![])
        }

        async fn get_statistics(&self, _time_range: Option<TimeRange>) -> Result<ProvenanceStats> {
            // Mock implementation
            Ok(ProvenanceStats {
                total_records: 0,
                total_verdicts: 0,
                acceptance_rate: 0.0,
                average_consensus_score: 0.0,
                average_compliance_score: 0.0,
                average_verification_quality: 0.0,
                most_active_judge: "Unknown".to_string(),
                common_violations: vec![],
                time_range: TimeRange {
                    start: Utc::now(),
                    end: Utc::now(),
                },
            })
        }

        async fn delete_record(&self, _id: &str) -> Result<()> {
            // Mock implementation
            Ok(())
        }
    }
}
