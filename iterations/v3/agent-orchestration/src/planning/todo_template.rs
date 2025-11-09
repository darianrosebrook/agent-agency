//! TODO Template System - Dependency Tracking with Quality Gate Enforcement
//!
//! Templates for structured TODO tracking with dependency management.
//! Prevents quality bypass by enforcing completion requirements.
//!
//! @author @darianrosebrook

use std::collections::{HashMap, HashSet};
use anyhow::{anyhow, Result};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// TODO template with dependency tracking
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TodoTemplate {
    /// Unique template identifier
    #[schemars(with = "String")]
    pub id: Uuid,

    /// Template name
    pub name: String,

    /// Template description
    pub description: String,

    /// Template version
    pub version: String,

    /// Risk tier this template applies to
    pub risk_tier: u8,

    /// Template metadata
    pub metadata: HashMap<String, serde_json::Value>,

    /// Steps in the template
    pub steps: Vec<TodoStep>,

    /// Dependencies between steps
    pub dependencies: Vec<TodoDependency>,

    /// Quality gates that cannot be bypassed
    pub quality_gates: Vec<String>,

    /// Created timestamp
    #[schemars(with = "String")]

    pub created_at: DateTime<Utc>,

    /// Last updated timestamp
    #[schemars(with = "String")]

    pub updated_at: DateTime<Utc>,
}

/// Individual TODO step in a template
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TodoStep {
    /// Step identifier
    pub id: String,

    /// Step title
    pub title: String,

    /// Step description
    pub description: String,

    /// Step priority
    #[schemars(with = "String")]
    pub priority: TodoPriority,

    /// Estimated effort (hours)
    pub estimated_hours: f64,

    /// Required capabilities
    pub required_capabilities: Vec<String>,

    /// Quality requirements
    pub quality_requirements: Vec<String>,

    /// Acceptance criteria
    pub acceptance_criteria: Vec<String>,

    /// Step type
    #[schemars(with = "String")]
    pub step_type: TodoStepType,

    /// Step metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Step priority levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum TodoPriority {
    Critical,
    High,
    Medium,
    Low,
}

/// Step types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum TodoStepType {
    Analysis,
    Design,
    Implementation,
    Testing,
    Documentation,
    Review,
    Deployment,
}

/// Dependency between TODO steps
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TodoDependency {
    /// Dependent step ID
    pub from_step: String,

    /// Required step ID
    pub to_step: String,

    /// Dependency type
    #[schemars(with = "String")]
    pub dependency_type: DependencyType,

    /// Dependency strength (0.0-1.0)
    pub strength: f64,

    /// Optional dependency (can be skipped)
    pub optional: bool,
}

/// Dependency types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum DependencyType {
    /// Must complete before starting
    Hard,

    /// Should complete before starting
    Soft,

    /// Can run in parallel but affects quality
    Parallel,

    /// Must complete before this step is considered done
    FinishToFinish,
}

/// Active TODO instance
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TodoInstance {
    /// Instance ID
    #[schemars(with = "String")]
    pub id: Uuid,

    /// Template ID this instance is based on
    #[schemars(with = "String")]
    pub template_id: Uuid,

    /// Plan ID this TODO instance is associated with.
    /// Links the TODO instance to a specific execution plan, allowing tracking
    /// of TODO progress within the context of a larger planning workflow.
    #[schemars(with = "String")]
    pub plan_id: Uuid,

    /// Milestone ID this TODO is associated with (optional).
    /// If provided, associates this TODO instance with a specific milestone
    /// within the execution plan, enabling milestone-level progress tracking
    /// and dependency management across multiple TODOs in the same milestone.
    pub milestone_id: Option<String>,

    /// Current step being worked on
    pub current_step: Option<String>,

    /// Completed steps
    pub completed_steps: HashSet<String>,

    /// Blocked steps and reasons
    pub blocked_steps: HashMap<String, String>,

    /// Step statuses
    pub step_statuses: HashMap<String, TodoStepStatus>,

    /// Quality gate verification results
    pub quality_verifications: HashMap<String, QualityVerification>,

    /// Instance metadata
    pub metadata: HashMap<String, serde_json::Value>,

    /// Created timestamp
    #[schemars(with = "String")]

    pub created_at: DateTime<Utc>,

    /// Last updated timestamp
    #[schemars(with = "String")]

    pub updated_at: DateTime<Utc>,
}

