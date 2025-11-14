//! Mistral LLM inference execution
//!
//! This module provides Mistral model inference capabilities including
//! constitutional reasoning, debate generation, and text generation.

use crate::ane::ane_errors::{ANEError, Result};
use crate::ane::models::mistral_model::{reasoning_templates, MistralModel};
use crate::ane::policy::{BackendPolicy, PerformancePolicy, TaskType};
use candle_core::{Device, IndexOp, Tensor};
use rand::Rng;
use schemars::JsonSchema;

/// Inference options for Mistral models
#[derive(Debug, Clone, JsonSchema)]
pub struct MistralInferenceOptions {
    pub max_tokens: usize,
    pub temperature: Option<f32>, // None = greedy sampling
    pub top_p: Option<f32>,       // None = no top-p filtering
    pub timeout_ms: u64,
    pub use_kv_cache: bool,
    /// Sequence length for input (policy-recommended if None)
    pub sequence_length: Option<usize>,
    /// Task type for policy-based optimization (auto-detected if None)
    pub task_type: Option<TaskType>,
    /// Backend policy (auto-selected if None)
    pub backend_policy: Option<BackendPolicy>,
}

impl Default for MistralInferenceOptions {
    fn default() -> Self {
        Self {
            max_tokens: 100,
            temperature: Some(0.7), // Enable temperature sampling
            top_p: Some(0.9),       // Enable top-p sampling
            timeout_ms: 30000,      // 30 seconds
            use_kv_cache: true,
            sequence_length: None,  // Will use policy recommendation
            task_type: None,        // Will auto-detect from input
            backend_policy: None,   // Will use policy recommendation
        }
    }
}

impl MistralInferenceOptions {
    /// Apply performance policy to determine optimal sequence length and backend
    ///
    /// This integrates the ANE performance policy system to automatically select
    /// optimal sequence length and backend based on task characteristics and
    /// benchmark findings.
    ///
    /// # Arguments
    /// * `input_length` - Length of input tokens
    /// * `policy` - Performance policy (uses default if None)
    ///
    /// # Returns
    /// Updated options with policy-recommended sequence length and backend
    pub fn with_policy(mut self, input_length: usize, policy: Option<&PerformancePolicy>) -> Self {
        // Create default policy if not provided (owned, not borrowed)
        let default_policy = PerformancePolicy::default();
        let policy = policy.unwrap_or(&default_policy);
        
        // Auto-detect task type if not set
        let task_type = self.task_type.unwrap_or_else(|| {
            TaskType::from_input(input_length, self.max_tokens)
        });
        
        // Get policy-recommended sequence length if not set
        if self.sequence_length.is_none() {
            self.sequence_length = Some(policy.recommended_sequence_length(task_type));
        }
        
        // Get policy-recommended backend if not set
        if self.backend_policy.is_none() {
            let seq_len = self.sequence_length.unwrap_or(policy.sequence_length.default);
            self.backend_policy = Some(policy.recommended_backend(seq_len));
        }
        
        self
    }
    
    /// Get effective sequence length (policy-recommended or explicit)
    pub fn effective_sequence_length(&self, input_length: usize) -> usize {
        self.sequence_length.unwrap_or_else(|| {
            let policy = PerformancePolicy::default();
            let task_type = self.task_type.unwrap_or_else(|| {
                TaskType::from_input(input_length, self.max_tokens)
            });
            policy.recommended_sequence_length(task_type)
        })
    }
    
    /// Get effective backend policy (policy-recommended or explicit)
    pub fn effective_backend_policy(&self, input_length: usize) -> BackendPolicy {
        self.backend_policy.unwrap_or_else(|| {
            let policy = PerformancePolicy::default();
            let seq_len = self.effective_sequence_length(input_length);
            policy.recommended_backend(seq_len)
        })
    }
    
