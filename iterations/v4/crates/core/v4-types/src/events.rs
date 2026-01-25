//! Agent Events (Event Sourcing)
//!
//! All state changes in V4 are represented as events.
//! Events are append-only and content-hashed for audit trail.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::council::{CouncilVerdict, JudgeScores};
use crate::operators::OperatorType;
use crate::verdict::GateType;
use crate::worker::WorkerStatus;

/// All events that can occur in the agent system
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum AgentEvent {
    // === Task Lifecycle ===
    /// A new task was created
    TaskCreated {
        task_id: String,
        title: String,
        #[schemars(with = "String")]
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// A task was decomposed into subtasks
    TaskDecomposed {
        task_id: String,
        subtask_ids: Vec<String>,
        #[schemars(with = "String")]
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// A task started execution
    TaskStarted {
        task_id: String,
        worker_id: String,
        #[schemars(with = "String")]
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// A task completed successfully
    TaskCompleted {
        task_id: String,
        result_hash: String,
        #[schemars(with = "String")]
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// A task failed
    TaskFailed {
        task_id: String,
        error: String,
        #[schemars(with = "String")]
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    // === Governance ===
    /// An invariant was checked
    InvariantChecked {
        invariant_id: String,
        passed: bool,
        violations: Vec<String>,
        #[schemars(with = "String")]
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// The council was convened for a decision
    CouncilConvened {
        task_id: String,
        session_id: String,
        judges: Vec<String>,
        #[schemars(with = "String")]
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// The council reached a verdict
    CouncilVerdict {
        task_id: String,
        session_id: String,
        verdict: CouncilVerdict,
        scores: JudgeScores,
        #[schemars(with = "String")]
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// A gate was checked
    GatePassed {
        gate: GateType,
        score: f64,
        threshold: f64,
        #[schemars(with = "String")]
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// A gate check failed
    GateFailed {
        gate: GateType,
        score: f64,
        threshold: f64,
        #[schemars(with = "String")]
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    // === Memory ===
    /// Information was stored in memory
    MemoryStored {
        key: String,
        content_hash: String,
        #[schemars(with = "String")]
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// Information was recalled from memory
    MemoryRecalled {
        key: String,
        provenance: Vec<String>,
        hop_count: u32,
        #[schemars(with = "String")]
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// Memory item decayed
    MemoryDecayed {
        key: String,
        old_weight: f64,
        new_weight: f64,
        #[schemars(with = "String")]
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    // === Execution ===
    /// An operator was invoked
    OperatorInvoked {
        operator: OperatorType,
        args_hash: String,
        #[schemars(with = "String")]
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// An operator completed
    OperatorCompleted {
        operator_class: String,
        result_hash: String,
        execution_time_ms: u64,
        #[schemars(with = "String")]
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// A tool was invoked
    ToolInvoked {
        tool_name: String,
        args_hash: String,
        #[schemars(with = "String")]
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// A tool completed
    ToolCompleted {
        tool_name: String,
        result_hash: String,
        execution_time_ms: u64,
        #[schemars(with = "String")]
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// A worker was spawned
    WorkerSpawned {
        worker_id: String,
        task_id: String,
        worker_type: String,
        #[schemars(with = "String")]
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// A worker completed
    WorkerCompleted {
        worker_id: String,
        status: WorkerStatus,
        #[schemars(with = "String")]
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    // === Checkpoints ===
    /// A checkpoint was created
    CheckpointCreated {
        checkpoint_id: String,
        label: String,
        state_hash: String,
        #[schemars(with = "String")]
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    /// State was restored from a checkpoint
    CheckpointRestored {
        checkpoint_id: String,
        #[schemars(with = "String")]
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

impl AgentEvent {
    /// Get the timestamp of this event
    pub fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
        match self {
            Self::TaskCreated { timestamp, .. } => *timestamp,
            Self::TaskDecomposed { timestamp, .. } => *timestamp,
            Self::TaskStarted { timestamp, .. } => *timestamp,
            Self::TaskCompleted { timestamp, .. } => *timestamp,
            Self::TaskFailed { timestamp, .. } => *timestamp,
            Self::InvariantChecked { timestamp, .. } => *timestamp,
            Self::CouncilConvened { timestamp, .. } => *timestamp,
            Self::CouncilVerdict { timestamp, .. } => *timestamp,
            Self::GatePassed { timestamp, .. } => *timestamp,
            Self::GateFailed { timestamp, .. } => *timestamp,
            Self::MemoryStored { timestamp, .. } => *timestamp,
            Self::MemoryRecalled { timestamp, .. } => *timestamp,
            Self::MemoryDecayed { timestamp, .. } => *timestamp,
            Self::OperatorInvoked { timestamp, .. } => *timestamp,
            Self::OperatorCompleted { timestamp, .. } => *timestamp,
            Self::ToolInvoked { timestamp, .. } => *timestamp,
            Self::ToolCompleted { timestamp, .. } => *timestamp,
            Self::WorkerSpawned { timestamp, .. } => *timestamp,
            Self::WorkerCompleted { timestamp, .. } => *timestamp,
            Self::CheckpointCreated { timestamp, .. } => *timestamp,
            Self::CheckpointRestored { timestamp, .. } => *timestamp,
        }
    }

    /// Get the event category for grouping
    pub fn category(&self) -> &'static str {
        match self {
            Self::TaskCreated { .. }
            | Self::TaskDecomposed { .. }
            | Self::TaskStarted { .. }
            | Self::TaskCompleted { .. }
            | Self::TaskFailed { .. } => "task",

            Self::InvariantChecked { .. }
            | Self::CouncilConvened { .. }
            | Self::CouncilVerdict { .. }
            | Self::GatePassed { .. }
            | Self::GateFailed { .. } => "governance",

            Self::MemoryStored { .. }
            | Self::MemoryRecalled { .. }
            | Self::MemoryDecayed { .. } => "memory",

            Self::OperatorInvoked { .. }
            | Self::OperatorCompleted { .. }
            | Self::ToolInvoked { .. }
            | Self::ToolCompleted { .. }
            | Self::WorkerSpawned { .. }
            | Self::WorkerCompleted { .. } => "execution",

            Self::CheckpointCreated { .. } | Self::CheckpointRestored { .. } => "checkpoint",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_categories() {
        let task_event = AgentEvent::TaskCreated {
            task_id: "t1".to_string(),
            title: "Test".to_string(),
            timestamp: chrono::Utc::now(),
        };
        assert_eq!(task_event.category(), "task");

        let governance_event = AgentEvent::InvariantChecked {
            invariant_id: "inv1".to_string(),
            passed: true,
            violations: vec![],
            timestamp: chrono::Utc::now(),
        };
        assert_eq!(governance_event.category(), "governance");
    }

    #[test]
    fn test_event_serialization() {
        let event = AgentEvent::TaskCreated {
            task_id: "t1".to_string(),
            title: "Test Task".to_string(),
            timestamp: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let parsed: AgentEvent = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, AgentEvent::TaskCreated { .. }));
    }
}
