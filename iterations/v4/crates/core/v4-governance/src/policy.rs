//! Policy Enforcement
//!
//! Defines policies that control what actions agents can take.
//! Implements fail-closed approach: unknown = denied.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use v4_types::operators::OperatorType;

use crate::modes::GovernanceMode;

/// Error types for policy enforcement
#[derive(Debug, Error)]
pub enum PolicyError {
    #[error("Action denied by policy: {0}")]
    Denied(String),

    #[error("Unknown action type - fail closed")]
    UnknownAction,

    #[error("Governance mode {0:?} does not allow this action")]
    ModeRestriction(GovernanceMode),

    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),
}

/// A request to perform an action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRequest {
    /// Unique request ID
    pub request_id: String,
    /// The operator being requested
    pub operator: OperatorType,
    /// Who is requesting (agent ID)
    pub requester: String,
    /// Task context (if any)
    pub task_id: Option<String>,
    /// Timestamp
    #[serde(with = "chrono::serde::ts_seconds")]
    pub requested_at: chrono::DateTime<chrono::Utc>,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl ActionRequest {
    /// Create a new action request
    pub fn new(operator: OperatorType, requester: String) -> Self {
        Self {
            request_id: uuid::Uuid::new_v4().to_string(),
            operator,
            requester,
            task_id: None,
            requested_at: chrono::Utc::now(),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Add task context
    pub fn with_task(mut self, task_id: String) -> Self {
        self.task_id = Some(task_id);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Check if this action has side effects
    pub fn has_side_effects(&self) -> bool {
        self.operator.has_side_effects()
    }

    /// Check if this is a dangerous action
    pub fn is_dangerous(&self) -> bool {
        match &self.operator {
            OperatorType::Control(ctrl) => {
                use v4_types::operators::ControlOp;
                matches!(ctrl, ControlOp::Terminate { .. } | ControlOp::Delegate { .. })
            }
            OperatorType::Memorize(_) => true, // All writes need review
            _ => false,
        }
    }
}

/// Result of a policy decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDecision {
    /// The request that was evaluated
    pub request_id: String,
    /// Whether the action is allowed
    pub allowed: bool,
    /// If not allowed, why
    pub denial_reason: Option<String>,
    /// If allowed with conditions, what are they
    pub conditions: Vec<String>,
    /// Whether human approval is required
    pub requires_approval: bool,
    /// Who needs to approve (if requires_approval)
    pub approver: Option<String>,
    /// Timestamp of decision
    #[serde(with = "chrono::serde::ts_seconds")]
    pub decided_at: chrono::DateTime<chrono::Utc>,
}

impl PolicyDecision {
    /// Create an allow decision
    pub fn allow(request_id: String) -> Self {
        Self {
            request_id,
            allowed: true,
            denial_reason: None,
            conditions: Vec::new(),
            requires_approval: false,
            approver: None,
            decided_at: chrono::Utc::now(),
        }
    }

    /// Create a deny decision
    pub fn deny(request_id: String, reason: String) -> Self {
        Self {
            request_id,
            allowed: false,
            denial_reason: Some(reason),
            conditions: Vec::new(),
            requires_approval: false,
            approver: None,
            decided_at: chrono::Utc::now(),
        }
    }

    /// Create a requires-approval decision
    pub fn require_approval(request_id: String, approver: String) -> Self {
        Self {
            request_id,
            allowed: false, // Not allowed until approved
            denial_reason: None,
            conditions: Vec::new(),
            requires_approval: true,
            approver: Some(approver),
            decided_at: chrono::Utc::now(),
        }
    }

    /// Add a condition to the decision
    pub fn with_condition(mut self, condition: String) -> Self {
        self.conditions.push(condition);
        self
    }
}

/// A policy violation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyViolation {
    /// Violation ID
    pub id: String,
    /// The request that violated policy
    pub request_id: String,
    /// What policy was violated
    pub policy_id: String,
    /// Description of the violation
    pub description: String,
    /// Severity (1-10, 10 being most severe)
    pub severity: u8,
    /// Timestamp
    #[serde(with = "chrono::serde::ts_seconds")]
    pub occurred_at: chrono::DateTime<chrono::Utc>,
}

/// Policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConfig {
    /// Blocked file patterns (glob)
    pub blocked_paths: Vec<String>,
    /// Allowed file patterns (glob)
    pub allowed_paths: Vec<String>,
    /// Maximum memory operations per minute
    pub max_memory_ops_per_minute: u32,
    /// Maximum file reads per minute
    pub max_file_reads_per_minute: u32,
    /// Maximum API calls per minute
    pub max_api_calls_per_minute: u32,
    /// Whether to allow web access
    pub allow_web_access: bool,
    /// Whether to allow code execution
    pub allow_code_execution: bool,
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            blocked_paths: vec![
                "**/node_modules/**".to_string(),
                "**/.git/**".to_string(),
                "**/target/**".to_string(),
                "**/.env".to_string(),
                "**/secrets/**".to_string(),
            ],
            allowed_paths: vec!["**/*.rs".to_string(), "**/*.ts".to_string(), "**/*.py".to_string()],
            max_memory_ops_per_minute: 100,
            max_file_reads_per_minute: 50,
            max_api_calls_per_minute: 30,
            allow_web_access: true,
            allow_code_execution: false,
        }
    }
}

