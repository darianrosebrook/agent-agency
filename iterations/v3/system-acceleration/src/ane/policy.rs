//! ANE Performance Policy - Adaptive sequence length and backend selection
//!
//! This module implements performance policies based on benchmark findings:
//! - Sequence length break-even points for CPU vs ANE
//! - Task-type aware backend selection
//! - Adaptive configuration based on workload characteristics
//!
//! ## Performance Characteristics (from benchmark data)
//!
//! Based on Mistral 7B FP16 benchmarks on Apple Silicon:
//!
//! | Sequence Length | CPU (ms) | ANE (ms) | Speedup | Recommendation |
//! |----------------|----------|----------|---------|----------------|
//! | 64 tokens      | 86.80    | 85.97    | 1.01x   | ANE slightly faster |
//! | 128 tokens     | 83.38    | 98.03    | 0.85x   | ❌ **Avoid** - CPU wins |
//! | 256 tokens     | 99.38    | 87.04    | 1.14x   | ✅ **Optimal** - Best ANE speedup |
//! | 512 tokens     | 94.64    | 84.36    | 1.12x   | ✅ Good - ANE faster |
//!
//! ## Break-Even Points
//!
//! - **ANE outperforms CPU**: seq ≤ 64 tokens (1.01x) or seq ≥ 256 tokens (1.14x)
//! - **Neutral zone**: ~128 tokens (1.0x, but CPU slightly faster at 0.85x)
//! - **ANE loses**: 128 tokens (0.85x - ANE 15% slower)
//!
//! ## Policy Strategy
//!
//! - **Low-latency tasks** (tool calls, classification, routing): Use ANE with 64-128 tokens
//! - **Standard tasks** (general inference): Use ANE with 256 tokens (optimal)
//! - **Long-context tasks** (heavy reasoning): Use ANE with 512 tokens or CPU-only
//!
//! The constitutional "local high-performance" requirement is framed as:
//! > Must achieve ≤X ms latency and ≥Y tokens/sec locally; choice of CPU/ANE is allowed
//! > to vary by sequence length and request type.

use crate::ane::ane_errors::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Task type for adaptive policy selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum TaskType {
    /// Low-latency tasks: tool calls, classification, routing
    /// Prefers shorter sequences (64-128 tokens) for fast response
    LowLatency,
    /// Standard inference tasks: general text generation
    /// Uses optimal sequence length (256 tokens) for best ANE performance
    Standard,
    /// Long-context tasks: heavy reasoning, document analysis
    /// Uses longer sequences (512 tokens) or CPU-only if needed
    LongContext,
}

impl TaskType {
    /// Determine task type from input characteristics
    pub fn from_input(input_length: usize, max_tokens: usize) -> Self {
        // Heuristic: if input is already long or we need many tokens, it's long-context
        if input_length > 200 || max_tokens > 500 {
            TaskType::LongContext
        } else if input_length < 50 && max_tokens < 100 {
            TaskType::LowLatency
        } else {
            TaskType::Standard
        }
    }
}

/// Backend selection policy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum BackendPolicy {
    /// Use ANE (CPU + Neural Engine)
    ANE,
    /// Use CPU only
    CPU,
    /// Auto-select based on sequence length and task type
    Auto,
}

/// Sequence length policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequenceLengthPolicy {
    /// Default sequence length for standard tasks
    pub default: usize,
    /// Sequence length for low-latency tasks
    pub low_latency: usize,
    /// Sequence length for long-context tasks
    pub long_context: usize,
    /// Minimum sequence length (safety limit)
    pub min: usize,
    /// Maximum sequence length (safety limit)
    pub max: usize,
}

impl Default for SequenceLengthPolicy {
    fn default() -> Self {
        // Based on benchmark findings:
        // - 256 tokens: optimal ANE performance (1.14x speedup)
        // - 64 tokens: good for low-latency (1.01x speedup)
        // - 512 tokens: good for long-context (1.12x speedup)
        // - 128 tokens: AVOID (0.85x - ANE slower than CPU)
        Self {
            default: 256,      // Optimal ANE performance
            low_latency: 64,   // Fast response, slight ANE advantage
            long_context: 512, // Larger context, good ANE performance
            min: 32,           // Safety minimum
            max: 1024,         // Safety maximum
        }
    }
}

/// Performance policy for ANE/CPU selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformancePolicy {
    /// Sequence length configuration
    pub sequence_length: SequenceLengthPolicy,
    /// Backend selection policy
    pub backend: BackendPolicy,
    /// Enable adaptive selection based on task type
    pub adaptive: bool,
}

impl Default for PerformancePolicy {
    fn default() -> Self {
        Self {
            sequence_length: SequenceLengthPolicy::default(),
            backend: BackendPolicy::Auto,
            adaptive: true,
        }
    }
}

impl PerformancePolicy {
    /// Create a new performance policy
    pub fn new() -> Self {
        Self::default()
    }

    /// Get recommended sequence length for a task type
    pub fn recommended_sequence_length(&self, task_type: TaskType) -> usize {
        match task_type {
            TaskType::LowLatency => self.sequence_length.low_latency,
            TaskType::Standard => self.sequence_length.default,
            TaskType::LongContext => self.sequence_length.long_context,
        }
    }

