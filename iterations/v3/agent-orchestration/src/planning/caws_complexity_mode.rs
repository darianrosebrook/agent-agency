//! CAWS Complexity Mode
//!
//! Implements CAWS complexity tiers (Simple, Standard, Enterprise) with
//! mode-aware quality requirements. Detects mode from .caws/config.yaml
//! or .caws/mode file.
//!
//! @author @darianrosebrook

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tracing::info;

/// CAWS complexity mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CawsComplexityMode {
    /// Simple mode: 70% coverage, 30% mutation (small projects, quick prototyping)
    Simple,

    /// Standard mode: 80% coverage, 50% mutation (balanced teams, standard projects)
    Standard,

    /// Enterprise mode: 90% coverage, 70% mutation (large teams, regulated projects)
    Enterprise,
}

impl CawsComplexityMode {
    /// Detect mode from .caws/config.yaml or .caws/mode file
    pub fn detect(project_root: &Path) -> Result<Self> {
        let caws_dir = project_root.join(".caws");

        // Try .caws/config.yaml first
        let config_path = caws_dir.join("config.yaml");
        if config_path.exists() {
            match Self::from_config_file(&config_path) {
                Ok(mode) => {
                    info!("Detected complexity mode from config.yaml: {:?}", mode);
                    return Ok(mode);
                }
                Err(e) => {
                    debug!("Failed to read mode from config.yaml: {}", e);
                }
            }
        }

        // Try .caws/config.json
        let config_json_path = caws_dir.join("config.json");
        if config_json_path.exists() {
            match Self::from_config_json(&config_json_path) {
                Ok(mode) => {
                    info!("Detected complexity mode from config.json: {:?}", mode);
                    return Ok(mode);
                }
                Err(e) => {
                    debug!("Failed to read mode from config.json: {}", e);
                }
            }
        }

        // Try .caws/mode file (simple text file)
        let mode_file = caws_dir.join("mode");
        if mode_file.exists() {
            match Self::from_mode_file(&mode_file) {
                Ok(mode) => {
                    info!("Detected complexity mode from mode file: {:?}", mode);
                    return Ok(mode);
                }
                Err(e) => {
                    debug!("Failed to read mode file: {}", e);
                }
            }
        }

        // Default to Standard if no config found
        info!("No complexity mode config found, defaulting to Standard");
        Ok(Self::Standard)
    }

    /// Read mode from config.yaml
    fn from_config_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path).context("Failed to read config.yaml")?;

        let config: CawsConfig =
            serde_yaml::from_str(&content).context("Failed to parse config.yaml")?;

        config
            .mode
            .ok_or_else(|| anyhow!("No mode field in config.yaml"))
    }

    /// Read mode from config.json
    fn from_config_json(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path).context("Failed to read config.json")?;

        let config: CawsConfigJson =
            serde_json::from_str(&content).context("Failed to parse config.json")?;

        config
            .mode
            .ok_or_else(|| anyhow!("No mode field in config.json"))
    }

    /// Read mode from simple text file
    fn from_mode_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .context("Failed to read mode file")?
            .trim()
            .to_lowercase();

        match content.as_str() {
            "simple" => Ok(Self::Simple),
            "standard" => Ok(Self::Standard),
            "enterprise" => Ok(Self::Enterprise),
            _ => Err(anyhow!("Invalid mode value: {}", content)),
        }
    }

    /// Get quality requirements for mode + risk tier combination
    pub fn quality_requirements(&self, risk_tier: u8) -> QualityRequirements {
        // Base requirements from complexity mode
        let (base_coverage, base_mutation) = match self {
            Self::Simple => (0.70, 0.30),
            Self::Standard => (0.80, 0.50),
            Self::Enterprise => (0.90, 0.70),
        };

        // Adjust based on risk tier (higher tier = higher requirements)
        let tier_multiplier = match risk_tier {
            1 => 1.0,  // Tier 1 uses base requirements
            2 => 0.95, // Tier 2 slightly lower
            3 => 0.90, // Tier 3 lower still
            _ => 1.0,
        };

        QualityRequirements {
            line_coverage: base_coverage * tier_multiplier,
            branch_coverage: base_coverage * 0.9 * tier_multiplier, // Branch coverage is 90% of line
            mutation_score: base_mutation * tier_multiplier,
            contracts_required: matches!(self, Self::Standard | Self::Enterprise),
            manual_review_required: matches!(self, Self::Enterprise) || risk_tier == 1,
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Simple => "Simple",
            Self::Standard => "Standard",
            Self::Enterprise => "Enterprise",
        }
    }

    /// Get description
    pub fn description(&self) -> &'static str {
        match self {
            Self::Simple => "Minimal CAWS for small projects and quick prototyping",
            Self::Standard => "Balanced CAWS with change management and quality gates",
            Self::Enterprise => "Full CAWS with comprehensive audit trails and compliance",
        }
    }
}

