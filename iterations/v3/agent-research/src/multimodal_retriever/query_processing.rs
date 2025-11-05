//! Query processing and validation

use schemars::JsonSchema;
use anyhow::Result;

use super::core::MultimodalQuery;
use agent_agency_contracts::types::research::QueryType;

/// Processed query with validated and normalized parameters

use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize) ]
pub struct ProcessedQuery {
    pub text: Option<String>,
    pub image_path: Option<std::path::PathBuf>,
    pub query_type: QueryType,
    pub project_scope: Option<String>,
    pub max_results: usize,
    pub anchor_timestamp: Option<chrono::DateTime<chrono::Utc>>,
    pub time_window_seconds: Option<u64>,
}

/// Query processor for parsing and validating search queries

#[derive(Debug, Serialize, Deserialize) ]
pub struct QueryProcessor {
    config: super::core::MultimodalRetrieverConfig,
}

impl QueryProcessor {
    /// Create a new query processor
    pub fn new(config: super::core::MultimodalRetrieverConfig) -> Result<Self> {
        Ok(Self { config })
    }

    /// Process a simple text query
    pub fn process_query(&self, query: &str, project_scope: Option<&str>) -> Result<ProcessedQuery> {
        Ok(ProcessedQuery {
            text: Some(query.to_string()),
            image_path: None,
            query_type: QueryType::Text,
            project_scope: project_scope.map(|s| s.to_string()),
            max_results: self.config.max_total_results,
            anchor_timestamp: None,
            time_window_seconds: None,
        })
    }

    /// Process a structured multimodal query
    pub fn process_multimodal_query(&self, query: MultimodalQuery) -> Result<ProcessedQuery> {
        Ok(ProcessedQuery {
            text: query.text,
            image_path: query.image_path,
            query_type: query.query_type,
            project_scope: query.project_scope,
            max_results: query.max_results,
            anchor_timestamp: query.anchor_timestamp,
            time_window_seconds: query.time_window_seconds,
        })
    }
}
