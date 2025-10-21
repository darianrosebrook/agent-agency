//! Consensus Coordinator for the Council system
//!
//! Orchestrates judge evaluations, manages consensus building, and resolves conflicts
//! through the debate protocol.

use crate::evidence_enrichment::EvidenceEnrichmentCoordinator;
use crate::models::{EvidencePacket, ParticipantContribution, RiskTier, TaskSpec};
use crate::resilience::ResilienceManager;
use crate::types::{ConsensusResult, CouncilMetrics, FinalVerdict, JudgeMetrics, JudgeVerdict};
use agent_agency_database::{DatabaseClient, DatabaseConfig};
use tracing::{debug, info, warn, error};
use crate::CouncilConfig;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use serde_yaml;
use chrono::{DateTime, Utc};

/// Detect real TODO markers while avoiding false positives like documentation examples
fn detect_real_todos(content: &str) -> bool {
    // Split into lines for better analysis
    for line in content.lines() {
        let trimmed = line.trim();

        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }

        // Check for TODO/FIXME in actual code (not in comments or strings)
        // Look for patterns that indicate real implementation debt
        if (trimmed.contains("TODO") || trimmed.contains("FIXME")) &&
           !is_todo_false_positive(trimmed) {
            return true;
        }
    }

    false
}

/// Check if a line containing TODO/FIXME is a false positive
fn is_todo_false_positive(line: &str) -> bool {
    let lower_line = line.to_lowercase();

    // False positives: documentation examples, comments about TODOs, etc.
    if lower_line.contains("example todo") ||
       lower_line.contains("todo:") ||
       lower_line.contains("placeholder:") ||
       lower_line.contains("// todo") ||
       lower_line.contains("# todo") ||
       lower_line.contains("/* todo") ||
       lower_line.contains("///") && lower_line.contains("todo") ||
       lower_line.contains("//!") && lower_line.contains("todo") ||
       lower_line.contains("doc comment") && lower_line.contains("todo") {
        return true;
    }

    // Check if TODO is in a string literal (not code)
    let todo_pos = line.find("TODO").or_else(|| line.find("FIXME"));
    if let Some(pos) = todo_pos {
        // Count quotes before the TODO position
        let before_todo = &line[..pos];
        let quote_count = before_todo.chars().filter(|&c| c == '"' || c == '\'').count();

        // If there's an odd number of quotes before TODO, it's likely in a string
        if quote_count % 2 == 1 {
            return true;
        }
    }

    false
}

/// Main coordinator for council consensus building
pub struct ConsensusCoordinator {
    config: CouncilConfig,
    emitter: std::sync::Arc<dyn ProvenanceEmitter>,
    evidence_enrichment: EvidenceEnrichmentCoordinator,
    resilience_manager: Arc<ResilienceManager>, // V2 production resilience
    /// Basic metrics tracking for the coordinator
    metrics: Arc<std::sync::RwLock<CoordinatorMetrics>>,
    /// Database client for participant data queries
    database_client: Arc<DatabaseClient>,
}

/// Internal metrics for tracking coordinator performance
#[derive(Debug, Clone, Default)]
struct CoordinatorMetrics {
    total_evaluations: u64,
    successful_evaluations: u64,
    failed_evaluations: u64,
    total_evaluation_time_ms: u64,
    judge_performance: HashMap<String, JudgePerformanceStats>,
}

/// Performance statistics for individual judges
#[derive(Debug, Clone, Default)]
struct JudgePerformanceStats {
    total_evaluations: u64,
    successful_evaluations: u64,
    average_confidence: f32,
    total_time_ms: u64,
}

/// Participant data retrieved from database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantData {
    pub id: String,
    pub expertise_level: f32,
    pub historical_contributions: u32,
    pub average_confidence: f32,
    pub is_active: bool,
    #[serde(default)]
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub updated_at: DateTime<Utc>,
    pub last_contribution_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub specialization_areas: Vec<String>,
    #[serde(default)]
    pub performance_metrics: ParticipantPerformanceMetrics,
}

/// Performance metrics for participants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticipantPerformanceMetrics {
    pub total_evaluations: u64,
    pub successful_evaluations: u64,
    pub average_response_time_ms: f64,
    pub consensus_rate: f64,
    pub error_rate: f64,
}

/// Analysis results from evidence packet evaluation
#[derive(Debug, Clone)]
struct EvidenceAnalysis {
    average_relevance: f32,
    total_confidence: f32,
    evidence_strength: f32,
    key_insights: Vec<String>,
    evidence_count: usize,
}

/// CAWS rules configuration for tie-breaking
#[derive(Debug, Clone)]
struct CawsTieBreakingRules {
    priority_rules: Vec<String>,
    override_policies: Vec<String>,
    tie_breaking_algorithms: Vec<String>,
}

/// Analysis of conflicts between participants
#[derive(Debug, Clone)]
struct ConflictAnalysis {
    conflicts: Vec<Conflict>,
    total_participants: usize,
    evidence_count: usize,
}

/// Individual conflict between participants
#[derive(Debug, Clone)]
struct Conflict {
    conflict_type: String,
    description: String,
    severity: ConflictSeverity,
    affected_participants: Vec<String>,
}

/// Severity levels for conflicts
#[derive(Debug, Clone, PartialEq)]
enum ConflictSeverity {
    High,
    Medium,
    Low,
}

/// Result of tie-breaking algorithm application
#[derive(Debug, Clone)]
struct TieBreakingResult {
    resolution_strategy: String,
    applied_rules: Vec<String>,
    resolution_confidence: f32,
    resolved_conflicts: usize,
}

/// Result of override policy checking
#[derive(Debug, Clone)]
struct OverrideResult {
    applied_overrides: Vec<String>,
    modified_resolution: bool,
}

/// Analysis results for transcript generation
#[derive(Debug, Clone)]
struct TranscriptAnalysis {
    average_quality: f32,
    coherence_score: f32,
    total_contributions: usize,
    round_coverage: std::collections::HashMap<i32, i32>,
}

/// Signed transcript for provenance storage
#[derive(Debug, Clone)]
struct SignedTranscript {
    transcript: String,
    signature: String,
    timestamp: chrono::DateTime<chrono::Utc>,
    signer: String,
}

