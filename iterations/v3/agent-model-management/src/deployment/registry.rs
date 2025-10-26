//! Deployment registry for tracking model deployments

use crate::deployment::orchestrator::DeploymentInfo;
use crate::types::*;
use crate::ModelManagementError;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Registry for tracking model deployments
#[derive(Debug)]
pub struct DeploymentRegistry {
    /// Active deployments
    deployments: Arc<RwLock<HashMap<String, DeploymentInfo>>>,
}

impl DeploymentRegistry {
    /// Create a new deployment registry
    pub fn new() -> Self {
        Self {
            deployments: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a deployment
    pub async fn register_deployment(&self, info: DeploymentInfo) -> Result<(), ModelManagementError> {
        let mut deployments = self.deployments.write().await;
        deployments.insert(info.model_id.clone(), info);
        Ok(())
    }

    /// Get deployment info
    pub async fn get_deployment(&self, model_id: &str) -> Result<Option<DeploymentInfo>, ModelManagementError> {
        let deployments = self.deployments.read().await;
        Ok(deployments.get(model_id).cloned())
    }
}
