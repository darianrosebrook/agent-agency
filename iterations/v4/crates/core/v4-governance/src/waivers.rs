//! CAWS Waiver Types
//!
//! Rust types for CAWS quality gate waivers. Waivers provide temporary,
//! justified exemptions from quality gates with expiration and audit trail.
//! Matches the schema at `.caws/schemas/waivers.schema.json`.
//!
//! Generated via MiniMax M2.5 worker-augmented pipeline, reviewed and
//! integrated by Claude.

use std::collections::HashMap;
use std::path::Path;

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::working_spec::WaiverCheck;

// ─── Gate Types ───────────────────────────────────────────────────────────────

/// Quality gate that a waiver can exempt.
///
/// Variant names are the canonical identifiers. The YAML files use different
/// snake_case strings (e.g., `coverage_threshold` for `Coverage`), handled
/// by `from_yaml_str()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WaiverGate {
    Coverage,
    Mutation,
    HiddenTodo,
    ChangeBudget,
    NamingBannedModifiers,
    DocumentationIntegrity,
    Contracts,
    Documentation,
    Security,
}

impl WaiverGate {
    /// Parse from the snake_case strings used in CAWS YAML files.
    pub fn from_yaml_str(s: &str) -> Option<Self> {
        match s {
            "coverage_threshold" | "coverage" => Some(Self::Coverage),
            "mutation_score" | "mutation" => Some(Self::Mutation),
            "hidden_todo" | "hidden_todos_check" | "hidden-todos" => Some(Self::HiddenTodo),
            "change_budget_check" | "change_budget" | "budget-compliance" => {
                Some(Self::ChangeBudget)
            }
            "naming_banned_modifiers_check" | "naming_banned_modifiers" => {
                Some(Self::NamingBannedModifiers)
            }
            "documentation_integrity_check" | "documentation_integrity" => {
                Some(Self::DocumentationIntegrity)
            }
            "documentation_quality" | "documentation-quality" => Some(Self::Documentation),
            "contracts" => Some(Self::Contracts),
            "security" => Some(Self::Security),
            // Also handle the kebab-case variants seen in active-waivers.yaml
            "quality-gates" => None, // Too broad — skip
            _ => None,
        }
    }
}

// ─── Status ───────────────────────────────────────────────────────────────────

/// Current status of a waiver.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WaiverStatus {
    Active,
    Expired,
    Revoked,
}

// ─── Waiver ───────────────────────────────────────────────────────────────────

/// A single quality gate waiver — one gate exemption per entry.
///
/// When the YAML source has `gates: [a, b]`, this expands into two `Waiver`
/// instances sharing the same `id`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Waiver {
    /// Waiver identifier (e.g., "WV-0371")
    pub id: String,
    /// Which gate is waived
    pub gate: WaiverGate,
    /// Why the waiver was granted
    pub reason: String,
    /// Who approved the waiver
    pub approved_by: String,
    /// When the waiver expires
    pub expiry: DateTime<Utc>,
    /// What compensating controls exist
    #[serde(default)]
    pub compensating_control: Option<String>,
    /// Current status
    pub status: WaiverStatus,
    /// Human-readable title
    #[serde(default)]
    pub title: Option<String>,
    /// Detailed description
    #[serde(default)]
    pub description: Option<String>,
    /// When the waiver was created
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
    /// Impact level (low, medium, high, critical)
    #[serde(default)]
    pub impact_level: Option<String>,
}

impl Waiver {
    /// Whether this waiver is currently active (correct status AND not expired).
    pub fn is_active(&self) -> bool {
        self.status == WaiverStatus::Active && self.expiry > Utc::now()
    }
}

impl WaiverCheck for Waiver {
    fn is_budget_waiver(&self) -> bool {
        matches!(self.gate, WaiverGate::ChangeBudget)
    }

    fn is_active_now(&self) -> bool {
        self.is_active()
    }
}

// ─── WaiverSet ────────────────────────────────────────────────────────────────

/// Collection of waivers, loaded from CAWS YAML files.
#[derive(Debug, Clone, Default)]
pub struct WaiverSet {
    waivers: Vec<Waiver>,
}

impl WaiverSet {
    /// Create an empty waiver set.
    pub fn new() -> Self {
        Self { waivers: Vec::new() }
    }

    /// Load waivers from a CAWS `active-waivers.yaml` file.
    ///
    /// The YAML format is:
    /// ```yaml
    /// waivers:
    ///   WV-XXXX:
    ///     id: WV-XXXX
    ///     gates: [coverage_threshold, hidden_todo]
    ///     reason: ...
    ///     expires_at: "2025-12-31"
    ///     approved_by: ...
    /// ```
    ///
    /// Each entry with `gates: [a, b]` expands into separate `Waiver` instances.
    pub fn load_from_yaml(path: &Path) -> Result<Self, WaiverError> {
        let content =
            std::fs::read_to_string(path).map_err(|e| WaiverError::Io(e.to_string()))?;
        Self::from_yaml_str(&content)
    }

