//! Mistral LLM inference execution
//!
//! This module provides the inference execution logic for Mistral models,
//! including text generation, constitutional reasoning, and debate protocols.

use crate::ane::ane_errors::{ANEError, Result};
use crate::ane::models::mistral_model::{MistralModel, reasoning_templates};
use crate::ane::TensorSpec;
use crate::ane::compat::coreml::coreml::{ModelRef, validate_io_schema, run_inference};
use candle_core::Tensor;
use crate::telemetry::TelemetryCollector;
use std::time::Instant;

/// Inference options for Mistral models
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone, PartialEq)]
pub enum ComplianceLevel {
    Full,
    Partial,
    None,
}

/// Risk tiers
#[derive(Debug, Clone, PartialEq)]
pub enum RiskTier {
    Tier1,
    Tier2,
    Tier3,
}

/// Verdict types
#[derive(Debug, Clone, PartialEq)]
pub enum Verdict {
    Approve,
    Modify,
    Reject,
}

/// Debate argument result
#[derive(Debug, Clone)]
pub struct DebateArgument {
    pub position: DebatePosition,
    pub argument: String,
    pub evidence_citations: Vec<String>,
    pub confidence_level: ConfidenceLevel,
}

/// Debate positions
#[derive(Debug, Clone, PartialEq)]
pub enum DebatePosition {
    Support,
    Challenge,
}

/// Confidence levels
#[derive(Debug, Clone, PartialEq)]
pub enum ConfidenceLevel {
    High,
    Medium,
    Low,
}

/// Execute constitutional reasoning with Mistral model
pub async fn deliberate_constitution(
    model: &mut MistralModel,
    task_spec: &str,
    evidence: &[String],
    debate_history: &[String],
    options: &MistralInferenceOptions,
) -> Result<ConstitutionalVerdict> {
    let start_time = Instant::now();

    // Generate constitutional analysis prompt
    let prompt = reasoning_templates::format_constitutional_analysis(
        task_spec,
        evidence,
        debate_history,
    )?;

    // Execute inference
    let response = generate_text(model, &prompt, options).await?;

    // Parse structured response
    let verdict = parse_constitutional_verdict(&response)?;

    // Update access time
    if let Ok(mut last_accessed) = model.last_accessed.lock() {
        *last_accessed = Instant::now();
    }

    // Record telemetry
    let duration = start_time.elapsed();
    model.telemetry.record_inference(duration.as_millis() as u64, true);

    Ok(verdict)
}

/// Generate debate argument with Mistral model
pub async fn generate_debate_argument(
    model: &mut MistralModel,
    debate_topic: &str,
    previous_arguments: &[String],
    evidence: &[String],
    options: &MistralInferenceOptions,
) -> Result<DebateArgument> {
    let start_time = Instant::now();

    // Generate debate argument prompt
    let prompt = reasoning_templates::format_debate_argument(
        debate_topic,
        previous_arguments,
        evidence,
    )?;

    // Execute inference
    let response = generate_text(model, &prompt, options).await?;

    // Parse structured response
    let argument = parse_debate_argument(&response)?;

    // Update access time
    if let Ok(mut last_accessed) = model.last_accessed.lock() {
        *last_accessed = Instant::now();
    }

    // Record telemetry
    let duration = start_time.elapsed();
    model.telemetry.record_inference(duration.as_millis() as u64, true);

    Ok(argument)
}

