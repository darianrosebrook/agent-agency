//! Local MLX Agent
//!
//! An A2A agent backed by local MLX inference on Apple Silicon.
//! Uses v4-inference's InferenceService for zero-cost, on-device generation.
//!
//! ## Usage
//!
//! ```sh
//! PROVIDER=local MODEL_PATH=/path/to/model cargo run --bin a2a_worker
//! ```

use std::collections::HashMap;
use std::sync::RwLock;

use async_trait::async_trait;
use v4_inference::{InferenceConfig, InferenceRequest, InferenceService};

use crate::error::A2AError;
use crate::handler::AgentHandler;
use crate::types::{
    AgentCapabilities, AgentCard, AgentProvider, AgentSkill, Artifact, CancelTaskRequest,
    GetTaskRequest, Message, Part, Role, SendMessageRequest, Task, TaskState,
    TaskStatus,
};

/// Configuration for the local MLX agent.
#[derive(Debug, Clone)]
pub struct LocalMLXConfig {
    /// Path to local model directory (or HuggingFace model ID).
    pub model_path: String,
    /// Display name for this local model.
    pub display_name: String,
    /// Maximum tokens to generate per request.
    pub max_tokens: u32,
    /// Default temperature.
    pub temperature: f32,
    /// System prompt prepended to every request.
    pub system_prompt: Option<String>,
    /// Inference timeout in seconds.
    pub timeout_secs: u64,
}

impl Default for LocalMLXConfig {
    fn default() -> Self {
        Self {
            model_path: String::new(),
            display_name: "Local MLX".to_string(),
            max_tokens: 2048,
            temperature: 0.7,
            system_prompt: None,
            timeout_secs: 120,
        }
    }
}

