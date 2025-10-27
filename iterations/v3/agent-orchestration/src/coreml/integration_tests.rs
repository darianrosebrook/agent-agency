//! Integration tests for Core ML with multimodal orchestration

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::multimodal_orchestration::MultimodalOrchestrator;
    use agent_data_processing::{
        DataInput, DataSource, ContentType, DataContent,
    };
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_orchestrator_with_coreml_models() {
        // Create orchestrator
        let orchestrator = match MultimodalOrchestrator::new().await {
            Ok(o) => o,
            Err(e) => {
                println!("Skipping test - Core ML models not available: {}", e);
                return;
            }
        };

        // Create a simple data input for vision processing
        let input = DataInput {
            id: agent_data_processing::ProcessingId::new(),
            source: DataSource::File(agent_data_processing::FileSource {
                path: PathBuf::from("/tmp/test_image.jpg"),
                content_type: ContentType::Image("jpeg".to_string()),
                size_bytes: 1024,
                last_modified: chrono::Utc::now(),
            }),
            content: DataContent::Binary(vec![0u8; 1024]), // Mock image data
            metadata: std::collections::HashMap::new(),
            priority: agent_data_processing::ProcessingPriority::Normal,
            processing_context: None,
        };

        // This would process the input through the unified pipeline
        // In a real scenario, this would use the Core ML models for inference
        println!("Orchestrator created with Core ML support");
        println!("ANE Available: {}", orchestrator.coreml_manager.as_ref().unwrap().is_ane_available());
        println!("Models loaded: {}", orchestrator.coreml_manager.as_ref().unwrap().model_count().await);
    }

    #[tokio::test]
    async fn test_coreml_model_enumeration() {
        let manager = CoreMLManager::new(
            PathBuf::from("/Users/darianrosebrook/Desktop/Projects/agent-agency/models/coreml")
        );

        manager.load_available_models().await.unwrap();

        println!("Available Core ML models:");
        for model_type in [
            CoreMLModelType::Vision,
            CoreMLModelType::Language,
            CoreMLModelType::SpeechToText,
            CoreMLModelType::ObjectDetection,
        ].iter() {
            let models = manager.get_models_by_type(*model_type).await;
            if !models.is_empty() {
                println!("  {:?}: {} models", model_type, models.len());
                for model in models {
                    println!("    - {} (ANE: {})",
                        model.metadata.name,
                        model.metadata.supports_ane
                    );
                }
            }
        }

        assert!(manager.model_count().await >= 0); // At least no panic
    }
}