    /// Get recommended backend for a sequence length
    ///
    /// Based on benchmark findings:
    /// - seq ≤ 64: ANE slightly faster (1.01x)
    /// - seq = 128: CPU faster (0.85x) - avoid ANE
    /// - seq ≥ 256: ANE faster (1.14x at 256, 1.12x at 512)
    pub fn recommended_backend(&self, sequence_length: usize) -> BackendPolicy {
        if !self.adaptive {
            return self.backend;
        }

        match self.backend {
            BackendPolicy::Auto => {
                // Avoid 128 tokens - ANE is 15% slower (0.85x)
                if sequence_length == 128 {
                    BackendPolicy::CPU
                } else if sequence_length <= 64 || sequence_length >= 256 {
                    // ANE wins at 64 (1.01x) and 256+ (1.14x at 256, 1.12x at 512)
                    BackendPolicy::ANE
                } else {
                    // For other lengths, default to CPU (conservative)
                    BackendPolicy::CPU
                }
            }
            policy => policy,
        }
    }

    /// Get optimal configuration for a task type
    pub fn optimal_config(&self, task_type: TaskType) -> (usize, BackendPolicy) {
        let seq_len = self.recommended_sequence_length(task_type);
        let backend = self.recommended_backend(seq_len);
        (seq_len, backend)
    }

    /// Validate sequence length against policy limits
    pub fn validate_sequence_length(&self, seq_len: usize) -> Result<usize> {
        if seq_len < self.sequence_length.min {
            return Err(crate::ane::ane_errors::ANEError::InvalidInput(format!(
                "Sequence length {} below minimum {}",
                seq_len, self.sequence_length.min
            )));
        }
        if seq_len > self.sequence_length.max {
            return Err(crate::ane::ane_errors::ANEError::InvalidInput(format!(
                "Sequence length {} above maximum {}",
                seq_len, self.sequence_length.max
            )));
        }
        Ok(seq_len)
    }

    /// Get performance characteristics for a sequence length
    ///
    /// Returns (cpu_latency_ms, ane_latency_ms, speedup, recommendation)
    pub fn performance_characteristics(&self, seq_len: usize) -> Option<(f64, f64, f64, &'static str)> {
        // Based on actual benchmark data from ANE_PERFORMANCE_INVESTIGATION_REPORT.md
        match seq_len {
            64 => Some((86.80, 85.97, 1.01, "ANE slightly faster")),
            128 => Some((83.38, 98.03, 0.85, "❌ Avoid - CPU 15% faster")),
            256 => Some((99.38, 87.04, 1.14, "✅ Optimal - Best ANE speedup")),
            512 => Some((94.64, 84.36, 1.12, "✅ Good - ANE faster")),
            _ => None, // No benchmark data for other lengths
        }
    }
}

impl fmt::Display for PerformancePolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PerformancePolicy(backend={:?}, adaptive={}, seq_len={{default={}, low_latency={}, long_context={}}})",
            self.backend,
            self.adaptive,
            self.sequence_length.default,
            self.sequence_length.low_latency,
            self.sequence_length.long_context
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_type_detection() {
        assert_eq!(
            TaskType::from_input(30, 50),
            TaskType::LowLatency
        );
        assert_eq!(
            TaskType::from_input(100, 200),
            TaskType::Standard
        );
        assert_eq!(
            TaskType::from_input(300, 600),
            TaskType::LongContext
        );
    }

    #[test]
    fn test_recommended_sequence_length() {
        let policy = PerformancePolicy::default();
        assert_eq!(policy.recommended_sequence_length(TaskType::LowLatency), 64);
        assert_eq!(policy.recommended_sequence_length(TaskType::Standard), 256);
        assert_eq!(policy.recommended_sequence_length(TaskType::LongContext), 512);
    }

    #[test]
    fn test_recommended_backend() {
        let policy = PerformancePolicy::default();
        
        // 64 tokens: ANE recommended (1.01x speedup)
        assert_eq!(policy.recommended_backend(64), BackendPolicy::ANE);
        
        // 128 tokens: CPU recommended (0.85x - ANE slower)
        assert_eq!(policy.recommended_backend(128), BackendPolicy::CPU);
        
        // 256 tokens: ANE recommended (1.14x speedup - optimal)
        assert_eq!(policy.recommended_backend(256), BackendPolicy::ANE);
        
        // 512 tokens: ANE recommended (1.12x speedup)
        assert_eq!(policy.recommended_backend(512), BackendPolicy::ANE);
    }

    #[test]
    fn test_optimal_config() {
        let policy = PerformancePolicy::default();
        
        let (seq_len, backend) = policy.optimal_config(TaskType::LowLatency);
        assert_eq!(seq_len, 64);
        assert_eq!(backend, BackendPolicy::ANE);
        
        let (seq_len, backend) = policy.optimal_config(TaskType::Standard);
        assert_eq!(seq_len, 256);
        assert_eq!(backend, BackendPolicy::ANE);
        
        let (seq_len, backend) = policy.optimal_config(TaskType::LongContext);
        assert_eq!(seq_len, 512);
        assert_eq!(backend, BackendPolicy::ANE);
    }

    #[test]
    fn test_validate_sequence_length() {
        let policy = PerformancePolicy::default();
        
        // Valid lengths
        assert!(policy.validate_sequence_length(64).is_ok());
        assert!(policy.validate_sequence_length(256).is_ok());
        assert!(policy.validate_sequence_length(512).is_ok());
        
        // Invalid lengths
        assert!(policy.validate_sequence_length(10).is_err()); // Below min
        assert!(policy.validate_sequence_length(2000).is_err()); // Above max
    }

    #[test]
    fn test_performance_characteristics() {
        let policy = PerformancePolicy::default();
        
        let (cpu, ane, speedup, rec) = policy.performance_characteristics(256).unwrap();
        assert_eq!(speedup, 1.14);
        assert!(rec.contains("Optimal"));
        
        let (cpu, ane, speedup, rec) = policy.performance_characteristics(128).unwrap();
        assert_eq!(speedup, 0.85);
        assert!(rec.contains("Avoid"));
    }
}

