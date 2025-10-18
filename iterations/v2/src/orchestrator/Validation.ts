/**
 * @fileoverview Input Validation utilities for Arbiter Orchestration (ARBITER-005)
 *
 * Provides comprehensive validation for all critical data structures
 * to prevent runtime errors and ensure data integrity.
 *
 * @author @darianrosebrook
 */

import {
  AgentProfile,
  RoutingDecision,
  Task,
  TaskType,
} from "../types/arbiter-orchestration";

// Re-export commonly used types
export { VerificationPriority } from "../types/verification";

/**
 * Validation result
 */
export interface ValidationResult {
  isValid: boolean;
  errors: ValidationError[];
  warnings: ValidationWarning[];
}

/**
 * Validation error
 */
export interface ValidationError {
  field: string;
  code: string;
  message: string;
  value?: any;
}

/**
 * Validation warning
 */
export interface ValidationWarning {
  field: string;
  code: string;
  message: string;
  value?: any;
}

/**
 * Validation utilities for Arbiter Orchestration
 */
export class ValidationUtils {
  /**
   * Validate a Task object
   */
  static validateTask(task: any): ValidationResult {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];

    // Required fields
    if (!task) {
      errors.push({
        field: "task",
        code: "REQUIRED",
        message: "Task is required",
      });
      return { isValid: false, errors, warnings };
    }

    if (
      !task.id ||
      typeof task.id !== "string" ||
      task.id.trim().length === 0
    ) {
      errors.push({
        field: "id",
        code: "INVALID_ID",
        message: "Task ID must be a non-empty string",
        value: task.id,
      });
    }

    if (
      !task.description ||
      typeof task.description !== "string" ||
      task.description.trim().length === 0
    ) {
      errors.push({
        field: "description",
        code: "INVALID_DESCRIPTION",
        message: "Task description must be a non-empty string",
        value: task.description,
      });
    }

    // Task type validation
    const validTypes: TaskType[] = [
      "code-review",
      "analysis",
      "research",
      "validation",
      "general",
    ];
    if (!validTypes.includes(task.type)) {
      errors.push({
        field: "type",
        code: "INVALID_TYPE",
        message: `Task type must be one of: ${validTypes.join(", ")}`,
        value: task.type,
      });
    }

    // Priority validation
    if (
      typeof task.priority !== "number" ||
      task.priority < 1 ||
      task.priority > 10
    ) {
      errors.push({
        field: "priority",
        code: "INVALID_PRIORITY",
        message: "Task priority must be a number between 1 and 10",
        value: task.priority,
      });
    }

    // Timeout validation
    if (typeof task.timeoutMs !== "number" || task.timeoutMs <= 0) {
      errors.push({
        field: "timeoutMs",
        code: "INVALID_TIMEOUT",
        message: "Task timeout must be a positive number",
        value: task.timeoutMs,
      });
    }

    // Attempts validation
    if (typeof task.attempts !== "number" || task.attempts < 0) {
      errors.push({
        field: "attempts",
        code: "INVALID_ATTEMPTS",
        message: "Task attempts must be a non-negative number",
        value: task.attempts,
      });
    }

    // Max attempts validation
    if (typeof task.maxAttempts !== "number" || task.maxAttempts <= 0) {
      errors.push({
        field: "maxAttempts",
        code: "INVALID_MAX_ATTEMPTS",
        message: "Task maxAttempts must be a positive number",
        value: task.maxAttempts,
      });
    }

    // Budget validation
    if (task.budget) {
      if (
        typeof task.budget.maxFiles !== "number" ||
        task.budget.maxFiles <= 0
      ) {
        errors.push({
          field: "budget.maxFiles",
          code: "INVALID_BUDGET_FILES",
          message: "Budget maxFiles must be a positive number",
          value: task.budget.maxFiles,
        });
      }

      if (typeof task.budget.maxLoc !== "number" || task.budget.maxLoc <= 0) {
        errors.push({
          field: "budget.maxLoc",
          code: "INVALID_BUDGET_LOC",
          message: "Budget maxLoc must be a positive number",
          value: task.budget.maxLoc,
        });
      }
    }

    // CreatedAt validation
    if (!(task.createdAt instanceof Date) || isNaN(task.createdAt.getTime())) {
      errors.push({
        field: "createdAt",
        code: "INVALID_CREATED_AT",
        message: "Task createdAt must be a valid Date object",
        value: task.createdAt,
      });
    }

    // Metadata validation (optional)
    if (task.metadata && typeof task.metadata !== "object") {
      warnings.push({
        field: "metadata",
        code: "INVALID_METADATA_TYPE",
        message: "Task metadata should be an object",
        value: task.metadata,
      });
    }

