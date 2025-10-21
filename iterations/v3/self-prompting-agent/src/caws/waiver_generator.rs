//! Waiver Generator for CAWS Budget Overrides
//!
//! Automatically generates waivers when council approves budget overruns,
//! with proper validation, expiration, and audit trails.
//!
//! @author @darianrosebrook

use std::path::PathBuf;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};

use super::budget_checker::BudgetLimits;

/// Waiver generator for budget overrides
pub struct WaiverGenerator {
    waiver_store: WaiverStore,
    default_expiry_hours: i64,
}

/// In-memory waiver store (could be backed by database)
#[derive(Debug, Default)]
pub struct WaiverStore {
    waivers: HashMap<Uuid, Waiver>,
    task_waivers: HashMap<Uuid, Vec<Uuid>>, // task_id -> waiver_ids
}

/// Waiver granting increased budget limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Waiver {
    pub id: Uuid,
    pub task_id: Uuid,
    pub waiver_type: WaiverType,
    pub granted_by: String,
    pub original_limits: BudgetLimits,
    pub granted_limits: BudgetLimits,
    pub justification: String,
    pub conditions: Vec<String>,
    pub expires_at: DateTime<Utc>,
    pub issued_at: DateTime<Utc>,
    pub used_count: usize,
    pub max_uses: usize,
    pub metadata: WaiverMetadata,
}

/// Type of waiver granted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WaiverType {
    BudgetOverrun,
    EmergencyFix,
    ExperimentalFeature,
    ThirdPartyIntegration,
    PerformanceCritical,
}

/// Additional metadata for waiver tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaiverMetadata {
    pub risk_assessment: String,
    pub monitoring_plan: String,
    pub rollback_plan: String,
    pub contact_info: String,
    pub priority_level: PriorityLevel,
}

/// Priority level for waiver processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PriorityLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Waiver validation result
#[derive(Debug, Clone)]
pub enum WaiverValidation {
    Valid(Waiver),
    Expired(Waiver),
    ExceededUsage(Waiver),
    NotFound,
    Invalid(String),
}

/// Waiver generation errors
#[derive(Debug, thiserror::Error)]
pub enum WaiverError {
    #[error("Waiver expired: {waiver_id}")]
    Expired { waiver_id: Uuid },

    #[error("Waiver usage exceeded: {waiver_id}")]
    UsageExceeded { waiver_id: Uuid },

    #[error("Invalid waiver: {reason}")]
    Invalid { reason: String },

    #[error("Waiver not found: {waiver_id}")]
    NotFound { waiver_id: Uuid },

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

impl Default for WaiverGenerator {
    fn default() -> Self {
        Self {
            waiver_store: WaiverStore::default(),
            default_expiry_hours: 24, // 24 hours
        }
    }
}

impl WaiverGenerator {
    /// Create new waiver generator
    pub fn new() -> Self {
        Self::default()
    }

    /// Generate waiver from council approval
    pub fn generate_waiver(
        &mut self,
        task_id: Uuid,
        waiver_type: WaiverType,
        granted_by: String,
        original_limits: BudgetLimits,
        granted_limits: BudgetLimits,
        justification: String,
        conditions: Vec<String>,
        metadata: WaiverMetadata,
    ) -> Result<Uuid, WaiverError> {
        let waiver = Waiver {
            id: Uuid::new_v4(),
            task_id,
            waiver_type,
            granted_by,
            original_limits,
            granted_limits,
            justification,
            conditions,
            expires_at: Utc::now() + Duration::hours(self.default_expiry_hours),
            issued_at: Utc::now(),
            used_count: 0,
            max_uses: 10, // Default max uses
            metadata,
        };

        self.waiver_store.store_waiver(waiver)?;
        Ok(self.waiver_store.get_waiver_id(task_id).unwrap())
    }

