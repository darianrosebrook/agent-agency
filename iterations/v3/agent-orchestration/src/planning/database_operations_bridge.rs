//! Database Operations Bridge
//!
//! Bridges DatabaseOperationsPort from contracts to the local DatabaseOperations trait.
//! This allows gradual migration from local types to port types.
//!
//! @author @darianrosebrook

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use agent_agency_contracts::ports::{
    DatabaseOperationsPort, DatabaseError,
    CreateExecutionPlanRequest, UpdateExecutionPlanRequest, ExecutionPlanRecord,
    CreateAuditEntryRequest, AuditEntryRecord,
    CreatePlanningSessionRequest, UpdatePlanningSessionRequest, PlanningSessionRecord,
    CreatePlanningTelemetryRequest, PlanningTelemetryRecord,
    CreatePlanningAuditEventRequest, PlanningAuditEventRecord,
    CreateJudgeRequest, JudgeRecord,
    CreateJudgeEvaluationRequest, JudgeEvaluationRecord,
    CreateWorkerRequest, UpdateWorkerRequest, WorkerRecord,
    CreateWaiverRequest, UpdateWaiverRequest, WaiverRecord,
    CreateExecutionResultRequest, ExecutionResultRecord,
    CreateCouncilSessionRequest, UpdateCouncilSessionRequest, CouncilSessionRecord,
};
use crate::planning::data_infrastructure_types::{
    models, CreateAuditTrailEntry, CreateCouncilSession, CreateExecutionPlan,
    CreateExecutionResult, CreateJudge, CreateJudgeEvaluation, CreatePlanningAuditEvent,
    CreatePlanningSession, CreatePlanningTelemetry, CreateWaiver, CreateWorker,
    DatabaseOperations, UpdateCouncilSession, UpdateExecutionPlan, UpdatePlanningSession,
    UpdateWaiver, UpdateWorker,
};

/// Bridge adapter that implements local DatabaseOperations trait by wrapping DatabaseOperationsPort
pub struct DatabaseOperationsBridge {
    port: Arc<dyn DatabaseOperationsPort>,
}

impl DatabaseOperationsBridge {
    /// Create a new bridge adapter
    pub fn new(port: Arc<dyn DatabaseOperationsPort>) -> Self {
        Self { port }
    }
}

#[async_trait]
impl DatabaseOperations for DatabaseOperationsBridge {
    async fn create_execution_plan(
        &self,
        plan: CreateExecutionPlan,
    ) -> Result<models::ExecutionPlan, anyhow::Error> {
        let request = CreateExecutionPlanRequest {
            id: Some(plan.id),
            workspace_id: plan.workspace_id.clone(),
            working_spec_id: plan.working_spec_id.unwrap_or_else(|| format!("PLAN-{}", plan.id)),
            title: plan.title.clone(),
            overview: plan.overview.clone(),
            state: "draft".to_string(),
            milestones: serde_json::json!([]),
            metadata: HashMap::new(),
        };

        let record = self.port.create_execution_plan(request)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        Ok(convert_execution_plan_record(record))
    }

    async fn get_execution_plan(
        &self,
        id: Uuid,
    ) -> Result<Option<models::ExecutionPlan>, anyhow::Error> {
        let record = self.port.get_execution_plan(id)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        Ok(record.map(convert_execution_plan_record))
    }

    async fn get_execution_plans(&self) -> Result<Vec<models::ExecutionPlan>, anyhow::Error> {
        let records = self.port.list_execution_plans()
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        Ok(records.into_iter().map(convert_execution_plan_record).collect())
    }

    async fn update_execution_plan(
        &self,
        id: Uuid,
        update: UpdateExecutionPlan,
    ) -> Result<models::ExecutionPlan, anyhow::Error> {
        let request = UpdateExecutionPlanRequest {
            title: update.title.clone(),
            overview: update.overview.clone(),
            state: update.status.clone(),
            milestones: None,
            metadata: None,
        };

        let record = self.port.update_execution_plan(id, request)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        Ok(convert_execution_plan_record(record))
    }

