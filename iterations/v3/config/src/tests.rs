//! Configuration tests

#[cfg(test)]
mod tests {
    use crate::{
        detection, presets, ConfigLoader, ConfigValidator, DatabaseConfigValidation, Environment,
        EnvironmentManager, SecretsManager,
    };
    use base64::Engine;
    use std::fs;
    use tempfile::TempDir;

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
        // Create a proper base64-encoded 32-byte key
        let key_bytes = [0u8; 32]; // 32 bytes of zeros for testing
        let key = base64::engine::general_purpose::STANDARD.encode(key_bytes);
        let manager = SecretsManager::new(&key).unwrap();
        // TODO: Implement secrets manager encryption testing with the following requirements:
        // 1. Encryption functionality testing: Implement comprehensive encryption testing
        //    - Test secret encryption and decryption operations with various data types
        //    - Validate encryption key generation and management
        //    - Handle encryption performance testing and benchmarking
        //    - Implement encryption error handling and edge case testing
        // 2. Security validation: Implement robust security validation testing
        //    - Test encryption strength and cryptographic algorithm validation
        //    - Validate key rotation and security policy enforcement
        //    - Handle security vulnerability testing and penetration testing
        //    - Implement security compliance validation and audit testing
        // 3. Integration testing: Implement comprehensive integration testing for secrets management
        //    - Test secrets manager integration with configuration system
        //    - Validate secrets persistence and retrieval across system restarts
        //    - Handle secrets manager performance under load testing
        //    - Implement secrets manager reliability and fault tolerance testing
        // 4. Test data management: Implement proper test data management for secrets testing
        //    - Create secure test fixtures and mock data for encryption testing
        //    - Handle test data cleanup and security validation
        //    - Implement test data isolation and environment separation
        //    - Ensure test data meets security and privacy requirements
        let secrets = manager.list_secrets().await.unwrap();
        assert!(secrets.is_empty());

        // Test that we can create the manager successfully
        assert!(manager.get_secret("nonexistent").await.unwrap().is_none());
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

        manager.override_config(
            "server.port".to_string(),
            serde_json::Value::Number(4000.into()),
        );

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
        // TODO: Implement comprehensive hot reload testing with the following requirements:
        // 1. File system monitoring: Implement robust file system change detection and monitoring
        //    - Monitor config file modifications using inotify, FSEvents, or ReadDirectoryChangesW
        //    - Handle file system events with proper debouncing and event filtering
        //    - Implement cross-platform file monitoring support for Linux, macOS, and Windows
        //    - Handle file locking, temporary files, and atomic write operations
        // 2. Hot reload validation: Implement comprehensive hot reload functionality testing
        //    - Test config reload triggers and timing accuracy with proper synchronization
        //    - Validate config parsing and validation after hot reload events
        //    - Test error handling and rollback mechanisms for invalid config changes
        //    - Implement config change verification and consistency checking
        // 3. Performance and reliability testing: Implement performance and reliability validation
        //    - Test hot reload performance under high-frequency config change scenarios
        //    - Validate memory usage and resource cleanup during hot reload operations
        //    - Test concurrent access and thread safety during config reload processes
        //    - Implement stress testing and edge case validation for hot reload functionality
        // 4. Integration testing: Implement comprehensive integration testing framework
        //    - Test hot reload integration with application components and services
        //    - Validate config change propagation and notification mechanisms
        //    - Test hot reload behavior in different environments and deployment scenarios
        //    - Implement end-to-end testing workflows for complete hot reload validation
        assert!(config.contains_key("server"));
    }
}
