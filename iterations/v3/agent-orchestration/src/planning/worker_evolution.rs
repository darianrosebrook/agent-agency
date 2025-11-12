//! Worker Evolution System
//!
//! Enables agents to craft and refine workers based on execution patterns and performance.
//!
//! @author @darianrosebrook

use std::sync::Arc;
use std::collections::HashMap;
use anyhow::Result;
use uuid::Uuid;
use tracing::{info, warn, debug};
use serde::{Deserialize, Serialize};

use crate::planning::reflexive_learner::LearningOutcome;
use crate::planning::DatabaseOperations;
use agent_workers::WorkerSpecialty;

/// Worker creation proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerCreationProposal {
    /// Proposed worker name
    pub proposed_name: String,
    
    /// Worker specialty
    pub specialty: WorkerSpecialty,
    
    /// Proposed capabilities (as JSON)
    pub capabilities: serde_json::Value,
    
    /// Rationale for creation
    pub rationale: String,
    
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
    
    /// Expected benefit (improvement estimate)
    pub expected_benefit: f64,
    
    /// Supporting evidence
    pub evidence: Vec<LearningOutcome>,
    
    /// Model name for the worker
    pub model_name: String,
    
    /// Endpoint for the worker
    pub endpoint: String,
}

/// Worker refinement proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerRefinementProposal {
    /// Worker ID to refine
    pub worker_id: Uuid,
    
    /// Type of refinement
    pub refinement_type: RefinementType,
    
    /// Changes to apply
    pub changes: WorkerCapabilityChanges,
    
    /// Rationale for refinement
    pub rationale: String,
    
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
    
    /// Expected benefit (improvement estimate)
    pub expected_benefit: f64,
    
    /// Supporting evidence
    pub evidence: Vec<LearningOutcome>,
}

/// Type of worker refinement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RefinementType {
    /// Add a new capability
    AddCapability { capability: String },
    
    /// Remove a capability (specialization)
    RemoveCapability { capability: String },
    
    /// Adjust performance scores
    AdjustScores {
        quality: Option<f32>,
        speed: Option<f32>,
        caws: Option<f32>,
    },
    
    /// Change worker specialty
    ChangeSpecialty { new_specialty: WorkerSpecialty },
}

/// Capability changes for worker refinement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerCapabilityChanges {
    /// Languages to add
    pub add_languages: Vec<String>,
    
    /// Languages to remove
    pub remove_languages: Vec<String>,
    
    /// Domains to add
    pub add_domains: Vec<String>,
    
    /// Domains to remove
    pub remove_domains: Vec<String>,
    
    /// Operations to add (read, write, execute, etc.)
    pub add_operations: Vec<String>,
    
    /// Operations to remove
    pub remove_operations: Vec<String>,
    
    /// Updated max context length
    pub max_context_length: Option<u32>,
    
    /// Updated max output length
    pub max_output_length: Option<u32>,
}

impl Default for WorkerCapabilityChanges {
    fn default() -> Self {
        Self {
            add_languages: Vec::new(),
            remove_languages: Vec::new(),
            add_domains: Vec::new(),
            remove_domains: Vec::new(),
            add_operations: Vec::new(),
            remove_operations: Vec::new(),
            max_context_length: None,
            max_output_length: None,
        }
    }
}

/// Configuration for worker evolution
#[derive(Debug, Clone)]
pub struct EvolutionConfig {
    /// Minimum confidence threshold for auto-creation (0.0 - 1.0)
    pub min_creation_confidence: f64,
    
    /// Minimum expected benefit for auto-creation
    pub min_creation_benefit: f64,
    
    /// Enable automatic worker creation
    pub enable_auto_creation: bool,
    
    /// Enable automatic worker refinement
    pub enable_auto_refinement: bool,
    
    /// Maximum number of workers
    pub max_workers: usize,
    
    /// Minimum performance threshold for worker retention
    pub min_performance_threshold: f64,
    
    /// Minimum outcomes required before proposing refinement
    pub min_outcomes_for_refinement: usize,
}

impl Default for EvolutionConfig {
    fn default() -> Self {
        Self {
            min_creation_confidence: 0.8,
            min_creation_benefit: 0.15, // 15% improvement
            enable_auto_creation: true,
            enable_auto_refinement: true,
            max_workers: 50,
            min_performance_threshold: 0.5,
            min_outcomes_for_refinement: 10,
        }
    }
}

