//! Tests for sandbox environment functionality

use std::path::PathBuf;
use self_prompting_agent::sandbox::*;
use self_prompting_agent::types::SelfPromptingAgentError;

#[tokio::test]
async fn test_sandbox_environment_creation_with_root_path() {
    let root_path = Some("/tmp/test_sandbox".to_string());
    let sandbox = SandboxEnvironment::new(root_path.clone()).unwrap();

    let status = sandbox.status();
    assert_eq!(status.root_path, Some(PathBuf::from("/tmp/test_sandbox")));
    assert_eq!(status.active, true);
    assert_eq!(status.security_level, SecurityLevel::Medium);
}

#[tokio::test]
async fn test_sandbox_environment_creation_without_root_path() {
    let sandbox = SandboxEnvironment::new(None).unwrap();

    let status = sandbox.status();
    assert_eq!(status.root_path, None);
    assert_eq!(status.active, true);
}

#[tokio::test]
async fn test_sandbox_execute_in_sandbox_success() {
    let sandbox = SandboxEnvironment::new(None).unwrap();

    let result = sandbox.execute_in_sandbox("test_operation").await.unwrap();
    assert_eq!(result, "Test executed successfully");
}

#[tokio::test]
async fn test_sandbox_execute_in_sandbox_invalid_operation() {
    let sandbox = SandboxEnvironment::new(None).unwrap();

    let result = sandbox.execute_in_sandbox("invalid_operation").await;
    assert!(result.is_err());

    match result.unwrap_err() {
        SelfPromptingAgentError::Sandbox(msg) => {
            assert_eq!(msg, "Operation not allowed in sandbox");
        }
        _ => panic!("Expected Sandbox error"),
    }
}

#[tokio::test]
async fn test_sandbox_execute_in_sandbox_generic_operation() {
    let sandbox = SandboxEnvironment::new(None).unwrap();

    let result = sandbox.execute_in_sandbox("custom_operation").await.unwrap();
    assert_eq!(result, "Executed: custom_operation");
}