impl Default for CawsComplexityMode {
    fn default() -> Self {
        Self::Standard
    }
}

/// Quality requirements for a mode + tier combination
#[derive(Debug, Clone)]
pub struct QualityRequirements {
    /// Minimum line coverage (0.0 to 1.0)
    pub line_coverage: f64,

    /// Minimum branch coverage (0.0 to 1.0)
    pub branch_coverage: f64,

    /// Minimum mutation score (0.0 to 1.0)
    pub mutation_score: f64,

    /// Whether contracts are required
    pub contracts_required: bool,

    /// Whether manual review is required
    pub manual_review_required: bool,
}

/// CAWS config structure (for YAML parsing)
#[derive(Debug, Deserialize)]
struct CawsConfig {
    mode: Option<CawsComplexityMode>,
}

/// CAWS config structure (for JSON parsing)
#[derive(Debug, Deserialize)]
struct CawsConfigJson {
    mode: Option<CawsComplexityMode>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_detect_from_mode_file() {
        let temp_dir = TempDir::new().unwrap();
        let caws_dir = temp_dir.path().join(".caws");
        fs::create_dir_all(&caws_dir).unwrap();

        // Create mode file
        fs::write(caws_dir.join("mode"), "enterprise").unwrap();

        let mode = CawsComplexityMode::detect(temp_dir.path()).unwrap();
        assert_eq!(mode, CawsComplexityMode::Enterprise);
    }

    #[test]
    fn test_detect_from_config_yaml() {
        let temp_dir = TempDir::new().unwrap();
        let caws_dir = temp_dir.path().join(".caws");
        fs::create_dir_all(&caws_dir).unwrap();

        // Create config.yaml
        fs::write(caws_dir.join("config.yaml"), "mode: simple\n").unwrap();

        let mode = CawsComplexityMode::detect(temp_dir.path()).unwrap();
        assert_eq!(mode, CawsComplexityMode::Simple);
    }

    #[test]
    fn test_default_to_standard() {
        let temp_dir = TempDir::new().unwrap();

        // No config files
        let mode = CawsComplexityMode::detect(temp_dir.path()).unwrap();
        assert_eq!(mode, CawsComplexityMode::Standard);
    }

    #[test]
    fn test_quality_requirements_simple() {
        let mode = CawsComplexityMode::Simple;
        let reqs = mode.quality_requirements(2);

        assert_eq!(reqs.line_coverage, 0.70 * 0.95);
        assert_eq!(reqs.mutation_score, 0.30 * 0.95);
        assert!(!reqs.contracts_required);
    }

    #[test]
    fn test_quality_requirements_standard() {
        let mode = CawsComplexityMode::Standard;
        let reqs = mode.quality_requirements(2);

        assert_eq!(reqs.line_coverage, 0.80 * 0.95);
        assert_eq!(reqs.mutation_score, 0.50 * 0.95);
        assert!(reqs.contracts_required);
    }

    #[test]
    fn test_quality_requirements_enterprise() {
        let mode = CawsComplexityMode::Enterprise;
        let reqs = mode.quality_requirements(1);

        assert_eq!(reqs.line_coverage, 0.90);
        assert_eq!(reqs.mutation_score, 0.70);
        assert!(reqs.contracts_required);
        assert!(reqs.manual_review_required);
    }
}