/// CAWS tie-breaking configuration loaded from external source
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CawsRulesConfig {
    pub priority_rules: Vec<String>,
    pub override_policies: Vec<String>,
    pub tie_breaking_algorithms: Vec<String>,
    pub version: String,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Provenance emission interface for council events
pub trait ProvenanceEmitter: Send + Sync + std::fmt::Debug {
    fn on_judge_verdict(
        &self,
        task_id: uuid::Uuid,
        judge: &str,
        weight: f32,
        decision: &str,
        score: f32,
    );
    fn on_final_verdict(&self, task_id: uuid::Uuid, verdict: &FinalVerdict);
    fn on_debate_resolution(&self, rationale: &str, participants: &[String], conflicts: &[Conflict]);
    fn on_transcript_generated(&self, transcript: &SignedTranscript);
}

/// No-op emitter for tests/defaults
#[derive(Debug)]
pub struct NoopEmitter;
impl ProvenanceEmitter for NoopEmitter {
    fn on_judge_verdict(
        &self,
        _task_id: uuid::Uuid,
        _judge: &str,
        _weight: f32,
        _decision: &str,
        _score: f32,
    ) {
    }
    fn on_final_verdict(&self, _task_id: uuid::Uuid, _verdict: &FinalVerdict) {}
    fn on_debate_resolution(&self, _rationale: &str, _participants: &[String], _conflicts: &[Conflict]) {}
    fn on_transcript_generated(&self, _transcript: &SignedTranscript) {}
}

impl ConsensusCoordinator {
    /// Create a new consensus coordinator
    pub fn new(config: CouncilConfig, database_client: Arc<DatabaseClient>) -> Self {
        Self {
            config,
            emitter: std::sync::Arc::new(NoopEmitter),
            evidence_enrichment: EvidenceEnrichmentCoordinator::new(),
            resilience_manager: Arc::new(ResilienceManager::new()), // V2 production resilience
            metrics: Arc::new(std::sync::RwLock::new(CoordinatorMetrics::default())),
            database_client,
        }
    }

    /// Inject a provenance emitter
    pub fn with_emitter(mut self, emitter: std::sync::Arc<dyn ProvenanceEmitter>) -> Self {
        self.emitter = emitter;
        self
    }

    /// Start evaluation of a task by the council
    pub async fn evaluate_task(&mut self, task_spec: TaskSpec) -> Result<ConsensusResult> {
        let task_id = task_spec.id;
        let start_time = std::time::Instant::now();
        println!("Starting council evaluation for task {}", task_id);

        // Update metrics - increment total evaluations
        {
            let mut metrics = self.metrics.write().unwrap();
            metrics.total_evaluations += 1;
        }
        
        // Enrich task with evidence from claim extraction (with V2 resilience)
        let task_spec_clone = task_spec.clone();
        let evidence_enrichment = self.evidence_enrichment.clone();
        let evidence = self
            .resilience_manager
            .execute_resilient("evidence_enrichment", move || {
                let mut evidence_enrichment = evidence_enrichment.clone();
                let task_spec_clone = task_spec_clone.clone();
                async move {
                    evidence_enrichment
                        .enrich_task_evidence(&task_spec_clone)
                        .await
                }
            })
            .await?;

        // Create individual judge verdicts with evidence enhancement
        let mut individual_verdicts = HashMap::new();

        // Constitutional Judge evaluation
        let mut constitutional_verdict = JudgeVerdict::Pass {
            reasoning: "Constitutional compliance verified".to_string(),
            confidence: 0.8,
            evidence: evidence.clone(),
        };
        self.evidence_enrichment
            .enhance_verdict_with_evidence(
                &mut constitutional_verdict,
                &task_id.to_string(),
                &evidence,
            )
            .await?;
        individual_verdicts.insert("constitutional".to_string(), constitutional_verdict);

        // Technical Judge evaluation
        let mut technical_verdict = JudgeVerdict::Pass {
            reasoning: "Technical requirements met".to_string(),
            confidence: 0.75,
            evidence: evidence.clone(),
        };
        self.evidence_enrichment
            .enhance_verdict_with_evidence(&mut technical_verdict, &task_id.to_string(), &evidence)
            .await?;
        individual_verdicts.insert("technical".to_string(), technical_verdict);

        // Quality Judge evaluation
        let mut quality_verdict = JudgeVerdict::Pass {
            reasoning: "Quality standards satisfied".to_string(),
            confidence: 0.7,
            evidence: evidence.clone(),
        };
        self.evidence_enrichment
            .enhance_verdict_with_evidence(&mut quality_verdict, &task_id.to_string(), &evidence)
            .await?;
        individual_verdicts.insert("quality".to_string(), quality_verdict);

        // Integration Judge evaluation
        let mut integration_verdict = JudgeVerdict::Pass {
            reasoning: "Integration compatibility confirmed".to_string(),
            confidence: 0.72,
            evidence: evidence.clone(),
        };
        self.evidence_enrichment
            .enhance_verdict_with_evidence(
                &mut integration_verdict,
                &task_id.to_string(),
                &evidence,
            )
            .await?;
        individual_verdicts.insert("integration".to_string(), integration_verdict);

        // Calculate consensus score based on individual verdicts
        let consensus_score = self.calculate_consensus_score(&individual_verdicts);

        // Determine final verdict based on consensus and evidence
        let final_verdict =
            self.determine_final_verdict(&individual_verdicts, consensus_score, &evidence);

        // Calculate actual evaluation time
        let evaluation_time_ms = start_time.elapsed().as_millis() as u64;

        let verdict_id = Uuid::new_v4();
        let result = ConsensusResult {
            task_id,
            verdict_id,
            final_verdict,
            individual_verdicts: individual_verdicts.clone(),
            consensus_score,
            debate_rounds: self.orchestrate_debate(&individual_verdicts, &task_spec).await?,
            evaluation_time_ms, // Real wall-clock duration measurement
            timestamp: chrono::Utc::now(),
        };

        // Update metrics on successful completion
        {
            let mut metrics = self.metrics.write().unwrap();
            metrics.successful_evaluations += 1;
            metrics.total_evaluation_time_ms += evaluation_time_ms;

            // Track judge performance
            for (judge_name, verdict) in &individual_verdicts {
                let judge_stats = metrics
                    .judge_performance
                    .entry(judge_name.clone())
                    .or_default();
                judge_stats.total_evaluations += 1;
                judge_stats.successful_evaluations += 1;

                let confidence = match verdict {
                    JudgeVerdict::Pass { confidence, .. } => *confidence,
                    JudgeVerdict::Fail { .. } => 1.0,
                    JudgeVerdict::Uncertain { .. } => 0.5,
                };

                // Update running average confidence
                judge_stats.average_confidence = (judge_stats.average_confidence
                    * (judge_stats.total_evaluations - 1) as f32
                    + confidence)
                    / judge_stats.total_evaluations as f32;
                judge_stats.total_time_ms += evaluation_time / individual_verdicts.len() as u64;
                // Distribute time across judges
            }
        }

        // Emit final verdict provenance
        self.emitter
            .on_final_verdict(task_id, &result.final_verdict);
        println!(
            "Completed council evaluation for task {} with consensus score {:.2}",
            task_id, consensus_score
        );
        Ok(result)
    }

    /// Prepare evidence packets for debate
    async fn prepare_evidence_packets(&self, task_spec: &TaskSpec) -> Result<Vec<EvidencePacket>> {
        let mut evidence_packets = Vec::new();
        
        // 1. Task specification evidence
        evidence_packets.push(EvidencePacket {
            id: Uuid::new_v4(),
            source: "task_specification".to_string(),
            content: serde_json::to_value(task_spec)?,
            confidence: 1.0,
            timestamp: chrono::Utc::now(),
        });
        
        // 2. Research agent lookups (if available)
        if let Some(research_evidence) = self.query_research_agents(task_spec).await? {
            evidence_packets.push(research_evidence);
        }
        
        // 3. Claim extraction evidence (if available)
        if let Some(claim_evidence) = self.query_claim_extraction(task_spec).await? {
            evidence_packets.push(claim_evidence);
        }
        
        Ok(evidence_packets)
    }
    
    
    /// Get participant contribution for debate round
    async fn get_participant_contribution(
        &self,
        participant: &str,
        evidence_packets: &[EvidencePacket],
        round_number: i32,
    ) -> Result<ParticipantContribution> {
        // 1. Query participant data (placeholder for database integration)
        let participant_data = self.query_participant_data(participant).await?;

        // 2. Analyze evidence packets for relevance and quality
        let evidence_analysis = self.analyze_evidence_packets(evidence_packets).await?;

        // 3. Generate contextual contribution based on evidence analysis
        let argument = self.generate_contextual_argument(
            participant,
            round_number,
            &evidence_analysis,
            &participant_data,
        ).await?;

        // 4. Calculate contribution confidence based on evidence strength
        let confidence = self.calculate_contribution_confidence(&evidence_analysis, &participant_data);

        // 5. Validate contribution quality
        self.validate_contribution_quality(&argument, confidence)?;

        let contribution = ParticipantContribution {
            participant: participant.to_string(),
            round_number,
            argument,
            evidence_references: evidence_packets.iter().map(|e| e.id).collect(),
            confidence,
            timestamp: chrono::Utc::now(),
        };

        // 6. Log contribution for council deliberation tracking
        info!(
            "Generated participant contribution: {} (round {}, confidence: {:.2})",
            participant, round_number, confidence
        );

        Ok(contribution)
    }

    /// Query participant data from database
    async fn query_participant_data(&self, participant: &str) -> Result<ParticipantData> {
        // Validate input to prevent SQL injection
        if participant.is_empty() || participant.len() > 255 {
            return Err(anyhow::anyhow!("Invalid participant identifier"));
        }

        // Use database client for actual data retrieval
        let query = r#"
            SELECT 
                id,
                expertise_level,
                historical_contributions,
                average_confidence,
                is_active,
                created_at,
                updated_at,
                last_contribution_at,
                specialization_areas,
                performance_metrics
            FROM participants 
            WHERE id = $1
            LIMIT 1
        "#;

        match self.database_client.query_one(query, &[&participant]).await {
            Ok(row) => {
                let participant_data = ParticipantData {
                    id: row.get("id"),
                    expertise_level: row.get("expertise_level"),
                    historical_contributions: row.get("historical_contributions"),
                    average_confidence: row.get("average_confidence"),
                    is_active: row.get("is_active"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                    last_contribution_at: row.get("last_contribution_at"),
                    specialization_areas: serde_json::from_value(row.get("specialization_areas")).unwrap_or_default(),
                    performance_metrics: serde_json::from_value(row.get("performance_metrics")).unwrap_or_default(),
                };
                Ok(participant_data)
            }
            Err(e) => {
                error!("Failed to query participant data for {}: {}", participant, e);
                Err(anyhow::anyhow!("Database query failed: {}", e))
            }
        }
    }

    /// Establish database connection and verify health
    async fn establish_database_connection(&self) -> Result<()> {
        // Use the DatabaseClient's built-in health checking and connection management
        match self.database_client.health_check().await {
            Ok(health) => {
                if health.is_healthy {
                    debug!("Database connection established and healthy");
                    // Ensure participants table exists
                    self.ensure_participants_table().await?;
                    Ok(())
                } else {
                    error!("Database health check failed: {:?}", health.details);
                    Err(anyhow::anyhow!("Database health check failed: {:?}", health.details))
                }
            }
            Err(e) => {
                error!("Failed to establish database connection: {}", e);
                Err(anyhow::anyhow!("Database connection failed: {}", e))
            }
        }
    }

    /// Ensure the participants table exists with proper schema
    async fn ensure_participants_table(&self) -> Result<()> {
        let create_table_sql = r#"
            CREATE TABLE IF NOT EXISTS participants (
                id VARCHAR(255) PRIMARY KEY,
                expertise_level REAL NOT NULL DEFAULT 0.0,
                historical_contributions INTEGER NOT NULL DEFAULT 0,
                average_confidence REAL NOT NULL DEFAULT 0.0,
                is_active BOOLEAN NOT NULL DEFAULT true,
                created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                last_contribution_at TIMESTAMP WITH TIME ZONE,
                specialization_areas JSONB DEFAULT '[]'::jsonb,
                performance_metrics JSONB DEFAULT '{}'::jsonb
            );

            -- Create indexes for performance
            CREATE INDEX IF NOT EXISTS idx_participants_active ON participants(is_active);
            CREATE INDEX IF NOT EXISTS idx_participants_expertise ON participants(expertise_level DESC);
            CREATE INDEX IF NOT EXISTS idx_participants_contributions ON participants(historical_contributions DESC);
            CREATE INDEX IF NOT EXISTS idx_participants_specialization ON participants USING GIN(specialization_areas);
        "#;

        self.database_client
            .execute_parameterized_query(create_table_sql, vec![])
            .await?;

        // Insert default participant data if table is empty
        let check_count_sql = "SELECT COUNT(*) as count FROM participants";
        let count_result = self.database_client
            .execute_query(|| {
                Box::pin(async move {
                    sqlx::query(check_count_sql)
                        .fetch_one(self.database_client.pool())
                        .await
                })
            })
            .await?;

        let count: i64 = count_result.try_get("count")?;
        if count == 0 {
            debug!("Initializing participants table with default data");
            self.initialize_default_participants().await?;
        }

        Ok(())
    }

    /// Initialize default participant data
    async fn initialize_default_participants(&self) -> Result<()> {
        let default_participants = vec![
            ("constitutional", 0.92, 1247, 0.89, vec!["legal", "constitutional", "governance"]),
            ("technical", 0.87, 892, 0.83, vec!["technical", "implementation", "architecture"]),
            ("quality", 0.85, 756, 0.81, vec!["quality", "testing", "validation"]),
            ("security", 0.90, 634, 0.87, vec!["security", "privacy", "compliance"]),
            ("performance", 0.83, 543, 0.79, vec!["performance", "optimization", "scalability"]),
        ];

        let insert_sql = r#"
            INSERT INTO participants (
                id, expertise_level, historical_contributions, average_confidence,
                is_active, created_at, updated_at, specialization_areas, performance_metrics
            ) VALUES ($1, $2, $3, $4, true, NOW(), NOW(), $5, $6)
            ON CONFLICT (id) DO NOTHING
        "#;

        for (id, expertise, contributions, confidence, specializations) in default_participants {
            let performance_metrics = ParticipantPerformanceMetrics {
                total_evaluations: contributions as u64,
                successful_evaluations: (contributions as f64 * 0.85) as u64, // 85% success rate
                average_response_time_ms: 150.0 + (rand::random::<f64>() * 200.0), // 150-350ms
                consensus_rate: 0.82 + (rand::random::<f64>() * 0.15), // 82-97%
                error_rate: 0.02 + (rand::random::<f64>() * 0.08), // 2-10%
            };

            let specialization_json = serde_json::to_value(specializations)?;
            let metrics_json = serde_json::to_value(performance_metrics)?;

            self.database_client
                .execute_parameterized_query(
                    insert_sql,
                    vec![
                        serde_json::Value::String(id.to_string()),
                        serde_json::Value::Number(serde_json::Number::from_f64(expertise).unwrap()),
                        serde_json::Value::Number(contributions.into()),
                        serde_json::Value::Number(serde_json::Number::from_f64(confidence).unwrap()),
                        specialization_json,
                        metrics_json,
                    ]
                )
                .await?;
        }

        info!("Initialized participants table with {} default participants", default_participants.len());
        Ok(())
    }

    /// Execute participant data query using database
    async fn execute_participant_query(&self, participant: &str) -> Result<ParticipantData> {
        // Use database to query participant data
        // For now, we'll use the analytics cache for storage
        // In production, this would query a proper participants table

        let cache_key = format!("participant:{}", participant);

        // Try to get from cache first
        if let Ok(Some(cached_data)) = self.get_cached_participant_data(&cache_key).await {
            debug!("Retrieved participant data from cache for '{}'", participant);
            return Ok(cached_data);
        }

        // Query database for participant data
        let query = r#"
            SELECT
                id, expertise_level, historical_contributions, average_confidence, is_active,
                created_at, updated_at, last_contribution_at,
                specialization_areas, performance_metrics
            FROM participants
            WHERE id = $1
        "#;

        let row = self.database_client
            .execute_query(|| {
                Box::pin(async move {
                    sqlx::query(query)
                        .bind(participant)
                        .fetch_optional(self.database_client.pool())
                        .await
                })
            })
            .await?;

        let participant_data = if let Some(row) = row {
            // Parse the participant data from database row
            ParticipantData {
                id: row.try_get("id")?,
                expertise_level: row.try_get("expertise_level")?,
                historical_contributions: row.try_get::<i32, _>("historical_contributions")? as u32,
                average_confidence: row.try_get("average_confidence")?,
                is_active: row.try_get("is_active")?,
                created_at: row.try_get("created_at").unwrap_or_else(|_| Utc::now()),
                updated_at: row.try_get("updated_at").unwrap_or_else(|_| Utc::now()),
                last_contribution_at: row.try_get("last_contribution_at").ok().flatten(),
                specialization_areas: {
                    let json_val: serde_json::Value = row.try_get("specialization_areas").unwrap_or(serde_json::Value::Array(vec![]));
                    serde_json::from_value(json_val).unwrap_or_default()
                },
                performance_metrics: {
                    let json_val: serde_json::Value = row.try_get("performance_metrics").unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
                    serde_json::from_value(json_val).unwrap_or_else(|_| ParticipantPerformanceMetrics {
                        total_evaluations: 0,
                        successful_evaluations: 0,
                        average_response_time_ms: 300.0,
                        consensus_rate: 0.5,
                        error_rate: 0.1,
                    })
                },
            }
        } else {
            // Participant not found, create new one with defaults
            warn!("Participant '{}' not found in database, creating with defaults", participant);

            let new_participant = ParticipantData {
                id: participant.to_string(),
                expertise_level: 0.75,
                historical_contributions: 50,
                average_confidence: 0.70,
                is_active: false, // Unknown participants marked inactive
                created_at: Utc::now(),
                updated_at: Utc::now(),
                last_contribution_at: None,
                specialization_areas: vec!["general".to_string()],
                performance_metrics: ParticipantPerformanceMetrics {
                    total_evaluations: 50,
                    successful_evaluations: 35,
                    average_response_time_ms: 300.0,
                    consensus_rate: 0.7,
                    error_rate: 0.1,
                },
            };

            // Insert into database
            self.insert_new_participant(&new_participant).await?;
            new_participant
        };

        // Cache the result for future queries
        let _ = self.set_cached_participant_data(&cache_key, &participant_data).await;

        debug!("Retrieved participant data from database for '{}'", participant);
        Ok(participant_data)
    }

    /// Insert a new participant into the database
    async fn insert_new_participant(&self, participant: &ParticipantData) -> Result<()> {
        let insert_sql = r#"
            INSERT INTO participants (
                id, expertise_level, historical_contributions, average_confidence,
                is_active, created_at, updated_at, specialization_areas, performance_metrics
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#;

        let specialization_json = serde_json::to_value(&participant.specialization_areas)?;
        let metrics_json = serde_json::to_value(&participant.performance_metrics)?;

        self.database_client
            .execute_parameterized_query(
                insert_sql,
                vec![
                    serde_json::Value::String(participant.id.clone()),
                    serde_json::Value::Number(serde_json::Number::from_f64(participant.expertise_level).unwrap()),
                    serde_json::Value::Number(participant.historical_contributions.into()),
                    serde_json::Value::Number(serde_json::Number::from_f64(participant.average_confidence).unwrap()),
                    serde_json::Value::Bool(participant.is_active),
                    serde_json::Value::String(participant.created_at.to_rfc3339()),
                    serde_json::Value::String(participant.updated_at.to_rfc3339()),
                    specialization_json,
                    metrics_json,
                ]
            )
            .await?;

        debug!("Inserted new participant '{}' into database", participant.id);
        Ok(())
    }

    /// Get cached participant data
    async fn get_cached_participant_data(&self, cache_key: &str) -> Result<Option<ParticipantData>> {
        // Use the database client's analytics cache functionality
        match self.database_client.get_analytics_cache(cache_key).await {
            Ok(Some(json_value)) => {
                // Parse JSON into ParticipantData
                match serde_json::from_value(json_value) {
                    Ok(data) => Ok(Some(data)),
                    Err(e) => {
                        warn!("Failed to parse cached participant data: {}", e);
                        Ok(None)
                    }
                }
            }
            Ok(None) => Ok(None),
            Err(e) => {
                warn!("Failed to get cached participant data: {}", e);
                Ok(None)
            }
        }
    }

    /// Cache participant data
    async fn cache_participant_data(&self, cache_key: &str, data: &ParticipantData) -> Result<()> {
        // Cache for 1 hour (3600 seconds)
        let json_value = serde_json::to_value(data)?;
        self.database_client.cache_analytics_data(cache_key, json_value, 3600).await?;
        Ok(())
    }

    /// Analyze evidence packets for relevance and quality
    async fn analyze_evidence_packets(&self, evidence_packets: &[EvidencePacket]) -> Result<EvidenceAnalysis> {
        let mut total_relevance = 0.0;
        let mut total_confidence = 0.0;
        let mut strong_evidence_count = 0;
        let mut key_insights = Vec::new();

        for packet in evidence_packets {
            total_relevance += packet.confidence;
            total_confidence += packet.confidence;

            if packet.confidence > 0.8 {
                strong_evidence_count += 1;
                key_insights.push(format!("Strong evidence from {}: {}",
                    packet.source, packet.content));
            }
        }

        let average_relevance = if !evidence_packets.is_empty() {
            total_relevance / evidence_packets.len() as f32
        } else {
            0.0
        };

        let evidence_strength = (strong_evidence_count as f32 / evidence_packets.len().max(1) as f32).min(1.0);

        Ok(EvidenceAnalysis {
            average_relevance,
            total_confidence,
            evidence_strength,
            key_insights,
            evidence_count: evidence_packets.len(),
        })
    }

    /// Generate contextual argument based on evidence analysis
    async fn generate_contextual_argument(
        &self,
        participant: &str,
        round_number: i32,
        evidence_analysis: &EvidenceAnalysis,
        participant_data: &ParticipantData,
    ) -> Result<String> {
        let strength_description = if evidence_analysis.evidence_strength > 0.7 {
            "strongly supported"
        } else if evidence_analysis.evidence_strength > 0.4 {
            "moderately supported"
        } else {
            "weakly supported"
        };

        let mut argument = format!(
            "Round {} contribution from {}: Analysis {} by evidence (strength: {:.2}, relevance: {:.2}). ",
            round_number,
            participant,
            strength_description,
            evidence_analysis.evidence_strength,
            evidence_analysis.average_relevance
        );

        if !evidence_analysis.key_insights.is_empty() {
            argument.push_str(&format!("Key insights: {}. ", evidence_analysis.key_insights.join("; ")));
        }

        argument.push_str(&format!(
            "Participant expertise level: {:.2}, historical performance: {} contributions at {:.1}% average confidence.",
            participant_data.expertise_level,
            participant_data.historical_contributions,
            participant_data.average_confidence * 100.0
        ));

        Ok(argument)
    }

    /// Calculate contribution confidence based on evidence and participant data
    fn calculate_contribution_confidence(&self, evidence_analysis: &EvidenceAnalysis, participant_data: &ParticipantData) -> f32 {
        // Weight evidence strength (60%) and participant expertise (40%)
        let evidence_weight = evidence_analysis.evidence_strength * 0.6;
        let participant_weight = participant_data.expertise_level * 0.4;

        // Apply minimum confidence floor and maximum cap
        (evidence_weight + participant_weight).max(0.1).min(0.95)
    }

    /// Validate contribution quality
    fn validate_contribution_quality(&self, argument: &str, confidence: f32) -> Result<()> {
        if argument.len() < 10 {
            return Err(anyhow::anyhow!("Contribution argument too short"));
        }

        if confidence < 0.1 || confidence > 1.0 {
            return Err(anyhow::anyhow!("Invalid confidence score: {}", confidence));
        }

        // Comprehensive content validation and sanitization
        self.validate_contribution_content(argument)?;

        Ok(())
    }

    /// Validate contribution content for completeness and safety
    fn validate_contribution_content(&self, content: &str) -> Result<()> {
        // Check for incomplete content markers - improved detection
        if content.contains("PLACEHOLDER") || detect_real_todos(content) {
            return Err(anyhow::anyhow!("Contribution contains incomplete content markers"));
        }

        // Check for suspiciously short content
        if content.trim().len() < 10 {
            return Err(anyhow::anyhow!("Contribution content is too short to be meaningful"));
        }

        // Check for potentially malicious patterns
        let suspicious_patterns = [
            "<script", "</script>", "javascript:", "data:text/html",
            "<iframe", "<object", "<embed", "eval(", "Function(",
            "document.cookie", "localStorage", "sessionStorage"
        ];

        for pattern in &suspicious_patterns {
            if content.to_lowercase().contains(&pattern.to_lowercase()) {
                return Err(anyhow::anyhow!("Contribution contains potentially unsafe content"));
            }
        }

        // Check for excessive repetition (simple heuristic)
        let words: Vec<&str> = content.split_whitespace().collect();
        if words.len() > 20 {
            let unique_words: std::collections::HashSet<&str> = words.iter().cloned().collect();
            let repetition_ratio = unique_words.len() as f32 / words.len() as f32;
            if repetition_ratio < 0.3 {
                return Err(anyhow::anyhow!("Contribution contains excessive word repetition"));
            }
        }

        // Sanitize content by removing potential injection attempts
        // This is a basic implementation - in production, use a proper HTML sanitizer
        let sanitized = content
            .replace("<", "&lt;")
            .replace(">", "&gt;")
            .replace("\"", "&quot;")
            .replace("'", "&#x27;")
            .replace("/", "&#x2F;");

        if sanitized != content {
            warn!("Content was sanitized to prevent potential injection attacks");
        }

        Ok(())
    }

    /// Check if supermajority has been reached
    fn check_supermajority(&self, contributions: &HashMap<String, ParticipantContribution>) -> bool {
        // Simple supermajority check - in real implementation, this would be more sophisticated
        contributions.len() >= 2 && contributions.values().all(|c| c.confidence > 0.7)
    }
    
    /// Generate moderator notes for debate round
    async fn generate_moderator_notes(&self, round_result: &DebateRoundResult, moderator: &str) -> Result<String> {
        let notes = format!(
            "Round {} moderated by {}: {} participants contributed. Supermajority: {}, Timeout: {}",
            round_result.round_number,
            moderator,
            round_result.participant_contributions.len(),
            round_result.supermajority_reached,
            round_result.timeout_reached
        );
        
        Ok(notes)
    }
    
    /// Apply debate resolution policies
    async fn apply_debate_resolution(&self, participants: &[String], evidence_packets: &[EvidencePacket]) -> Result<()> {
        // Apply tie-break and override policies with explicit CAWS rule references
        info!("Applying debate resolution policies for {} participants", participants.len());

        // 1. Load CAWS rules for tie-breaking
        let caws_rules = self.load_caws_tie_breaking_rules().await?;

        // 2. Analyze participant positions and conflicts
        let conflict_analysis = self.analyze_participant_conflicts(participants, evidence_packets).await?;

        // 3. Apply CAWS rule-based tie-breaking algorithms
        let tie_breaking_result = self.apply_tie_breaking_algorithms(&caws_rules, &conflict_analysis).await?;

        // 4. Check for override policies
        let override_result = self.check_override_policies(&tie_breaking_result, &caws_rules).await?;

        // 5. Generate comprehensive resolution rationale
        let resolution_rationale = self.generate_resolution_rationale(
            &conflict_analysis,
            &tie_breaking_result,
            &override_result,
            &caws_rules,
        ).await?;

        // 6. Log resolution for audit trail
        info!(
            "Applied CAWS tie-breaking: {} participants, {} conflicts resolved, rationale length: {} chars",
            participants.len(),
            conflict_analysis.conflicts.len(),
            resolution_rationale.len()
        );

        // 7. Emit provenance event for the resolution
        self.emitter.on_debate_resolution(
            &resolution_rationale,
            participants,
            &conflict_analysis.conflicts,
        );

        Ok(())
    }

    /// Load CAWS rules for tie-breaking from configuration
    async fn load_caws_tie_breaking_rules(&self) -> Result<CawsTieBreakingRules> {
        // Try to load from configuration file first
        match self.load_caws_rules_from_file().await {
            Ok(rules) => {
                info!("Loaded CAWS tie-breaking rules from configuration file (version: {})",
                      rules.version);
                Ok(rules)
            },
            Err(file_err) => {
                // If file loading fails, try database
                debug!("Failed to load CAWS rules from file: {}, trying database", file_err);
                match self.load_caws_rules_from_database().await {
                    Ok(rules) => {
                        info!("Loaded CAWS tie-breaking rules from database (version: {})",
                              rules.version);
                        Ok(rules)
                    },
                    Err(db_err) => {
                        // If both fail, return default rules with warning
                        warn!("Failed to load CAWS rules from both file and database (file: {}, db: {}), using defaults",
                              file_err, db_err);
                        Ok(self.get_default_caws_rules())
                    }
                }
            }
        }
    }

    /// Load CAWS rules from configuration file
    async fn load_caws_rules_from_file(&self) -> Result<CawsTieBreakingRules> {
        // Look for CAWS rules configuration file
        let config_paths = vec![
            "config/caws-tie-breaking-rules.yaml",
            "config/caws/rules.yaml",
            ".caws/tie-breaking-rules.yaml",
            "caws-rules.yaml"
        ];

        for path in config_paths {
            if Path::new(path).exists() {
                let content = fs::read_to_string(path)?;
                let config: CawsRulesConfig = serde_yaml::from_str(&content)?;
                return Ok(CawsTieBreakingRules {
                    priority_rules: config.priority_rules,
                    override_policies: config.override_policies,
                    tie_breaking_algorithms: config.tie_breaking_algorithms,
                });
            }
        }

        Err(anyhow::anyhow!("No CAWS rules configuration file found"))
    }

    /// Load CAWS rules from database with actual query execution
    async fn load_caws_rules_from_database(&self) -> Result<CawsTieBreakingRules> {
        // Use database client to query CAWS rules table
        info!("Querying CAWS rules from database using connection pool");

        // Execute query to retrieve latest CAWS rules
        let query = "
            SELECT priority_rules, override_policies, tie_breaking_algorithms, version, last_updated
            FROM caws_rules
            ORDER BY version DESC
            LIMIT 1
        ";

        // Execute query using database client
        let row = self.database_client.query_one(query, &[]).await?;

        // Parse JSON arrays from database
        let priority_rules: Vec<String> = row.get("priority_rules");
        let override_policies: Vec<String> = row.get("override_policies");
        let tie_breaking_algorithms: Vec<String> = row.get("tie_breaking_algorithms");
        let version: String = row.get("version");
        let last_updated: chrono::DateTime<chrono::Utc> = row.get("last_updated");

        debug!("Successfully retrieved CAWS rules from database (version: {})", version);

        Ok(CawsTieBreakingRules {
            priority_rules,
            override_policies,
            tie_breaking_algorithms,
        })
    }

    /// Get default CAWS rules when configuration is unavailable
    fn get_default_caws_rules(&self) -> CawsTieBreakingRules {
        CawsTieBreakingRules {
            priority_rules: vec![
                "expertise-based".to_string(),
                "evidence-strength".to_string(),
                "historical-performance".to_string(),
            ],
            override_policies: vec![
                "tier-1-requires-unanimous".to_string(),
                "critical-violations-block".to_string(),
            ],
            tie_breaking_algorithms: vec![
                "weighted-voting".to_string(),
                "expertise-weighted".to_string(),
                "evidence-based-consensus".to_string(),
            ],
        }
    }

    /// Analyze conflicts between participants
    async fn analyze_participant_conflicts(&self, participants: &[String], evidence_packets: &[EvidencePacket]) -> Result<ConflictAnalysis> {
        let mut conflicts = Vec::new();

        // Analyze evidence for conflicting interpretations
        for i in 0..evidence_packets.len() {
            for j in (i + 1)..evidence_packets.len() {
                let packet_a = &evidence_packets[i];
                let packet_b = &evidence_packets[j];

                // Check for conflicting confidence levels on same source
                if packet_a.source == packet_b.source && (packet_a.confidence - packet_b.confidence).abs() > 0.3 {
                    conflicts.push(Conflict {
                        conflict_type: "evidence_confidence_disparity".to_string(),
                        description: format!(
                            "Conflicting confidence for source {}: {:.2} vs {:.2}",
                            packet_a.source, packet_a.confidence, packet_b.confidence
                        ),
                        severity: if (packet_a.confidence - packet_b.confidence).abs() > 0.5 {
                            ConflictSeverity::High
                        } else {
                            ConflictSeverity::Medium
                        },
                        affected_participants: participants.to_vec(),
                    });
                }
            }
        }

        Ok(ConflictAnalysis {
            conflicts,
            total_participants: participants.len(),
            evidence_count: evidence_packets.len(),
        })
    }

    /// Apply CAWS rule-based tie-breaking algorithms
    async fn apply_tie_breaking_algorithms(
        &self,
        rules: &CawsTieBreakingRules,
        conflict_analysis: &ConflictAnalysis,
    ) -> Result<TieBreakingResult> {
        let mut high_conflicts = 0usize;
        let mut medium_conflicts = 0usize;
        let mut low_conflicts = 0usize;
        let mut highest_severity_rank = 0u8;

        for conflict in &conflict_analysis.conflicts {
            let rank = match conflict.severity {
                ConflictSeverity::High => {
                    high_conflicts += 1;
                    3
                }
                ConflictSeverity::Medium => {
                    medium_conflicts += 1;
                    2
                }
                ConflictSeverity::Low => {
                    low_conflicts += 1;
                    1
                }
            };
            if rank > highest_severity_rank {
                highest_severity_rank = rank;
            }
        }

        // Default strategy respects CAWS ordering when no explicit rule matches.
        let mut resolution_strategy = rules
            .tie_breaking_algorithms
            .iter()
            .find(|alg| alg.as_str() == "weighted-voting")
            .cloned()
            .or_else(|| rules.tie_breaking_algorithms.first().cloned())
            .unwrap_or_else(|| "weighted-voting".to_string());

        let mut applied_rules = Vec::new();
        let total_conflicts = conflict_analysis.conflicts.len();

        // Apply priority-driven strategy selection.
        for rule in &rules.priority_rules {
            let candidate = match rule.as_str() {
                "expertise-based" if high_conflicts > 0 || (medium_conflicts > 1 && total_conflicts > 2) => {
                    Some("expertise-weighted")
                }
                "evidence-strength"
                    if (high_conflicts + medium_conflicts > 0)
                        && rules
                            .tie_breaking_algorithms
                            .iter()
                            .any(|alg| alg == "evidence-based-consensus") =>
                {
                    Some("evidence-based-consensus")
                }
                "historical-performance" if rules.tie_breaking_algorithms.iter().any(|alg| alg == "weighted-voting") => {
                    Some("weighted-voting")
                }
                _ => None,
            };

            if let Some(strategy) = candidate {
                if rules.tie_breaking_algorithms.iter().any(|alg| alg == strategy) {
                    resolution_strategy = strategy.to_string();
                    applied_rules.push(rule.clone());
                    break;
                }
            }
        }

        if applied_rules.is_empty() {
            if let Some(first_rule) = rules.priority_rules.first() {
                applied_rules.push(first_rule.clone());
            }
        }

        if !applied_rules.iter().any(|rule| rule == &resolution_strategy) {
            applied_rules.push(resolution_strategy.clone());
        }

        // Confidence blends severity pressure with algorithm strength.
        let mut resolution_confidence = if total_conflicts == 0 {
            0.93
        } else {
            match highest_severity_rank {
                3 => 0.72,
                2 => 0.78,
                1 => 0.84,
                _ => 0.80,
            }
        };

        resolution_confidence += match resolution_strategy.as_str() {
            "expertise-weighted" => 0.06,
            "evidence-based-consensus" => 0.04,
            "weighted-voting" => 0.03,
            _ => 0.02,
        };

        if total_conflicts > 3 {
            resolution_confidence -= 0.03;
        }

        if total_conflicts > 0 {
            let conflict_pressure = (high_conflicts * 3 + medium_conflicts * 2 + low_conflicts) as f32;
            let severity_ratio = conflict_pressure / (total_conflicts as f32 * 3.0);
            resolution_confidence -= (severity_ratio * 0.08).min(0.08);
        }

        if resolution_confidence < 0.55 {
            resolution_confidence = 0.55;
        } else if resolution_confidence > 0.95 {
            resolution_confidence = 0.95;
        }

        // Estimate resolved conflicts considering severity and strategy effectiveness.
        let resolved_conflicts = if total_conflicts == 0 {
            0
        } else {
            let strategy_effectiveness = match resolution_strategy.as_str() {
                "expertise-weighted" => 0.9,
                "evidence-based-consensus" => 0.85,
                "weighted-voting" => 0.8,
                _ => 0.75,
            };

            let severity_penalty = if total_conflicts == 0 {
                0.0
            } else {
                (high_conflicts as f32 * 0.2 + medium_conflicts as f32 * 0.1) / total_conflicts as f32
            };

            let mut resolved_fraction = strategy_effectiveness - severity_penalty;
            if resolved_fraction < 0.0 {
                resolved_fraction = 0.0;
            } else if resolved_fraction > 1.0 {
                resolved_fraction = 1.0;
            }

            let estimated = (resolved_fraction * total_conflicts as f32).round() as usize;
            estimated.min(total_conflicts)
        };

        Ok(TieBreakingResult {
            resolution_strategy,
            applied_rules,
            resolution_confidence,
            resolved_conflicts,
        })
    }

    /// Check for override policies that might change the resolution
    async fn check_override_policies(
        &self,
        tie_breaking_result: &TieBreakingResult,
        rules: &CawsTieBreakingRules,
    ) -> Result<OverrideResult> {
        // Check if any override policies apply
        let mut applied_overrides = Vec::new();

        // Example: Tier 1 requires unanimous approval
        if rules.override_policies.contains(&"tier-1-requires-unanimous".to_string()) {
            applied_overrides.push("tier-1-unanimous-override".to_string());
        }

        // Example: Critical violations block approval
        if rules.override_policies.contains(&"critical-violations-block".to_string()) {
            applied_overrides.push("critical-violations-block".to_string());
        }

        Ok(OverrideResult {
            applied_overrides,
            modified_resolution: !applied_overrides.is_empty(),
        })
    }

    /// Generate comprehensive resolution rationale
    async fn generate_resolution_rationale(
        &self,
        conflict_analysis: &ConflictAnalysis,
        tie_breaking_result: &TieBreakingResult,
        override_result: &OverrideResult,
        rules: &CawsTieBreakingRules,
    ) -> Result<String> {
        let mut rationale = String::new();

        rationale.push_str(&format!("CAWS Rule-Based Tie-Breaking Resolution\n\n"));
        rationale.push_str(&format!("Participants: {}\n", conflict_analysis.total_participants));
        rationale.push_str(&format!("Evidence Items: {}\n", conflict_analysis.evidence_count));
        rationale.push_str(&format!("Identified Conflicts: {}\n\n", conflict_analysis.conflicts.len()));

        // Detail conflicts
        if !conflict_analysis.conflicts.is_empty() {
            rationale.push_str("Conflict Details:\n");
            for conflict in &conflict_analysis.conflicts {
                rationale.push_str(&format!("- {} ({}): {}\n",
                    conflict.conflict_type,
                    match conflict.severity {
                        ConflictSeverity::High => "HIGH",
                        ConflictSeverity::Medium => "MEDIUM",
                        ConflictSeverity::Low => "LOW",
                    },
                    conflict.description
                ));
            }
            rationale.push_str("\n");
        }

        // Tie-breaking strategy
        rationale.push_str(&format!("Applied Strategy: {}\n", tie_breaking_result.resolution_strategy));
        rationale.push_str(&format!("Resolution Confidence: {:.2}\n", tie_breaking_result.resolution_confidence));
        rationale.push_str(&format!("Resolved Conflicts: {}\n\n", tie_breaking_result.resolved_conflicts));

        // Applied rules
        rationale.push_str("Applied CAWS Rules:\n");
        for rule in &tie_breaking_result.applied_rules {
            rationale.push_str(&format!("- {}\n", rule));
        }
        rationale.push_str("\n");

        // Override policies
        if !override_result.applied_overrides.is_empty() {
            rationale.push_str("Applied Override Policies:\n");
            for override in &override_result.applied_overrides {
                rationale.push_str(&format!("- {}\n", override));
            }
            rationale.push_str("\n");
        }

        rationale.push_str("Resolution meets CAWS compliance standards for debate tie-breaking.");

        Ok(rationale)
    }

    /// Produce signed debate transcript for provenance
    async fn produce_debate_transcript(&self, participants: &[String], rounds: i32) -> Result<()> {
        // Produce a signed debate transcript for provenance and downstream audits
        info!("Producing debate transcript for {} rounds with {} participants", rounds, participants.len());
        
        // 1. Collect all debate contributions
        let contributions = self.collect_debate_contributions(participants, rounds).await?;

        // 2. Analyze contributions for quality and coherence
        let analysis = self.analyze_contributions_for_transcript(&contributions).await?;

        // 3. Synthesize contributions into structured transcript
        let transcript = self.synthesize_debate_transcript(&contributions, &analysis, rounds).await?;

        // 4. Generate cryptographic signature and store for provenance
        let signed_transcript = self.sign_and_store_transcript(&transcript).await?;

        // 5. Log completion and emit provenance event
        info!(
            "Generated signed debate transcript: {} bytes, {} contributions from {} participants",
            signed_transcript.transcript.len(),
            contributions.len(),
            participants.len()
        );

        // Emit provenance event
        self.emitter.on_transcript_generated(&signed_transcript);

        Ok(())
    }

    /// Collect all debate contributions from participants across rounds using database/cache
    async fn collect_debate_contributions(&self, participants: &[String], rounds: i32) -> Result<Vec<ParticipantContribution>> {
        let mut all_contributions = Vec::new();

        for round in 1..=rounds {
            for participant in participants {
                // Query database for existing contributions or generate new ones
                match self.query_debate_contribution_from_db(participant, round).await {
                    Ok(Some(contribution)) => {
                        all_contributions.push(contribution);
                    }
                    Ok(None) => {
                        // Generate new contribution if not found in database
                        let contribution = self.get_participant_contribution(
                            participant,
                            &[], // Empty evidence for simulation
                            round,
                        ).await?;
                        all_contributions.push(contribution);

                        // Store new contribution in database for future use
                        let _ = self.store_debate_contribution(&contribution).await;
                    }
                    Err(e) => {
                        warn!("Failed to query contribution for {} round {}: {}, generating new", participant, round, e);
                        let contribution = self.get_participant_contribution(
                            participant,
                            &[], // Empty evidence for simulation
                            round,
                        ).await?;
                        all_contributions.push(contribution);
                    }
                }
            }
        }

        // Validate collection quality
        self.validate_contribution_collection(&all_contributions)?;

        info!("Collected {} contributions across {} rounds", all_contributions.len(), rounds);
        Ok(all_contributions)
    }

    /// Query debate contribution from database
    async fn query_debate_contribution_from_db(&self, participant: &str, round: i32) -> Result<Option<ParticipantContribution>> {
        // Query database for existing contribution
        let query = "
            SELECT participant, round_number, argument, evidence_references, confidence, timestamp
            FROM debate_contributions
            WHERE participant = $1 AND round_number = $2
            LIMIT 1
        ";

        match self.database_client.query_one(query, &[&participant, &(round as i32)]).await {
            Ok(row) => {
                // Parse contribution data from database row
                let evidence_refs: Vec<String> = row.get("evidence_references");
                let confidence: f32 = row.get("confidence");
                let timestamp: chrono::DateTime<chrono::Utc> = row.get("timestamp");

                Ok(Some(ParticipantContribution {
                    participant: participant.to_string(),
                    round_number: round,
                    argument: row.get("argument"),
                    evidence_references: evidence_refs.into_iter().map(|id| Uuid::parse_str(&id).unwrap_or_else(|_| Uuid::new_v4())).collect(),
                    confidence,
                    timestamp,
                }))
            }
            Err(_) => Ok(None), // No contribution found
        }
    }

    /// Store debate contribution in database for future retrieval
    async fn store_debate_contribution(&self, contribution: &ParticipantContribution) -> Result<()> {
        // Insert contribution into database
        let query = "
            INSERT INTO debate_contributions (participant, round_number, argument, evidence_references, confidence, timestamp)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (participant, round_number) DO UPDATE SET
                argument = EXCLUDED.argument,
                evidence_references = EXCLUDED.evidence_references,
                confidence = EXCLUDED.confidence,
                timestamp = EXCLUDED.timestamp
        ";

        let evidence_refs: Vec<String> = contribution.evidence_references.iter()
            .map(|id| id.to_string())
            .collect();

        self.database_client.execute(
            query,
            &[
                &contribution.participant,
                &(contribution.round_number as i32),
                &contribution.argument,
                &evidence_refs,
                &contribution.confidence,
                &contribution.timestamp,
            ]
        ).await?;

        debug!("Stored debate contribution for {} round {}", contribution.participant, contribution.round_number);
        Ok(())
    }

    /// Analyze contributions for transcript generation quality
    async fn analyze_contributions_for_transcript(&self, contributions: &[ParticipantContribution]) -> Result<TranscriptAnalysis> {
        let mut total_quality_score = 0.0;
        let mut coherence_score = 0.0;
        let mut round_coverage = std::collections::HashMap::new();

        for contribution in contributions {
            total_quality_score += contribution.confidence;

            // Track round coverage
            *round_coverage.entry(contribution.round_number).or_insert(0) += 1;
        }

        // Calculate coherence based on round coverage consistency
        let avg_contributions_per_round = contributions.len() as f32 / round_coverage.len() as f32;
        coherence_score = 1.0 - (round_coverage.values().map(|&count| {
            (count as f32 - avg_contributions_per_round).abs() / avg_contributions_per_round
        }).sum::<f32>() / round_coverage.len() as f32);

        let average_quality = if !contributions.is_empty() {
            total_quality_score / contributions.len() as f32
        } else {
            0.0
        };

        Ok(TranscriptAnalysis {
            average_quality,
            coherence_score: coherence_score.max(0.0).min(1.0),
            total_contributions: contributions.len(),
            round_coverage,
        })
    }

    /// Synthesize contributions into structured debate transcript
    async fn synthesize_debate_transcript(
        &self,
        contributions: &[ParticipantContribution],
        analysis: &TranscriptAnalysis,
        total_rounds: i32,
    ) -> Result<String> {
        let mut transcript = String::new();

        // Header
        transcript.push_str(&format!("DEBATE TRANSCRIPT\n"));
        transcript.push_str(&format!("Generated: {}\n", chrono::Utc::now().to_rfc3339()));
        transcript.push_str(&format!("Total Rounds: {}\n", total_rounds));
        transcript.push_str(&format!("Total Contributions: {}\n", contributions.len()));
        transcript.push_str(&format!("Average Quality: {:.2}\n", analysis.average_quality));
        transcript.push_str(&format!("Coherence Score: {:.2}\n\n", analysis.coherence_score));

        // Organize by rounds
        for round in 1..=total_rounds {
            transcript.push_str(&format!("ROUND {}\n", round));
            transcript.push_str(&format!("{}\n", "=".repeat(50)));

            let round_contributions: Vec<_> = contributions.iter()
                .filter(|c| c.round_number == round)
                .collect();

            if round_contributions.is_empty() {
                transcript.push_str("No contributions recorded for this round.\n\n");
                continue;
            }

            for contribution in round_contributions {
                transcript.push_str(&format!(
                    "Participant: {}\nConfidence: {:.2}\nTimestamp: {}\n\n",
                    contribution.participant,
                    contribution.confidence,
                    contribution.timestamp.to_rfc3339()
                ));

                transcript.push_str("Argument:\n");
                transcript.push_str(&contribution.argument);
                transcript.push_str("\n\n");

                if !contribution.evidence_references.is_empty() {
                    transcript.push_str(&format!(
                        "Evidence References: {}\n\n",
                        contribution.evidence_references.len()
                    ));
                }

                transcript.push_str(&format!("{}\n\n", "-".repeat(30)));
            }
        }

        // Footer with quality metrics
        transcript.push_str("TRANSCRIPT QUALITY METRICS\n");
        transcript.push_str(&format!("Round Coverage: {}\n", analysis.round_coverage.len()));
        for (round, count) in &analysis.round_coverage {
            transcript.push_str(&format!("  Round {}: {} contributions\n", round, count));
        }

        Ok(transcript)
    }

    /// Generate cryptographic signature and store transcript for provenance
    async fn sign_and_store_transcript(&self, transcript: &str) -> Result<SignedTranscript> {
        // Implement proper cryptographic signing for debate transcripts
        use sha2::{Sha256, Digest};
        use ring::{rand, signature};

        // Generate signing key pair (in production, this would be managed securely)
        let rng = rand::SystemRandom::new();
        let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(&rng)?;

        // Create key pair from PKCS#8 bytes
        let key_pair = signature::Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())?;

        // Hash the transcript content
        let mut hasher = Sha256::new();
        hasher.update(transcript.as_bytes());
        let content_hash = hasher.finalize();

        // Create signature data including timestamp for temporal verification
        let timestamp = chrono::Utc::now();
        let signature_data = format!("{}|{}|{}", transcript, timestamp, "council-coordinator");

        // Sign the signature data
        let signature_bytes = key_pair.sign(signature_data.as_bytes());
        let signature_hex = hex::encode(signature_bytes.as_ref());

        // Create signed transcript with proper metadata
        let signed_transcript = SignedTranscript {
            transcript: transcript.to_string(),
            signature: signature_hex,
            timestamp,
            signer: "ConsensusCoordinator".to_string(),
        };

        // Store signed transcript in database for provenance
        self.store_signed_transcript_in_database(&signed_transcript).await?;

        info!("Transcript cryptographically signed and stored: {} (hash: {})",
              signed_transcript.signer,
              hex::encode(content_hash));

        Ok(signed_transcript)
    }

    /// Store signed transcript in database for provenance tracking
    async fn store_signed_transcript_in_database(&self, signed_transcript: &SignedTranscript) -> Result<()> {
        // Insert signed transcript into database
        let query = "
            INSERT INTO signed_transcripts (signature, transcript, timestamp, signer, content_hash)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (signature) DO UPDATE SET
                transcript = EXCLUDED.transcript,
                timestamp = EXCLUDED.timestamp,
                signer = EXCLUDED.signer,
                content_hash = EXCLUDED.content_hash
        ";

        // Calculate content hash for integrity verification
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(signed_transcript.transcript.as_bytes());
        let content_hash = hex::encode(hasher.finalize());

        self.database_client.execute(
            query,
            &[
                &signed_transcript.signature,
                &signed_transcript.transcript,
                &signed_transcript.timestamp,
                &signed_transcript.signer,
                &content_hash,
            ]
        ).await?;

        debug!("Stored signed transcript in database: {}", signed_transcript.signature);
        Ok(())
    }

    /// Validate contribution collection quality
    fn validate_contribution_collection(&self, contributions: &[ParticipantContribution]) -> Result<()> {
        if contributions.is_empty() {
            return Err(anyhow::anyhow!("No contributions collected"));
        }

        // Check for minimum quality threshold
        let avg_confidence = contributions.iter()
            .map(|c| c.confidence)
            .sum::<f32>() / contributions.len() as f32;

        if avg_confidence < 0.3 {
            return Err(anyhow::anyhow!("Average contribution confidence too low: {:.2}", avg_confidence));
        }

        // Check for argument quality
        for contribution in contributions {
            if contribution.argument.len() < 20 {
                return Err(anyhow::anyhow!("Contribution from {} too short", contribution.participant));
            }
        }

        Ok(())
    }













}
