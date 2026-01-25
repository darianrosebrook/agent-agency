//! V4 Sandbox
//!
//! Sandboxed execution environment for the Agent Agency V4 system.
//!
//! This crate provides:
//! - **Policy System**: Configurable security policies with multiple levels
//! - **Sandbox Environment**: Isolated execution environment with resource tracking
//! - **Policy Enforcement**: Automatic checking of file, network, and process operations
//! - **Audit Logging**: Complete audit trail of all operations
//!
//! # Security Levels
//!
//! - **Permissive**: Development only, minimal restrictions
//! - **Standard**: Default for most operations
//! - **Restricted**: High security with limited file/network access
//! - **Maximum**: Read-only, no network, maximum isolation
//!
//! # Example
//!
//! ```rust,ignore
//! use v4_sandbox::{SandboxBuilder, SandboxPolicy, SandboxedExecutor};
//! use v4_tools::ToolRegistry;
//! use std::sync::Arc;
//!
//! // Create a sandbox with standard policy
//! let sandbox = Arc::new(
//!     SandboxBuilder::new("my-sandbox")
//!         .policy(SandboxPolicy::standard())
//!         .working_dir("/tmp/sandbox")
//!         .env("MY_VAR", "value")
//!         .build()
//! );
//!
//! // Initialize the sandbox
//! sandbox.initialize().await?;
//!
//! // Create executor
//! let registry = Arc::new(ToolRegistry::new());
//! let executor = SandboxedExecutor::new(sandbox.clone(), registry);
//!
//! // Execute operators within the sandbox
//! let record = executor.execute(&operator).await?;
//!
//! // Cleanup
//! sandbox.teardown().await?;
//! ```

mod environment;
mod executor;
mod policy;

