//! Channel-based bridge to thread-confined CoreML operations
//!
//! This module provides a Send/Sync interface to CoreML inference that delegates
//! to a dedicated thread holding the non-Send CoreML handles.

use std::sync::Arc;
use tokio::sync::oneshot;
use crossbeam::channel::{self, Receiver, Sender};

use crate::error::{CouncilError, CouncilResult};
use crate::judge::JudgeVerdict;

/// Opaque model reference that can be sent across threads
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModelRef(u64);

impl ModelRef {
    /// Create a new unique model reference
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

/// Request for CoreML inference operation
#[derive(Debug, Clone)]
pub struct InferenceRequest {
    pub prompt: String,
    pub model_path: std::path::PathBuf,
    pub judge_config: crate::judge::JudgeConfig,
    pub model_ref: Option<ModelRef>,
}

/// Result of CoreML inference operation
#[derive(Debug)]
pub enum InferenceResult {
    Success(String),
    Error(String),
}

/// Trait for CoreML inference operations
pub trait CoreMlInvoker {
    fn invoke_inference(&mut self, request: InferenceRequest) -> InferenceResult;
}

/// Default placeholder implementation for CoreML operations
/// This will be replaced with actual CoreML integration
pub struct DefaultCoreMlInvoker;

impl CoreMlInvoker for DefaultCoreMlInvoker {
    fn invoke_inference(&mut self, request: InferenceRequest) -> InferenceResult {
        // Placeholder implementation - return a mock response
        tracing::warn!("Using placeholder CoreML inference for prompt: {}", request.prompt);
        InferenceResult::Success("Mock CoreML response - integration pending".to_string())
    }
}

/// Channel message for inference operations
pub struct InferenceMessage {
    pub request: InferenceRequest,
    pub response_tx: oneshot::Sender<CouncilResult<String>>,
}

/// Thread-safe client for CoreML inference operations.
/// All operations are delegated to a dedicated thread via channels.
#[derive(Clone, Debug)]
pub struct ModelClient {
    tx: Sender<InferenceMessage>,
}

impl ModelClient {
    /// Create a new model client with a dedicated invoker thread
    pub fn new() -> Self {
        let (tx, rx) = channel::unbounded();
        Self::spawn_invoker_thread(rx);
        Self { tx }
    }

    /// Spawn the dedicated thread that handles CoreML operations
    fn spawn_invoker_thread(rx: Receiver<InferenceMessage>) {
        std::thread::spawn(move || {
            let mut invoker = DefaultCoreMlInvoker;
            tracing::info!("CoreML invoker thread started");

            while let Ok(message) = rx.recv() {
                let result = invoker.invoke_inference(message.request);

                // Convert InferenceResult to CouncilResult<String>
                let council_result = match result {
                    InferenceResult::Success(text) => Ok(text),
                    InferenceResult::Error(msg) => Err(CouncilError::JudgeError {
                        judge_id: "mistral-judge".to_string(),
                        message: msg,
                    }),
                };

                // Send response back (ignore send errors - receiver may have dropped)
                let _ = message.response_tx.send(council_result);
            }

            tracing::info!("CoreML invoker thread shutting down");
        });
    }

    /// Queue an inference operation and wait for the result
    pub async fn enqueue_inference(
        &self,
        request: InferenceRequest,
    ) -> CouncilResult<String> {
        let (response_tx, response_rx) = oneshot::channel();

        let message = InferenceMessage {
            request,
            response_tx,
        };

        // Send request to the invoker thread
        self.tx
            .send(message)
            .map_err(|_| CouncilError::JudgeError {
                judge_id: "mistral-judge".to_string(),
                message: "Failed to enqueue inference request".to_string(),
            })?;

        // Wait for response
        response_rx
            .await
            .map_err(|_| CouncilError::JudgeError {
                judge_id: "mistral-judge".to_string(),
                message: "Inference request was dropped".to_string(),
            })?
    }
}

impl Default for ModelClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension trait for converting judge results to inference requests
pub trait ToInferenceRequest {
    fn to_inference_request(&self, judge_config: &crate::judge::JudgeConfig) -> InferenceRequest;
}

impl ToInferenceRequest for &str {
    fn to_inference_request(&self, judge_config: &crate::judge::JudgeConfig) -> InferenceRequest {
        InferenceRequest {
            prompt: self.to_string(),
            model_path: std::path::Path::new("models/mistral/Mistral-7B-Instruct-v0.3.mlpackage")
                .to_path_buf(),
            judge_config: judge_config.clone(),
            model_ref: None, // Will be set when model is loaded
        }
    }
}
