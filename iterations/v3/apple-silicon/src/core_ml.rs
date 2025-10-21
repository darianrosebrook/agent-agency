// [refactor candidate]: split into core_ml/mod.rs - main module file only
//! Core ML Manager
//!
//! Manages Core ML models for Apple Silicon optimization and inference.
//!
//! ## Redis Cache Integration
//!
//! The CoreMLManager supports Redis caching for inference results to improve performance:
//!
//! ```rust,no_run