/// Worker evolution engine
pub struct WorkerEvolutionEngine {
    /// Database operations for worker management
    db_ops: Arc<dyn DatabaseOperations>,
    
    /// Configuration
    config: EvolutionConfig,
    
    /// Pending creation proposals
    creation_proposals: Arc<tokio::sync::RwLock<Vec<WorkerCreationProposal>>>,
    
    /// Pending refinement proposals
    refinement_proposals: Arc<tokio::sync::RwLock<Vec<WorkerRefinementProposal>>>,
}

impl WorkerEvolutionEngine {
    /// Create new worker evolution engine
    pub fn new(db_ops: Arc<dyn DatabaseOperations>, config: EvolutionConfig) -> Self {
        Self {
            db_ops,
            config,
            creation_proposals: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            refinement_proposals: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }
    
    /// Process learning outcomes and generate proposals
    pub async fn process_outcomes(
        &self,
        outcomes: &[&LearningOutcome],
    ) -> Result<(Vec<WorkerCreationProposal>, Vec<WorkerRefinementProposal>)> {
        info!("Processing {} outcomes for worker evolution", outcomes.len());
        
        let creation_proposals = self.generate_creation_proposals(outcomes).await?;
        let refinement_proposals = self.generate_refinement_proposals(outcomes).await?;
        
        // Store proposals
        {
            let mut proposals = self.creation_proposals.write().await;
            proposals.extend(creation_proposals.clone());
        }
        
        {
            let mut proposals = self.refinement_proposals.write().await;
            proposals.extend(refinement_proposals.clone());
        }
        
        Ok((creation_proposals, refinement_proposals))
    }
    
    /// Generate worker creation proposals based on patterns
    async fn generate_creation_proposals(
        &self,
        outcomes: &[&LearningOutcome],
    ) -> Result<Vec<WorkerCreationProposal>> {
        let mut proposals = Vec::new();
        
        // Group outcomes by task characteristics
        let mut task_groups: HashMap<String, Vec<&LearningOutcome>> = HashMap::new();
        for outcome in outcomes {
            let key = format!("{}:{}", outcome.task_characteristics.task_type, 
                            outcome.task_characteristics.required_capabilities.join(","));
            task_groups.entry(key).or_insert_with(Vec::new).push(outcome);
        }
        
        // Check for patterns that suggest need for specialized worker
        for (task_key, group_outcomes) in task_groups {
            if group_outcomes.len() < self.config.min_outcomes_for_refinement {
                continue;
            }
            
            // Check if suitable worker exists
            let required_capabilities: Vec<String> = group_outcomes[0]
                .task_characteristics.required_capabilities.clone();
            
            let workers = self.db_ops.get_workers().await?;
            let has_suitable_worker = workers.iter().any(|w| {
                let worker_caps: &serde_json::Value = &w.capabilities;
                required_capabilities.iter().all(|cap| {
                    worker_caps.get(cap).and_then(|v| v.as_bool()).unwrap_or(false)
                })
            });
            
            debug!("Task group '{}' with {} outcomes: has_suitable_worker={}, required_caps={:?}", 
                   task_key, group_outcomes.len(), has_suitable_worker, required_capabilities);
            
            if has_suitable_worker {
                debug!("Skipping proposal - suitable worker already exists");
                continue; // Worker already exists
            }
            
            // Calculate average quality for this task type
            let avg_quality: f64 = group_outcomes.iter()
                .map(|o| o.quality_score)
                .sum::<f64>() / group_outcomes.len() as f64;
            
            // Calculate success rate
            let success_rate = group_outcomes.iter()
                .filter(|o| o.success)
                .count() as f64 / group_outcomes.len() as f64;
            
            debug!("Pattern analysis: success_rate={:.2}, avg_quality={:.2}, outcomes={}", 
                   success_rate, avg_quality, group_outcomes.len());
            
            // Generate proposal if pattern is strong
            if success_rate > 0.6 && avg_quality > 0.65 {
                let task_type = &group_outcomes[0].task_characteristics.task_type;
                let specialty = self.infer_specialty_from_task_type(task_type);
                
                let capabilities = self.build_capabilities_from_outcomes(&group_outcomes);
                
                let confidence = (success_rate * 0.5 + avg_quality * 0.5).min(1.0);
                let expected_benefit = (avg_quality - 0.65).max(0.0); // Improvement over baseline
                
                if confidence >= self.config.min_creation_confidence &&
                   expected_benefit >= self.config.min_creation_benefit {
                    proposals.push(WorkerCreationProposal {
                        proposed_name: format!("{} Specialist", task_type),
                        specialty,
                        capabilities,
                        rationale: format!(
                            "Detected {} tasks with {}% success rate and {:.2} avg quality. \
                             No suitable worker exists.",
                            group_outcomes.len(),
                            success_rate * 100.0,
                            avg_quality
                        ),
                        confidence,
                        expected_benefit,
                        evidence: group_outcomes.iter().map(|o| (**o).clone()).collect(),
                        model_name: "adaptive-model".to_string(),
                        endpoint: "http://localhost:8000".to_string(),
                    });
                }
            }
        }
        
        Ok(proposals)
    }
    
    /// Generate worker refinement proposals based on performance
    async fn generate_refinement_proposals(
        &self,
        outcomes: &[&LearningOutcome],
    ) -> Result<Vec<WorkerRefinementProposal>> {
        let mut proposals = Vec::new();
        
        // Group outcomes by worker
        let mut worker_outcomes: HashMap<Uuid, Vec<&LearningOutcome>> = HashMap::new();
        for outcome in outcomes {
            worker_outcomes.entry(outcome.worker_id)
                .or_insert_with(Vec::new)
                .push(outcome);
        }
        
        // Analyze each worker's performance
        for (worker_id, worker_outcomes) in worker_outcomes {
            if worker_outcomes.len() < self.config.min_outcomes_for_refinement {
                continue;
            }
            
            let worker = match self.db_ops.get_worker(worker_id).await {
                Ok(Some(w)) => w,
                Ok(None) | Err(_) => continue, // Worker not found, skip
            };
            
            // Check for capability gaps
            let mut missing_capabilities = Vec::new();
            for outcome in &worker_outcomes {
                for required_cap in &outcome.task_characteristics.required_capabilities {
                    let worker_caps: &serde_json::Value = &worker.capabilities;
                    if !worker_caps.get(required_cap).and_then(|v| v.as_bool()).unwrap_or(false) {
                        if !missing_capabilities.contains(required_cap) {
                            missing_capabilities.push(required_cap.clone());
                        }
                    }
                }
            }
            
            // Propose adding frequently missing capabilities
            for cap in missing_capabilities {
                let cap_usage_count = worker_outcomes.iter()
                    .filter(|o| o.task_characteristics.required_capabilities.contains(&cap))
                    .count();
                
                debug!("Capability '{}' used in {} tasks for worker {}", cap, cap_usage_count, worker_id);
                
                if cap_usage_count >= 5 { // Used in at least 5 tasks
                    let success_rate = worker_outcomes.iter()
                        .filter(|o| o.task_characteristics.required_capabilities.contains(&cap) && o.success)
                        .count() as f64 / cap_usage_count as f64;
                    
                    debug!("Capability '{}' success rate: {:.2}%", cap, success_rate * 100.0);
                    
                    if success_rate > 0.7 {
                        let mut changes = WorkerCapabilityChanges::default();
                        changes.add_operations.push(cap.clone());
                        
                        proposals.push(WorkerRefinementProposal {
                            worker_id,
                            refinement_type: RefinementType::AddCapability {
                                capability: cap.clone(),
                            },
                            changes,
                            rationale: format!(
                                "Worker successfully handled {} tasks requiring '{}' capability \
                                 ({}% success rate). Adding capability.",
                                cap_usage_count,
                                cap,
                                success_rate * 100.0
                            ),
                            confidence: success_rate.min(0.95),
                            expected_benefit: 0.1, // 10% improvement
                            evidence: worker_outcomes.iter()
                                .filter(|o| o.task_characteristics.required_capabilities.contains(&cap))
                                .map(|o| (**o).clone())
                                .collect(),
                        });
                    }
                }
            }
        }
        
        Ok(proposals)
    }
    
    /// Evaluate and execute approved proposals
    pub async fn evaluate_and_execute(&self) -> Result<EvolutionResults> {
        let mut results = EvolutionResults::default();
        
        // Check worker count limit
        let workers = self.db_ops.get_workers().await?;
        let active_worker_count = workers.iter().filter(|w| w.is_active).count();
        
        if active_worker_count >= self.config.max_workers {
            warn!("Worker limit reached ({}), skipping creation proposals", active_worker_count);
        } else {
            // Evaluate creation proposals
            let creation_proposals = {
                let proposals = self.creation_proposals.read().await;
                proposals.clone()
            };
            
            for proposal in creation_proposals {
                if self.should_approve_creation(&proposal, active_worker_count).await? {
                    match self.create_worker_from_proposal(&proposal).await {
                        Ok(worker) => {
                            info!("Created worker from proposal: {}", worker.name);
                            results.workers_created += 1;
                        }
                        Err(e) => {
                            warn!("Failed to create worker from proposal: {}", e);
                            results.creation_failures += 1;
                        }
                    }
                }
            }
        }
        
        // Evaluate refinement proposals
        if self.config.enable_auto_refinement {
            let refinement_proposals = {
                let proposals = self.refinement_proposals.read().await;
                proposals.clone()
            };
            
            for proposal in refinement_proposals {
                if self.should_approve_refinement(&proposal).await? {
                    match self.refine_worker_from_proposal(&proposal).await {
                        Ok(_) => {
                            info!("Refined worker {} from proposal", proposal.worker_id);
                            results.workers_refined += 1;
                        }
                        Err(e) => {
                            warn!("Failed to refine worker from proposal: {}", e);
                            results.refinement_failures += 1;
                        }
                    }
                }
            }
        }
        
        // Clear processed proposals
        {
            let mut proposals = self.creation_proposals.write().await;
            proposals.clear();
        }
        
        {
            let mut proposals = self.refinement_proposals.write().await;
            proposals.clear();
        }
        
        Ok(results)
    }
    
    /// Check if creation proposal should be approved
    async fn should_approve_creation(
        &self,
        proposal: &WorkerCreationProposal,
        current_worker_count: usize,
    ) -> Result<bool> {
        if !self.config.enable_auto_creation {
            return Ok(false);
        }
        
        if current_worker_count >= self.config.max_workers {
            return Ok(false);
        }
        
        Ok(proposal.confidence >= self.config.min_creation_confidence &&
           proposal.expected_benefit >= self.config.min_creation_benefit)
    }
    
    /// Check if refinement proposal should be approved
    async fn should_approve_refinement(
        &self,
        proposal: &WorkerRefinementProposal,
    ) -> Result<bool> {
        if !self.config.enable_auto_refinement {
            return Ok(false);
        }
        
        Ok(proposal.confidence >= 0.7 && proposal.expected_benefit >= 0.05)
    }
    
    /// Create worker from approved proposal
    async fn create_worker_from_proposal(
        &self,
        proposal: &WorkerCreationProposal,
    ) -> Result<crate::planning::models::Worker> {
        use crate::planning::data_infrastructure_types::CreateWorker;
        
        let worker = CreateWorker {
            name: proposal.proposed_name.clone(),
            worker_type: "mcp".to_string(),
            specialty: Some(format!("{:?}", proposal.specialty)),
            model_name: proposal.model_name.clone(),
            endpoint: proposal.endpoint.clone(),
            capabilities: proposal.capabilities.clone(),
            performance_history: serde_json::json!({}),
            is_active: true,
        };
        
        self.db_ops.create_worker(worker).await
    }
    
    /// Refine worker from approved proposal
    async fn refine_worker_from_proposal(
        &self,
        proposal: &WorkerRefinementProposal,
    ) -> Result<()> {
        let mut worker = self.db_ops.get_worker(proposal.worker_id).await?
            .ok_or_else(|| anyhow::anyhow!("Worker {} not found", proposal.worker_id))?;
        let mut capabilities: serde_json::Value = worker.capabilities.clone();
        
        // Apply changes based on refinement type
        match &proposal.refinement_type {
            RefinementType::AddCapability { capability } => {
                capabilities[capability] = serde_json::json!(true);
            }
            RefinementType::RemoveCapability { capability } => {
                capabilities[capability] = serde_json::json!(false);
            }
            RefinementType::AdjustScores { quality, speed, caws } => {
                if let Some(q) = quality {
                    capabilities["quality_score"] = serde_json::json!(*q);
                }
                if let Some(s) = speed {
                    capabilities["speed_score"] = serde_json::json!(*s);
                }
                if let Some(c) = caws {
                    capabilities["caws_awareness"] = serde_json::json!(*c);
                }
            }
            RefinementType::ChangeSpecialty { new_specialty } => {
                worker.specialty = Some(format!("{:?}", new_specialty));
            }
        }
        
        // Apply capability changes
        for lang in &proposal.changes.add_languages {
            if let Some(languages) = capabilities.get_mut("languages") {
                if let Some(lang_array) = languages.as_array_mut() {
                    if !lang_array.iter().any(|l| l.as_str() == Some(lang)) {
                        lang_array.push(serde_json::json!(lang));
                    }
                }
            }
        }
        
        for op in &proposal.changes.add_operations {
            capabilities[op] = serde_json::json!(true);
        }
        
        for op in &proposal.changes.remove_operations {
            capabilities[op] = serde_json::json!(false);
        }
        
        if let Some(max_ctx) = proposal.changes.max_context_length {
            capabilities["max_context_length"] = serde_json::json!(max_ctx);
        }
        
        if let Some(max_out) = proposal.changes.max_output_length {
            capabilities["max_output_length"] = serde_json::json!(max_out);
        }
        
        worker.capabilities = capabilities;
        
        // Update worker in database
        use crate::planning::data_infrastructure_types::UpdateWorker;
        let update = UpdateWorker {
            name: None,
            worker_type: None,
            specialty: worker.specialty.clone(),
            model_name: None,
            endpoint: None,
            capabilities: Some(worker.capabilities.clone()),
            performance_history: None,
            is_active: None,
        };
        
        self.db_ops.update_worker(proposal.worker_id, update).await?;
        
        Ok(())
    }
    
    /// Infer worker specialty from task type
    fn infer_specialty_from_task_type(&self, task_type: &str) -> WorkerSpecialty {
        match task_type.to_lowercase().as_str() {
            t if t.contains("react") => WorkerSpecialty::ReactComponent,
            t if t.contains("file") || t.contains("edit") => WorkerSpecialty::FileEditing,
            t if t.contains("test") => WorkerSpecialty::Testing { frameworks: vec![] },
            t if t.contains("doc") => WorkerSpecialty::Documentation { formats: vec![] },
            t if t.contains("refactor") => WorkerSpecialty::Refactoring { patterns: vec![] },
            t if t.contains("compile") => WorkerSpecialty::Compilation,
            t if t.contains("security") => WorkerSpecialty::Security,
            t if t.contains("performance") => WorkerSpecialty::Performance,
            t if t.contains("code") || t.contains("generate") => WorkerSpecialty::CodeGeneration,
            _ => WorkerSpecialty::General,
        }
    }
    
    /// Build capabilities JSON from learning outcomes
    fn build_capabilities_from_outcomes(
        &self,
        outcomes: &[&LearningOutcome],
    ) -> serde_json::Value {
        let mut capabilities = serde_json::json!({
            "languages": [],
            "domains": [],
            "max_context_length": 8192,
            "max_output_length": 4096,
        });
        
        // Extract common capabilities from outcomes
        let mut all_caps: std::collections::HashSet<String> = std::collections::HashSet::new();
        for outcome in outcomes {
            all_caps.extend(outcome.task_characteristics.required_capabilities.clone());
        }
        
        // Add operations
        for cap in &all_caps {
            capabilities[cap] = serde_json::json!(true);
        }
        
        // Infer languages and domains from task type
        if let Some(first_outcome) = outcomes.first() {
            let task_type = &first_outcome.task_characteristics.task_type;
            if task_type.contains("typescript") || task_type.contains("ts") {
                capabilities["languages"].as_array_mut().unwrap().push(serde_json::json!("typescript"));
            }
            if task_type.contains("rust") {
                capabilities["languages"].as_array_mut().unwrap().push(serde_json::json!("rust"));
            }
            if task_type.contains("python") {
                capabilities["languages"].as_array_mut().unwrap().push(serde_json::json!("python"));
            }
        }
        
        capabilities
    }
}

/// Results from evolution evaluation
#[derive(Debug, Default)]
pub struct EvolutionResults {
    pub workers_created: usize,
    pub workers_refined: usize,
    pub creation_failures: usize,
    pub refinement_failures: usize,
}