/// Step status in a TODO instance
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TodoStepStatus {
    /// Step ID
    pub step_id: String,

    /// Current status
    #[schemars(with = "String")]
    pub status: StepStatus,

    /// Assigned worker ID
    pub assigned_worker: Option<String>,

    /// Started timestamp
    #[schemars(with = "String")]
    pub started_at: Option<DateTime<Utc>>,

    /// Completed timestamp
    #[schemars(with = "String")]
    pub completed_at: Option<DateTime<Utc>>,

    /// Progress percentage (0-100)
    pub progress: u8,

    /// Notes and comments
    pub notes: Vec<String>,

    /// Quality verification results
    pub quality_results: Vec<QualityResult>,
}

/// Step status types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum StepStatus {
    /// Not yet started
    Pending,

    /// Currently being worked on
    InProgress,

    /// Completed successfully
    Completed,

    /// Blocked by dependency or issue
    Blocked,

    /// Cancelled or skipped
    Cancelled,

    /// Failed quality verification
    Failed,
}

/// Quality verification result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QualityResult {
    /// Gate that was verified
    pub gate: String,

    /// Verification result
    pub result: bool,

    /// Verification details
    pub details: String,

    /// Verified timestamp
    #[schemars(with = "String")]

    pub verified_at: DateTime<Utc>,

    /// Verified by
    pub verified_by: String,
}

/// Quality verification status
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QualityVerification {
    /// Gate name
    pub gate: String,

    /// Verification required
    pub required: bool,

    /// Verification completed
    pub completed: bool,

    /// Verification result
    pub result: Option<bool>,

    /// Last verified timestamp
    #[schemars(with = "String")]
    pub last_verified: Option<DateTime<Utc>>,

    /// Verification attempts
    pub attempts: u32,
}

/// TODO template system

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TodoTemplateSystem {
    /// Available templates
    templates: HashMap<String, TodoTemplate>,

    /// Active TODO instances
    active_instances: HashMap<Uuid, TodoInstance>,

    /// Quality gate enforcer
    quality_enforcer: QualityGateEnforcer,
}

/// Quality gate enforcer

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct QualityGateEnforcer {
    /// Enforced gates that cannot be bypassed
    enforced_gates: HashSet<String>,
}

