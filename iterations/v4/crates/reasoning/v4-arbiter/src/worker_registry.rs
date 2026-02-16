//! Worker Registry
//!
//! Tracks available workers for task delegation. The registry maintains
//! an in-memory directory of workers with their capabilities, skills,
//! and availability status.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};
use v4_types::{WorkerCapability, WorkerType};

/// Worker availability status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkerStatus {
    /// Ready to accept tasks
    Available,
    /// Currently executing a task
    Busy,
    /// Not responding / disconnected
    Offline,
}

/// A registered worker entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerEntry {
    /// Unique worker identifier
    pub worker_id: String,
    /// A2A endpoint URL
    pub url: String,
    /// Execution environment type
    pub worker_type: WorkerType,
    /// Capabilities this worker supports
    pub capabilities: Vec<WorkerCapability>,
    /// Skill IDs this worker can handle (e.g. "draft-content", "generate-code")
    pub skills: Vec<String>,
    /// Current status
    pub status: WorkerStatus,
    /// When the worker was registered
    pub registered_at: chrono::DateTime<chrono::Utc>,
    /// Last heartbeat timestamp
    pub last_heartbeat: Option<chrono::DateTime<chrono::Utc>>,
}

/// Thread-safe worker registry
#[derive(Clone)]
pub struct WorkerRegistry {
    workers: Arc<RwLock<HashMap<String, WorkerEntry>>>,
}

impl WorkerRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            workers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a worker
    pub fn register(&self, entry: WorkerEntry) {
        let mut workers = self.workers.write().unwrap();
        workers.insert(entry.worker_id.clone(), entry);
    }

    /// Unregister a worker
    pub fn unregister(&self, worker_id: &str) {
        let mut workers = self.workers.write().unwrap();
        workers.remove(worker_id);
    }

    /// Get a worker by ID
    pub fn get(&self, worker_id: &str) -> Option<WorkerEntry> {
        let workers = self.workers.read().unwrap();
        workers.get(worker_id).cloned()
    }

    /// Find workers with a specific capability
    pub fn find_by_capability(&self, cap: &WorkerCapability) -> Vec<WorkerEntry> {
        let workers = self.workers.read().unwrap();
        workers
            .values()
            .filter(|w| w.capabilities.contains(cap))
            .cloned()
            .collect()
    }

    /// Find workers that support a specific skill
    pub fn find_by_skill(&self, skill_id: &str) -> Vec<WorkerEntry> {
        let workers = self.workers.read().unwrap();
        workers
            .values()
            .filter(|w| w.skills.iter().any(|s| s == skill_id))
            .cloned()
            .collect()
    }

    /// Find workers of a specific type
    pub fn find_by_type(&self, worker_type: &WorkerType) -> Vec<WorkerEntry> {
        let workers = self.workers.read().unwrap();
        workers
            .values()
            .filter(|w| &w.worker_type == worker_type)
            .cloned()
            .collect()
    }

    /// Find all available workers
    pub fn find_available(&self) -> Vec<WorkerEntry> {
        let workers = self.workers.read().unwrap();
        workers
            .values()
            .filter(|w| w.status == WorkerStatus::Available)
            .cloned()
            .collect()
    }

    /// Update a worker's status
    pub fn update_status(&self, worker_id: &str, status: WorkerStatus) {
        let mut workers = self.workers.write().unwrap();
        if let Some(worker) = workers.get_mut(worker_id) {
            worker.status = status;
        }
    }

    /// Update a worker's heartbeat to now
    pub fn update_heartbeat(&self, worker_id: &str) {
        let mut workers = self.workers.write().unwrap();
        if let Some(worker) = workers.get_mut(worker_id) {
            worker.last_heartbeat = Some(chrono::Utc::now());
        }
    }

    /// Total registered workers
    pub fn count(&self) -> usize {
        let workers = self.workers.read().unwrap();
        workers.len()
    }

    /// List all registered workers
    pub fn list(&self) -> Vec<WorkerEntry> {
        let workers = self.workers.read().unwrap();
        workers.values().cloned().collect()
    }
}

