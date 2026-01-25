//! Sandbox Policy
//!
//! Security policies for sandboxed execution.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Sandbox security level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityLevel {
    /// Minimal restrictions - development only
    Permissive,
    /// Standard restrictions - default for most operations
    Standard,
    /// High security - restricted file and network access
    Restricted,
    /// Maximum security - read-only, no network
    Maximum,
}

impl Default for SecurityLevel {
    fn default() -> Self {
        Self::Standard
    }
}

/// Sandbox policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxPolicy {
    /// Security level
    pub level: SecurityLevel,
    /// Allowed read paths (glob patterns supported)
    pub allowed_read_paths: Vec<PathBuf>,
    /// Allowed write paths (glob patterns supported)
    pub allowed_write_paths: Vec<PathBuf>,
    /// Blocked paths (absolute deny)
    pub blocked_paths: Vec<PathBuf>,
    /// Whether network access is allowed
    pub allow_network: bool,
    /// Allowed network hosts (if network is allowed)
    pub allowed_hosts: Vec<String>,
    /// Whether process spawning is allowed
    pub allow_spawn: bool,
    /// Allowed executables (if spawn is allowed)
    pub allowed_executables: Vec<String>,
    /// Maximum memory in bytes
    pub max_memory_bytes: u64,
    /// Maximum execution time in milliseconds
    pub max_execution_ms: u64,
    /// Maximum output size in bytes
    pub max_output_bytes: u64,
    /// Maximum file size for writes in bytes
    pub max_file_size_bytes: u64,
}

impl Default for SandboxPolicy {
    fn default() -> Self {
        Self::standard()
    }
}

impl SandboxPolicy {
    /// Create a permissive policy (development)
    pub fn permissive() -> Self {
        Self {
            level: SecurityLevel::Permissive,
            allowed_read_paths: vec![PathBuf::from("/")],
            allowed_write_paths: vec![PathBuf::from("/tmp")],
            blocked_paths: vec![
                PathBuf::from("/etc/shadow"),
                PathBuf::from("/etc/passwd"),
            ],
            allow_network: true,
            allowed_hosts: vec!["*".to_string()],
            allow_spawn: true,
            allowed_executables: vec!["*".to_string()],
            max_memory_bytes: 2 * 1024 * 1024 * 1024, // 2GB
            max_execution_ms: 300_000,                 // 5 minutes
            max_output_bytes: 10 * 1024 * 1024,        // 10MB
            max_file_size_bytes: 100 * 1024 * 1024,    // 100MB
        }
    }

    /// Create a standard policy (default)
    pub fn standard() -> Self {
        Self {
            level: SecurityLevel::Standard,
            allowed_read_paths: vec![
                PathBuf::from("/tmp"),
                PathBuf::from("/var/tmp"),
            ],
            allowed_write_paths: vec![PathBuf::from("/tmp")],
            blocked_paths: vec![
                PathBuf::from("/etc"),
                PathBuf::from("/root"),
                PathBuf::from("/home"),
                PathBuf::from(".env"),
                PathBuf::from(".git/config"),
            ],
            allow_network: false,
            allowed_hosts: vec![],
            allow_spawn: false,
            allowed_executables: vec![],
            max_memory_bytes: 512 * 1024 * 1024, // 512MB
            max_execution_ms: 60_000,             // 1 minute
            max_output_bytes: 1024 * 1024,        // 1MB
            max_file_size_bytes: 10 * 1024 * 1024, // 10MB
        }
    }

    /// Create a restricted policy
    pub fn restricted() -> Self {
        Self {
            level: SecurityLevel::Restricted,
            allowed_read_paths: vec![PathBuf::from("/tmp")],
            allowed_write_paths: vec![],
            blocked_paths: vec![
                PathBuf::from("/etc"),
                PathBuf::from("/root"),
                PathBuf::from("/home"),
                PathBuf::from("/var"),
                PathBuf::from("."),
            ],
            allow_network: false,
            allowed_hosts: vec![],
            allow_spawn: false,
            allowed_executables: vec![],
            max_memory_bytes: 256 * 1024 * 1024, // 256MB
            max_execution_ms: 30_000,             // 30 seconds
            max_output_bytes: 256 * 1024,         // 256KB
            max_file_size_bytes: 1024 * 1024,     // 1MB
        }
    }

    /// Create a maximum security policy
    pub fn maximum() -> Self {
        Self {
            level: SecurityLevel::Maximum,
            allowed_read_paths: vec![],
            allowed_write_paths: vec![],
            blocked_paths: vec![PathBuf::from("/")],
            allow_network: false,
            allowed_hosts: vec![],
            allow_spawn: false,
            allowed_executables: vec![],
            max_memory_bytes: 128 * 1024 * 1024, // 128MB
            max_execution_ms: 10_000,             // 10 seconds
            max_output_bytes: 64 * 1024,          // 64KB
            max_file_size_bytes: 0,               // No writes
        }
    }