    async fn create_audit_trail_entry(
        &self,
        entry: CreateAuditTrailEntry,
    ) -> Result<models::AuditTrailEntry, anyhow::Error> {
        let task_id = entry
            .metadata
            .get("task_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .unwrap_or_else(Uuid::new_v4);

        let request = CreateAuditEntryRequest {
            task_id,
            event_type: entry.event_type.clone(),
            description: entry.description.clone(),
            metadata: entry.metadata.clone(),
        };

        let record = self.port.create_audit_entry(request)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        Ok(convert_audit_entry_record(record))
    }

    async fn get_audit_trail_entries(
        &self,
        task_id: Uuid,
    ) -> Result<Vec<models::AuditTrailEntry>, anyhow::Error> {
        let records = self.port.get_audit_entries(task_id)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        Ok(records.into_iter().map(convert_audit_entry_record).collect())
    }

    async fn get_audit_trail_entry(
        &self,
        id: Uuid,
    ) -> Result<Option<models::AuditTrailEntry>, anyhow::Error> {
        let record = self.port.get_audit_entry(id)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        Ok(record.map(convert_audit_entry_record))
    }

    async fn create_planning_session(
        &self,
        session: CreatePlanningSession,
    ) -> Result<models::PlanningSession, anyhow::Error> {
        let request = CreatePlanningSessionRequest {
            plan_id: session.plan_id,
            metadata: session.metadata.clone(),
        };

        let record = self.port.create_planning_session(request)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        Ok(convert_planning_session_record(record))
    }

    async fn get_planning_session(
        &self,
        id: Uuid,
    ) -> Result<Option<models::PlanningSession>, anyhow::Error> {
        let record = self.port.get_planning_session(id)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        Ok(record.map(convert_planning_session_record))
    }

    async fn update_planning_session(
        &self,
        id: Uuid,
        session: UpdatePlanningSession,
    ) -> Result<(), anyhow::Error> {
        let request = UpdatePlanningSessionRequest {
            status: session.status.clone(),
            metadata: session.metadata.clone(),
        };

        self.port.update_planning_session(id, request)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        Ok(())
    }

    async fn create_planning_telemetry(
        &self,
        telemetry: CreatePlanningTelemetry,
    ) -> Result<models::PlanningTelemetry, anyhow::Error> {
        let request = CreatePlanningTelemetryRequest {
            session_id: telemetry.session_id,
            metric_name: telemetry.metric_name.clone(),
            metric_value: telemetry.metric_value,
            metadata: telemetry.metadata.clone(),
        };

        let record = self.port.create_planning_telemetry(request)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        Ok(convert_planning_telemetry_record(record))
    }

    async fn get_planning_telemetry(
        &self,
        plan_id: Uuid,
        metric_type: Option<String>,
    ) -> Result<Vec<models::PlanningTelemetry>, anyhow::Error> {
        let records = self.port.get_planning_telemetry(plan_id, metric_type)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        Ok(records.into_iter().map(convert_planning_telemetry_record).collect())
    }

    async fn create_planning_audit_event(
        &self,
        event: CreatePlanningAuditEvent,
    ) -> Result<(), anyhow::Error> {
        // Extract milestone_id and worker_id from metadata if present
        let mut metadata = event.metadata.clone();
        
        let request = CreatePlanningAuditEventRequest {
            plan_id: event.plan_id,
            event_type: event.event_type.clone(),
            description: event.description.clone(),
            metadata,
        };

        self.port.create_planning_audit_event(request)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        Ok(())
    }

    async fn get_planning_audit_events(
        &self,
        plan_id: Uuid,
    ) -> Result<Vec<models::PlanningAuditEvent>, anyhow::Error> {
        let records = self.port.get_planning_audit_events(plan_id)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        Ok(records.into_iter().map(convert_planning_audit_event_record).collect())
    }

    async fn delete_execution_plan(&self, id: Uuid) -> Result<(), anyhow::Error> {
        self.port.delete_execution_plan(id)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        Ok(())
    }

    async fn create_judge(&self, judge: CreateJudge) -> Result<models::Judge, anyhow::Error> {
        let request = CreateJudgeRequest {
            id: Some(judge.id),
            name: judge.name.clone(),
            judge_type: judge.judge_type.clone(),
            configuration: judge.configuration.clone(),
        };

        let record = self.port.create_judge(request)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        Ok(convert_judge_record(record))
    }

    async fn get_judge(&self, id: Uuid) -> Result<Option<models::Judge>, anyhow::Error> {
        let record = self.port.get_judge(id)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        Ok(record.map(convert_judge_record))
    }

    async fn get_judges(&self) -> Result<Vec<models::Judge>, anyhow::Error> {
        let records = self.port.get_judges()
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        Ok(records.into_iter().map(convert_judge_record).collect())
    }

    async fn create_judge_evaluation(
        &self,
        evaluation: CreateJudgeEvaluation,
    ) -> Result<models::JudgeEvaluation, anyhow::Error> {
        // Extract score from evaluation JSON if available
        let score = evaluation.evaluation
            .get("score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let request = CreateJudgeEvaluationRequest {
            judge_id: evaluation.judge_id,
            task_id: evaluation.task_id,
            evaluation: evaluation.evaluation.clone(),
            score,
        };

        let record = self.port.create_judge_evaluation(request)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        Ok(convert_judge_evaluation_record(record))
    }

    async fn get_judge_evaluations(
        &self,
        task_id: Uuid,
    ) -> Result<Vec<models::JudgeEvaluation>, anyhow::Error> {
        let records = self.port.get_judge_evaluations(task_id)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        Ok(records.into_iter().map(convert_judge_evaluation_record).collect())
    }

    async fn get_workers(&self) -> Result<Vec<models::Worker>, anyhow::Error> {
        let records = self.port.get_workers()
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        Ok(records.into_iter().map(convert_worker_record).collect())
    }

    async fn get_worker(&self, id: Uuid) -> Result<Option<models::Worker>, anyhow::Error> {
        let record = self.port.get_worker(id)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        Ok(record.map(convert_worker_record))
    }

    async fn create_worker(&self, worker: CreateWorker) -> Result<models::Worker, anyhow::Error> {
        let request = CreateWorkerRequest {
            name: worker.name.clone(),
            worker_type: worker.worker_type.clone(),
            specialty: worker.specialty.clone(),
            model_name: worker.model_name.clone(),
            endpoint: worker.endpoint.clone(),
            capabilities: worker.capabilities.clone(),
            performance_history: worker.performance_history.clone(),
            is_active: worker.is_active,
        };

        let record = self.port.create_worker(request)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        Ok(convert_worker_record(record))
    }

    async fn update_worker(
        &self,
        id: Uuid,
        update: UpdateWorker,
    ) -> Result<models::Worker, anyhow::Error> {
        let request = UpdateWorkerRequest {
            name: update.name.clone(),
            worker_type: update.worker_type.clone(),
            specialty: update.specialty.clone(),
            model_name: update.model_name.clone(),
            endpoint: update.endpoint.clone(),
            capabilities: update.capabilities.clone(),
            performance_history: update.performance_history.clone(),
            is_active: update.is_active,
        };

        let record = self.port.update_worker(id, request)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        Ok(convert_worker_record(record))
    }

    async fn get_waivers(
        &self,
        status: Option<String>,
    ) -> Result<Vec<models::Waiver>, anyhow::Error> {
        let records = self.port.get_waivers(status)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        Ok(records.into_iter().map(convert_waiver_record).collect())
    }

    async fn create_waiver(&self, waiver: CreateWaiver) -> Result<models::Waiver, anyhow::Error> {
        let request = CreateWaiverRequest {
            plan_id: waiver.plan_id,
            waiver_type: "general".to_string(), // Default waiver type
            reason: waiver.reason.clone(),
            approved_by: "system".to_string(), // Default approver
            gates: waiver.waived_gates.clone(),
            impact_level: "medium".to_string(), // Default impact level
            mitigation_plan: None,
            expires_at: None,
        };

        let record = self.port.create_waiver(request)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        Ok(convert_waiver_record(record))
    }

    async fn update_waiver(
        &self,
        id: Uuid,
        update: UpdateWaiver,
    ) -> Result<models::Waiver, anyhow::Error> {
        let request = UpdateWaiverRequest {
            status: Some(update.status.clone()),
            mitigation_plan: None,
            expires_at: None,
        };

        let record = self.port.update_waiver(id, request)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        Ok(convert_waiver_record(record))
    }

    async fn create_execution_result(
        &self,
        result: CreateExecutionResult,
    ) -> Result<models::PlanExecutionResult, anyhow::Error> {
        let request = CreateExecutionResultRequest {
            plan_id: result.plan_id,
            success: result.success,
            milestones_completed: result.milestones_completed as i32,
            total_duration_ms: result.total_duration_ms as i64,
            evidence: result.evidence.clone(),
            metrics: result.metrics.clone(),
            final_state: result.final_state.clone(),
            timeline: result.timeline.clone(),
        };

        let record = self.port.create_execution_result(request)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        Ok(convert_execution_result_record(record))
    }

    async fn get_execution_result(
        &self,
        plan_id: Uuid,
    ) -> Result<Option<models::PlanExecutionResult>, anyhow::Error> {
        let record = self.port.get_execution_result(plan_id)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        Ok(record.map(convert_execution_result_record))
    }

    async fn create_council_session(
        &self,
        session: CreateCouncilSession,
    ) -> Result<models::CouncilSession, anyhow::Error> {
        let request = CreateCouncilSessionRequest {
            session_id: session.session_id,
            task_id: session.task_id,
            working_spec_id: session.working_spec_id.clone(),
            review_context: session.review_context.clone(),
            status: session.status.clone(),
            selected_judges: session.selected_judges.clone(),
            contributions: session.contributions.clone(),
            progress: session.progress,
            metadata: session.metadata.clone(),
        };

        let record = self.port.create_council_session(request)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        Ok(convert_council_session_record(record))
    }

    async fn get_council_session(
        &self,
        session_id: Uuid,
    ) -> Result<Option<models::CouncilSession>, anyhow::Error> {
        let record = self.port.get_council_session(session_id)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        Ok(record.map(convert_council_session_record))
    }

    async fn get_council_session_by_task(
        &self,
        task_id: Uuid,
    ) -> Result<Option<models::CouncilSession>, anyhow::Error> {
        let record = self.port.get_council_session_by_task(task_id)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        Ok(record.map(convert_council_session_record))
    }

    async fn update_council_session(
        &self,
        session_id: Uuid,
        update: UpdateCouncilSession,
    ) -> Result<models::CouncilSession, anyhow::Error> {
        let request = UpdateCouncilSessionRequest {
            status: update.status.clone(),
            selected_judges: update.selected_judges.clone(),
            contributions: update.contributions.clone(),
            aggregation_result: update.aggregation_result.clone(),
            final_decision: update.final_decision.clone(),
            progress: update.progress,
            completed_at: update.completed_at,
            metadata: update.metadata.clone(),
        };

        let record = self.port.update_council_session(session_id, request)
            .await
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        Ok(convert_council_session_record(record))
    }
}

// Conversion functions from port types to local types

fn convert_execution_plan_record(record: ExecutionPlanRecord) -> models::ExecutionPlan {
    models::ExecutionPlan {
        id: record.id,
        session_id: record.session_id,
        workspace_id: record.workspace_id,
        working_spec_id: record.working_spec_id,
        title: record.title,
        overview: record.overview,
        state: record.state,
        milestones: record.milestones,
        dependency_graph: record.dependency_graph,
        change_budget: record.change_budget,
        quality_gates: record.quality_gates,
        evidence_requirements: record.evidence_requirements,
        active_waivers: record.active_waivers,
        metadata: record.metadata,
        created_at: record.created_at,
        updated_at: record.updated_at,
        approved_at: record.approved_at,
        completed_at: record.completed_at,
    }
}

fn convert_audit_entry_record(record: AuditEntryRecord) -> models::AuditTrailEntry {
    models::AuditTrailEntry {
        id: record.id,
        event_type: record.event_type,
        description: record.description,
        timestamp: record.timestamp,
        metadata: record.metadata,
    }
}

fn convert_planning_session_record(record: PlanningSessionRecord) -> models::PlanningSession {
    models::PlanningSession {
        id: record.id,
        plan_id: record.plan_id,
        status: record.status,
        created_at: record.created_at,
        updated_at: record.updated_at,
        metadata: record.metadata,
    }
}

fn convert_planning_telemetry_record(record: PlanningTelemetryRecord) -> models::PlanningTelemetry {
    models::PlanningTelemetry {
        id: record.id,
        session_id: record.session_id,
        metric_name: record.metric_name,
        metric_value: record.metric_value,
        timestamp: record.timestamp,
        metadata: record.metadata,
    }
}

fn convert_planning_audit_event_record(record: PlanningAuditEventRecord) -> models::PlanningAuditEvent {
    models::PlanningAuditEvent {
        id: record.id,
        session_id: record.session_id,
        event_type: record.event_type,
        description: record.description,
        timestamp: record.timestamp,
        metadata: record.metadata,
    }
}

fn convert_judge_record(record: JudgeRecord) -> models::Judge {
    models::Judge {
        id: record.id,
        name: record.name,
        judge_type: record.judge_type,
        configuration: record.configuration,
        is_active: record.is_active,
        metadata: record.metadata,
        created_at: record.created_at,
        updated_at: record.updated_at,
    }
}

fn convert_judge_evaluation_record(record: JudgeEvaluationRecord) -> models::JudgeEvaluation {
    models::JudgeEvaluation {
        id: record.id,
        judge_id: record.judge_id,
        task_id: record.task_id,
        evaluation: record.evaluation,
        score: record.score,
        metadata: record.metadata,
        created_at: record.created_at,
    }
}

fn convert_worker_record(record: WorkerRecord) -> models::Worker {
    models::Worker {
        id: record.id,
        name: record.name,
        worker_type: record.worker_type,
        specialty: record.specialty,
        model_name: record.model_name,
        endpoint: record.endpoint,
        capabilities: record.capabilities,
        performance_history: record.performance_history,
        is_active: record.is_active,
        metadata: record.metadata,
        created_at: record.created_at,
        updated_at: record.updated_at,
    }
}

fn convert_waiver_record(record: WaiverRecord) -> models::Waiver {
    models::Waiver {
        id: record.id,
        plan_id: record.plan_id,
        waiver_type: record.waiver_type,
        reason: record.reason,
        approved_by: record.approved_by,
        status: record.status,
        gates: record.gates,
        impact_level: record.impact_level,
        mitigation_plan: record.mitigation_plan,
        created_at: record.created_at,
        expires_at: record.expires_at,
        metadata: record.metadata,
    }
}

fn convert_execution_result_record(record: ExecutionResultRecord) -> models::PlanExecutionResult {
    models::PlanExecutionResult {
        plan_id: record.plan_id,
        success: record.success,
        milestones_completed: record.milestones_completed,
        total_duration_ms: record.total_duration_ms,
        evidence: record.evidence,
        metrics: record.metrics,
        final_state: record.final_state,
        timeline: record.timeline,
        created_at: record.created_at,
        updated_at: record.updated_at,
    }
}

fn convert_council_session_record(record: CouncilSessionRecord) -> models::CouncilSession {
    models::CouncilSession {
        id: record.id,
        session_id: record.session_id,
        task_id: record.task_id,
        working_spec_id: record.working_spec_id,
        review_context: record.review_context,
        status: record.status,
        selected_judges: record.selected_judges,
        contributions: record.contributions,
        aggregation_result: record.aggregation_result,
        final_decision: record.final_decision,
        progress: record.progress,
        started_at: record.started_at,
        completed_at: record.completed_at,
        created_at: record.created_at,
        updated_at: record.updated_at,
        metadata: record.metadata,
    }
}
