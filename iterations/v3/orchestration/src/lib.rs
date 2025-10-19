//! Orchestration core for V3
pub mod adapter;
pub mod caws_runtime;
pub mod db;
pub mod orchestrate;
pub mod persistence;
pub mod persistence_postgres;
pub mod provenance;
pub mod multimodal_orchestration;

// Re-export key components
pub use multimodal_orchestration::{
    MultimodalOrchestrator, ProcessingResult, ProcessingStatus, ProcessingStats,
};
