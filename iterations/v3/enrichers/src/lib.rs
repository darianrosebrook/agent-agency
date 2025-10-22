//! Multimodal enrichers for V3 RAG system
//!
//! Enriches ingested content with semantic analysis:
//! - Vision OCR: Extract structured text and document layout
//! - ASR/Diarization: Transcribe audio and attribute speakers
//! - Entity Extraction: Identify named entities and topics
//! - Visual Captioning: Generate descriptions for images

pub mod asr_enricher;
pub mod circuit_breaker;
pub mod entity_enricher;
pub mod python_bridge;
pub mod types;
pub mod vision_enricher;
pub mod vision_stubs;
pub mod visual_caption_enricher;

pub use asr_enricher::AsrEnricher;
pub use circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitState};
pub use entity_enricher::EntityEnricher;
pub use types::*;
pub use vision_enricher::VisionEnricher;
pub use visual_caption_enricher::VisualCaptionEnricher;
