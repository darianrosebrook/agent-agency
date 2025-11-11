//! Orchestrator service management
//!
//! Manages the Mistral CoreML model instance used for task orchestration
//! in autonomous testing scenarios.

use tracing::info;
#[cfg(feature = "full")]
use agent_orchestration::coreml::{CoreMLManager, CoreMLModelType};
use std::path::PathBuf;
use std::sync::Arc;

/// Service for managing Mistral CoreML orchestrator
pub struct OrchestratorService {
    #[cfg(feature = "full")]
    coreml_manager: Option<Arc<CoreMLManager>>,
    model_path: PathBuf,
    #[cfg(feature = "full")]
    mistral_model: Option<Arc<agent_orchestration::coreml::CoreMLModel>>,
}

impl OrchestratorService {
    /// Create a new orchestrator service
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Find Mistral model path - check /models/coreml or use env var
        let model_path = std::env::var("COREML_MODELS_PATH")
            .map(|p| PathBuf::from(p))
            .unwrap_or_else(|_| PathBuf::from("/models/coreml"));

        #[cfg(feature = "full")]
        {
            // Initialize CoreML manager
            let coreml_manager = Arc::new(CoreMLManager::new(model_path.clone()));

            info!("Orchestrator service initialized with CoreML manager, model path: {:?}", model_path);

            Ok(Self {
                coreml_manager: Some(coreml_manager),
                model_path,
                mistral_model: None,
            })
        }

        #[cfg(not(feature = "full"))]
        {
            info!("Orchestrator service initialized in basic mode (no CoreML), model path: {:?}", model_path);
            Ok(Self {
                model_path,
            })
        }
    }

    /// Start the orchestrator service
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        #[cfg(feature = "full")]
        {
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
                let ane_available = manager.is_ane_available();
                info!("CoreML orchestrator service started with {} models loaded, ANE available: {}",
                    model_count, ane_available);
            } else {
                return Err("CoreML manager not initialized".into());
            }
        }

        #[cfg(not(feature = "full"))]
        {
            info!("Starting orchestrator service in basic mode (no CoreML)");
        }

        Ok(())
    }

    /// Stop the orchestrator service
    pub async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        #[cfg(feature = "full")]
        {
            info!("Stopping Mistral CoreML orchestrator service");
            // Clear the loaded model reference
            self.mistral_model = None;
        }

        #[cfg(not(feature = "full"))]
        {
            info!("Stopping orchestrator service");
        }

        info!("Orchestrator service stopped");
        Ok(())
    }

    /// Check if the service is healthy
    pub async fn is_healthy(&self) -> bool {
        #[cfg(feature = "full")]
        {
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

        #[cfg(not(feature = "full"))]
        {
            // In basic mode, service is always considered healthy
            info!("Orchestrator healthy: running in basic mode (no CoreML)");
            true
        }
    }

    /// Get CoreML manager reference
    #[cfg(feature = "full")]
    pub fn coreml_manager(&self) -> Option<&Arc<CoreMLManager>> {
        self.coreml_manager.as_ref()
    }

    /// Get Mistral model reference
    #[cfg(feature = "full")]
    pub fn mistral_model(&self) -> Option<&Arc<agent_orchestration::coreml::CoreMLModel>> {
        self.mistral_model.as_ref()
    }

    /// Get model path
    pub fn model_path(&self) -> &PathBuf {
        &self.model_path
    }
}

