//! Governance Modes
//!
//! Defines the governance modes that control agent autonomy levels.
//! Based on Sterling's graduated autonomy approach.

use serde::{Deserialize, Serialize};

/// Governance mode controlling agent autonomy level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GovernanceMode {
    /// All actions require explicit human approval
    /// Used during initial deployment or after incidents
    Strict,

    /// Dangerous actions require approval, safe actions proceed
    /// Normal operating mode for most deployments
    Supervised,

    /// Agent operates within policy bounds without approval
    /// Only for well-tested, low-risk task types
    Autonomous,

    /// Emergency shutdown - no actions allowed
    /// Triggered by critical invariant violations
    Lockdown,
}

impl GovernanceMode {
    /// Check if this mode allows autonomous execution of safe actions
    pub fn allows_safe_actions(&self) -> bool {
        matches!(self, Self::Supervised | Self::Autonomous)
    }

    /// Check if this mode allows autonomous execution of dangerous actions
    pub fn allows_dangerous_actions(&self) -> bool {
        matches!(self, Self::Autonomous)
    }

    /// Check if this mode requires approval for all actions
    pub fn requires_all_approval(&self) -> bool {
        matches!(self, Self::Strict | Self::Lockdown)
    }

    /// Check if this mode blocks all execution
    pub fn is_locked(&self) -> bool {
        matches!(self, Self::Lockdown)
    }

    /// Get the escalation mode (one level more restrictive)
    pub fn escalate(&self) -> Self {
        match self {
            Self::Autonomous => Self::Supervised,
            Self::Supervised => Self::Strict,
            Self::Strict => Self::Lockdown,
            Self::Lockdown => Self::Lockdown,
        }
    }

    /// Get the de-escalation mode (one level less restrictive)
    pub fn de_escalate(&self) -> Self {
        match self {
            Self::Lockdown => Self::Strict,
            Self::Strict => Self::Supervised,
            Self::Supervised => Self::Autonomous,
            Self::Autonomous => Self::Autonomous,
        }
    }
}

impl Default for GovernanceMode {
    fn default() -> Self {
        // Default to Supervised - safe default for most deployments
        Self::Supervised
    }
}

/// A mode transition request with justification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeTransition {
    /// Current mode
    pub from: GovernanceMode,
    /// Requested mode
    pub to: GovernanceMode,
    /// Reason for the transition
    pub reason: String,
    /// Who/what requested the transition
    pub requested_by: String,
    /// Timestamp of request
    #[serde(with = "chrono::serde::ts_seconds")]
    pub requested_at: chrono::DateTime<chrono::Utc>,
    /// Whether this transition was approved
    pub approved: Option<bool>,
    /// Who approved (if approved)
    pub approved_by: Option<String>,
}

impl ModeTransition {
    /// Create a new transition request
    pub fn request(from: GovernanceMode, to: GovernanceMode, reason: String, requested_by: String) -> Self {
        Self {
            from,
            to,
            reason,
            requested_by,
            requested_at: chrono::Utc::now(),
            approved: None,
            approved_by: None,
        }
    }

    /// Check if this is an escalation (more restrictive)
    pub fn is_escalation(&self) -> bool {
        matches!(
            (self.from, self.to),
            (GovernanceMode::Autonomous, GovernanceMode::Supervised)
                | (GovernanceMode::Autonomous, GovernanceMode::Strict)
                | (GovernanceMode::Autonomous, GovernanceMode::Lockdown)
                | (GovernanceMode::Supervised, GovernanceMode::Strict)
                | (GovernanceMode::Supervised, GovernanceMode::Lockdown)
                | (GovernanceMode::Strict, GovernanceMode::Lockdown)
        )
    }

    /// Check if this is a de-escalation (less restrictive)
    pub fn is_de_escalation(&self) -> bool {
        matches!(
            (self.from, self.to),
            (GovernanceMode::Lockdown, GovernanceMode::Strict)
                | (GovernanceMode::Lockdown, GovernanceMode::Supervised)
                | (GovernanceMode::Lockdown, GovernanceMode::Autonomous)
                | (GovernanceMode::Strict, GovernanceMode::Supervised)
                | (GovernanceMode::Strict, GovernanceMode::Autonomous)
                | (GovernanceMode::Supervised, GovernanceMode::Autonomous)
        )
    }

    /// Approve this transition
    pub fn approve(mut self, approved_by: String) -> Self {
        self.approved = Some(true);
        self.approved_by = Some(approved_by);
        self
    }

    /// Deny this transition
    pub fn deny(mut self) -> Self {
        self.approved = Some(false);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_permissions() {
        assert!(!GovernanceMode::Strict.allows_safe_actions());
        assert!(GovernanceMode::Supervised.allows_safe_actions());
        assert!(GovernanceMode::Autonomous.allows_safe_actions());
        assert!(!GovernanceMode::Lockdown.allows_safe_actions());

        assert!(!GovernanceMode::Strict.allows_dangerous_actions());
        assert!(!GovernanceMode::Supervised.allows_dangerous_actions());
        assert!(GovernanceMode::Autonomous.allows_dangerous_actions());
        assert!(!GovernanceMode::Lockdown.allows_dangerous_actions());
    }

    #[test]
    fn test_mode_escalation() {
        assert_eq!(GovernanceMode::Autonomous.escalate(), GovernanceMode::Supervised);
        assert_eq!(GovernanceMode::Supervised.escalate(), GovernanceMode::Strict);
        assert_eq!(GovernanceMode::Strict.escalate(), GovernanceMode::Lockdown);
        assert_eq!(GovernanceMode::Lockdown.escalate(), GovernanceMode::Lockdown);
    }

    #[test]
    fn test_mode_de_escalation() {
        assert_eq!(GovernanceMode::Lockdown.de_escalate(), GovernanceMode::Strict);
        assert_eq!(GovernanceMode::Strict.de_escalate(), GovernanceMode::Supervised);
        assert_eq!(GovernanceMode::Supervised.de_escalate(), GovernanceMode::Autonomous);
        assert_eq!(GovernanceMode::Autonomous.de_escalate(), GovernanceMode::Autonomous);
    }

    #[test]
    fn test_transition_classification() {
        let escalation = ModeTransition::request(
            GovernanceMode::Autonomous,
            GovernanceMode::Supervised,
            "Test".to_string(),
            "system".to_string(),
        );
        assert!(escalation.is_escalation());
        assert!(!escalation.is_de_escalation());

        let de_escalation = ModeTransition::request(
            GovernanceMode::Strict,
            GovernanceMode::Supervised,
            "Test".to_string(),
            "admin".to_string(),
        );
        assert!(!de_escalation.is_escalation());
        assert!(de_escalation.is_de_escalation());
    }

    #[test]
    fn test_default_mode() {
        assert_eq!(GovernanceMode::default(), GovernanceMode::Supervised);
    }
}