    /// Convert backend policy to MLComputeUnits for model loading
    pub fn to_compute_units(&self, input_length: usize) -> crate::ane::compat::coreml::MLComputeUnits {
        use crate::ane::compat::coreml::MLComputeUnits;
        match self.effective_backend_policy(input_length) {
            BackendPolicy::ANE => MLComputeUnits::CpuAndNeuralEngine,
            BackendPolicy::CPU => MLComputeUnits::CpuOnly,
            BackendPolicy::Auto => {
                // Auto-select based on sequence length
                let policy = PerformancePolicy::default();
                let seq_len = self.effective_sequence_length(input_length);
                match policy.recommended_backend(seq_len) {
                    BackendPolicy::ANE => MLComputeUnits::CpuAndNeuralEngine,
                    BackendPolicy::CPU => MLComputeUnits::CpuOnly,
                    BackendPolicy::Auto => MLComputeUnits::CpuAndNeuralEngine, // Default to ANE
                }
            }
        }
    }
}

/// Constitutional reasoning result
#[derive(Debug, Clone, JsonSchema)]
pub struct ConstitutionalVerdict {
    pub compliance_level: ComplianceLevel,
    pub risk_assessment: RiskTier,
    pub key_concerns: Vec<String>,
    pub recommendations: Vec<String>,
    pub verdict: Verdict,
    pub justification: String,
    pub confidence_score: f32,
}

/// Compliance levels
#[derive(Debug, Clone, PartialEq, JsonSchema)]
pub enum ComplianceLevel {
    Full,
    Partial,
    None,
}

/// Re-export RiskTier from contracts
pub use agent_agency_contracts::types::planning::RiskTier;

/// Verdict types
#[derive(Debug, Clone, PartialEq, JsonSchema)]
pub enum Verdict {
    Approve,
    Modify,
    Reject,
}

/// Debate argument result
#[derive(Debug, Clone, JsonSchema)]
pub struct DebateArgument {
    pub position: DebatePosition,
    pub argument: String,
    pub evidence_citations: Vec<String>,
    pub confidence_level: ConfidenceLevel,
}

/// Debate positions
#[derive(Debug, Clone, PartialEq, JsonSchema)]
pub enum DebatePosition {
    Support,
    Challenge,
}

/// Confidence levels
#[derive(Debug, Clone, PartialEq, JsonSchema)]
pub enum ConfidenceLevel {
    High,
    Medium,
    Low,
}

/// Constitutional reasoning using Mistral model
pub async fn deliberate_constitution(
    model: &mut MistralModel,
    task_spec: &str,
    evidence: &[String],
    debate_history: &[String],
    options: &MistralInferenceOptions,
) -> Result<ConstitutionalVerdict> {
    // Format the constitutional analysis prompt
    let prompt =
        reasoning_templates::format_constitutional_analysis(task_spec, evidence, debate_history)?;

    // Generate response using Mistral model
    let response = generate_text(model, &prompt, options).await?;

    // Parse the structured response
    parse_constitutional_response(&response)
}

/// Generate debate argument using Mistral model
pub async fn generate_debate_argument(
    model: &mut MistralModel,
    debate_topic: &str,
    previous_arguments: &[String],
    evidence: &[String],
    options: &MistralInferenceOptions,
) -> Result<DebateArgument> {
    // Format the debate argument prompt
    let prompt =
        reasoning_templates::format_debate_argument(debate_topic, previous_arguments, evidence)?;

    // Generate response using Mistral model
    let response = generate_text(model, &prompt, options).await?;

    // Parse the structured response
    parse_debate_response(&response)
}

