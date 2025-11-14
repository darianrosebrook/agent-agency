//! Thin DTOs for data processing domain
//!
//! Lightweight representations of processing concepts that are shared
//! across multiple crates. Domain-specific logic remains in local wrappers.
//!
//! @author @darianrosebrook

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Processing identifier for content processing operations
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct ProcessingId(#[schemars(with = "String")] pub Uuid);

impl std::fmt::Display for ProcessingId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for ProcessingId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ProcessingId(Uuid::parse_str(s)?))
    }
}

/// Content type classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ContentType {
    /// Text content
    Text,
    /// Binary data
    Binary,
    /// Structured data (JSON, XML, etc.)
    Structured,
}

/// Processed content result (thin DTO)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProcessedContent {
    /// Processing identifier
    pub id: ProcessingId,
    /// Content type
    pub content_type: ContentType,
    /// Processing metadata
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}