    /// Add an allowed read path
    pub fn allow_read(mut self, path: impl Into<PathBuf>) -> Self {
        self.allowed_read_paths.push(path.into());
        self
    }

    /// Add an allowed write path
    pub fn allow_write(mut self, path: impl Into<PathBuf>) -> Self {
        self.allowed_write_paths.push(path.into());
        self
    }

    /// Block a path
    pub fn block(mut self, path: impl Into<PathBuf>) -> Self {
        self.blocked_paths.push(path.into());
        self
    }

    /// Allow network access to specific hosts
    pub fn allow_hosts(mut self, hosts: Vec<String>) -> Self {
        self.allow_network = true;
        self.allowed_hosts = hosts;
        self
    }

    /// Allow specific executables
    pub fn allow_executables(mut self, executables: Vec<String>) -> Self {
        self.allow_spawn = true;
        self.allowed_executables = executables;
        self
    }

    /// Set maximum execution time
    pub fn with_max_execution_ms(mut self, ms: u64) -> Self {
        self.max_execution_ms = ms;
        self
    }

    /// Set maximum memory
    pub fn with_max_memory(mut self, bytes: u64) -> Self {
        self.max_memory_bytes = bytes;
        self
    }
}

/// Policy checker for validating operations
#[derive(Debug)]
pub struct PolicyChecker {
    policy: SandboxPolicy,
    canonical_blocked: HashSet<PathBuf>,
}

impl PolicyChecker {
    /// Create a new policy checker
    pub fn new(policy: SandboxPolicy) -> Self {
        let canonical_blocked: HashSet<PathBuf> = policy
            .blocked_paths
            .iter()
            .filter_map(|p| std::fs::canonicalize(p).ok())
            .collect();

        Self {
            policy,
            canonical_blocked,
        }
    }

    /// Check if a path is readable
    pub fn can_read(&self, path: &Path) -> PolicyResult {
        // First check blocked paths
        if self.is_blocked(path) {
            return PolicyResult::Denied(format!(
                "Path is blocked: {}",
                path.display()
            ));
        }

        // Check if any allowed pattern matches
        for allowed in &self.policy.allowed_read_paths {
            if path.starts_with(allowed) || self.matches_pattern(path, allowed) {
                return PolicyResult::Allowed;
            }
        }

        PolicyResult::Denied(format!(
            "Path not in allowed read paths: {}",
            path.display()
        ))
    }

    /// Check if a path is writable
    pub fn can_write(&self, path: &Path) -> PolicyResult {
        // First check blocked paths
        if self.is_blocked(path) {
            return PolicyResult::Denied(format!(
                "Path is blocked: {}",
                path.display()
            ));
        }

        // Check if any allowed pattern matches
        for allowed in &self.policy.allowed_write_paths {
            if path.starts_with(allowed) || self.matches_pattern(path, allowed) {
                return PolicyResult::Allowed;
            }
        }

        PolicyResult::Denied(format!(
            "Path not in allowed write paths: {}",
            path.display()
        ))
    }

    /// Check if network access to a host is allowed
    pub fn can_access_network(&self, host: &str) -> PolicyResult {
        if !self.policy.allow_network {
            return PolicyResult::Denied("Network access is disabled".to_string());
        }

        for allowed in &self.policy.allowed_hosts {
            if allowed == "*" || allowed == host {
                return PolicyResult::Allowed;
            }
            // Simple wildcard matching
            if allowed.starts_with("*.") && host.ends_with(&allowed[1..]) {
                return PolicyResult::Allowed;
            }
        }

        PolicyResult::Denied(format!("Host not allowed: {}", host))
    }

    /// Check if an executable can be spawned
    pub fn can_spawn(&self, executable: &str) -> PolicyResult {
        if !self.policy.allow_spawn {
            return PolicyResult::Denied("Process spawning is disabled".to_string());
        }

        for allowed in &self.policy.allowed_executables {
            if allowed == "*" || allowed == executable {
                return PolicyResult::Allowed;
            }
        }

        PolicyResult::Denied(format!("Executable not allowed: {}", executable))
    }

    /// Check if a file size is within limits
    pub fn check_file_size(&self, size: u64) -> PolicyResult {
        if size > self.policy.max_file_size_bytes {
            return PolicyResult::Denied(format!(
                "File size {} exceeds limit {}",
                size, self.policy.max_file_size_bytes
            ));
        }
        PolicyResult::Allowed
    }

    /// Check if output size is within limits
    pub fn check_output_size(&self, size: u64) -> PolicyResult {
        if size > self.policy.max_output_bytes {
            return PolicyResult::Denied(format!(
                "Output size {} exceeds limit {}",
                size, self.policy.max_output_bytes
            ));
        }
        PolicyResult::Allowed
    }

    /// Get the policy
    pub fn policy(&self) -> &SandboxPolicy {
        &self.policy
    }

