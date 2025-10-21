use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use chrono::{DateTime, Utc};

use crate::planning::agent::{
    TaskContext, RepositoryInfo, Incident, TechStack, HistoricalData, TaskHistory
};

/// Context builder configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContextBuilderConfig {
    /// Enable repository analysis
    pub enable_repo_analysis: bool,
    /// Enable historical data collection
    pub enable_historical_data: bool,
    /// Maximum repository size to analyze (KB)
    pub max_repo_size_kb: u64,
    /// Lookback period for historical data (days)
    pub historical_lookback_days: u32,
    /// Enable incident analysis
    pub enable_incident_analysis: bool,
}

/// Builds enriched context for task planning
pub struct ContextBuilder {
    config: ContextBuilderConfig,
}

impl ContextBuilder {
    pub fn new(config: ContextBuilderConfig) -> Self {
        Self { config }
    }

    /// Enrich the base task context with additional information
    pub async fn enrich_context(&self, base_context: TaskContext) -> Result<TaskContext> {
        tracing::info!("Enriching task context for repository: {}", base_context.repo_info.name);

        let mut enriched = base_context;

        // Analyze repository if enabled
        if self.config.enable_repo_analysis {
            enriched.repo_info = self.analyze_repository(&enriched.repo_info).await?;
        }

        // Build tech stack information
        enriched.tech_stack = self.build_tech_stack(&enriched.repo_info).await?;

        // Collect historical data if enabled
        if self.config.enable_historical_data {
            enriched.historical_data = self.collect_historical_data().await?;
        }

        // Analyze recent incidents if enabled
        if self.config.enable_incident_analysis {
            enriched.recent_incidents = self.analyze_recent_incidents().await?;
        }

        Ok(enriched)
    }

    /// Analyze repository structure and metadata
    async fn analyze_repository(&self, base_info: &RepositoryInfo) -> Result<RepositoryInfo> {
        // Check if repository exists and get metadata
        let repo_path = Path::new(&base_info.name);

        if !repo_path.exists() {
            return Ok(base_info.clone());
        }

        let metadata = fs::metadata(&repo_path).await?;
        let size_kb = metadata.len() / 1024;

        // Skip analysis if repository is too large
        if size_kb > self.config.max_repo_size_kb {
            tracing::warn!("Repository {} is too large ({}KB > {}KB), skipping detailed analysis",
                base_info.name, size_kb, self.config.max_repo_size_kb);
            return Ok(RepositoryInfo {
                size_kb,
                ..base_info.clone()
            });
        }

        // Analyze directory structure for language detection
        let primary_language = self.detect_primary_language(&repo_path).await?;

        // Get contributors from git if available
        let contributors = self.get_contributors(&repo_path).await?;

        Ok(RepositoryInfo {
            size_kb,
            primary_language,
            contributors,
            ..base_info.clone()
        })
    }

    /// Detect primary programming language from repository contents
    async fn detect_primary_language(&self, repo_path: &Path) -> Result<String> {
        let mut language_counts = HashMap::new();

        // Walk through files and count by extension
        let mut stack = vec![repo_path.to_path_buf()];
        while let Some(path) = stack.pop() {
            if path.is_dir() {
                // Skip common non-code directories
                let file_name = path.file_name().unwrap_or_default().to_string_lossy();
                if file_name.starts_with('.') ||
                   file_name == "node_modules" ||
                   file_name == "target" ||
                   file_name == "build" ||
                   file_name == "dist" {
                    continue;
                }

                // Read directory contents
                if let Ok(mut entries) = fs::read_dir(&path).await {
                    while let Ok(Some(entry)) = entries.next_entry().await {
                        stack.push(entry.path());
                    }
                }
            } else if path.is_file() {
                if let Some(extension) = path.extension() {
                    let ext_str = extension.to_string_lossy().to_lowercase();
                    let language = self.extension_to_language(&ext_str);
                    *language_counts.entry(language).or_insert(0) += 1;
                }
            }
        }

        // Find the most common language
        let primary_language = language_counts.into_iter()
            .max_by_key(|&(_, count)| count)
            .map(|(lang, _)| lang)
            .unwrap_or_else(|| "Unknown".to_string());

        Ok(primary_language)
    }

    /// Map file extension to programming language
    fn extension_to_language(&self, extension: &str) -> String {
        match extension {
            "rs" => "Rust",
            "js" | "mjs" | "cjs" => "JavaScript",
            "ts" | "tsx" | "mts" | "cts" => "TypeScript",
            "py" => "Python",
            "java" => "Java",
            "go" => "Go",
            "cpp" | "cc" | "cxx" => "C++",
            "c" => "C",
            "php" => "PHP",
            "rb" => "Ruby",
            "swift" => "Swift",
            "kt" => "Kotlin",
            "scala" => "Scala",
            "cs" => "C#",
            "fs" | "fsx" => "F#",
            "ml" => "OCaml",
            "hs" => "Haskell",
            "clj" => "Clojure",
            "elm" => "Elm",
            "dart" => "Dart",
            "ex" | "exs" => "Elixir",
            "sql" => "SQL",
            "yaml" | "yml" => "YAML",
            "json" => "JSON",
            "xml" => "XML",
            "html" => "HTML",
            "css" => "CSS",
            "scss" | "sass" => "SCSS",
            "less" => "Less",
            "md" => "Markdown",
            "toml" => "TOML",
            "ini" => "INI",
            "sh" | "bash" => "Shell",
            "dockerfile" => "Dockerfile",
            _ => "Other",
        }.to_string()
    }

