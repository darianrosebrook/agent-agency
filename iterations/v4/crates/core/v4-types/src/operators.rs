//! Operator Taxonomy (Sterling-style)
//!
//! All operations are classified into 5 types following Sterling's operator taxonomy.
//! This provides a principled categorization for tools and actions.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Operator type classification following Sterling's S/M/P/K/C taxonomy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum OperatorType {
    /// Seek - Information retrieval operations
    Seek(SeekOp),
    /// Memorize - Store information operations
    Memorize(MemorizeOp),
    /// Perceive - Interpret input operations
    Perceive(PerceiveOp),
    /// Knowledge - Apply domain knowledge operations
    Knowledge(KnowledgeOp),
    /// Control - Flow control operations
    Control(ControlOp),
}

impl OperatorType {
    /// Get the single-character operator class for audit trail
    pub fn class(&self) -> &'static str {
        match self {
            Self::Seek(_) => "S",
            Self::Memorize(_) => "M",
            Self::Perceive(_) => "P",
            Self::Knowledge(_) => "K",
            Self::Control(_) => "C",
        }
    }

    /// Get a human-readable description of the operator class
    pub fn class_description(&self) -> &'static str {
        match self {
            Self::Seek(_) => "Information Retrieval",
            Self::Memorize(_) => "Store Information",
            Self::Perceive(_) => "Interpret Input",
            Self::Knowledge(_) => "Apply Domain Knowledge",
            Self::Control(_) => "Flow Control",
        }
    }

    /// Check if this operator type can have side effects
    pub fn has_side_effects(&self) -> bool {
        match self {
            Self::Seek(_) => false,      // Read-only
            Self::Memorize(_) => true,   // Writes to memory
            Self::Perceive(_) => false,  // Read-only interpretation
            Self::Knowledge(_) => false, // Pure transformation
            Self::Control(_) => true,    // Flow changes
        }
    }
}

/// Seek operations - Information retrieval
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum SeekOp {
    /// Read a file from the filesystem
    ReadFile { path: String },
    /// Search code with a pattern
    SearchCode { pattern: String, path: Option<String> },
    /// Query the knowledge graph memory
    QueryMemory { query: String, max_hops: u32 },
    /// Search the web
    WebSearch { query: String },
    /// Fetch a URL
    WebFetch { url: String },
    /// List directory contents
    ListDirectory { path: String },
    /// Get git status
    GitStatus { path: Option<String> },
}

/// Memorize operations - Store information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum MemorizeOp {
    /// Store a fact in memory
    StoreFact { key: String, value: String },
    /// Log a decision for audit trail
    LogDecision { decision_id: String, reasoning: String },
    /// Create a checkpoint
    CreateCheckpoint { label: String },
    /// Store execution result
    StoreResult { task_id: String, result_hash: String },
}

/// Perceive operations - Interpret input
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum PerceiveOp {
    /// Parse user intent from natural language
    ParseIntent { input: String },
    /// Extract entities from text
    ExtractEntities { text: String },
    /// Classify input type
    ClassifyInput { input: String },
    /// Parse structured data
    ParseStructured { format: String, data: String },
}

/// Knowledge operations - Apply domain knowledge
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum KnowledgeOp {
    /// Apply a code pattern
    ApplyCodePattern { pattern_id: String },
    /// Lookup API conventions
    LookupApiConvention { api: String },
    /// Apply refactoring rule
    ApplyRefactoring { rule_id: String },
    /// Validate against schema
    ValidateSchema { schema_id: String, data: String },
}

/// Control operations - Flow control
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ControlOp {
    /// Branch based on condition
    Branch { condition: String, true_path: String, false_path: String },
    /// Loop with termination condition
    Loop { condition: String, max_iterations: u32 },
    /// Delegate to another agent
    Delegate { agent_id: String, task: String },
    /// Wait for external event
    Wait { event: String, timeout_ms: u64 },
    /// Terminate execution
    Terminate { reason: String },
}

/// Operator execution result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OperatorResult {
    /// The operator that was executed
    pub operator: OperatorType,
    /// Whether execution succeeded
    pub success: bool,
    /// Result data (if successful)
    pub data: Option<serde_json::Value>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Content hash for audit trail
    pub content_hash: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operator_classes() {
        let seek = OperatorType::Seek(SeekOp::ReadFile { path: "test.rs".to_string() });
        assert_eq!(seek.class(), "S");
        assert!(!seek.has_side_effects());

        let memorize = OperatorType::Memorize(MemorizeOp::StoreFact {
            key: "k".to_string(),
            value: "v".to_string(),
        });
        assert_eq!(memorize.class(), "M");
        assert!(memorize.has_side_effects());

        let perceive = OperatorType::Perceive(PerceiveOp::ParseIntent { input: "test".to_string() });
        assert_eq!(perceive.class(), "P");
        assert!(!perceive.has_side_effects());

        let knowledge = OperatorType::Knowledge(KnowledgeOp::ApplyCodePattern {
            pattern_id: "singleton".to_string(),
        });
        assert_eq!(knowledge.class(), "K");
        assert!(!knowledge.has_side_effects());

        let control = OperatorType::Control(ControlOp::Terminate { reason: "done".to_string() });
        assert_eq!(control.class(), "C");
        assert!(control.has_side_effects());
    }

    #[test]
    fn test_operator_class_descriptions() {
        // Test that class_description returns non-empty strings
        let seek = OperatorType::Seek(SeekOp::ReadFile { path: "test.rs".to_string() });
        assert!(!seek.class_description().is_empty());
        assert!(seek.class_description().contains("Retrieval"));

        let memorize = OperatorType::Memorize(MemorizeOp::StoreFact {
            key: "k".to_string(),
            value: "v".to_string(),
        });
        assert!(!memorize.class_description().is_empty());
        assert!(memorize.class_description().contains("Store"));

        let perceive = OperatorType::Perceive(PerceiveOp::ParseIntent { input: "test".to_string() });
        assert!(!perceive.class_description().is_empty());
        assert!(perceive.class_description().contains("Interpret"));

        let knowledge = OperatorType::Knowledge(KnowledgeOp::ApplyCodePattern {
            pattern_id: "singleton".to_string(),
        });
        assert!(!knowledge.class_description().is_empty());
        assert!(knowledge.class_description().contains("Knowledge"));

        let control = OperatorType::Control(ControlOp::Terminate { reason: "done".to_string() });
        assert!(!control.class_description().is_empty());
        assert!(control.class_description().contains("Control"));
    }

    #[test]
    fn test_operator_class_descriptions_are_unique() {
        // Ensure each operator type has a unique description
        let descriptions = [
            OperatorType::Seek(SeekOp::ReadFile { path: "test".to_string() }).class_description(),
            OperatorType::Memorize(MemorizeOp::StoreFact {
                key: "k".to_string(),
                value: "v".to_string(),
            }).class_description(),
            OperatorType::Perceive(PerceiveOp::ParseIntent { input: "test".to_string() }).class_description(),
            OperatorType::Knowledge(KnowledgeOp::ApplyCodePattern {
                pattern_id: "test".to_string(),
            }).class_description(),
            OperatorType::Control(ControlOp::Terminate { reason: "done".to_string() }).class_description(),
        ];

        // Check all descriptions are unique
        let mut unique = std::collections::HashSet::new();
        for desc in descriptions {
            assert!(unique.insert(desc), "Duplicate description: {}", desc);
        }
    }
}
