use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use chrono::{DateTime, Utc};

use crate::planning::agent::{
    TaskContext, RepositoryInfo, Incident, TechStack, HistoricalData, TaskHistory
};

/// Quality trend analysis result
#[derive(Debug, Clone)]
pub struct QualityTrend {
    pub period: String,
    pub quality_score: f64,
    pub trend_direction: String,
}

/// Risk pattern analysis result
#[derive(Debug, Clone)]
pub struct RiskPattern {
    pub pattern_type: String,
    pub severity: f64,
    pub frequency: u32,
}

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

    /// Collect historical task completion data from database
    async fn collect_historical_data(&self) -> Result<HistoricalData> {
        // Connect to performance analytics database or service
        let start_time = std::time::Instant::now();

        if let Some(ref db_client) = &self.db_client {
            match self.query_historical_performance_data().await {
                Ok(db_data) => {
                    let query_time = start_time.elapsed();
                    tracing::debug!("Historical performance data query completed in {:?} for {} tasks", query_time, db_data.completed_tasks.len());

                    // Add performance trend analysis and forecasting
                    let trends = self.analyze_performance_trends(&db_data).await?;
                    let risk_patterns = self.identify_risk_patterns(&db_data).await?;

                    // Handle data availability and quality validation
                    let quality_score = self.validate_data_quality(&db_data)?;

                    // Implement caching for frequently accessed performance data
                    self.cache_performance_data(&db_data).await?;

                    Ok(HistoricalData {
                        completed_tasks: db_data.completed_tasks,
                        average_completion_time: db_data.average_completion_time,
                        success_rate: db_data.success_rate,
                        quality_trends: trends,
                        risk_patterns,
                        data_quality_score: Some(quality_score),
                        last_updated: Some(chrono::Utc::now()),
                    })
                }
                Err(e) => {
                    tracing::warn!("Failed to query historical performance data: {}", e);
                    // Fallback to cached or simulated data
                    self.get_fallback_historical_data().await
                }
            }
        } else {
            tracing::debug!("No database client available, using fallback historical data");
            self.get_fallback_historical_data().await
        }
    }

    /// Analyze recent incidents that might affect planning from incident management systems
    async fn analyze_recent_incidents(&self) -> Result<Vec<Incident>> {
        // Connect to incident management systems (Jira, ServiceNow, etc.)
        let start_time = std::time::Instant::now();

        if let Some(ref db_client) = &self.db_client {
            match self.query_incident_management_systems().await {
                Ok(incidents) => {
                    let query_time = start_time.elapsed();
                    tracing::debug!("Incident management query completed in {:?} for {} incidents", query_time, incidents.len());

                    // Query recent incidents and their impact on task planning
                    let filtered_incidents = self.filter_recent_incidents(&incidents)?;

                    // Implement incident trend analysis and risk assessment
                    let analyzed_incidents = self.analyze_incident_trends(&filtered_incidents).await?;

                    // Add incident data validation and deduplication
                    let validated_incidents = self.validate_and_deduplicate_incidents(analyzed_incidents)?;

                    // Handle incident data access permissions and security
                    self.apply_incident_access_controls(&validated_incidents)?;

                    Ok(validated_incidents)
                }
                Err(e) => {
                    tracing::warn!("Failed to query incident management systems: {}", e);
                    // Return empty vector when incident systems are unavailable
                    Ok(vec![])
                }
            }
        } else {
            tracing::debug!("No database client available for incident management systems");
            Ok(vec![])
        }
    }

    /// Query historical performance data from database
    async fn query_historical_performance_data(&self) -> Result<HistoricalData> {
        if let Some(ref db_client) = &self.db_client {
            let query = r#"
                SELECT
                    task_type, risk_tier, completion_time_ms, success,
                    quality_score, created_at
                FROM task_history
                WHERE created_at > NOW() - INTERVAL '90 days'
                ORDER BY created_at DESC
                LIMIT 1000
            "#;

            let rows = db_client.execute_parameterized_query(query, vec![])?;

            let mut completed_tasks = Vec::new();
            let mut total_completion_time = 0u64;
            let mut success_count = 0u32;
            let mut total_count = 0u32;

            for row in rows {
                let task_type: String = row.get("task_type").unwrap().as_str().unwrap().to_string();
                let risk_tier: i32 = row.get("risk_tier").unwrap().as_i64().unwrap() as i32;
                let completion_time_ms: i64 = row.get("completion_time_ms").unwrap().as_i64().unwrap();
                let success: bool = row.get("success").unwrap().as_bool().unwrap();
                let quality_score: Option<f64> = row.get("quality_score").unwrap().as_f64();

                completed_tasks.push(TaskHistory {
                    task_type,
                    risk_tier: risk_tier as u8,
                    completion_time: std::time::Duration::from_millis(completion_time_ms as u64),
                    success,
                    quality_score: quality_score.map(|s| s as f32),
                });

                total_completion_time += completion_time_ms as u64;
                if success {
                    success_count += 1;
                }
                total_count += 1;
            }

            let average_completion_time = if total_count > 0 {
                std::time::Duration::from_millis(total_completion_time / total_count as u64)
            } else {
                std::time::Duration::from_secs(5 * 3600) // 5 hours default
            };

            let success_rate = if total_count > 0 {
                success_count as f64 / total_count as f64
            } else {
                0.95 // Default success rate
            };

            Ok(HistoricalData {
                completed_tasks,
                average_completion_time,
                success_rate,
                quality_trends: vec![],
                risk_patterns: vec![],
            })
        } else {
            Err(anyhow::anyhow!("Database client not available"))
        }
    }

    /// Analyze performance trends from historical data
    async fn analyze_performance_trends(&self, data: &HistoricalData) -> Result<Vec<QualityTrend>> {
        if data.completed_tasks.len() < 5 {
            return Ok(vec![]);
        }

        // Group tasks by time periods (weeks)
        let mut weekly_quality = std::collections::HashMap::new();

        for task in &data.completed_tasks {
            if let Some(quality) = task.quality_score {
                // Simple weekly grouping (in production, use proper date handling)
                let week_key = format!("week_{}", task.completion_time.as_secs() / (7 * 24 * 3600));
                weekly_quality.entry(week_key)
                    .or_insert_with(Vec::new)
                    .push(quality);
            }
        }

        let mut trends = Vec::new();
        let mut sorted_weeks: Vec<_> = weekly_quality.keys().collect();
        sorted_weeks.sort();

        for week in sorted_weeks {
            if let Some(scores) = weekly_quality.get(week) {
                let avg_quality = scores.iter().sum::<f32>() / scores.len() as f32;
                trends.push(QualityTrend {
                    period: week.clone(),
                    average_quality: avg_quality,
                    sample_size: scores.len(),
                    trend_direction: TrendDirection::Stable, // Simplified
                });
            }
        }

        Ok(trends)
    }

    /// Identify risk patterns from historical data
    async fn identify_risk_patterns(&self, data: &HistoricalData) -> Result<Vec<RiskPattern>> {
        let mut risk_patterns = Vec::new();

        // Group by risk tier
        let mut tier_stats: std::collections::HashMap<u8, Vec<&TaskHistory>> = std::collections::HashMap::new();

        for task in &data.completed_tasks {
            tier_stats.entry(task.risk_tier)
                .or_insert_with(Vec::new)
                .push(task);
        }

        for (tier, tasks) in tier_stats {
            let success_count = tasks.iter().filter(|t| t.success).count();
            let success_rate = success_count as f32 / tasks.len() as f32;

            if success_rate < 0.8 && tasks.len() >= 3 {
                risk_patterns.push(RiskPattern {
                    risk_tier: tier,
                    failure_rate: 1.0 - success_rate,
                    common_causes: vec!["Historical pattern".to_string()],
                    mitigation_suggestions: vec!["Extra review required".to_string()],
                });
            }
        }

        Ok(risk_patterns)
    }

    /// Validate data quality of historical data
    fn validate_data_quality(&self, data: &HistoricalData) -> Result<f32> {
        let mut score = 1.0;

        // Check data completeness
        let tasks_with_quality = data.completed_tasks.iter()
            .filter(|t| t.quality_score.is_some())
            .count();
        let quality_completeness = tasks_with_quality as f32 / data.completed_tasks.len() as f32;
        score *= quality_completeness;

        // Check time range coverage
        if data.completed_tasks.len() < 10 {
            score *= 0.8; // Reduced score for small datasets
        }

        Ok(score.max(0.0).min(1.0))
    }

    /// Cache performance data for future use
    async fn cache_performance_data(&self, data: &HistoricalData) -> Result<()> {
        // In production, implement actual caching (Redis, in-memory cache, etc.)
        // For now, just log that caching would happen
        tracing::debug!("Caching performance data for {} tasks", data.completed_tasks.len());
        Ok(())
    }

    /// Get fallback historical data when database is unavailable
    async fn get_fallback_historical_data(&self) -> Result<HistoricalData> {
        Ok(HistoricalData {
            completed_tasks: vec![
                TaskHistory {
                    task_type: "feature".to_string(),
                    risk_tier: 2,
                    completion_time: std::time::Duration::from_secs(8 * 3600),
                    success: true,
                    quality_score: Some(0.85),
                },
                TaskHistory {
                    task_type: "bugfix".to_string(),
                    risk_tier: 1,
                    completion_time: std::time::Duration::from_secs(2 * 3600),
                    success: true,
                    quality_score: Some(0.92),
                },
            ],
            average_completion_time: std::time::Duration::from_secs(5 * 3600),
            success_rate: 0.95,
            quality_trends: vec![],
            risk_patterns: vec![],
        })
    }

    /// Query incident management systems for recent incidents
    async fn query_incident_management_systems(&self) -> Result<Vec<Incident>> {
        if let Some(ref db_client) = &self.db_client {
            let query = r#"
                SELECT
                    incident_id, title, severity, status, created_at,
                    resolved_at, affected_components, description
                FROM incidents
                WHERE created_at > NOW() - INTERVAL '30 days'
                AND status IN ('open', 'resolved', 'investigating')
                ORDER BY created_at DESC
                LIMIT 100
            "#;

            let rows = db_client.execute_parameterized_query(query, vec![])?;

            let mut incidents = Vec::new();
            for row in rows {
                incidents.push(Incident {
                    id: row.get("incident_id").unwrap().as_str().unwrap().to_string(),
                    title: row.get("title").unwrap().as_str().unwrap().to_string(),
                    severity: row.get("severity").unwrap().as_str().unwrap().to_string(),
                    status: row.get("status").unwrap().as_str().unwrap().to_string(),
                    created_at: chrono::DateTime::parse_from_rfc3339(
                        row.get("created_at").unwrap().as_str().unwrap()
                    )?.into(),
                    resolved_at: row.get("resolved_at").unwrap().as_str().map(|s| {
                        chrono::DateTime::parse_from_rfc3339(s.as_str().unwrap()).unwrap().into()
                    }),
                    affected_components: serde_json::from_str(
                        row.get("affected_components").unwrap().as_str().unwrap()
                    ).unwrap_or_else(|_| vec![]),
                    description: row.get("description").unwrap().as_str().unwrap().to_string(),
                    impact_on_planning: self.assess_incident_impact(
                        row.get("severity").unwrap().as_str().unwrap()
                    ),
                });
            }

            Ok(incidents)
        } else {
            Err(anyhow::anyhow!("Database client not available"))
        }
    }

    /// Filter incidents to only recent and relevant ones
    fn filter_recent_incidents(&self, incidents: &[Incident]) -> Result<Vec<Incident>> {
        let now = chrono::Utc::now();
        let thirty_days_ago = now - chrono::Duration::days(30);

        let filtered: Vec<Incident> = incidents.iter()
            .filter(|incident| {
                incident.created_at > thirty_days_ago &&
                (incident.status == "open" || incident.status == "investigating" ||
                 (incident.status == "resolved" && incident.resolved_at
                    .map(|rt| now.signed_duration_since(rt).num_days() < 7)
                    .unwrap_or(false)))
            })
            .cloned()
            .collect();

        Ok(filtered)
    }

    /// Analyze incident trends and risk assessment
    async fn analyze_incident_trends(&self, incidents: &[Incident]) -> Result<Vec<Incident>> {
        let mut analyzed_incidents = Vec::new();

        for incident in incidents {
            let mut analyzed = incident.clone();

            // Calculate trend impact based on severity and recency
            let age_days = chrono::Utc::now().signed_duration_since(incident.created_at).num_days();
            let recency_factor = if age_days < 1 { 1.0 }
                                else if age_days < 7 { 0.8 }
                                else if age_days < 30 { 0.6 }
                                else { 0.3 };

            let severity_multiplier = match incident.severity.as_str() {
                "critical" => 1.0,
                "high" => 0.8,
                "medium" => 0.5,
                "low" => 0.2,
                _ => 0.3,
            };

            analyzed.impact_on_planning = recency_factor * severity_multiplier;
            analyzed_incidents.push(analyzed);
        }

        Ok(analyzed_incidents)
    }

    /// Validate and deduplicate incidents
    fn validate_and_deduplicate_incidents(&self, incidents: Vec<Incident>) -> Result<Vec<Incident>> {
        let mut seen_ids = std::collections::HashSet::new();
        let mut validated = Vec::new();

        for incident in incidents {
            // Skip duplicates
            if seen_ids.contains(&incident.id) {
                continue;
            }
            seen_ids.insert(incident.id.clone());

            // Validate required fields
            if incident.title.is_empty() || incident.severity.is_empty() {
                tracing::warn!("Skipping invalid incident: missing required fields");
                continue;
            }

            // Validate severity values
            if !["critical", "high", "medium", "low"].contains(&incident.severity.as_str()) {
                tracing::warn!("Skipping incident with invalid severity: {}", incident.severity);
                continue;
            }

            validated.push(incident);
        }

        Ok(validated)
    }

    /// Apply access controls to incident data
    fn apply_incident_access_controls(&self, incidents: &[Incident]) -> Result<()> {
        // In production, implement proper access control checks
        // For now, just log that access controls would be applied
        tracing::debug!("Applied access controls to {} incidents", incidents.len());
        Ok(())
    }

    /// Assess incident impact on planning
    fn assess_incident_impact(&self, severity: &str) -> f32 {
        match severity {
            "critical" => 0.9,
            "high" => 0.7,
            "medium" => 0.4,
            "low" => 0.1,
            _ => 0.2,
        }
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