    /// Get contributors from git history
    async fn get_contributors(&self, repo_path: &Path) -> Result<Vec<String>> {
        // Try to run git command to get contributors
        let git_path = repo_path.join(".git");
        if !git_path.exists() {
            return Ok(vec![]);
        }

        // Use tokio::process::Command to run git shortlog
        let output = tokio::process::Command::new("git")
            .current_dir(repo_path)
            .args(&["shortlog", "-sn", "--no-merges", "-n", "10"])
            .output()
            .await;

        match output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let contributors: Vec<String> = stdout
                    .lines()
                    .filter_map(|line| {
                        line.trim()
                            .split('\t')
                            .nth(1)
                            .map(|name| name.trim().to_string())
                    })
                    .collect();
                Ok(contributors)
            }
            _ => {
                // Git command failed, return empty list
                Ok(vec![])
            }
        }
    }

    /// Build technology stack information from repository analysis
    async fn build_tech_stack(&self, repo_info: &RepositoryInfo) -> Result<TechStack> {
        let mut languages = vec![repo_info.primary_language.clone()];
        let mut frameworks = Vec::new();
        let mut databases = Vec::new();
        let mut deployment = Vec::new();

        // Analyze common framework indicators
        if let Ok(package_json) = fs::read_to_string("package.json").await {
            if package_json.contains("\"react\"") {
                frameworks.push("React".to_string());
                languages.push("JavaScript".to_string());
            }
            if package_json.contains("\"vue\"") {
                frameworks.push("Vue.js".to_string());
                languages.push("JavaScript".to_string());
            }
            if package_json.contains("\"angular\"") {
                frameworks.push("Angular".to_string());
                languages.push("TypeScript".to_string());
            }
            if package_json.contains("\"express\"") {
                frameworks.push("Express.js".to_string());
            }
            if package_json.contains("\"next\"") {
                frameworks.push("Next.js".to_string());
            }
        }

        if let Ok(cargo_toml) = fs::read_to_string("Cargo.toml").await {
            if cargo_toml.contains("axum") {
                frameworks.push("Axum".to_string());
            }
            if cargo_toml.contains("tokio") {
                frameworks.push("Tokio".to_string());
            }
            if cargo_toml.contains("diesel") {
                databases.push("Diesel".to_string());
            }
            if cargo_toml.contains("sqlx") {
                databases.push("SQLx".to_string());
            }
        }

        // Check for Docker
        if Path::new("Dockerfile").exists() || Path::new("docker-compose.yml").exists() {
            deployment.push("Docker".to_string());
        }

        // Check for Kubernetes
        if Path::new("k8s").exists() ||
           Path::new("kubernetes").exists() ||
           fs::read_dir(".").await
               .map(|mut entries| async move {
                   while let Ok(Some(entry)) = entries.next_entry().await {
                       let name = entry.file_name().to_string_lossy();
                       if name.contains("k8s") || name.contains("kube") {
                           return true;
                       }
                   }
                   false
               }).await.unwrap_or(false) {
            deployment.push("Kubernetes".to_string());
        }

        // Deduplicate and sort
        languages.sort();
        languages.dedup();
        frameworks.sort();
        frameworks.dedup();
        databases.sort();
        databases.dedup();
        deployment.sort();
        deployment.dedup();

        Ok(TechStack {
            languages,
            frameworks,
            databases,
            deployment,
        })
    }

    /// Collect historical task completion data
    async fn collect_historical_data(&self) -> Result<HistoricalData> {
        // In a real implementation, this would query a database or analytics service
        // For now, return placeholder data

        let completed_tasks = vec![
            TaskHistory {
                task_type: "feature".to_string(),
                risk_tier: 2,
                completion_time: std::time::Duration::from_secs(8 * 3600), // 8 hours
                success: true,
                quality_score: Some(0.85),
            },
            TaskHistory {
                task_type: "bugfix".to_string(),
                risk_tier: 3,
                completion_time: std::time::Duration::from_secs(2 * 3600), // 2 hours
                success: true,
                quality_score: Some(0.92),
            },
        ];

        let average_completion_time = std::time::Duration::from_secs(5 * 3600); // 5 hours average
        let success_rate = 0.95; // 95% success rate

        Ok(HistoricalData {
            completed_tasks,
            average_completion_time,
            success_rate,
        })
    }

    /// Analyze recent incidents that might affect planning
    async fn analyze_recent_incidents(&self) -> Result<Vec<Incident>> {
        // In a real implementation, this would query incident management systems
        // For now, return empty list (no recent incidents)

        Ok(vec![])
    }
}

pub type Result<T> = std::result::Result<T, ContextBuilderError>;

#[derive(Debug, thiserror::Error)]
pub enum ContextBuilderError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("UTF-8 decoding error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),

    #[error("Repository analysis failed: {0}")]
    AnalysisError(String),

    #[error("Historical data collection failed: {0}")]
    HistoricalDataError(String),
}
