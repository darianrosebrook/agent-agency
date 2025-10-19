//! @darianrosebrook
//! Shared types for multimodal ingestors

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Result of a single ingest operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestResult {
    pub document_id: Uuid,
    pub uri: String,
    pub sha256: String,
    pub kind: DocumentKind,
    pub project_scope: Option<String>,
    pub segments: Vec<Segment>,
    pub speech_turns: Option<Vec<SpeechTurn>>,
    pub diagram_data: Option<DiagramData>,
    pub ingested_at: DateTime<Utc>,
    pub quality_score: f32,
    pub toolchain: String,
}

/// Document type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DocumentKind {
    Video,
    Slides,
    Diagram,
    Transcript,
}

impl std::fmt::Display for DocumentKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocumentKind::Video => write!(f, "video"),
            DocumentKind::Slides => write!(f, "slides"),
            DocumentKind::Diagram => write!(f, "diagram"),
            DocumentKind::Transcript => write!(f, "transcript"),
        }
    }
}

/// Segment: time/space slice within document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segment {
    pub id: Uuid,
    pub segment_type: SegmentType,
    pub t0: Option<f32>,  // seconds
    pub t1: Option<f32>,
    pub bbox: Option<BoundingBox>,
    pub content_hash: String,
    pub quality_score: f32,
    pub stability_score: Option<f32>,  // for video keyframes
    pub blocks: Vec<Block>,
}

/// Segment type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SegmentType {
    Slide,
    Speech,
    Diagram,
    Scene,
}

impl std::fmt::Display for SegmentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SegmentType::Slide => write!(f, "slide"),
            SegmentType::Speech => write!(f, "speech"),
            SegmentType::Diagram => write!(f, "diagram"),
            SegmentType::Scene => write!(f, "scene"),
        }
    }
}

/// Block: semantic unit within segment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub id: Uuid,
    pub role: BlockRole,
    pub text: String,
    pub bbox: Option<BoundingBox>,
    pub ocr_confidence: Option<f32>,
    pub raw_bytes: Option<Vec<u8>>,  // for visual content
}

/// Block role
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum BlockRole {
    Title,
    Bullet,
    Code,
    Table,
    Figure,
    Caption,
}

impl std::fmt::Display for BlockRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockRole::Title => write!(f, "title"),
            BlockRole::Bullet => write!(f, "bullet"),
            BlockRole::Code => write!(f, "code"),
            BlockRole::Table => write!(f, "table"),
            BlockRole::Figure => write!(f, "figure"),
            BlockRole::Caption => write!(f, "caption"),
        }
    }
}

/// Bounding box (normalized coordinates)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Speech turn (aligned with document timestamps)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechTurn {
    pub id: Uuid,
    pub speaker_id: Option<String>,
    pub provider: String,  // 'whisperx', 'apple', etc.
    pub t0: f32,
    pub t1: f32,
    pub text: String,
    pub confidence: f32,
    pub word_timings: Vec<WordTiming>,
}

/// Fine-grained word timing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordTiming {
    pub t0: f32,
    pub t1: f32,
    pub token: String,
}

/// Diagram data (graph-structured)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagramData {
    pub entities: Vec<DiagramEntity>,
    pub edges: Vec<DiagramEdge>,
    pub render_png: Option<Vec<u8>>,
}

/// Diagram entity (node/label)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagramEntity {
    pub id: Uuid,
    pub entity_type: String,  // 'node', 'edge', 'label'
    pub normalized_name: String,
    pub attributes: HashMap<String, serde_json::Value>,
}

/// Diagram edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagramEdge {
    pub id: Uuid,
    pub src: Uuid,
    pub dst: Uuid,
    pub label: Option<String>,
}

/// Scene detection configuration
#[derive(Debug, Clone)]
pub struct SceneDetectorConfig {
    pub ssim_threshold: f32,
    pub phash_hamming_distance: u8,
    pub token_delta_threshold: f32,
}

impl Default for SceneDetectorConfig {
    fn default() -> Self {
        Self {
            ssim_threshold: 0.95,
            phash_hamming_distance: 10,
            token_delta_threshold: 0.3,
        }
    }
}

/// Frame sampling configuration
#[derive(Debug, Clone)]
pub struct FrameSamplerConfig {
    pub fps_target: f32,
    pub quality_threshold: f32,
    pub window_ms: u32,
}

impl Default for FrameSamplerConfig {
    fn default() -> Self {
        Self {
            fps_target: 3.0,
            quality_threshold: 0.5,
            window_ms: 500,
        }
    }
}

/// File watching event
#[derive(Debug, Clone)]
pub struct FileEvent {
    pub path: std::path::PathBuf,
    pub event_type: FileEventType,
}

#[derive(Debug, Clone, Copy)]
pub enum FileEventType {
    Created,
    Modified,
    Removed,
}
