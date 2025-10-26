//! Quality gate runner

use crate::checks::*;
use crate::config::{QualityGateConfig, QualityGateResults, QualityViolation};
use crate::rules::*;
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;
use walkdir::WalkDir;

/// Quality gate runner
pub struct QualityGateRunner {
    config: QualityGateConfig,
    rules: Vec<Box<dyn QualityRule>>,
}

impl QualityGateRunner {
    /// Create a new quality gate runner with default configuration
    pub fn new() -> Self {
        Self::with_config(QualityGateConfig::default())
    }

    /// Create a new quality gate runner with custom configuration
    pub fn with_config(config: QualityGateConfig) -> Self {
        let rules: Vec<Box<dyn QualityRule>> = vec![
            Box::new(GodObjectRule),
            Box::new(DuplicateNameRule),
            Box::new(FunctionComplexityRule),
            Box::new(StructComplexityRule),
            Box::new(PlaceholderRule),
        ];

        Self { config, rules }
    }

    /// Run quality gates on a directory
    pub async fn run_on_directory(&self, dir_path: &Path) -> Result<QualityGateResults, anyhow::Error> {
        let start_time = Instant::now();
        let mut results = QualityGateResults::new();

        // Collect all source files
        let files = self.collect_source_files(dir_path)?;

        results.total_files_checked = files.len();

        // Run per-file rules
        for (file_path, content) in &files {
            for rule in &self.rules {
                let violations = rule.check_file(file_path, content, &self.config);
                for violation in violations {
                    results.add_violation(violation);
                }
            }
        }

        // Run global checks
        let duplicate_violations = check_duplicate_names(&files, &self.config);
        for violation in duplicate_violations {
            results.add_violation(violation);
        }

        let arch_violations = check_architecture_violations(&files, &self.config);
        for violation in arch_violations {
            results.add_violation(violation);
        }

        let security_violations = check_security_violations(&files, &self.config);
        for violation in security_violations {
            results.add_violation(violation);
        }

        let dependency_violations = check_dependency_violations(&files, &self.config);
        for violation in dependency_violations {
            results.add_violation(violation);
        }

        results.execution_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(results)
    }

    /// Run quality gates on a single file
    pub fn run_on_file(&self, file_path: &Path) -> Result<QualityGateResults, anyhow::Error> {
        let start_time = Instant::now();
        let mut results = QualityGateResults::new();

        if !file_path.exists() {
            return Err(anyhow::anyhow!("File does not exist: {}", file_path.display()));
        }

        let content = std::fs::read_to_string(file_path)?;
        let file_path_str = file_path.to_string_lossy().to_string();

        results.total_files_checked = 1;

        // Run per-file rules
        for rule in &self.rules {
            let violations = rule.check_file(&file_path_str, &content, &self.config);
            for violation in violations {
                results.add_violation(violation);
            }
        }

        results.execution_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(results)
    }

    /// Collect all source files in a directory
    fn collect_source_files(&self, dir_path: &Path) -> Result<HashMap<String, String>, anyhow::Error> {
        let mut files = HashMap::new();

        for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();

            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == "rs" {
                        let path_str = path.to_string_lossy().to_string();

                        // Check if file should be excluded
                        let mut should_exclude = false;
                        for dir in &self.config.exclude_dirs {
                            if path_str.starts_with(dir) {
                                should_exclude = true;
                                break;
                            }
                        }

                        if !should_exclude {
                            for pattern in &self.config.exclude_patterns {
                                if path_str.contains(pattern) {
                                    should_exclude = true;
                                    break;
                                }
                            }
                        }

                        if !should_exclude {
                            match std::fs::read_to_string(path) {
                                Ok(content) => {
                                    files.insert(path_str, content);
                                }
                                Err(e) => {
                                    tracing::warn!("Failed to read file {}: {}", path.display(), e);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(files)
    }
}

impl Default for QualityGateRunner {
    fn default() -> Self {
        Self::new()
    }
}

/// Run quality gates with default configuration
pub async fn run_quality_gates(dir_path: &Path) -> Result<QualityGateResults, anyhow::Error> {
    let runner = QualityGateRunner::new();
    runner.run_on_directory(dir_path).await
}

/// Run quality gates with custom configuration
pub async fn run_quality_gates_with_config(dir_path: &Path, config: QualityGateConfig) -> Result<QualityGateResults, anyhow::Error> {
    let runner = QualityGateRunner::with_config(config);
    runner.run_on_directory(dir_path).await
}