/// Generate text using Mistral model
pub async fn generate_text(
    model: &mut MistralModel,
    prompt: &str,
    options: &MistralInferenceOptions,
) -> Result<String> {
    // Update last accessed time (std::sync::Mutex, no await needed)
    if let Ok(mut last_accessed) = model.last_accessed.lock() {
        *last_accessed = std::time::Instant::now();
    }

    // Check circuit breaker
    if model.circuit_breaker.is_open() {
        return Err(ANEError::CircuitBreakerOpen(
            "Circuit breaker is open".to_string(),
        ));
    }

    // Tokenize input
    let input_tokens = model.tokenizer.encode(prompt)?;

    // Apply performance policy to determine optimal sequence length and backend
    let policy = PerformancePolicy::default();
    let options = options.clone().with_policy(input_tokens.len(), Some(&policy));
    
    // Get effective sequence length from policy (or use explicit if set)
    let effective_seq_len = options.effective_sequence_length(input_tokens.len());
    
    // Log policy decision for observability
    let task_type = options.task_type.unwrap_or_else(|| {
        TaskType::from_input(input_tokens.len(), options.max_tokens)
    });
    let backend = options.effective_backend_policy(input_tokens.len());
    tracing::info!(
        "Policy decision: task_type={:?}, seq_len={}, backend={:?}, input_len={}",
        task_type, effective_seq_len, backend, input_tokens.len()
    );
    
    // Check input length against effective sequence length (policy-recommended or explicit)
    // Use effective_seq_len as the context window limit for this inference
    let context_length = effective_seq_len.min(model.schema.context_length); // Cap at model's max context window
    if input_tokens.len() > context_length {
        return Err(ANEError::ContextTooLong(format!(
            "Input length {} exceeds effective context window {} (policy-recommended: {}, model max: {})",
            input_tokens.len(),
            context_length,
            effective_seq_len,
            model.schema.context_length
        )));
    }

    // TODO: Integrate ANE for Mistral inference
    // - [ ] Use ANE device instead of CPU for inference
    // - [ ] Configure ANE execution options for Mistral model
    // - [ ] Handle ANE inference errors and fallback to CPU
    // - [ ] Add performance benchmarks for ANE vs CPU
    // - [ ] Add unit tests with ANE device
    // - [ ] Add integration tests with real ANE inference
    // TODO: Integrate ANE device for inference:
    // 1. ANE device setup: Set up ANE device for inference
    //    - Initialize ANE device and resources
    //    - Configure ANE for model execution
    //    - Handle ANE device errors and fallback
    // 2. ANE inference: Execute inference on ANE
    //    - Load model onto ANE device
    //    - Execute inference using ANE
    //    - Handle ANE inference errors gracefully
    // 3. Performance optimization: Optimize ANE usage
    //    - Measure ANE vs CPU performance
    //    - Optimize ANE inference pipeline
    //    - Support ANE-specific optimizations
    // ACCEPTANCE CRITERIA:
    // - ANE device is used for inference when available
    // - ANE inference provides performance improvements
    // - CPU fallback works when ANE unavailable
    // DEPENDENCIES:
    // - ANE device API (Required)
    // - ANE inference backend (Required)
    // PRIORITY: High
    // Prepare input tensor
    let device = Device::Cpu;
    // Keep token ids as i32; the backend embeds internally
    // Note: Candle requires converting to f32 for tensor creation, but we'll treat them as IDs
    let input_tokens_f32: Vec<f32> = input_tokens.iter().map(|&x| x as f32).collect();
    let _input_tensor = Tensor::from_slice(
        input_tokens_f32.as_slice(),
        (input_tokens_f32.len(),),
        &device,
    )?
    .unsqueeze(0)?; // Add batch dimension

    // Generate tokens
    let mut generated_tokens = input_tokens.clone();
    let mut kv_cache = model.kv_cache.lock().await;

    // Get EOS token ID from tokenizer
    let eos_id = model.tokenizer.eos_id().unwrap_or(2);

    // Pre-allocate capacity to avoid reallocations
    generated_tokens.reserve(input_tokens.len() + options.max_tokens);

    for _ in 0..options.max_tokens {
        // Prepare input for next token prediction
        let input_len = generated_tokens.len();
        let input_slice = &generated_tokens[input_len.saturating_sub(context_length)..];

        // Keep token ids as i32; the backend embeds internally
        // Note: Candle requires converting to f32 for tensor creation, but we'll treat them as IDs
        let input_slice_f32: Vec<f32> = input_slice.iter().map(|&x| x as f32).collect();
        let input_tensor = Tensor::from_slice(
            input_slice_f32.as_slice(),
            (input_slice_f32.len(),),
            &device,
        )?
        .unsqueeze(0)?;

        // Run inference through Core ML with timeout
        let step_future = run_mistral_inference(model, &input_tensor);
        let logits = match tokio::time::timeout(
            std::time::Duration::from_millis(options.timeout_ms),
            step_future,
        )
        .await
        {
            Ok(result) => result?,
            Err(_) => {
                model.circuit_breaker.record_failure();
                return Err(ANEError::Timeout(options.timeout_ms));
            }
        };

        // Sample next token
        let next_token = sample_token(&logits, &options)?;

        // Check for end token
        if next_token == eos_id {
            break;
        }

        generated_tokens.push(next_token);

        // Update KV cache if enabled
        if options.use_kv_cache {
            if let Err(e) = kv_cache.step() {
                tracing::warn!("KV cache step failed: {}", e);
                // Continue without cache for this step
            }
        }
    }

    // Decode generated tokens
    let generated_text = model
        .tokenizer
        .decode(&generated_tokens[input_tokens.len()..])?;

    // Record successful inference
    model.circuit_breaker.record_success();

    Ok(generated_text)
}

