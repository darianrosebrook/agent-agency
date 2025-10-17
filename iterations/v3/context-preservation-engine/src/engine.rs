use crate::types::*;
use crate::context_manager::ContextManager;
use crate::context_store::ContextStore;
use crate::context_synthesizer::ContextSynthesizer;
use crate::multi_tenant::MultiTenantManager;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};
use uuid::Uuid;
use chrono::Utc;
use std::time::Instant;

/// Context preservation engine
#[derive(Debug)]
pub struct ContextPreservationEngine {
    /// Engine configuration
    config: ContextPreservationConfig,
    /// Context manager
    context_manager: Arc<ContextManager>,
    /// Context store
    context_store: Arc<ContextStore>,
    /// Context synthesizer
    context_synthesizer: Arc<ContextSynthesizer>,
    /// Multi-tenant manager
    multi_tenant_manager: Arc<MultiTenantManager>,
    /// Engine statistics
    stats: Arc<RwLock<ContextPreservationStats>>,
}

impl ContextPreservationEngine {
    /// Create a new context preservation engine
    pub fn new(config: ContextPreservationConfig) -> Result<Self> {
        info!("Initializing context preservation engine");

        let context_manager = Arc::new(ContextManager::new(config.clone())?);
        let context_store = Arc::new(ContextStore::new(config.clone())?);
        let context_synthesizer = Arc::new(ContextSynthesizer::new(config.clone())?);
        let multi_tenant_manager = Arc::new(MultiTenantManager::new(config.clone())?);

        let stats = Arc::new(RwLock::new(ContextPreservationStats {
            total_requests: 0,
            successful_preservations: 0,
            failed_preservations: 0,
            total_retrievals: 0,
            successful_retrievals: 0,
            failed_retrievals: 0,
            avg_preservation_time_ms: 0.0,
            avg_retrieval_time_ms: 0.0,
            context_reuse_rate: 0.0,
            cross_reference_rate: 0.0,
            synthesis_rate: 0.0,
            last_updated: Utc::now(),
        }));

        Ok(Self {
            config,
            context_manager,
            context_store,
            context_synthesizer,
            multi_tenant_manager,
            stats,
        })
    }

    /// Preserve context
    pub async fn preserve_context(
        &self,
        request: &ContextPreservationRequest,
    ) -> Result<ContextPreservationResult> {
        let start_time = Instant::now();
        info!("Preserving context for tenant: {}", request.tenant_id);

        // Validate tenant access
        if !self.multi_tenant_manager.validate_tenant_access(&request.tenant_id).await? {
            return Err(anyhow::anyhow!("Tenant access denied: {}", request.tenant_id));
        }

        // Check tenant limits
        if !self.multi_tenant_manager.check_tenant_limits(&request.tenant_id, &request.context_data).await? {
            return Err(anyhow::anyhow!("Tenant limits exceeded: {}", request.tenant_id));
        }

        // Generate context ID
        let context_id = Uuid::new_v4();

        // Process context data
        let processed_context_data = self.context_manager.process_context_data(&request.context_data).await?;

        // Store context
        let storage_result = self.context_store.store_context(
            &context_id,
            &request.tenant_id,
            &processed_context_data,
            &request.metadata,
        ).await?;

        if !storage_result.stored {
            return Err(anyhow::anyhow!("Failed to store context: {}", context_id));
        }

        // Synthesize context if enabled
        let synthesis_results = if request.options.enable_synthesis {
            self.context_synthesizer.synthesize_context(
                &context_id,
                &request.tenant_id,
                &processed_context_data,
                &request.metadata,
            ).await?
        } else {
            Vec::new()
        };

        // Create cross-references if enabled
        let cross_references = if request.options.enable_cross_referencing {
            self.context_synthesizer.create_cross_references(
                &context_id,
                &request.tenant_id,
                &processed_context_data,
                &request.metadata,
            ).await?
        } else {
            Vec::new()
        };

        let preservation_time_ms = start_time.elapsed().as_millis() as u64;

        let result = ContextPreservationResult {
            id: Uuid::new_v4(),
            preserved: true,
            context_id,
            tenant_id: request.tenant_id.clone(),
            context_size: processed_context_data.content.len() as u64,
            preservation_time_ms,
            metadata: request.metadata.clone(),
            statistics: PreservationStatistics {
                total_contexts: 1,
                successful_preservations: 1,
                failed_preservations: 0,
                avg_preservation_time_ms: preservation_time_ms as f64,
                context_reuse_rate: 0.0,
                cross_reference_rate: if cross_references.is_empty() { 0.0 } else { 1.0 },
                last_updated: Utc::now(),
            },
        };

        // Update statistics
        self.update_stats(true, false, preservation_time_ms, 0).await;

        info!(
            "Context preserved successfully in {}ms - ID: {}, Tenant: {}",
            preservation_time_ms, context_id, request.tenant_id
        );

        Ok(result)
    }