    /// Generate budget overrun waiver (convenience method)
    pub fn generate_budget_overrun_waiver(
        &mut self,
        task_id: Uuid,
        granted_by: String,
        original_limits: BudgetLimits,
        granted_limits: BudgetLimits,
        justification: String,
        risk_assessment: String,
    ) -> Result<Uuid, WaiverError> {
        self.generate_waiver(
            task_id,
            WaiverType::BudgetOverrun,
            granted_by,
            original_limits,
            granted_limits,
            justification,
            vec![
                "Monitor file changes closely".to_string(),
                "Validate changes before final commit".to_string(),
                "Document any unexpected issues".to_string(),
            ],
            WaiverMetadata {
                risk_assessment,
                monitoring_plan: "Track all file modifications and LOC changes".to_string(),
                rollback_plan: "Use Git worktree rollback capabilities".to_string(),
                contact_info: "agent-agency-monitoring@example.com".to_string(),
                priority_level: PriorityLevel::Medium,
            },
        )
    }

    /// Validate waiver for use
    pub fn validate_waiver(&self, waiver_id: Uuid) -> WaiverValidation {
        match self.waiver_store.get_waiver(waiver_id) {
            Some(waiver) => {
                // Check expiry
                if Utc::now() > waiver.expires_at {
                    return WaiverValidation::Expired(waiver);
                }

                // Check usage
                if waiver.used_count >= waiver.max_uses {
                    return WaiverValidation::ExceededUsage(waiver);
                }

                WaiverValidation::Valid(waiver)
            }
            None => WaiverValidation::NotFound,
        }
    }

    /// Use waiver (increment usage counter)
    pub fn use_waiver(&mut self, waiver_id: Uuid) -> Result<(), WaiverError> {
        match self.validate_waiver(waiver_id) {
            WaiverValidation::Valid(mut waiver) => {
                waiver.used_count += 1;
                self.waiver_store.update_waiver(waiver)?;
                Ok(())
            }
            WaiverValidation::Expired(_) => Err(WaiverError::Expired { waiver_id }),
            WaiverValidation::ExceededUsage(_) => Err(WaiverError::UsageExceeded { waiver_id }),
            WaiverValidation::NotFound => Err(WaiverError::NotFound { waiver_id }),
            WaiverValidation::Invalid(reason) => Err(WaiverError::Invalid { reason }),
        }
    }

    /// Get active waivers for task
    pub fn get_active_waivers(&self, task_id: Uuid) -> Vec<Waiver> {
        self.waiver_store.get_task_waivers(task_id)
            .into_iter()
            .filter_map(|waiver_id| {
                match self.validate_waiver(waiver_id) {
                    WaiverValidation::Valid(waiver) => Some(waiver),
                    _ => None,
                }
            })
            .collect()
    }

    /// Get waiver by ID
    pub fn get_waiver(&self, waiver_id: Uuid) -> Option<Waiver> {
        self.waiver_store.get_waiver(waiver_id)
    }

    /// Clean up expired waivers
    pub fn cleanup_expired(&mut self) -> Result<usize, WaiverError> {
        let expired_ids: Vec<Uuid> = self.waiver_store.waivers.keys()
            .filter(|&&id| {
                if let Some(waiver) = self.waiver_store.get_waiver(id) {
                    Utc::now() > waiver.expires_at
                } else {
                    false
                }
            })
            .cloned()
            .collect();

        let count = expired_ids.len();

        for id in expired_ids {
            self.waiver_store.remove_waiver(id)?;
        }

        Ok(count)
    }

    /// Export waivers to YAML for persistence
    pub fn export_waivers(&self, task_id: Uuid, output_path: PathBuf) -> Result<(), WaiverError> {
        let waivers = self.get_active_waivers(task_id);

        if waivers.is_empty() {
            return Ok(());
        }

        let export_data = WaiverExport {
            task_id,
            exported_at: Utc::now(),
            waivers,
        };

        let yaml = serde_yaml::to_string(&export_data)
            .map_err(WaiverError::Serialization)?;

        std::fs::write(output_path, yaml)
            .map_err(|e| WaiverError::Storage(e.to_string()))?;

        Ok(())
    }
}

/// Export format for waivers
#[derive(Debug, Serialize, Deserialize)]
struct WaiverExport {
    task_id: Uuid,
    exported_at: DateTime<Utc>,
    waivers: Vec<Waiver>,
}

impl WaiverStore {
    /// Store waiver in memory
    fn store_waiver(&mut self, waiver: Waiver) -> Result<(), WaiverError> {
        let waiver_id = waiver.id;
        let task_id = waiver.task_id;

        self.waivers.insert(waiver_id, waiver);
        self.task_waivers.entry(task_id).or_insert_with(Vec::new).push(waiver_id);

        Ok(())
    }

