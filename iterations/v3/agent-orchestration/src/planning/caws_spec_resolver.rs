//! CAWS Spec Resolver
//!
//! Implements CAWS spec resolution priority system for multi-agent workflows.
//! Resolves feature-specific specs using the priority order:
//! 1. Feature-specific spec (via spec_id): .caws/specs/<id>.yaml
//! 2. Explicit path (via spec_file)
//! 3. Auto-detect: If only 1 spec exists, use it automatically
//! 4. Legacy fallback: .caws/working-spec.yaml
//!
//! @author @darianrosebrook

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tracing::{debug, info, warn};

/// Spec information for listing and selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecInfo {
    /// Spec identifier (filename without .yaml extension)
    pub id: String,

    /// Full path to spec file
    pub path: PathBuf,

    /// Spec title (from working spec)
    pub title: String,

    /// Risk tier (from working spec)
    pub risk_tier: u8,

    /// Last modified timestamp
    pub last_modified: SystemTime,
}

/// CAWS Spec Resolver
pub struct CawsSpecResolver {
    /// Project root directory
    project_root: PathBuf,

    /// CAWS directory path (.caws)
    caws_directory: PathBuf,

    /// Specs directory path (.caws/specs)
    specs_directory: PathBuf,
}

impl CawsSpecResolver {
    /// Create new spec resolver
    pub fn new(project_root: impl AsRef<Path>) -> Result<Self> {
        let project_root = project_root.as_ref().to_path_buf();
        let caws_directory = project_root.join(".caws");
        let specs_directory = caws_directory.join("specs");

        Ok(Self {
            project_root,
            caws_directory,
            specs_directory,
        })
    }

