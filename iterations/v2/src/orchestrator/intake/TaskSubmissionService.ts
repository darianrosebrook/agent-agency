/**
 * @fileoverview Shared task submission logic with intake validation and audit hooks.
 */

import {
  AuditEventType,
  AuditSeverity,
} from "../../observability/AuditLogger.js";
import { SubmitTaskOptions } from "../runtime/ArbiterRuntime.js";
import { TaskIntakeIssue, TaskIntakeProcessor } from "./TaskIntakeProcessor.js";

export interface TaskSubmissionResult {
  taskId: string;
  status: "accepted" | "rejected" | "queued" | "error";
  message?: string;
  estimatedCompletionTime?: Date;
}

export interface IntakeAuditLogger {
  logAuditEvent: (
    type: AuditEventType,
    severity: AuditSeverity,
    actor: string,
    resource: string,
    action: string,
    outcome: string,
    details?: Record<string, unknown>
  ) => Promise<void>;
}

export interface IntakeRuntimeAdapter {
  submitTask: (
    options: SubmitTaskOptions
  ) => Promise<{ taskId: string; assignmentId?: string; queued: boolean }>;
}

export interface TaskSubmissionDependencies {
  intakeProcessor: TaskIntakeProcessor;
  runtime?: IntakeRuntimeAdapter;
  auditLogger?: IntakeAuditLogger;
  generateTaskId: () => string;
}

function formatIntakeErrors(issues: TaskIntakeIssue[]): string {
  if (issues.length === 0) {
    return "Task payload rejected by intake guardrails";
  }

  return issues.map((issue) => `${issue.code}: ${issue.message}`).join("; ");
}

export async function processTaskSubmission(
  rawTask: any,
  deps: TaskSubmissionDependencies
): Promise<TaskSubmissionResult> {
  const { intakeProcessor, auditLogger, runtime, generateTaskId } = deps;

  // Construct a proper task object from the raw task payload
  const taskObject = {
    id: generateTaskId(),
    type: rawTask?.type || "general",
    description: rawTask?.description || "",
    // Add default values for required fields
    requiredCapabilities: {},
    priority: 5,
    timeoutMs: 60000,
    budget: {
      maxFiles: 10,
      maxLoc: 500,
    },
    createdAt: new Date(),
    metadata: rawTask?.metadata || {},
    attempts: 0,
    maxAttempts: 3,
    // Include specPath in metadata if provided
    ...(rawTask?.specPath && {
      metadata: {
        ...(rawTask?.metadata || {}),
        specPath: rawTask.specPath,
      },
    }),
  };

  const intakeResult = await intakeProcessor.process({
    payload: JSON.stringify(taskObject),
    metadata: {
      contentType: "application/json",
      surface:
        rawTask?.surface ??
        rawTask?.metadata?.surface ??
        rawTask?.metadata?.context?.surface,
      priorityHint: rawTask?.priority,
    },
  });

  if (intakeResult.status === "rejected") {
    const message = formatIntakeErrors(intakeResult.errors);

    if (auditLogger) {
      await auditLogger.logAuditEvent(
        AuditEventType.TASK_SUBMISSION,
        AuditSeverity.HIGH,
        "system",
        "task-queue",
        "submit",
        "failure",
        {
          errors: intakeResult.errors,
          warnings: intakeResult.warnings,
        }
      );
    }

    return {
      taskId: `rejected-${Date.now()}`,
      status: "rejected",
      message,
    };
  }

  const sanitizedTask = {
    ...intakeResult.sanitizedTask!,
  };

  // Use the task ID we already generated
  const submissionId = sanitizedTask.id;

  sanitizedTask.metadata = {
    ...sanitizedTask.metadata,
    intake: {
      chunkCount: intakeResult.metadata.chunkCount,
      chunkSizeBytes: intakeResult.metadata.chunkSizeBytes,
      rawSizeBytes: intakeResult.metadata.rawSizeBytes,
      warnings: intakeResult.warnings,
    },
  };

  if (auditLogger) {
    await auditLogger.logAuditEvent(
      AuditEventType.TASK_SUBMISSION,
      AuditSeverity.LOW,
      "system",
      "task-queue",
      "submit",
      "success",
      {
        taskId: submissionId,
        taskType: sanitizedTask.type,
        description: sanitizedTask.description?.substring(0, 100),
      }
    );
  }

  if (!runtime) {
    return {
      taskId: submissionId,
      status: "error",
      message: "Runtime not available",
    };
  }

  await runtime.submitTask({
    description: sanitizedTask.description,
    metadata: sanitizedTask.metadata,
    type: sanitizedTask.type,
    task: sanitizedTask,
  });

  return {
    taskId: submissionId,
    status: "accepted",
    message: "Task accepted for processing",
    estimatedCompletionTime: new Date(Date.now() + 300000),
  };
}
