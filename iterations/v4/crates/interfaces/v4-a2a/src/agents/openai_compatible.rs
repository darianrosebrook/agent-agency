//! OpenAI-Compatible Agent
//!
//! A generic A2A agent backed by any OpenAI-compatible chat completion API.
//! Works with MiniMax, DeepSeek, OpenRouter, Ollama, vLLM, and any provider
//! that implements the OpenAI `/v1/chat/completions` endpoint.
//!
//! This is the workhorse for delegating bulk tasks to cheap worker models.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::cost::CostMonitor;
use crate::error::A2AError;
use crate::handler::AgentHandler;
use crate::types::{
    AgentCapabilities, AgentCard, AgentProvider, AgentSkill, Artifact, CancelTaskRequest,
    GetTaskRequest, Message, Part, Role, SendMessageRequest, StreamEvent, Task, TaskState,
    TaskStatus,
};

// ============================================================================
// Configuration
// ============================================================================

/// Configuration for an OpenAI-compatible LLM provider.
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    /// Base URL for the API (e.g., `https://api.minimax.io/v1`).
    pub base_url: String,
    /// API key for authentication.
    pub api_key: String,
    /// Model identifier (e.g., `MiniMax-M2.5`, `deepseek-chat`).
    pub model: String,
    /// Display name for this provider (used in agent card).
    pub display_name: String,
    /// Maximum tokens to generate per request.
    pub max_tokens: u32,
    /// Temperature (0.0-1.0].
    pub temperature: f32,
    /// System prompt prepended to every request.
    pub system_prompt: Option<String>,
    /// Cost per million input tokens (for routing decisions).
    pub cost_per_m_input: Option<f64>,
    /// Cost per million output tokens (for routing decisions).
    pub cost_per_m_output: Option<f64>,
}

/// Pre-built configuration for MiniMax M2.5.
pub fn minimax_config(api_key: impl Into<String>) -> ProviderConfig {
    ProviderConfig {
        base_url: "https://api.minimax.io/v1".to_string(),
        api_key: api_key.into(),
        model: "MiniMax-M2.5".to_string(),
        display_name: "MiniMax M2.5".to_string(),
        max_tokens: 4096,
        temperature: 1.0,
        system_prompt: None,
        cost_per_m_input: Some(0.255),
        cost_per_m_output: Some(1.0),
    }
}

/// Pre-built configuration for MiniMax M2.5 highspeed (cheaper, faster).
pub fn minimax_highspeed_config(api_key: impl Into<String>) -> ProviderConfig {
    ProviderConfig {
        base_url: "https://api.minimax.io/v1".to_string(),
        api_key: api_key.into(),
        model: "MiniMax-M2.5-highspeed".to_string(),
        display_name: "MiniMax M2.5 Highspeed".to_string(),
        max_tokens: 4096,
        temperature: 1.0,
        system_prompt: None,
        cost_per_m_input: Some(0.255),
        cost_per_m_output: Some(1.0),
    }
}

/// Pre-built configuration for DeepSeek V3 via OpenRouter.
pub fn deepseek_openrouter_config(api_key: impl Into<String>) -> ProviderConfig {
    ProviderConfig {
        base_url: "https://openrouter.ai/api/v1".to_string(),
        api_key: api_key.into(),
        model: "deepseek/deepseek-chat-v3-0324".to_string(),
        display_name: "DeepSeek V3".to_string(),
        max_tokens: 4096,
        temperature: 0.7,
        system_prompt: None,
        cost_per_m_input: Some(0.14),
        cost_per_m_output: Some(0.28),
    }
}

/// Pre-built configuration for a local Ollama model.
pub fn ollama_config(model: impl Into<String>) -> ProviderConfig {
    ProviderConfig {
        base_url: "http://localhost:11434/v1".to_string(),
        api_key: "ollama".to_string(), // Ollama doesn't need a real key
        model: model.into(),
        display_name: "Ollama (local)".to_string(),
        max_tokens: 4096,
        temperature: 0.7,
        system_prompt: None,
        cost_per_m_input: Some(0.0), // Free — local inference
        cost_per_m_output: Some(0.0),
    }
}

// ============================================================================
// OpenAI API types (subset needed for chat completions)
// ============================================================================