    /// Check if path is blocked
    fn is_blocked(&self, path: &Path) -> bool {
        // Check against blocked paths
        for blocked in &self.policy.blocked_paths {
            if path.starts_with(blocked) || path.ends_with(blocked) {
                return true;
            }
            // Check path components
            if let Some(name) = path.file_name() {
                if let Some(blocked_name) = blocked.file_name() {
                    if name == blocked_name {
                        return true;
                    }
                }
            }
        }

        // Check canonical paths
        if let Ok(canonical) = std::fs::canonicalize(path) {
            for blocked in &self.canonical_blocked {
                if canonical.starts_with(blocked) {
                    return true;
                }
            }
        }

        false
    }

    /// Simple pattern matching for paths
    fn matches_pattern(&self, path: &Path, pattern: &Path) -> bool {
        let pattern_str = pattern.to_string_lossy();
        let path_str = path.to_string_lossy();

        // Handle glob-like patterns
        if pattern_str.contains('*') {
            // Simple glob: /tmp/* matches /tmp/anything
            let prefix = pattern_str.trim_end_matches("/*").trim_end_matches('*');
            return path_str.starts_with(prefix);
        }

        path.starts_with(pattern)
    }
}

/// Result of policy check
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyResult {
    /// Operation is allowed
    Allowed,
    /// Operation is denied with reason
    Denied(String),
}

impl PolicyResult {
    /// Check if allowed
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allowed)
    }

    /// Check if denied
    pub fn is_denied(&self) -> bool {
        matches!(self, Self::Denied(_))
    }

    /// Get denial reason
    pub fn reason(&self) -> Option<&str> {
        match self {
            Self::Denied(reason) => Some(reason),
            Self::Allowed => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_levels() {
        assert_eq!(SandboxPolicy::default().level, SecurityLevel::Standard);
        assert_eq!(SandboxPolicy::permissive().level, SecurityLevel::Permissive);
        assert_eq!(SandboxPolicy::restricted().level, SecurityLevel::Restricted);
        assert_eq!(SandboxPolicy::maximum().level, SecurityLevel::Maximum);
    }

    #[test]
    fn test_policy_can_read() {
        let policy = SandboxPolicy::standard();
        let checker = PolicyChecker::new(policy);

        // /tmp should be readable
        assert!(checker.can_read(Path::new("/tmp/test.txt")).is_allowed());

        // /etc should be blocked
        assert!(checker.can_read(Path::new("/etc/passwd")).is_denied());
    }

    #[test]
    fn test_policy_can_write() {
        let policy = SandboxPolicy::standard();
        let checker = PolicyChecker::new(policy);

        // /tmp should be writable
        assert!(checker.can_write(Path::new("/tmp/output.txt")).is_allowed());

        // /etc should be blocked
        assert!(checker.can_write(Path::new("/etc/anything")).is_denied());
    }

    #[test]
    fn test_policy_blocked_files() {
        let policy = SandboxPolicy::standard();
        let checker = PolicyChecker::new(policy);

        // .env files should be blocked
        assert!(checker.can_read(Path::new("/project/.env")).is_denied());
    }

    #[test]
    fn test_policy_network() {
        let standard = PolicyChecker::new(SandboxPolicy::standard());
        assert!(standard.can_access_network("example.com").is_denied());

        let with_hosts = PolicyChecker::new(
            SandboxPolicy::standard().allow_hosts(vec!["example.com".to_string()]),
        );
        assert!(with_hosts.can_access_network("example.com").is_allowed());
        assert!(with_hosts.can_access_network("other.com").is_denied());
    }

    #[test]
    fn test_policy_spawn() {
        let standard = PolicyChecker::new(SandboxPolicy::standard());
        assert!(standard.can_spawn("ls").is_denied());

        let with_spawn = PolicyChecker::new(
            SandboxPolicy::standard().allow_executables(vec!["ls".to_string()]),
        );
        assert!(with_spawn.can_spawn("ls").is_allowed());
        assert!(with_spawn.can_spawn("rm").is_denied());
    }

    #[test]
    fn test_policy_builder() {
        let policy = SandboxPolicy::restricted()
            .allow_read("/custom/read")
            .allow_write("/custom/write")
            .block("/sensitive")
            .with_max_execution_ms(5000);

        assert!(policy.allowed_read_paths.contains(&PathBuf::from("/custom/read")));
        assert!(policy.allowed_write_paths.contains(&PathBuf::from("/custom/write")));
        assert!(policy.blocked_paths.contains(&PathBuf::from("/sensitive")));
        assert_eq!(policy.max_execution_ms, 5000);
    }

    #[test]
    fn test_policy_file_size() {
        let policy = SandboxPolicy::standard();
        let checker = PolicyChecker::new(policy);

        // Within limit
        assert!(checker.check_file_size(1024).is_allowed());

        // Exceeds limit
        assert!(checker.check_file_size(100 * 1024 * 1024).is_denied());
    }
}
