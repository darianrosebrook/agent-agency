//! @darianrosebrook
//! Shared types for multimodal enrichers

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Result of Vision OCR enrichment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrResult {
    pub blocks: Vec<OcrBlock>,
    pub tables: Vec<Table>,
    pub text_regions: Vec<TextRegion>,
    pub confidence: f32,
    pub processing_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrBlock {
    pub id: Uuid,
    pub role: String, // "title", "bullet", "paragraph", etc.
    pub text: String,
    pub bbox: BoundingBox,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextRegion {
    pub text: String,
    pub bbox: BoundingBox,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    pub id: Uuid,
    pub rows: usize,
    pub cols: usize,
    pub cells: Vec<TableCell>,
    pub bbox: BoundingBox,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableCell {
    pub row: usize,
    pub col: usize,
    pub text: String,
    pub is_header: bool,
}

/// Result of ASR/Diarization enrichment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsrResult {
    pub turns: Vec<SpeechSegment>,
    pub speakers: Vec<Speaker>,
    pub language: Option<String>,
    pub confidence: f32,
    pub processing_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechSegment {
    pub id: Uuid,
    pub speaker_id: Option<String>,
    pub t0: f32,
    pub t1: f32,
    pub text: String,
    pub confidence: f32,
    pub word_timings: Vec<WordTiming>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordTiming {
    pub t0: f32,
    pub t1: f32,
    pub token: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Speaker {
    pub speaker_id: String,
    pub name: Option<String>,
    pub turn_count: usize,
    pub total_duration_ms: u64,
}

/// Result of entity extraction enrichment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityResult {
    pub entities: Vec<ExtractedEntity>,
    pub topics: Vec<Topic>,
    pub chapters: Vec<Chapter>,
    pub processing_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedEntity {
    pub id: Uuid,
    pub entity_type: String, // "person", "organization", "location", "date", etc.
    pub text: String,
    pub normalized: String,
    pub confidence: f32,
    pub pii: bool,
    pub span_start: usize,
    pub span_end: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topic {
    pub name: String,
    pub keywords: Vec<String>,
    pub confidence: f32,
    pub occurrence_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub title: String,
    pub t0: f32,
    pub t1: f32,
    pub description: Option<String>,
}

/// Result of visual captioning enrichment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptionResult {
    pub caption: String,
    pub confidence: f32,
    pub tags: Vec<String>,
    pub processing_time_ms: u64,
}

/// Enricher configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnricherConfig {
    pub vision_timeout_ms: u64,
    pub asr_provider: String, // "whisperx", "apple", etc.
    pub entity_ner_enabled: bool,
    pub caption_max_tokens: usize,
    pub circuit_breaker_threshold: usize,
    pub circuit_breaker_timeout_ms: u64,
}

impl Default for EnricherConfig {
    fn default() -> Self {
        Self {
            vision_timeout_ms: 5000,
            asr_provider: "whisperx".to_string(),
            entity_ner_enabled: true,
            caption_max_tokens: 50,
            circuit_breaker_threshold: 3,
            circuit_breaker_timeout_ms: 60000,
        }
    }
}
