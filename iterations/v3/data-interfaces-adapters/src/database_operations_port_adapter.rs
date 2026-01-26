//! Database Operations Port Adapter
//!
//! Implements DatabaseOperationsPort from agent-agency-contracts by wrapping
//! the existing DatabaseOperationsAdapter and converting between port types and local types.
//!
//! @author @darianrosebrook

use async_trait::async_trait;
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

use super::database_operations_adapter::DatabaseOperationsAdapter;
use agent_orchestration::planning::data_infrastructure_types::{
    CreateAuditTrailEntry, CreateCouncilSession, CreateExecutionPlan,
    CreateExecutionResult, CreateJudge, CreateJudgeEvaluation, CreatePlanningAuditEvent,
    CreatePlanningSession, CreatePlanningTelemetry, CreateWaiver, CreateWorker,
    DatabaseOperations,
    UpdateCouncilSession, UpdateExecutionPlan, UpdatePlanningSession, UpdateWaiver, UpdateWorker,
};

/// Port adapter that wraps DatabaseOperationsAdapter and converts types
pub struct DatabaseOperationsPortAdapter {
    inner: Arc<DatabaseOperationsAdapter>,
}

impl DatabaseOperationsPortAdapter {
    /// Create a new port adapter wrapping the existing adapter
    pub fn new(inner: Arc<DatabaseOperationsAdapter>) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl DatabaseOperationsPort for DatabaseOperationsPortAdapter {
    async fn create_execution_plan(
        &self,
        plan: CreateExecutionPlanRequest,
    ) -> Result<ExecutionPlanRecord, DatabaseError> {
        let id = plan.id.unwrap_or_else(Uuid::new_v4);
        let local_plan = CreateExecutionPlan {
            id,
            workspace_id: plan.workspace_id.clone(),
            title: plan.title.clone(),
            overview: plan.overview.clone(),
            working_spec_id: Some(plan.working_spec_id.clone()),
        };

        let local_result = DatabaseOperations::create_execution_plan(&*self.inner, local_plan)
            .await
            .map_err(|e| DatabaseError::Unknown(e.to_string()))?;

        Ok(convert_execution_plan_to_record(local_result))
    }

    async fn get_execution_plan(
        &self,
        id: Uuid,
    ) -> Result<Option<ExecutionPlanRecord>, DatabaseError> {
        let local_result = DatabaseOperations::get_execution_plan(&*self.inner, id)
            .await
            .map_err(|e| DatabaseError::Unknown(e.to_string()))?;

        Ok(local_result.map(convert_execution_plan_to_record))
    }

    async fn list_execution_plans(&self) -> Result<Vec<ExecutionPlanRecord>, DatabaseError> {
        let local_results = DatabaseOperations::get_execution_plans(&*self.inner)
            .await
            .map_err(|e| DatabaseError::Unknown(e.to_string()))?;

        Ok(local_results.into_iter().map(convert_execution_plan_to_record).collect())
    }

    async fn update_execution_plan(
        &self,
        id: Uuid,
        update: UpdateExecutionPlanRequest,
    ) -> Result<ExecutionPlanRecord, DatabaseError> {
        let local_update = UpdateExecutionPlan {
            id,
            title: update.title.clone(),
            overview: update.overview.clone(),
            status: update.state.clone(),
        };

        let local_result = DatabaseOperations::update_execution_plan(&*self.inner, id, local_update)
            .await
            .map_err(|e| DatabaseError::Unknown(e.to_string()))?;

        Ok(convert_execution_plan_to_record(local_result))
    }

    async fn delete_execution_plan(&self, id: Uuid) -> Result<(), DatabaseError> {
        DatabaseOperations::delete_execution_plan(&*self.inner, id)
            .await
            .map_err(|e| DatabaseError::Unknown(e.to_string()))?;

        Ok(())
    }

    async fn create_audit_entry(
        &self,
        entry: CreateAuditEntryRequest,
    ) -> Result<AuditEntryRecord, DatabaseError> {
        let mut metadata = entry.metadata.clone();
        metadata.insert("task_id".to_string(), serde_json::Value::String(entry.task_id.to_string()));

        let local_entry = CreateAuditTrailEntry {
            event_type: entry.event_type.clone(),
            description: entry.description.clone(),
            metadata,
        };

        let local_result = DatabaseOperations::create_audit_trail_entry(&*self.inner, local_entry)
            .await
            .map_err(|e| DatabaseError::Unknown(e.to_string()))?;

        Ok(convert_audit_entry_to_record(local_result))
    }

    async fn get_audit_entries(
        &self,
        task_id: Uuid,
    ) -> Result<Vec<AuditEntryRecord>, DatabaseError> {
        let local_results = DatabaseOperations::get_audit_trail_entries(&*self.inner, task_id)
            .await
            .map_err(|e| DatabaseError::Unknown(e.to_string()))?;

        Ok(local_results.into_iter().map(convert_audit_entry_to_record).collect())
    }

    async fn get_audit_entry(&self, id: Uuid) -> Result<Option<AuditEntryRecord>, DatabaseError> {
        let local_result = DatabaseOperations::get_audit_trail_entry(&*self.inner, id)
            .await
            .map_err(|e| DatabaseError::Unknown(e.to_string()))?;

        Ok(local_result.map(convert_audit_entry_to_record))
    }

    async fn create_planning_session(
        &self,
        session: CreatePlanningSessionRequest,
    ) -> Result<PlanningSessionRecord, DatabaseError> {
        let local_session = CreatePlanningSession {
            plan_id: session.plan_id,
            metadata: session.metadata.clone(),
        };

        let local_result = DatabaseOperations::create_planning_session(&*self.inner, local_session)
            .await
            .map_err(|e| DatabaseError::Unknown(e.to_string()))?;

        Ok(convert_planning_session_to_record(local_result))
    }

    async fn get_planning_session(
        &self,
        id: Uuid,
    ) -> Result<Option<PlanningSessionRecord>, DatabaseError> {
        let local_result = DatabaseOperations::get_planning_session(&*self.inner, id)
            .await
            .map_err(|e| DatabaseError::Unknown(e.to_string()))?;

        Ok(local_result.map(convert_planning_session_to_record))
    }

    async fn update_planning_session(
        &self,
        id: Uuid,
        update: UpdatePlanningSessionRequest,
    ) -> Result<(), DatabaseError> {
        let local_update = UpdatePlanningSession {
            id,
            status: update.status.clone(),
            metadata: update.metadata.clone(),
        };

        DatabaseOperations::update_planning_session(&*self.inner, id, local_update)
            .await
            .map_err(|e| DatabaseError::Unknown(e.to_string()))?;

        Ok(())
    }

    async fn create_planning_telemetry(
        &self,
        telemetry: CreatePlanningTelemetryRequest,
    ) -> Result<PlanningTelemetryRecord, DatabaseError> {
        let local_telemetry = CreatePlanningTelemetry {
            session_id: telemetry.session_id,
            metric_name: telemetry.metric_name.clone(),
            metric_value: telemetry.metric_value,
            metadata: telemetry.metadata.clone(),
        };

        let local_result = DatabaseOperations::create_planning_telemetry(&*self.inner, local_telemetry)
            .await
            .map_err(|e| DatabaseError::Unknown(e.to_string()))?;

        Ok(convert_planning_telemetry_to_record(local_result))
    }

    async fn get_planning_telemetry(
        &self,
        plan_id: Uuid,
        metric_type: Option<String>,
    ) -> Result<Vec<PlanningTelemetryRecord>, DatabaseError> {
        // Note: The port uses plan_id but local uses session_id
        // For now, we'll use plan_id as session_id (this may need refinement)
        let local_results = DatabaseOperations::get_planning_telemetry(&*self.inner, plan_id, metric_type)
            .await
            .map_err(|e| DatabaseError::Unknown(e.to_string()))?;

        Ok(local_results.into_iter().map(convert_planning_telemetry_to_record).collect())
    }

    async fn create_planning_audit_event(
        &self,
        event: CreatePlanningAuditEventRequest,
    ) -> Result<(), DatabaseError> {
        let local_event = CreatePlanningAuditEvent {
            plan_id: event.plan_id,
            event_type: event.event_type.clone(),
            description: event.description.clone(),
            metadata: event.metadata.clone(),
        };

        DatabaseOperations::create_planning_audit_event(&*self.inner, local_event)
            .await
            .map_err(|e| DatabaseError::Unknown(e.to_string()))?;

        Ok(())
    }

    async fn get_planning_audit_events(
        &self,
        plan_id: Uuid,
    ) -> Result<Vec<PlanningAuditEventRecord>, DatabaseError> {
        let local_results = DatabaseOperations::get_planning_audit_events(&*self.inner, plan_id)
            .await
            .map_err(|e| DatabaseError::Unknown(e.to_string()))?;

        Ok(local_results.into_iter().map(convert_planning_audit_event_to_record).collect())
    }

    async fn create_judge(&self, judge: CreateJudgeRequest) -> Result<JudgeRecord, DatabaseError> {
        let id = judge.id.unwrap_or_else(Uuid::new_v4);
        let local_judge = CreateJudge {
            id,
            name: judge.name.clone(),
            judge_type: judge.judge_type.clone(),
            configuration: judge.configuration.clone(),
        };

        let local_result = DatabaseOperations::create_judge(&*self.inner, local_judge)
            .await
            .map_err(|e| DatabaseError::Unknown(e.to_string()))?;

        Ok(convert_judge_to_record(local_result))
    }

    async fn get_judge(&self, id: Uuid) -> Result<Option<JudgeRecord>, DatabaseError> {
        let local_result = DatabaseOperations::get_judge(&*self.inner, id)
            .await
            .map_err(|e| DatabaseError::Unknown(e.to_string()))?;

        Ok(local_result.map(convert_judge_to_record))
    }

    async fn get_judges(&self) -> Result<Vec<JudgeRecord>, DatabaseError> {
        let local_results = DatabaseOperations::get_judges(&*self.inner)
            .await
            .map_err(|e| DatabaseError::Unknown(e.to_string()))?;

        Ok(local_results.into_iter().map(convert_judge_to_record).collect())
    }

    async fn create_judge_evaluation(
        &self,
        evaluation: CreateJudgeEvaluationRequest,
    ) -> Result<JudgeEvaluationRecord, DatabaseError> {
        let local_evaluation = CreateJudgeEvaluation {
            judge_id: evaluation.judge_id,
            task_id: evaluation.task_id,
            evaluation: evaluation.evaluation.clone(),
            score: evaluation.score,
        };

        let local_result = DatabaseOperations::create_judge_evaluation(&*self.inner, local_evaluation)
            .await
            .map_err(|e| DatabaseError::Unknown(e.to_string()))?;

        Ok(convert_judge_evaluation_to_record(local_result))
    }

    async fn get_judge_evaluations(
        &self,
        task_id: Uuid,
    ) -> Result<Vec<JudgeEvaluationRecord>, DatabaseError> {
        let local_results = DatabaseOperations::get_judge_evaluations(&*self.inner, task_id)
            .await
            .map_err(|e| DatabaseError::Unknown(e.to_string()))?;

        Ok(local_results.into_iter().map(convert_judge_evaluation_to_record).collect())
    }

    async fn get_workers(&self) -> Result<Vec<WorkerRecord>, DatabaseError> {
        let local_results = DatabaseOperations::get_workers(&*self.inner)
            .await
            .map_err(|e| DatabaseError::Unknown(e.to_string()))?;

        Ok(local_results.into_iter().map(convert_worker_to_record).collect())
    }

    async fn get_worker(&self, id: Uuid) -> Result<Option<WorkerRecord>, DatabaseError> {
        let local_result = DatabaseOperations::get_worker(&*self.inner, id)
            .await
            .map_err(|e| DatabaseError::Unknown(e.to_string()))?;

        Ok(local_result.map(convert_worker_to_record))
    }

    async fn create_worker(
        &self,
        worker: CreateWorkerRequest,
    ) -> Result<WorkerRecord, DatabaseError> {
        let local_worker = CreateWorker {
            name: worker.name.clone(),
            worker_type: worker.worker_type.clone(),
            specialty: worker.specialty.clone(),
            model_name: worker.model_name.clone(),
            endpoint: worker.endpoint.clone(),
            capabilities: worker.capabilities.clone(),
            performance_history: worker.performance_history.clone(),
            is_active: worker.is_active,
        };

        let local_result = DatabaseOperations::create_worker(&*self.inner, local_worker)
            .await
            .map_err(|e| DatabaseError::Unknown(e.to_string()))?;

        Ok(convert_worker_to_record(local_result))
    }

    async fn update_worker(
        &self,
        id: Uuid,
        update: UpdateWorkerRequest,
    ) -> Result<WorkerRecord, DatabaseError> {
        let local_update = UpdateWorker {
            name: update.name.clone(),
            worker_type: update.worker_type.clone(),
            specialty: update.specialty.clone(),
            model_name: update.model_name.clone(),
            endpoint: update.endpoint.clone(),
            capabilities: update.capabilities.clone(),
            performance_history: update.performance_history.clone(),
            is_active: update.is_active,
        };

        let local_result = DatabaseOperations::update_worker(&*self.inner, id, local_update)
            .await
            .map_err(|e| DatabaseError::Unknown(e.to_string()))?;

        Ok(convert_worker_to_record(local_result))
    }

    async fn get_waivers(
        &self,
        status: Option<String>,
    ) -> Result<Vec<WaiverRecord>, DatabaseError> {
        let local_results = DatabaseOperations::get_waivers(&*self.inner, status)
            .await
            .map_err(|e| DatabaseError::Unknown(e.to_string()))?;

        Ok(local_results.into_iter().map(convert_waiver_to_record).collect())
    }

    async fn create_waiver(
        &self,
        waiver: CreateWaiverRequest,
    ) -> Result<WaiverRecord, DatabaseError> {
        let local_waiver = CreateWaiver {
            plan_id: waiver.plan_id,
            reason: waiver.reason.clone(),
            waived_gates: waiver.gates.clone(),
        };

        let local_result = DatabaseOperations::create_waiver(&*self.inner, local_waiver)
            .await
            .map_err(|e| DatabaseError::Unknown(e.to_string()))?;

        Ok(convert_waiver_to_record(local_result))
    }

    async fn update_waiver(
        &self,
        id: Uuid,
        update: UpdateWaiverRequest,
    ) -> Result<WaiverRecord, DatabaseError> {
        let local_update = UpdateWaiver {
            id,
            status: update.status.unwrap_or_else(|| "active".to_string()),
        };

        let local_result = DatabaseOperations::update_waiver(&*self.inner, id, local_update)
            .await
            .map_err(|e| DatabaseError::Unknown(e.to_string()))?;

        Ok(convert_waiver_to_record(local_result))
    }

    async fn create_execution_result(
        &self,
        result: CreateExecutionResultRequest,
    ) -> Result<ExecutionResultRecord, DatabaseError> {
        let local_result = CreateExecutionResult {
            plan_id: result.plan_id,
            success: result.success,
            milestones_completed: result.milestones_completed as usize,
            total_duration_ms: result.total_duration_ms as u64,
            evidence: result.evidence.clone(),
            metrics: result.metrics.clone(),
            final_state: result.final_state.clone(),
            timeline: result.timeline.clone(),
        };

        let local_record = DatabaseOperations::create_execution_result(&*self.inner, local_result)
            .await
            .map_err(|e| DatabaseError::Unknown(e.to_string()))?;

        Ok(convert_execution_result_to_record(local_record))
    }

    async fn get_execution_result(
        &self,
        plan_id: Uuid,
    ) -> Result<Option<ExecutionResultRecord>, DatabaseError> {
        let local_result = DatabaseOperations::get_execution_result(&*self.inner, plan_id)
            .await
            .map_err(|e| DatabaseError::Unknown(e.to_string()))?;

        Ok(local_result.map(convert_execution_result_to_record))
    }

    async fn create_council_session(
        &self,
        session: CreateCouncilSessionRequest,
    ) -> Result<CouncilSessionRecord, DatabaseError> {
        let local_session = CreateCouncilSession {
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

        let local_result = DatabaseOperations::create_council_session(&*self.inner, local_session)
            .await
            .map_err(|e| DatabaseError::Unknown(e.to_string()))?;

        Ok(convert_council_session_to_record(local_result))
    }

    async fn get_council_session(
        &self,
        session_id: Uuid,
    ) -> Result<Option<CouncilSessionRecord>, DatabaseError> {
        let local_result = DatabaseOperations::get_council_session(&*self.inner, session_id)
            .await
            .map_err(|e| DatabaseError::Unknown(e.to_string()))?;

        Ok(local_result.map(convert_council_session_to_record))
    }

    async fn get_council_session_by_task(
        &self,
        task_id: Uuid,
    ) -> Result<Option<CouncilSessionRecord>, DatabaseError> {
        let local_result = DatabaseOperations::get_council_session_by_task(&*self.inner, task_id)
            .await
            .map_err(|e| DatabaseError::Unknown(e.to_string()))?;

        Ok(local_result.map(convert_council_session_to_record))
    }

    async fn update_council_session(
        &self,
        session_id: Uuid,
        update: UpdateCouncilSessionRequest,
    ) -> Result<CouncilSessionRecord, DatabaseError> {
        let local_update = UpdateCouncilSession {
            status: update.status.clone(),
            selected_judges: update.selected_judges.clone(),
            contributions: update.contributions.clone(),
            aggregation_result: update.aggregation_result.clone(),
            final_decision: update.final_decision.clone(),
            progress: update.progress,
            completed_at: update.completed_at,
            metadata: update.metadata.clone(),
        };

        let local_result = DatabaseOperations::update_council_session(&*self.inner, session_id, local_update)
            .await
            .map_err(|e| DatabaseError::Unknown(e.to_string()))?;

        Ok(convert_council_session_to_record(local_result))
    }

    async fn health_check(&self) -> Result<bool, DatabaseError> {
        // Simple health check - try to get execution plans
        match self.list_execution_plans().await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false), // Database might be down but we don't want to fail hard
        }
    }
}

