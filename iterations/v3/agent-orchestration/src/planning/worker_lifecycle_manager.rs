//! Worker Lifecycle Manager
//!
//! Manages worker lifecycle events including completion callbacks,
//! worker assignment tracking, and completion-to-council flow coordination.
//!
//! @author @darianrosebrook

use std::sync::Arc;
use std::collections::HashMap;
use anyhow::Result;
use uuid::Uuid;
use tokio::sync::RwLock;
use tracing::{info, warn};

use agent_agency_contracts::execution_artifacts::ExecutionArtifacts;
use agent_agency_contracts::planning_io::Milestone;

use crate::planning::council_integration::CouncilIntegration;

/// Worker lifecycle event
#[derive(Debug, Clone)]
pub enum WorkerLifecycleEvent {
    /// Worker assigned to milestone
    Assigned {
        worker_id: Uuid,
        milestone_id: String,
        assigned_at: chrono::DateTime<chrono::Utc>,
    },
    
    /// Worker started execution
    Started {
        worker_id: Uuid,
        milestone_id: String,
        started_at: chrono::DateTime<chrono::Utc>,
    },
    
    /// Worker completed execution
    Completed {
        worker_id: Uuid,
        milestone_id: String,
        artifacts: ExecutionArtifacts,
        completed_at: chrono::DateTime<chrono::Utc>,
    },
    
    /// Worker failed execution
    Failed {
        worker_id: Uuid,
        milestone_id: String,
        error: String,
        failed_at: chrono::DateTime<chrono::Utc>,
    },
}

/// Worker assignment tracking
#[derive(Debug, Clone)]
struct WorkerAssignment {
    worker_id: Uuid,
    milestone_id: String,
    assigned_at: chrono::DateTime<chrono::Utc>,
    status: AssignmentStatus,
    artifacts: Option<ExecutionArtifacts>,
}

/// Assignment status
#[derive(Debug, Clone, PartialEq, Eq)]
enum AssignmentStatus {
    Assigned,
    InProgress,
    Completed,
    Failed,
}

/// Worker lifecycle manager
pub struct WorkerLifecycleManager {
    /// Active worker assignments
    assignments: Arc<RwLock<HashMap<Uuid, WorkerAssignment>>>,
    
    /// Council integration for presenting completed work
    council_integration: Arc<dyn CouncilIntegration>,
}

impl WorkerLifecycleManager {
    /// Create new worker lifecycle manager
    pub fn new(council_integration: Arc<dyn CouncilIntegration>) -> Self {
        Self {
            assignments: Arc::new(RwLock::new(HashMap::new())),
            council_integration,
        }
    }

    /// Handle worker assignment
    pub async fn handle_assignment(
        &self,
        worker_id: Uuid,
        milestone: &Milestone,
    ) -> Result<()> {
        info!("Worker {} assigned to milestone {}", worker_id, milestone.id);

        let assignment = WorkerAssignment {
            worker_id,
            milestone_id: milestone.id.clone(),
            assigned_at: chrono::Utc::now(),
            status: AssignmentStatus::Assigned,
            artifacts: None,
        };

        let mut assignments = self.assignments.write().await;
        assignments.insert(worker_id, assignment);

        Ok(())
    }

    /// Handle worker completion
    pub async fn handle_completion(
        &self,
        worker_id: Uuid,
        artifacts: ExecutionArtifacts,
    ) -> Result<()> {
        info!("Worker {} completed execution", worker_id);

        // Update assignment status
        let milestone_id = {
            let mut assignments = self.assignments.write().await;
            if let Some(assignment) = assignments.get_mut(&worker_id) {
                assignment.status = AssignmentStatus::Completed;
                assignment.artifacts = Some(artifacts.clone());
                assignment.milestone_id.clone()
            } else {
                warn!("No assignment found for worker {}", worker_id);
                return Err(anyhow::anyhow!("No assignment found for worker {}", worker_id));
            }
        };

        // Present completed work to council (CAWS Pleading stage)
        self.council_integration.present_work(
            &[artifacts],
            &milestone_id,
            worker_id,
        ).await?;

        Ok(())
    }

    /// Handle worker failure
    pub async fn handle_failure(
        &self,
        worker_id: Uuid,
        error: String,
    ) -> Result<()> {
        warn!("Worker {} failed execution: {}", worker_id, error);

        let mut assignments = self.assignments.write().await;
        if let Some(assignment) = assignments.get_mut(&worker_id) {
            assignment.status = AssignmentStatus::Failed;
        }

        Ok(())
    }

    /// Get assignment for worker
    pub async fn get_assignment(&self, worker_id: Uuid) -> Option<WorkerAssignment> {
        let assignments = self.assignments.read().await;
        assignments.get(&worker_id).cloned()
    }

    /// List all active assignments
    pub async fn list_assignments(&self) -> Vec<WorkerAssignment> {
        let assignments = self.assignments.read().await;
        assignments.values().cloned().collect()
    }

    /// Cleanup completed assignments
    pub async fn cleanup_completed(&self) -> Result<()> {
        let mut assignments = self.assignments.write().await;
        assignments.retain(|_, assignment| {
            !matches!(assignment.status, AssignmentStatus::Completed | AssignmentStatus::Failed)
        });
        Ok(())
    }
}