impl TodoTemplateSystem {
    /// Create new TODO template system
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
            active_instances: HashMap::new(),
            quality_enforcer: QualityGateEnforcer::new(),
        }
    }

    /// Register a TODO template
    pub fn register_template(&mut self, template: TodoTemplate) -> Result<()> {
        // Validate template
        self.validate_template(&template)?;

        // Register template
        self.templates.insert(template.name.clone(), template);
        Ok(())
    }

    /// Create TODO instance from template
    pub fn create_instance(&mut self, template_name: &str, plan_id: Uuid, milestone_id: Option<String>) -> Result<Uuid> {
        let template = self.templates.get(template_name)
            .ok_or_else(|| anyhow!("Template '{}' not found", template_name))?;

        let instance = TodoInstance {
            id: Uuid::new_v4(),
            template_id: template.id,
            plan_id,
            milestone_id: milestone_id.clone(),
            current_step: None,
            completed_steps: HashSet::new(),
            blocked_steps: HashMap::new(),
            step_statuses: self.initialize_step_statuses(template),
            quality_verifications: self.initialize_quality_verifications(template),
            metadata: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let instance_id = instance.id;
        self.active_instances.insert(instance_id, instance);

        Ok(instance_id)
    }

    /// Start working on a TODO step
    pub async fn start_step(&mut self, instance_id: Uuid, step_id: &str, worker_id: Option<String>) -> Result<()> {
        // Check dependencies and quality gates before getting mutable reference
        let instance = self.active_instances.get(&instance_id)
            .ok_or_else(|| anyhow!("Instance {} not found", instance_id))?;
        
        // Check if step can be started (dependencies satisfied)
        if !self.can_start_step(instance, step_id)? {
            return Err(anyhow!("Cannot start step '{}': dependencies not satisfied", step_id));
        }

        // Check quality gates for prerequisites
        if !self.quality_enforcer.can_start_step(instance, step_id)? {
            return Err(anyhow!("Cannot start step '{}': quality gates not satisfied", step_id));
        }
        
        // Now get mutable reference to update the instance
        let instance = self.active_instances.get_mut(&instance_id)
            .ok_or_else(|| anyhow!("Instance {} not found", instance_id))?;

        // Update step status
        if let Some(status) = instance.step_statuses.get_mut(step_id) {
            status.status = StepStatus::InProgress;
            status.started_at = Some(Utc::now());
            status.assigned_worker = worker_id;
        }

        instance.current_step = Some(step_id.to_string());
        instance.updated_at = Utc::now();

        Ok(())
    }

    /// Complete a TODO step
    pub async fn complete_step(&mut self, instance_id: Uuid, step_id: &str, notes: Option<String>) -> Result<()> {
        // Get template info first (immutable borrow)
        let instance_info = self.active_instances.get(&instance_id)
            .ok_or_else(|| anyhow!("Instance {} not found", instance_id))?;
        let template = self.get_template_for_instance(instance_info)?;
        let total_steps = template.steps.len();

        // Now get mutable instance
        let instance = self.active_instances.get_mut(&instance_id).unwrap();

        // Verify quality gates are satisfied
        if !self.quality_enforcer.verify_step_completion(instance, step_id).await? {
            return Err(anyhow!("Cannot complete step '{}': quality gates failed", step_id));
        }

        // Update step status
        if let Some(status) = instance.step_statuses.get_mut(step_id) {
            status.status = StepStatus::Completed;
            status.completed_at = Some(Utc::now());
            status.progress = 100;

            if let Some(note) = notes {
                status.notes.push(note);
            }
        }

        instance.completed_steps.insert(step_id.to_string());
        instance.updated_at = Utc::now();

        // Check if instance is complete
        let completed_count = instance.completed_steps.len();

        if completed_count >= total_steps {
            instance.current_step = None;
            // Could add completion timestamp, final verification, etc.
        }

        Ok(())
    }

    /// Fail a TODO step
    pub async fn fail_step(&mut self, instance_id: Uuid, step_id: &str, reason: &str) -> Result<()> {
        let instance = self.active_instances.get_mut(&instance_id)
            .ok_or_else(|| anyhow!("Instance {} not found", instance_id))?;

        if let Some(status) = instance.step_statuses.get_mut(step_id) {
            status.status = StepStatus::Failed;
            status.notes.push(format!("Failed: {}", reason));
        }

        instance.updated_at = Utc::now();

        Ok(())
    }

    /// Check if a step can be started
    pub fn can_start_step(&self, instance: &TodoInstance, step_id: &str) -> Result<bool> {
        let template = self.get_template_for_instance(instance)?;

        // Find dependencies for this step
        let dependencies: Vec<_> = template.dependencies.iter()
            .filter(|dep| dep.from_step == step_id)
            .collect();

        for dep in dependencies {
            if dep.optional {
                continue; // Optional dependencies don't block
            }

            match dep.dependency_type {
                DependencyType::Hard => {
                    // Must be completed
                    if !instance.completed_steps.contains(&dep.to_step) {
                        return Ok(false);
                    }
                }
                DependencyType::Soft => {
                    // Should be completed but not required
                    // Allow but with warning
                }
                DependencyType::Parallel => {
                    // Can run in parallel, no blocking
                }
                DependencyType::FinishToFinish => {
                    // Must complete before this finishes
                    if instance.completed_steps.contains(&dep.to_step) {
                        return Ok(false);
                    }
                }
            }
        }

        Ok(true)
    }

    /// Get next available steps
    pub fn get_next_steps(&self, instance: &TodoInstance) -> Result<Vec<String>> {
        let template = self.get_template_for_instance(instance)?;
        let mut available_steps = Vec::new();

        for step in &template.steps {
            // Skip already completed or in progress steps
            if instance.completed_steps.contains(&step.id) {
                continue;
            }

            if let Some(current) = &instance.current_step {
                if current == &step.id {
                    continue;
                }
            }

            // Check if step is blocked
            if instance.blocked_steps.contains_key(&step.id) {
                continue;
            }

            // Check dependencies
            if self.can_start_step(instance, &step.id)? {
                available_steps.push(step.id.clone());
            }
        }

        Ok(available_steps)
    }

    /// Get instance progress
    pub fn get_instance_progress(&self, instance: &TodoInstance) -> Result<TodoProgress> {
        let template = self.get_template_for_instance(instance)?;
        let total_steps = template.steps.len();
        let completed_steps = instance.completed_steps.len();
        let in_progress_steps = instance.step_statuses.values()
            .filter(|s| s.status == StepStatus::InProgress)
            .count();

        let overall_progress = if total_steps > 0 {
            (completed_steps as f64 / total_steps as f64) * 100.0
        } else {
            100.0
        };

        Ok(TodoProgress {
            total_steps,
            completed_steps,
            in_progress_steps,
            blocked_steps: instance.blocked_steps.len(),
            overall_progress,
        })
    }

    /// Validate template structure
    fn validate_template(&self, template: &TodoTemplate) -> Result<()> {
        // Check for duplicate step IDs
        let mut step_ids = HashSet::new();
        for step in &template.steps {
            if !step_ids.insert(step.id.clone()) {
                return Err(anyhow!("Duplicate step ID: {}", step.id));
            }
        }

        // Check dependencies reference valid steps
        for dep in &template.dependencies {
            if !step_ids.contains(&dep.from_step) {
                return Err(anyhow!("Dependency references invalid step: {}", dep.from_step));
            }
            if !step_ids.contains(&dep.to_step) {
                return Err(anyhow!("Dependency references invalid step: {}", dep.to_step));
            }
        }

        // Check for cycles in dependencies
        if self.has_circular_dependencies(template) {
            return Err(anyhow!("Template contains circular dependencies"));
        }

        Ok(())
    }

    /// Check for circular dependencies in template
    fn has_circular_dependencies(&self, template: &TodoTemplate) -> bool {
        let mut graph = HashMap::new();

        // Build adjacency list
        for dep in &template.dependencies {
            graph.entry(dep.from_step.clone())
                .or_insert_with(Vec::new)
                .push(dep.to_step.clone());
        }

        // Check for cycles using DFS
        let mut visited = HashSet::new();
        let mut recursion_stack = HashSet::new();

        for step_id in template.steps.iter().map(|s| s.id.clone()) {
            if self.has_cycle(&graph, &step_id, &mut visited, &mut recursion_stack) {
                return true;
            }
        }

        false
    }

    fn has_cycle(
        &self,
        graph: &HashMap<String, Vec<String>>,
        step_id: &str,
        visited: &mut HashSet<String>,
        recursion_stack: &mut HashSet<String>,
    ) -> bool {
        if recursion_stack.contains(step_id) {
            return true;
        }

        if visited.contains(step_id) {
            return false;
        }

        visited.insert(step_id.to_string());
        recursion_stack.insert(step_id.to_string());

        if let Some(neighbors) = graph.get(step_id) {
            for neighbor in neighbors {
                if self.has_cycle(graph, neighbor, visited, recursion_stack) {
                    return true;
                }
            }
        }

        recursion_stack.remove(step_id);
        false
    }

    /// Get template for instance
    pub fn get_template_for_instance(&self, instance: &TodoInstance) -> Result<&TodoTemplate> {
        // Find template by ID
        for template in self.templates.values() {
            if template.id == instance.template_id {
                return Ok(template);
            }
        }

        Err(anyhow!("Template not found for instance"))
    }

    /// Initialize step statuses for new instance
    fn initialize_step_statuses(&self, template: &TodoTemplate) -> HashMap<String, TodoStepStatus> {
        let mut statuses = HashMap::new();

        for step in &template.steps {
            statuses.insert(step.id.clone(), TodoStepStatus {
                step_id: step.id.clone(),
                status: StepStatus::Pending,
                assigned_worker: None,
                started_at: None,
                completed_at: None,
                progress: 0,
                notes: vec![],
                quality_results: vec![],
            });
        }

        statuses
    }

    /// Initialize quality verifications
    fn initialize_quality_verifications(&self, template: &TodoTemplate) -> HashMap<String, QualityVerification> {
        let mut verifications = HashMap::new();

        for gate in &template.quality_gates {
            verifications.insert(gate.clone(), QualityVerification {
                gate: gate.clone(),
                required: true,
                completed: false,
                result: None,
                last_verified: None,
                attempts: 0,
            });
        }

        verifications
    }

    /// Check if instance is complete
    fn is_instance_complete(&self, instance: &TodoInstance) -> bool {
        let template = self.get_template_for_instance(instance).unwrap();
        let total_steps = template.steps.len();
        let completed_count = instance.completed_steps.len();

        completed_count >= total_steps
    }

    /// Mark instance as complete
    fn mark_instance_complete(&self, instance: &mut TodoInstance) {
        instance.current_step = None;
        // Could add completion timestamp, final verification, etc.
    }

    /// Get TODO instance by ID
    pub fn get_instance(&self, instance_id: Uuid) -> Result<&TodoInstance> {
        self.active_instances
            .get(&instance_id)
            .ok_or_else(|| anyhow!("TODO instance {} not found", instance_id))
    }

    /// Get TODO instance by ID (mutable)
    pub fn get_instance_mut(&mut self, instance_id: Uuid) -> Result<&mut TodoInstance> {
        self.active_instances
            .get_mut(&instance_id)
            .ok_or_else(|| anyhow!("TODO instance {} not found", instance_id))
    }

    /// Get TODO instance by plan ID
    pub fn get_instance_by_plan_id(&self, plan_id: Uuid) -> Result<&TodoInstance> {
        self.active_instances
            .values()
            .find(|instance| instance.plan_id == plan_id)
            .ok_or_else(|| anyhow!("No TODO instance found for plan {}", plan_id))
    }

    /// Get TODO instance by plan ID (mutable)
    pub fn get_instance_by_plan_id_mut(&mut self, plan_id: Uuid) -> Result<&mut TodoInstance> {
        self.active_instances
            .values_mut()
            .find(|instance| instance.plan_id == plan_id)
            .ok_or_else(|| anyhow!("No TODO instance found for plan {}", plan_id))
    }

    /// Check if step dependencies are satisfied for a milestone
    pub fn can_progress_to_milestone_step(&self, instance: &TodoInstance, step_id: &str) -> Result<bool> {
        // Check if step can be started (dependencies satisfied)
        self.can_start_step(instance, step_id)?;

        // Check quality gates
        self.quality_enforcer.can_start_step(instance, step_id)
    }

    /// Get blocking reasons for a step
    pub fn get_blocking_reasons(&self, instance: &TodoInstance, step_id: &str) -> Vec<String> {
        let mut reasons = Vec::new();

        // Check if step is explicitly blocked
        if let Some(reason) = instance.blocked_steps.get(step_id) {
            reasons.push(format!("Step blocked: {}", reason));
        }

        // Check dependencies
        if let Ok(template) = self.get_template_for_instance(instance) {
            for dep in &template.dependencies {
                if dep.from_step == step_id && !dep.optional {
                    if !instance.completed_steps.contains(&dep.to_step) {
                        reasons.push(format!(
                            "Dependency not satisfied: {} depends on {}",
                            step_id, dep.to_step
                        ));
                    }
                }
            }
        }

        reasons
    }
}

