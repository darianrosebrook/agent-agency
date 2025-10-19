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
        // 1. Test basic encryption/decryption functionality
        test_basic_encryption_decryption(&manager).await;

        // 2. Test various data types
        test_encryption_different_data_types(&manager).await;

        // 3. Test encryption key validation
        test_encryption_key_validation().await;

        // 4. Test error handling and edge cases
        test_encryption_error_handling(&manager).await;

        // 5. Test security properties
        test_encryption_security_properties(&manager).await;

        // 6. Test concurrent access
        test_concurrent_secret_access(&manager).await;

        // 7. Test integration with list functionality
        let secrets = manager.list_secrets().await.unwrap();
        assert!(secrets.is_empty()); // Should be empty initially

        // Test that we can create the manager successfully
        assert!(manager.get_secret("nonexistent").await.unwrap().is_none());
    }

    async fn test_basic_encryption_decryption(manager: &SecretsManager) {
        let test_secret = "my-super-secret-password";
        let secret_name = "test-secret";

        // Store a secret
        manager
            .store_secret(secret_name, test_secret, Some("Test secret"), vec!["test".to_string()])
            .await
            .expect("Failed to store secret");

        // Retrieve the secret
        let retrieved = manager
            .get_secret(secret_name)
            .await
            .expect("Failed to get secret")
            .expect("Secret not found");

        assert_eq!(retrieved.value.as_str(), test_secret);
        assert_eq!(retrieved.metadata.name, secret_name);
        assert_eq!(retrieved.metadata.description, Some("Test secret".to_string()));
        assert!(retrieved.metadata.tags.contains(&"test".to_string()));
    }

    async fn test_encryption_different_data_types(manager: &SecretsManager) {
        let test_cases = vec![
            ("simple-string", "hello world"),
            ("with-special-chars", "password!@#$%^&*()"),
            ("json-data", r#"{"key": "value", "number": 123}"#),
            ("long-string", &"A".repeat(1000)), // 1KB string
            ("unicode", "héllo wörld 🌍"),
            ("empty-string", ""),
        ];

        for (name, value) in test_cases {
            // Store secret
            manager
                .store_secret(name, value, None, vec![])
                .await
                .expect(&format!("Failed to store secret: {}", name));

            // Retrieve and verify
            let retrieved = manager
                .get_secret(name)
                .await
                .expect(&format!("Failed to get secret: {}", name))
                .expect(&format!("Secret not found: {}", name));

            assert_eq!(retrieved.value.as_str(), value, "Secret value mismatch for: {}", name);
        }
    }

    async fn test_encryption_key_validation() {
        // Test invalid key lengths
        let invalid_keys = vec![
            "", // Empty
            "short", // Too short
            &"A".repeat(31), // 31 bytes
            &"A".repeat(33), // 33 bytes
        ];

        for invalid_key in invalid_keys {
            let result = SecretsManager::new(invalid_key);
            assert!(result.is_err(), "Should reject invalid key length: {}", invalid_key.len());
        }

        // Test valid key
        let valid_key_bytes = [42u8; 32]; // 32 bytes
        let valid_key = base64::engine::general_purpose::STANDARD.encode(valid_key_bytes);
        let result = SecretsManager::new(&valid_key);
        assert!(result.is_ok(), "Should accept valid 32-byte key");
    }

    async fn test_encryption_error_handling(manager: &SecretsManager) {
        // Test retrieving non-existent secret
        let result = manager.get_secret("non-existent").await;
        assert!(result.is_ok(), "Getting non-existent secret should not error");
        assert!(result.unwrap().is_none(), "Non-existent secret should return None");

        // Test overwriting existing secret
        manager
            .store_secret("overwrite-test", "original", None, vec![])
            .await
            .expect("Failed to store original secret");

        let original = manager
            .get_secret("overwrite-test")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(original.value.as_str(), "original");

        // Overwrite
        manager
            .store_secret("overwrite-test", "updated", None, vec![])
            .await
            .expect("Failed to overwrite secret");

        let updated = manager
            .get_secret("overwrite-test")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated.value.as_str(), "updated");
    }

    async fn test_encryption_security_properties(manager: &SecretsManager) {
        let secret_value = "ultra-secret-password-12345";

        // Store secret
        manager
            .store_secret("security-test", secret_value, None, vec![])
            .await
            .expect("Failed to store secret for security test");

        // Retrieve secret
        let retrieved = manager
            .get_secret("security-test")
            .await
            .expect("Failed to retrieve secret for security test")
            .expect("Secret not found for security test");

        // Verify the secret value is correct
        assert_eq!(retrieved.value.as_str(), secret_value);

        // Verify metadata is properly set
        assert_eq!(retrieved.metadata.name, "security-test");
        assert!(retrieved.metadata.created_at <= chrono::Utc::now());
        assert!(retrieved.metadata.updated_at <= chrono::Utc::now());
    }

    async fn test_concurrent_secret_access(manager: &SecretsManager) {
        use std::sync::Arc;
        use tokio::task;

        let manager = Arc::new(manager);
        let mut handles = vec![];

        // Spawn multiple tasks that concurrently access secrets
        for i in 0..10 {
            let manager_clone = Arc::clone(&manager);
            let handle = task::spawn(async move {
                let secret_name = format!("concurrent-test-{}", i);
                let secret_value = format!("value-{}", i);

                // Store secret
                manager_clone
                    .store_secret(&secret_name, &secret_value, None, vec![])
                    .await
                    .expect(&format!("Failed to store concurrent secret {}", i));

                // Retrieve and verify
                let retrieved = manager_clone
                    .get_secret(&secret_name)
                    .await
                    .expect(&format!("Failed to get concurrent secret {}", i))
                    .expect(&format!("Concurrent secret {} not found", i));

                assert_eq!(retrieved.value.as_str(), secret_value);
            });
            handles.push(handle);
        }

        // Wait for all concurrent operations to complete
        for handle in handles {
            handle.await.expect("Concurrent secret access task failed");
        }
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

        // Test 1: Basic reload functionality
        test_basic_hot_reload(&loader, &config_path).await;

        // Test 2: Invalid config handling
        test_invalid_config_handling(&loader, &config_path).await;

        // Test 3: Concurrent access during reload
        test_concurrent_reload_access(&loader, &config_path).await;

        // Test 4: Rapid successive changes
        test_rapid_config_changes(&loader, &config_path).await;

        // Test 5: Large config files
        test_large_config_handling(&loader, &config_path).await;
    }

    async fn test_basic_hot_reload(loader: &ConfigLoader, config_path: &std::path::Path) {
        // Test basic config reload functionality
        let config1 = loader.get_config().await;
        assert_eq!(config1.get("server.port").unwrap().as_u64().unwrap(), 3000);

        // Update config file
        let updated_config = r#"{"server": {"port": 4000, "host": "localhost"}}"#;
        fs::write(config_path, updated_config).unwrap();

        // Wait for file system and reload
        tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;

        // Reload and verify
        loader.load().await.unwrap();
        let config2 = loader.get_config().await;
        assert_eq!(config2.get("server.port").unwrap().as_u64().unwrap(), 4000);
        assert_eq!(config2.get("server.host").unwrap().as_str().unwrap(), "localhost");

        // Verify old values are gone
        assert!(config2.get("server.port").unwrap().as_u64().unwrap() != 3000);
    }

    async fn test_invalid_config_handling(loader: &ConfigLoader, config_path: &std::path::Path) {
        // Write valid config first
        let valid_config = r#"{"server": {"port": 3000}}"#;
        fs::write(config_path, valid_config).unwrap();
        loader.load().await.unwrap();

        let config_before = loader.get_config().await;
        assert_eq!(config_before.get("server.port").unwrap().as_u64().unwrap(), 3000);

        // Write invalid JSON
        let invalid_config = r#"{"server": {"port": 4000, "invalid": }"#; // Missing value
        fs::write(config_path, invalid_config).unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;

        // Attempt reload - should handle error gracefully and keep old config
        let reload_result = loader.load().await;
        assert!(reload_result.is_err(), "Should fail to load invalid config");

        // Config should remain unchanged
        let config_after = loader.get_config().await;
        assert_eq!(config_after.get("server.port").unwrap().as_u64().unwrap(), 3000);

        // Restore valid config
        fs::write(config_path, valid_config).unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
        loader.load().await.unwrap();
    }

    async fn test_concurrent_reload_access(loader: &ConfigLoader, config_path: &std::path::Path) {
        use std::sync::Arc;
        use tokio::task;

        let loader = Arc::new(loader);
        let mut handles = vec![];

        // Spawn multiple tasks that concurrently access config during reload
        for i in 0..5 {
            let loader_clone = Arc::clone(&loader);
            let config_path = config_path.to_path_buf();
            let handle = task::spawn(async move {
                // Write new config
                let new_config = format!(r#"{{"server": {{"port": {}}}}}"#, 3000 + i);
                fs::write(&config_path, new_config).unwrap();

                // Small delay to simulate concurrent access
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

                // Try to reload
                let _ = loader_clone.load().await;

                // Read config
                let config = loader_clone.get_config().await;
                let port = config.get("server.port").unwrap().as_u64().unwrap();
                assert!(port >= 3000 && port <= 3004, "Port should be within expected range: {}", port);
            });
            handles.push(handle);
        }

        // Wait for all concurrent operations
        for handle in handles {
            handle.await.expect("Concurrent reload task failed");
        }
    }

    async fn test_rapid_config_changes(loader: &ConfigLoader, config_path: &std::path::Path) {
        let initial_config = r#"{"server": {"port": 3000}}"#;
        fs::write(config_path, initial_config).unwrap();
        loader.load().await.unwrap();

        // Rapidly change config multiple times
        for port in 3001..3011 {
            let config = format!(r#"{{"server": {{"port": {}}}}}"#, port);
            fs::write(config_path, config).unwrap();

            // Very short delay between changes
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

            // Try to reload
            let _ = loader.load().await;
        }

        // Final check - should have some reasonable port value
        let final_config = loader.get_config().await;
        let final_port = final_config.get("server.port").unwrap().as_u64().unwrap();
        assert!(final_port >= 3000 && final_port <= 3010, "Final port should be reasonable: {}", final_port);
    }

    async fn test_large_config_handling(loader: &ConfigLoader, config_path: &std::path::Path) {
        // Create a large config with many entries
        let mut large_config = serde_json::Map::new();
        let mut server_config = serde_json::Map::new();

        server_config.insert("port".to_string(), serde_json::Value::Number(3000.into()));
        server_config.insert("host".to_string(), serde_json::Value::String("localhost".to_string()));

        // Add many dummy entries to make it large
        for i in 0..1000 {
            server_config.insert(format!("dummy_{}", i), serde_json::Value::String(format!("value_{}", i)));
        }

        large_config.insert("server".to_string(), serde_json::Value::Object(server_config));

        let config_json = serde_json::to_string(&large_config).unwrap();
        fs::write(config_path, config_json).unwrap();

        // Test loading large config
        let start_time = std::time::Instant::now();
        loader.load().await.expect("Should handle large config");
        let load_time = start_time.elapsed();

        // Should load within reasonable time (less than 1 second for 1000 entries)
        assert!(load_time < std::time::Duration::from_secs(1), "Large config took too long: {:?}", load_time);

        // Verify config is accessible
        let loaded_config = loader.get_config().await;
        assert_eq!(loaded_config.get("server.port").unwrap().as_u64().unwrap(), 3000);
        assert_eq!(loaded_config.get("server.host").unwrap().as_str().unwrap(), "localhost");
        assert!(loaded_config.get("server.dummy_999").is_some());
    }
}
