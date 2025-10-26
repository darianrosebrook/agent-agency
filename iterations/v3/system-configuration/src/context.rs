//! Common context abstractions for different execution contexts

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Common trait for all context objects
pub trait Context {
    fn id(&self) -> &str;
    fn timestamp(&self) -> DateTime<Utc>;
    fn metadata(&self) -> &HashMap<String, serde_json::Value>;
}

/// Common trait for contexts that have execution timeouts
pub trait TimeoutContext {
    fn timeout_ms(&self) -> u64;
    fn is_expired(&self) -> bool {
        let now = Utc::now();
        let start = self.start_time();
        let timeout_duration = chrono::Duration::milliseconds(self.timeout_ms() as i64);
        now.signed_duration_since(start) > timeout_duration
    }

    fn start_time(&self) -> DateTime<Utc>;
    fn time_remaining_ms(&self) -> i64 {
        let now = Utc::now();
        let start = self.start_time();
        let timeout_duration = chrono::Duration::milliseconds(self.timeout_ms() as i64);
        let elapsed = now.signed_duration_since(start);
        (timeout_duration - elapsed).num_milliseconds().max(0)
    }
}

/// Common trait for contexts that involve tasks
pub trait TaskContext {
    fn task_id(&self) -> &str;
    fn parameters(&self) -> &HashMap<String, serde_json::Value>;
}

/// Common trait for contexts that involve workers/agents
pub trait WorkerContext {
    fn worker_id(&self) -> &str;
    fn capabilities(&self) -> &[String];
}

/// Generic execution context that can be used across domains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonExecutionContext {
    pub id: String,
    pub task_id: String,
    pub worker_id: Option<String>,
    pub start_time: DateTime<Utc>,
    pub timeout_ms: u64,
    pub parameters: HashMap<String, serde_json::Value>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Context for CommonExecutionContext {
    fn id(&self) -> &str {
        &self.id
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.start_time
    }

    fn metadata(&self) -> &HashMap<String, serde_json::Value> {
        &self.metadata
    }
}

impl TimeoutContext for CommonExecutionContext {
    fn timeout_ms(&self) -> u64 {
        self.timeout_ms
    }

    fn start_time(&self) -> DateTime<Utc> {
        self.start_time
    }
}

impl TaskContext for CommonExecutionContext {
    fn task_id(&self) -> &str {
        &self.task_id
    }

    fn parameters(&self) -> &HashMap<String, serde_json::Value> {
        &self.parameters
    }
}

/// Workspace context for operations that need workspace information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceContext {
    pub id: String,
    pub workspace_root: String,
    pub git_branch: String,
    pub recent_changes: Vec<String>,
    pub dependencies: HashMap<String, String>,
    pub environment: DeploymentEnvironment,
    pub created_at: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Context for WorkspaceContext {
    fn id(&self) -> &str {
        &self.id
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.created_at
    }

    fn metadata(&self) -> &HashMap<String, serde_json::Value> {
        &self.metadata
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentEnvironment {
    Development,
    Testing,
    Staging,
    Production,
}

/// Request context for HTTP/API operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestContext {
    pub id: String,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub start_time: DateTime<Utc>,
    pub timeout_ms: u64,
    pub headers: HashMap<String, String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Context for RequestContext {
    fn id(&self) -> &str {
        &self.id
    }

    fn timestamp(&self) -> DateTime<Utc> {
        self.start_time
    }

    fn metadata(&self) -> &HashMap<String, serde_json::Value> {
        &self.metadata
    }
}

impl TimeoutContext for RequestContext {
    fn timeout_ms(&self) -> u64 {
        self.timeout_ms
    }

    fn start_time(&self) -> DateTime<Utc> {
        self.start_time
    }
}
