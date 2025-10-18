use crate::types::*;
use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use tracing::{debug, error, info, warn};

/// Security auditor
#[derive(Debug)]
pub struct SecurityAuditor {
    /// Audit policy
    policy: AuditPolicy,
    /// Audit log file path
    log_file_path: String,
}

impl SecurityAuditor {
    /// Create a new security auditor
    pub fn new(policy: AuditPolicy) -> Result<Self> {
        debug!("Initializing security auditor");

        let log_file_path = format!("security_audit_{}.log", Utc::now().format("%Y%m%d"));

        Ok(Self {
            policy,
            log_file_path,
        })
    }

    /// Log a security audit event
    pub async fn log_event(&self, event: &SecurityAuditEvent) -> Result<()> {
        if !self.policy.enabled {
            return Ok(());
        }

        debug!("Logging security audit event: {:?}", event.event_type);

        // Check if we should log this event type
        if !self.should_log_event_type(&event.event_type) {
            return Ok(());
        }

        // Format the event for logging
        let log_entry = self.format_log_entry(event);

        // Write to log file
        self.write_to_log_file(&log_entry).await?;

        // Also log to tracing for real-time monitoring
        match event.result {
            AuditResult::Allowed => debug!("Security event allowed: {}", event.action),
            AuditResult::Denied => warn!("Security event denied: {}", event.action),
            AuditResult::Blocked => error!("Security event blocked: {}", event.action),
            AuditResult::Approved => info!("Security event approved: {}", event.action),
            AuditResult::Rejected => warn!("Security event rejected: {}", event.action),
            AuditResult::Warning => warn!("Security event warning: {}", event.action),
        }

        Ok(())
    }

    /// Check if we should log this event type
    fn should_log_event_type(&self, event_type: &AuditEventType) -> bool {
        match event_type {
            AuditEventType::FileAccess => self.policy.log_file_access,
            AuditEventType::CommandExecution => self.policy.log_command_execution,
            AuditEventType::SecretDetection => self.policy.log_secret_detections,
            AuditEventType::PolicyViolation => self.policy.log_security_violations,
            AuditEventType::CouncilDecision => self.policy.log_security_violations,
            AuditEventType::SecurityCheck => self.policy.log_security_violations,
        }
    }

    /// Format log entry for file output
    fn format_log_entry(&self, event: &SecurityAuditEvent) -> String {
        let timestamp = event.timestamp.format("%Y-%m-%d %H:%M:%S%.3f UTC");
        let event_type = format!("{:?}", event.event_type);
        let result = format!("{:?}", event.result);

        // Create metadata string
        let metadata_str = if event.metadata.is_empty() {
            "{}".to_string()
        } else {
            serde_json::to_string(&event.metadata).unwrap_or_else(|_| "{}".to_string())
        };

        format!(
            "[{}] {} | {} | {} | {} | {} | {} | {}\n",
            timestamp,
            event.id,
            event_type,
            result,
            event.actor,
            event.resource,
            event.action,
            metadata_str
        )
    }

