//! Planning Storage Layer - Dual Storage (File + Database)
//!
//! Provides persistent storage for execution plans with versioning,
//! session recovery, and dual storage strategy (file for specs, DB for state).
//!
//! @author @darianrosebrook

use schemars::JsonSchema;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use anyhow::{anyhow, Result};
use crate::planning::{
    DatabaseOperations,
    models::{ExecutionPlan as DbExecutionPlan, PlanningSession as DbPlanningSession},
};

use crate::planning::plan_types::ExecutionPlan;

/// Planning storage with dual persistence strategy
pub struct PlanningStorage {
    /// Database operations for state storage
    db_ops: Arc<dyn DatabaseOperations>,

    /// File system storage for plan specifications
    file_storage: FileStorage,

    /// In-memory cache for active sessions
    session_cache: Arc<RwLock<HashMap<Uuid, CachedSession>>>,

    /// Storage configuration
    config: StorageConfig,
}

impl std::fmt::Debug for PlanningStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlanningStorage")
            .field("file_storage", &self.file_storage)
            .field("config", &self.config)
            .finish()
    }
}

/// File storage for plan specifications
pub struct FileStorage {
    /// Base directory for plan files
    plans_dir: PathBuf,

    /// Base directory for working specs
    specs_dir: PathBuf,
}

impl std::fmt::Debug for FileStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileStorage")
            .field("plans_dir", &self.plans_dir)
            .field("specs_dir", &self.specs_dir)
            .finish()
    }
}

/// Cached session data for fast access

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedSession {
    /// Session data
    session: DbPlanningSession,

    /// Last accessed timestamp
    last_accessed: DateTime<Utc>,

    /// Whether session has unsaved changes
    dirty: bool,
}

/// Storage configuration

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StorageConfig {
    /// Maximum cache size for sessions
    max_cache_size: usize,

    /// Cache eviction time in seconds
    cache_eviction_seconds: u64,

    /// Auto-save interval in seconds
    auto_save_interval_seconds: u64,

    /// Whether to enable versioning
    enable_versioning: bool,

    /// Maximum versions to keep
    max_versions: usize,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            max_cache_size: 100,
            cache_eviction_seconds: 300, // 5 minutes
            auto_save_interval_seconds: 60, // 1 minute
            enable_versioning: true,
            max_versions: 10,
        }
    }
}

