//! Ollama service management
//!
//! Manages local Ollama instance for model inference in autonomous testing.

use std::process::{Command, Stdio};
use tracing::{info, warn, error};

/// Service for managing local Ollama instance
pub struct OllamaService {
    base_url: String,
    model_name: String,
    process_handle: Option<std::process::Child>,
}

impl OllamaService {
    /// Create a new Ollama service
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self {
            base_url: "http://localhost:11434".to_string(),
            model_name: "llama2:7b".to_string(),
            process_handle: None,
        })
    }

    /// Start the Ollama service
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting Ollama service");

        // Check if Ollama is already running
        if self.is_healthy().await {
            info!("Ollama service already running");
            return Ok(());
        }

        // Start Ollama serve process
        match Command::new("ollama")
            .arg("serve")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(child) => {
                self.process_handle = Some(child);
                info!("Ollama service started");

                // Wait for service to be ready
                self.wait_for_startup().await?;
                Ok(())
            }
            Err(e) => {
                Err(format!("Failed to start Ollama service: {}", e).into())
            }
        }
    }

    /// Stop the Ollama service
    pub async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Stopping Ollama service");

        if let Some(mut handle) = self.process_handle.take() {
            match handle.kill() {
                Ok(_) => info!("Ollama service stopped"),
                Err(e) => warn!("Failed to kill Ollama process: {}", e),
            }
        }

        Ok(())
    }

    /// Check if Ollama service is healthy
    pub async fn is_healthy(&self) -> bool {
        match reqwest::get(&format!("{}/api/tags", self.base_url)).await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    /// Pull a model if not available
    pub async fn ensure_model(&self, model_name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Ensuring model {} is available", model_name);

        // Check if model is available
        if self.is_model_available(model_name).await? {
            info!("Model {} is already available", model_name);
            return Ok(());
        }

        // Pull the model
        info!("Pulling model {}", model_name);
        let output = Command::new("ollama")
            .args(&["pull", model_name])
            .output()?;

        if output.status.success() {
            info!("Successfully pulled model {}", model_name);
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Failed to pull model {}: {}", model_name, stderr).into())
        }
    }

    /// Check if a model is available
    pub async fn is_model_available(&self, model_name: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let response = reqwest::get(&format!("{}/api/tags", self.base_url)).await?;
        let tags: serde_json::Value = response.json().await?;

        if let Some(models) = tags["models"].as_array() {
            Ok(models.iter().any(|model| {
                model["name"].as_str().map(|name| name == model_name).unwrap_or(false)
            }))
        } else {
            Ok(false)
        }
    }

    /// Generate text using a model
    pub async fn generate(&self, model: &str, prompt: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();
        let request_body = serde_json::json!({
            "model": model,
            "prompt": prompt,
            "stream": false
        });

        let response = client
            .post(&format!("{}/api/generate", self.base_url))
            .json(&request_body)
            .send()
            .await?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            Ok(result["response"].as_str().unwrap_or("").to_string())
        } else {
            Err(format!("Ollama generation failed: {}", response.status()).into())
        }
    }

    /// Get service base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Get model name
    pub fn model_name(&self) -> &str {
        &self.model_name
    }

    /// Wait for Ollama service to start up
    async fn wait_for_startup(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let max_attempts = 30; // 30 seconds
        let delay = std::time::Duration::from_millis(1000);

        for attempt in 1..=max_attempts {
            if self.is_healthy().await {
                info!("Ollama service ready after {} attempts", attempt);
                return Ok(());
            }

            if attempt < max_attempts {
                tokio::time::sleep(delay).await;
            }
        }

        Err("Ollama service failed to start within timeout".into())
    }
}

impl Drop for OllamaService {
    fn drop(&mut self) {
        if let Some(mut handle) = self.process_handle.take() {
            let _ = handle.kill();
        }
    }
}