/// Policy enforcer
pub struct Policy {
    /// Current governance mode
    mode: GovernanceMode,
    /// Policy configuration
    config: PolicyConfig,
    /// Violation log
    violations: Vec<PolicyViolation>,
}

impl Policy {
    /// Create a new policy with default configuration
    pub fn new(mode: GovernanceMode) -> Self {
        Self {
            mode,
            config: PolicyConfig::default(),
            violations: Vec::new(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(mode: GovernanceMode, config: PolicyConfig) -> Self {
        Self {
            mode,
            config,
            violations: Vec::new(),
        }
    }

    /// Get current governance mode
    pub fn mode(&self) -> GovernanceMode {
        self.mode
    }

    /// Update governance mode
    pub fn set_mode(&mut self, mode: GovernanceMode) {
        self.mode = mode;
    }

    /// Evaluate an action request
    pub fn evaluate(&self, request: &ActionRequest) -> PolicyDecision {
        // Fail-closed: lockdown mode denies everything
        if self.mode.is_locked() {
            return PolicyDecision::deny(
                request.request_id.clone(),
                "System is in lockdown mode".to_string(),
            );
        }

        // Check if action is known
        // (All OperatorType variants are known, so this always passes)

        // Check if action has side effects
        if request.has_side_effects() {
            // In strict mode, all side effects need approval
            if self.mode.requires_all_approval() {
                return PolicyDecision::require_approval(request.request_id.clone(), "admin".to_string());
            }

            // In supervised mode, dangerous actions need approval
            if !self.mode.allows_dangerous_actions() && request.is_dangerous() {
                return PolicyDecision::require_approval(request.request_id.clone(), "supervisor".to_string());
            }
        }

        // Check operator-specific policies
        match &request.operator {
            OperatorType::Seek(seek) => self.check_seek_policy(request, seek),
            OperatorType::Memorize(_) => {
                // Memory operations are allowed but rate-limited
                PolicyDecision::allow(request.request_id.clone())
                    .with_condition(format!(
                        "Rate limit: {} ops/min",
                        self.config.max_memory_ops_per_minute
                    ))
            }
            OperatorType::Control(ctrl) => self.check_control_policy(request, ctrl),
            _ => PolicyDecision::allow(request.request_id.clone()),
        }
    }

    /// Check seek operation policies
    fn check_seek_policy(
        &self,
        request: &ActionRequest,
        seek: &v4_types::operators::SeekOp,
    ) -> PolicyDecision {
        use v4_types::operators::SeekOp;

        match seek {
            SeekOp::ReadFile { path } | SeekOp::ListDirectory { path } => {
                // Check blocked paths
                for blocked in &self.config.blocked_paths {
                    let pattern = blocked.trim_matches('*');
                    if path.contains(pattern) {
                        return PolicyDecision::deny(
                            request.request_id.clone(),
                            format!("Path {} is blocked by policy", path),
                        );
                    }
                }
                PolicyDecision::allow(request.request_id.clone())
            }
            SeekOp::WebSearch { .. } | SeekOp::WebFetch { .. } => {
                if !self.config.allow_web_access {
                    return PolicyDecision::deny(
                        request.request_id.clone(),
                        "Web access is disabled by policy".to_string(),
                    );
                }
                PolicyDecision::allow(request.request_id.clone())
            }
            _ => PolicyDecision::allow(request.request_id.clone()),
        }
    }

    /// Check control operation policies
    fn check_control_policy(
        &self,
        request: &ActionRequest,
        ctrl: &v4_types::operators::ControlOp,
    ) -> PolicyDecision {
        use v4_types::operators::ControlOp;

        match ctrl {
            ControlOp::Terminate { reason } => {
                // Always log termination decisions
                PolicyDecision::allow(request.request_id.clone())
                    .with_condition(format!("Termination logged: {}", reason))
            }
            ControlOp::Delegate { agent_id, .. } => {
                // Delegation needs approval in supervised mode
                if !self.mode.allows_dangerous_actions() {
                    return PolicyDecision::require_approval(
                        request.request_id.clone(),
                        format!("supervisor:delegation:{}", agent_id),
                    );
                }
                PolicyDecision::allow(request.request_id.clone())
            }
            ControlOp::Loop { max_iterations, .. } => {
                // Check iteration bounds (INV-CORE-07)
                if *max_iterations > 1000 {
                    return PolicyDecision::deny(
                        request.request_id.clone(),
                        format!(
                            "Loop iteration limit {} exceeds maximum 1000",
                            max_iterations
                        ),
                    );
                }
                PolicyDecision::allow(request.request_id.clone())
            }
            _ => PolicyDecision::allow(request.request_id.clone()),
        }
    }

    /// Record a policy violation
    pub fn record_violation(&mut self, request_id: String, policy_id: String, description: String, severity: u8) {
        let violation = PolicyViolation {
            id: uuid::Uuid::new_v4().to_string(),
            request_id,
            policy_id,
            description,
            severity: severity.min(10),
            occurred_at: chrono::Utc::now(),
        };
        self.violations.push(violation);

        // Auto-escalate on severe violations
        if severity >= 8 {
            self.mode = self.mode.escalate();
        }
    }

    /// Get all violations
    pub fn violations(&self) -> &[PolicyViolation] {
        &self.violations
    }

    /// Get severe violations (severity >= 7)
    pub fn severe_violations(&self) -> Vec<&PolicyViolation> {
        self.violations.iter().filter(|v| v.severity >= 7).collect()
    }
}

impl Default for Policy {
    fn default() -> Self {
        Self::new(GovernanceMode::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use v4_types::operators::{ControlOp, SeekOp};

    #[test]
    fn test_safe_action_allowed() {
        let policy = Policy::new(GovernanceMode::Supervised);
        let request = ActionRequest::new(
            OperatorType::Seek(SeekOp::ReadFile {
                path: "src/main.rs".to_string(),
            }),
            "agent-1".to_string(),
        );

        let decision = policy.evaluate(&request);
        assert!(decision.allowed);
    }

    #[test]
    fn test_blocked_path_denied() {
        let policy = Policy::new(GovernanceMode::Supervised);
        let request = ActionRequest::new(
            OperatorType::Seek(SeekOp::ReadFile {
                path: "project/node_modules/package/index.js".to_string(),
            }),
            "agent-1".to_string(),
        );

        let decision = policy.evaluate(&request);
        assert!(!decision.allowed);
        assert!(decision.denial_reason.unwrap().contains("blocked"));
    }

    #[test]
    fn test_lockdown_denies_all() {
        let policy = Policy::new(GovernanceMode::Lockdown);
        let request = ActionRequest::new(
            OperatorType::Seek(SeekOp::ReadFile {
                path: "src/main.rs".to_string(),
            }),
            "agent-1".to_string(),
        );

        let decision = policy.evaluate(&request);
        assert!(!decision.allowed);
        assert!(decision.denial_reason.unwrap().contains("lockdown"));
    }

    #[test]
    fn test_dangerous_action_needs_approval_in_supervised() {
        let policy = Policy::new(GovernanceMode::Supervised);
        let request = ActionRequest::new(
            OperatorType::Control(ControlOp::Delegate {
                agent_id: "agent-2".to_string(),
                task: "subtask".to_string(),
            }),
            "agent-1".to_string(),
        );

        let decision = policy.evaluate(&request);
        assert!(!decision.allowed);
        assert!(decision.requires_approval);
    }

    #[test]
    fn test_dangerous_action_allowed_in_autonomous() {
        let policy = Policy::new(GovernanceMode::Autonomous);
        let request = ActionRequest::new(
            OperatorType::Control(ControlOp::Delegate {
                agent_id: "agent-2".to_string(),
                task: "subtask".to_string(),
            }),
            "agent-1".to_string(),
        );

        let decision = policy.evaluate(&request);
        assert!(decision.allowed);
    }

    #[test]
    fn test_loop_iteration_limit() {
        let policy = Policy::new(GovernanceMode::Autonomous);

        // Valid loop
        let request = ActionRequest::new(
            OperatorType::Control(ControlOp::Loop {
                condition: "true".to_string(),
                max_iterations: 100,
            }),
            "agent-1".to_string(),
        );
        let decision = policy.evaluate(&request);
        assert!(decision.allowed);

        // Excessive loop
        let request = ActionRequest::new(
            OperatorType::Control(ControlOp::Loop {
                condition: "true".to_string(),
                max_iterations: 5000,
            }),
            "agent-1".to_string(),
        );
        let decision = policy.evaluate(&request);
        assert!(!decision.allowed);
    }

    #[test]
    fn test_violation_auto_escalates() {
        let mut policy = Policy::new(GovernanceMode::Autonomous);

        // Record severe violation
        policy.record_violation(
            "req-1".to_string(),
            "POL-001".to_string(),
            "Critical security violation".to_string(),
            9,
        );

        // Mode should have escalated
        assert_eq!(policy.mode(), GovernanceMode::Supervised);
    }

    #[test]
    fn test_web_access_policy() {
        // Default allows web access
        let policy = Policy::new(GovernanceMode::Supervised);
        let request = ActionRequest::new(
            OperatorType::Seek(SeekOp::WebSearch {
                query: "rust async".to_string(),
            }),
            "agent-1".to_string(),
        );
        let decision = policy.evaluate(&request);
        assert!(decision.allowed);

        // Disable web access
        let mut config = PolicyConfig::default();
        config.allow_web_access = false;
        let policy = Policy::with_config(GovernanceMode::Supervised, config);
        let decision = policy.evaluate(&request);
        assert!(!decision.allowed);
    }
}
