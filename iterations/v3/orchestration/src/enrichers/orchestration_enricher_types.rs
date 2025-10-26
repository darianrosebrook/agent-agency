use std::collections::HashMap;

/// Enriched block with multimodal content
#[derive(Debug, Clone)]
pub struct EnrichedBlock {
    pub id: String,
    pub original_content: String,
    pub enriched_content: String,
    pub enrichment_type: String,
    pub confidence: f64,
    pub metadata: HashMap<String, String>,
}