impl QualityGateEnforcer {
    /// Create new quality gate enforcer
    pub fn new() -> Self {
        let mut enforced_gates = HashSet::new();

        // Core quality gates that cannot be bypassed
        enforced_gates.insert("test_coverage".to_string());
        enforced_gates.insert("security_scan".to_string());
        enforced_gates.insert("type_check".to_string());
        enforced_gates.insert("lint_check".to_string());
        enforced_gates.insert("contract_validation".to_string());
        enforced_gates.insert("performance_budget".to_string());

        Self { enforced_gates }
    }

    /// Check if step can be started (quality gates satisfied)
    pub fn can_start_step(&self, instance: &TodoInstance, step_id: &str) -> Result<bool> {
        // Check if step has required quality prerequisites
        if let Some(verification) = instance.quality_verifications.get("test_coverage") {
            if !verification.completed || verification.result != Some(true) {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Verify step completion quality gates
    pub async fn verify_step_completion(&self, instance: &TodoInstance, step_id: &str) -> Result<bool> {
        // Check quality verification results stored in the step status
        if let Some(step_status) = instance.step_statuses.get(step_id) {
            // Verify all quality results for this step
            for quality_result in &step_status.quality_results {
                if let Some(gate_name) = quality_result.gate.strip_prefix("gate_") {
                    if self.enforced_gates.contains(gate_name) {
                        if !quality_result.result {
                            return Ok(false);
                        }
                    }
                }
            }
        }

        // Check instance-level quality verifications
        for (gate_name, verification) in &instance.quality_verifications {
            if self.enforced_gates.contains(gate_name) && verification.required {
                if !verification.completed || verification.result != Some(true) {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }
}

/// Progress information for TODO instance
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TodoProgress {
    pub total_steps: usize,
    pub completed_steps: usize,
    pub in_progress_steps: usize,
    pub blocked_steps: usize,
    pub overall_progress: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_agency_contracts::planning_io::MilestoneScope;

    #[test]
    fn test_template_validation() {
        let system = TodoTemplateSystem::new();

        // Valid template
        let template = TodoTemplate {
            id: Uuid::new_v4(),
            name: "test-template".to_string(),
            description: "Test template".to_string(),
            version: "1.0.0".to_string(),
            risk_tier: 2,
            metadata: HashMap::new(),
            steps: vec![
                TodoStep {
                    id: "step1".to_string(),
                    title: "Step 1".to_string(),
                    description: "First step".to_string(),
                    priority: TodoPriority::High,
                    estimated_hours: 2.0,
                    required_capabilities: vec!["analysis".to_string()],
                    quality_requirements: vec![],
                    acceptance_criteria: vec!["Step 1 complete".to_string()],
                    step_type: TodoStepType::Analysis,
                    metadata: HashMap::new(),
                },
            ],
            dependencies: vec![],
            quality_gates: vec!["test_coverage".to_string()],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert!(system.validate_template(&template).is_ok());
    }

    #[test]
    fn test_circular_dependency_detection() {
        let system = TodoTemplateSystem::new();

        // Template with circular dependency
        let template = TodoTemplate {
            id: Uuid::new_v4(),
            name: "circular-template".to_string(),
            description: "Template with circular dependency".to_string(),
            version: "1.0.0".to_string(),
            risk_tier: 2,
            metadata: HashMap::new(),
            steps: vec![
                TodoStep {
                    id: "step1".to_string(),
                    title: "Step 1".to_string(),
                    description: "First step".to_string(),
                    priority: TodoPriority::High,
                    estimated_hours: 1.0,
                    required_capabilities: vec![],
                    quality_requirements: vec![],
                    acceptance_criteria: vec![],
                    step_type: TodoStepType::Analysis,
                    metadata: HashMap::new(),
                },
                TodoStep {
                    id: "step2".to_string(),
                    title: "Step 2".to_string(),
                    description: "Second step".to_string(),
                    priority: TodoPriority::High,
                    estimated_hours: 1.0,
                    required_capabilities: vec![],
                    quality_requirements: vec![],
                    acceptance_criteria: vec![],
                    step_type: TodoStepType::Implementation,
                    metadata: HashMap::new(),
                },
            ],
            dependencies: vec![
                TodoDependency {
                    from_step: "step1".to_string(),
                    to_step: "step2".to_string(),
                    dependency_type: DependencyType::Hard,
                    strength: 1.0,
                    optional: false,
                },
                TodoDependency {
                    from_step: "step2".to_string(),
                    to_step: "step1".to_string(),
                    dependency_type: DependencyType::Hard,
                    strength: 1.0,
                    optional: false,
                },
            ],
            quality_gates: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert!(system.validate_template(&template).is_err());
    }

    #[test]
    fn test_quality_gate_enforcement() {
        let enforcer = QualityGateEnforcer::new();

        // Test that enforced gates are tracked
        assert!(enforcer.enforced_gates.contains("test_coverage"));
        assert!(enforcer.enforced_gates.contains("security_scan"));
        assert!(enforcer.enforced_gates.contains("type_check"));
    }
}