    /// Parse waivers from a YAML string.
    pub fn from_yaml_str(yaml: &str) -> Result<Self, WaiverError> {
        let data: YamlWaiverFile =
            serde_yaml::from_str(yaml).map_err(|e| WaiverError::Parse(e.to_string()))?;

        let mut waivers = Vec::new();

        for (key, entry) in &data.waivers {
            let expiry = parse_expiry(&entry.expires_at)?;
            let status = if expiry > Utc::now() {
                WaiverStatus::Active
            } else {
                WaiverStatus::Expired
            };

            let created_at = entry
                .created_at
                .as_deref()
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc));

            let impact_level = entry
                .risk_assessment
                .as_ref()
                .and_then(|r| r.get("impact_level"))
                .and_then(|v| v.as_str())
                .map(String::from);

            let compensating_control = entry
                .risk_assessment
                .as_ref()
                .and_then(|r| r.get("mitigation_plan"))
                .and_then(|v| v.as_str())
                .map(String::from);

            for gate_str in &entry.gates {
                if let Some(gate) = WaiverGate::from_yaml_str(gate_str) {
                    waivers.push(Waiver {
                        id: entry.id.clone().unwrap_or_else(|| key.clone()),
                        gate,
                        reason: entry.reason.clone(),
                        approved_by: entry
                            .approved_by
                            .clone()
                            .unwrap_or_else(|| "unknown".to_string()),
                        expiry,
                        compensating_control: compensating_control.clone(),
                        status,
                        title: entry.title.clone(),
                        description: entry.description.clone(),
                        created_at,
                        impact_level: impact_level.clone(),
                    });
                }
                // Skip unrecognized gates silently — the YAML has some like
                // "quality-gates" that are too broad to map.
            }
        }

        Ok(Self { waivers })
    }

    /// Find active waivers for a specific gate.
    pub fn find_for_gate(&self, gate: WaiverGate) -> Vec<&Waiver> {
        self.waivers
            .iter()
            .filter(|w| w.gate == gate && w.is_active())
            .collect()
    }

    /// Get all waivers (active and expired).
    pub fn all(&self) -> &[Waiver] {
        &self.waivers
    }

    /// Count of all waivers.
    pub fn len(&self) -> usize {
        self.waivers.len()
    }

    /// Whether the set is empty.
    pub fn is_empty(&self) -> bool {
        self.waivers.is_empty()
    }

    /// Check if any active waiver exists for a given gate.
    pub fn has_active_waiver(&self, gate: WaiverGate) -> bool {
        self.waivers
            .iter()
            .any(|w| w.gate == gate && w.is_active())
    }
}

// ─── YAML Deserialization Helpers ─────────────────────────────────────────────

/// Top-level YAML structure for active-waivers.yaml
#[derive(Debug, Deserialize)]
struct YamlWaiverFile {
    #[serde(default)]
    waivers: HashMap<String, YamlWaiverEntry>,
}

/// Single waiver entry in the YAML file
#[derive(Debug, Clone, Deserialize)]
struct YamlWaiverEntry {
    #[serde(default)]
    id: Option<String>,
    gates: Vec<String>,
    reason: String,
    #[serde(default)]
    expires_at: Option<String>,
    #[serde(default)]
    approved_by: Option<String>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    created_at: Option<String>,
    #[serde(default)]
    risk_assessment: Option<HashMap<String, serde_json::Value>>,
    // We ignore other fields (metadata, etc.) gracefully via flatten
}

/// Parse the `expires_at` field which can be either a date ("2025-12-31")
/// or a datetime string ("2025-12-31T00:00:00Z").
fn parse_expiry(value: &Option<String>) -> Result<DateTime<Utc>, WaiverError> {
    let s = value
        .as_deref()
        .ok_or_else(|| WaiverError::Parse("Missing expires_at field".to_string()))?;

    // Try RFC3339 first (full datetime)
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Ok(dt.with_timezone(&Utc));
    }

    // Try date-only format (e.g., "2025-12-31")
    if let Ok(date) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        let dt = date
            .and_hms_opt(23, 59, 59)
            .expect("valid time")
            .and_utc();
        return Ok(dt);
    }

    Err(WaiverError::Parse(format!(
        "Could not parse expires_at: '{s}'. Expected RFC3339 or YYYY-MM-DD."
    )))
}

