//! Task router for intelligent task distribution
//!
//! Routes tasks to appropriate workers based on capabilities, load, and performance.

use schemars::JsonSchema;
use serde::{Serialize, Deserialize};
use crate::worker_errors::WorkerError;
use crate::specialized_workers::SpecializedWorker;
use std::collections::HashMap;

/// Task router configuration

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct RouterConfig {
    pub capability_matching_threshold: f32,
    pub load_balancing_enabled: bool,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            capability_matching_threshold: 0.7,
            load_balancing_enabled: true,
        }
    }
}

/// Task router for intelligent distribution
pub struct TaskRouter {
    config: RouterConfig,
    workers: HashMap<String, Box<dyn SpecializedWorker + Send + Sync>>,
}

impl TaskRouter {
    pub fn new(config: RouterConfig) -> Self {
        Self {
            config,
            workers: HashMap::new(),
        }
    }

    pub fn register_worker(&mut self, name: String, worker: Box<dyn SpecializedWorker + Send + Sync>) {
        self.workers.insert(name, worker);
    }

    pub async fn route_task(&self, task: String, required_capabilities: &[String]) -> Result<String, WorkerError> {
        // Find best worker for the task
        for (name, worker) in &self.workers {
            let worker_caps = worker.capabilities();
            let match_score = self.calculate_match_score(required_capabilities, &worker_caps);

            if match_score >= self.config.capability_matching_threshold {
                return worker.execute(task).await;
            }
        }

        Err(WorkerError::NoSuitableWorker { message: "No worker found with required capabilities".to_string() })
    }

    fn calculate_match_score(&self, required: &[String], available: &[String]) -> f32 {
        let matches = required.iter()
            .filter(|req| available.contains(req))
            .count();
        matches as f32 / required.len() as f32
    }
}