#[tokio::test]
async fn test_sandbox_validate_path_allowed() {
    let sandbox = SandboxEnvironment::new(None).unwrap();

    let allowed_path = PathBuf::from("/tmp/test_file");
    let result = sandbox.validate_path(&allowed_path);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_sandbox_validate_path_not_allowed() {
    let sandbox = SandboxEnvironment::new(None).unwrap();

    let forbidden_path = PathBuf::from("/etc/passwd");
    let result = sandbox.validate_path(&forbidden_path);
    assert!(result.is_err());

    match result.unwrap_err() {
        SelfPromptingAgentError::Sandbox(msg) => {
            assert!(msg.contains("Path not allowed"));
            assert!(msg.contains("/etc/passwd"));
        }
        _ => panic!("Expected Sandbox error"),
    }
}

#[tokio::test]
async fn test_sandbox_validate_path_var_tmp_allowed() {
    let sandbox = SandboxEnvironment::new(None).unwrap();

    let allowed_path = PathBuf::from("/var/tmp/cache_file");
    let result = sandbox.validate_path(&allowed_path);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_sandbox_create_temp_file() {
    let sandbox = SandboxEnvironment::new(None).unwrap();
    let content = "test content for temp file";

    let temp_path = sandbox.create_temp_file(content).await.unwrap();

    // Verify file was created
    assert!(temp_path.exists());

    // Verify content
    let read_content = tokio::fs::read_to_string(&temp_path).await.unwrap();
    assert_eq!(read_content, content);

    // Cleanup
    tokio::fs::remove_file(&temp_path).await.unwrap();
}

#[tokio::test]
async fn test_sandbox_cleanup() {
    let sandbox = SandboxEnvironment::new(None).unwrap();

    let result = sandbox.cleanup().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_sandbox_status() {
    let sandbox = SandboxEnvironment::new(Some("/tmp/test".to_string())).unwrap();

    let status = sandbox.status();

    assert_eq!(status.active, true);
    assert_eq!(status.root_path, Some(PathBuf::from("/tmp/test")));
    assert_eq!(status.allowed_operations.len(), 3);
    assert!(status.allowed_operations.contains(&"file_read".to_string()));
    assert!(status.allowed_operations.contains(&"file_write".to_string()));
    assert!(status.allowed_operations.contains(&"command_execute".to_string()));
    assert_eq!(status.security_level, SecurityLevel::Medium);
}

#[tokio::test]
async fn test_sandbox_status_no_root_path() {
    let sandbox = SandboxEnvironment::new(None).unwrap();

    let status = sandbox.status();

    assert_eq!(status.active, true);
    assert_eq!(status.root_path, None);
    assert_eq!(status.security_level, SecurityLevel::Medium);
}

#[test]
fn test_security_level_variants() {
    assert_eq!(SecurityLevel::Low, SecurityLevel::Low);
    assert_eq!(SecurityLevel::Medium, SecurityLevel::Medium);
    assert_eq!(SecurityLevel::High, SecurityLevel::High);
    assert_eq!(SecurityLevel::Maximum, SecurityLevel::Maximum);
    assert_ne!(SecurityLevel::Low, SecurityLevel::High);
}

#[test]
fn test_sandbox_config_default() {
    let config = SandboxConfig::default();

    assert_eq!(config.max_memory_mb, 512);
    assert_eq!(config.max_cpu_percent, 50.0);
    assert_eq!(config.network_access, false);
    assert_eq!(config.file_system_access, true);
    assert_eq!(config.allowed_commands.len(), 3);
    assert!(config.allowed_commands.contains(&"cat".to_string()));
    assert!(config.allowed_commands.contains(&"grep".to_string()));
    assert!(config.allowed_commands.contains(&"ls".to_string()));
}

#[tokio::test]
async fn test_resource_monitor_creation() {
    let config = SandboxConfig::default();
    let monitor = ResourceMonitor::new(config.clone());

    // The config field is private, so we test indirectly through methods
    let usage = monitor.get_usage().await;
    assert_eq!(usage.memory_mb, 100);
    assert_eq!(usage.cpu_percent, 25.0);
    assert_eq!(usage.active_processes, 2);
}

#[tokio::test]
async fn test_resource_monitor_check_limits() {
    let config = SandboxConfig::default();
    let monitor = ResourceMonitor::new(config);

    let result = monitor.check_limits().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_resource_monitor_get_usage() {
    let config = SandboxConfig::default();
    let monitor = ResourceMonitor::new(config);

    let usage = monitor.get_usage().await;

    assert_eq!(usage.memory_mb, 100);
    assert_eq!(usage.cpu_percent, 25.0);
    assert_eq!(usage.active_processes, 2);
}

#[test]
fn test_resource_usage_creation() {
    let usage = ResourceUsage {
        memory_mb: 256,
        cpu_percent: 75.5,
        active_processes: 5,
    };

    assert_eq!(usage.memory_mb, 256);
    assert_eq!(usage.cpu_percent, 75.5);
    assert_eq!(usage.active_processes, 5);
}

#[test]
fn test_debug_formatting() {
    let sandbox = SandboxEnvironment::new(Some("/tmp/test".to_string())).unwrap();
    let status = sandbox.status();

    let debug_str = format!("{:?}", status);
    assert!(debug_str.contains("SandboxStatus"));
    assert!(debug_str.contains("active: true"));

    let level = SecurityLevel::High;
    let debug_str = format!("{:?}", level);
    assert_eq!(debug_str, "High");
}

#[test]
fn test_clone_implementations() {
    let status1 = SandboxStatus {
        active: true,
        root_path: Some(PathBuf::from("/tmp")),
        allowed_operations: vec!["read".to_string()],
        security_level: SecurityLevel::High,
    };

    let status2 = status1.clone();
    assert_eq!(status1.active, status2.active);
    assert_eq!(status1.security_level, status2.security_level);

    let config1 = SandboxConfig::default();
    let config2 = config1.clone();
    assert_eq!(config1.max_memory_mb, config2.max_memory_mb);

    let usage1 = ResourceUsage {
        memory_mb: 100,
        cpu_percent: 50.0,
        active_processes: 3,
    };
    let usage2 = usage1.clone();
    assert_eq!(usage1.memory_mb, usage2.memory_mb);
}
