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
  EvaluationContext,
  OperationContext,
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
    private _alertManager: AlertManager,
    private _auditLogger: AuditLogger,
    private _config: ViolationConfig = {
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
          actions: [
            {
              type: "log",
              target: "error",
              parameters: {
                error: error instanceof Error ? error.message : "Unknown error",
              },
              executed: false,
              error: error instanceof Error ? error.message : "Unknown error",
            },
          ],
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
  private determineActions(
    violation: ConstitutionalViolation
  ): ViolationAction[] {
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
        await this.alertManager.escalateViolation(
          violation,
          operation,
          context
        );
        break;

      case "modify":
        await this.modifyOperation(violation, operation, context);
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
    const blockActions = actions.filter((a) => a.type === "block");
    if (blockActions.length > 0 && blockActions.some((a) => !a.executed)) {
      return true;
    }

    return false;
  }

  /**
   * Create timeout promise for violation handling
   */
  private timeoutPromise(
    violationId: string,
    timeoutMs: number
  ): Promise<ViolationResponse> {
    return new Promise((_, reject) =>
      setTimeout(
        () =>
          reject(new Error(`Violation handling timeout for ${violationId}`)),
        timeoutMs
      )
    );
  }

  /**
   * Modifies an operation to make it compliant with policy
   */
  private async modifyOperation(
    violation: ConstitutionalViolation,
    operation: OperationContext,
    context: EvaluationContext
  ): Promise<void> {
    console.log(
      `Modifying operation for violation ${violation.id}: ${violation.message}`
    );

    // Apply modifications based on violation principle
    const modifiedOperation = { ...operation };

    // Generic sanitization for all violations
    modifiedOperation.payload = this.sanitizeParameters(operation.payload);

    // Principle-specific modifications
    switch (violation.principle) {
      case "safety":
        modifiedOperation.payload = this.applySafetyModifications(
          operation.payload
        );
        break;

      case "privacy":
        modifiedOperation.payload = this.applyPrivacyModifications(
          operation.payload
        );
        break;

      case "reliability":
        modifiedOperation.payload = this.applyReliabilityModifications(
          operation.payload
        );
        break;

      default:
        // Keep generic sanitization
        break;
    }

    // Log the modification
    await this.auditLogger.logViolation(violation, modifiedOperation, context);

    console.log(
      `Operation modified successfully for violation ${violation.id}`
    );

    // Emit modification event for monitoring
    this.emit("operation_modified", {
      violationId: violation.id,
      originalOperation: operation,
      modifiedOperation,
      timestamp: new Date().toISOString(),
    });
  }

  /**
   * Apply safety-related modifications
   */
  private applySafetyModifications(payload: any): any {
    const modified = { ...payload };

    // Remove potentially dangerous operations
    const dangerousActions = [
      "delete_all_data",
      "drop_database",
      "format_disk",
      "shutdown_system",
      "kill_process",
      "remove_files",
      "clear_logs",
      "reset_config",
    ];

    for (const action of dangerousActions) {
      if (
        modified[action] ||
        modified.action === action ||
        modified.dangerousAction === action
      ) {
        delete modified[action];
        if (modified.action === action) delete modified.action;
        if (modified.dangerousAction === action)
          delete modified.dangerousAction;
      }
    }

    // Restrict permissions to safe levels
    if (modified.permissions) {
      modified.permissions = this.restrictPermissions(modified.permissions);
    }

    // Sanitize file paths
    if (modified.filePath) {
      modified.filePath = this.sanitizeFilePath(modified.filePath);
    }

    return modified;
  }

  /**
   * Apply privacy-related modifications
   */
  private applyPrivacyModifications(payload: any): any {
    const modified = { ...payload };

    // Remove sensitive data fields completely
    const sensitiveFields = [
      "password",
      "token",
      "apiKey",
      "ssn",
      "creditCard",
      "bankAccount",
    ];

    for (const field of sensitiveFields) {
      if (modified[field]) {
        delete modified[field];
      }
    }

    // Sanitize personal data fields (don't delete, just anonymize)
    const personalDataFields = ["personalData", "sensitiveInfo"];
    for (const field of personalDataFields) {
      if (modified[field] && typeof modified[field] === "string") {
        // Anonymize personal data in strings
        modified[field] = modified[field]
          .replace(
            /\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b/g,
            "[EMAIL]"
          )
          .replace(/\b\d{3}[-.]?\d{3}[-.]?\d{4}\b/g, "[PHONE]")
          .replace(/\b\d{3}-\d{2}-\d{4}\b/g, "[SSN]")
          .replace(/\b\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}\b/g, "[CARD]");
      }
    }

    // Anonymize personal data in all string values
    for (const [key, value] of Object.entries(modified)) {
      if (typeof value === "string") {
        // Remove email addresses
        modified[key] = value.replace(
          /\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b/g,
          "[EMAIL]"
        );
        // Remove phone numbers
        modified[key] = value.replace(
          /\b\d{3}[-.]?\d{3}[-.]?\d{4}\b/g,
          "[PHONE]"
        );
        // Remove SSN patterns
        modified[key] = value.replace(/\b\d{3}-\d{2}-\d{4}\b/g, "[SSN]");
        // Remove credit card patterns
        modified[key] = value.replace(
          /\b\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}\b/g,
          "[CARD]"
        );
      }
    }

    // Anonymize user identifiers
    if (modified.userId) {
      modified.userId = this.hashString(modified.userId);
    }

    return modified;
  }

  /**
   * Apply reliability-related modifications
   */
  private applyReliabilityModifications(payload: any): any {
    const modified = { ...payload };

    // Set minimum timeout if zero or undefined
    if (!modified.timeout || modified.timeout <= 0) {
      modified.timeout = 5000; // Default 5 seconds
    }

    // Limit resource allocations for reliability
    if (modified.memoryLimit && modified.memoryLimit > 512) {
      modified.memoryLimit = 512; // Cap at 512MB
    }

    if (modified.timeout && modified.timeout > 30000) {
      modified.timeout = 30000; // Cap at 30 seconds
    }

    if (modified.maxConcurrent && modified.maxConcurrent > 10) {
      modified.maxConcurrent = 10; // Cap at 10 concurrent operations
    }

    // Limit retry count
    if (modified.retries && modified.retries > 10) {
      modified.retries = 10; // Cap at 10 retries
    }

    // Limit batch size
    if (modified.batchSize && modified.batchSize > 1000) {
      modified.batchSize = 1000; // Cap at 1000 items
    }

    // Ensure retry configuration
    if (!modified.retryCount && !modified.retries) {
      modified.retryCount = 3;
    }

    return modified;
  }

  /**
   * Sanitize operation payload
   */
  private sanitizeParameters(payload: any): any {
    if (!payload || typeof payload !== "object") {
      // If it's a string, sanitize it
      if (typeof payload === "string") {
        return this.sanitizeString(payload);
      }
      return payload;
    }

    if (Array.isArray(payload)) {
      return payload.map((item) => this.sanitizeParameters(item));
    }

    const sanitized = { ...payload };

    // Remove potentially dangerous parameters
    const dangerousKeys = ["eval", "exec", "shell", "command", "script"];
    for (const key of dangerousKeys) {
      if (sanitized[key]) {
        delete sanitized[key];
      }
    }

    // Recursively sanitize nested objects and arrays
    for (const [key, value] of Object.entries(sanitized)) {
      if (typeof value === "string") {
        sanitized[key] = this.sanitizeString(value);
      } else if (typeof value === "object" && value !== null) {
        sanitized[key] = this.sanitizeParameters(value);
      }
    }

    return sanitized;
  }

  /**
   * Hash a string for anonymization
   */
  private hashString(value: string): string {
    // Simple hash for anonymization (not cryptographically secure)
    let hash = 0;
    for (let i = 0; i < value.length; i++) {
      const char = value.charCodeAt(i);
      hash = (hash << 5) - hash + char;
      hash = hash & hash; // Convert to 32-bit integer
    }
    return Math.abs(hash).toString(16);
  }

  /**
   * Sanitize file paths to prevent directory traversal
   */
  private sanitizeFilePath(filePath: string): string {
    // Remove dangerous path components
    return filePath
      .replace(/\.\./g, "") // Remove parent directory references
      .replace(/^[\/\\]+/, "") // Remove leading slashes
      .replace(/[\/\\]+$/, ""); // Remove trailing slashes
  }

  /**
   * Restrict permissions to safe levels
   */
  private restrictPermissions(permissions: any): any {
    // If permissions object/array, restrict to read-only
    if (Array.isArray(permissions)) {
      return permissions.filter((p) => p === "read");
    }

    if (typeof permissions === "object") {
      return { read: true, write: false, execute: false };
    }

    // If it's a string, restrict to readonly
    if (typeof permissions === "string") {
      return "readonly";
    }

    return permissions;
  }

  /**
   * Sanitize string values
   */
  private sanitizeString(value: string): string {
    if (typeof value !== "string") {
      return value;
    }

    return (
      value
        // XSS prevention
        .replace(/<script\b[^<]*(?:(?!<\/script>)<[^<]*)*<\/script>/gi, "")
        .replace(/javascript:/gi, "")
        .replace(/on\w+\s*=/gi, "")
        // SQL injection prevention
        .replace(/;\s*drop\s+table\s+/gi, "")
        .replace(/;\s*delete\s+from\s+/gi, "")
        .replace(/;\s*update\s+set\s+/gi, "")
        .replace(/;\s*insert\s+into\s+/gi, "")
        .replace(/--.*$/gm, "")
        .replace(/\/\*[\s\S]*?\*\//g, "")
        // Command injection prevention
        .replace(/;\s*rm\s+-rf\s+/gi, "")
        .replace(/;\s*cat\s+\/etc\/passwd/gi, "")
        .replace(/;\s*cat\s+\/etc\/shadow/gi, "")
        .replace(/;\s*wget\s+/gi, "")
        .replace(/;\s*curl\s+/gi, "")
        .replace(/;\s*nc\s+/gi, "")
        .replace(/;\s*netcat\s+/gi, "")
        // Remove dangerous function calls - use a more aggressive approach
        .replace(/eval\s*\([^)]*\)/gi, "[BLOCKED]")
        .replace(/exec\s*\([^)]*\)/gi, "[BLOCKED]")
        .replace(/system\s*\([^)]*\)/gi, "[BLOCKED]")
        .replace(/shell_exec\s*\([^)]*\)/gi, "[BLOCKED]")
        // Remove any remaining eval/exec references
        .replace(/eval/gi, "[BLOCKED]")
        .replace(/exec/gi, "[BLOCKED]")
        .replace(/system/gi, "[BLOCKED]")
        .replace(/shell_exec/gi, "[BLOCKED]")
        // Remove dangerous file paths
        .replace(/\.\.\//g, "")
        .replace(/\.\.\\/g, "")
        .replace(/\/etc\/passwd/gi, "")
        .replace(/\/etc\/shadow/gi, "")
        .replace(/C:\\Windows\\System32/gi, "")
    );
  }

  /**
   * Update configuration
   */
  updateConfig(newConfig: Partial<ViolationConfig>): void {
    this.config = { ...this.config, ...newConfig };
  }
}

// TODO: Implement these interfaces
// Placeholder interfaces for dependencies
export interface AlertManager {
  sendAlert(_alert: AlertMessage): Promise<void>;
  escalateViolation(
    _violation: ConstitutionalViolation,
    _operation: OperationContext,
    _context: EvaluationContext
  ): Promise<void>;
}

export interface AuditLogger {
  logViolation(
    _violation: ConstitutionalViolation,
    _operation: OperationContext,
    _context: EvaluationContext
  ): Promise<void>;
}