    /// Resolve spec using CAWS priority system
    ///
    /// Priority order:
    /// 1. Feature-specific spec (via spec_id): .caws/specs/<id>.yaml
    /// 2. Explicit path (via spec_file)
    /// 3. Auto-detect: If only 1 spec exists, use it automatically
    /// 4. Legacy fallback: .caws/working-spec.yaml
    pub fn resolve_spec(&self, spec_id: Option<&str>, spec_file: Option<&Path>) -> Result<PathBuf> {
        // Priority 1: Feature-specific spec via spec_id
        if let Some(id) = spec_id {
            let spec_path = self.specs_directory.join(format!("{}.yaml", id));
            if spec_path.exists() {
                info!("Resolved spec via spec_id: {}", id);
                return Ok(spec_path);
            } else {
                return Err(anyhow!(
                    "Spec '{}' not found at {}",
                    id,
                    spec_path.display()
                ));
            }
        }

        // Priority 2: Explicit path
        if let Some(path) = spec_file {
            let resolved_path = if path.is_absolute() {
                path.to_path_buf()
            } else {
                self.project_root.join(path)
            };

            if resolved_path.exists() {
                info!(
                    "Resolved spec via explicit path: {}",
                    resolved_path.display()
                );
                return Ok(resolved_path);
            } else {
                return Err(anyhow!("Spec file not found: {}", resolved_path.display()));
            }
        }

        // Priority 3: Auto-detect (if only 1 spec exists)
        let available_specs = self.list_specs()?;
        if available_specs.len() == 1 {
            let spec = &available_specs[0];
            info!(
                "Auto-detected single spec: {} ({})",
                spec.id,
                spec.path.display()
            );
            return Ok(spec.path.clone());
        }

        // Priority 4: Legacy fallback
        let legacy_spec = self.caws_directory.join("working-spec.yaml");
        if legacy_spec.exists() {
            // Warn if multiple specs exist (multi-agent context)
            if available_specs.len() > 1 {
                warn!(
                    "Multiple specs detected ({}) but using legacy working-spec.yaml",
                    available_specs.len()
                );
                warn!("Consider using feature-specific specs: .caws/specs/<feature-id>.yaml");
                warn!(
                    "Available specs: {}",
                    available_specs
                        .iter()
                        .map(|s| s.id.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
            info!(
                "Resolved spec via legacy fallback: {}",
                legacy_spec.display()
            );
            return Ok(legacy_spec);
        }

        Err(anyhow!(
            "No CAWS spec found. Create one at .caws/working-spec.yaml or .caws/specs/<id>.yaml"
        ))
    }

    /// List all available specs
    pub fn list_specs(&self) -> Result<Vec<SpecInfo>> {
        let mut specs = Vec::new();

        // Check specs directory
        if self.specs_directory.exists() {
            let entries =
                fs::read_dir(&self.specs_directory).context("Failed to read specs directory")?;

            for entry in entries {
                let entry = entry.context("Failed to read directory entry")?;
                let path = entry.path();

                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                    if let Some(id) = path.file_stem().and_then(|s| s.to_str()) {
                        match self.load_spec_info(&path, id) {
                            Ok(info) => specs.push(info),
                            Err(e) => {
                                warn!("Failed to load spec info for {}: {}", path.display(), e);
                            }
                        }
                    }
                }
            }
        }

        // Also check legacy spec
        let legacy_spec = self.caws_directory.join("working-spec.yaml");
        if legacy_spec.exists() {
            match self.load_spec_info(&legacy_spec, "working-spec") {
                Ok(mut info) => {
                    info.id = "working-spec".to_string();
                    specs.push(info);
                }
                Err(e) => {
                    debug!("Failed to load legacy spec info: {}", e);
                }
            }
        }

        Ok(specs)
    }

    /// Load spec information from file
    fn load_spec_info(&self, path: &Path, id: &str) -> Result<SpecInfo> {
        let metadata = fs::metadata(path).context("Failed to read spec file metadata")?;

        let last_modified = metadata
            .modified()
            .context("Failed to get modification time")?;

        // Try to parse YAML to extract title and risk_tier
        let content = fs::read_to_string(path).context("Failed to read spec file")?;

        let (title, risk_tier) = self.parse_spec_metadata(&content)?;

        Ok(SpecInfo {
            id: id.to_string(),
            path: path.to_path_buf(),
            title,
            risk_tier,
            last_modified,
        })
    }

    /// Parse basic metadata from spec YAML
    fn parse_spec_metadata(&self, content: &str) -> Result<(String, u8)> {
        // Simple YAML parsing for title and risk_tier
        // For full parsing, we'd use serde_yaml, but this avoids extra dependencies
        let mut title = String::new();
        let mut risk_tier = 2u8; // Default to tier 2

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("title:") {
                if let Some(t) = trimmed.strip_prefix("title:") {
                    title = t.trim().trim_matches('"').trim_matches('\'').to_string();
                }
            } else if trimmed.starts_with("risk_tier:") {
                if let Some(rt) = trimmed.strip_prefix("risk_tier:") {
                    if let Ok(rt_num) = rt.trim().parse::<u8>() {
                        risk_tier = rt_num;
                    }
                }
            }

            // Stop after finding both
            if !title.is_empty() && risk_tier != 2 {
                break;
            }
        }

        // Fallback title if not found
        if title.is_empty() {
            title = "Untitled Spec".to_string();
        }

        Ok((title, risk_tier))
    }

    /// Detect if multi-agent context (multiple specs exist)
    pub fn is_multi_agent_context(&self) -> bool {
        match self.list_specs() {
            Ok(specs) => specs.len() > 1,
            Err(_) => false,
        }
    }

    /// Get project root
    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    /// Get CAWS directory
    pub fn caws_directory(&self) -> &Path {
        &self.caws_directory
    }

    /// Get specs directory
    pub fn specs_directory(&self) -> &Path {
        &self.specs_directory
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_resolve_feature_spec() {
        let temp_dir = TempDir::new().unwrap();
        let caws_dir = temp_dir.path().join(".caws");
        let specs_dir = caws_dir.join("specs");
        fs::create_dir_all(&specs_dir).unwrap();

        // Create feature spec
        let spec_path = specs_dir.join("user-auth.yaml");
        fs::write(&spec_path, "title: User Authentication\nrisk_tier: 2\n").unwrap();

        let resolver = CawsSpecResolver::new(temp_dir.path()).unwrap();
        let resolved = resolver.resolve_spec(Some("user-auth"), None).unwrap();

        assert_eq!(resolved, spec_path);
    }

    #[test]
    fn test_resolve_legacy_spec() {
        let temp_dir = TempDir::new().unwrap();
        let caws_dir = temp_dir.path().join(".caws");
        fs::create_dir_all(&caws_dir).unwrap();

        // Create legacy spec
        let legacy_path = caws_dir.join("working-spec.yaml");
        fs::write(&legacy_path, "title: Legacy Spec\nrisk_tier: 2\n").unwrap();

        let resolver = CawsSpecResolver::new(temp_dir.path()).unwrap();
        let resolved = resolver.resolve_spec(None, None).unwrap();

        assert_eq!(resolved, legacy_path);
    }

    #[test]
    fn test_auto_detect_single_spec() {
        let temp_dir = TempDir::new().unwrap();
        let caws_dir = temp_dir.path().join(".caws");
        let specs_dir = caws_dir.join("specs");
        fs::create_dir_all(&specs_dir).unwrap();

        // Create single spec
        let spec_path = specs_dir.join("feature-1.yaml");
        fs::write(&spec_path, "title: Feature 1\nrisk_tier: 2\n").unwrap();

        let resolver = CawsSpecResolver::new(temp_dir.path()).unwrap();
        let resolved = resolver.resolve_spec(None, None).unwrap();

        assert_eq!(resolved, spec_path);
    }

    #[test]
    fn test_multi_agent_context_detection() {
        let temp_dir = TempDir::new().unwrap();
        let caws_dir = temp_dir.path().join(".caws");
        let specs_dir = caws_dir.join("specs");
        fs::create_dir_all(&specs_dir).unwrap();

        // Create multiple specs
        fs::write(
            specs_dir.join("feature-1.yaml"),
            "title: Feature 1\nrisk_tier: 2\n",
        )
        .unwrap();
        fs::write(
            specs_dir.join("feature-2.yaml"),
            "title: Feature 2\nrisk_tier: 2\n",
        )
        .unwrap();

        let resolver = CawsSpecResolver::new(temp_dir.path()).unwrap();
        assert!(resolver.is_multi_agent_context());
    }

    #[test]
    fn test_list_specs() {
        let temp_dir = TempDir::new().unwrap();
        let caws_dir = temp_dir.path().join(".caws");
        let specs_dir = caws_dir.join("specs");
        fs::create_dir_all(&specs_dir).unwrap();

        // Create specs
        fs::write(
            specs_dir.join("feature-1.yaml"),
            "title: Feature 1\nrisk_tier: 1\n",
        )
        .unwrap();
        fs::write(
            specs_dir.join("feature-2.yaml"),
            "title: Feature 2\nrisk_tier: 3\n",
        )
        .unwrap();

        let resolver = CawsSpecResolver::new(temp_dir.path()).unwrap();
        let specs = resolver.list_specs().unwrap();

        assert_eq!(specs.len(), 2);
        assert!(specs
            .iter()
            .any(|s| s.id == "feature-1" && s.risk_tier == 1));
        assert!(specs
            .iter()
            .any(|s| s.id == "feature-2" && s.risk_tier == 3));
    }
}