#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    id: Option<String>,
    choices: Vec<ChatChoice>,
    #[serde(default)]
    usage: Option<UsageInfo>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatChoiceMessage,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChatChoiceMessage {
    #[serde(default)]
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UsageInfo {
    #[serde(default)]
    prompt_tokens: u32,
    #[serde(default)]
    completion_tokens: u32,
    #[serde(default)]
    total_tokens: u32,
}

// ============================================================================
// Agent implementation
// ============================================================================

/// Strip `<think>...</think>` reasoning blocks from model output.
///
/// Models like MiniMax M2.5 and DeepSeek R1 emit chain-of-thought reasoning
/// wrapped in think tags. We preserve the actual response and discard the reasoning.
fn strip_think_tags(text: &str) -> String {
    // Find the last </think> tag and return everything after it
    if let Some(end_pos) = text.rfind("</think>") {
        let after = &text[end_pos + "</think>".len()..];
        let trimmed = after.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    // If no think tags or nothing after them, check for opening tag only (incomplete)
    if let Some(start_pos) = text.find("<think>") {
        let before = text[..start_pos].trim();
        if !before.is_empty() {
            return before.to_string();
        }
    }

    // No think tags at all — return as-is
    text.to_string()
}

/// Usage statistics for cost tracking.
#[derive(Debug, Clone, Default)]
pub struct UsageStats {
    /// Total input tokens consumed.
    pub total_input_tokens: u64,
    /// Total output tokens consumed.
    pub total_output_tokens: u64,
    /// Total requests made.
    pub total_requests: u64,
    /// Estimated cost in USD.
    pub estimated_cost_usd: f64,
}

/// An A2A agent backed by an OpenAI-compatible chat completion API.
///
/// This wraps any provider (MiniMax, DeepSeek, OpenRouter, Ollama, etc.)
/// and exposes it as a standard A2A agent that the orchestrator can delegate to.
pub struct OpenAICompatibleAgent {
    config: ProviderConfig,
    card: AgentCard,
    http: reqwest::Client,
    tasks: RwLock<HashMap<String, Task>>,
    usage: RwLock<UsageStats>,
    cost_monitor: Option<Arc<CostMonitor>>,
}

impl OpenAICompatibleAgent {
    /// Create a new agent from a provider configuration and a list of skills.
    pub fn new(config: ProviderConfig, skills: Vec<AgentSkill>) -> Self {
        let card = AgentCard {
            name: config.display_name.clone(),
            url: String::new(), // Set by the server when it starts
            version: "0.1.0".to_string(),
            skills,
            description: Some(format!(
                "A2A worker agent backed by {} (model: {})",
                config.display_name, config.model
            )),
            documentation_url: None,
            provider: Some(AgentProvider {
                name: "Agent Agency".to_string(),
                url: None,
                support_contact: None,
            }),
            capabilities: Some(AgentCapabilities {
                a2a_version: Some("0.3".to_string()),
                streaming: Some(false), // TODO: add streaming support
                push_notifications: Some(false),
                state_transition_history: Some(true),
            }),
            security_schemes: None,
            security: None,
            default_input_modes: vec!["text".to_string()],
            default_output_modes: vec!["text".to_string()],
            supports_authenticated_extended_card: None,
        };

        Self {
            config,
            card,
            http: reqwest::Client::new(),
            tasks: RwLock::new(HashMap::new()),
            usage: RwLock::new(UsageStats::default()),
            cost_monitor: None,
        }
    }

    /// Set a CostMonitor for centralized cost tracking
    pub fn with_cost_monitor(mut self, monitor: Arc<CostMonitor>) -> Self {
        self.cost_monitor = Some(monitor);
        self
    }

    /// Get current usage statistics.
    pub fn usage_stats(&self) -> UsageStats {
        self.usage.read().unwrap().clone()
    }

    /// Get the provider configuration.
    pub fn config(&self) -> &ProviderConfig {
        &self.config
    }

    /// Call the chat completion API.
    async fn chat_completion(
        &self,
        messages: Vec<ChatMessage>,
    ) -> Result<(String, Option<UsageInfo>), A2AError> {
        let url = format!("{}/chat/completions", self.config.base_url);

        let request = ChatCompletionRequest {
            model: self.config.model.clone(),
            messages,
            max_tokens: Some(self.config.max_tokens),
            temperature: Some(self.config.temperature),
            stream: Some(false),
        };

        tracing::debug!(
            model = %self.config.model,
            url = %url,
            "Calling chat completion API"
        );

        let response = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| A2AError::Http(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown".to_string());
            return Err(A2AError::Http(format!(
                "Chat completion failed ({}): {}",
                status, body
            )));
        }

        let completion: ChatCompletionResponse = response
            .json()
            .await
            .map_err(|e| A2AError::Http(e.to_string()))?;

        let raw_content = completion
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();

        // Strip <think>...</think> reasoning blocks from models that emit them
        let content = strip_think_tags(&raw_content);

        // Track usage
        if let Some(ref usage) = completion.usage {
            let mut stats = self.usage.write().unwrap();
            stats.total_input_tokens += u64::from(usage.prompt_tokens);
            stats.total_output_tokens += u64::from(usage.completion_tokens);
            stats.total_requests += 1;

            if let (Some(input_cost), Some(output_cost)) =
                (self.config.cost_per_m_input, self.config.cost_per_m_output)
            {
                stats.estimated_cost_usd += (f64::from(usage.prompt_tokens) / 1_000_000.0)
                    * input_cost
                    + (f64::from(usage.completion_tokens) / 1_000_000.0) * output_cost;
            }

            // Record to centralized CostMonitor if configured
            if let Some(ref monitor) = self.cost_monitor {
                let provider_name = self.config.display_name.to_lowercase().replace(' ', "-");
                monitor.record_usage(
                    &provider_name,
                    u64::from(usage.prompt_tokens),
                    u64::from(usage.completion_tokens),
                );
            }
        }

        Ok((content, completion.usage))
    }

    /// Convert A2A message parts to chat messages.
    fn to_chat_messages(&self, message: &Message) -> Vec<ChatMessage> {
        let mut messages = Vec::new();

        // Add system prompt if configured
        if let Some(ref system) = self.config.system_prompt {
            messages.push(ChatMessage {
                role: "system".to_string(),
                content: system.clone(),
            });
        }

        // Convert A2A parts to a single content string
        let content: String = message
            .parts
            .iter()
            .filter_map(|part| match part {
                Part::Text { text, .. } => Some(text.clone()),
                Part::Data { data, .. } => {
                    Some(serde_json::to_string_pretty(data).unwrap_or_default())
                }
                Part::File { .. } => Some("[file content omitted]".to_string()),
            })
            .collect::<Vec<_>>()
            .join("\n");

        let role = match message.role {
            Role::User => "user",
            Role::Agent => "assistant",
        };

        messages.push(ChatMessage {
            role: role.to_string(),
            content,
        });

        messages
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
impl AgentHandler for OpenAICompatibleAgent {
    fn agent_card(&self) -> &AgentCard {
        &self.card
    }

    async fn handle_message(&self, request: SendMessageRequest) -> Result<Task, A2AError> {
        let task_id = request
            .message
            .task_id
            .clone()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let context_id = request.message.context_id.clone();

        // Convert to chat messages and call the API
        let chat_messages = self.to_chat_messages(&request.message);
        let (response_text, _usage) = self.chat_completion(chat_messages).await?;

        let task = Task {
            id: task_id,
            context_id,
            status: TaskStatus {
                state: TaskState::Completed,
                message: Some(Message::text(Role::Agent, &response_text)),
                timestamp: Some(chrono::Utc::now()),
            },
            kind: "task".to_string(),
            history: Some(vec![request.message]),
            artifacts: Some(vec![Artifact {
                artifact_id: uuid::Uuid::new_v4().to_string(),
                parts: vec![Part::Text {
                    text: response_text,
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

    async fn handle_message_stream(
        &self,
        request: SendMessageRequest,
    ) -> Result<Vec<StreamEvent>, A2AError> {
        // For now, fall back to non-streaming and wrap in events
        let task = self.handle_message(request).await?;
        let context_id = task.context_id.clone().unwrap_or_default();

        let mut events = vec![StreamEvent::StatusUpdate {
            task_id: task.id.clone(),
            context_id: context_id.clone(),
            status: TaskStatus {
                state: TaskState::Working,
                message: None,
                timestamp: Some(chrono::Utc::now()),
            },
            is_final: false,
            metadata: None,
        }];

        if let Some(ref artifacts) = task.artifacts {
            for artifact in artifacts {
                events.push(StreamEvent::ArtifactUpdate {
                    task_id: task.id.clone(),
                    context_id: context_id.clone(),
                    artifact: artifact.clone(),
                    append: Some(false),
                    last_chunk: Some(true),
                    metadata: None,
                });
            }
        }

        events.push(StreamEvent::StatusUpdate {
            task_id: task.id.clone(),
            context_id,
            status: task.status,
            is_final: true,
            metadata: None,
        });

        Ok(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimax_config() {
        let config = minimax_config("test-key");
        assert_eq!(config.base_url, "https://api.minimax.io/v1");
        assert_eq!(config.model, "MiniMax-M2.5");
        assert!(config.cost_per_m_input.is_some());
    }

    #[test]
    fn test_deepseek_config() {
        let config = deepseek_openrouter_config("test-key");
        assert!(config.base_url.contains("openrouter"));
        assert!(config.model.contains("deepseek"));
    }

    #[test]
    fn test_ollama_config() {
        let config = ollama_config("llama3.2");
        assert_eq!(config.base_url, "http://localhost:11434/v1");
        assert_eq!(config.cost_per_m_input, Some(0.0));
    }

    #[test]
    fn test_agent_card_generation() {
        let config = minimax_config("test-key");
        let agent = OpenAICompatibleAgent::new(
            config,
            vec![AgentSkill {
                id: "draft".to_string(),
                name: "Draft Content".to_string(),
                description: "Generate draft text".to_string(),
                tags: None,
                examples: None,
                input_modes: None,
                output_modes: None,
            }],
        );

        let card = agent.agent_card();
        assert!(card.name.contains("MiniMax"));
        assert_eq!(card.skills.len(), 1);
    }

    #[test]
    fn test_message_conversion() {
        let mut config = minimax_config("test-key");
        config.system_prompt = Some("You are a helpful assistant.".to_string());

        let agent = OpenAICompatibleAgent::new(config, vec![]);

        let message = Message::text(Role::User, "Hello");
        let chat_messages = agent.to_chat_messages(&message);

        assert_eq!(chat_messages.len(), 2); // system + user
        assert_eq!(chat_messages[0].role, "system");
        assert_eq!(chat_messages[1].role, "user");
        assert_eq!(chat_messages[1].content, "Hello");
    }

    #[test]
    fn test_message_conversion_no_system() {
        let config = minimax_config("test-key");
        let agent = OpenAICompatibleAgent::new(config, vec![]);

        let message = Message::text(Role::User, "Hello");
        let chat_messages = agent.to_chat_messages(&message);

        assert_eq!(chat_messages.len(), 1); // just user
        assert_eq!(chat_messages[0].role, "user");
    }

    #[test]
    fn test_usage_stats_default() {
        let config = minimax_config("test-key");
        let agent = OpenAICompatibleAgent::new(config, vec![]);
        let stats = agent.usage_stats();
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.estimated_cost_usd, 0.0);
    }

    #[test]
    fn test_data_part_conversion() {
        let config = minimax_config("test-key");
        let agent = OpenAICompatibleAgent::new(config, vec![]);

        let message = Message {
            message_id: "test".to_string(),
            role: Role::User,
            parts: vec![
                Part::Text {
                    text: "Here is some data:".to_string(),
                    metadata: None,
                },
                Part::Data {
                    data: {
                        let mut m = HashMap::new();
                        m.insert("key".to_string(), serde_json::json!("value"));
                        m
                    },
                    metadata: None,
                },
            ],
            kind: "message".to_string(),
            task_id: None,
            context_id: None,
            reference_task_ids: None,
            metadata: None,
        };

        let chat_messages = agent.to_chat_messages(&message);
        assert_eq!(chat_messages.len(), 1);
        assert!(chat_messages[0].content.contains("Here is some data:"));
        assert!(chat_messages[0].content.contains("key"));
    }

    #[test]
    fn test_strip_think_tags_basic() {
        let input = "<think>Let me reason about this...</think>Here is the answer.";
        assert_eq!(strip_think_tags(input), "Here is the answer.");
    }

    #[test]
    fn test_strip_think_tags_multiline_reasoning() {
        let input = "<think>\nStep 1: Consider the problem\nStep 2: Formulate response\n</think>\n\nThe answer is 42.";
        assert_eq!(strip_think_tags(input), "The answer is 42.");
    }

    #[test]
    fn test_strip_think_tags_no_tags() {
        let input = "Just a normal response with no reasoning.";
        assert_eq!(strip_think_tags(input), input);
    }

    #[test]
    fn test_strip_think_tags_empty_think() {
        let input = "<think></think>Clean output.";
        assert_eq!(strip_think_tags(input), "Clean output.");
    }

    #[test]
    fn test_strip_think_tags_only_think_no_content() {
        // If the entire response is inside think tags, return as-is
        let input = "<think>Only reasoning, no actual answer</think>";
        assert_eq!(strip_think_tags(input), input);
    }

    #[test]
    fn test_strip_think_tags_incomplete_open_only() {
        // Model started thinking but never closed — return content before the tag
        let input = "Some preamble<think>reasoning that never ends...";
        assert_eq!(strip_think_tags(input), "Some preamble");
    }

    #[test]
    fn test_strip_think_tags_multiple_blocks() {
        // Multiple think blocks — take everything after the last </think>
        let input = "<think>first</think>middle<think>second</think>Final answer.";
        assert_eq!(strip_think_tags(input), "Final answer.");
    }

    #[test]
    fn test_with_cost_monitor() {
        let monitor = Arc::new(CostMonitor::new());
        let config = minimax_config("test-key");
        let agent = OpenAICompatibleAgent::new(config, vec![]).with_cost_monitor(monitor.clone());

        // CostMonitor is set
        assert!(agent.cost_monitor.is_some());

        // Initial cost is zero
        assert_eq!(monitor.total_estimated_cost(), 0.0);
    }
}
