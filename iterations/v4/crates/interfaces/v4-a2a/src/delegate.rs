//! Delegate Tool
//!
//! Delegates tasks to remote A2A workers via the A2A protocol.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use sha2::{Digest, Sha256};

use crate::client::A2AClient;
#[cfg(test)]
use crate::types::Artifact;
use crate::types::{Part, Task as A2ATask};
use v4_tools::tool::{Tool, ToolCapability, ToolCategory, ToolError, ToolMetadata};
use v4_types::operators::{ControlOp, OperatorResult};
use v4_types::OperatorType;

/// Tool that delegates tasks to remote A2A workers
pub struct DelegateTool {
    clients: Arc<HashMap<String, A2AClient>>,
}

impl DelegateTool {
    /// Create a new DelegateTool with a map of agent_id -> A2AClient
    pub fn new(clients: Arc<HashMap<String, A2AClient>>) -> Self {
        Self { clients }
    }

    /// Create from a plain HashMap (wraps in Arc)
    pub fn from_map(clients: HashMap<String, A2AClient>) -> Self {
        Self {
            clients: Arc::new(clients),
        }
    }

    /// Extract text content from A2A task artifacts
    pub fn extract_text(task: &A2ATask) -> String {
        let mut texts = Vec::new();

        if let Some(artifacts) = &task.artifacts {
            for artifact in artifacts {
                for part in &artifact.parts {
                    if let Part::Text { text, .. } = part {
                        texts.push(text.as_str());
                    }
                }
            }
        }

        // Also check history for agent responses
        if texts.is_empty() {
            if let Some(history) = &task.history {
                for msg in history {
                    if msg.role == crate::types::Role::Agent {
                        for part in &msg.parts {
                            if let Part::Text { text, .. } = part {
                                texts.push(text.as_str());
                            }
                        }
                    }
                }
            }
        }

        texts.join("\n")
    }

    /// List available agent IDs
    pub fn available_agents(&self) -> Vec<&str> {
        self.clients.keys().map(|s| s.as_str()).collect()
    }
}

#[async_trait]
impl Tool for DelegateTool {
    fn id(&self) -> &str {
        "builtin:delegate"
    }

    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            id: self.id().to_string(),
            name: "Delegate".to_string(),
            description: "Delegate a task to a remote A2A worker agent".to_string(),
            version: "1.0.0".to_string(),
            category: ToolCategory::Control,
            capabilities: vec![ToolCapability::ExecuteCode, ToolCapability::NetworkAccess],
            supported_operators: vec!["C".to_string()],
            requires_sandbox: false,
        }
    }

    async fn execute(&self, operator: &OperatorType) -> Result<OperatorResult, ToolError> {
        let start = std::time::Instant::now();

        match operator {
            OperatorType::Control(ControlOp::Delegate { agent_id, task }) => {
                let span = tracing::info_span!(
                    "a2a_delegation",
                    agent_id = agent_id.as_str(),
                    task_preview = &task[..task.len().min(80)],
                );
                let _guard = span.enter();

                let client = self.clients.get(agent_id).ok_or_else(|| {
                    ToolError::NotFound(format!("No A2A client registered for agent '{}'", agent_id))
                })?;

                let a2a_task = client.send_message(task.as_str()).await.map_err(|e| {
                    tracing::warn!(agent_id = agent_id.as_str(), error = %e, "A2A delegation failed");
                    ToolError::ExecutionFailed(format!("A2A delegation failed: {}", e))
                })?;

                let response_text = Self::extract_text(&a2a_task);
                let artifact_count = a2a_task
                    .artifacts
                    .as_ref()
                    .map(|a| a.len())
                    .unwrap_or(0);

                let elapsed_ms = start.elapsed().as_millis() as u64;
                tracing::info!(
                    agent_id = agent_id.as_str(),
                    duration_ms = elapsed_ms,
                    artifact_count = artifact_count,
                    "A2A delegation complete"
                );

                let mut hasher = Sha256::new();
                hasher.update(response_text.as_bytes());
                let hash = hex::encode(hasher.finalize());

                Ok(OperatorResult {
                    operator: operator.clone(),
                    success: true,
                    data: Some(serde_json::json!({
                        "agent_id": agent_id,
                        "response_text": response_text,
                        "artifact_count": artifact_count,
                        "task_state": format!("{:?}", a2a_task.status.state),
                    })),
                    error: None,
                    execution_time_ms: elapsed_ms,
                    content_hash: Some(hash),
                })
            }
            _ => Err(ToolError::InvalidOperator(
                "DelegateTool only handles Delegate operations".to_string(),
            )),
        }
    }

    fn can_execute(&self, operator: &OperatorType) -> bool {
        matches!(operator, OperatorType::Control(ControlOp::Delegate { .. }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delegate_tool_metadata() {
        let tool = DelegateTool::from_map(HashMap::new());
        let meta = tool.metadata();

        assert_eq!(meta.id, "builtin:delegate");
        assert_eq!(meta.category, ToolCategory::Control);
        assert!(meta.capabilities.contains(&ToolCapability::ExecuteCode));
        assert!(meta.capabilities.contains(&ToolCapability::NetworkAccess));
        assert!(!meta.requires_sandbox);
    }

    #[test]
    fn test_delegate_can_execute() {
        let tool = DelegateTool::from_map(HashMap::new());

        let delegate_op = OperatorType::Control(ControlOp::Delegate {
            agent_id: "minimax".to_string(),
            task: "write a haiku".to_string(),
        });
        assert!(tool.can_execute(&delegate_op));

        let terminate_op = OperatorType::Control(ControlOp::Terminate {
            reason: "done".to_string(),
        });
        assert!(!tool.can_execute(&terminate_op));
    }

    #[test]
    fn test_delegate_available_agents() {
        let clients = HashMap::new();
        // Can't create real A2AClient without network, so just test empty
        assert!(DelegateTool::from_map(clients).available_agents().is_empty());
    }

    #[test]
    fn test_extract_text_from_artifacts() {
        use crate::types::{TaskState, TaskStatus};

        let task = A2ATask {
            id: "test".to_string(),
            context_id: None,
            status: TaskStatus {
                state: TaskState::Completed,
                message: None,
                timestamp: None,
            },
            kind: "task".to_string(),
            history: None,
            artifacts: Some(vec![Artifact {
                artifact_id: "a1".to_string(),
                parts: vec![
                    Part::Text {
                        text: "Hello".to_string(),
                        metadata: None,
                    },
                    Part::Text {
                        text: "World".to_string(),
                        metadata: None,
                    },
                ],
                name: None,
                description: None,
                metadata: None,
            }]),
            metadata: None,
        };

        let text = DelegateTool::extract_text(&task);
        assert_eq!(text, "Hello\nWorld");
    }

    #[test]
    fn test_extract_text_empty() {
        use crate::types::{TaskState, TaskStatus};

        let task = A2ATask {
            id: "test".to_string(),
            context_id: None,
            status: TaskStatus {
                state: TaskState::Completed,
                message: None,
                timestamp: None,
            },
            kind: "task".to_string(),
            history: None,
            artifacts: None,
            metadata: None,
        };

        let text = DelegateTool::extract_text(&task);
        assert!(text.is_empty());
    }
}
