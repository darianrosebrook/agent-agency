//! Orchestrator service management
//!
//! Manages the Mistral CoreML model instance used for task orchestration
//! in autonomous testing scenarios.

use std::process::{Command, Stdio};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn, error};
use agent_orchestration::coreml::{CoreMLManager, CoreMLModelType};
use std::path::PathBuf;

/// Service for managing Mistral CoreML orchestrator
pub struct OrchestratorService {
    coreml_manager: Option<Arc<CoreMLManager>>,
    model_path: PathBuf,
    mistral_model: Option<Arc<agent_orchestration::coreml::CoreMLModel>>,
}

impl OrchestratorService {
    /// Create a new orchestrator service
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Find Mistral model path - check /models/coreml or use env var
        let model_path = std::env::var("COREML_MODELS_PATH")
            .map(|p| PathBuf::from(p))
            .unwrap_or_else(|_| PathBuf::from("/models/coreml"));

        // Initialize CoreML manager
        let coreml_manager = Arc::new(CoreMLManager::new(model_path.clone()));

        info!("Orchestrator service initialized with CoreML manager, model path: {:?}", model_path);

        Ok(Self {
            coreml_manager: Some(coreml_manager),
            model_path,
            mistral_model: None,
        })
    }

    /// Start the orchestrator service
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting Mistral CoreML orchestrator service");

        if let Some(manager) = &self.coreml_manager {
            // Load available CoreML models
            manager.load_available_models().await?;

            // Get the Mistral model
            self.mistral_model = manager.get_model(CoreMLModelType::Language, "Mistral-7B-Instruct-FP16").await;

            if let Some(model) = &self.mistral_model {
                info!("Mistral CoreML model loaded: {}, ANE supported: {}",
                    model.metadata.name, model.metadata.supports_ane);
            } else {
                warn!("Mistral model not found, service will operate in CPU-only mode");
            }

            let model_count = manager.model_count().await;
            info!("CoreML orchestrator service started with {} models loaded, ANE available: {}",
                model_count, manager.is_ane_available());
        } else {
            return Err("CoreML manager not initialized".into());
        }

        Ok(())
    }

    /// Stop the orchestrator service
    pub async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Stopping Mistral CoreML orchestrator service");

        // Clear the loaded model reference
        self.mistral_model = None;

        info!("Mistral CoreML orchestrator service stopped");
        Ok(())
    }

    /// Check if the service is healthy
    pub async fn is_healthy(&self) -> bool {
        // Check if CoreML manager is initialized and Mistral model is loaded
        if let Some(manager) = &self.coreml_manager {
            // Service is healthy if ANE is available or at least CPU models are loaded
            let ane_available = manager.is_ane_available();
            let has_mistral = self.mistral_model.is_some();
            let model_count = manager.model_count().await > 0;

            if ane_available && has_mistral {
                info!("Orchestrator healthy: ANE available with Mistral model loaded");
                true
            } else if model_count {
                info!("Orchestrator healthy: CPU models loaded (ANE: {})", ane_available);
                true
            } else {
                warn!("Orchestrator unhealthy: no models loaded");
                false
            }
        } else {
            warn!("Orchestrator unhealthy: CoreML manager not initialized");
            false
        }
    }

    /// Get CoreML manager reference
    pub fn coreml_manager(&self) -> Option<&Arc<CoreMLManager>> {
        self.coreml_manager.as_ref()
    }

    /// Get Mistral model reference
    pub fn mistral_model(&self) -> Option<&Arc<agent_orchestration::coreml::CoreMLModel>> {
        self.mistral_model.as_ref()
    }

    /// Get model path
    pub fn model_path(&self) -> &PathBuf {
        &self.model_path
    }
}

