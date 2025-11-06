//! Orchestration Service Adapter
//!
//! Adapts `agent-orchestration` implementations to `data-interfaces` service traits.

use async_trait::async_trait;
use data_interfaces::service_contracts::{
    OrchestrationService, ServiceError, TaskStatus, TaskStatusEnum,
};
use agent_agency_contracts::{
    WorkingSpec, TaskExecutionResult, TaskContext,
};
use std::sync::Arc;
use uuid::Uuid;
use agent_orchestration::{
    types::OrchestratorConfig,
    adapter::LegacyOrchestratorAdapter,
};
use chrono::Utc;

/// Adapter for orchestration service
pub struct OrchestrationServiceAdapter {
    adapter: Arc<LegacyOrchestratorAdapter>,
}

impl OrchestrationServiceAdapter {
    /// Create a new orchestration service adapter
    pub async fn new(config: OrchestratorConfig) -> Result<Self, ServiceError> {
        let adapter = LegacyOrchestratorAdapter::new(config)
            .await
            .map_err(|e| ServiceError::Internal(format!("Failed to create adapter: {}", e)))?;
        Ok(Self {
            adapter: Arc::new(adapter),
        })
    }
    
    /// Create with default configuration
    pub async fn with_defaults() -> Result<Self, ServiceError> {
        Self::new(OrchestratorConfig::default()).await
    }
}

#[async_trait]
impl OrchestrationService for OrchestrationServiceAdapter {
    async fn orchestrate_task(
        &self,
        spec: WorkingSpec,
        context: TaskContext,
    ) -> Result<TaskExecutionResult, ServiceError> {
        // Convert TaskContext to TaskDescriptor
        use agent_agency_contracts::types::planning::{TaskDescriptor, BlastRadius, RiskTier};
        use agent_agency_contracts::planning_io::ChangeBudget;
        use agent_agency_contracts::task_request::ScopeRestrictions;
        
        let task_descriptor = TaskDescriptor {
            task_id: context.task_id,
            description: format!("Orchestrate task {}", context.task_id),
            change_budget: ChangeBudget {
                max_files: 25,
                max_loc: 1000,
                max_migrations: 0,
                allow_breaking_changes: false,
                allow_new_dependencies: false,
                enforcement_mode: agent_agency_contracts::planning_io::BudgetEnforcement::Strict,
            },
            priority: agent_agency_contracts::types::planning::TaskPriority::Normal,
            execution_mode: agent_agency_contracts::types::planning::ExecutionMode::Auto,
            risk_tier: Some(RiskTier::Tier2),
            blast_radius: BlastRadius {
                modules: vec![],
                data_migration: false,
                external_deps: vec![],
            },
            scope_in: ScopeRestrictions {
                allowed_paths: vec![],
                blocked_paths: vec![],
            },
            scope_out: None,
            acceptance: None,
        };
        
        // Create diff stats placeholder
        use agent_orchestration::types::DiffStats;
        let diff_stats = DiffStats {
            files_changed: 0,
            lines_added: 0,
            lines_removed: 0,
            lines_modified: 0,
            files_added: 0,
            files_modified: 0,
            files_deleted: 0,
            lines_deleted: 0,
            binary_files_changed: 0,
        };
        
        // Call orchestration adapter
        self.adapter.orchestrate_task(
            &spec,
            &task_descriptor,
            &diff_stats,
            false, // tests_added
            true,  // deterministic
        ).await
        .map_err(|e| ServiceError::Internal(format!("Orchestration failed: {}", e)))
    }
    
    async fn get_task_status(&self, task_id: &Uuid) -> Result<TaskStatus, ServiceError> {
        // TODO: Implement actual status retrieval
        // For now, return a placeholder status
        Ok(TaskStatus {
            task_id: *task_id,
            status: TaskStatusEnum::Running,
            progress_percent: None,
            error_message: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }
    
    async fn pause_task(&self, task_id: &Uuid) -> Result<(), ServiceError> {
        // TODO: Implement actual pause logic
        Err(ServiceError::Internal("Pause not yet implemented".to_string()))
    }
    
    async fn resume_task(&self, task_id: &Uuid) -> Result<(), ServiceError> {
        // TODO: Implement actual resume logic
        Err(ServiceError::Internal("Resume not yet implemented".to_string()))
    }
    
    async fn cancel_task(&self, task_id: &Uuid) -> Result<(), ServiceError> {
        // TODO: Implement actual cancel logic
        Err(ServiceError::Internal("Cancel not yet implemented".to_string()))
    }
}


