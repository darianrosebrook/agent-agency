//! Content processing and management

use std::sync::Arc;
use crate::{ConfigurationUpdate, ContentProcessingConfig, ContentProcessor};

/// Content processor manager

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ContentProcessorManagerr {
    processor: Arc<ContentProcessor>,
    config: ResearchAgentConfig,
}

impl ContentProcessorManager {
    /// Create a new content processor manager
    pub async fn new(config: ResearchAgentConfig) -> Result<Self> {
        let processor = Arc::new(ContentProcessor::new(ContentProcessingConfig {
            enable_cleaning: true,
            enable_markdown: true,
            enable_text_extraction: true,
            max_content_length: 1000000,
            enable_summarization: false,
        }));

        Ok(Self { processor, config })
    }

    /// Process content
    pub async fn process_content(&self, content: &str) -> Result<Option<crate::ProcessedContent>> {
        Ok(self.processor.process_content(content).await?)
    }

    /// Update configuration
    pub async fn update_config(&self, _update: ConfigurationUpdate) -> Result<()> {
        Ok(())
    }
}