/// Build a LocalMLXConfig from environment variables.
pub fn local_mlx_config_from_env() -> LocalMLXConfig {
    let model_path = std::env::var("MODEL_PATH").unwrap_or_default();
    let display_name = std::env::var("MODEL_NAME")
        .unwrap_or_else(|_| "Local MLX".to_string());
    let max_tokens = std::env::var("MAX_TOKENS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(2048);
    let temperature = std::env::var("TEMPERATURE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0.7);
    let system_prompt = std::env::var("SYSTEM_PROMPT").ok();
    let timeout_secs = std::env::var("INFERENCE_TIMEOUT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(120);

    LocalMLXConfig {
        model_path,
        display_name,
        max_tokens,
        temperature,
        system_prompt,
        timeout_secs,
    }
}

/// An A2A agent backed by local MLX inference.
///
/// This wraps v4-inference's InferenceService, providing zero-cost local
/// inference on Apple Silicon hardware. The agent exposes the same skills
/// as remote workers but runs entirely on-device.
pub struct LocalMLXAgent {
    config: LocalMLXConfig,
    card: AgentCard,
    service: InferenceService,
    tasks: RwLock<HashMap<String, Task>>,
    model_loaded: RwLock<bool>,
}

impl LocalMLXAgent {
    /// Create a new local MLX agent.
    pub fn new(config: LocalMLXConfig, skills: Vec<AgentSkill>) -> Self {
        let inference_config = if config.model_path.is_empty() {
            // No model path — use mock for development
            tracing::warn!("No MODEL_PATH set, using mock inference provider");
            InferenceConfig::mock()
        } else {
            InferenceConfig::mlx(&config.model_path)
        };

        let card = AgentCard {
            name: config.display_name.clone(),
            url: String::new(),
            version: "0.1.0".to_string(),
            skills,
            description: Some(format!(
                "Local MLX inference agent (model: {})",
                if config.model_path.is_empty() {
                    "mock"
                } else {
                    &config.model_path
                }
            )),
            documentation_url: None,
            provider: Some(AgentProvider {
                name: "Agent Agency (Local)".to_string(),
                url: None,
                support_contact: None,
            }),
            capabilities: Some(AgentCapabilities {
                a2a_version: Some("0.3".to_string()),
                streaming: Some(false),
                push_notifications: Some(false),
                state_transition_history: Some(true),
            }),
            security_schemes: None,
            security: None,
            default_input_modes: vec!["text".to_string()],
            default_output_modes: vec!["text".to_string()],
            supports_authenticated_extended_card: None,
        };

        let service = InferenceService::new(inference_config);

        Self {
            config,
            card,
            service,
            tasks: RwLock::new(HashMap::new()),
            model_loaded: RwLock::new(false),
        }
    }

    /// Load the model. Must be called before inference.
    pub async fn load_model(&self) -> Result<(), A2AError> {
        self.service
            .load_model()
            .await
            .map_err(|e| A2AError::Internal(format!("Failed to load model: {e}")))?;

        *self.model_loaded.write().unwrap() = true;
        tracing::info!(
            model = %self.config.model_path,
            "Local MLX model loaded"
        );
        Ok(())
    }

    /// Check if the model is loaded.
    pub fn is_model_loaded(&self) -> bool {
        *self.model_loaded.read().unwrap()
    }

    /// Extract text content from A2A message parts.
    fn extract_prompt(&self, message: &Message) -> String {
        let mut parts = Vec::new();

        // Prepend system prompt if configured
        if let Some(ref system) = self.config.system_prompt {
            parts.push(format!("System: {system}"));
        }

        // Extract text from message parts
        for part in &message.parts {
            match part {
                Part::Text { text, .. } => parts.push(text.clone()),
                Part::Data { data, .. } => {
                    if let Ok(json) = serde_json::to_string_pretty(data) {
                        parts.push(json);
                    }
                }
                Part::File { .. } => parts.push("[file content omitted]".to_string()),
            }
        }

        parts.join("\n")
    }

    /// Store a task and return it.
    fn store_task(&self, task: Task) -> Task {
        self.tasks
            .write()
            .unwrap()
            .insert(task.id.clone(), task.clone());
        task
    }
}

#[async_trait]
impl AgentHandler for LocalMLXAgent {
    fn agent_card(&self) -> &AgentCard {
        &self.card
    }

    async fn handle_message(&self, request: SendMessageRequest) -> Result<Task, A2AError> {
        // Check model is loaded
        if !self.is_model_loaded() {
            return Err(A2AError::Internal(
                "Model not loaded. Call load_model() before sending messages.".to_string(),
            ));
        }

        let task_id = request
            .message
            .task_id
            .clone()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let context_id = request.message.context_id.clone();

        // Build inference request
        let prompt = self.extract_prompt(&request.message);
        let inference_request = InferenceRequest::new(&prompt)
            .with_max_tokens(self.config.max_tokens)
            .with_temperature(self.config.temperature);

        // Run inference
        let response = self
            .service
            .infer(inference_request)
            .await
            .map_err(|e| A2AError::Internal(format!("Inference failed: {e}")))?;

        tracing::debug!(
            tokens = response.tokens_generated,
            tok_per_sec = response.tokens_per_second,
            "Local inference completed"
        );

        let task = Task {
            id: task_id,
            context_id,
            status: TaskStatus {
                state: TaskState::Completed,
                message: Some(Message::text(Role::Agent, &response.text)),
                timestamp: Some(chrono::Utc::now()),
            },
            kind: "task".to_string(),
            history: Some(vec![request.message]),
            artifacts: Some(vec![Artifact {
                artifact_id: uuid::Uuid::new_v4().to_string(),
                parts: vec![Part::Text {
                    text: response.text,
                    metadata: None,
                }],
                name: None,
                description: None,
                metadata: None,
            }]),
            metadata: None,
        };

        Ok(self.store_task(task))
    }

    async fn get_task(&self, request: GetTaskRequest) -> Result<Task, A2AError> {
        self.tasks
            .read()
            .unwrap()
            .get(&request.id)
            .cloned()
            .ok_or_else(|| A2AError::TaskNotFound(request.id))
    }

    async fn cancel_task(&self, request: CancelTaskRequest) -> Result<Task, A2AError> {
        let mut tasks = self.tasks.write().unwrap();
        let task = tasks
            .get_mut(&request.id)
            .ok_or_else(|| A2AError::TaskNotFound(request.id.clone()))?;

        if task.status.state.is_terminal() {
            return Err(A2AError::TaskNotCancelable(request.id));
        }

        task.status = TaskStatus {
            state: TaskState::Canceled,
            message: None,
            timestamp: Some(chrono::Utc::now()),
        };

        Ok(task.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_skills() -> Vec<AgentSkill> {
        vec![AgentSkill {
            id: "draft-content".to_string(),
            name: "Draft Content".to_string(),
            description: "Generate draft text content".to_string(),
            tags: None,
            examples: None,
            input_modes: None,
            output_modes: None,
        }]
    }

    #[test]
    fn test_agent_card_zero_cost() {
        let config = LocalMLXConfig::default();
        let agent = LocalMLXAgent::new(config, test_skills());
        let card = agent.agent_card();
        assert!(card.name.contains("Local"));
        assert!(card.description.as_ref().unwrap().contains("mock"));
    }

    #[test]
    fn test_agent_card_with_model_path() {
        let config = LocalMLXConfig {
            model_path: "/tmp/my-model".to_string(),
            display_name: "Mistral 7B".to_string(),
            ..Default::default()
        };
        let agent = LocalMLXAgent::new(config, test_skills());
        let card = agent.agent_card();
        assert_eq!(card.name, "Mistral 7B");
        assert!(card.description.as_ref().unwrap().contains("/tmp/my-model"));
    }

    #[test]
    fn test_extract_prompt_basic() {
        let config = LocalMLXConfig::default();
        let agent = LocalMLXAgent::new(config, vec![]);
        let message = Message::text(Role::User, "Hello world");
        let prompt = agent.extract_prompt(&message);
        assert_eq!(prompt, "Hello world");
    }

    #[test]
    fn test_extract_prompt_with_system() {
        let config = LocalMLXConfig {
            system_prompt: Some("You are helpful.".to_string()),
            ..Default::default()
        };
        let agent = LocalMLXAgent::new(config, vec![]);
        let message = Message::text(Role::User, "Hello");
        let prompt = agent.extract_prompt(&message);
        assert!(prompt.contains("System: You are helpful."));
        assert!(prompt.contains("Hello"));
    }

    #[test]
    fn test_not_loaded_by_default() {
        let config = LocalMLXConfig::default();
        let agent = LocalMLXAgent::new(config, vec![]);
        assert!(!agent.is_model_loaded());
    }

    #[tokio::test]
    async fn test_error_when_model_not_loaded() {
        let config = LocalMLXConfig::default();
        let agent = LocalMLXAgent::new(config, vec![]);

        let request = SendMessageRequest {
            message: Message::text(Role::User, "Hello"),
            configuration: None,
        };

        let result = agent.handle_message(request).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("not loaded"));
    }

    #[tokio::test]
    async fn test_end_to_end_mock() {
        let config = LocalMLXConfig::default();
        let agent = LocalMLXAgent::new(config, test_skills());

        // Load mock model
        agent.load_model().await.unwrap();
        assert!(agent.is_model_loaded());

        // Send a message
        let request = SendMessageRequest {
            message: Message::text(Role::User, "Write a haiku"),
            configuration: None,
        };

        let task = agent.handle_message(request).await.unwrap();
        assert_eq!(task.status.state, TaskState::Completed);
        assert!(task.artifacts.is_some());

        let artifacts = task.artifacts.unwrap();
        assert!(!artifacts.is_empty());
        if let Part::Text { ref text, .. } = artifacts[0].parts[0] {
            assert!(!text.is_empty());
        }
    }

    #[tokio::test]
    async fn test_get_task_after_message() {
        let config = LocalMLXConfig::default();
        let agent = LocalMLXAgent::new(config, test_skills());
        agent.load_model().await.unwrap();

        let request = SendMessageRequest {
            message: Message::text(Role::User, "Test"),
            configuration: None,
        };
        let task = agent.handle_message(request).await.unwrap();

        let retrieved = agent
            .get_task(GetTaskRequest {
                id: task.id.clone(),
                history_length: None,
            })
            .await
            .unwrap();
        assert_eq!(retrieved.id, task.id);
    }

    #[tokio::test]
    async fn test_task_not_found() {
        let config = LocalMLXConfig::default();
        let agent = LocalMLXAgent::new(config, vec![]);
        let result = agent
            .get_task(GetTaskRequest {
                id: "nonexistent".to_string(),
                history_length: None,
            })
            .await;
        assert!(matches!(result, Err(A2AError::TaskNotFound(_))));
    }

    #[test]
    fn test_config_from_env_defaults() {
        // With no env vars set, should produce defaults
        let config = local_mlx_config_from_env();
        assert!(config.model_path.is_empty() || !config.model_path.is_empty()); // just testing it doesn't panic
        assert_eq!(config.max_tokens, 2048);
    }
}
