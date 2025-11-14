//! Test harness utilities for E2E testing
//!
//! Provides infrastructure for:
//! - Test environment lifecycle management
//! - Local service management (Mistral, Ollama, PostgreSQL)
//! - Isolated workspace creation (Git worktrees)
//! - Assertion framework for Council verdicts and CAWS compliance

pub mod assertions;
pub mod environment;

#[cfg(feature = "full")]
pub use assertions::FactChecker;
pub use assertions::{
    AssertionFramework, AssertionType, CawsComplianceResult, Citation, SourceFile,
};
pub use environment::{default_process_output, TestEnvironment, TestWorkspace};

use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};

use crate::services::{OllamaService, OrchestratorService, PostgresService, ServiceManager};
#[cfg(feature = "full")]
use crate::test_helpers::create_test_autonomous_executor;
// autonomous_executor module doesn't exist in agent-orchestration
// #[cfg(feature = "full")]
// use agent_orchestration::autonomous_executor::AutonomousExecutor;

/// Manager for local services required by E2E tests
pub struct LocalServiceManager {
    orchestrator: Arc<Mutex<OrchestratorService>>,
    ollama: Arc<Mutex<OllamaService>>,
    postgres: Arc<Mutex<PostgresService>>,
    service_manager: ServiceManager,
}

impl LocalServiceManager {
    /// Create a new service manager
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        info!("Initializing local service manager");

        // Create comprehensive service manager to check and start dependencies
        let service_manager = ServiceManager::new();

        // Check status of all services
        let statuses = service_manager.check_all_services().await;
        info!("Service status check:");
        for status in &statuses {
            if status.healthy {
                info!("  ✅ {}: Running", status.name);
            } else {
                warn!("  ⚠️  {}: Not running", status.name);
            }
        }

        let orchestrator = Arc::new(Mutex::new(OrchestratorService::new().await?));
        let ollama = Arc::new(Mutex::new(OllamaService::new().await?));
        let postgres = Arc::new(Mutex::new(PostgresService::new().await?));

        Ok(Self {
            orchestrator,
            ollama,
            postgres,
            service_manager,
        })
    }

    /// Start all services (with automatic dependency management)
    pub async fn start_all(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting all local services with automatic dependency management");

        // Ensure required dependencies are running
        let required = vec!["postgres", "ollama"];
        if let Err(e) = self.service_manager.ensure_all_services(&required).await {
            warn!("Some services could not be started automatically: {}", e);
            warn!("Tests may fail if services are not available");
        }

        // Start services in dependency order
        {
            let mut orchestrator = self.orchestrator.lock().await;
            orchestrator.start().await?;
        }

        {
            let mut pg = self.postgres.lock().await;
            pg.start().await?;
        }

        {
            let mut ollama = self.ollama.lock().await;
            ollama.start().await?;
        }

        info!("All services started");
        Ok(())
    }

    /// Stop all services
    pub async fn stop_all(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Stopping all local services");

        // Stop in reverse dependency order
        {
            let mut ollama = self.ollama.lock().await;
            ollama.stop().await?;
        }

        {
            let mut pg = self.postgres.lock().await;
            pg.stop().await?;
        }

        {
            let mut orchestrator = self.orchestrator.lock().await;
            orchestrator.stop().await?;
        }

        info!("All services stopped");
        Ok(())
    }

    /// Wait for all services to be healthy
    pub async fn wait_for_healthy(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Waiting for all services to be healthy");

        // Wait for each service with timeout
        let timeout_duration = std::time::Duration::from_secs(120); // 2 minutes

        tokio::time::timeout(timeout_duration, async {
            loop {
                let mut all_healthy = true;

                {
                    let orchestrator = self.orchestrator.lock().await;
                    if !orchestrator.is_healthy().await {
                        all_healthy = false;
                    }
                }

                {
                    let pg = self.postgres.lock().await;
                    if !pg.is_healthy().await {
                        all_healthy = false;
                    }
                }

                {
                    let ollama = self.ollama.lock().await;
                    if !ollama.is_healthy().await {
                        all_healthy = false;
                    }
                }

                if all_healthy {
                    break;
                }

                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        })
        .await?;

        info!("All services are healthy");
        Ok(())
    }

    /// Get access to individual services
    pub fn orchestrator(&self) -> Arc<Mutex<OrchestratorService>> {
        Arc::clone(&self.orchestrator)
    }

    pub fn ollama(&self) -> Arc<Mutex<OllamaService>> {
        Arc::clone(&self.ollama)
    }

    pub fn postgres(&self) -> Arc<Mutex<PostgresService>> {
        Arc::clone(&self.postgres)
    }

    /// Get service manager for checking/starting dependencies
    pub fn service_manager(&self) -> &ServiceManager {
        &self.service_manager
    }
}
