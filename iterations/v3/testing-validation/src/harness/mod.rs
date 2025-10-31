//! Test harness utilities for E2E testing
//!
//! Provides infrastructure for:
//! - Test environment lifecycle management
//! - Local service management (Mistral, Ollama, PostgreSQL)
//! - Isolated workspace creation (Git worktrees)
//! - Assertion framework for Council verdicts and CAWS compliance

pub mod environment;
pub mod assertions;

pub use environment::{TestEnvironment, TestWorkspace};
pub use assertions::AssertionFramework;

use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

use crate::services::{OrchestratorService, OllamaService, PostgresService};
use crate::test_helpers::create_test_autonomous_executor;
use agent_orchestration::autonomous_executor::AutonomousExecutor;
use std::sync::Arc;

/// Manager for local services required by E2E tests
pub struct LocalServiceManager {
    orchestrator: Arc<Mutex<OrchestratorService>>,
    ollama: Arc<Mutex<OllamaService>>,
    postgres: Arc<Mutex<PostgresService>>,
}

impl LocalServiceManager {
    /// Create a new service manager
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        info!("Initializing local service manager");

        let orchestrator = Arc::new(Mutex::new(OrchestratorService::new().await?));
        let ollama = Arc::new(Mutex::new(OllamaService::new().await?));
        let postgres = Arc::new(Mutex::new(PostgresService::new().await?));

        Ok(Self {
            orchestrator,
            ollama,
            postgres,
        })
    }

    /// Start all services
    pub async fn start_all(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting all local services");

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
        }).await?;

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
}
