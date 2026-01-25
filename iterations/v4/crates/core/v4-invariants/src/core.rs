//! Core Invariants (Sterling-style INV-CORE-*)
//!
//! These are the 11 core invariants from Sterling that ensure
//! safe and auditable agent behavior.

use serde::{Deserialize, Serialize};

/// Invariant identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InvariantId(&'static str);

impl InvariantId {
    /// Get the string representation
    pub fn as_str(&self) -> &'static str {
        self.0
    }
}

impl std::fmt::Display for InvariantId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Core invariants from Sterling architecture
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoreInvariant {
    /// INV-CORE-01: No free-form chain-of-thought in decision loops
    /// LLM outputs in decision loops must be structured (JSON, enum), not free-form text.
    NoFreeFormCoT,

    /// INV-CORE-02: All task state in explicit store, not LLM context
    /// State must be in database/event log, not transformer KV cache.
    ExplicitStateOnly,

    /// INV-CORE-03: Bounded working memory with decay
    /// Memory must have size limits and decay mechanism.
    BoundedMemory,

    /// INV-CORE-04: Deterministic operator selection
    /// Given same state, same operator must be selected.
    DeterministicSelection,

    /// INV-CORE-05: Provenance for all decisions
    /// Every decision must have auditable provenance chain.
    ProvenanceRequired,

    /// INV-CORE-06: No nested LLM calls in single decision
    /// One LLM call per decision step maximum.
    NoNestedLLMCalls,

    /// INV-CORE-07: Termination guarantee
    /// All loops must have bounded iteration limits.
    TerminationGuarantee,

    /// INV-CORE-08: No hidden routers
    /// All task routing decisions must be logged with reasoning.
    NoHiddenRouters,

    /// INV-CORE-09: Fail-closed on uncertainty
    /// If uncertain, deny/stop rather than proceed.
    FailClosed,

    /// INV-CORE-10: Cryptographic audit trail
    /// All events must be content-hashed and append-only.
    CryptoAuditTrail,

    /// INV-CORE-11: Sealed external interface
    /// Tools cannot mutate agent internal state except via governed operators.
    SealedExternalInterface,
}

impl CoreInvariant {
    /// Get the invariant ID
    pub fn id(&self) -> InvariantId {
        match self {
            Self::NoFreeFormCoT => InvariantId("INV-CORE-01"),
            Self::ExplicitStateOnly => InvariantId("INV-CORE-02"),
            Self::BoundedMemory => InvariantId("INV-CORE-03"),
            Self::DeterministicSelection => InvariantId("INV-CORE-04"),
            Self::ProvenanceRequired => InvariantId("INV-CORE-05"),
            Self::NoNestedLLMCalls => InvariantId("INV-CORE-06"),
            Self::TerminationGuarantee => InvariantId("INV-CORE-07"),
            Self::NoHiddenRouters => InvariantId("INV-CORE-08"),
            Self::FailClosed => InvariantId("INV-CORE-09"),
            Self::CryptoAuditTrail => InvariantId("INV-CORE-10"),
            Self::SealedExternalInterface => InvariantId("INV-CORE-11"),
        }
    }

    /// Get a human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Self::NoFreeFormCoT => {
                "LLM outputs in decision loops must be structured, not free-form text"
            }
            Self::ExplicitStateOnly => {
                "All task state must be in explicit stores, not LLM context"
            }
            Self::BoundedMemory => "Working memory must have size limits and decay mechanism",
            Self::DeterministicSelection => {
                "Given same state, same operator must be selected"
            }
            Self::ProvenanceRequired => "Every decision must have auditable provenance chain",
            Self::NoNestedLLMCalls => "One LLM call per decision step maximum",
            Self::TerminationGuarantee => "All loops must have bounded iteration limits",
            Self::NoHiddenRouters => {
                "All task routing decisions must be logged with reasoning"
            }
            Self::FailClosed => "If uncertain, deny/stop rather than proceed",
            Self::CryptoAuditTrail => "All events must be content-hashed and append-only",
            Self::SealedExternalInterface => {
                "Tools cannot mutate agent internal state except via governed operators"
            }
        }
    }

    /// Get all core invariants
    pub fn all() -> [Self; 11] {
        [
            Self::NoFreeFormCoT,
            Self::ExplicitStateOnly,
            Self::BoundedMemory,
            Self::DeterministicSelection,
            Self::ProvenanceRequired,
            Self::NoNestedLLMCalls,
            Self::TerminationGuarantee,
            Self::NoHiddenRouters,
            Self::FailClosed,
            Self::CryptoAuditTrail,
            Self::SealedExternalInterface,
        ]
    }

    /// Check if this invariant is critical (blocks all execution if violated)
    pub fn is_critical(&self) -> bool {
        matches!(
            self,
            Self::NoFreeFormCoT
                | Self::ExplicitStateOnly
                | Self::FailClosed
                | Self::SealedExternalInterface
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invariant_ids() {
        assert_eq!(CoreInvariant::NoFreeFormCoT.id().as_str(), "INV-CORE-01");
        assert_eq!(
            CoreInvariant::SealedExternalInterface.id().as_str(),
            "INV-CORE-11"
        );
    }

    #[test]
    fn test_all_invariants_count() {
        assert_eq!(CoreInvariant::all().len(), 11);
    }

    #[test]
    fn test_critical_invariants() {
        assert!(CoreInvariant::NoFreeFormCoT.is_critical());
        assert!(CoreInvariant::FailClosed.is_critical());
        assert!(!CoreInvariant::BoundedMemory.is_critical());
    }

    #[test]
    fn test_invariant_ids_unique() {
        let ids: Vec<_> = CoreInvariant::all().iter().map(|i| i.id().as_str()).collect();
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(ids.len(), unique.len(), "All invariant IDs must be unique");
    }

    #[test]
    fn test_invariant_id_display() {
        let id = CoreInvariant::NoFreeFormCoT.id();
        let display = format!("{}", id);
        assert_eq!(display, "INV-CORE-01");
        assert!(!display.is_empty());
    }

    #[test]
    fn test_description_not_empty_and_distinct() {
        let descriptions: Vec<_> = CoreInvariant::all()
            .iter()
            .map(|i| i.description())
            .collect();

        // All descriptions should be non-empty
        for desc in &descriptions {
            assert!(!desc.is_empty(), "Description should not be empty");
        }

        // All descriptions should be unique
        let unique: std::collections::HashSet<_> = descriptions.iter().collect();
        assert_eq!(descriptions.len(), unique.len(), "All descriptions must be unique");
    }

    #[test]
    fn test_description_content() {
        // Verify specific descriptions contain expected keywords
        assert!(CoreInvariant::NoFreeFormCoT.description().contains("structured"));
        assert!(CoreInvariant::BoundedMemory.description().contains("memory"));
        assert!(CoreInvariant::TerminationGuarantee.description().contains("bound"));
    }
}
