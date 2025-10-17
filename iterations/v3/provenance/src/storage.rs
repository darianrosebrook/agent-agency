//! Storage implementation for provenance records
//!
//! Provides database storage for provenance records using the existing database infrastructure

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::types::*;

/// Database-backed provenance storage
pub struct DatabaseProvenanceStorage {
    // Database connection would be injected here
    // For now, this is a placeholder implementation
}

impl DatabaseProvenanceStorage {
    /// Create a new database provenance storage
    pub fn new() -> Self {
        Self {
            // Initialize database connection
        }
    }
}

#[async_trait]
impl super::service::ProvenanceStorage for DatabaseProvenanceStorage {
    async fn store_record(&self, record: &ProvenanceRecord) -> Result<()> {
        // TODO: Implement database storage
        // This would use the existing database infrastructure from agent-agency-database
        tracing::info!("Storing provenance record: {}", record.id);
        Ok(())
    }

    async fn update_record(&self, record: &ProvenanceRecord) -> Result<()> {
        // TODO: Implement database update
        tracing::info!("Updating provenance record: {}", record.id);
        Ok(())
    }

    async fn get_record(&self, id: Uuid) -> Result<Option<ProvenanceRecord>> {
        // TODO: Implement database retrieval
        tracing::info!("Getting provenance record: {}", id);
        Ok(None)
    }

    async fn query_records(&self, query: &ProvenanceQuery) -> Result<Vec<ProvenanceRecord>> {
        // TODO: Implement database query
        tracing::info!("Querying provenance records");
        Ok(vec![])
    }

    async fn get_statistics(&self, time_range: Option<TimeRange>) -> Result<ProvenanceStats> {
        // TODO: Implement statistics calculation from database
        Ok(ProvenanceStats {
            total_records: 0,
            total_verdicts: 0,
            acceptance_rate: 0.0,
            average_consensus_score: 0.0,
            average_compliance_score: 0.0,
            average_verification_quality: 0.0,
            most_active_judge: "Unknown".to_string(),
            common_violations: vec![],
            time_range: time_range.unwrap_or_else(|| TimeRange {
                start: Utc::now(),
                end: Utc::now(),
            }),
        })
    }

    async fn delete_record(&self, id: Uuid) -> Result<()> {
        // TODO: Implement database deletion
        tracing::info!("Deleting provenance record: {}", id);
        Ok(())
    }
}

/// In-memory provenance storage for testing
pub struct InMemoryProvenanceStorage {
    records: HashMap<Uuid, ProvenanceRecord>,
}

impl InMemoryProvenanceStorage {
    /// Create a new in-memory storage
    pub fn new() -> Self {
        Self {
            records: HashMap::new(),
        }
    }
}

#[async_trait]
impl super::service::ProvenanceStorage for InMemoryProvenanceStorage {
    async fn store_record(&self, record: &ProvenanceRecord) -> Result<()> {
        // Note: This is a simplified implementation for testing
        // In a real implementation, you'd need to handle concurrent access
        tracing::info!("Storing provenance record in memory: {}", record.id);
        Ok(())
    }

    async fn update_record(&self, record: &ProvenanceRecord) -> Result<()> {
        tracing::info!("Updating provenance record in memory: {}", record.id);
        Ok(())
    }

    async fn get_record(&self, id: Uuid) -> Result<Option<ProvenanceRecord>> {
        tracing::info!("Getting provenance record from memory: {}", id);
        Ok(self.records.get(&id).cloned())
    }

    async fn query_records(&self, query: &ProvenanceQuery) -> Result<Vec<ProvenanceRecord>> {
        tracing::info!("Querying provenance records from memory");
        
        let mut results = Vec::new();
        
        for record in self.records.values() {
            let mut matches = true;
            
            if let Some(task_id) = query.task_id {
                if record.task_id != task_id {
                    matches = false;
                }
            }
            
            if let Some(verdict_id) = query.verdict_id {
                if record.verdict_id != verdict_id {
                    matches = false;
                }
            }
            
            if let Some(ref time_range) = query.time_range {
                if record.timestamp < time_range.start || record.timestamp > time_range.end {
                    matches = false;
                }
            }
            
            if matches {
                results.push(record.clone());
            }
        }
        
        // Apply limit and offset
        let offset = query.offset.unwrap_or(0) as usize;
        let limit = query.limit.unwrap_or(1000) as usize;
        
        results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        let start = offset;
        let end = std::cmp::min(start + limit, results.len());
        
        Ok(results[start..end].to_vec())
    }

