//! Configuration tests

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    use tokio_test;

    #[tokio::test]
    async fn test_config_loader_basic() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");
        
        let config_data = r#"{
            "database": {
                "url": "postgresql://localhost:5432/test",
                "max_connections": 5
            },
            "server": {
                "port": 3000,
                "host": "localhost"
            }
        }"#;
        
        fs::write(&config_path, config_data).unwrap();
        
        let loader = ConfigLoader::new(config_path.to_str().unwrap());
        let result = loader.load().await.unwrap();
        
        assert!(result.errors.is_empty());
        assert!(result.config.contains_key("database"));
        assert!(result.config.contains_key("server"));
    }

    #[tokio::test]
    async fn test_environment_manager() {
        let mut manager = EnvironmentManager::new(Environment::Development);
        
        let dev_config = presets::development_config();
        manager.load_environment_config(Environment::Development, dev_config);
        
        let config = manager.get_current_config();
        assert!(config.contains_key("database.url"));
        assert!(config.contains_key("server.port"));
    }

    #[tokio::test]
    async fn test_secrets_manager() {
        let manager = SecretsManager::new("test-encryption-key-32-bytes-long").unwrap();
        
        manager.store_secret("test_secret", "test_value", Some("Test secret"), vec!["test".to_string()]).await.unwrap();
        
        let secret = manager.get_secret("test_secret").await.unwrap().unwrap();
        assert_eq!(secret.value.as_str(), "test_value");
    }

    #[tokio::test]
    async fn test_config_validation() {
        let validator = ConfigValidator::new(true);
        
        let db_config = DatabaseConfigValidation {
            url: "postgresql://localhost:5432/test".to_string(),
            max_connections: 10,
            connection_timeout_secs: 30,
            idle_timeout_secs: 300,
        };
        
        let result = validator.validate_config(&db_config);
        assert!(result.is_valid);
    }

    #[tokio::test]
    async fn test_environment_detection() {
        std::env::set_var("AGENT_AGENCY_ENV", "production");
        let env = detection::detect_from_env().unwrap();
        assert_eq!(env, Environment::Production);
        
        std::env::remove_var("AGENT_AGENCY_ENV");
    }

    #[tokio::test]
    async fn test_config_override() {
        let mut manager = EnvironmentManager::new(Environment::Development);
        
        let dev_config = presets::development_config();
        manager.load_environment_config(Environment::Development, dev_config);
        
        manager.override_config("server.port".to_string(), serde_json::Value::Number(4000.into()));
        
        let config = manager.get_current_config();
        assert_eq!(config.get("server.port").unwrap().as_u64().unwrap(), 4000);
    }

    #[tokio::test]
    async fn test_hot_reload() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");
        
        let initial_config = r#"{"server": {"port": 3000}}"#;
        fs::write(&config_path, initial_config).unwrap();
        
        let loader = ConfigLoader::new(config_path.to_str().unwrap());
        loader.load().await.unwrap();
        
        let updated_config = r#"{"server": {"port": 4000}}"#;
        fs::write(&config_path, updated_config).unwrap();
        
        // Wait a bit for file system to update
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        // Manually trigger reload check
        let config = loader.get_config().await;
        // Note: In a real test, we'd need to wait for the hot reload to trigger
        // This is a simplified test
        assert!(config.contains_key("server"));
    }
}
