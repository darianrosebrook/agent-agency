//! Orchestration core for V3
pub mod adapter;
pub mod artifacts;
pub mod caws_runtime;
pub mod db;
pub mod orchestrate;
pub mod persistence;
pub mod persistence_postgres;
pub mod planning;
pub mod provenance;
pub mod quality;
pub mod refinement;
pub mod tracking;
pub mod multimodal_orchestration;

// Re-export key components
pub use multimodal_orchestration::{
    MultimodalOrchestrator, ProcessingResult, ProcessingStatus, ProcessingStats,
};