/// Run Mistral inference through Core ML
async fn run_mistral_inference(model: &MistralModel, input_tensor: &Tensor) -> Result<Tensor> {
    use crate::ane::compat::coreml_module::run_inference;

    // Get the model reference from SafeModelHandle
    let model_ref = model.handle.get_model_ref(); // SafeModelHandle.get_model_ref() returns ModelRef

    // Convert input tensor to f32 slice
    // Input tensor shape: [batch_size, sequence_length]
    let input_dims = input_tensor.dims();
    if input_dims.len() != 2 {
        return Err(ANEError::InvalidInput(format!(
            "Expected 2D input tensor [batch, seq_len], got shape: {:?}",
            input_dims
        )));
    }

    let batch_size = input_dims[0];
    let seq_len = input_dims[1];

    // Flatten tensor to f32 slice
    let input_data = input_tensor
        .flatten_all()
        .map_err(|e| ANEError::InferenceFailed(format!("Failed to flatten input tensor: {}", e)))?
        .to_vec1::<f32>()
        .map_err(|e| {
            ANEError::InferenceFailed(format!("Failed to convert tensor to f32: {}", e))
        })?;

    // Input shape for CoreML: [batch_size, sequence_length]
    let input_shape = vec![batch_size, seq_len];

    // Mistral models typically use "input_ids" as the input feature name
    // This may need to be configurable based on the actual model schema
    let input_name = "input_ids".to_string();

    // CRITICAL: Wrap blocking FFI call in spawn_blocking to prevent async runtime starvation
    // The Core ML FFI call is synchronous and can block for extended periods. If called
    // directly in async context, it can block the async runtime thread and prevent watchdog
    // check-ins, causing kernel panics. spawn_blocking moves the work to a separate thread pool.
    let output_tensor = tokio::task::spawn_blocking(move || {
        run_inference(model_ref, &input_name, &input_data, &input_shape)
    })
    .await
    .map_err(|e| ANEError::Internal(format!("Inference task panicked: {}", e)))?
    .map_err(|e| ANEError::InferenceFailed(format!("CoreML inference failed: {}", e)))?;

    // Output tensor shape should be [batch_size, vocab_size] for logits
    // Verify output shape matches expectations
    let output_dims = output_tensor.dims();
    if output_dims.len() != 2 {
        return Err(ANEError::Internal(format!(
            "Expected 2D output tensor [batch, vocab], got shape: {:?}",
            output_dims
        )));
    }

    if output_dims[0] != batch_size {
        return Err(ANEError::Internal(format!(
            "Output batch size {} doesn't match input batch size {}",
            output_dims[0], batch_size
        )));
    }

    Ok(output_tensor)
}