// ─── Errors ───────────────────────────────────────────────────────────────────

/// Errors when loading or processing waivers.
#[derive(Debug, Error)]
pub enum WaiverError {
    #[error("IO error: {0}")]
    Io(String),
    #[error("Parse error: {0}")]
    Parse(String),
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_YAML: &str = r#"
waivers:
  WV-001:
    id: WV-001
    gates:
      - coverage_threshold
      - mutation_score
    reason: "Legacy code coverage waiver"
    expires_at: "2030-01-01T00:00:00Z"
    approved_by: "darianrosebrook"
    title: "Legacy Module Waiver"
    description: "Waiver for legacy modules during migration"
    risk_assessment:
      impact_level: "low"
      mitigation_plan: "Migration scheduled for Q2"

  WV-002:
    id: WV-002
    gates:
      - change_budget_check
    reason: "Infrastructure upgrade exceeds budget"
    expires_at: "2030-06-01"
    approved_by: "engineering-lead"
    title: "Budget Waiver"

  WV-EXPIRED:
    id: WV-EXPIRED
    gates:
      - hidden_todo
    reason: "Old waiver that should be expired"
    expires_at: "2020-01-01"
    approved_by: "someone"
"#;

    // ── Gate Parsing ───────────────────────────────────────────────────────

    #[test]
    fn test_gate_from_yaml_str() {
        assert_eq!(
            WaiverGate::from_yaml_str("coverage_threshold"),
            Some(WaiverGate::Coverage)
        );
        assert_eq!(
            WaiverGate::from_yaml_str("mutation_score"),
            Some(WaiverGate::Mutation)
        );
        assert_eq!(
            WaiverGate::from_yaml_str("hidden_todo"),
            Some(WaiverGate::HiddenTodo)
        );
        assert_eq!(
            WaiverGate::from_yaml_str("hidden_todos_check"),
            Some(WaiverGate::HiddenTodo)
        );
        assert_eq!(
            WaiverGate::from_yaml_str("change_budget_check"),
            Some(WaiverGate::ChangeBudget)
        );
        assert_eq!(
            WaiverGate::from_yaml_str("naming_banned_modifiers_check"),
            Some(WaiverGate::NamingBannedModifiers)
        );
        assert_eq!(
            WaiverGate::from_yaml_str("documentation_integrity_check"),
            Some(WaiverGate::DocumentationIntegrity)
        );
        assert_eq!(WaiverGate::from_yaml_str("unknown_gate"), None);
    }

    // ── Waiver Active Check ────────────────────────────────────────────────

    #[test]
    fn test_is_active_future_expiry() {
        let waiver = Waiver {
            id: "WV-TEST".to_string(),
            gate: WaiverGate::Coverage,
            reason: "test".to_string(),
            approved_by: "tester".to_string(),
            expiry: Utc::now() + chrono::Duration::days(30),
            compensating_control: None,
            status: WaiverStatus::Active,
            title: None,
            description: None,
            created_at: None,
            impact_level: None,
        };
        assert!(waiver.is_active());
    }

    #[test]
    fn test_is_active_past_expiry() {
        let waiver = Waiver {
            id: "WV-TEST".to_string(),
            gate: WaiverGate::Coverage,
            reason: "test".to_string(),
            approved_by: "tester".to_string(),
            expiry: Utc::now() - chrono::Duration::days(30),
            compensating_control: None,
            status: WaiverStatus::Active,
            title: None,
            description: None,
            created_at: None,
            impact_level: None,
        };
        assert!(!waiver.is_active());
    }

    #[test]
    fn test_is_active_revoked_status() {
        let waiver = Waiver {
            id: "WV-TEST".to_string(),
            gate: WaiverGate::Coverage,
            reason: "test".to_string(),
            approved_by: "tester".to_string(),
            expiry: Utc::now() + chrono::Duration::days(30),
            compensating_control: None,
            status: WaiverStatus::Revoked,
            title: None,
            description: None,
            created_at: None,
            impact_level: None,
        };
        assert!(!waiver.is_active());
    }

    // ── WaiverCheck Trait ──────────────────────────────────────────────────

    #[test]
    fn test_waiver_check_budget() {
        let budget_waiver = Waiver {
            id: "WV-B".to_string(),
            gate: WaiverGate::ChangeBudget,
            reason: "budget".to_string(),
            approved_by: "lead".to_string(),
            expiry: Utc::now() + chrono::Duration::days(30),
            compensating_control: None,
            status: WaiverStatus::Active,
            title: None,
            description: None,
            created_at: None,
            impact_level: None,
        };
        assert!(budget_waiver.is_budget_waiver());
        assert!(budget_waiver.is_active_now());

        let coverage_waiver = Waiver {
            id: "WV-C".to_string(),
            gate: WaiverGate::Coverage,
            reason: "coverage".to_string(),
            approved_by: "lead".to_string(),
            expiry: Utc::now() + chrono::Duration::days(30),
            compensating_control: None,
            status: WaiverStatus::Active,
            title: None,
            description: None,
            created_at: None,
            impact_level: None,
        };
        assert!(!coverage_waiver.is_budget_waiver());
    }

