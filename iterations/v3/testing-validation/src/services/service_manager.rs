//! Comprehensive Service Manager for Test Infrastructure
//!
//! Automatically checks and starts all required external dependencies:
//! - PostgreSQL (via Docker or local process)
//! - Ollama (checks if running, starts if needed)
//! - Embedding Service (checks Ollama endpoint)
//! - API Server (starts if not running)
//! - CoreML Models (validates presence)
//!
//! @author @darianrosebrook

use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, warn, debug};
use reqwest::Client;

/// Service status information
#[derive(Debug, Clone)]
pub struct ServiceStatus {
    pub name: String,
    pub running: bool,
    pub healthy: bool,
    pub endpoint: Option<String>,
    pub error: Option<String>,
}

/// Comprehensive service manager that checks and starts all dependencies
pub struct ServiceManager {
    postgres_url: String,
    ollama_url: String,
    embedding_url: String,
    api_server_port: u16,
    coreml_models_path: String,
    http_client: Client,
}

impl ServiceManager {
    /// Create a new service manager with default configuration
    pub fn new() -> Self {
        Self {
            postgres_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://postgres@localhost:5432/postgres".to_string()),
            ollama_url: std::env::var("OLLAMA_URL")
                .unwrap_or_else(|_| "http://localhost:11434".to_string()),
            embedding_url: std::env::var("EMBEDDING_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:11434".to_string()),
            api_server_port: std::env::var("API_SERVER_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000),
            coreml_models_path: std::env::var("COREML_MODELS_PATH")
                .unwrap_or_else(|_| {
                    // Default to workspace root models directory
                    // This will be resolved relative to workspace root in get_possible_model_paths
                    "models/coreml".to_string()
                }),
            http_client: Client::new(),
        }
    }

    /// Check status of all services
    pub async fn check_all_services(&self) -> Vec<ServiceStatus> {
        let mut statuses = Vec::new();

        statuses.push(self.check_postgres().await);
        statuses.push(self.check_ollama().await);
        statuses.push(self.check_embedding_service().await);
        statuses.push(self.check_api_server().await);
        statuses.push(self.check_coreml_models().await);

        statuses
    }

    /// Check PostgreSQL status
    pub async fn check_postgres(&self) -> ServiceStatus {
        info!("Checking PostgreSQL service...");

        // Try to connect to PostgreSQL
        let running = self.check_postgres_connection().await;
        let healthy = running;

        ServiceStatus {
            name: "PostgreSQL".to_string(),
            running,
            healthy,
            endpoint: Some(self.postgres_url.clone()),
            error: if !running {
                Some("PostgreSQL not accessible".to_string())
            } else {
                None
            },
        }
    }

    /// Check PostgreSQL connection
    async fn check_postgres_connection(&self) -> bool {
        // Try to connect using sqlx with timeout
        match tokio::time::timeout(
            Duration::from_secs(2),
            sqlx::postgres::PgPoolOptions::new()
                .max_connections(1)
                .connect(&self.postgres_url)
        ).await
        {
            Ok(Ok(pool)) => {
                // Test with a simple query
                match tokio::time::timeout(
                    Duration::from_secs(1),
                    sqlx::query("SELECT 1").execute(&pool)
                ).await
                {
                    Ok(Ok(_)) => {
                        pool.close().await;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    /// Check Ollama status
    pub async fn check_ollama(&self) -> ServiceStatus {
        info!("Checking Ollama service...");

        let running = self.check_ollama_health().await;
        let healthy = running;

        ServiceStatus {
            name: "Ollama".to_string(),
            running,
            healthy,
            endpoint: Some(self.ollama_url.clone()),
            error: if !running {
                Some("Ollama not accessible".to_string())
            } else {
                None
            },
        }
    }

    /// Check Ollama health via HTTP
    async fn check_ollama_health(&self) -> bool {
        match self.http_client
            .get(&format!("{}/api/tags", self.ollama_url))
            .timeout(Duration::from_secs(2))
            .send()
            .await
        {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    /// Check embedding service status
    pub async fn check_embedding_service(&self) -> ServiceStatus {
        info!("Checking embedding service...");

        // Embedding service is usually the same as Ollama
        let running = self.check_embedding_endpoint().await;
        let healthy = running;

        ServiceStatus {
            name: "Embedding Service".to_string(),
            running,
            healthy,
            endpoint: Some(self.embedding_url.clone()),
            error: if !running {
                Some("Embedding service not accessible".to_string())
            } else {
                None
            },
        }
    }

    /// Check embedding service endpoint
    async fn check_embedding_endpoint(&self) -> bool {
        // Check if embedding endpoint is available
        match self.http_client
            .get(&format!("{}/api/v1/embeddings", self.embedding_url))
            .timeout(Duration::from_secs(2))
            .send()
            .await
        {
            Ok(response) => {
                // Even if it returns an error, the endpoint exists
                response.status().as_u16() < 500
            }
            Err(_) => false,
        }
    }

    /// Check API server status
    pub async fn check_api_server(&self) -> ServiceStatus {
        info!("Checking API server...");

        let endpoint = format!("http://localhost:{}", self.api_server_port);
        let running = self.check_api_server_health(&endpoint).await;
        let healthy = running;

        ServiceStatus {
            name: "API Server".to_string(),
            running,
            healthy,
            endpoint: Some(endpoint),
            error: if !running {
                Some("API server not accessible".to_string())
            } else {
                None
            },
        }
    }

    /// Check API server health
    async fn check_api_server_health(&self, endpoint: &str) -> bool {
        // Try health endpoint or root
        let health_endpoints = [
            format!("{}/health", endpoint),
            format!("{}/api/health", endpoint),
            format!("{}/", endpoint),
        ];

        for health_url in &health_endpoints {
            match self.http_client
                .get(health_url)
                .timeout(Duration::from_secs(2))
                .send()
                .await
            {
                Ok(response) if response.status().is_success() => return true,
                Ok(_) => continue,
                Err(_) => continue,
            }
        }

        false
    }

    /// Check CoreML models availability
    pub async fn check_coreml_models(&self) -> ServiceStatus {
        info!("Checking CoreML models...");

        let (models_exist, found_path) = self.check_models_present().await;
        let healthy = models_exist;

        ServiceStatus {
            name: "CoreML Models".to_string(),
            running: models_exist,
            healthy,
            endpoint: Some(found_path.unwrap_or_else(|| self.coreml_models_path.clone())),
            error: if !models_exist {
                let paths = self.get_possible_model_paths();
                Some(format!("CoreML models not found. Checked locations:\n  {}", 
                    paths.iter()
                        .take(5) // Show first 5 paths
                        .map(|p| p.display().to_string())
                        .collect::<Vec<_>>()
                        .join("\n  ")))
            } else {
                None
            },
        }
    }

    /// Check if CoreML models are present
    /// Returns (found, path_where_found)
    async fn check_models_present(&self) -> (bool, Option<String>) {

        // Try multiple possible locations for models
        let possible_paths = self.get_possible_model_paths();
        
        for models_path in &possible_paths {
            if models_path.exists() {
                // Check for expected model files
                let expected_models = [
                    ("fastvit/FastViTT8F16.mlpackage.mlmodelc", "FastViTT8F16.mlpackage.mlmodelc"),
                    ("mistral/StatefulMistral7BInstructFP16.mlpackage.mlmodelc", "StatefulMistral7BInstructFP16.mlpackage.mlmodelc"),
                ];

                let mut found_count = 0;
                for (subpath, name) in &expected_models {
                    // Try subdirectory first (fastvit/, mistral/)
                    let model_path = models_path.join(subpath);
                    if model_path.exists() {
                        found_count += 1;
                        continue;
                    }
                    
                    // Try direct path
                    let model_path = models_path.join(name);
                    if model_path.exists() {
                        found_count += 1;
                        continue;
                    }
                    
                    debug!("Model not found: {} or {}", models_path.join(subpath).display(), models_path.join(name).display());
                }

                if found_count >= 1 {
                    // At least one model found - good enough for now
                    // Normalize path (resolve .. and .)
                    let normalized_path = models_path.canonicalize()
                        .unwrap_or_else(|_| models_path.clone())
                        .display()
                        .to_string();
                    return (true, Some(normalized_path));
                }
            }
        }

        (false, None)
    }
    
    /// Get possible model paths to check
    fn get_possible_model_paths(&self) -> Vec<std::path::PathBuf> {
        use std::path::PathBuf;
        
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        
        let mut paths = Vec::new();
        
        // 1. Use configured path (could be absolute or relative)
        if !self.coreml_models_path.is_empty() {
            let configured = PathBuf::from(&self.coreml_models_path);
            if configured.is_absolute() {
                paths.push(configured);
            } else {
                // Try relative to current directory
                paths.push(current_dir.join(&configured));
                // Try relative to workspace root (go up from iterations/v3/testing-validation)
                paths.push(current_dir.join("../../models/coreml"));
                paths.push(current_dir.join("../../../models/coreml"));
            }
        }
        
        // 2. Try common locations relative to workspace root
        paths.push(current_dir.join("../../models/coreml")); // From testing-validation
        paths.push(current_dir.join("../../../models/coreml")); // From testing-validation (alternative)
        paths.push(current_dir.join("models/coreml")); // From workspace root
        paths.push(PathBuf::from("models/coreml")); // Relative to current dir
        
        // 3. Try absolute paths from common project structures
        // (Future: could check HOME directory, but unlikely)
        
        // Remove duplicates
        paths.sort();
        paths.dedup();
        
        paths
    }

    /// Start PostgreSQL if not running
    pub async fn ensure_postgres(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let status = self.check_postgres().await;
        
        if status.running {
            info!("PostgreSQL is already running");
            return Ok(());
        }

        info!("PostgreSQL is not running, attempting to start...");

        // Try to start via Docker
        if self.start_postgres_docker().await.is_ok() {
            info!("PostgreSQL started via Docker");
            return Ok(());
        }

        // Try to start via docker-compose
        if self.start_postgres_compose().await.is_ok() {
            info!("PostgreSQL started via docker-compose");
            return Ok(());
        }

        warn!("Could not start PostgreSQL automatically. Please start manually:");
        warn!("  docker run -d -p 5432:5432 -e POSTGRES_PASSWORD=postgres postgres:15");
        warn!("  or");
        warn!("  docker-compose -f docker-compose.test.yml up -d postgres");

        Err("PostgreSQL could not be started automatically".into())
    }

    /// Start PostgreSQL via Docker
    async fn start_postgres_docker(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let output = Command::new("docker")
            .args(&[
                "run", "-d",
                "--name", "agent_agency_test_postgres",
                "-p", "5432:5432",
                "-e", "POSTGRES_PASSWORD=postgres",
                "-e", "POSTGRES_USER=postgres",
                "postgres:15",
            ])
            .output()?;

        if output.status.success() {
            // Wait for PostgreSQL to be ready
            self.wait_for_postgres().await?;
            Ok(())
        } else {
            Err("Failed to start PostgreSQL container".into())
        }
    }

    /// Start PostgreSQL via docker-compose
    async fn start_postgres_compose(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Find docker-compose file
        let compose_files = [
            "iterations/v3/testing-validation/docker-compose.test.yml",
            "testing-validation/docker-compose.test.yml",
            "docker-compose.test.yml",
        ];

        for compose_file in &compose_files {
            if std::path::Path::new(compose_file).exists() {
                let output = Command::new("docker-compose")
                    .args(&["-f", compose_file, "up", "-d", "postgres"])
                    .output()?;

                if output.status.success() {
                    self.wait_for_postgres().await?;
                    return Ok(());
                }
            }
        }

        Err("docker-compose file not found".into())
    }

    /// Wait for PostgreSQL to be ready
    async fn wait_for_postgres(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for _ in 0..30 {
            if self.check_postgres_connection().await {
                return Ok(());
            }
            sleep(Duration::from_secs(1)).await;
        }
        Err("PostgreSQL did not become ready in time".into())
    }

    /// Ensure Ollama is running
    pub async fn ensure_ollama(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let status = self.check_ollama().await;
        
        if status.running {
            info!("Ollama is already running");
            return Ok(());
        }

        info!("Ollama is not running, attempting to start...");

        // Try to start Ollama
        match Command::new("ollama")
            .arg("serve")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(_) => {
                info!("Ollama process started, waiting for readiness...");
                self.wait_for_ollama().await?;
                Ok(())
            }
            Err(e) => {
                warn!("Could not start Ollama: {}. Please start manually: ollama serve", e);
                Err(format!("Ollama could not be started: {}", e).into())
            }
        }
    }

    /// Wait for Ollama to be ready
    async fn wait_for_ollama(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for _ in 0..30 {
            if self.check_ollama_health().await {
                return Ok(());
            }
            sleep(Duration::from_secs(1)).await;
        }
        Err("Ollama did not become ready in time".into())
    }

    /// Ensure embedding service is available
    pub async fn ensure_embedding_service(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let status = self.check_embedding_service().await;
        
        if status.running {
            info!("Embedding service is already available");
            return Ok(());
        }

        // Embedding service is usually Ollama, so ensure Ollama is running
        warn!("Embedding service not available, ensuring Ollama is running...");
        self.ensure_ollama().await?;

        // Check again
        let status = self.check_embedding_service().await;
        if status.running {
            Ok(())
        } else {
            Err("Embedding service still not available after starting Ollama".into())
        }
    }

    /// Ensure API server is running
    pub async fn ensure_api_server(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let status = self.check_api_server().await;
        
        if status.running {
            info!("API server is already running");
            return Ok(());
        }

        info!("API server is not running, attempting to start...");

        // Find and start API server - check multiple possible locations
        use std::path::PathBuf;
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        
        // Find workspace root
        let workspace_root = current_dir
            .ancestors()
            .find(|p| {
                let cargo_toml = p.join("Cargo.toml");
                let iterations = p.join("iterations");
                cargo_toml.exists() && iterations.exists()
            })
            .or_else(|| current_dir.ancestors().find(|p| p.join("Cargo.toml").exists()))
            .unwrap_or_else(|| current_dir.as_path());
        
        let v3_dir = workspace_root.join("iterations/v3");
        
        // Check for binary in common locations (including target-specific directories)
        let possible_binary_paths = [
            v3_dir.join("target/debug/agent-agency-api-server"),
            v3_dir.join("target/aarch64-apple-darwin/debug/agent-agency-api-server"),
            v3_dir.join("data-interfaces-adapters/target/debug/agent-agency-api-server"),
            workspace_root.join("target/debug/agent-agency-api-server"),
            current_dir.join("target/debug/agent-agency-api-server"),
        ];

        for binary_path in &possible_binary_paths {
            if binary_path.exists() {
                info!("Found API server binary at: {}", binary_path.display());
                return self.start_api_server_binary(binary_path).await;
            }
        }

        // Binary not found - try to build it
        info!("API server binary not found, attempting to build...");
        self.build_and_start_api_server(&v3_dir).await?;
        self.wait_for_api_server().await?;
        Ok(())
    }

    /// Build and start API server
    async fn build_and_start_api_server(&self, v3_dir: &std::path::Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use tokio::process::Command as TokioCommand;
        
        info!("Building API server (this may take a minute)...");
        let build_output = TokioCommand::new("cargo")
            .arg("build")
            .arg("--bin")
            .arg("agent-agency-api-server")
            .arg("-p")
            .arg("data-interfaces-adapters")
            .current_dir(v3_dir)
            .output()
            .await?;
        
        if !build_output.status.success() {
            let stderr = String::from_utf8_lossy(&build_output.stderr);
            warn!("API server build failed: {}", stderr);
            return Err(format!("Failed to build API server: {}", stderr).into());
        }
        
        info!("API server built successfully");
        
        // Try to find the binary after build
        let possible_binary_paths = [
            v3_dir.join("target/debug/agent-agency-api-server"),
            v3_dir.join("target/aarch64-apple-darwin/debug/agent-agency-api-server"),
        ];
        
        for binary_path in &possible_binary_paths {
            if binary_path.exists() {
                return self.start_api_server_binary(binary_path).await;
            }
        }
        
        Err("API server binary not found after build".into())
    }
    
    /// Start API server binary
    async fn start_api_server_binary(&self, binary_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use tokio::process::Command as TokioCommand;
        
        info!("Starting API server from: {}", binary_path.display());
        
        // Start the server in the background
        let mut child = TokioCommand::new(binary_path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        
        // Give it a moment to start
        sleep(Duration::from_millis(500)).await;
        
        // Check if it's still running
        match child.try_wait() {
            Ok(Some(status)) => {
                return Err(format!("API server exited immediately with status: {:?}", status).into());
            }
            Ok(None) => {
                info!("API server process started successfully");
                // Don't wait for the child - let it run in background
                // The process will be cleaned up when the test harness stops
            }
            Err(e) => {
                return Err(format!("Failed to check API server status: {}", e).into());
            }
        }
        
        Ok(())
    }

    /// Wait for API server to be ready
    async fn wait_for_api_server(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let endpoint = format!("http://localhost:{}", self.api_server_port);
        for _ in 0..30 {
            if self.check_api_server_health(&endpoint).await {
                return Ok(());
            }
            sleep(Duration::from_secs(1)).await;
        }
        Err("API server did not become ready in time".into())
    }

    /// Ensure CoreML models are available
    pub async fn ensure_coreml_models(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let status = self.check_coreml_models().await;
        
        if status.running {
            info!("CoreML models are available");
            return Ok(());
        }

        warn!("CoreML models not found at: {}", self.coreml_models_path);
        warn!("Please ensure models are available at the expected location");
        Err("CoreML models not available".into())
    }

    /// Ensure all required services are running
    pub async fn ensure_all_services(&self, required: &[&str]) -> Result<Vec<ServiceStatus>, Box<dyn std::error::Error + Send + Sync>> {
        info!("Ensuring all required services are running...");

        let mut statuses = Vec::new();

        // Check and start services based on requirements
        if required.contains(&"postgres") || required.contains(&"database") {
            self.ensure_postgres().await?;
            statuses.push(self.check_postgres().await);
        }

        if required.contains(&"ollama") {
            self.ensure_ollama().await?;
            statuses.push(self.check_ollama().await);
        }

        if required.contains(&"embedding") || required.contains(&"embeddings") {
            self.ensure_embedding_service().await?;
            statuses.push(self.check_embedding_service().await);
        }

        if required.contains(&"api") || required.contains(&"api-server") {
            self.ensure_api_server().await?;
            statuses.push(self.check_api_server().await);
        }

        if required.contains(&"coreml") || required.contains(&"models") {
            self.ensure_coreml_models().await?;
            statuses.push(self.check_coreml_models().await);
        }

        // Verify all required services are healthy
        for status in &statuses {
            if !status.healthy {
                return Err(format!("Service {} is not healthy", status.name).into());
            }
        }

        info!("All required services are running and healthy");
        Ok(statuses)
    }
}

impl Default for ServiceManager {
    fn default() -> Self {
        Self::new()
    }
}