    async fn get_statistics(&self, time_range: Option<TimeRange>) -> Result<ProvenanceStats> {
        let records: Vec<&ProvenanceRecord> = if let Some(ref range) = time_range {
            self.records.values()
                .filter(|record| record.timestamp >= range.start && record.timestamp <= range.end)
                .collect()
        } else {
            self.records.values().collect()
        };
        
        if records.is_empty() {
            return Ok(ProvenanceStats {
                total_records: 0,
                total_verdicts: 0,
                acceptance_rate: 0.0,
                average_consensus_score: 0.0,
                average_compliance_score: 0.0,
                average_verification_quality: 0.0,
                most_active_judge: "Unknown".to_string(),
                common_violations: vec![],
                time_range: time_range.unwrap_or_else(|| TimeRange {
                    start: Utc::now(),
                    end: Utc::now(),
                }),
            });
        }
        
        let total_records = records.len() as u64;
        let total_verdicts = records.len() as u64;
        
        let accepted_count = records.iter()
            .filter(|r| r.is_accepted())
            .count();
        let acceptance_rate = accepted_count as f32 / total_records as f32;
        
        let average_consensus_score = records.iter()
            .map(|r| r.consensus_score)
            .sum::<f32>() / total_records as f32;
        
        let average_compliance_score = records.iter()
            .map(|r| r.caws_compliance.compliance_score)
            .sum::<f32>() / total_records as f32;
        
        let average_verification_quality = records.iter()
            .filter_map(|r| r.claim_verification.as_ref())
            .map(|v| v.verification_quality)
            .sum::<f32>() / records.iter().filter(|r| r.claim_verification.is_some()).count() as f32;
        
        // Find most active judge
        let mut judge_counts: HashMap<String, u32> = HashMap::new();
        for record in &records {
            for judge_id in record.judge_verdicts.keys() {
                *judge_counts.entry(judge_id.clone()).or_insert(0) += 1;
            }
        }
        
        let most_active_judge = judge_counts.iter()
            .max_by_key(|(_, count)| *count)
            .map(|(judge_id, _)| judge_id.clone())
            .unwrap_or_else(|| "Unknown".to_string());
        
        // Calculate common violations
        let mut violation_counts: HashMap<String, u32> = HashMap::new();
        for record in &records {
            for violation in &record.caws_compliance.violations {
                *violation_counts.entry(violation.rule.clone()).or_insert(0) += 1;
            }
        }
        
        let mut common_violations = violation_counts.iter()
            .map(|(rule, count)| ViolationStats {
                rule: rule.clone(),
                count: *count as u64,
                severity_distribution: HashMap::new(), // Simplified for this implementation
                average_resolution_time_ms: 0.0,
            })
            .collect::<Vec<_>>();
        
        common_violations.sort_by(|a, b| b.count.cmp(&a.count));
        common_violations.truncate(10); // Top 10 violations
        
        Ok(ProvenanceStats {
            total_records,
            total_verdicts,
            acceptance_rate,
            average_consensus_score,
            average_compliance_score,
            average_verification_quality,
            most_active_judge,
            common_violations,
            time_range: time_range.unwrap_or_else(|| TimeRange {
                start: Utc::now(),
                end: Utc::now(),
            }),
        })
    }

    async fn delete_record(&self, id: Uuid) -> Result<()> {
        tracing::info!("Deleting provenance record from memory: {}", id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_in_memory_storage() {
        let storage = InMemoryProvenanceStorage::new();
        
        let record = create_test_provenance_record();
        storage.store_record(&record).await.unwrap();
        
        let retrieved = storage.get_record(record.id).await.unwrap();
        assert!(retrieved.is_some());
        
        let query = ProvenanceQuery {
            task_id: Some(record.task_id),
            verdict_id: None,
            decision_type: None,
            time_range: None,
            judge_id: None,
            compliance_status: None,
            limit: None,
            offset: None,
        };
        
        let results = storage.query_records(&query).await.unwrap();
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_in_memory_statistics() {
        let storage = InMemoryProvenanceStorage::new();
        
        let stats = storage.get_statistics(None).await.unwrap();
        assert_eq!(stats.total_records, 0);
        assert_eq!(stats.acceptance_rate, 0.0);
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
}

