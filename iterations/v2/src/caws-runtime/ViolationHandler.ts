/**
 * Violation Handler
 *
 * Processes constitutional violations and executes appropriate responses.
 * Handles alerting, blocking, logging, and escalation.
 *
 * @author @darianrosebrook
 */

import { EventEmitter } from "events";
import {
  ConstitutionalViolation,
  OperationContext,
  EvaluationContext,
  ViolationSeverity,
} from "../types/caws-constitutional";

export interface ViolationResponse {
  violation: ConstitutionalViolation;
  actions: ViolationAction[];
  escalationRequired: boolean;
  timestamp: Date;
  responseTime: number; // milliseconds
}

export interface ViolationAction {
  type: "alert" | "block" | "modify" | "log" | "escalate";
  target: string;
  parameters?: Record<string, any>;
  executed: boolean;
  executionTime?: number;
  error?: string;
}

export interface AlertMessage {
  title: string;
  message: string;
  severity: ViolationSeverity;
  operationId: string;
  agentId?: string;
  userId?: string;
  details: ConstitutionalViolation;
  timestamp: Date;
}

export interface AuditLogEntry {
  violation: ConstitutionalViolation;
  operation: OperationContext;
  context: EvaluationContext;
  actions: ViolationAction[];
  timestamp: Date;
}

export interface ViolationConfig {
  alertEnabled: boolean;
  blockEnabled: boolean;
  logEnabled: boolean;
  escalationEnabled: boolean;
  timeoutMs: number;
}

export class ViolationHandler extends EventEmitter {
  constructor(
    private alertManager: AlertManager,
    private auditLogger: AuditLogger,
    private config: ViolationConfig = {
      alertEnabled: true,
      blockEnabled: true,
      logEnabled: true,
      escalationEnabled: true,
      timeoutMs: 5000,
    }
  ) {
    super();
  }

  /**
   * Handle multiple violations
   */
  async handleViolations(
    violations: ConstitutionalViolation[],
    operation: OperationContext,
    context: EvaluationContext,
    timeoutMs: number = this.config.timeoutMs
  ): Promise<ViolationResponse[]> {
    const responses: ViolationResponse[] = [];

    for (const violation of violations) {
      try {
        const response = await Promise.race([
          this.handleViolation(violation, operation, context),
          this.timeoutPromise(violation.id, timeoutMs),
        ]);
        responses.push(response);
      } catch (error) {
        // Create error response
        const errorResponse: ViolationResponse = {
          violation,
          actions: [{
            type: "log",
            target: "error",
            parameters: { error: error instanceof Error ? error.message : "Unknown error" },
            executed: false,
            error: error instanceof Error ? error.message : "Unknown error",
          }],
          escalationRequired: violation.severity === "critical",
          timestamp: new Date(),
          responseTime: 0,
        };
        responses.push(errorResponse);
      }
    }

    return responses;
  }

  /**
   * Handle a single violation
   */
  private async handleViolation(
    violation: ConstitutionalViolation,
    operation: OperationContext,
    context: EvaluationContext
  ): Promise<ViolationResponse> {
    const startTime = Date.now();
    const actions: ViolationAction[] = [];

    // Determine actions based on severity and configuration
    const requiredActions = this.determineActions(violation);

    // Execute actions
    for (const action of requiredActions) {
      try {
        await this.executeAction(action, violation, operation, context);
        action.executed = true;
        action.executionTime = Date.now() - startTime;
      } catch (error) {
        action.executed = false;
        action.error = error instanceof Error ? error.message : "Unknown error";
        action.executionTime = Date.now() - startTime;
      }
      actions.push(action);
    }

    const escalationRequired = this.requiresEscalation(violation, actions);
    const responseTime = Date.now() - startTime;

    const response: ViolationResponse = {
      violation,
      actions,
      escalationRequired,
      timestamp: new Date(),
      responseTime,
    };

    this.emit("violation:handled", {
      violationId: violation.id,
      operationId: operation.id,
      actionCount: actions.length,
      escalationRequired,
      responseTime,
      timestamp: new Date(),
    });

    return response;
  }

