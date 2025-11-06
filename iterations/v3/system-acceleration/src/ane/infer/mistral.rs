//! Mistral LLM inference execution
//!
//! This module provides Mistral model inference capabilities including
//! constitutional reasoning, debate generation, and text generation.

use schemars::JsonSchema;
use crate::ane::ane_errors::{ANEError, Result};
use crate::ane::models::mistral_model::{MistralModel, reasoning_templates};
use candle_core::{Tensor, Device, IndexOp};
use rand::Rng;

/// Inference options for Mistral models
#[derive(Debug, Clone, JsonSchema)]
pub struct MistralInferenceOptions {
    pub max_tokens: usize,
    pub temperature: Option<f32>, // None = greedy sampling
    pub top_p: Option<f32>,       // None = no top-p filtering
    pub timeout_ms: u64,
    pub use_kv_cache: bool,
}

impl Default for MistralInferenceOptions {
    fn default() -> Self {
        Self {
            max_tokens: 100,
            temperature: Some(0.7), // Enable temperature sampling
            top_p: Some(0.9),       // Enable top-p sampling
            timeout_ms: 30000,      // 30 seconds
            use_kv_cache: true,
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
    let prompt = reasoning_templates::format_constitutional_analysis(
        task_spec,
        evidence,
        debate_history,
    )?;

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
    let prompt = reasoning_templates::format_debate_argument(
        debate_topic,
        previous_arguments,
        evidence,
    )?;

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
        return Err(ANEError::CircuitBreakerOpen("Circuit breaker is open".to_string()));
    }

    // Tokenize input
    let input_tokens = model.tokenizer.encode(prompt)?;
    
    // Check if input fits in context window
    let context_length = model.schema.context_length;
    if input_tokens.len() > context_length {
        return Err(ANEError::ContextTooLong(format!(
            "Input length {} exceeds context window {}", 
            input_tokens.len(), 
            context_length
        )));
    }

    // TODO: Integrate ANE for Mistral inference
    // - [ ] Use ANE device instead of CPU for inference
    // - [ ] Configure ANE execution options for Mistral model
    // - [ ] Handle ANE inference errors and fallback to CPU
    // - [ ] Add performance benchmarks for ANE vs CPU
    // - [ ] Add unit tests with ANE device
    // - [ ] Add integration tests with real ANE inference
    // Prepare input tensor
    let device = Device::Cpu; // Use CPU for now, ANE integration will come later
    // Keep token ids as i32; the backend embeds internally
    // Note: Candle requires converting to f32 for tensor creation, but we'll treat them as IDs
    let input_tokens_f32: Vec<f32> = input_tokens.iter().map(|&x| x as f32).collect();
    let _input_tensor = Tensor::from_slice(input_tokens_f32.as_slice(), (input_tokens_f32.len(),), &device)?
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
        let input_tensor = Tensor::from_slice(input_slice_f32.as_slice(), (input_slice_f32.len(),), &device)?
            .unsqueeze(0)?;

        // Run inference through Core ML with timeout
        let step_future = run_mistral_inference(model, &input_tensor);
        let logits = match tokio::time::timeout(
            std::time::Duration::from_millis(options.timeout_ms),
            step_future
        ).await {
            Ok(result) => result?,
            Err(_) => {
                model.circuit_breaker.record_failure();
                return Err(ANEError::Timeout(options.timeout_ms));
            }
        };
        
        // Sample next token
        let next_token = sample_token(&logits, options)?;
        
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
    let generated_text = model.tokenizer.decode(&generated_tokens[input_tokens.len()..])?;
    
    // Record successful inference
    model.circuit_breaker.record_success();
    
    Ok(generated_text)
}

/// Run Mistral inference through Core ML
async fn run_mistral_inference(
    model: &MistralModel,
    input_tensor: &Tensor,
) -> Result<Tensor> {
    // Use the model's Core ML handle for inference
    let result = model.handle.with_handle(|_handle| -> Result<Tensor> {
        // TODO: Implement real Core ML inference for Mistral
        // - [ ] Call actual Core ML inference API with handle
        // - [ ] Process input tokens through Core ML model
        // - [ ] Extract output tensor from Core ML results
        // - [ ] Handle inference errors and timeouts
        // - [ ] Add unit tests with mock Core ML outputs
        // - [ ] Add integration tests with real Core ML inference
        // This would call the actual Core ML inference
        // For now, return a placeholder tensor
        let device = Device::Cpu;
        let batch_size = input_tensor.dims()[0];
        
        // Return [B, V] shape (last token logits only) to reduce I/O and simplify sampling
        let vocab_size = model.tokenizer.vocab_size().unwrap_or(32000) as usize;
        Tensor::zeros(&[batch_size, vocab_size], candle_core::DType::F32, &device)
            .map_err(|e| ANEError::InferenceFailed(format!("Failed to create tensor: {}", e)))
    });

    match result {
        Some(logits_result) => logits_result,
        None => Err(ANEError::InferenceFailed("Failed to access model handle".to_string())),
    }
}

/// Sample next token from logits
fn sample_token(logits: &Tensor, options: &MistralInferenceOptions) -> Result<i32> {
    // Logits shape is [B, V] (last token logits only)
    // Extract logits for batch index 0
    let logits = logits.i((0, ..))?;
    
    // Fast path: greedy sampling (temperature=None, top_p=None)
    if options.temperature.is_none() && options.top_p.is_none() {
        let logits_vec: Vec<f32> = logits.to_vec1()?;
        let argmax = logits_vec.iter()
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
    let argmax = probs_vec.iter()
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
        return Err(ANEError::InvalidInput(
            format!("Logits must be float type, got {:?}", dtype)
        ));
    }
    
    // Log-sum-exp normalization for numerical stability
    // Convert to Vec for stable computation
    let logits_vec: Vec<f32> = logits.to_vec1()?;
    
    // Find max logit
    let max_logit = logits_vec.iter()
        .fold(f32::NEG_INFINITY, |acc, &x| acc.max(x));
    
    // Compute exp(shifted) and sum in a single pass for efficiency
    let mut exp_sum = 0.0;
    let exp_shifted: Vec<f32> = logits_vec.iter()
        .map(|&x| {
            let exp_val = (x - max_logit).exp();
            exp_sum += exp_val;
            exp_val
        })
        .collect();
    
    // Compute probabilities: exp(shifted) / sumexp
    let probs_vec: Vec<f32> = exp_shifted.iter()
        .map(|&x| x / exp_sum)
        .collect();
    
    // Sort indices by probability (descending)
    let mut indices: Vec<usize> = (0..probs_vec.len()).collect();
    indices.sort_unstable_by(|&i, &j| {
        probs_vec[j].partial_cmp(&probs_vec[i]).unwrap()
    });
    
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
                key_concerns = concerns_str.split(',').map(|s| s.trim().to_string()).collect();
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
                evidence_citations = citations_str.split(',').map(|s| s.trim().to_string()).collect();
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
