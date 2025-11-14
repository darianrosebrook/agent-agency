//! CAWS Quality Gates Integration
//!
//! Integrates CAWS quality gates script (run-quality-gates.mjs) with waiver recognition
//! into the adjudication cycle. This module invokes the quality gates script and
//! parses its JSON output to extract violations and waiver information.
//!
//! @author @darianrosebrook

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{debug, info, warn};

use super::caws_complexity_mode::CawsComplexityMode;

/// Quality gates execution result with waiver information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CawsQualityGateResult {
    /// Whether all non-waived gates passed
    pub passed: bool,

    /// Total violations found
    pub total_violations: usize,

    /// Non-waived violations (blocking)
    pub blocking_violations: usize,

    /// Waived violations (non-blocking)
    pub waived_violations: usize,

    /// Active waivers count
    pub active_waivers: usize,

    /// Detailed violation information
    pub violations: Vec<QualityGateViolation>,

    /// Active waivers information
    pub waivers: Vec<WaiverInfo>,

    /// Execution context (commit, push, ci)
    pub context: String,

    /// Files scoped for checking
    pub files_scoped: usize,
}

/// Individual quality gate violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGateViolation {
    /// Gate name (e.g., "naming", "duplication")
    pub gate: String,

    /// Violation type
    pub r#type: String,

    /// Violation message
    pub message: String,

    /// File path (if applicable)
    pub file: Option<String>,

    /// Whether this violation is waived
    pub waived: bool,

    /// Waiver ID if waived
    pub waived_by: Option<String>,

    /// Waiver title if waived
    pub waiver_title: Option<String>,

    /// Waiver expiration date if waived
    pub waiver_expires: Option<String>,
}

/// Waiver information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaiverInfo {
    /// Waiver ID
    pub id: String,

    /// Waiver title
    pub title: String,

    /// Gates covered by this waiver
    pub gates: Vec<String>,

    /// Expiration date
    pub expires_at: String,
}

/// CAWS Quality Gates Executor
pub struct CawsQualityGateExecutor {
    /// Project root directory
    project_root: PathBuf,

    /// Path to quality gates script
    quality_gates_script: PathBuf,
}

impl CawsQualityGateExecutor {
    /// Create a new CAWS quality gate executor
    pub fn new(project_root: impl AsRef<Path>) -> Result<Self> {
        let project_root = project_root.as_ref().to_path_buf();

        // Find quality gates script relative to project root
        // Try agent-agency/scripts/quality-gates/run-quality-gates.mjs first
        let script_paths = vec![
            project_root.join("scripts/quality-gates/run-quality-gates.mjs"),
            project_root.join("../scripts/quality-gates/run-quality-gates.mjs"),
            project_root.join("../../scripts/quality-gates/run-quality-gates.mjs"),
        ];

        let quality_gates_script = script_paths
            .iter()
            .find(|p| p.exists())
            .ok_or_else(|| anyhow::anyhow!("Could not find run-quality-gates.mjs script"))?
            .clone();

        Ok(Self {
            project_root,
            quality_gates_script,
        })
    }

    /// Execute CAWS quality gates and return results with waiver information
    pub async fn execute_quality_gates(
        &self,
        context: &str, // "commit", "push", or "ci"
    ) -> Result<CawsQualityGateResult> {
        self.execute_quality_gates_with_mode(context, None).await
    }

    /// Execute CAWS quality gates with complexity mode
    pub async fn execute_quality_gates_with_mode(
        &self,
        context: &str, // "commit", "push", or "ci"
        complexity_mode: Option<CawsComplexityMode>,
    ) -> Result<CawsQualityGateResult> {
        info!("Executing CAWS quality gates with context: {}", context);

        // Build command to execute quality gates script
        let mut cmd = Command::new("node");
        cmd.arg(&self.quality_gates_script)
            .arg("--context")
            .arg(context)
            .arg("--json")
            .arg("--quiet");

        // Add mode parameter if provided
        if let Some(mode) = complexity_mode {
            let mode_str = match mode {
                CawsComplexityMode::Simple => "simple",
                CawsComplexityMode::Standard => "standard",
                CawsComplexityMode::Enterprise => "enterprise",
            };
            cmd.arg("--mode").arg(mode_str);
            debug!("Using complexity mode: {}", mode_str);
        }

        let output = cmd
            .current_dir(&self.project_root)
            .output()
            .context("Failed to execute quality gates script")?;

        // Parse JSON output
        let stdout_str = String::from_utf8_lossy(&output.stdout);

        // The script outputs JSON to stdout when --json flag is used
        let report: serde_json::Value = serde_json::from_str(&stdout_str)
            .context("Failed to parse quality gates JSON output")?;

        // Extract violations and waiver information
        let violations = self.parse_violations(&report)?;
        let waivers = self.parse_waivers(&report)?;

        // Separate waived and blocking violations
        let waived_violations: Vec<_> = violations.iter().filter(|v| v.waived).cloned().collect();

        let blocking_violations: Vec<_> =
            violations.iter().filter(|v| !v.waived).cloned().collect();

        let total_violations = violations.len();
        let blocking_count = blocking_violations.len();
        let waived_count = waived_violations.len();
        let active_waivers_count = waivers.len();

        let passed = blocking_count == 0;

        // Extract context and files_scoped from report
        let context_str = report
            .get("context")
            .and_then(|c| c.as_str())
            .unwrap_or(context)
            .to_string();

        let files_scoped = report
            .get("files_scoped")
            .and_then(|f| f.as_u64())
            .unwrap_or(0) as usize;

        if passed {
            info!(
                "Quality gates passed: {} violations ({} waived, {} blocking)",
                total_violations, waived_count, blocking_count
            );
        } else {
            warn!(
                "Quality gates failed: {} blocking violations ({} waived)",
                blocking_count, waived_count
            );
        }

        Ok(CawsQualityGateResult {
            passed,
            total_violations,
            blocking_violations: blocking_count,
            waived_violations: waived_count,
            active_waivers: active_waivers_count,
            violations,
            waivers,
            context: context_str,
            files_scoped,
        })
    }

