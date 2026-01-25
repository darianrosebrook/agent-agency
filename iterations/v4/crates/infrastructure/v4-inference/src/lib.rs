//! v4-inference - Local LLM Inference
//!
//! This crate provides local LLM inference capabilities for Agent Agency V4.
//!
//! ## Features
//!
//! - **Provider abstraction**: Swappable backends (Mock, MLX, CoreML)
//! - **Mock provider**: Development and testing without model weights
//! - **MLX support**: Apple Silicon optimized via MLX framework (requires `mlx` feature)
//! - **CoreML support**: Legacy Apple Silicon inference (requires `coreml` feature)
//! - **Metrics**: Request tracking and performance monitoring
//!
//! ## Usage
//!
//! ```rust,no_run
//! use v4_inference::{InferenceService, InferenceConfig, InferenceRequest};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create service with mock provider for development
//!     let service = InferenceService::new(InferenceConfig::mock());
//!
//!     // Or use MLX for production on Apple Silicon:
//!     // let service = InferenceService::new(InferenceConfig::mlx("/path/to/model"));
//!
//!     // Load the model
//!     service.load_model().await?;
//!
//!     // Run inference
//!     let request = InferenceRequest::new("Explain recursion")
//!         .with_max_tokens(100)
//!         .with_temperature(0.7);
//!
//!     let response = service.infer(request).await?;
//!     println!("Generated: {}", response.text);
//!     println!("Tokens/sec: {:.1}", response.tokens_per_second);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Architecture
//!
//! ```text
//! InferenceService
//!     │
//!     ▼
//! InferenceProvider (trait)
//!     ├── MockProvider (development/testing)
//!     ├── MLXProvider (production, Apple Silicon - recommended)
//!     └── CoreMLProvider (legacy, macOS only)
//! ```
//!
//! ## Provider Selection
//!
//! | Provider | Feature Flag | Platform | Use Case |
//! |----------|--------------|----------|----------|
//! | Mock | (default) | Any | Development, testing |
//! | MLX | `mlx` | macOS (Apple Silicon) | Production (recommended) |
//! | CoreML | `coreml` | macOS | Legacy support |
//!
//! ## Model Portfolio
//!
//! | Model | Size | Role | Use Case |
//! |-------|------|------|----------|
//! | Worker | 1-4B | Code edits, tool-use | Task execution |
//! | Judge | 1-3B | Constitutional adjudication | Council evaluation |
//! | Drafter | ~1B | Speculative decoding | Fast inference |

pub mod config;
pub mod error;
pub mod mock;
pub mod provider;
pub mod service;
pub mod types;

#[cfg(feature = "mlx")]
pub mod mlx;

// Re-exports for convenience
pub use config::{InferenceConfig, ModelConfig, ProviderType};
pub use error::InferenceError;
pub use mock::MockProvider;
pub use provider::{InferenceProvider, ProviderFactory};
pub use service::{InferenceMetrics, InferenceService};
pub use types::{
    FinishReason, InferenceRequest, InferenceResponse, ModelInfo, ProviderStatus, TokenEvent,
};

#[cfg(feature = "mlx")]
pub use mlx::{MLXConfig, MLXDevice, MLXProvider};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crate_exports() {
        // Verify key types are accessible
        let _config = InferenceConfig::default();
        let _request = InferenceRequest::new("test");
    }

    #[tokio::test]
    async fn test_end_to_end() {
        let service = InferenceService::new(InferenceConfig::mock());

        service.load_model().await.unwrap();

        let request = InferenceRequest::new("What is 2+2?")
            .with_max_tokens(50)
            .with_temperature(0.0);

        let response = service.infer(request).await.unwrap();

        assert!(!response.text.is_empty());
        assert_eq!(response.finish_reason, FinishReason::Complete);
    }
}