    /// Update existing waiver
    fn update_waiver(&mut self, waiver: Waiver) -> Result<(), WaiverError> {
        self.waivers.insert(waiver.id, waiver);
        Ok(())
    }

    /// Remove waiver
    fn remove_waiver(&mut self, waiver_id: Uuid) -> Result<(), WaiverError> {
        if let Some(waiver) = self.waivers.remove(&waiver_id) {
            if let Some(task_waivers) = self.task_waivers.get_mut(&waiver.task_id) {
                task_waivers.retain(|&id| id != waiver_id);
            }
        }
        Ok(())
    }

    /// Get waiver by ID
    fn get_waiver(&self, waiver_id: Uuid) -> Option<Waiver> {
        self.waivers.get(&waiver_id).cloned()
    }

    /// Get waiver ID for task (returns most recent)
    fn get_waiver_id(&self, task_id: Uuid) -> Option<Uuid> {
        self.task_waivers.get(&task_id)
            .and_then(|waivers| waivers.last())
            .copied()
    }

    /// Get all waivers for task
    fn get_task_waivers(&self, task_id: Uuid) -> Vec<Uuid> {
        self.task_waivers.get(&task_id)
            .cloned()
            .unwrap_or_default()
    }
}

impl Waiver {
    /// Check if waiver is still valid
    pub fn is_valid(&self) -> bool {
        Utc::now() <= self.expires_at && self.used_count < self.max_uses
    }

    /// Get remaining uses
    pub fn remaining_uses(&self) -> usize {
        if self.used_count >= self.max_uses {
            0
        } else {
            self.max_uses - self.used_count
        }
    }

    /// Get time until expiry
    pub fn time_until_expiry(&self) -> Duration {
        let now = Utc::now();
        if now >= self.expires_at {
            Duration::zero()
        } else {
            self.expires_at.signed_duration_since(now).to_std()
                .unwrap_or(Duration::zero())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::caws::budget_checker::BudgetLimits;

    #[test]
    fn test_waiver_generation() {
        let mut generator = WaiverGenerator::new();

        let original_limits = BudgetLimits { max_files: 10, max_loc: 1000 };
        let granted_limits = BudgetLimits { max_files: 15, max_loc: 1500 };

        let waiver_id = generator.generate_budget_overrun_waiver(
            Uuid::new_v4(),
            "test-council".to_string(),
            original_limits,
            granted_limits,
            "Need more budget for complex refactoring".to_string(),
            "Medium risk - monitored closely".to_string(),
        ).unwrap();

        let validation = generator.validate_waiver(waiver_id);
        match validation {
            WaiverValidation::Valid(waiver) => {
                assert_eq!(waiver.original_limits.max_files, 10);
                assert_eq!(waiver.granted_limits.max_files, 15);
                assert!(waiver.is_valid());
            }
            _ => panic!("Waiver should be valid"),
        }
    }

    #[test]
    fn test_waiver_usage_tracking() {
        let mut generator = WaiverGenerator::new();

        let waiver_id = generator.generate_budget_overrun_waiver(
            Uuid::new_v4(),
            "test-council".to_string(),
            BudgetLimits { max_files: 5, max_loc: 500 },
            BudgetLimits { max_files: 10, max_loc: 1000 },
            "Test waiver".to_string(),
            "Low risk".to_string(),
        ).unwrap();

        // Use waiver multiple times
        for _ in 0..5 {
            generator.use_waiver(waiver_id).unwrap();
        }

        // Check usage
        if let WaiverValidation::Valid(waiver) = generator.validate_waiver(waiver_id) {
            assert_eq!(waiver.used_count, 5);
            assert_eq!(waiver.remaining_uses(), 5); // max_uses = 10
        }

        // Exceed usage limit
        for _ in 0..6 {
            let _ = generator.use_waiver(waiver_id); // Some will fail
        }

        // Should now be exceeded
        match generator.validate_waiver(waiver_id) {
            WaiverValidation::ExceededUsage(_) => {} // Expected
            other => panic!("Expected ExceededUsage, got {:?}", other),
        }
    }
}