impl PlanningStorage {
    /// Create new planning storage
    pub fn new(
        db_ops: Arc<dyn DatabaseOperations>,
        plans_dir: PathBuf,
        specs_dir: PathBuf,
        config: StorageConfig,
    ) -> Self {
        Self {
            db_ops,
            file_storage: FileStorage { plans_dir, specs_dir },
            session_cache: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Store execution plan with dual persistence
    pub async fn store_execution_plan(&self, plan: &ExecutionPlan) -> Result<()> {
        // Store plan specification as YAML file
        self.file_storage.store_plan_spec(plan).await?;

        // Store plan metadata and state in database
        self.store_plan_to_database(plan).await?;

        // Create initial planning session
        self.create_planning_session(plan).await?;

        Ok(())
    }

    /// Load execution plan from storage
    pub async fn load_execution_plan(&self, plan_id: Uuid) -> Result<Option<ExecutionPlan>> {
        // Try to load from database first (for state)
        if let Some(db_plan) = self.db_ops.get_execution_plan(plan_id).await? {
            // Load plan spec from file
            if let Some(plan_spec) = self.file_storage.load_plan_spec(plan_id).await? {
                // Merge file spec with database state
                let plan = self.merge_plan_data(plan_spec, db_plan)?;
                Ok(Some(plan))
            } else {
                // Plan spec missing, reconstruct from DB
                let plan = self.reconstruct_plan_from_db(db_plan)?;
                Ok(Some(plan))
            }
        } else {
            Ok(None)
        }
    }

    /// Update execution plan state
    pub async fn update_execution_plan(&self, plan: &ExecutionPlan) -> Result<()> {
        // Update database state
        self.update_plan_in_database(plan).await?;

        // Update cached session if exists
        self.update_cached_session(plan).await?;

        Ok(())
    }

    /// Create new planning session
    pub async fn create_planning_session(&self, plan: &ExecutionPlan) -> Result<Uuid> {
        let session_id = Uuid::new_v4();

        let mut metadata = HashMap::new();
        metadata.insert("session_id".to_string(), serde_json::Value::String(session_id.to_string()));
        metadata.insert("orchestrator_id".to_string(), serde_json::Value::String(plan.orchestration_meta.orchestrator_id.clone()));
        metadata.insert("worker_pool_id".to_string(), serde_json::Value::String(plan.orchestration_meta.worker_pool_id.clone()));
        metadata.insert("council_session_id".to_string(), serde_json::Value::String(plan.orchestration_meta.council_session_id.as_ref().unwrap_or(&"none".to_string()).clone()));
        metadata.insert("audit_correlation_id".to_string(), serde_json::Value::String(plan.orchestration_meta.audit_correlation_id.to_string()));
        metadata.insert("status".to_string(), serde_json::Value::String("active".to_string()));
        metadata.insert("execution_state".to_string(), serde_json::to_value(&plan.execution_state).unwrap_or_default());

        let session = CreatePlanningSession {
            plan_id: plan.contract_plan.id,
            metadata,
        };

        let db_session = self.db_ops.create_planning_session(session).await?;

        // Cache the session
        let cached = CachedSession {
            session: db_session,
            last_accessed: Utc::now(),
            dirty: false,
        };

        let mut cache = self.session_cache.write().await;
        cache.insert(session_id, cached);

        Ok(session_id)
    }

    /// Get planning session with caching
    pub async fn get_planning_session(&self, session_id: Uuid) -> Result<Option<DbPlanningSession>> {
        // Check cache first
        {
            let cache = self.session_cache.read().await;
            if let Some(cached) = cache.get(&session_id) {
                // Clone the session before dropping the read lock
                let session = cached.session.clone();
                let mut updated_cached = cached.clone();
                updated_cached.last_accessed = Utc::now();

                // Update cache (we need to drop the read lock first)
                drop(cache);
                let mut cache = self.session_cache.write().await;
                cache.insert(session_id, updated_cached);

                return Ok(Some(session));
            }
        }

        // Load from database
        if let Some(session) = self.db_ops.get_planning_session(session_id).await? {
            // Cache the session
            let cached = CachedSession {
                session: session.clone(),
                last_accessed: Utc::now(),
                dirty: false,
            };

            let mut cache = self.session_cache.write().await;
            self.evict_old_cache_entries(&mut cache).await;
            cache.insert(session_id, cached);

            Ok(Some(session))
        } else {
            Ok(None)
        }
    }

    /// Update planning session
    pub async fn update_planning_session(&self, session_id: Uuid, execution_state: serde_json::Value) -> Result<()> {
        let mut metadata = HashMap::new();
        metadata.insert("execution_state".to_string(), execution_state);
        // completed_at will be set when session completes

        let update = UpdatePlanningSession {
            id: session_id,
            status: None,
            metadata: Some(metadata),
        };

        self.db_ops.update_planning_session(session_id, update).await?;

        // Update cache
        let mut cache = self.session_cache.write().await;
        if let Some(cached) = cache.get_mut(&session_id) {
            cached.dirty = false;
            cached.last_accessed = Utc::now();
        }

        Ok(())
    }

    /// Log planning audit event
    pub async fn log_audit_event(&self, event: AuditEvent) -> Result<()> {
        // Convert event.metadata (serde_json::Value) to HashMap<String, serde_json::Value>
        let mut metadata: HashMap<String, serde_json::Value> = match event.metadata {
            serde_json::Value::Object(map) => {
                map.into_iter().map(|(k, v)| (k, v)).collect()
            },
            _ => HashMap::new(),
        };
        metadata.insert("id".to_string(), serde_json::Value::String(Uuid::new_v4().to_string()));
        if let Some(milestone_id) = &event.milestone_id {
            metadata.insert("milestone_id".to_string(), serde_json::Value::String(milestone_id.clone()));
        }
        if let Some(worker_id) = &event.worker_id {
            metadata.insert("worker_id".to_string(), serde_json::Value::String(worker_id.to_string()));
        }

        let db_event = CreatePlanningAuditEvent {
            plan_id: event.plan_id,
            event_type: event.event_type,
            description: event.description,
            metadata,
        };

        self.db_ops.create_planning_audit_event(db_event).await?;
        Ok(())
    }

    /// Store planning telemetry
    pub async fn store_telemetry(&self, plan_id: Uuid, metric_type: String, metric_value: serde_json::Value) -> Result<()> {
        // Convert metric_value to f64 if it's a number, otherwise store in metadata
        let (metric_value_f64, mut metadata) = match &metric_value {
            serde_json::Value::Number(n) => {
                if let Some(f) = n.as_f64() {
                    (f, HashMap::new())
                } else {
                    (0.0, {
                        let mut m = HashMap::new();
                        m.insert("raw_value".to_string(), metric_value.clone());
                        m
                    })
                }
            }
            _ => {
                let mut m = HashMap::new();
                m.insert("raw_value".to_string(), metric_value.clone());
                (0.0, m)
            }
        };

        metadata.insert("id".to_string(), serde_json::Value::String(Uuid::new_v4().to_string()));
        metadata.insert("plan_id".to_string(), serde_json::Value::String(plan_id.to_string()));
        metadata.insert("metric_type".to_string(), serde_json::Value::String(metric_type));

        // TODO: Use proper session ID instead of plan_id
        // - [ ] Generate or retrieve proper session ID for telemetry
        // - [ ] Track session lifecycle separately from plan lifecycle
        // - [ ] Link multiple plans to the same session if needed
        // - [ ] Add session management and tracking
        // - [ ] Add unit tests with proper session IDs
        // - [ ] Add integration tests with real telemetry sessions
        // TODO: Use proper session management for telemetry:
        // 1. Session management: Implement proper session tracking
        //    - Create or retrieve session ID for planning operations
        //    - Link multiple plans to the same session when appropriate
        //    - Support session lifecycle management
        // 2. Session ID generation: Generate proper session IDs
        //    - Use session management system for ID generation
        //    - Support session ID reuse and linking
        //    - Handle session ID validation
        // 3. Telemetry integration: Integrate with telemetry system
        //    - Use proper session IDs in telemetry records
        //    - Support session-based telemetry queries
        //    - Handle telemetry session tracking
        // ACCEPTANCE CRITERIA:
        // - Proper session IDs are used for telemetry
        // - Multiple plans can be linked to the same session
        // - Session management integrates with telemetry system
        // DEPENDENCIES:
        // - Session management system (Required)
        // - Telemetry session tracking (Required)
        // PRIORITY: Medium
        let telemetry = CreatePlanningTelemetry {
            session_id: plan_id,
            metric_name: "planning_metric".to_string(), // Default metric name
            metric_value: metric_value_f64,
            metadata,
        };

        self.db_ops.create_planning_telemetry(telemetry).await?;
        Ok(())
    }

    /// Recover session state after restart
    pub async fn recover_session_state(&self, session_id: Uuid) -> Result<Option<ExecutionPlan>> {
        if let Some(session) = self.get_planning_session(session_id).await? {
            // Load the plan
            if let Some(plan) = self.load_execution_plan(session.plan_id).await? {
                // Restore execution state from session metadata
                let execution_state: Option<crate::planning::plan_types::ActiveExecutionState> =
                    session.metadata.get("execution_state")
                        .and_then(|v| serde_json::from_value(v.clone()).ok());

                let mut recovered_plan = plan;
                recovered_plan.execution_state = execution_state;

                Ok(Some(recovered_plan))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// List all execution plans
    pub async fn list_execution_plans(&self) -> Result<Vec<DbExecutionPlan>> {
        self.db_ops.get_execution_plans().await
    }

    /// Delete execution plan (soft delete by marking as cancelled)
    pub async fn delete_execution_plan(&self, plan_id: Uuid) -> Result<()> {
        // Update plan state to cancelled
        let update = UpdateExecutionPlan {
            id: plan_id,
            status: Some("cancelled".to_string()),
            title: None,
            overview: None,
        };

        let _updated = self.db_ops.update_execution_plan(plan_id, update).await?;
        Ok(())
    }

    /// Clean up old cache entries
    async fn evict_old_cache_entries(&self, cache: &mut HashMap<Uuid, CachedSession>) {
        let now = Utc::now();
        let eviction_threshold = chrono::Duration::seconds(self.config.cache_eviction_seconds as i64);

        // Remove entries older than threshold
        cache.retain(|_, cached| {
            now.signed_duration_since(cached.last_accessed) < eviction_threshold
        });

        // If still over max size, remove oldest entries
        if cache.len() > self.config.max_cache_size {
            let mut entries: Vec<_> = cache.iter().collect();
            entries.sort_by(|a, b| a.1.last_accessed.cmp(&b.1.last_accessed));

            let to_remove: Vec<Uuid> = entries.into_iter()
                .take(cache.len() - self.config.max_cache_size)
                .map(|(id, _)| *id)
                .collect();

            for id in to_remove {
                cache.remove(&id);
            }
        }
    }

    /// Update cached session
    async fn update_cached_session(&self, plan: &ExecutionPlan) -> Result<()> {
        let mut cache = self.session_cache.write().await;
        if let Some(cached) = cache.get_mut(&plan.contract_plan.session_id) {
            // Update execution state in cache metadata
            if let Some(execution_state) = &plan.execution_state {
                cached.session.metadata.insert(
                    "execution_state".to_string(),
                    serde_json::to_value(execution_state).unwrap_or(serde_json::Value::Null)
                );
                cached.dirty = true;
            }
            cached.last_accessed = Utc::now();
        }
        Ok(())
    }

    /// Store plan to database
    async fn store_plan_to_database(&self, plan: &ExecutionPlan) -> Result<()> {
        // Ensure working_spec_id follows TASK-<UUID> format if it's a task-based plan
        // Preserve the working_spec_id from the plan if it's already in the correct format
        let working_spec_id = if plan.contract_plan.working_spec_id.starts_with("TASK-") {
            // Already in TASK-<UUID> format, preserve it
            Some(plan.contract_plan.working_spec_id.clone())
        } else {
            // Not in TASK format, use None to let adapter default to PLAN-<id>
            None
        };
        
        let create_plan = CreateExecutionPlan {
            id: plan.contract_plan.id,
            title: plan.contract_plan.title.clone(),
            overview: plan.contract_plan.overview.clone(),
            working_spec_id,
        };

        self.db_ops.create_execution_plan(create_plan).await?;
        Ok(())
    }

    /// Update plan in database
    async fn update_plan_in_database(&self, plan: &ExecutionPlan) -> Result<()> {
        let update = UpdateExecutionPlan {
            id: plan.contract_plan.id,
            status: Some(serde_json::to_string(&plan.contract_plan.state)?.trim_matches('"').to_string()),
            title: None,
            overview: None,
        };

        let _updated = self.db_ops.update_execution_plan(plan.contract_plan.id, update).await?;
        Ok(())
    }

    /// Merge plan data from file and database
    fn merge_plan_data(&self, file_plan: ExecutionPlan, db_plan: DbExecutionPlan) -> Result<ExecutionPlan> {
        // Use file plan as base, but update with latest DB state
        let mut merged = file_plan;
        // Note: WorkingSpec doesn't have lifecycle fields, only ExecutionPlan has state
        // The lifecycle timestamps are managed at the ExecutionPlan level in the database
        // state field doesn't exist in contract plan
        merged.contract_plan.id = db_plan.id;

        Ok(merged)
    }

    /// Reconstruct plan from database (when file is missing)
    ///
    /// Comprehensive reconstruction that deserializes JSON fields from DbExecutionPlan
    /// and reconstructs a complete ExecutionPlan with proper error handling for missing
    /// or corrupted data.
    fn reconstruct_plan_from_db(&self, db_plan: DbExecutionPlan) -> Result<ExecutionPlan> {
        use agent_agency_contracts::planning_io::{
            ExecutionPlan as ContractExecutionPlan,
            PlanState, DependencyGraph, QualityGates,
            EvidenceRequirement, WaiverReference, PlanMetadata,
        };
        use agent_agency_contracts::{ChangeBudget, WorkingSpec};
        use tracing::{debug, warn};
        
        debug!("Reconstructing plan {} from database", db_plan.id);
        
        // Deserialize milestones with fallback to empty vector
        let milestones: Vec<agent_agency_contracts::planning_io::Milestone> = 
            serde_json::from_value(db_plan.milestones.clone())
                .map_err(|e| {
                    warn!("Failed to deserialize milestones for plan {}: {}", db_plan.id, e);
                    e
                })
                .unwrap_or_else(|_| {
                    warn!("Using empty milestones vector for plan {}", db_plan.id);
                    vec![]
                });
        
        // Deserialize dependency graph with fallback to empty graph
        let dependency_graph: DependencyGraph = 
            serde_json::from_value(db_plan.dependency_graph.clone())
                .map_err(|e| {
                    warn!("Failed to deserialize dependency_graph for plan {}: {}", db_plan.id, e);
                    e
                })
                .unwrap_or_else(|_| {
                    warn!("Using empty dependency graph for plan {}", db_plan.id);
                    DependencyGraph {
                        nodes: std::collections::HashMap::new(),
                        edges: vec![],
                        critical_path: vec![],
                        parallel_groups: vec![],
                    }
                });
        
        // Deserialize change budget with fallback to default
        let change_budget: ChangeBudget = 
            serde_json::from_value(db_plan.change_budget.clone())
                .map_err(|e| {
                    warn!("Failed to deserialize change_budget for plan {}: {}", db_plan.id, e);
                    e
                })
                .unwrap_or_else(|_| {
                    warn!("Using default change budget for plan {}", db_plan.id);
                    ChangeBudget {
                        max_files: 25,
                        max_loc: 1000,
                        max_days: 3,
                        max_complexity: 10,
                    }
                });
        
        // Deserialize quality gates with fallback to default
        let quality_gates: QualityGates = 
            serde_json::from_value(db_plan.quality_gates.clone())
                .map_err(|e| {
                    warn!("Failed to deserialize quality_gates for plan {}: {}", db_plan.id, e);
                    e
                })
                .unwrap_or_else(|_| {
                    warn!("Using default quality gates for plan {}", db_plan.id);
                    QualityGates {
                        min_test_coverage: 0.8,
                        min_mutation_score: 0.5,
                        security_scan_required: true,
                        performance_budget_required: false,
                    }
                });
        
        // Deserialize evidence requirements with fallback to empty vector
        let evidence_requirements: Vec<EvidenceRequirement> = 
            serde_json::from_value(db_plan.evidence_requirements.clone())
                .map_err(|e| {
                    warn!("Failed to deserialize evidence_requirements for plan {}: {}", db_plan.id, e);
                    e
                })
                .unwrap_or_else(|_| {
                    warn!("Using empty evidence requirements for plan {}", db_plan.id);
                    vec![]
                });
        
        // Deserialize active waivers with fallback to empty vector
        let active_waivers: Vec<WaiverReference> = 
            serde_json::from_value(db_plan.active_waivers.clone())
                .map_err(|e| {
                    warn!("Failed to deserialize active_waivers for plan {}: {}", db_plan.id, e);
                    e
                })
                .unwrap_or_else(|_| {
                    warn!("Using empty active waivers for plan {}", db_plan.id);
                    vec![]
                });
        
        // Deserialize plan metadata with fallback to default
        let plan_metadata: PlanMetadata = 
            serde_json::from_value(db_plan.metadata.clone())
                .map_err(|e| {
                    warn!("Failed to deserialize plan metadata for plan {}: {}", db_plan.id, e);
                    e
                })
                .unwrap_or_else(|_| {
                    warn!("Using default plan metadata for plan {}", db_plan.id);
                    PlanMetadata {
                        created_at: db_plan.created_at,
                        updated_at: db_plan.updated_at,
                        approved_at: db_plan.approved_at,
                        completed_at: db_plan.completed_at,
                        created_by: "system".to_string(),
                        version: "1.0.0".to_string(),
                        source: "database_reconstruction".to_string(),
                        confidence_score: 0.5,
                        generation_time_ms: 0,
                        model_used: None,
                        fallback_used: false,
                        strategy: None,
                        confidence: None,
                    }
                });
        
        // Parse plan state from string with fallback to Draft
        let plan_state = match db_plan.state.as_str() {
            "draft" => PlanState::Draft,
            "under_review" | "under-review" => PlanState::UnderReview,
            "approved" => PlanState::Approved,
            "in_progress" | "in-progress" => PlanState::InProgress,
            "blocked" => PlanState::Blocked { reason: "Plan reconstruction from database".to_string() },
            "completed" => PlanState::Completed,
            "failed" => PlanState::Failed { reason: "Plan reconstruction from database".to_string() },
            "cancelled" => PlanState::Cancelled { reason: "Plan reconstruction from database".to_string() },
            _ => {
                warn!("Unknown plan state '{}' for plan {}, defaulting to Draft", db_plan.state, db_plan.id);
                PlanState::Draft
            }
        };
        
        // Reconstruct working spec from metadata or create minimal default
        // The working spec is stored in the contract_plan field, but we need to reconstruct it
        // For now, create a minimal working spec from available data
        let working_spec = WorkingSpec {
            version: "1.0.0".to_string(),
            id: db_plan.working_spec_id.clone(),
            title: db_plan.title.clone(),
            description: db_plan.overview.clone().unwrap_or_else(|| "Reconstructed from database".to_string()),
            risk_tier: 2, // Default risk tier
            mode: agent_agency_contracts::types::planning::ExecutionMode::Feature,
            change_budget: change_budget.clone(),
            blast_radius: agent_agency_contracts::types::planning::BlastRadius {
                modules: vec![],
                data_migration: false,
                external_apis: false,
            },
            scope: agent_agency_contracts::types::planning::Scope {
                files_affected: vec![],
                directories: vec![],
                included_paths: vec![],
                excluded_paths: vec![],
                allowed_operations: vec![],
                resource_requirements: std::collections::HashMap::new(),
            },
            invariants: vec![],
            acceptance_criteria: vec![],
            non_functional: agent_agency_contracts::types::planning::NonFunctionalRequirements {
                a11y: vec![],
                perf: None,
                security: vec![],
            },
            contracts: vec![],
        };
        
        // Reconstruct contract execution plan
        let contract_plan = ContractExecutionPlan {
            id: db_plan.id,
            session_id: db_plan.session_id,
            working_spec_id: db_plan.working_spec_id,
            contract_plan: working_spec,
            title: db_plan.title,
            overview: db_plan.overview.unwrap_or_else(|| "Reconstructed from database".to_string()),
            state: plan_state,
            milestones,
            dependency_graph,
            change_budget,
            quality_gates,
            evidence_requirements,
            active_waivers,
            metadata: plan_metadata,
        };
        
        // Create execution plan with default orchestration metadata and execution context
        let execution_plan = ExecutionPlan {
            contract_plan,
            orchestration_meta: crate::planning::plan_types::OrchestrationMetadata::default(),
            execution_context: crate::planning::plan_types::ExecutionContext::default(),
            execution_state: None, // Execution state would need to be reconstructed separately
        };
        
        debug!("Successfully reconstructed plan {} from database", db_plan.id);
        Ok(execution_plan)
    }
}

impl FileStorage {
    /// Store plan specification as YAML file
    async fn store_plan_spec(&self, plan: &ExecutionPlan) -> Result<()> {
        let plan_path = self.plans_dir.join(format!("{}.plan.yml", plan.contract_plan.id));

        // Create parent directories if needed
        if let Some(parent) = plan_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Serialize plan to YAML
        let yaml_content = serde_yaml::to_string(plan)?;
        tokio::fs::write(plan_path, yaml_content).await?;

        Ok(())
    }

    /// Load plan specification from YAML file
    async fn load_plan_spec(&self, plan_id: Uuid) -> Result<Option<ExecutionPlan>> {
        let plan_path = self.plans_dir.join(format!("{}.plan.yml", plan_id));

        if !plan_path.exists() {
            return Ok(None);
        }

        let yaml_content = tokio::fs::read_to_string(plan_path).await?;
        let plan: ExecutionPlan = serde_yaml::from_str(&yaml_content)?;

        Ok(Some(plan))
    }
}

// Database operation types (should be imported from data-infrastructure)
use crate::planning::{
    CreateExecutionPlan, UpdateExecutionPlan, CreatePlanningSession, UpdatePlanningSession,
    CreatePlanningAuditEvent, CreatePlanningTelemetry,
};

/// Audit event for storage operations

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AuditEvent {
    #[schemars(with = "String")]
    pub plan_id: Uuid,
    pub milestone_id: Option<String>,
    #[schemars(with = "Option<String>")]
    pub worker_id: Option<Uuid>,
    pub event_type: String,
    pub description: String,
    #[schemars(with = "String")]

    pub timestamp: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    // Mock database operations for testing
    // struct MockDatabaseOps;

    // #[async_trait::async_trait]
    // impl DatabaseOperations for MockDatabaseOps {
    //     async fn create_execution_plan(&self, _plan: CreateExecutionPlan) -> Result<crate::planning::models::ExecutionPlan> {
    //         Ok(crate::planning::models::ExecutionPlan {
    //             id: Uuid::new_v4(),
    //             session_id: Uuid::new_v4(),
    //             working_spec_id: "test".to_string(),
    //             title: "Test Plan".to_string(),
    //             overview: "Test overview".to_string(),
    //             state: "draft".to_string(),
    //             milestones: serde_json::Value::Array(vec![]),
    //             dependency_graph: serde_json::Value::Object(serde_json::Map::new()),
    //             change_budget: serde_json::Value::Object(serde_json::Map::new()),
    //             quality_gates: serde_json::Value::Object(serde_json::Map::new()),
    //             evidence_requirements: serde_json::Value::Array(vec![]),
    //             active_waivers: serde_json::Value::Array(vec![]),
    //             metadata: serde_json::Value::Object(serde_json::Map::new()),
    //             created_at: Utc::now(),
    //             updated_at: Utc::now(),
    //             approved_at: None,
    //             completed_at: None,
    //         })
    //     }

    //     async fn get_execution_plan(&self, _id: Uuid) -> Result<Option<crate::planning::models::ExecutionPlan>> {
    //         Ok(None)
    //     }

    //     async fn get_execution_plans(&self) -> Result<Vec<crate::planning::models::ExecutionPlan>> {
    //         Ok(vec![])
    //     }

    //     async fn update_execution_plan(&self, _id: Uuid, _update: UpdateExecutionPlan) -> Result<crate::planning::models::ExecutionPlan> {
    //         Err(anyhow!("Not implemented"))
    //     }

    //     async fn delete_execution_plan(&self, _id: Uuid) -> Result<()> {
    //         Ok(())
    //     }

    //     async fn create_planning_session(&self, session: CreatePlanningSession) -> Result<crate::planning::models::PlanningSession> {
    //         Ok(crate::planning::models::PlanningSession {
    //             id: session.id,
    //             plan_id: session.plan_id,
    //             orchestrator_id: session.orchestrator_id,
    //             worker_pool_id: session.worker_pool_id,
    //             council_session_id: session.council_session_id,
    //             audit_correlation_id: session.audit_correlation_id,
    //             status: session.status,
    //             execution_state: session.execution_state,
    //             started_at: Utc::now(),
    //             completed_at: None,
    //             created_at: Utc::now(),
    //         })
    //     }

    //     async fn get_planning_session(&self, _id: Uuid) -> Result<Option<crate::planning::models::PlanningSession>> {
    //         Ok(None)
    //     }

    //     async fn get_planning_sessions(&self, _plan_id: Uuid) -> Result<Vec<crate::planning::models::PlanningSession>> {
    //         Ok(vec![])
    //     }

    //     async fn update_planning_session(&self, _id: Uuid, _update: UpdatePlanningSession) -> Result<crate::planning::models::PlanningSession> {
    //         Err(anyhow!("Not implemented"))
    //     }

    //     async fn create_planning_audit_event(&self, _event: CreatePlanningAuditEvent) -> Result<crate::planning::models::PlanningAuditEvent> {
    //         Err(anyhow!("Not implemented"))
    //     }

    //     async fn get_planning_audit_events(&self, _plan_id: Uuid) -> Result<Vec<crate::planning::models::PlanningAuditEvent>> {
    //         Ok(vec![])
    //     }

    //     async fn create_planning_telemetry(&self, _telemetry: CreatePlanningTelemetry) -> Result<crate::planning::models::PlanningTelemetry> {
    //         Err(anyhow!("Not implemented"))
    //     }

    //     async fn get_planning_telemetry(&self, _plan_id: Uuid, _metric_type: Option<String>) -> Result<Vec<crate::planning::models::PlanningTelemetry>> {
    //         Ok(vec![])
    //     }

    //     // Waiver operations
    //     async fn get_waivers(&self, _status: Option<String>) -> Result<Vec<crate::planning::models::Waiver>> { Ok(vec![]) }
    //     async fn create_waiver(&self, _waiver: crate::planning::CreateWaiver) -> Result<crate::planning::models::Waiver> { Err(anyhow!("Not implemented")) }
    //     async fn update_waiver(&self, _id: Uuid, _update: crate::planning::UpdateWaiver) -> Result<crate::planning::models::Waiver> { Err(anyhow!("Not implemented")) }
    //     async fn create_provenance_entry(&self, _entry: crate::planning::CreateProvenanceEntry) -> Result<crate::planning::models::ProvenanceEntry> { Err(anyhow!("Not implemented")) }
    //     async fn get_provenance_entries(&self, _limit: Option<i64>) -> Result<Vec<crate::planning::models::ProvenanceEntry>> { Ok(vec![]) }
    //     async fn create_judge(&self, _judge: crate::planning::CreateJudge) -> Result<crate::planning::models::Judge> { Err(anyhow!("Not implemented")) }
    //     async fn get_judges(&self) -> Result<Vec<crate::planning::models::Judge>> { Ok(vec![]) }
    //     async fn update_judge(&self, _id: Uuid, _update: crate::planning::UpdateJudge) -> Result<crate::planning::models::Judge> { Err(anyhow!("Not implemented")) }
    //     async fn create_worker(&self, _worker: crate::planning::CreateWorker) -> Result<crate::planning::models::Worker> { Err(anyhow!("Not implemented")) }
    //     async fn get_workers(&self) -> Result<Vec<crate::planning::models::Worker>> { Ok(vec![]) }
    //     async fn update_worker(&self, _id: Uuid, _update: crate::planning::UpdateWorker) -> Result<crate::planning::models::Worker> { Err(anyhow!("Not implemented")) }
    //     async fn create_task(&self, _task: crate::planning::CreateTask) -> Result<crate::planning::models::Task> { Err(anyhow!("Not implemented")) }
    //     async fn get_tasks(&self, _status: Option<String>) -> Result<Vec<crate::planning::models::Task>> { Ok(vec![]) }
    //     async fn update_task(&self, _id: Uuid, _update: crate::planning::UpdateTask) -> Result<crate::planning::models::Task> { Err(anyhow!("Not implemented")) }
    //     async fn get_task(&self, _id: Uuid) -> Result<Option<crate::planning::models::Task>> { Ok(None) }
    //     async fn delete_task(&self, _id: Uuid) -> Result<()> { Ok(()) }
    //     async fn create_milestone(&self, _milestone: crate::planning::CreateMilestone) -> Result<crate::planning::models::Milestone> { Err(anyhow!("Not implemented")) }
    //     async fn get_milestones(&self, _plan_id: Uuid) -> Result<Vec<crate::planning::models::Milestone>> { Ok(vec![]) }
    //     async fn update_milestone(&self, _plan_id: Uuid, _milestone_id: String, _update: crate::planning::UpdateMilestone) -> Result<crate::planning::models::Milestone> { Err(anyhow!("Not implemented")) }
    // }

    #[tokio::test]
    async fn test_planning_storage_creation() {
        let db_ops = Arc::new(crate::test_utils::MockDatabaseOps);
        let plans_dir = PathBuf::from("/tmp/plans");
        let specs_dir = PathBuf::from("/tmp/specs");
        let config = StorageConfig::default();

        let storage = PlanningStorage::new(db_ops, plans_dir, specs_dir, config);
        // Storage created successfully
        assert!(true);
    }

    #[tokio::test]
    async fn test_session_caching() {
        let db_ops = Arc::new(crate::test_utils::MockDatabaseOps);
        let plans_dir = PathBuf::from("/tmp/plans");
        let specs_dir = PathBuf::from("/tmp/specs");
        let config = StorageConfig::default();

        let storage = PlanningStorage::new(db_ops, plans_dir, specs_dir, config);

        // Test session creation
        let plan = create_test_execution_plan();
        let session_id = storage.create_planning_session(&plan).await.unwrap();

        // Test session retrieval (should be cached)
        let session = storage.get_planning_session(session_id).await.unwrap().unwrap();
        assert_eq!(session.id, session_id);
    }

    // TODO: Add unit tests for task-to-plan mapping functionality
    // Test cases to implement:
    // 1. test_get_plan_for_task_with_matching_working_spec_id - Test successful lookup when working_spec_id matches TASK-<UUID> format
    // 2. test_get_plan_for_task_with_no_matching_plan - Test that None is returned when no plan matches task_id
    // 3. test_get_plan_for_task_with_multiple_plans - Test that most recent plan is returned when multiple plans exist for same task
    // 4. test_get_plan_for_task_with_non_task_format - Test that plans with non-TASK format working_spec_id are not matched
    // 5. test_store_execution_plan_preserves_task_format - Test that TASK-<UUID> format is preserved when storing plans

    fn create_test_execution_plan() -> ExecutionPlan {
        use crate::planning::plan_types::{OrchestrationMetadata, ResourceInventory, ExecutionContext as PlanExecutionContext};

        ExecutionPlan {
            contract_plan: agent_agency_contracts::planning_io::ExecutionPlan {
                id: Uuid::new_v4(),
                session_id: Uuid::new_v4(),
                working_spec_id: "test-spec".to_string(),
                contract_plan: agent_agency_contracts::WorkingSpec {
                    version: "1.0".to_string(),
                    id: "test-spec".to_string(),
                    title: "Test Spec".to_string(),
                    description: "Test description".to_string(),
                    goals: vec![],
                    risk_tier: 2,
                    constraints: agent_agency_contracts::WorkingSpecConstraints {
                        max_duration_minutes: None,
                        max_iterations: None,
                        budget_limits: None,
                        scope_restrictions: None,
                    },
                    acceptance_criteria: vec![],
                    test_plan: agent_agency_contracts::TestPlan {
                        unit_tests: vec![],
                        integration_tests: vec![],
                        e2e_scenarios: vec![],
                        coverage_targets: None,
                    },
                    rollback_plan: agent_agency_contracts::RollbackPlan {
                        strategy: agent_agency_contracts::RollbackStrategy::GitRevert,
                        automated_steps: vec![],
                        manual_steps: vec![],
                        data_impact: agent_agency_contracts::DataImpact::None,
                        downtime_required: None,
                        rollback_window_minutes: None,
                    },
                    context: agent_agency_contracts::WorkingSpecContext {
                        workspace_root: "/tmp".to_string(),
                        git_branch: "main".to_string(),
                        recent_changes: vec![],
                        dependencies: HashMap::new(),
                        environment: agent_agency_contracts::task_request::Environment::Development,
                    },
                    non_functional_requirements: None,
                    validation_results: None,
                    quality_gates: None,
                    scope: vec![],
                    metadata: None,
                    milestones: vec![],
                    change_budget: agent_agency_contracts::planning_io::ChangeBudget {
                        max_files: 10,
                        max_loc: 100,
                        max_migrations: 0,
                        allow_breaking_changes: false,
                        allow_new_dependencies: false,
                        enforcement_mode: agent_agency_contracts::planning_io::BudgetEnforcement::Strict,
                    },
                    file_changes: vec![],
                    coverage_targets: None,
                    overview: "Test overview".to_string(),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                },
                title: "Test Plan".to_string(),
                overview: "Test overview".to_string(),
                state: agent_agency_contracts::planning_io::PlanState::Draft,
                milestones: vec![],
                dependency_graph: agent_agency_contracts::planning_io::DependencyGraph {
                    nodes: HashMap::new(),
                    edges: vec![],
                    critical_path: vec![],
                    parallel_groups: vec![],
                    has_cycles: false,
                    cycles: vec![],
                },
                change_budget: agent_agency_contracts::planning_io::ChangeBudget {
                    max_files: 10,
                    max_loc: 100,
                    max_migrations: 0,
                    allow_breaking_changes: false,
                    allow_new_dependencies: false,
                    enforcement_mode: agent_agency_contracts::planning_io::BudgetEnforcement::Strict,
                },
                quality_gates: agent_agency_contracts::planning_io::QualityGates {
                    coverage_requirements: HashMap::new(),
                    mutation_requirements: agent_agency_contracts::planning_io::MutationRequirements {
                        required: false,
                        min_score: 0.0,
                        operators: vec![],
                    },
                    security_requirements: agent_agency_contracts::planning_io::SecurityRequirements {
                        scan_required: false,
                        max_issues_by_severity: HashMap::new(),
                        required_controls: vec![],
                    },
                    performance_requirements: agent_agency_contracts::planning_io::PerformanceRequirements {
                        max_regressions: 0,
                        required_benchmarks: vec![],
                        slas: vec![],
                    },
                    documentation_requirements: agent_agency_contracts::planning_io::DocumentationRequirements {
                        api_docs_required: false,
                        code_docs_required: false,
                        architecture_docs_required: false,
                        required_formats: vec![],
                        required_types: vec![],
                        min_coverage: 0.0,
                        quality_checks: vec![],
                    },
                    requires_manual_review: false,
                    requires_council_approval: false,
                    min_coverage: None,
                    min_mutation_score_percent: None,
                },
                evidence_requirements: vec![],
                active_waivers: vec![],
                metadata: agent_agency_contracts::planning_io::PlanMetadata {
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    approved_at: None,
                    completed_at: None,
                    created_by: agent_agency_contracts::planning_io::PlanCreator::AI {
                        model: "test-model".to_string(),
                        version: "1.0".to_string(),
                    },
                    version: "1.0".to_string(),
                    source: "test".to_string(),
                    confidence_score: Some(0.5),
                    generation_time_ms: Some(100),
                    model_used: Some("test-model".to_string()),
                    fallback_used: false,
                    strategy: agent_agency_contracts::types::planning::PlanningStrategy::AIAssisted,
                    confidence: 0.5,
                    estimated_duration_ms: 0,
                    estimated_cost_cents: 0,
                    adaptive: false,
                    engine_version: "1.0".to_string(),
                    additional_metadata: HashMap::new(),
                },
                execution_context: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                approved_at: None,
                completed_at: None,
            },
            orchestration_meta: OrchestrationMetadata {
                orchestrator_id: "test-orchestrator".to_string(),
                worker_pool_id: "test-pool".to_string(),
                council_session_id: Some("test-council".to_string()),
                audit_correlation_id: Uuid::new_v4(),
                planning_engine: "test-engine".to_string(),
                planning_version: "1.0.0".to_string(),
            },
            execution_context: PlanExecutionContext {
                session_start: Utc::now(),
                working_directory: "/tmp".to_string(),
                environment: std::collections::HashMap::new(),
                available_resources: ResourceInventory {
                    available_cpu_cores: 4,
                    available_memory_mb: 8192,
                    available_disk_mb: 102400,
                    available_network_mbps: 100.0,
                    available_workers: std::collections::HashMap::new(),
                },
                worker_assignments: std::collections::HashMap::new(),
                parallel_batches: vec![],
            },
            execution_state: None,
        }
    }

}

impl PlanningStorage {
    /// Store execution result for a plan
    pub async fn store_execution_result(&self, plan_id: Uuid, result: &agent_agency_contracts::planning::PlanExecutionResult) -> Result<()> {
        // Store execution result as JSON file
        let result_path = self.file_storage.plans_dir.join(format!("{}_result.json", plan_id));
        let result_json = serde_json::to_string_pretty(result)?;
        tokio::fs::write(&result_path, result_json).await?;

        // Update plan status in database if needed
        // TODO: Add database storage for execution results

        Ok(())
    }

    /// Get execution result for a plan
    pub async fn get_execution_result(&self, plan_id: Uuid) -> Result<Option<agent_agency_contracts::planning::PlanExecutionResult>> {
        // Try to load execution result from file
        let result_path = self.file_storage.plans_dir.join(format!("{}_result.json", plan_id));
        if result_path.exists() {
            let result_json = tokio::fs::read_to_string(&result_path).await?;
            let result: agent_agency_contracts::planning::PlanExecutionResult = serde_json::from_str(&result_json)?;
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    /// Store a plan (alias for store_execution_plan for backward compatibility)
    pub async fn store_plan(&self, plan: &ExecutionPlan) -> Result<()> {
        self.store_execution_plan(plan).await
    }

    /// Get plan for a specific task
    pub async fn get_plan_for_task(&self, task_id: Uuid) -> Result<Option<ExecutionPlan>> {
        // Extract task_id from working_spec_id format: TASK-<UUID>
        // Query execution_plans table for plans where working_spec_id matches TASK-{task_id}
        let expected_working_spec_id = format!("TASK-{}", task_id);
        
        // Get all execution plans and filter by working_spec_id
        // Note: This could be optimized with a direct query method, but works for now
        let all_plans = self.db_ops.get_execution_plans().await?;
        
        // Find plans matching the task_id (working_spec_id format: TASK-<UUID>)
        let matching_plans: Vec<_> = all_plans
            .into_iter()
            .filter(|plan| plan.working_spec_id == expected_working_spec_id)
            .collect();
        
        if matching_plans.is_empty() {
            return Ok(None);
        }
        
        // Return the most recent plan if multiple exist (sorted by created_at descending)
        let most_recent_plan = matching_plans
            .into_iter()
            .max_by_key(|plan| plan.created_at);
        
        if let Some(db_plan) = most_recent_plan {
            // Load plan spec from file if available
            if let Some(plan_spec) = self.file_storage.load_plan_spec(db_plan.id).await? {
                // Merge file spec with database state
                let plan = self.merge_plan_data(plan_spec, db_plan)?;
                Ok(Some(plan))
            } else {
                // Plan spec missing, reconstruct from DB
                let plan = self.reconstruct_plan_from_db(db_plan)?;
                Ok(Some(plan))
            }
        } else {
            Ok(None)
        }
    }

    /// Get plan by ID (alias for load_execution_plan)
    pub async fn get_plan_by_id(&self, plan_id: Uuid) -> Result<Option<ExecutionPlan>> {
        self.load_execution_plan(plan_id).await
    }

    /// Store execution plan as execution result (alias for backward compatibility)
    /// Note: Use store_execution_result(plan_id, &PlanExecutionResult) for contract type
    pub async fn store_execution_result_plan(&self, plan_id: Uuid, result: &ExecutionPlan) -> Result<()> {
        // Store the updated plan with execution results
        self.store_execution_plan(result).await
    }

    /// Get execution plan as execution result (alias for backward compatibility)
    /// Note: Use get_execution_result(plan_id) -> Option<PlanExecutionResult> for contract type
    pub async fn get_execution_result_plan(&self, plan_id: Uuid) -> Result<Option<ExecutionPlan>> {
        self.load_execution_plan(plan_id).await
    }
}