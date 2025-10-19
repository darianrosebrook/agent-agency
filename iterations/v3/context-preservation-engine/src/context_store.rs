use crate::types::*;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, warn};
use uuid::Uuid;

/// Context store for persistent storage and retrieval of contexts
#[derive(Debug)]
pub struct ContextStore {
    /// Store configuration
    config: ContextPreservationConfig,
    /// In-memory context storage (context_id -> (data, metadata))
    context_storage: Arc<RwLock<HashMap<Uuid, (ContextData, ContextMetadata)>>>,
    /// In-memory relationship storage (context_id -> relationships)
    relationship_storage: Arc<RwLock<HashMap<Uuid, Vec<ContextRelationship>>>>,
    /// In-memory cross-reference storage (context_id -> cross_references)
    cross_reference_storage: Arc<RwLock<HashMap<Uuid, Vec<CrossReference>>>>,
    /// In-memory synthesis result storage (context_id -> synthesis_results)
    synthesis_storage: Arc<RwLock<HashMap<Uuid, Vec<SynthesisResult>>>>,
}

impl ContextStore {
    /// Create a new context store
    pub fn new(config: ContextPreservationConfig) -> Result<Self> {
        debug!("Initializing context store");
        Ok(Self {
            config,
            context_storage: Arc::new(RwLock::new(HashMap::new())),
            relationship_storage: Arc::new(RwLock::new(HashMap::new())),
            cross_reference_storage: Arc::new(RwLock::new(HashMap::new())),
            synthesis_storage: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Store context
    pub async fn store_context(
        &self,
        context_id: &Uuid,
        tenant_id: &str,
        context_data: &ContextData,
        metadata: &ContextMetadata,
    ) -> Result<StorageResult> {
        debug!("Storing context: {} for tenant: {}", context_id, tenant_id);

        // Validate context size against configuration limits
        if context_data.content.len() as u64 > self.config.storage.max_context_size {
            return Err(anyhow::anyhow!(
                "Context size {} exceeds maximum allowed size {}",
                context_data.content.len(),
                self.config.storage.max_context_size
            ));
        }

        // Check tenant limits if multi-tenant is enabled
        if self.config.multi_tenant.enabled {
            let default_limits = TenantLimits {
                max_contexts: self.config.storage.max_contexts_per_tenant,
                max_context_size: self.config.storage.max_context_size,
                retention_hours: self.config.storage.retention_hours,
                max_concurrent_operations: 10,
            };
            let tenant_limits = self
                .config
                .multi_tenant
                .tenant_limits
                .get(tenant_id)
                .unwrap_or(&default_limits);

            if context_data.content.len() as u64 > tenant_limits.max_context_size {
                return Err(anyhow::anyhow!(
                    "Context size {} exceeds tenant limit {}",
                    context_data.content.len(),
                    tenant_limits.max_context_size
                ));
            }

            // TODO: Implement proper tenant context limits checking instead of simplified counting
            // - [ ] Integrate with tenant management system for dynamic limits
            // - [ ] Implement context size estimation with compression awareness
            // - [ ] Add context aging and automatic cleanup policies
            // - [ ] Support different context types with varying size limits
            // - [ ] Implement context usage analytics and quota tracking
            // - [ ] Add context compression and deduplication
            // - [ ] Support context prioritization and eviction strategies
            // Check total contexts for this tenant (simplified check)
            let storage = self.context_storage.read().await;
            let tenant_context_count = storage
                .values()
                .filter(|(_, meta)| {
                    meta.relationships
                        .iter()
                        .any(|rel| rel.description.contains(tenant_id))
                })
                .count();

            if tenant_context_count >= tenant_limits.max_contexts as usize {
                return Err(anyhow::anyhow!(
                    "Tenant {} has reached maximum context limit {}",
                    tenant_id,
                    tenant_limits.max_contexts
                ));
            }
        }

        // Store context data and metadata
        let start_time = std::time::Instant::now();
        {
            let mut storage = self.context_storage.write().await;
            storage.insert(*context_id, (context_data.clone(), metadata.clone()));
        }

        // Store relationships if present
        if !metadata.relationships.is_empty() {
            let mut relationship_storage = self.relationship_storage.write().await;
            relationship_storage.insert(*context_id, metadata.relationships.clone());
        }

        let storage_time_ms = start_time.elapsed().as_millis() as u64;

        debug!(
            "Successfully stored context {} for tenant {} in {}ms",
            context_id, tenant_id, storage_time_ms
        );

        Ok(StorageResult {
            stored: true,
            storage_id: *context_id,
            storage_time_ms,
        })
    }

    /// Retrieve context
    pub async fn retrieve_context(
        &self,
        context_id: &Uuid,
        tenant_id: &str,
    ) -> Result<Option<(ContextData, ContextMetadata)>> {
        debug!(
            "Retrieving context: {} for tenant: {}",
            context_id, tenant_id
        );

        // Query in-memory storage
        let storage = self.context_storage.read().await;

        if let Some((context_data, metadata)) = storage.get(context_id) {
            // Check tenant access if multi-tenant is enabled
            if self.config.multi_tenant.enabled {
                // TODO: Implement proper tenant access control instead of simplified relationship checking
                // - [ ] Integrate with proper tenant management and authentication system
                // - [ ] Implement role-based access control (RBAC) for context operations
                // - [ ] Add context ownership and sharing permissions
                // - [ ] Support hierarchical tenant relationships and inheritance
                // - [ ] Implement access auditing and logging
                // - [ ] Add context encryption for tenant isolation
                // - [ ] Support cross-tenant context sharing with explicit permissions
                // Simple tenant validation based on relationships (this is a simplified check)
                let has_tenant_access = metadata
                    .relationships
                    .iter()
                    .any(|rel| rel.description.contains(tenant_id));

                if !has_tenant_access
                    && self.config.multi_tenant.isolation_level == TenantIsolationLevel::Strict
                {
                    warn!(
                        "Access denied: tenant {} cannot access context {}",
                        tenant_id, context_id
                    );
                    return Ok(None);
                }
            }

            debug!(
                "Successfully retrieved context {} for tenant {}",
                context_id, tenant_id
            );
            Ok(Some((context_data.clone(), metadata.clone())))
        } else {
            debug!("Context {} not found", context_id);
            Ok(None)
        }
    }

    /// Get context relationships
    pub async fn get_context_relationships(
        &self,
        context_id: &Uuid,
    ) -> Result<Vec<ContextRelationship>> {
        debug!("Getting relationships for context: {}", context_id);

        // Query relationship storage
        let relationship_storage = self.relationship_storage.read().await;

        if let Some(relationships) = relationship_storage.get(context_id) {
            debug!(
                "Found {} relationships for context {}",
                relationships.len(),
                context_id
            );
            Ok(relationships.clone())
        } else {
            debug!("No relationships found for context {}", context_id);
            Ok(Vec::new())
        }
    }

    /// Get context cross-references
    pub async fn get_context_cross_references(
        &self,
        context_id: &Uuid,
    ) -> Result<Vec<CrossReference>> {
        debug!("Getting cross-references for context: {}", context_id);

        // Query cross-reference storage
        let cross_reference_storage = self.cross_reference_storage.read().await;

        if let Some(cross_references) = cross_reference_storage.get(context_id) {
            debug!(
                "Found {} cross-references for context {}",
                cross_references.len(),
                context_id
            );
            Ok(cross_references.clone())
        } else {
            debug!("No cross-references found for context {}", context_id);
            Ok(Vec::new())
        }
    }

    /// Get context synthesis results
    pub async fn get_context_synthesis_results(
        &self,
        context_id: &Uuid,
    ) -> Result<Vec<SynthesisResult>> {
        debug!("Getting synthesis results for context: {}", context_id);

        // Query synthesis result storage
        let synthesis_storage = self.synthesis_storage.read().await;

        if let Some(synthesis_results) = synthesis_storage.get(context_id) {
            debug!(
                "Found {} synthesis results for context {}",
                synthesis_results.len(),
                context_id
            );
            Ok(synthesis_results.clone())
        } else {
            debug!("No synthesis results found for context {}", context_id);
            Ok(Vec::new())
        }
    }

    /// Health check
    pub async fn health_check(&self) -> Result<bool> {
        debug!("Performing context store health check");

        // Check in-memory storage availability
        let context_count = self.context_storage.read().await.len();
        let relationship_count = self.relationship_storage.read().await.len();
        let cross_ref_count = self.cross_reference_storage.read().await.len();
        let synthesis_count = self.synthesis_storage.read().await.len();

        debug!(
            "Context store health check: {} contexts, {} relationships, {} cross-refs, {} synthesis results",
            context_count, relationship_count, cross_ref_count, synthesis_count
        );

        // Check configuration validity
        if self.config.storage.max_context_size == 0 {
            warn!("Invalid configuration: max_context_size is 0");
            return Ok(false);
        }

        // Check if we can perform basic operations
        let test_context_id = Uuid::new_v4();
        let test_data = ContextData {
            content: "health_check_test".to_string(),
            format: ContextFormat::Text,
            encoding: "utf-8".to_string(),
            compression: None,
            checksum: "test".to_string(),
        };

        let test_metadata = ContextMetadata {
            context_type: ContextType::System,
            priority: ContextPriority::Low,
            tags: vec!["health_check".to_string()],
            description: "Health check test".to_string(),
            source: "system".to_string(),
            version: "1.0".to_string(),
            dependencies: vec![],
            relationships: vec![],
        };

        // Test storage operation
        match self
            .store_context(
                &test_context_id,
                "health_check_tenant",
                &test_data,
                &test_metadata,
            )
            .await
        {
            Ok(result) => {
                if !result.stored {
                    warn!("Health check failed: context storage returned stored=false");
                    return Ok(false);
                }
            }
            Err(e) => {
                error!("Health check failed during storage test: {}", e);
                return Ok(false);
            }
        }

        // Test retrieval operation
        match self
            .retrieve_context(&test_context_id, "health_check_tenant")
            .await
        {
            Ok(Some(_)) => {
                debug!("Health check passed: storage and retrieval working");
                Ok(true)
            }
            Ok(None) => {
                warn!("Health check failed: context not found after storage");
                Ok(false)
            }
            Err(e) => {
                error!("Health check failed during retrieval test: {}", e);
                Ok(false)
            }
        }
    }
}

/// Storage result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageResult {
    /// Whether storage was successful
    pub stored: bool,
    /// Storage ID
    pub storage_id: Uuid,
    /// Storage time (milliseconds)
    pub storage_time_ms: u64,
}