pub use environment::{
    AuditAction, AuditEntry, AuditResult, ResourceUsage, SandboxEnvironment, SandboxError,
    SandboxId, SandboxState,
};
pub use executor::{
    SandboxBuilder, SandboxedExecutionRecord, SandboxedExecutor, SandboxedExecutorConfig,
    SandboxedExecutorError,
};
pub use policy::{PolicyChecker, PolicyResult, SandboxPolicy, SecurityLevel};

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::sync::Arc;
    use v4_tools::ToolRegistry;
    use v4_types::operators::SeekOp;
    use v4_types::OperatorType;

    #[tokio::test]
    async fn test_sandbox_lifecycle() {
        // Create sandbox
        let sandbox = SandboxBuilder::new("lifecycle-test")
            .policy(SandboxPolicy::standard())
            .working_dir("/tmp/sandbox-lifecycle-test")
            .build();

        assert_eq!(sandbox.state().await, SandboxState::Initializing);

        // Initialize
        sandbox.initialize().await.unwrap();
        assert_eq!(sandbox.state().await, SandboxState::Ready);

        // Activate
        sandbox.activate().await.unwrap();
        assert_eq!(sandbox.state().await, SandboxState::Active);

        // Teardown
        sandbox.teardown().await.unwrap();
        assert_eq!(sandbox.state().await, SandboxState::Destroyed);
    }

    #[tokio::test]
    async fn test_policy_levels() {
        // Permissive policy
        let permissive = PolicyChecker::new(SandboxPolicy::permissive());
        assert!(permissive.can_read(Path::new("/tmp/test.txt")).is_allowed());
        assert!(permissive.can_spawn("ls").is_allowed());

        // Standard policy
        let standard = PolicyChecker::new(SandboxPolicy::standard());
        assert!(standard.can_read(Path::new("/tmp/test.txt")).is_allowed());
        assert!(standard.can_spawn("ls").is_denied());
        assert!(standard.can_access_network("example.com").is_denied());

        // Restricted policy
        let restricted = PolicyChecker::new(SandboxPolicy::restricted());
        assert!(restricted.can_write(Path::new("/tmp/output.txt")).is_denied());

        // Maximum policy
        let maximum = PolicyChecker::new(SandboxPolicy::maximum());
        assert!(maximum.can_read(Path::new("/tmp/test.txt")).is_denied());
    }

    #[tokio::test]
    async fn test_sandbox_resource_tracking() {
        let sandbox = SandboxBuilder::new("resource-test")
            .policy(SandboxPolicy::standard())
            .build();

        // Record various operations
        sandbox.record_read(1000).await;
        sandbox.record_write(500).await;
        sandbox.record_file_created().await;
        sandbox.record_file_modified().await;

        let usage = sandbox.resource_usage().await;
        assert_eq!(usage.bytes_read, 1000);
        assert_eq!(usage.bytes_written, 500);
        assert_eq!(usage.files_created, 1);
        assert_eq!(usage.files_modified, 1);
    }

    #[tokio::test]
    async fn test_sandbox_audit_logging() {
        let sandbox = SandboxBuilder::new("audit-test")
            .policy(SandboxPolicy::standard())
            .build();

        // Perform operations that get audited
        let _ = sandbox.can_read(Path::new("/tmp/file.txt")).await;
        let _ = sandbox.can_write(Path::new("/etc/passwd")).await;
        let _ = sandbox.can_access_network("example.com").await;

        let log = sandbox.audit_log().await;
        assert_eq!(log.len(), 3);

        // Verify audit entry types
        assert!(matches!(log[0].action, AuditAction::FileRead));
        assert!(matches!(log[1].action, AuditAction::FileWrite));
        assert!(matches!(log[2].action, AuditAction::NetworkAccess));
    }

    #[tokio::test]
    async fn test_sandboxed_executor_policy_enforcement() {
        let sandbox = Arc::new(
            SandboxBuilder::new("executor-test")
                .policy(SandboxPolicy::restricted())
                .build(),
        );

        sandbox.initialize().await.unwrap();

        let registry = Arc::new(ToolRegistry::new());
        let executor = SandboxedExecutor::new(sandbox.clone(), registry);

        // Try to read from blocked path
        let operator = OperatorType::Seek(SeekOp::ReadFile {
            path: "/etc/passwd".to_string(),
        });

        let result = executor.execute(&operator).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_policy_builder() {
        let policy = SandboxPolicy::restricted()
            .allow_read("/custom/read")
            .allow_write("/custom/write")
            .block("/sensitive")
            .allow_hosts(vec!["api.example.com".to_string()])
            .allow_executables(vec!["cat".to_string(), "ls".to_string()])
            .with_max_execution_ms(5000)
            .with_max_memory(1024 * 1024);

        let checker = PolicyChecker::new(policy);

        assert!(checker.can_access_network("api.example.com").is_allowed());
        assert!(checker.can_access_network("other.com").is_denied());
        assert!(checker.can_spawn("cat").is_allowed());
        assert!(checker.can_spawn("rm").is_denied());
    }

    #[test]
    fn test_security_level_defaults() {
        assert_eq!(SecurityLevel::default(), SecurityLevel::Standard);
        assert_eq!(SandboxPolicy::default().level, SecurityLevel::Standard);
    }

    #[tokio::test]
    async fn test_sandbox_environment_variables() {
        let sandbox = SandboxBuilder::new("env-test")
            .env("PATH", "/usr/bin")
            .env("HOME", "/tmp")
            .env("USER", "sandbox")
            .build();

        assert_eq!(sandbox.env_vars.get("PATH"), Some(&"/usr/bin".to_string()));
        assert_eq!(sandbox.env_vars.get("HOME"), Some(&"/tmp".to_string()));
        assert_eq!(sandbox.env_vars.get("USER"), Some(&"sandbox".to_string()));
    }

    #[tokio::test]
    async fn test_path_resolution() {
        let sandbox = SandboxBuilder::new("path-test")
            .working_dir("/tmp/sandbox-path")
            .build();

        // Relative path should be resolved
        let relative = sandbox.resolve_path(Path::new("file.txt"));
        assert_eq!(relative, std::path::PathBuf::from("/tmp/sandbox-path/file.txt"));

        // Absolute path should remain unchanged
        let absolute = sandbox.resolve_path(Path::new("/etc/passwd"));
        assert_eq!(absolute, std::path::PathBuf::from("/etc/passwd"));
    }
}
