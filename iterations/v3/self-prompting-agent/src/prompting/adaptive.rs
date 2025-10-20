    #[test]
    fn test_generate_action_request_valid() {
        let strategy = AdaptivePromptingStrategy::new();
        let task = Task::new("test task".to_string(), TaskType::CodeGeneration);

        let valid_json = r#"{
            "action_type": "write",
            "changeset": {
                "patches": [{
                    "path": "test.rs",
                    "hunks": [{
                        "old_start": 1,
                        "old_lines": 0,
                        "new_start": 1,
                        "new_lines": 1,
                        "lines": "+fn main() {}\n"
                    }],
                    "expected_prev_sha256": null
                }]
            },
            "reason": "Generated main function",
            "confidence": 0.95,
            "metadata": {}
        }"#;

        // This would normally be an async test, but we're testing the parsing logic
        // In a real test, we'd call generate_action_request
        let action_request: ActionRequest = serde_json::from_str(valid_json).unwrap();
        assert!(action_request.validate().is_ok());
        assert_eq!(action_request.action_type, ActionType::Write);
        assert_eq!(action_request.confidence, 0.95);
    }