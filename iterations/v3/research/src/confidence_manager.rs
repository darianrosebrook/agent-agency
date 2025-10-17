//! Confidence Manager for Knowledge Updates
//!
//! Implements immutable knowledge updates with confidence-based reinforcement.
//! Manages knowledge decay and skepticism for external knowledge sources.
//!
//! Ported from V2 ConfidenceManager.ts with Rust optimizations.

use crate::types::*;
use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Confidence reinforcement request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceReinforcementRequest {
    pub entity_id: String,
    pub content: String,
    pub source: String,
    pub confidence: f64,
    pub reinforcement_type: ReinforcementType,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Type of confidence reinforcement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReinforcementType {
    /// Direct confirmation from authoritative source
    DirectConfirmation,
    /// Multiple independent sources agree
    Consensus,
    /// Expert validation
    ExpertValidation,
    /// Cross-reference validation
    CrossReference,
    /// Time-based decay
    TimeDecay,
}

/// Knowledge update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeUpdateRequest {
    pub entity_id: String,
    pub content: String,
    pub source: String,
    pub confidence: f64,
    pub entity_type: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Knowledge entry with confidence tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEntry {
    pub id: Uuid,
    pub entity_id: String,
    pub content: String,
    pub source: String,
    pub confidence: f64,
    pub entity_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub decay_rate: f64,
    pub reinforcement_count: u32,
    pub last_reinforcement: Option<DateTime<Utc>>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Confidence manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceManagerConfig {
    pub default_decay_rate: f64,
    pub reinforcement_step: f64,
    pub skepticism_threshold: f64,
    pub max_confidence: f64,
    pub min_confidence: f64,
    pub enable_decay: bool,
    pub enable_reinforcement: bool,
}

impl Default for ConfidenceManagerConfig {
    fn default() -> Self {
        Self {
            default_decay_rate: 0.95, // 5% decay per update
            reinforcement_step: 0.05, // 5% confidence increase
            skepticism_threshold: 0.5, // Flag for review below this
            max_confidence: 1.0,
            min_confidence: 0.0,
            enable_decay: true,
            enable_reinforcement: true,
        }
    }
}

/// Confidence manager trait
#[async_trait]
pub trait IConfidenceManager: Send + Sync {
    /// Update knowledge entry with new information (immutable update)
    async fn update_knowledge(&self, request: KnowledgeUpdateRequest) -> Result<()>;
    
    /// Reinforce confidence for existing knowledge
    async fn reinforce_confidence(&self, request: ConfidenceReinforcementRequest) -> Result<()>;
    
    /// Get confidence score for an entity
    async fn get_confidence(&self, entity_id: &str) -> Result<f64>;
    
    /// Get knowledge entry with confidence information
    async fn get_knowledge_entry(&self, entity_id: &str) -> Result<Option<KnowledgeEntry>>;
    
    /// Apply time-based decay to knowledge entries
    async fn apply_decay(&self, entity_ids: Vec<String>) -> Result<()>;
    
    /// Get entries below skepticism threshold
    async fn get_skeptical_entries(&self) -> Result<Vec<KnowledgeEntry>>;
}

/// Confidence manager implementation
#[derive(Debug)]
pub struct ConfidenceManager {
    config: ConfidenceManagerConfig,
    knowledge_entries: Arc<RwLock<HashMap<String, KnowledgeEntry>>>,
    reinforcement_history: Arc<RwLock<HashMap<String, Vec<ConfidenceReinforcementRequest>>>>,
}

impl ConfidenceManager {
    /// Create a new confidence manager
    pub fn new(config: ConfidenceManagerConfig) -> Self {
        Self {
            config,
            knowledge_entries: Arc::new(RwLock::new(HashMap::new())),
            reinforcement_history: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Calculate confidence score with decay applied
    fn calculate_decayed_confidence(&self, entry: &KnowledgeEntry) -> f64 {
        if !self.config.enable_decay {
            return entry.confidence;
        }

        let now = Utc::now();
        let time_since_update = now.signed_duration_since(entry.updated_at);
        let hours_since_update = time_since_update.num_hours() as f64;
        
        // Apply exponential decay: confidence * decay_rate^(hours/24)
        let decay_factor = self.config.default_decay_rate.powf(hours_since_update / 24.0);
        let decayed_confidence = entry.confidence * decay_factor;
        
        // Ensure confidence stays within bounds
        decayed_confidence.max(self.config.min_confidence).min(self.config.max_confidence)
    }

    /// Calculate reinforcement bonus based on type
    fn calculate_reinforcement_bonus(&self, reinforcement_type: &ReinforcementType) -> f64 {
        match reinforcement_type {
            ReinforcementType::DirectConfirmation => 0.15, // 15% boost
            ReinforcementType::Consensus => 0.10, // 10% boost
            ReinforcementType::ExpertValidation => 0.12, // 12% boost
            ReinforcementType::CrossReference => 0.08, // 8% boost
            ReinforcementType::TimeDecay => -0.05, // 5% decay
        }
    }

    /// Validate confidence score
    fn validate_confidence(&self, confidence: f64) -> Result<f64> {
        if confidence < self.config.min_confidence || confidence > self.config.max_confidence {
            return Err(anyhow::anyhow!(
                "Confidence score {} is outside valid range [{}, {}]",
                confidence,
                self.config.min_confidence,
                self.config.max_confidence
            ));
        }
        Ok(confidence)
    }
}

#[async_trait]
impl IConfidenceManager for ConfidenceManager {
    async fn update_knowledge(&self, request: KnowledgeUpdateRequest) -> Result<()> {
        let confidence = self.validate_confidence(request.confidence)?;
        
        let now = Utc::now();
        let entry = KnowledgeEntry {
            id: Uuid::new_v4(),
            entity_id: request.entity_id.clone(),
            content: request.content,
            source: request.source,
            confidence,
            entity_type: request.entity_type,
            created_at: now,
            updated_at: now,
            decay_rate: self.config.default_decay_rate,
            reinforcement_count: 0,
            last_reinforcement: None,
            metadata: request.metadata,
        };

        let entity_id = request.entity_id.clone();
        let mut entries = self.knowledge_entries.write().await;
        entries.insert(entity_id.clone(), entry);
        
        info!(
            "Updated knowledge for entity {} with confidence {}",
            entity_id, confidence
        );
        
        Ok(())
    }

    async fn reinforce_confidence(&self, request: ConfidenceReinforcementRequest) -> Result<()> {
        if !self.config.enable_reinforcement {
            return Ok(());
        }

        let mut entries = self.knowledge_entries.write().await;
        let mut history = self.reinforcement_history.write().await;

        if let Some(entry) = entries.get_mut(&request.entity_id) {
            let bonus = self.calculate_reinforcement_bonus(&request.reinforcement_type);
            let new_confidence = (entry.confidence + bonus)
                .max(self.config.min_confidence)
                .min(self.config.max_confidence);

            entry.confidence = new_confidence;
            entry.reinforcement_count += 1;
            entry.last_reinforcement = Some(Utc::now());
            entry.updated_at = Utc::now();

            // Store reinforcement history
            let entity_id = request.entity_id.clone();
            let old_confidence = entry.confidence - bonus;
            history
                .entry(entity_id.clone())
                .or_insert_with(Vec::new)
                .push(request);

            info!(
                "Reinforced confidence for entity {}: {} -> {} (bonus: {})",
                entity_id, old_confidence, new_confidence, bonus
            );
        } else {
            warn!(
                "Attempted to reinforce confidence for unknown entity: {}",
                request.entity_id
            );
        }

        Ok(())
    }

    async fn get_confidence(&self, entity_id: &str) -> Result<f64> {
        let entries = self.knowledge_entries.read().await;
        if let Some(entry) = entries.get(entity_id) {
            Ok(self.calculate_decayed_confidence(entry))
        } else {
            Ok(0.0) // Default confidence for unknown entities
        }
    }

    async fn get_knowledge_entry(&self, entity_id: &str) -> Result<Option<KnowledgeEntry>> {
        let entries = self.knowledge_entries.read().await;
        Ok(entries.get(entity_id).cloned())
    }

    async fn apply_decay(&self, entity_ids: Vec<String>) -> Result<()> {
        if !self.config.enable_decay {
            return Ok(());
        }

        let mut entries = self.knowledge_entries.write().await;
        let now = Utc::now();

        for entity_id in entity_ids {
            if let Some(entry) = entries.get_mut(&entity_id) {
                let decayed_confidence = self.calculate_decayed_confidence(entry);
                entry.confidence = decayed_confidence;
                entry.updated_at = now;

                debug!(
                    "Applied decay to entity {}: confidence -> {}",
                    entity_id, decayed_confidence
                );
            }
        }

        Ok(())
    }

    async fn get_skeptical_entries(&self) -> Result<Vec<KnowledgeEntry>> {
        let entries = self.knowledge_entries.read().await;
        let skeptical_entries: Vec<KnowledgeEntry> = entries
            .values()
            .filter(|entry| {
                let decayed_confidence = self.calculate_decayed_confidence(entry);
                decayed_confidence < self.config.skepticism_threshold
            })
            .cloned()
            .collect();

        Ok(skeptical_entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_confidence_manager_basic_operations() {
        let config = ConfidenceManagerConfig::default();
        let manager = ConfidenceManager::new(config);

        // Test knowledge update
        let request = KnowledgeUpdateRequest {
            entity_id: "test-entity".to_string(),
            content: "Test content".to_string(),
            source: "test-source".to_string(),
            confidence: 0.8,
            entity_type: "test".to_string(),
            metadata: HashMap::new(),
        };

        manager.update_knowledge(request).await.unwrap();
        
        let confidence = manager.get_confidence("test-entity").await.unwrap();
        assert_eq!(confidence, 0.8);

        // Test confidence reinforcement
        let reinforcement = ConfidenceReinforcementRequest {
            entity_id: "test-entity".to_string(),
            content: "Test content".to_string(),
            source: "test-source".to_string(),
            confidence: 0.8,
            reinforcement_type: ReinforcementType::DirectConfirmation,
            metadata: HashMap::new(),
        };

        manager.reinforce_confidence(reinforcement).await.unwrap();
        
        let updated_confidence = manager.get_confidence("test-entity").await.unwrap();
        assert!(updated_confidence > 0.8); // Should be higher due to reinforcement
    }

    #[tokio::test]
    async fn test_confidence_decay() {
        let config = ConfidenceManagerConfig {
            default_decay_rate: 0.9, // 10% decay per day
            ..Default::default()
        };
        let manager = ConfidenceManager::new(config);

        let request = KnowledgeUpdateRequest {
            entity_id: "test-entity".to_string(),
            content: "Test content".to_string(),
            source: "test-source".to_string(),
            confidence: 1.0,
            entity_type: "test".to_string(),
            metadata: HashMap::new(),
        };

        manager.update_knowledge(request).await.unwrap();
        
        // Simulate time passing by manually updating the entry
        let mut entries = manager.knowledge_entries.write().await;
        if let Some(entry) = entries.get_mut("test-entity") {
            entry.updated_at = Utc::now() - chrono::Duration::hours(25); // 25 hours ago
        }
        drop(entries);

        let decayed_confidence = manager.get_confidence("test-entity").await.unwrap();
        assert!(decayed_confidence < 1.0); // Should be decayed
    }

    #[tokio::test]
    async fn test_skeptical_entries() {
        let config = ConfidenceManagerConfig {
            skepticism_threshold: 0.5,
            ..Default::default()
        };
        let manager = ConfidenceManager::new(config);

        // Add a low-confidence entry
        let request = KnowledgeUpdateRequest {
            entity_id: "low-confidence-entity".to_string(),
            content: "Low confidence content".to_string(),
            source: "unreliable-source".to_string(),
            confidence: 0.3,
            entity_type: "test".to_string(),
            metadata: HashMap::new(),
        };

        manager.update_knowledge(request).await.unwrap();
        
        let skeptical_entries = manager.get_skeptical_entries().await.unwrap();
        assert_eq!(skeptical_entries.len(), 1);
        assert_eq!(skeptical_entries[0].entity_id, "low-confidence-entity");
    }
}
