//! Policy integration helpers for Mistral inference
//!
//! This module provides helper functions to integrate the performance policy
//! system with Mistral model loading and inference.

use crate::ane::compat::coreml::MLComputeUnits;
use crate::ane::models::mistral_model::MistralCompilationOptions;
use crate::ane::policy::{BackendPolicy, PerformancePolicy};
use crate::ane::infer::mistral::MistralInferenceOptions;

/// Convert BackendPolicy to MLComputeUnits for model compilation
pub fn backend_policy_to_compute_units(backend: BackendPolicy) -> MLComputeUnits {
    match backend {
        BackendPolicy::ANE => MLComputeUnits::CpuAndNeuralEngine,
        BackendPolicy::CPU => MLComputeUnits::CpuOnly,
        BackendPolicy::Auto => {
            // For Auto, default to ANE (will be optimized at runtime)
            MLComputeUnits::CpuAndNeuralEngine
        }
    }
}

/// Create MistralCompilationOptions from inference options and policy
///
/// This helper integrates the performance policy system with model compilation,
/// ensuring the model is loaded with the correct compute units based on the
/// policy recommendations.
pub fn create_compilation_options_from_policy(
    inference_options: &MistralInferenceOptions,
    input_length: usize,
    _policy: Option<&PerformancePolicy>,
) -> MistralCompilationOptions {
    // Get backend recommendation from policy (policy is used via inference_options)
    let backend = inference_options
        .effective_backend_policy(input_length);
    
    // Convert to compute units
    let compute_units = backend_policy_to_compute_units(backend);
    
    // Get effective sequence length for context length configuration
    let effective_seq_len = inference_options.effective_sequence_length(input_length);
    
    // Create compilation options
    MistralCompilationOptions {
        compute_units: Some(match compute_units {
            MLComputeUnits::CpuOnly => "cpu".to_string(),
            MLComputeUnits::CpuAndGpu => "cpuAndGpu".to_string(),
            MLComputeUnits::CpuAndNeuralEngine => "cpuAndNeuralEngine".to_string(),
            MLComputeUnits::All => "all".to_string(),
        }),
        context_length: Some(effective_seq_len),
        ..Default::default()
    }
}

/// Apply policy recommendations to inference options
///
/// This is a convenience function that applies the performance policy to
/// inference options, making it easy to use policy-based optimization.
pub fn apply_policy_to_options(
    options: MistralInferenceOptions,
    input_length: usize,
    policy: Option<&PerformancePolicy>,
) -> MistralInferenceOptions {
    options.with_policy(input_length, policy)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_policy_to_compute_units() {
        assert_eq!(
            backend_policy_to_compute_units(BackendPolicy::ANE),
            MLComputeUnits::CpuAndNeuralEngine
        );
        assert_eq!(
            backend_policy_to_compute_units(BackendPolicy::CPU),
            MLComputeUnits::CpuOnly
        );
        assert_eq!(
            backend_policy_to_compute_units(BackendPolicy::Auto),
            MLComputeUnits::CpuAndNeuralEngine
        );
    }

    #[test]
    fn test_create_compilation_options_from_policy() {
        let options = MistralInferenceOptions::default();
        let compilation_options = create_compilation_options_from_policy(&options, 100, None);
        
        // Should have compute units set
        assert!(compilation_options.compute_units.is_some());
        // Should have context length set
        assert!(compilation_options.context_length.is_some());
    }
}