/// Sample next token from logits
fn sample_token(logits: &Tensor, options: &MistralInferenceOptions) -> Result<i32> {
    // Logits shape is [B, V] (last token logits only)
    // Extract logits for batch index 0
    let logits = logits.i((0, ..))?;

    // Fast path: greedy sampling (temperature=None, top_p=None)
    if options.temperature.is_none() && options.top_p.is_none() {
        let logits_vec: Vec<f32> = logits.to_vec1()?;
        let argmax = logits_vec
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0);
        return Ok(argmax as i32);
    }

    // Apply temperature if specified
    let logits = if let Some(temp) = options.temperature {
        let temp_tensor = Tensor::new(&[temp], logits.device())?;
        (&logits / &temp_tensor)?
    } else {
        logits
    };

    // Apply top-p filtering if specified (stable version)
    let logits = if let Some(top_p) = options.top_p {
        apply_top_p_filtering_stable(&logits, top_p)?
    } else {
        logits
    };

    // Convert to probabilities
    let probs = candle_nn::ops::softmax_last_dim(&logits)?;

    // Sample from the distribution
    let probs_vec: Vec<f32> = probs.to_vec1()?;
    let mut rng = rand::thread_rng();
    let random_val: f32 = rng.gen();

    let mut cumulative = 0.0;
    for (i, prob) in probs_vec.iter().enumerate() {
        cumulative += prob;
        if random_val <= cumulative {
            return Ok(i as i32);
        }
    }

    // Fallback to argmax
    let argmax = probs_vec
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .map(|(i, _)| i)
        .unwrap_or(0);

    Ok(argmax as i32)
}

/// Apply top-p (nucleus) filtering with numerically stable log-sum-exp
fn apply_top_p_filtering_stable(logits: &Tensor, top_p: f32) -> Result<Tensor> {
    use candle_core::DType;

    // Ensure logits are float type
    let dtype = logits.dtype();
    if !matches!(dtype, DType::F32 | DType::F16 | DType::BF16) {
        return Err(ANEError::InvalidInput(format!(
            "Logits must be float type, got {:?}",
            dtype
        )));
    }

    // Log-sum-exp normalization for numerical stability
    // Convert to Vec for stable computation
    let logits_vec: Vec<f32> = logits.to_vec1()?;

    // Find max logit
    let max_logit = logits_vec
        .iter()
        .fold(f32::NEG_INFINITY, |acc, &x| acc.max(x));

    // Compute exp(shifted) and sum in a single pass for efficiency
    let mut exp_sum = 0.0;
    let exp_shifted: Vec<f32> = logits_vec
        .iter()
        .map(|&x| {
            let exp_val = (x - max_logit).exp();
            exp_sum += exp_val;
            exp_val
        })
        .collect();

    // Compute probabilities: exp(shifted) / sumexp
    let probs_vec: Vec<f32> = exp_shifted.iter().map(|&x| x / exp_sum).collect();

    // Sort indices by probability (descending)
    let mut indices: Vec<usize> = (0..probs_vec.len()).collect();
    indices.sort_unstable_by(|&i, &j| probs_vec[j].partial_cmp(&probs_vec[i]).unwrap());

    // Find cutoff: how many top tokens to keep to reach top_p probability mass
    let mut cumulative_prob = 0.0;
    let mut keep_count = 0;

    for &idx in &indices {
        cumulative_prob += probs_vec[idx];
        keep_count += 1;
        if cumulative_prob >= top_p {
            break;
        }
    }

    // Create mask: keep top-p set, mask rest with -inf
    let mut filtered_logits = vec![f32::NEG_INFINITY; logits_vec.len()];

    // Keep only the top-p tokens (first keep_count indices)
    for &idx in indices.iter().take(keep_count) {
        filtered_logits[idx] = logits_vec[idx];
    }

    Tensor::new(&*filtered_logits, logits.device())
        .map_err(|e| ANEError::InferenceFailed(format!("Tensor creation failed: {}", e)))
}