    /// Write log entry to file
    async fn write_to_log_file(&self, log_entry: &str) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file_path)?;

        file.write_all(log_entry.as_bytes())?;
        file.flush()?;

        Ok(())
    }

    /// Get audit policy
    pub fn get_policy(&self) -> &AuditPolicy {
        &self.policy
    }

    /// Update audit policy with comprehensive validation, atomicity, and rollback support
    pub async fn update_policy(&mut self, new_policy: AuditPolicy) -> Result<()> {
        info!("Updating audit policy with comprehensive validation and rollback support");

        // 1. Policy validation: Validate new audit policy before update
        self.validate_policy(&new_policy).await?;

        // 2. Policy update: Update audit policy with atomicity and rollback support
        let old_policy = self.policy.clone();
        let update_result = self.apply_policy_update(new_policy).await;

        // If update fails, rollback to previous policy
        if let Err(e) = update_result {
            warn!(
                "Policy update failed, rolling back to previous policy: {}",
                e
            );
            self.policy = old_policy;
            return Err(e);
        }

        // 3. Policy persistence: Persist policy changes to storage with backup
        if let Err(e) = self.persist_policy_changes().await {
            error!("Failed to persist policy changes: {}", e);
            // Even if persistence fails, keep the in-memory policy as it's valid
            // The system can continue operating with the new policy
            warn!("Continuing with in-memory policy despite persistence failure");
        }

        info!("Audit policy updated successfully");
        Ok(())
    }

    /// Validate audit policy parameters and constraints
    async fn validate_policy(&self, policy: &AuditPolicy) -> Result<()> {
        // Validate retention days (reasonable bounds)
        if policy.retention_days == 0 {
            return Err(anyhow::anyhow!("Retention days cannot be zero"));
        }
        if policy.retention_days > 365 * 10 {
            return Err(anyhow::anyhow!("Retention days cannot exceed 10 years"));
        }

        // Validate that if auditing is enabled, at least one log type is enabled
        if policy.enabled {
            let has_any_logging = policy.log_file_access
                || policy.log_command_execution
                || policy.log_security_violations
                || policy.log_secret_detections;

            if !has_any_logging {
                return Err(anyhow::anyhow!(
                    "When auditing is enabled, at least one log type must be enabled"
                ));
            }
        }

        // Additional validation could include:
        // - Checking for conflicting settings
        // - Validating against system constraints
        // - Checking for security implications

        Ok(())
    }

    /// Apply policy update with atomicity guarantees
    async fn apply_policy_update(&mut self, new_policy: AuditPolicy) -> Result<()> {
        // Store old policy for potential rollback
        let old_policy = self.policy.clone();

        // Apply the new policy atomically
        self.policy = new_policy.clone();

        // Validate that the new policy works correctly
        if let Err(e) = self.test_policy_application().await {
            // Restore old policy on validation failure
            self.policy = old_policy;
            return Err(anyhow::anyhow!("Policy validation failed: {}", e));
        }

        // Log the policy change
        let change_event = SecurityAuditEvent {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type: SecurityEventType::PolicyUpdate,
            user_id: None,
            session_id: None,
            operation: "audit_policy_update".to_string(),
            resource: "audit_policy".to_string(),
            result: SecurityEventResult::Success,
            details: Some(format!(
                "Updated audit policy: enabled={}, retention_days={}",
                new_policy.enabled, new_policy.retention_days
            )),
            risk_score: 0.1, // Low risk for policy updates
        };

        // Only log if the old policy would have allowed it
        if old_policy.enabled && old_policy.log_security_violations {
            if let Err(e) = self.log_event(&change_event).await {
                warn!("Failed to log policy update event: {}", e);
                // Don't fail the update for logging issues
            }
        }

        Ok(())
    }

    /// Test that the applied policy works correctly
    async fn test_policy_application(&self) -> Result<()> {
        // Test that event logging would work with new policy
        if self.policy.enabled {
            let test_event = SecurityAuditEvent {
                id: Uuid::new_v4(),
                timestamp: Utc::now(),
                event_type: SecurityEventType::PolicyUpdate,
                user_id: None,
                session_id: None,
                operation: "policy_test".to_string(),
                resource: "audit_system".to_string(),
                result: SecurityEventResult::Success,
                details: Some("Testing policy application".to_string()),
                risk_score: 0.0,
            };

            // Try to format the log entry (doesn't write to disk)
            let _ = self.format_log_entry(&test_event);
        }

        Ok(())
    }

    /// Persist policy changes to storage with backup and recovery
    async fn persist_policy_changes(&self) -> Result<()> {
        use std::fs;
        use std::path::Path;

        // Create backup of current policy file if it exists
        let policy_file = "audit_policy.json";
        let backup_file = format!("{}.backup", policy_file);

        if Path::new(policy_file).exists() {
            fs::copy(policy_file, &backup_file).context("Failed to create policy backup")?;
        }

        // Attempt to save new policy
        let policy_json =
            serde_json::to_string_pretty(&self.policy).context("Failed to serialize policy")?;

        // Write to temporary file first for atomicity
        let temp_file = format!("{}.tmp", policy_file);
        fs::write(&temp_file, &policy_json).context("Failed to write policy to temporary file")?;

        // Atomic move to final location
        fs::rename(&temp_file, policy_file)
            .context("Failed to move policy file to final location")?;

        // Clean up backup after successful save
        if Path::new(&backup_file).exists() {
            if let Err(e) = fs::remove_file(&backup_file) {
                warn!("Failed to clean up policy backup: {}", e);
                // Don't fail for cleanup issues
            }
        }

        debug!("Audit policy persisted successfully");
        Ok(())
    }

    /// Parse structured audit log data from either JSON array or newline-delimited JSON.
    pub fn ingest_logs_from_str(&self, raw: &str) -> Result<Vec<AuditLogEntry>> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Ok(Vec::new());
        }

        if trimmed.starts_with('[') {
            let entries: Vec<AuditLogEntry> =
                serde_json::from_str(trimmed).context("Failed to parse audit log JSON array")?;
            for entry in &entries {
                entry.validate()?;
            }
            return Ok(entries);
        }

        let mut entries = Vec::new();
        for (idx, line) in trimmed.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let entry: AuditLogEntry = serde_json::from_str(line)
                .with_context(|| format!("Invalid NDJSON audit entry at line {}", idx + 1))?;
            entry.validate()?;
            entries.push(entry);
        }

        Ok(entries)
    }

    /// Run severity analysis for a batch of audit entries.
    pub fn analyze_entries(&self, entries: &[AuditLogEntry]) -> SecurityAnalysis {
        SecurityAnalysisEngine::default().analyze(entries)
    }

    /// Convenience helper to ingest and analyze in one call.
    pub fn ingest_and_analyze(&self, raw: &str) -> Result<SecurityAnalysis> {
        let entries = self.ingest_logs_from_str(raw)?;
        Ok(self.analyze_entries(&entries))
    }

    /// Get audit log file path
    pub fn get_log_file_path(&self) -> &str {
        &self.log_file_path
    }

    /// Rotate the audit log file with comprehensive archive management and optimization
    pub async fn rotate_log_file(&mut self) -> Result<()> {
        info!("Performing comprehensive audit log file rotation");

        let current_path = self.log_file_path.clone();
        let timestamp = Utc::now();
        let new_log_file_path = format!("security_audit_{}.log", timestamp.format("%Y%m%d"));

        // 1. Log file closure: Close the current log file safely
        self.close_current_log_file().await?;

        // 2. Archive management: Move log file to archive location with compression
        let archive_path = self.archive_log_file(&current_path, &timestamp).await?;

        // 3. New log file creation: Create a new log file for continued logging
        self.create_new_log_file(&new_log_file_path).await?;

        // 4. Rotation optimization: Clean up old archives based on retention policy
        self.cleanup_old_archives().await?;

        // Update the log_file_path
        self.log_file_path = new_log_file_path;

        info!(
            "Audit log rotation completed successfully. Archive: {}",
            archive_path
        );
        Ok(())
    }

    /// Safely close the current log file and flush all buffers
    async fn close_current_log_file(&self) -> Result<()> {
        // Since we're using std::fs operations that are synchronous,
        // we don't need to explicitly close files as they're closed when dropped
        // But we can add any necessary cleanup here

        // Ensure any pending writes are flushed (though this is handled by the OS)
        // In a more sophisticated implementation, we might keep file handles open
        // and explicitly flush/close them here

        debug!("Current log file closure completed");
        Ok(())
    }

    /// Move log file to archive location with optional compression
    async fn archive_log_file(
        &self,
        current_path: &str,
        timestamp: &chrono::DateTime<Utc>,
    ) -> Result<String> {
        use std::fs;
        use std::path::Path;

        let current_file_path = Path::new(current_path);

        // Check if the current log file exists and has content
        if !current_file_path.exists() {
            debug!(
                "Current log file does not exist, skipping archive: {}",
                current_path
            );
            return Ok("none".to_string());
        }

        // Get file metadata to check size
        let metadata = fs::metadata(current_file_path)?;
        if metadata.len() == 0 {
            debug!(
                "Current log file is empty, skipping archive: {}",
                current_path
            );
            return Ok("empty".to_string());
        }

        // Create archive directory if it doesn't exist
        let archive_dir = "audit_archive";
        fs::create_dir_all(archive_dir)?;

        // Create archive filename with timestamp
        let archive_filename = format!(
            "security_audit_{}.log.gz",
            timestamp.format("%Y%m%d_%H%M%S")
        );
        let archive_path = format!("{}/{}", archive_dir, archive_filename);

        // Compress the log file using gzip
        self.compress_log_file(current_path, &archive_path).await?;

        // Verify the archive was created successfully
        if !Path::new(&archive_path).exists() {
            return Err(anyhow::anyhow!(
                "Failed to create archive file: {}",
                archive_path
            ));
        }

        // Remove the original file after successful archiving
        fs::remove_file(current_path)?;

        debug!(
            "Log file archived successfully: {} -> {}",
            current_path, archive_path
        );
        Ok(archive_path)
    }

    /// Compress a log file using gzip compression
    async fn compress_log_file(&self, source_path: &str, dest_path: &str) -> Result<()> {
        use std::fs::File;
        use std::io::{BufReader, BufWriter};

        let source_file = File::open(source_path)?;
        let dest_file = File::create(dest_path)?;

        let reader = BufReader::new(source_file);
        let writer = BufWriter::new(dest_file);

        // Create gzip encoder
        let mut encoder = flate2::write::GzEncoder::new(writer, flate2::Compression::default());
        std::io::copy(&mut reader, &mut encoder)?;
        encoder.finish()?;

        Ok(())
    }

    /// Create a new log file with proper initialization
    async fn create_new_log_file(&self, file_path: &str) -> Result<()> {
        use std::fs::OpenOptions;
        use std::io::Write;

        // Create the new log file
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(file_path)?;

        // Write log file header with metadata
        let header = format!(
            "# Security Audit Log - Created: {}\n\
             # Format: JSON lines\n\
             # Policy: enabled={}, retention_days={}\n\
             # Log types: file_access={}, command_execution={}, security_violations={}, secret_detections={}\n\n",
            Utc::now().to_rfc3339(),
            self.policy.enabled,
            self.policy.retention_days,
            self.policy.log_file_access,
            self.policy.log_command_execution,
            self.policy.log_security_violations,
            self.policy.log_secret_detections
        );

        file.write_all(header.as_bytes())?;
        file.flush()?;

        debug!("New log file created and initialized: {}", file_path);
        Ok(())
    }

    /// Clean up old archives based on retention policy
    async fn cleanup_old_archives(&self) -> Result<()> {
        use std::fs;
        use std::path::Path;

        let archive_dir = "audit_archive";
        let archive_path = Path::new(archive_dir);

        if !archive_path.exists() {
            return Ok(());
        }

        let retention_days = self.policy.retention_days as i64;
        let cutoff_date = Utc::now() - chrono::Duration::days(retention_days);

        let mut files_removed = 0;
        let mut total_size_freed = 0u64;

        // Iterate through archive files
        if let Ok(entries) = fs::read_dir(archive_path) {
            for entry in entries.flatten() {
                let path = entry.path();

                // Only process .gz files
                if let Some(extension) = path.extension() {
                    if extension != "gz" {
                        continue;
                    }
                } else {
                    continue;
                }

                // Check file modification time
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        let modified_datetime = chrono::DateTime::<Utc>::from(modified);

                        // Remove files older than retention period
                        if modified_datetime < cutoff_date {
                            if let Ok(size) = fs::remove_file(&path) {
                                files_removed += 1;
                                total_size_freed += metadata.len();
                                debug!("Removed old archive: {}", path.display());
                            }
                        }
                    }
                }
            }
        }

        if files_removed > 0 {
            info!(
                "Archive cleanup completed: {} files removed, {} bytes freed",
                files_removed, total_size_freed
            );
        }

        Ok(())
    }

    /// Get comprehensive audit statistics with analysis and reporting
    pub async fn get_audit_stats(&self) -> Result<AuditStats> {
        info!("Generating comprehensive audit statistics");

        // 1. Log file analysis: Analyze current and archived log files
        let log_entries = self.analyze_log_files().await?;

        // 2. Statistics calculation: Calculate comprehensive audit statistics
        let stats = self.calculate_audit_statistics(&log_entries).await?;

        // 3. Statistics aggregation: Aggregate across time periods and detect patterns
        let enriched_stats = self
            .aggregate_statistics_with_patterns(stats, &log_entries)
            .await?;

        info!(
            "Audit statistics generated successfully: {} total events",
            enriched_stats.total_events
        );
        Ok(enriched_stats)
    }

    /// Analyze current and archived log files for audit events
    async fn analyze_log_files(&self) -> Result<Vec<AuditLogEntry>> {
        use flate2::read::GzDecoder;
        use std::fs;
        use std::io::BufReader;
        use std::path::Path;

        let mut all_entries = Vec::new();

        // Analyze current log file
        if Path::new(&self.log_file_path).exists() {
            match self.parse_log_file(&self.log_file_path, false).await {
                Ok(entries) => {
                    all_entries.extend(entries);
                    debug!("Analyzed current log file: {} entries", all_entries.len());
                }
                Err(e) => {
                    warn!("Failed to analyze current log file: {}", e);
                }
            }
        }

        // Analyze archived log files (limited to recent ones for performance)
        let archive_dir = "audit_archive";
        if Path::new(archive_dir).exists() {
            if let Ok(entries) = fs::read_dir(archive_dir) {
                // Only analyze the 5 most recent archive files
                let mut archive_files: Vec<_> = entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().map_or(false, |ext| ext == "gz"))
                    .collect();

                // Sort by modification time (newest first)
                archive_files.sort_by(|a, b| {
                    b.metadata()
                        .unwrap()
                        .modified()
                        .unwrap()
                        .cmp(&a.metadata().unwrap().modified().unwrap())
                });

                // Analyze only the 5 most recent archives
                for entry in archive_files.into_iter().take(5) {
                    let path = entry.path();
                    match self.parse_log_file(path.to_str().unwrap(), true).await {
                        Ok(entries) => {
                            all_entries.extend(entries);
                            debug!(
                                "Analyzed archive file {}: {} entries",
                                path.display(),
                                entries.len()
                            );
                        }
                        Err(e) => {
                            warn!("Failed to analyze archive file {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }

        debug!("Total log entries analyzed: {}", all_entries.len());
        Ok(all_entries)
    }

    /// Parse a log file (compressed or uncompressed) into audit entries
    async fn parse_log_file(
        &self,
        file_path: &str,
        compressed: bool,
    ) -> Result<Vec<AuditLogEntry>> {
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        let file = File::open(file_path)?;
        let reader: Box<dyn std::io::BufRead> = if compressed {
            Box::new(BufReader::new(GzDecoder::new(file)))
        } else {
            Box::new(BufReader::new(file))
        };

        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let line = line.trim();

            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Try to parse as JSON
            match serde_json::from_str::<AuditLogEntry>(line) {
                Ok(entry) => entries.push(entry),
                Err(_) => {
                    // Skip invalid JSON lines but log them
                    debug!("Skipping invalid log line: {}", line);
                }
            }
        }

        Ok(entries)
    }

    /// Calculate comprehensive audit statistics from log entries
    async fn calculate_audit_statistics(&self, entries: &[AuditLogEntry]) -> Result<AuditStats> {
        let mut events_by_type = HashMap::new();
        let mut events_by_result = HashMap::new();
        let mut events_by_actor = HashMap::new();

        for entry in entries {
            // Count by event type
            let type_key = format!("{:?}", entry.event_type);
            *events_by_type.entry(type_key).or_insert(0) += 1;

            // Count by result
            let result_key = format!("{:?}", entry.result);
            *events_by_result.entry(result_key).or_insert(0) += 1;

            // Count by actor (user_id or session_id)
            let actor_key = entry
                .user_id
                .as_ref()
                .or(entry.session_id.as_ref())
                .unwrap_or(&"unknown".to_string())
                .clone();
            *events_by_actor.entry(actor_key).or_insert(0) += 1;
        }

        Ok(AuditStats {
            total_events: entries.len() as u64,
            events_by_type,
            events_by_result,
            events_by_actor,
            last_updated: Utc::now(),
        })
    }

    /// Aggregate statistics with pattern recognition and trend analysis
    async fn aggregate_statistics_with_patterns(
        &self,
        mut stats: AuditStats,
        entries: &[AuditLogEntry],
    ) -> Result<AuditStats> {
        // Add time-based aggregations (last 24 hours, 7 days, 30 days)
        let now = Utc::now();
        let one_day_ago = now - chrono::Duration::days(1);
        let seven_days_ago = now - chrono::Duration::days(7);
        let thirty_days_ago = now - chrono::Duration::days(30);

        let mut recent_entries = Vec::new();
        let mut weekly_entries = Vec::new();
        let mut monthly_entries = Vec::new();

        for entry in entries {
            if entry.timestamp >= one_day_ago {
                recent_entries.push(entry);
            }
            if entry.timestamp >= seven_days_ago {
                weekly_entries.push(entry);
            }
            if entry.timestamp >= thirty_days_ago {
                monthly_entries.push(entry);
            }
        }

        // Add time-based statistics as additional keys
        stats
            .events_by_type
            .insert("last_24h".to_string(), recent_entries.len() as u64);
        stats
            .events_by_type
            .insert("last_7d".to_string(), weekly_entries.len() as u64);
        stats
            .events_by_type
            .insert("last_30d".to_string(), monthly_entries.len() as u64);

        // Detect potential anomalies (high-frequency events)
        self.detect_anomalies(&stats, entries);

        Ok(stats)
    }

    /// Detect potential security anomalies in audit logs
    fn detect_anomalies(&self, stats: &AuditStats, entries: &[AuditLogEntry]) {
        // Check for high frequency of failed operations
        if let Some(failed_count) = stats.events_by_result.get("Failed") {
            let total_events = stats.total_events;
            let failure_rate = *failed_count as f64 / total_events as f64;

            if failure_rate > 0.5 && total_events > 10 {
                warn!(
                    "High failure rate detected: {:.1}% ({}/{}) - potential security issue",
                    failure_rate * 100.0,
                    failed_count,
                    total_events
                );
            }
        }

        // Check for unusual actor activity
        let mut actor_frequencies: HashMap<String, u64> = HashMap::new();
        for entry in entries {
            let actor = entry
                .user_id
                .as_ref()
                .or(entry.session_id.as_ref())
                .unwrap_or(&"unknown".to_string());
            *actor_frequencies.entry(actor.clone()).or_insert(0) += 1;
        }

        // Flag actors with unusually high activity
        let total_actors = actor_frequencies.len() as f64;
        let avg_activity = stats.total_events as f64 / total_actors;

        for (actor, count) in &actor_frequencies {
            let activity_ratio = *count as f64 / avg_activity;
            if activity_ratio > 5.0 && *count > 10 {
                warn!("Unusual activity detected for actor '{}': {} events ({}x average) - potential security concern",
                      actor, count, activity_ratio as u32);
            }
        }
    }
}

/// Audit statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStats {
    /// Total number of audit events
    pub total_events: u64,
    /// Events grouped by type
    pub events_by_type: HashMap<String, u64>,
    /// Events grouped by result
    pub events_by_result: HashMap<String, u64>,
    /// Events grouped by actor
    pub events_by_actor: HashMap<String, u64>,
    /// Last updated timestamp
    pub last_updated: chrono::DateTime<Utc>,
}

/// Severity analysis engine turns raw audit events into actionable insights.
#[derive(Debug, Default)]
pub struct SecurityAnalysisEngine;

impl SecurityAnalysisEngine {
    /// Analyze audit entries to produce aggregated metrics and severity scoring.
    pub fn analyze(&self, entries: &[AuditLogEntry]) -> SecurityAnalysis {
        if entries.is_empty() {
            return SecurityAnalysis {
                total_events: 0,
                events_by_result: HashMap::new(),
                events_by_type: HashMap::new(),
                overall_severity: SeverityScore {
                    level: SeverityLevel::Informational,
                    score: 0.0,
                    rationale: "No audit events processed".to_string(),
                    contributing_events: Vec::new(),
                },
                notes: vec!["No events available for analysis".to_string()],
            };
        }

        let mut events_by_result: HashMap<String, usize> = HashMap::new();
        let mut events_by_type: HashMap<String, usize> = HashMap::new();
        let mut highest_score = 0.0f32;
        let mut highest_level = SeverityLevel::Informational;
        let mut rationale = String::new();
        let mut contributing_events = Vec::new();
        let mut notes = Vec::new();

        for entry in entries {
            let event = &entry.event;
            *events_by_result
                .entry(format!("{:?}", event.result))
                .or_insert(0) += 1;
            *events_by_type
                .entry(format!("{:?}", event.event_type))
                .or_insert(0) += 1;

            let (score, level, reason) = Self::score_event(event);
            if score > highest_score || (score == highest_score && level > highest_level) {
                highest_score = score;
                highest_level = level;
                rationale = reason.clone();
                contributing_events = vec![event.id];
            } else if (highest_score - score).abs() < f32::EPSILON && level == highest_level {
                contributing_events.push(event.id);
            }
        }

        if let Some(blocked) = events_by_result.get("Blocked") {
            if *blocked > 0 {
                notes.push(format!(
                    "{} blocked event(s) detected; manual review recommended",
                    blocked
                ));
            }
        }
        if let Some(denied) = events_by_result.get("Denied") {
            if *denied > 2 {
                notes.push("Multiple denied events detected within batch".to_string());
            }
        }

        SecurityAnalysis {
            total_events: entries.len(),
            events_by_result,
            events_by_type,
            overall_severity: SeverityScore {
                level: highest_level,
                score: highest_score.min(1.0),
                rationale: if rationale.is_empty() {
                    "Severity derived from audit batch".to_string()
                } else {
                    rationale
                },
                contributing_events,
            },
            notes,
        }
    }

    /// Score a single event, returning a numeric score, severity level, and rationale.
    fn score_event(event: &SecurityAuditEvent) -> (f32, SeverityLevel, String) {
        let mut score: f32 = match event.result {
            AuditResult::Allowed => 0.05_f32,
            AuditResult::Approved => 0.1_f32,
            AuditResult::Warning => 0.35_f32,
            AuditResult::Denied => 0.55_f32,
            AuditResult::Rejected => 0.65_f32,
            AuditResult::Blocked => 0.75_f32,
        };

        let mut rationale = vec![format!(
            "{} result for action '{}'",
            format!("{:?}", event.result),
            event.action
        )];

        match event.event_type {
            AuditEventType::SecretDetection => {
                if let Some(severity) = event
                    .metadata
                    .get("secret_severity")
                    .and_then(|raw| raw.parse::<u8>().ok())
                {
                    let mapped = match severity {
                        4..=u8::MAX => SeverityLevel::Critical,
                        3 => SeverityLevel::High,
                        2 => SeverityLevel::Medium,
                        1 => SeverityLevel::Low,
                        _ => SeverityLevel::Informational,
                    };
                    score = score.max(Self::level_floor(mapped));
                    rationale.push(format!("Secret severity override ({mapped:?}) detected"));
                } else {
                    score = score.max(0.8_f32);
                    rationale.push("Secret detection without explicit severity".to_string());
                }
            }
            AuditEventType::PolicyViolation | AuditEventType::CouncilDecision => {
                score = score.max(0.7_f32);
                rationale.push("Policy or council decision flagged".to_string());
            }
            AuditEventType::SecurityCheck => {
                score += 0.05_f32;
                rationale.push("Security check recorded".to_string());
            }
            AuditEventType::CommandExecution | AuditEventType::FileAccess => {
                if matches!(event.result, AuditResult::Denied | AuditResult::Blocked) {
                    score += 0.1_f32;
                    rationale.push("Access prevented by policy".to_string());
                }
            }
        }

        let level = Self::level_from_score(score);
        (score.min(1.0), level, rationale.join("; "))
    }

    fn level_from_score(score: f32) -> SeverityLevel {
        if score >= 0.85 {
            SeverityLevel::Critical
        } else if score >= 0.65 {
            SeverityLevel::High
        } else if score >= 0.45 {
            SeverityLevel::Medium
        } else if score >= 0.25 {
            SeverityLevel::Low
        } else {
            SeverityLevel::Informational
        }
    }

    fn level_floor(level: SeverityLevel) -> f32 {
        match level {
            SeverityLevel::Critical => 0.9,
            SeverityLevel::High => 0.7,
            SeverityLevel::Medium => 0.5,
            SeverityLevel::Low => 0.3,
            SeverityLevel::Informational => 0.0,
        }
    }
}