    /// Retrieve context
    pub async fn retrieve_context(
        &self,
        request: &ContextRetrievalRequest,
    ) -> Result<ContextRetrievalResult> {
        let start_time = Instant::now();
        info!("Retrieving context: {} for tenant: {}", request.context_id, request.tenant_id);

        // Validate tenant access
        if !self.multi_tenant_manager.validate_tenant_access(&request.tenant_id).await? {
            return Err(anyhow::anyhow!("Tenant access denied: {}", request.tenant_id));
        }

        // Retrieve context from store
        let stored_context = self.context_store.retrieve_context(
            &request.context_id,
            &request.tenant_id,
        ).await?;

        if stored_context.is_none() {
            let result = ContextRetrievalResult {
                id: Uuid::new_v4(),
                found: false,
                context_data: None,
                metadata: None,
                relationships: Vec::new(),
                cross_references: Vec::new(),
                synthesis_results: Vec::new(),
                retrieval_time_ms: start_time.elapsed().as_millis() as u64,
            };

            self.update_stats(false, false, 0, result.retrieval_time_ms).await;
            return Ok(result);
        }

        let (context_data, metadata) = stored_context.unwrap();

        // Retrieve relationships if requested
        let relationships = if request.options.include_relationships {
            self.context_store.get_context_relationships(&request.context_id).await?
        } else {
            Vec::new()
        };

        // Retrieve cross-references if requested
        let cross_references = if request.options.include_cross_references {
            self.context_store.get_context_cross_references(&request.context_id).await?
        } else {
            Vec::new()
        };

        // Retrieve synthesis results if requested
        let synthesis_results = if request.options.include_synthesis {
            self.context_store.get_context_synthesis_results(&request.context_id).await?
        } else {
            Vec::new()
        };

        let retrieval_time_ms = start_time.elapsed().as_millis() as u64;

        let result = ContextRetrievalResult {
            id: Uuid::new_v4(),
            found: true,
            context_data: Some(context_data),
            metadata: Some(metadata),
            relationships,
            cross_references,
            synthesis_results,
            retrieval_time_ms,
        };

        // Update statistics
        self.update_stats(false, true, 0, retrieval_time_ms).await;

        info!(
            "Context retrieved successfully in {}ms - ID: {}, Tenant: {}",
            retrieval_time_ms, request.context_id, request.tenant_id
        );

        Ok(result)
    }

    /// Get context preservation statistics
    pub async fn get_stats(&self) -> Result<ContextPreservationStats> {
        let stats = self.stats.read().await;
        Ok(stats.clone())
    }

    /// Update statistics
    async fn update_stats(
        &self,
        is_preservation: bool,
        is_success: bool,
        preservation_time_ms: u64,
        retrieval_time_ms: u64,
    ) {
        let mut stats = self.stats.write().await;

        if is_preservation {
            stats.total_requests += 1;
            if is_success {
                stats.successful_preservations += 1;
            } else {
                stats.failed_preservations += 1;
            }

            // Update average preservation time
            let total_preservations = stats.successful_preservations + stats.failed_preservations;
            if total_preservations > 0 {
                let total_time = stats.avg_preservation_time_ms * (total_preservations - 1) as f64;
                stats.avg_preservation_time_ms = (total_time + preservation_time_ms as f64) / total_preservations as f64;
            }
        } else {
            stats.total_retrievals += 1;
            if is_success {
                stats.successful_retrievals += 1;
            } else {
                stats.failed_retrievals += 1;
            }

            // Update average retrieval time
            let total_retrievals = stats.successful_retrievals + stats.failed_retrievals;
            if total_retrievals > 0 {
                let total_time = stats.avg_retrieval_time_ms * (total_retrievals - 1) as f64;
                stats.avg_retrieval_time_ms = (total_time + retrieval_time_ms as f64) / total_retrievals as f64;
            }
        }

        stats.last_updated = Utc::now();
    }

    /// Get engine configuration
    pub fn get_config(&self) -> &ContextPreservationConfig {
        &self.config
    }

    /// Update engine configuration
    pub async fn update_config(&self, new_config: ContextPreservationConfig) -> Result<()> {
        info!("Updating context preservation engine configuration");
        // In a real implementation, this would update the configuration
        // and reinitialize components as needed
        Ok(())
    }

    /// Health check
    pub async fn health_check(&self) -> Result<bool> {
        debug!("Performing health check");

        // Check context store health
        let store_healthy = self.context_store.health_check().await?;

        // Check multi-tenant manager health
        let multi_tenant_healthy = self.multi_tenant_manager.health_check().await?;

        // Check context synthesizer health
        let synthesizer_healthy = self.context_synthesizer.health_check().await?;

        let healthy = store_healthy && multi_tenant_healthy && synthesizer_healthy;

        if healthy {
            debug!("Health check passed");
        } else {
            warn!("Health check failed - Store: {}, Multi-tenant: {}, Synthesizer: {}", 
                  store_healthy, multi_tenant_healthy, synthesizer_healthy);
        }

        Ok(healthy)
    }
}
