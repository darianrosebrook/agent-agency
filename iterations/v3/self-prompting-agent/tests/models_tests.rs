#[cfg(test)]
mod tests {
    use self_prompting_agent::models::{ModelRegistry, ModelProvider, GenerationOptions, OllamaProvider, ExpertSelectionRouter};
    use self_prompting_agent::SelfPromptingAgentError;
    use async_trait::async_trait;
    use std::sync::Arc;

    // Mock model provider for testing
    struct MockModelProvider {
        name: String,
        available: bool,
    }

    #[async_trait]
    impl ModelProvider for MockModelProvider {
        async fn generate(&self, prompt: &str, _options: &GenerationOptions) -> Result<String, SelfPromptingAgentError> {
            if !self.available {
                return Err(SelfPromptingAgentError::ModelProvider(format!("Model '{}' is unavailable", self.name)));
            }
            Ok(format!("Mock response to: {}", prompt))
        }

        fn name(&self) -> &str {
            &self.name
        }

        async fn is_available(&self) -> bool {
            self.available
        }
    }

    #[tokio::test]
    async fn test_generation_options_default() {
        let options = GenerationOptions::default();

        assert_eq!(options.max_tokens, Some(2048));
        assert_eq!(options.temperature, Some(0.7));
        assert_eq!(options.top_p, Some(0.9));
        assert!(options.stop_sequences.is_empty());
        assert!(options.model_name.is_none());
    }

    #[tokio::test]
    async fn test_generation_options_custom() {
        let options = GenerationOptions {
            max_tokens: Some(1024),
            temperature: Some(0.5),
            top_p: Some(0.8),
            stop_sequences: vec!["STOP".to_string(), "END".to_string()],
            model_name: Some("custom-model".to_string()),
        };

        assert_eq!(options.max_tokens, Some(1024));
        assert_eq!(options.temperature, Some(0.5));
        assert_eq!(options.top_p, Some(0.8));
        assert_eq!(options.stop_sequences, vec!["STOP".to_string(), "END".to_string()]);
        assert_eq!(options.model_name, Some("custom-model".to_string()));
    }

    #[tokio::test]
    async fn test_model_registry_creation() {
        let registry = ModelRegistry::new();
        assert!(registry.list_providers().is_empty());
        assert!(registry.get_default_provider().is_none());
    }

    #[tokio::test]
    async fn test_model_registry_register_provider() {
        let mut registry = ModelRegistry::new();

        let provider = Arc::new(MockModelProvider {
            name: "test-provider".to_string(),
            available: true,
        });

        registry.register_provider("test-provider".to_string(), provider);

        let providers = registry.list_providers();
        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0], "test-provider");
        assert!(registry.get_default_provider().is_some());
    }

    #[tokio::test]
    async fn test_model_registry_get_provider() {
        let mut registry = ModelRegistry::new();

        let provider = Arc::new(MockModelProvider {
            name: "test-provider".to_string(),
            available: true,
        });

        registry.register_provider("test-provider".to_string(), provider);

        let retrieved = registry.get_provider("test-provider");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name(), "test-provider");

        let non_existent = registry.get_provider("non-existent");
        assert!(non_existent.is_none());
    }

    #[tokio::test]
    async fn test_model_registry_generate_with_provider() {
        let mut registry = ModelRegistry::new();

        let provider = Arc::new(MockModelProvider {
            name: "test-provider".to_string(),
            available: true,
        });

        registry.register_provider("test-provider".to_string(), provider);

        let options = GenerationOptions::default();
        let result = registry.generate_with_provider("test-provider", "Hello world", &options).await.unwrap();

        assert_eq!(result, "Mock response to: Hello world");
    }

    #[tokio::test]
    async fn test_model_registry_generate_with_default_provider() {
        let mut registry = ModelRegistry::new();

        let provider = Arc::new(MockModelProvider {
            name: "test-provider".to_string(),
            available: true,
        });

        registry.register_provider("test-provider".to_string(), provider);

        let options = GenerationOptions::default();
        let result = registry.generate("Hello world", &options).await.unwrap();

        assert_eq!(result, "Mock response to: Hello world");
    }

    #[tokio::test]
    async fn test_model_registry_generate_unavailable_provider() {
        let mut registry = ModelRegistry::new();

        let provider = Arc::new(MockModelProvider {
            name: "test-provider".to_string(),
            available: false,
        });

        registry.register_provider("test-provider".to_string(), provider);

        let options = GenerationOptions::default();
        let result = registry.generate_with_provider("test-provider", "Hello world", &options).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            SelfPromptingAgentError::ModelProvider(msg) => {
                assert!(msg.contains("test-provider"));
            }
            _ => panic!("Expected ModelProvider error"),
        }
    }

    #[tokio::test]
    async fn test_model_registry_no_default_provider() {
        let registry = ModelRegistry::new();

        let options = GenerationOptions::default();
        let result = registry.generate("Hello world", &options).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            SelfPromptingAgentError::ModelProvider(msg) => {
                assert_eq!(msg, "No default provider configured");
            }
            _ => panic!("Expected ModelProvider error"),
        }
    }

    #[tokio::test]
    async fn test_model_registry_provider_not_found() {
        let registry = ModelRegistry::new();

        let options = GenerationOptions::default();
        let result = registry.generate_with_provider("non-existent", "Hello world", &options).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            SelfPromptingAgentError::ModelProvider(msg) => {
                assert_eq!(msg, "Provider 'non-existent' not found");
            }
            _ => panic!("Expected ModelProvider error"),
        }
    }

    #[tokio::test]
    async fn test_ollama_provider_creation() {
        let provider = OllamaProvider::new("http://localhost:11434".to_string(), "llama2".to_string());

        assert_eq!(provider.name(), "Ollama");
    }

    #[tokio::test]
    async fn test_ollama_provider_with_custom_model() {
        let provider = OllamaProvider::new("http://localhost:11434".to_string(), "codellama".to_string());

        assert_eq!(provider.name(), "Ollama");
    }

    #[tokio::test]
    async fn test_expert_selection_router_creation() {
        let registry = Arc::new(ModelRegistry::new());
        let router = ExpertSelectionRouter::new(registry.clone());

        // Test that router can select an expert when no providers are available
        let result = router.select_expert("test task").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            SelfPromptingAgentError::ModelProvider(msg) => {
                assert_eq!(msg, "No providers available");
            }
            _ => panic!("Expected ModelProvider error"),
        }
    }

    #[tokio::test]
    async fn test_mock_model_provider() {
        let provider = MockModelProvider {
            name: "mock-provider".to_string(),
            available: true,
        };

        assert_eq!(provider.name(), "mock-provider");
        assert!(provider.is_available().await);

        let options = GenerationOptions::default();
        let result = provider.generate("test prompt", &options).await.unwrap();
        assert_eq!(result, "Mock response to: test prompt");
    }

    #[tokio::test]
    async fn test_mock_model_provider_unavailable() {
        let provider = MockModelProvider {
            name: "mock-provider".to_string(),
            available: false,
        };

        assert_eq!(provider.name(), "mock-provider");
        assert!(!provider.is_available().await);

        let options = GenerationOptions::default();
        let result = provider.generate("test prompt", &options).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            SelfPromptingAgentError::ModelProvider(msg) => {
                assert!(msg.contains("mock-provider"));
                assert!(msg.contains("unavailable"));
            }
            _ => panic!("Expected ModelProvider error"),
        }
    }
}