/// Generate text with Mistral model
pub async fn generate_text(
    model: &mut MistralModel,
    prompt: &str,
    options: &MistralInferenceOptions,
) -> Result<String> {
    let start_time = Instant::now();
    
    // Check if prompt fits in context
    if !model.tokenizer.fits_context(prompt, model.schema.context_length)? {
        return Err(ANEError::InvalidInput(
            format!("Prompt too long for context window ({} tokens)",
                model.schema.context_length)
        ));
    }

    // Encode prompt
    let mut tokens = model.tokenizer.encode(prompt)?;

    // Generate tokens
    let mut generated_tokens = Vec::new();

    for _ in 0..options.max_tokens {
        // Check timeout
        if start_time.elapsed().as_millis() > options.timeout_ms as u128 {
            break;
        }

        // Prepare input for model
        let input_tokens = prepare_model_input(&tokens, &model.schema)?;

        // Run inference (placeholder - needs actual CoreML integration)
        let next_token = run_inference_step(model, &input_tokens, options.temperature, options.top_p).await?;

        // Add to generated tokens
        generated_tokens.push(next_token);
        tokens.push(next_token);

        // Update KV cache if enabled
        if options.use_kv_cache {
            if let Ok(mut kv_cache) = model.kv_cache.lock() {
                kv_cache.update(&tokens); // Update with current tokens
            }
        }

        // Check for end token
        if next_token == model.tokenizer.vocab_size()? - 1 { // EOS token
            break;
        }

        // Apply sampling (simplified greedy for now)
        // TODO: Implement temperature and top-p sampling
    }

    // Decode generated tokens
    model.tokenizer.decode(&generated_tokens)
}

/// Prepare model input from tokens
fn prepare_model_input(tokens: &[i32], schema: &crate::ane::models::mistral_model::ModelSchema) -> Result<Vec<i32>> {
    // Ensure tokens fit in context window
    let max_length = schema.context_length;
    if tokens.len() > max_length {
        return Err(ANEError::InvalidInput(
            format!("Token sequence too long: {} > {}", tokens.len(), max_length)
        ));
    }

    // Pad or truncate as needed
    let mut input_tokens = tokens.to_vec();

    // Ensure minimum length for model
    if input_tokens.len() < schema.inputs[0].shape[1] {
        // Pad with zeros (simplified - should pad with EOS or specific token)
        input_tokens.resize(schema.inputs[0].shape[1], 0);
    }

    Ok(input_tokens)
}

/// Run single inference step using CoreML
async fn run_inference_step(
    model: &MistralModel, 
    input_tokens: &[i32],
    temperature: Option<f32>,
    top_p: Option<f32>
) -> Result<i32> {
    use crate::ane::compat::coreml;
    use candle_core::{Tensor, Device};

    // Validate input
    if input_tokens.is_empty() {
        return Err(ANEError::InvalidInput("Input tokens cannot be empty".to_string()));
    }

    if input_tokens.len() > 2048 { // Use a reasonable default max sequence length
        return Err(ANEError::InvalidInput(
            format!("Input too long: {} tokens exceeds max sequence length 2048",
                   input_tokens.len())
        ));
    }

    // Get CoreML model handle using the safe handle
    let model_ref = ModelRef::new();

    // Convert tokens to tensor - need to convert i32 to f32 for candle
    let input_data: Vec<f32> = input_tokens.iter().map(|&x| x as f32).collect();
    let input_tensor = Tensor::new(&*input_data, &Device::Cpu)
        .map_err(|e| ANEError::Internal(format!("Failed to create input tensor: {}", e)))?;

    // Reshape to [batch_size=1, seq_len]
    let input_tensor = input_tensor.unsqueeze(0)
        .map_err(|e| ANEError::Internal(format!("Failed to reshape tensor: {}", e)))?;

    // Create input specification for CoreML
    let input_spec = TensorSpec {
        name: "input_ids".to_string(),
        dtype: "I32".to_string(),
        shape: vec![1, input_tokens.len()], // [batch_size, seq_len]
        required: true,
        batch_capable: false,
    };

    // Validate tensor against model input requirements
    coreml::coreml::validate_io_schema(&input_tensor, &input_spec)?;

    // Prepare output specification
    let output_spec = TensorSpec {
        name: "logits".to_string(),
        dtype: "F32".to_string(),
        shape: vec![1, input_tokens.len(), 32000], // [batch, seq, vocab] - using default vocab size
        required: true,
        batch_capable: false,
    };

    // Run CoreML inference
    let outputs = coreml::coreml::run_inference(
        model_ref,
        "input_ids",
        &input_data,
        &[1, input_tokens.len()]
    ).map_err(|e| ANEError::Internal(format!("CoreML inference failed: {}", e)))?;

    // Extract logits from output (run_inference returns a single tensor)
    let logits = outputs;

    // Get logits for the last token position [batch=0, position=-1, vocab]
    let last_token_logits = logits.narrow(1, input_tokens.len() - 1, 1)?
        .squeeze(1)?; // Remove sequence dimension

    // Apply temperature scaling if enabled
    let scaled_logits = if let Some(temperature) = temperature {
        if temperature != 1.0 {
            last_token_logits.div(&Tensor::new(&[temperature], &Device::Cpu)?)?
        } else {
            last_token_logits
        }
    } else {
        last_token_logits
    };

    // Apply sampling strategy
    let next_token = match (temperature, top_p) {
        (None, _) => sample_greedy(&scaled_logits)?, // Greedy sampling
        (Some(_), None) => sample_greedy(&scaled_logits)?, // Greedy if no top-p
        (Some(_), Some(top_p)) if top_p >= 1.0 => sample_greedy(&scaled_logits)?, // Greedy if top-p disabled
        (Some(_), Some(top_p)) => sample_top_p(&scaled_logits, top_p)?, // Top-p sampling
    };

    Ok(next_token)
}