// Conversion functions from local types to port record types

fn convert_execution_plan_to_record(plan: agent_orchestration::planning::data_infrastructure_types::models::ExecutionPlan) -> ExecutionPlanRecord {
    ExecutionPlanRecord {
        id: plan.id,
        session_id: plan.session_id,
        workspace_id: plan.workspace_id,
        working_spec_id: plan.working_spec_id,
        title: plan.title,
        overview: plan.overview,
        state: plan.state,
        milestones: plan.milestones,
        dependency_graph: plan.dependency_graph,
        change_budget: plan.change_budget,
        quality_gates: plan.quality_gates,
        evidence_requirements: plan.evidence_requirements,
        active_waivers: plan.active_waivers,
        metadata: plan.metadata,
        created_at: plan.created_at,
        updated_at: plan.updated_at,
        approved_at: plan.approved_at,
        completed_at: plan.completed_at,
    }
}

fn convert_audit_entry_to_record(entry: agent_orchestration::planning::data_infrastructure_types::models::AuditTrailEntry) -> AuditEntryRecord {
    AuditEntryRecord {
        id: entry.id,
        task_id: entry.metadata
            .get("task_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
            .unwrap_or_else(Uuid::new_v4),
        event_type: entry.event_type,
        description: entry.description,
        timestamp: entry.timestamp,
        metadata: entry.metadata,
    }
}

fn convert_planning_session_to_record(session: agent_orchestration::planning::data_infrastructure_types::models::PlanningSession) -> PlanningSessionRecord {
    PlanningSessionRecord {
        id: session.id,
        plan_id: session.plan_id,
        status: session.status,
        created_at: session.created_at,
        updated_at: session.updated_at,
        metadata: session.metadata,
    }
}

fn convert_planning_telemetry_to_record(telemetry: agent_orchestration::planning::data_infrastructure_types::models::PlanningTelemetry) -> PlanningTelemetryRecord {
    PlanningTelemetryRecord {
        id: telemetry.id,
        session_id: telemetry.session_id,
        metric_name: telemetry.metric_name,
        metric_value: telemetry.metric_value,
        timestamp: telemetry.timestamp,
        metadata: telemetry.metadata,
    }
}

fn convert_planning_audit_event_to_record(event: agent_orchestration::planning::data_infrastructure_types::models::PlanningAuditEvent) -> PlanningAuditEventRecord {
    PlanningAuditEventRecord {
        id: event.id,
        session_id: event.session_id,
        event_type: event.event_type,
        description: event.description,
        timestamp: event.timestamp,
        metadata: event.metadata,
    }
}

fn convert_judge_to_record(judge: agent_orchestration::planning::data_infrastructure_types::models::Judge) -> JudgeRecord {
    JudgeRecord {
        id: judge.id,
        name: judge.name,
        judge_type: judge.judge_type,
        configuration: judge.configuration,
        is_active: judge.is_active,
        metadata: judge.metadata,
        created_at: judge.created_at,
        updated_at: judge.updated_at,
    }
}

fn convert_judge_evaluation_to_record(eval: agent_orchestration::planning::data_infrastructure_types::models::JudgeEvaluation) -> JudgeEvaluationRecord {
    JudgeEvaluationRecord {
        id: eval.id,
        judge_id: eval.judge_id,
        task_id: eval.task_id,
        evaluation: eval.evaluation,
        score: eval.score,
        metadata: eval.metadata,
        created_at: eval.created_at,
    }
}

fn convert_worker_to_record(worker: agent_orchestration::planning::data_infrastructure_types::models::Worker) -> WorkerRecord {
    WorkerRecord {
        id: worker.id,
        name: worker.name,
        worker_type: worker.worker_type,
        specialty: worker.specialty,
        model_name: worker.model_name,
        endpoint: worker.endpoint,
        capabilities: worker.capabilities,
        performance_history: worker.performance_history,
        is_active: worker.is_active,
        metadata: worker.metadata,
        created_at: worker.created_at,
        updated_at: worker.updated_at,
    }
}

fn convert_waiver_to_record(waiver: agent_orchestration::planning::data_infrastructure_types::models::Waiver) -> WaiverRecord {
    WaiverRecord {
        id: waiver.id,
        plan_id: waiver.plan_id,
        waiver_type: waiver.waiver_type,
        reason: waiver.reason,
        approved_by: waiver.approved_by,
        status: waiver.status,
        gates: waiver.gates,
        impact_level: waiver.impact_level,
        mitigation_plan: waiver.mitigation_plan,
        created_at: waiver.created_at,
        expires_at: waiver.expires_at,
        metadata: waiver.metadata,
    }
}

fn convert_execution_result_to_record(result: agent_orchestration::planning::data_infrastructure_types::models::PlanExecutionResult) -> ExecutionResultRecord {
    ExecutionResultRecord {
        plan_id: result.plan_id,
        success: result.success,
        milestones_completed: result.milestones_completed,
        total_duration_ms: result.total_duration_ms,
        evidence: result.evidence,
        metrics: result.metrics,
        final_state: result.final_state,
        timeline: result.timeline,
        created_at: result.created_at,
        updated_at: result.updated_at,
    }
}

fn convert_council_session_to_record(session: agent_orchestration::planning::data_infrastructure_types::models::CouncilSession) -> CouncilSessionRecord {
    CouncilSessionRecord {
        id: session.id,
        session_id: session.session_id,
        task_id: session.task_id,
        working_spec_id: session.working_spec_id,
        review_context: session.review_context,
        status: session.status,
        selected_judges: session.selected_judges,
        contributions: session.contributions,
        aggregation_result: session.aggregation_result,
        final_decision: session.final_decision,
        progress: session.progress,
        started_at: session.started_at,
        completed_at: session.completed_at,
        created_at: session.created_at,
        updated_at: session.updated_at,
        metadata: session.metadata,
    }
}