  /**
   * Determine appropriate actions for violation
   */
  private determineActions(violation: ConstitutionalViolation): ViolationAction[] {
    const actions: ViolationAction[] = [];

    switch (violation.severity) {
      case ViolationSeverity.LOW:
        if (this.config.logEnabled) {
          actions.push({
            type: "log",
            target: "compliance-log",
            parameters: { level: "info" },
            executed: false,
          });
        }
        break;

      case ViolationSeverity.MEDIUM:
        if (this.config.alertEnabled) {
          actions.push({
            type: "alert",
            target: "team-alerts",
            parameters: { priority: "medium" },
            executed: false,
          });
        }
        if (this.config.logEnabled) {
          actions.push({
            type: "log",
            target: "compliance-log",
            parameters: { level: "warn" },
            executed: false,
          });
        }
        break;

      case ViolationSeverity.HIGH:
        if (this.config.alertEnabled) {
          actions.push({
            type: "alert",
            target: "security-team",
            parameters: { priority: "high", immediate: true },
            executed: false,
          });
        }
        if (this.config.logEnabled) {
          actions.push({
            type: "log",
            target: "compliance-log",
            parameters: { level: "error" },
            executed: false,
          });
        }
        if (this.config.escalationEnabled) {
          actions.push({
            type: "escalate",
            target: "management",
            parameters: { priority: "high" },
            executed: false,
          });
        }
        break;

      case ViolationSeverity.CRITICAL:
        if (this.config.blockEnabled) {
          actions.push({
            type: "block",
            target: "operation",
            parameters: { reason: violation.message },
            executed: false,
          });
        }
        if (this.config.alertEnabled) {
          actions.push({
            type: "alert",
            target: "executive-team",
            parameters: { priority: "critical", immediate: true },
            executed: false,
          });
        }
        if (this.config.logEnabled) {
          actions.push({
            type: "log",
            target: "compliance-log",
            parameters: { level: "fatal" },
            executed: false,
          });
        }
        if (this.config.escalationEnabled) {
          actions.push({
            type: "escalate",
            target: "executive",
            parameters: { priority: "critical" },
            executed: false,
          });
        }
        break;
    }

    return actions;
  }

  /**
   * Execute a violation action
   */
  private async executeAction(
    action: ViolationAction,
    violation: ConstitutionalViolation,
    operation: OperationContext,
    context: EvaluationContext
  ): Promise<void> {
    switch (action.type) {
      case "alert":
        await this.alertManager.sendAlert({
          title: `Constitutional Violation: ${violation.principle}`,
          message: violation.message,
          severity: violation.severity,
          operationId: operation.id,
          agentId: context.agentId,
          userId: context.userId,
          details: violation,
          timestamp: new Date(),
        });
        break;

      case "log":
        await this.auditLogger.logViolation(violation, operation, context);
        break;

      case "block":
        // In a real implementation, this would block the operation
        throw new Error(`Operation blocked: ${action.parameters?.reason}`);
        break;

      case "escalate":
        await this.alertManager.escalateViolation(violation, operation, context);
        break;

      case "modify":
        // In a real implementation, this would modify the operation
        console.warn(`Modification action not implemented for violation ${violation.id}`);
        break;
    }
  }

  /**
   * Check if violation requires escalation
   */
  private requiresEscalation(
    violation: ConstitutionalViolation,
    actions: ViolationAction[]
  ): boolean {
    // Escalate if severity is high/critical or if blocking actions failed
    if (violation.severity === "high" || violation.severity === "critical") {
      return true;
    }

    // Escalate if blocking actions were attempted but failed
    const blockActions = actions.filter(a => a.type === "block");
    if (blockActions.length > 0 && blockActions.some(a => !a.executed)) {
      return true;
    }

    return false;
  }

  /**
   * Create timeout promise for violation handling
   */
  private timeoutPromise(violationId: string, timeoutMs: number): Promise<ViolationResponse> {
    return new Promise((_, reject) =>
      setTimeout(
        () => reject(new Error(`Violation handling timeout for ${violationId}`)),
        timeoutMs
      )
    );
  }

  /**
   * Update configuration
   */
  updateConfig(newConfig: Partial<ViolationConfig>): void {
    this.config = { ...this.config, ...newConfig };
  }
}

// Placeholder interfaces for dependencies
export interface AlertManager {
  sendAlert(alert: AlertMessage): Promise<void>;
  escalateViolation(
    violation: ConstitutionalViolation,
    operation: OperationContext,
    context: EvaluationContext
  ): Promise<void>;
}

export interface AuditLogger {
  logViolation(
    violation: ConstitutionalViolation,
    operation: OperationContext,
    context: EvaluationContext
  ): Promise<void>;
}