/// Sample next token using greedy approach
fn sample_greedy(logits: &Tensor) -> Result<i32> {
    use candle_core::Tensor;

    // Get the token with highest probability
    let token_id = logits.argmax(0)?
        .to_scalar::<i64>()
        .map_err(|e| ANEError::Internal(format!("Failed to get argmax: {}", e)))?;

    Ok(token_id as i32)
}

/// Sample next token using top-p (nucleus) sampling
fn sample_top_p(logits: &Tensor, top_p: f32) -> Result<i32> {
    use candle_core::{Tensor, Device};
    use rand::prelude::*;

    // Convert logits to probabilities
    let probs = candle_nn::ops::softmax(logits, 0)?;

    // Get probabilities as slice
    let probs_data = probs.to_vec1::<f32>()
        .map_err(|e| ANEError::Internal(format!("Failed to extract probabilities: {}", e)))?;

    // Sort probabilities and indices in descending order
    let mut prob_indices: Vec<(f32, usize)> = probs_data.iter().enumerate()
        .map(|(i, &p)| (p, i))
        .collect();
    prob_indices.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

    // Find cutoff point for top-p
    let mut cumulative_prob = 0.0;
    let mut cutoff_idx = 0;

    for (i, (prob, _)) in prob_indices.iter().enumerate() {
        cumulative_prob += prob;
        if cumulative_prob >= top_p {
            cutoff_idx = i + 1;
            break;
        }
    }

    // If we haven't reached top_p, include all tokens
    if cutoff_idx == 0 {
        cutoff_idx = prob_indices.len();
    }

    // Create filtered probabilities
    let filtered_probs: Vec<f32> = prob_indices.iter()
        .take(cutoff_idx)
        .map(|(prob, _)| *prob)
        .collect();

    // Normalize probabilities
    let sum: f32 = filtered_probs.iter().sum();
    let normalized_probs: Vec<f32> = filtered_probs.iter()
        .map(|p| p / sum)
        .collect();

    // Sample from filtered distribution
    let mut rng = rand::thread_rng();
    let random_val: f32 = rng.gen();
    let mut cumulative = 0.0;

    for (i, prob) in normalized_probs.iter().enumerate() {
        cumulative += prob;
        if random_val <= cumulative {
            return Ok(prob_indices[i].1 as i32);
        }
    }

    // Fallback (should not happen)
    Ok(prob_indices[0].1 as i32)
}

