//! Tests for learning bridge functionality

use chrono::Utc;
use self_prompting_agent::learning_bridge::*;
use self_prompting_agent::types::SelfPromptingAgentError;

#[tokio::test]
async fn test_learning_bridge_creation() {
    let bridge = LearningBridge::new();
    // Test that bridge is created successfully - it's a unit struct
    assert_eq!(std::mem::size_of::<LearningBridge>(), 0); // Unit struct
}

#[tokio::test]
async fn test_process_signal() {
    let bridge = LearningBridge::new();
    let signal = LearningSignal {
        signal_type: "task_completion".to_string(),
        value: 1.0,
        context: "test_context".to_string(),
        timestamp: Utc::now(),
    };

    let result = bridge.process_signal(signal).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_process_signal_different_types() {
    let bridge = LearningBridge::new();

    let signals = vec![
        LearningSignal {
            signal_type: "success".to_string(),
            value: 1.0,
            context: "positive_outcome".to_string(),
            timestamp: Utc::now(),
        },
        LearningSignal {
            signal_type: "failure".to_string(),
            value: -1.0,
            context: "error_occurred".to_string(),
            timestamp: Utc::now(),
        },
        LearningSignal {
            signal_type: "performance".to_string(),
            value: 0.75,
            context: "execution_metrics".to_string(),
            timestamp: Utc::now(),
        },
    ];

    for signal in signals {
        let result = bridge.process_signal(signal).await;
        assert!(result.is_ok(), "Failed to process signal");
    }
}

#[tokio::test]
async fn test_get_recommendations() {
    let bridge = LearningBridge::new();

    let result = bridge.get_recommendations("test context").await;
    assert!(result.is_ok());

    let recommendations = result.unwrap();
    assert_eq!(recommendations.len(), 2);
    assert!(recommendations.contains(&"Consider using more specific prompts".to_string()));
    assert!(recommendations.contains(&"Try breaking complex tasks into smaller steps".to_string()));
}

#[tokio::test]
async fn test_get_recommendations_different_contexts() {
    let bridge = LearningBridge::new();

    let contexts = vec![
        "code generation",
        "error handling",
        "performance optimization",
        "user interaction",
    ];

    for context in contexts {
        let result = bridge.get_recommendations(context).await;
        assert!(result.is_ok(), "Failed to get recommendations for context: {}", context);
        let recommendations = result.unwrap();
        assert_eq!(recommendations.len(), 2); // Always returns 2 stub recommendations
    }
}

#[test]
fn test_learning_signal_creation() {
    let timestamp = Utc::now();
    let signal = LearningSignal {
        signal_type: "test_signal".to_string(),
        value: 0.85,
        context: "test_context".to_string(),
        timestamp,
    };

    assert_eq!(signal.signal_type, "test_signal");
    assert_eq!(signal.value, 0.85);
    assert_eq!(signal.context, "test_context");
    assert_eq!(signal.timestamp, timestamp);
}

#[test]
fn test_learning_signal_debug() {
    let signal = LearningSignal {
        signal_type: "debug_test".to_string(),
        value: 0.5,
        context: "debug_context".to_string(),
        timestamp: Utc::now(),
    };

    let debug_str = format!("{:?}", signal);
    assert!(debug_str.contains("debug_test"));
    assert!(debug_str.contains("debug_context"));
}

#[test]
fn test_learning_signal_clone() {
    let original = LearningSignal {
        signal_type: "original".to_string(),
        value: 1.0,
        context: "original_context".to_string(),
        timestamp: Utc::now(),
    };

    let cloned = original.clone();

    assert_eq!(original.signal_type, cloned.signal_type);
    assert_eq!(original.value, cloned.value);
    assert_eq!(original.context, cloned.context);
    assert_eq!(original.timestamp, cloned.timestamp);
}

#[tokio::test]
async fn test_reflexive_learning_system_creation() {
    let system = ReflexiveLearningSystem::new();
    // Test that system is created successfully - it's a unit struct
    assert_eq!(std::mem::size_of::<ReflexiveLearningSystem>(), 0); // Unit struct
}

#[tokio::test]
async fn test_reflexive_learning_system_process_signal() {
    let system = ReflexiveLearningSystem::new();
    let signal = LearningSignal {
        signal_type: "system_test".to_string(),
        value: 0.9,
        context: "system_context".to_string(),
        timestamp: Utc::now(),
    };

    let result = system.process_signal(signal).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_reflexive_learning_system_generate_insights() {
    let system = ReflexiveLearningSystem::new();

    let result = system.generate_insights().await;
    assert!(result.is_ok());

    let insights = result.unwrap();
    assert_eq!(insights.len(), 1);
    assert_eq!(insights[0], "Learning system operational");
}

// Debug implementations are not available for these structs

#[test]
fn test_learning_signal_partial_eq() {
    let timestamp = Utc::now();
    let signal1 = LearningSignal {
        signal_type: "test".to_string(),
        value: 1.0,
        context: "ctx".to_string(),
        timestamp,
    };

    let signal2 = LearningSignal {
        signal_type: "test".to_string(),
        value: 1.0,
        context: "ctx".to_string(),
        timestamp,
    };

    // LearningSignal doesn't implement PartialEq, so we can't test equality directly
    // But we can test that the fields are accessible
    assert_eq!(signal1.signal_type, signal2.signal_type);
    assert_eq!(signal1.value, signal2.value);
    assert_eq!(signal1.context, signal2.context);
    assert_eq!(signal1.timestamp, signal2.timestamp);
}