    /// Parse violations from quality gates report
    fn parse_violations(&self, report: &serde_json::Value) -> Result<Vec<QualityGateViolation>> {
        let empty_vec = Vec::new();
        let violations_array = report
            .get("violations")
            .and_then(|v| v.as_array())
            .unwrap_or(&empty_vec);

        let mut violations = Vec::new();

        for violation_json in violations_array {
            let gate = violation_json
                .get("gate")
                .and_then(|g| g.as_str())
                .unwrap_or("unknown")
                .to_string();

            let violation_type = violation_json
                .get("type")
                .and_then(|t| t.as_str())
                .unwrap_or("unknown")
                .to_string();

            let message = violation_json
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("")
                .to_string();

            let file = violation_json
                .get("file")
                .and_then(|f| f.as_str())
                .map(|f| f.to_string());

            // Check if violation is waived
            let waived_by = violation_json
                .get("waivedBy")
                .and_then(|w| w.as_str())
                .map(|w| w.to_string());

            let waived = waived_by.is_some();

            let waiver_title = violation_json
                .get("waiverTitle")
                .and_then(|t| t.as_str())
                .map(|t| t.to_string());

            let waiver_expires = violation_json
                .get("waiverExpires")
                .and_then(|e| e.as_str())
                .map(|e| e.to_string());

            violations.push(QualityGateViolation {
                gate,
                r#type: violation_type,
                message,
                file,
                waived,
                waived_by,
                waiver_title,
                waiver_expires,
            });
        }

        Ok(violations)
    }

    /// Parse waivers from quality gates report
    fn parse_waivers(&self, report: &serde_json::Value) -> Result<Vec<WaiverInfo>> {
        let empty_vec = Vec::new();
        let waivers_obj = report
            .get("waivers")
            .and_then(|w| w.get("details"))
            .and_then(|d| d.as_array())
            .unwrap_or(&empty_vec);

        let mut waivers = Vec::new();

        for waiver_json in waivers_obj {
            let id = waiver_json
                .get("id")
                .and_then(|i| i.as_str())
                .unwrap_or("unknown")
                .to_string();

            let title = waiver_json
                .get("title")
                .and_then(|t| t.as_str())
                .unwrap_or("Unknown waiver")
                .to_string();

            let gates = waiver_json
                .get("gates")
                .and_then(|g| g.as_array())
                .map(|g| {
                    g.iter()
                        .filter_map(|gate| gate.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default();

            let expires_at = waiver_json
                .get("expires_at")
                .and_then(|e| e.as_str())
                .unwrap_or("")
                .to_string();

            waivers.push(WaiverInfo {
                id,
                title,
                gates,
                expires_at,
            });
        }

        Ok(waivers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_violations_with_waivers() {
        // Try to find project root by looking for scripts directory
        // Start from current directory and walk up
        let project_root = std::env::current_dir()
            .ok()
            .and_then(|mut path| {
                loop {
                    if path
                        .join("scripts/quality-gates/run-quality-gates.mjs")
                        .exists()
                    {
                        return Some(path);
                    }
                    if !path.pop() {
                        break;
                    }
                }
                None
            })
            .unwrap_or_else(|| std::path::PathBuf::from("."));

        // If script not found, skip test with a clear message
        let executor = match CawsQualityGateExecutor::new(&project_root) {
            Ok(exec) => exec,
            Err(e) => {
                eprintln!("Skipping test: Quality gates script not found: {}", e);
                return;
            }
        };

        let report_json = serde_json::json!({
            "violations": [
                {
                    "gate": "naming",
                    "type": "banned_pattern",
                    "message": "File uses banned naming pattern",
                    "file": "src/test.ts",
                    "waivedBy": "WAIVER-001",
                    "waiverTitle": "Emergency hotfix waiver",
                    "waiverExpires": "2024-12-31T23:59:59Z"
                },
                {
                    "gate": "duplication",
                    "type": "high_duplication",
                    "message": "High code duplication detected",
                    "file": "src/utils.ts"
                }
            ],
            "waivers": {
                "active": 1,
                "applied": 1,
                "details": [
                    {
                        "id": "WAIVER-001",
                        "title": "Emergency hotfix waiver",
                        "gates": ["naming"],
                        "expires_at": "2024-12-31T23:59:59Z"
                    }
                ]
            }
        });

        let violations = executor.parse_violations(&report_json).unwrap();
        assert_eq!(violations.len(), 2);
        assert!(violations[0].waived);
        assert_eq!(violations[0].waived_by, Some("WAIVER-001".to_string()));
        assert!(!violations[1].waived);
    }
}