    return {
      isValid: errors.length === 0,
      errors,
      warnings,
    };
  }

  /**
   * Validate an AgentProfile object
   */
  static validateAgentProfile(profile: any): ValidationResult {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];

    if (!profile) {
      errors.push({
        field: "profile",
        code: "REQUIRED",
        message: "Agent profile is required",
      });
      return { isValid: false, errors, warnings };
    }

    // Required string fields
    const requiredStrings = ["id", "name", "modelFamily"];
    for (const field of requiredStrings) {
      if (
        !profile[field] ||
        typeof profile[field] !== "string" ||
        profile[field].trim().length === 0
      ) {
        errors.push({
          field,
          code: `INVALID_${field.toUpperCase()}`,
          message: `Agent ${field} must be a non-empty string`,
          value: profile[field],
        });
      }
    }

    // Capabilities validation
    if (!profile.capabilities || typeof profile.capabilities !== "object") {
      errors.push({
        field: "capabilities",
        code: "INVALID_CAPABILITIES",
        message: "Agent capabilities must be an object",
        value: profile.capabilities,
      });
    }

    // Performance history validation
    if (!Array.isArray(profile.performanceHistory)) {
      errors.push({
        field: "performanceHistory",
        code: "INVALID_PERFORMANCE_HISTORY",
        message: "Performance history must be an array",
        value: profile.performanceHistory,
      });
    }

    return {
      isValid: errors.length === 0,
      errors,
      warnings,
    };
  }

  /**
   * Validate a RoutingDecision object
   */
  static validateRoutingDecision(decision: any): ValidationResult {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];

    if (!decision) {
      errors.push({
        field: "decision",
        code: "REQUIRED",
        message: "Routing decision is required",
      });
      return { isValid: false, errors, warnings };
    }

    if (!decision.id || typeof decision.id !== "string") {
      errors.push({
        field: "id",
        code: "INVALID_ID",
        message: "Routing decision ID must be a string",
        value: decision.id,
      });
    }

    if (!decision.selectedAgent) {
      errors.push({
        field: "selectedAgent",
        code: "MISSING_AGENT",
        message: "Selected agent is required",
        value: decision.selectedAgent,
      });
    } else {
      const agentValidation = this.validateAgentProfile(decision.selectedAgent);
      if (!agentValidation.isValid) {
        errors.push({
          field: "selectedAgent",
          code: "INVALID_AGENT",
          message: "Selected agent is invalid",
          value: decision.selectedAgent,
        });
      }
    }

    if (
      typeof decision.confidence !== "number" ||
      decision.confidence < 0 ||
      decision.confidence > 1
    ) {
      errors.push({
        field: "confidence",
        code: "INVALID_CONFIDENCE",
        message: "Confidence must be a number between 0 and 1",
        value: decision.confidence,
      });
    }

    if (!decision.strategy || typeof decision.strategy !== "string") {
      errors.push({
        field: "strategy",
        code: "INVALID_STRATEGY",
        message: "Routing strategy must be a string",
        value: decision.strategy,
      });
    }

    if (!decision.reason || typeof decision.reason !== "string") {
      errors.push({
        field: "reason",
        code: "INVALID_REASON",
        message: "Routing reason must be a string",
        value: decision.reason,
      });
    }

    return {
      isValid: errors.length === 0,
      errors,
      warnings,
    };
  }

  /**
   * Validate a TaskAssignment object
   */
  static validateTaskAssignment(assignment: any): ValidationResult {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];

    if (!assignment) {
      errors.push({
        field: "assignment",
        code: "REQUIRED",
        message: "Task assignment is required",
      });
      return { isValid: false, errors, warnings };
    }

    if (!assignment.id || typeof assignment.id !== "string") {
      errors.push({
        field: "id",
        code: "INVALID_ID",
        message: "Assignment ID must be a string",
        value: assignment.id,
      });
    }

    // Validate task
    if (!assignment.task) {
      errors.push({
        field: "task",
        code: "MISSING_TASK",
        message: "Assignment task is required",
      });
    } else {
      const taskValidation = this.validateTask(assignment.task);
      if (!taskValidation.isValid) {
        errors.push({
          field: "task",
          code: "INVALID_TASK",
          message: "Assignment task is invalid",
        });
      }
    }

    // Validate agent
    if (!assignment.agent) {
      errors.push({
        field: "agent",
        code: "MISSING_AGENT",
        message: "Assignment agent is required",
      });
    } else {
      const agentValidation = this.validateAgentProfile(assignment.agent);
      if (!agentValidation.isValid) {
        errors.push({
          field: "agent",
          code: "INVALID_AGENT",
          message: "Assignment agent is invalid",
        });
      }
    }

    // Validate routing decision
    if (!assignment.routingDecision) {
      errors.push({
        field: "routingDecision",
        code: "MISSING_ROUTING_DECISION",
        message: "Assignment routing decision is required",
      });
    } else {
      const decisionValidation = this.validateRoutingDecision(
        assignment.routingDecision
      );
      if (!decisionValidation.isValid) {
        errors.push({
          field: "routingDecision",
          code: "INVALID_ROUTING_DECISION",
          message: "Assignment routing decision is invalid",
        });
      }
    }

    // Validate timestamps
    if (!(assignment.assignedAt instanceof Date)) {
      errors.push({
        field: "assignedAt",
        code: "INVALID_ASSIGNED_AT",
        message: "AssignedAt must be a Date object",
        value: assignment.assignedAt,
      });
    }

    if (!(assignment.deadline instanceof Date)) {
      errors.push({
        field: "deadline",
        code: "INVALID_DEADLINE",
        message: "Deadline must be a Date object",
        value: assignment.deadline,
      });
    }

    return {
      isValid: errors.length === 0,
      errors,
      warnings,
    };
  }

  /**
   * Validate TaskQueue configuration
   */
  static validateTaskQueueConfig(config: any): ValidationResult {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];

    if (!config) {
      errors.push({
        field: "config",
        code: "REQUIRED",
        message: "TaskQueue configuration is required",
      });
      return { isValid: false, errors, warnings };
    }

    // Capacity validation
    if (typeof config.maxCapacity !== "number" || config.maxCapacity <= 0) {
      errors.push({
        field: "maxCapacity",
        code: "INVALID_CAPACITY",
        message: "Max capacity must be a positive number",
        value: config.maxCapacity,
      });
    }

    // Timeout validation
    if (
      typeof config.defaultTimeoutMs !== "number" ||
      config.defaultTimeoutMs <= 0
    ) {
      errors.push({
        field: "defaultTimeoutMs",
        code: "INVALID_TIMEOUT",
        message: "Default timeout must be a positive number",
        value: config.defaultTimeoutMs,
      });
    }

    // Max retries validation
    if (typeof config.maxRetries !== "number" || config.maxRetries < 0) {
      errors.push({
        field: "maxRetries",
        code: "INVALID_MAX_RETRIES",
        message: "Max retries must be a non-negative number",
        value: config.maxRetries,
      });
    }

    return {
      isValid: errors.length === 0,
      errors,
      warnings,
    };
  }

  /**
   * Format validation result as readable string
   */
  static formatValidationResult(result: ValidationResult): string {
    const lines: string[] = [];

    if (result.isValid) {
      lines.push("✅ Validation passed");
    } else {
      lines.push("❌ Validation failed");
      lines.push("");

      if (result.errors.length > 0) {
        lines.push("Errors:");
        result.errors.forEach((error, i) => {
          lines.push(`  ${i + 1}. ${error.field}: ${error.message}`);
          if (error.value !== undefined) {
            lines.push(`     Value: ${JSON.stringify(error.value)}`);
          }
        });
      }

      if (result.warnings.length > 0) {
        lines.push("");
        lines.push("Warnings:");
        result.warnings.forEach((warning, i) => {
          lines.push(`  ${i + 1}. ${warning.field}: ${warning.message}`);
        });
      }
    }

    return lines.join("\n");
  }
}

/**
 * Guard function that throws on validation failure
 */
export function validateTask(task: any): asserts task is Task {
  const result = ValidationUtils.validateTask(task);
  if (!result.isValid) {
    throw new Error(
      `Task validation failed:\n${ValidationUtils.formatValidationResult(
        result
      )}`
    );
  }
}

/**
 * Guard function for agent profiles
 */
export function validateAgentProfile(
  profile: any
): asserts profile is AgentProfile {
  const result = ValidationUtils.validateAgentProfile(profile);
  if (!result.isValid) {
    throw new Error(
      `Agent profile validation failed:\n${ValidationUtils.formatValidationResult(
        result
      )}`
    );
  }
}

/**
 * Guard function for routing decisions
 */
export function validateRoutingDecision(
  decision: any
): asserts decision is RoutingDecision {
  const result = ValidationUtils.validateRoutingDecision(decision);
  if (!result.isValid) {
    throw new Error(
      `Routing decision validation failed:\n${ValidationUtils.formatValidationResult(
        result
      )}`
    );
  }
}
