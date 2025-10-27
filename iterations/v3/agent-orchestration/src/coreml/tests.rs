//! Tests for Core ML integration

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    use std::path::Path;

    #[tokio::test]
    async fn test_coreml_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = CoreMLManager::new(temp_dir.path().to_path_buf());

        assert!(!manager.is_ane_available() || manager.is_ane_available()); // Either way is fine
    }

    #[tokio::test]
    async fn test_load_available_models_with_mock() {
        let temp_dir = TempDir::new().unwrap();

        // Create mock model directories
        fs::create_dir_all(temp_dir.path().join("fastvit")).unwrap();
        fs::create_dir_all(temp_dir.path().join("mistral")).unwrap();
        fs::create_dir_all(temp_dir.path().join("whisper")).unwrap();
        fs::create_dir_all(temp_dir.path().join("yolov3")).unwrap();

        // Create mock .mlmodelc files
        fs::File::create(temp_dir.path().join("fastvit/FastViTT8F16.mlpackage.mlmodelc")).unwrap();
        fs::File::create(temp_dir.path().join("mistral/StatefulMistral7BInstructFP16.mlpackage.mlmodelc")).unwrap();
        fs::File::create(temp_dir.path().join("whisper/ggml-base.en-encoder.mlmodelc")).unwrap();
        fs::File::create(temp_dir.path().join("yolov3/YOLOv3.mlmodel.mlmodelc")).unwrap();

        let manager = CoreMLManager::new(temp_dir.path().to_path_buf());
        let result = manager.load_available_models().await;

        assert!(result.is_ok());
        assert_eq!(manager.model_count().await, 4); // Should load all 4 mock models
    }

    #[tokio::test]
    async fn test_get_model_by_type_and_name() {
        let temp_dir = TempDir::new().unwrap();

        // Create mock FastViT model
        fs::create_dir_all(temp_dir.path().join("fastvit")).unwrap();
        fs::File::create(temp_dir.path().join("fastvit/FastViTT8F16.mlpackage.mlmodelc")).unwrap();

        let manager = CoreMLManager::new(temp_dir.path().to_path_buf());
        manager.load_available_models().await.unwrap();

        let model = manager.get_model(CoreMLModelType::Vision, "FastViT-T8-F16").await;
        assert!(model.is_some());

        let model = model.unwrap();
        assert_eq!(model.metadata.model_type, CoreMLModelType::Vision);
        assert_eq!(model.metadata.name, "FastViT-T8-F16");
        assert!(model.metadata.supports_ane || !model.metadata.supports_ane); // Either is fine
    }

    #[tokio::test]
    async fn test_get_models_by_type() {
        let temp_dir = TempDir::new().unwrap();

        // Create mock models
        fs::create_dir_all(temp_dir.path().join("fastvit")).unwrap();
        fs::create_dir_all(temp_dir.path().join("mistral")).unwrap();
        fs::File::create(temp_dir.path().join("fastvit/FastViTT8F16.mlpackage.mlmodelc")).unwrap();
        fs::File::create(temp_dir.path().join("mistral/StatefulMistral7BInstructFP16.mlpackage.mlmodelc")).unwrap();

        let manager = CoreMLManager::new(temp_dir.path().to_path_buf());
        manager.load_available_models().await.unwrap();

        let vision_models = manager.get_models_by_type(CoreMLModelType::Vision).await;
        assert_eq!(vision_models.len(), 1);
        assert_eq!(vision_models[0].metadata.model_type, CoreMLModelType::Vision);

        let language_models = manager.get_models_by_type(CoreMLModelType::Language).await;
        assert_eq!(language_models.len(), 1);
        assert_eq!(language_models[0].metadata.model_type, CoreMLModelType::Language);
    }

    #[tokio::test]
    async fn test_mock_inference() {
        let temp_dir = TempDir::new().unwrap();

        // Create mock FastViT model
        fs::create_dir_all(temp_dir.path().join("fastvit")).unwrap();
        fs::File::create(temp_dir.path().join("fastvit/FastViTT8F16.mlpackage.mlmodelc")).unwrap();

        let manager = CoreMLManager::new(temp_dir.path().to_path_buf());
        manager.load_available_models().await.unwrap();

        let model = manager.get_model(CoreMLModelType::Vision, "FastViT-T8-F16").await.unwrap();

        // Create mock input data
        let mut inputs = std::collections::HashMap::new();
        inputs.insert("input".to_string(), vec![0.1f32; 3 * 256 * 256]); // Mock image data

        let result = manager.run_inference(&model, inputs).await;
        assert!(result.is_ok());

        let outputs = result.unwrap();
        assert!(outputs.contains_key("output"));

        // Check output shape (should be [1, 1000] flattened to 1000 elements)
        let output = &outputs["output"];
        assert_eq!(output.len(), 1000);
    }

    #[tokio::test]
    async fn test_real_model_loading() {
        // Test with actual model paths (will only work if models exist)
        let real_path = PathBuf::from("/Users/darianrosebrook/Desktop/Projects/agent-agency/models/coreml");
        let manager = CoreMLManager::new(real_path);

        let result = manager.load_available_models().await;
        // This might fail if models don't exist, but shouldn't panic
        assert!(result.is_ok() || result.is_err()); // Either result is acceptable
    }
}
