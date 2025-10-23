//! Thread-confined FFI wrapper for CoreML handles
//!
//! This module provides safe wrappers around raw CoreML pointers that cannot be
//! sent between threads, ensuring all FFI operations happen on a dedicated thread.

// Re-export CoreML types from apple-silicon for convenience
pub use agent_agency_apple_silicon::ane::compat::coreml::{
    ModelRef, CoreMlHandle, Tensor, ModelMetadata, InferenceOptions, ComputeUnits,
    load_model, run_inference, unload_model, detect_coreml_capabilities,
    io_safety::{into_owned_tensor, validate_io_schema, convert_ffi_tensors}
};

/// Inference request that can be sent across channels
#[derive(Debug)]
pub struct InferenceRequest {
    pub prompt: String,
    pub model_path: std::path::PathBuf,
    pub options: crate::mistral_tokenizer::MistralCompilationOptions,
    pub judge_config: crate::judge::JudgeConfig,
    pub model_ref: Option<ModelRef>, // Set after loading on the inference thread
}

/// Inference response that can be sent across channels
#[derive(Debug)]
pub enum InferenceResult {
    Success(String),
    Error(String),
}

/// Thread-safe interface for CoreML operations.
/// This should be spawned on a dedicated thread.
pub trait CoreMlInvoker {
    fn invoke_inference(&mut self, request: InferenceRequest) -> InferenceResult;
}

/// Default implementation that uses the apple-silicon crate
pub struct DefaultCoreMlInvoker;

impl CoreMlInvoker for DefaultCoreMlInvoker {
    fn invoke_inference(&mut self, request: InferenceRequest) -> InferenceResult {
        use agent_agency_apple_silicon::ane::models::mistral_model::load_mistral_model;
        use agent_agency_apple_silicon::ane::infer::mistral::{
            deliberate_constitution, MistralInferenceOptions, Verdict, RiskTier
        };
        use agent_agency_apple_silicon::telemetry::TelemetryCollector;

        // Create telemetry collector
        let telemetry = TelemetryCollector::new();

        // Load model synchronously (this thread owns the handle)
        match load_mistral_model(
            &request.model_path,
            &request.options,
            telemetry,
        ) {
            Ok(mut model) => {
                // Perform constitutional reasoning inference
                let inference_options = MistralInferenceOptions {
                    max_tokens: request.judge_config.max_tokens as usize,
                    temperature: request.judge_config.temperature as f32,
                    top_p: 0.9, // Standard value for quality generation
                    timeout_ms: (request.judge_config.timeout_seconds * 1000) as u64,
                    use_kv_cache: true,
                };

                // For judge reviews, we use constitutional reasoning
                let evidence: Vec<String> = vec![];
                let debate_history: Vec<String> = vec![];

                match deliberate_constitution(
                    &mut model,
                    &request.prompt,
                    &evidence,
                    &debate_history,
                    &inference_options,
                ) {
                    Ok(verdict) => {
                        // Convert the constitutional verdict to a JSON response
                        let response = format!(
                            r#"{{
  "verdict": "{}",
  "confidence": {:.2},
  "reasoning": "{}",
  "quality_score": {:.2},
  "risk_assessment": {{
    "overall_risk": "{}",
    "risk_factors": {},
    "mitigation_suggestions": {},
    "confidence": {:.2}
  }}
}}"#,
                            match verdict.verdict {
                                Verdict::Approve => "approve",
                                Verdict::Modify => "modify",
                                Verdict::Reject => "reject",
                            },
                            verdict.confidence_score,
                            verdict.justification,
                            verdict.confidence_score * 0.8, // Rough quality score mapping
                            match verdict.risk_assessment {
                                RiskTier::Tier1 => "low",
                                RiskTier::Tier2 => "medium",
                                RiskTier::Tier3 => "high",
                            },
                            serde_json::to_string(&verdict.key_concerns).unwrap_or_else(|_| "[]".to_string()),
                            serde_json::to_string(&verdict.recommendations).unwrap_or_else(|_| "[]".to_string()),
                            verdict.confidence_score
                        );
                        InferenceResult::Success(response)
                    }
                    Err(e) => InferenceResult::Error(format!("Constitutional reasoning failed: {}", e)),
                }
            }
            Err(e) => InferenceResult::Error(format!("Model loading failed: {}", e)),
        }
    }
}