/// Parse constitutional verdict from model response
fn parse_constitutional_verdict(response: &str) -> Result<ConstitutionalVerdict> {
    // Parse structured response format
    let compliance_level = parse_compliance_level(response)?;
    let risk_assessment = parse_risk_tier(response)?;
    let key_concerns = parse_key_concerns(response)?;
    let recommendations = parse_recommendations(response)?;
    let verdict = parse_verdict(response)?;
    let justification = parse_justification(response)?;
    let confidence_score = parse_confidence_score(response)?;

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

/// Parse debate argument from model response
fn parse_debate_argument(response: &str) -> Result<DebateArgument> {
    let position = parse_debate_position(response)?;
    let argument = parse_argument_text(response)?;
    let evidence_citations = parse_evidence_citations(response)?;
    let confidence_level = parse_confidence_level(response)?;

    Ok(DebateArgument {
        position,
        argument,
        evidence_citations,
        confidence_level,
    })
}

// Parsing helper functions
fn parse_compliance_level(response: &str) -> Result<ComplianceLevel> {
    if response.contains("COMPLIANCE_LEVEL: FULL") {
        Ok(ComplianceLevel::Full)
    } else if response.contains("COMPLIANCE_LEVEL: PARTIAL") {
        Ok(ComplianceLevel::Partial)
    } else if response.contains("COMPLIANCE_LEVEL: NONE") {
        Ok(ComplianceLevel::None)
    } else {
        Ok(ComplianceLevel::Partial) // Default
    }
}

fn parse_risk_tier(response: &str) -> Result<RiskTier> {
    if response.contains("RISK_ASSESSMENT: TIER_1") {
        Ok(RiskTier::Tier1)
    } else if response.contains("RISK_ASSESSMENT: TIER_2") {
        Ok(RiskTier::Tier2)
    } else if response.contains("RISK_ASSESSMENT: TIER_3") {
        Ok(RiskTier::Tier3)
    } else {
        Ok(RiskTier::Tier2) // Default
    }
}

fn parse_key_concerns(response: &str) -> Result<Vec<String>> {
    // Extract concerns from KEY_CONCERNS section
    extract_list_section(response, "KEY_CONCERNS:")
}

fn parse_recommendations(response: &str) -> Result<Vec<String>> {
    // Extract recommendations from RECOMMENDATIONS section
    extract_list_section(response, "RECOMMENDATIONS:")
}

fn parse_verdict(response: &str) -> Result<Verdict> {
    if response.contains("VERDICT: APPROVE") {
        Ok(Verdict::Approve)
    } else if response.contains("VERDICT: MODIFY") {
        Ok(Verdict::Modify)
    } else if response.contains("VERDICT: REJECT") {
        Ok(Verdict::Reject)
    } else {
        Ok(Verdict::Modify) // Default
    }
}

fn parse_justification(response: &str) -> Result<String> {
    extract_section(response, "JUSTIFICATION:")
}

fn parse_confidence_score(response: &str) -> Result<f32> {
    // Extract confidence score (default 0.8 if not found)
    if response.contains("HIGH") {
        Ok(0.9)
    } else if response.contains("MEDIUM") {
        Ok(0.7)
    } else if response.contains("LOW") {
        Ok(0.5)
    } else {
        Ok(0.8)
    }
}

fn parse_debate_position(response: &str) -> Result<DebatePosition> {
    if response.contains("POSITION: SUPPORT") {
        Ok(DebatePosition::Support)
    } else if response.contains("POSITION: CHALLENGE") {
        Ok(DebatePosition::Challenge)
    } else {
        Ok(DebatePosition::Support) // Default
    }
}

fn parse_argument_text(response: &str) -> Result<String> {
    extract_section(response, "ARGUMENT:")
}

fn parse_evidence_citations(response: &str) -> Result<Vec<String>> {
    extract_list_section(response, "EVIDENCE_CITATIONS:")
}

fn parse_confidence_level(response: &str) -> Result<ConfidenceLevel> {
    if response.contains("CONFIDENCE_LEVEL: HIGH") {
        Ok(ConfidenceLevel::High)
    } else if response.contains("CONFIDENCE_LEVEL: MEDIUM") {
        Ok(ConfidenceLevel::Medium)
    } else if response.contains("CONFIDENCE_LEVEL: LOW") {
        Ok(ConfidenceLevel::Low)
    } else {
        Ok(ConfidenceLevel::Medium) // Default
    }
}

// Utility functions for parsing
fn extract_section(response: &str, section_header: &str) -> Result<String> {
    if let Some(start) = response.find(section_header) {
        let start_idx = start + section_header.len();
        let remaining = &response[start_idx..];

        // Find next section or end
        let end_patterns = [
            "\nCOMPLIANCE_LEVEL:",
            "\nRISK_ASSESSMENT:",
            "\nKEY_CONCERNS:",
            "\nRECOMMENDATIONS:",
            "\nVERDICT:",
            "\nJUSTIFICATION:",
            "\nPOSITION:",
            "\nARGUMENT:",
            "\nEVIDENCE_CITATIONS:",
            "\nCONFIDENCE_LEVEL:",
        ];

        let mut end_idx = remaining.len();
        for pattern in &end_patterns {
            if let Some(pos) = remaining.find(pattern) {
                end_idx = end_idx.min(pos);
            }
        }

        Ok(remaining[..end_idx].trim().to_string())
    } else {
        Ok(String::new())
    }
}

fn extract_list_section(response: &str, section_header: &str) -> Result<Vec<String>> {
    let section = extract_section(response, section_header)?;
    let items: Vec<String> = section
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();

    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inference_options_default() {
        let options = MistralInferenceOptions::default();
        assert_eq!(options.max_tokens, 100);
        assert_eq!(options.temperature, 0.7);
        assert_eq!(options.top_p, 0.9);
        assert_eq!(options.timeout_ms, 30000);
        assert!(options.use_kv_cache);
    }

    #[test]
    fn test_parse_compliance_level() {
        assert_eq!(
            parse_compliance_level("COMPLIANCE_LEVEL: FULL").unwrap(),
            ComplianceLevel::Full
        );
        assert_eq!(
            parse_compliance_level("COMPLIANCE_LEVEL: PARTIAL").unwrap(),
            ComplianceLevel::Partial
        );
        assert_eq!(
            parse_compliance_level("COMPLIANCE_LEVEL: NONE").unwrap(),
            ComplianceLevel::None
        );
    }

    #[test]
    fn test_parse_risk_tier() {
        assert_eq!(
            parse_risk_tier("RISK_ASSESSMENT: TIER_1").unwrap(),
            RiskTier::Tier1
        );
        assert_eq!(
            parse_risk_tier("RISK_ASSESSMENT: TIER_2").unwrap(),
            RiskTier::Tier2
        );
        assert_eq!(
            parse_risk_tier("RISK_ASSESSMENT: TIER_3").unwrap(),
            RiskTier::Tier3
        );
    }

    #[test]
    fn test_parse_verdict() {
        assert_eq!(
            parse_verdict("VERDICT: APPROVE").unwrap(),
            Verdict::Approve
        );
        assert_eq!(
            parse_verdict("VERDICT: MODIFY").unwrap(),
            Verdict::Modify
        );
        assert_eq!(
            parse_verdict("VERDICT: REJECT").unwrap(),
            Verdict::Reject
        );
    }

    #[test]
    fn test_parse_debate_position() {
        assert_eq!(
            parse_debate_position("POSITION: SUPPORT").unwrap(),
            DebatePosition::Support
        );
        assert_eq!(
            parse_debate_position("POSITION: CHALLENGE").unwrap(),
            DebatePosition::Challenge
        );
    }

    #[test]
    fn test_extract_section() {
        let response = "VERDICT: APPROVE\nJUSTIFICATION: Good work\nKEY_CONCERNS: None";
        assert_eq!(
            extract_section(response, "JUSTIFICATION:").unwrap(),
            "Good work"
        );
    }

    #[test]
    fn test_extract_list_section() {
        let response = "KEY_CONCERNS:\n- Issue 1\n- Issue 2\nRECOMMENDATIONS: Fix it";
        let concerns = extract_list_section(response, "KEY_CONCERNS:").unwrap();
        assert_eq!(concerns.len(), 2);
        assert_eq!(concerns[0], "- Issue 1");
        assert_eq!(concerns[1], "- Issue 2");
    }
}