impl Default for WorkerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(id: &str, status: WorkerStatus) -> WorkerEntry {
        WorkerEntry {
            worker_id: id.to_string(),
            url: format!("http://localhost:300{}", id.chars().last().unwrap_or('0')),
            worker_type: WorkerType::Local,
            capabilities: vec![WorkerCapability::General],
            skills: vec!["draft-content".to_string()],
            status,
            registered_at: chrono::Utc::now(),
            last_heartbeat: None,
        }
    }

    #[test]
    fn test_register_and_get() {
        let registry = WorkerRegistry::new();
        let entry = make_entry("w1", WorkerStatus::Available);
        registry.register(entry);

        assert_eq!(registry.count(), 1);
        let w = registry.get("w1").unwrap();
        assert_eq!(w.worker_id, "w1");
        assert_eq!(w.status, WorkerStatus::Available);
    }

    #[test]
    fn test_unregister() {
        let registry = WorkerRegistry::new();
        registry.register(make_entry("w1", WorkerStatus::Available));
        registry.register(make_entry("w2", WorkerStatus::Available));
        assert_eq!(registry.count(), 2);

        registry.unregister("w1");
        assert_eq!(registry.count(), 1);
        assert!(registry.get("w1").is_none());
        assert!(registry.get("w2").is_some());
    }

    #[test]
    fn test_find_by_capability() {
        let registry = WorkerRegistry::new();

        let mut code_worker = make_entry("code1", WorkerStatus::Available);
        code_worker.capabilities = vec![WorkerCapability::CodeEditing, WorkerCapability::General];

        let mut research_worker = make_entry("research1", WorkerStatus::Available);
        research_worker.capabilities = vec![WorkerCapability::Research];

        registry.register(code_worker);
        registry.register(research_worker);

        let code_workers = registry.find_by_capability(&WorkerCapability::CodeEditing);
        assert_eq!(code_workers.len(), 1);
        assert_eq!(code_workers[0].worker_id, "code1");

        let general_workers = registry.find_by_capability(&WorkerCapability::General);
        assert_eq!(general_workers.len(), 1);
    }

    #[test]
    fn test_find_by_skill() {
        let registry = WorkerRegistry::new();

        let mut w1 = make_entry("w1", WorkerStatus::Available);
        w1.skills = vec!["draft-content".to_string(), "review-text".to_string()];

        let mut w2 = make_entry("w2", WorkerStatus::Available);
        w2.skills = vec!["generate-code".to_string()];

        registry.register(w1);
        registry.register(w2);

        let drafters = registry.find_by_skill("draft-content");
        assert_eq!(drafters.len(), 1);
        assert_eq!(drafters[0].worker_id, "w1");

        let coders = registry.find_by_skill("generate-code");
        assert_eq!(coders.len(), 1);
        assert_eq!(coders[0].worker_id, "w2");

        let none = registry.find_by_skill("nonexistent");
        assert!(none.is_empty());
    }

    #[test]
    fn test_find_available() {
        let registry = WorkerRegistry::new();
        registry.register(make_entry("w1", WorkerStatus::Available));
        registry.register(make_entry("w2", WorkerStatus::Busy));
        registry.register(make_entry("w3", WorkerStatus::Offline));
        registry.register(make_entry("w4", WorkerStatus::Available));

        let available = registry.find_available();
        assert_eq!(available.len(), 2);
    }

    #[test]
    fn test_update_status() {
        let registry = WorkerRegistry::new();
        registry.register(make_entry("w1", WorkerStatus::Available));

        registry.update_status("w1", WorkerStatus::Busy);
        let w = registry.get("w1").unwrap();
        assert_eq!(w.status, WorkerStatus::Busy);

        registry.update_status("w1", WorkerStatus::Offline);
        let w = registry.get("w1").unwrap();
        assert_eq!(w.status, WorkerStatus::Offline);
    }

    #[test]
    fn test_update_heartbeat() {
        let registry = WorkerRegistry::new();
        registry.register(make_entry("w1", WorkerStatus::Available));

        let w = registry.get("w1").unwrap();
        assert!(w.last_heartbeat.is_none());

        registry.update_heartbeat("w1");
        let w = registry.get("w1").unwrap();
        assert!(w.last_heartbeat.is_some());
    }

    #[test]
    fn test_find_by_type() {
        let registry = WorkerRegistry::new();

        let mut remote = make_entry("remote1", WorkerStatus::Available);
        remote.worker_type = WorkerType::Remote {
            url: "http://remote:3001".to_string(),
        };

        registry.register(make_entry("local1", WorkerStatus::Available));
        registry.register(remote);

        let locals = registry.find_by_type(&WorkerType::Local);
        assert_eq!(locals.len(), 1);

        let remotes = registry.find_by_type(&WorkerType::Remote {
            url: "http://remote:3001".to_string(),
        });
        assert_eq!(remotes.len(), 1);
    }

    #[test]
    fn test_list() {
        let registry = WorkerRegistry::new();
        registry.register(make_entry("w1", WorkerStatus::Available));
        registry.register(make_entry("w2", WorkerStatus::Busy));

        let all = registry.list();
        assert_eq!(all.len(), 2);
    }
}