    // ── YAML Loading ───────────────────────────────────────────────────────

    #[test]
    fn test_load_sample_yaml() {
        let set = WaiverSet::from_yaml_str(SAMPLE_YAML).unwrap();
        // WV-001 has 2 gates, WV-002 has 1, WV-EXPIRED has 1 = 4 total
        assert_eq!(set.len(), 4);
    }

    #[test]
    fn test_load_yaml_expands_gates() {
        let set = WaiverSet::from_yaml_str(SAMPLE_YAML).unwrap();
        let coverage: Vec<_> = set.all().iter().filter(|w| w.gate == WaiverGate::Coverage).collect();
        assert_eq!(coverage.len(), 1);
        assert_eq!(coverage[0].id, "WV-001");

        let mutation: Vec<_> = set.all().iter().filter(|w| w.gate == WaiverGate::Mutation).collect();
        assert_eq!(mutation.len(), 1);
        assert_eq!(mutation[0].id, "WV-001");
    }

    #[test]
    fn test_load_yaml_date_only_format() {
        let set = WaiverSet::from_yaml_str(SAMPLE_YAML).unwrap();
        let budget: Vec<_> = set
            .all()
            .iter()
            .filter(|w| w.gate == WaiverGate::ChangeBudget)
            .collect();
        assert_eq!(budget.len(), 1);
        assert_eq!(budget[0].id, "WV-002");
        // Date-only "2030-06-01" parsed to end of day
        assert!(chrono::Datelike::year(&budget[0].expiry) == 2030);
    }

    #[test]
    fn test_load_yaml_expired_waiver_gets_status() {
        let set = WaiverSet::from_yaml_str(SAMPLE_YAML).unwrap();
        let expired: Vec<_> = set
            .all()
            .iter()
            .filter(|w| w.id == "WV-EXPIRED")
            .collect();
        assert_eq!(expired.len(), 1);
        assert_eq!(expired[0].status, WaiverStatus::Expired);
        assert!(!expired[0].is_active());
    }

    #[test]
    fn test_load_yaml_risk_assessment() {
        let set = WaiverSet::from_yaml_str(SAMPLE_YAML).unwrap();
        let wv001: Vec<_> = set.all().iter().filter(|w| w.id == "WV-001").collect();
        assert!(!wv001.is_empty());
        assert_eq!(wv001[0].impact_level.as_deref(), Some("low"));
        assert!(wv001[0].compensating_control.is_some());
    }

    // ── find_for_gate ──────────────────────────────────────────────────────

    #[test]
    fn test_find_for_gate_active_only() {
        let set = WaiverSet::from_yaml_str(SAMPLE_YAML).unwrap();
        // HiddenTodo waiver exists but is expired → should not appear
        let hidden = set.find_for_gate(WaiverGate::HiddenTodo);
        assert!(hidden.is_empty());
    }

    #[test]
    fn test_find_for_gate_returns_active() {
        let set = WaiverSet::from_yaml_str(SAMPLE_YAML).unwrap();
        let coverage = set.find_for_gate(WaiverGate::Coverage);
        assert_eq!(coverage.len(), 1);
        assert_eq!(coverage[0].id, "WV-001");
    }

    #[test]
    fn test_find_for_gate_budget() {
        let set = WaiverSet::from_yaml_str(SAMPLE_YAML).unwrap();
        let budget = set.find_for_gate(WaiverGate::ChangeBudget);
        assert_eq!(budget.len(), 1);
        assert_eq!(budget[0].id, "WV-002");
    }

    #[test]
    fn test_has_active_waiver() {
        let set = WaiverSet::from_yaml_str(SAMPLE_YAML).unwrap();
        assert!(set.has_active_waiver(WaiverGate::Coverage));
        assert!(set.has_active_waiver(WaiverGate::ChangeBudget));
        assert!(!set.has_active_waiver(WaiverGate::HiddenTodo)); // expired
        assert!(!set.has_active_waiver(WaiverGate::Security)); // none exists
    }

    #[test]
    fn test_empty_waiver_set() {
        let yaml = "waivers: {}";
        let set = WaiverSet::from_yaml_str(yaml).unwrap();
        assert!(set.is_empty());
        assert_eq!(set.len(), 0);
    }
}