/// Parse constitutional analysis response
fn parse_constitutional_response(response: &str) -> Result<ConstitutionalVerdict> {
    let mut compliance_level = ComplianceLevel::Partial;
    let mut risk_assessment = RiskTier::Tier2;
    let mut key_concerns = Vec::new();
    let mut recommendations = Vec::new();
    let mut verdict = Verdict::Approve;
    let mut justification = String::new();
    let mut confidence_score = 0.5;

    // Parse structured response
    for line in response.lines() {
        let line = line.trim();
        if line.starts_with("COMPLIANCE_LEVEL:") {
            compliance_level = match line.split(':').nth(1).unwrap_or("").trim() {
                "FULL" => ComplianceLevel::Full,
                "PARTIAL" => ComplianceLevel::Partial,
                "NONE" => ComplianceLevel::None,
                _ => ComplianceLevel::Partial,
            };
        } else if line.starts_with("RISK_ASSESSMENT:") {
            risk_assessment = match line.split(':').nth(1).unwrap_or("").trim() {
                "TIER_1" => RiskTier::Tier1,
                "TIER_2" => RiskTier::Tier2,
                "TIER_3" => RiskTier::Tier3,
                _ => RiskTier::Tier2,
            };
        } else if line.starts_with("KEY_CONCERNS:") {
            let concerns_str = line.split(':').nth(1).unwrap_or("").trim();
            if !concerns_str.is_empty() {
                key_concerns = concerns_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect();
            }
        } else if line.starts_with("RECOMMENDATIONS:") {
            let recs_str = line.split(':').nth(1).unwrap_or("").trim();
            if !recs_str.is_empty() {
                recommendations = recs_str.split(',').map(|s| s.trim().to_string()).collect();
            }
        } else if line.starts_with("VERDICT:") {
            verdict = match line.split(':').nth(1).unwrap_or("").trim() {
                "APPROVE" => Verdict::Approve,
                "MODIFY" => Verdict::Modify,
                "REJECT" => Verdict::Reject,
                _ => Verdict::Approve,
            };
        } else if line.starts_with("JUSTIFICATION:") {
            justification = line.split(':').nth(1).unwrap_or("").trim().to_string();
        }
    }

    // Calculate confidence score based on response quality
    if !justification.is_empty() && !key_concerns.is_empty() {
        confidence_score = 0.8;
    } else if !justification.is_empty() {
        confidence_score = 0.6;
    }

    Ok(ConstitutionalVerdict {
        compliance_level,
        risk_assessment,
        key_concerns,
        recommendations,
        verdict,
        justification,
        confidence_score,
    })
}

/// Parse debate argument response
fn parse_debate_response(response: &str) -> Result<DebateArgument> {
    let mut position = DebatePosition::Support;
    let mut argument = String::new();
    let mut evidence_citations = Vec::new();
    let mut confidence_level = ConfidenceLevel::Medium;

    // Parse structured response
    for line in response.lines() {
        let line = line.trim();
        if line.starts_with("POSITION:") {
            position = match line.split(':').nth(1).unwrap_or("").trim() {
                "SUPPORT" => DebatePosition::Support,
                "CHALLENGE" => DebatePosition::Challenge,
                _ => DebatePosition::Support,
            };
        } else if line.starts_with("ARGUMENT:") {
            argument = line.split(':').nth(1).unwrap_or("").trim().to_string();
        } else if line.starts_with("EVIDENCE_CITATIONS:") {
            let citations_str = line.split(':').nth(1).unwrap_or("").trim();
            if !citations_str.is_empty() {
                evidence_citations = citations_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect();
            }
        } else if line.starts_with("CONFIDENCE_LEVEL:") {
            confidence_level = match line.split(':').nth(1).unwrap_or("").trim() {
                "HIGH" => ConfidenceLevel::High,
                "MEDIUM" => ConfidenceLevel::Medium,
                "LOW" => ConfidenceLevel::Low,
                _ => ConfidenceLevel::Medium,
            };
        }
    }

    Ok(DebateArgument {
        position,
        argument,
        evidence_citations,
        confidence_level,
    })
}
