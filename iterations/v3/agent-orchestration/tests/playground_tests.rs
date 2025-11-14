//! Direct playground tests that can run without the full evaluation framework

#[cfg(feature = "evaluation")]
mod playground_tests {
    use agent_orchestration::evaluation::playground::PlaygroundManager;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_playground_manager_creation() {
        let manager = PlaygroundManager::new();
        // Verify manager was created successfully by checking it can create a scenario
        let result = manager.setup_scenario("test-init").await;
        assert!(result.is_ok());
        // Cleanup
        let _ = manager.cleanup_scenario("test-init").await;
    }

    #[tokio::test]
    async fn test_setup_and_cleanup_scenario() {
        let temp_dir = TempDir::new().unwrap();
        let manager = PlaygroundManager::with_root(temp_dir.path().to_path_buf());

        let scenario_id = "test-scenario-001";

        // Setup scenario
        let result = manager.setup_scenario(scenario_id).await;
        assert!(result.is_ok());

        // Verify directory exists
        let scenario_dir = manager.get_scenario_dir(scenario_id);
        assert!(scenario_dir.exists());

        // Cleanup scenario
        let result = manager.cleanup_scenario(scenario_id).await;
        assert!(result.is_ok());

        // Verify directory is removed
        assert!(!scenario_dir.exists());
    }

    #[tokio::test]
    async fn test_create_test_file() {
        let temp_dir = TempDir::new().unwrap();
        let manager = PlaygroundManager::with_root(temp_dir.path().to_path_buf());

        let scenario_id = "test-scenario-002";
        manager.setup_scenario(scenario_id).await.unwrap();

        let file_path = manager
            .create_test_file(scenario_id, "test.rs", "fn main() { println!(\"Hello\"); }")
            .await;

        assert!(file_path.is_ok());
        let path = file_path.unwrap();
        assert!(path.exists());

        // Verify content
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("Hello"));
    }

    #[tokio::test]
    async fn test_create_broken_file() {
        let temp_dir = TempDir::new().unwrap();
        let manager = PlaygroundManager::with_root(temp_dir.path().to_path_buf());

        let scenario_id = "test-scenario-003";
        manager.setup_scenario(scenario_id).await.unwrap();

        let file_path = manager
            .create_broken_file(scenario_id, "broken.rs", "compilation")
            .await;
        assert!(file_path.is_ok());

        let path = file_path.unwrap();
        assert!(path.exists());

        // Verify content has error
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("Type mismatch") || content.contains("\"hello\""));
    }

    #[tokio::test]
    async fn test_scaffold_comprehensive_broken_files() {
        let temp_dir = TempDir::new().unwrap();
        let manager = PlaygroundManager::with_root(temp_dir.path().to_path_buf());

        let scenario_id = "test-scenario-comprehensive";

        // Scaffold comprehensive broken files
        let result = manager
            .scaffold_comprehensive_broken_files(scenario_id)
            .await;
        assert!(result.is_ok());

        let created_files = result.unwrap();
        assert_eq!(created_files.len(), 3);

        // Verify all three files exist
        let rust_path = manager.get_scenario_dir(scenario_id).join("broken-rust.rs");
        let types_path = manager
            .get_scenario_dir(scenario_id)
            .join("broken-types.ts");
        let python_path = manager
            .get_scenario_dir(scenario_id)
            .join("broken-python.py");

        assert!(rust_path.exists(), "broken-rust.rs should exist");
        assert!(types_path.exists(), "broken-types.ts should exist");
        assert!(python_path.exists(), "broken-python.py should exist");

        // Verify Rust file content
        let rust_content = fs::read_to_string(&rust_path).unwrap();
        assert!(rust_content.contains("Duplicate struct definition"));
        assert!(rust_content.contains("Type mismatch"));
        assert!(rust_content.contains("TODO:"));
        assert!(rust_content.contains("PLACEHOLDER:"));
        assert!(rust_content.contains("MOCK DATA:"));

        // Verify TypeScript file content
        let types_content = fs::read_to_string(&types_path).unwrap();
        assert!(types_content.contains("Duplicate interface definition"));
        assert!(types_content.contains("Type mismatch"));
        assert!(types_content.contains("TODO:"));
        assert!(types_content.contains("PLACEHOLDER:"));
        assert!(types_content.contains("MOCK DATA:"));

        // Verify Python file content
        let python_content = fs::read_to_string(&python_path).unwrap();
        assert!(python_content.contains("Missing import"));
        assert!(python_content.contains("TODO:"));
        assert!(python_content.contains("PLACEHOLDER:"));
        assert!(python_content.contains("MOCK DATA:"));
        assert!(python_content.contains("broken_indentation"));
    }
}
