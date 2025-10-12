/**
 * MCP Server Types
 *
 * Type definitions for Arbiter MCP server.
 *
 * @author @darianrosebrook
 */

import type { WorkingSpec } from "../../types/caws-types.js";

/**
 * Arbiter MCP tool names
 */
export type ArbiterToolName =
  | "arbiter_validate"
  | "arbiter_assign_task"
  | "arbiter_monitor_progress"
  | "arbiter_generate_verdict";

/**
 * MCP tool response format
 */
export interface MCPToolResponse {
  content: Array<{
    type: "text" | "image" | "resource";
    text?: string;
    data?: string;
    mimeType?: string;
  }>;
  isError?: boolean;
}

/**
 * Arbiter validate tool arguments
 */
export interface ArbiterValidateArgs {
  /** Working spec to validate */
  spec?: WorkingSpec;
  /** Path to spec file */
  specPath?: string;
  /** Project root directory */
  projectRoot?: string;
  /** Enable auto-fix */
  autoFix?: boolean;
  /** Show suggestions */
  suggestions?: boolean;
  /** Orchestration context */
  orchestrationContext?: {
    taskId?: string;
    agentId?: string;
    timestamp?: string;
  };
}

/**
 * Arbiter assign task tool arguments
 */
export interface ArbiterAssignTaskArgs {
  /** Task working spec */
  spec: WorkingSpec;
  /** Available agent IDs */
  availableAgents?: string[];
  /** Agent selection strategy */
  strategy?: "capability" | "performance" | "round-robin" | "least-loaded";
  /** Task priority */
  priority?: "low" | "medium" | "high" | "critical";
  /** Orchestration context */
  orchestrationContext?: {
    taskId?: string;
    agentId?: string;
    timestamp?: string;
  };
}

/**
 * Arbiter monitor progress tool arguments
 */
export interface ArbiterMonitorProgressArgs {
  /** Task ID to monitor */
  taskId: string;
  /** Project root directory */
  projectRoot?: string;
  /** Include detailed metrics */
  detailed?: boolean;
  /** Budget thresholds to check */
  thresholds?: {
    warning?: number; // 0-1 (e.g., 0.8 for 80%)
    critical?: number; // 0-1 (e.g., 0.95 for 95%)
  };
}

/**
 * Arbiter generate verdict tool arguments
 */
export interface ArbiterGenerateVerdictArgs {
  /** Task ID for verdict */
  taskId: string;
  /** Working spec used */
  spec: WorkingSpec;
  /** Final implementation artifacts */
  artifacts?: {
    filesChanged?: string[];
    testsAdded?: number;
    coverage?: number;
    mutationScore?: number;
  };
  /** Quality gate results */
  qualityGates?: {
    gate: string;
    passed: boolean;
    score?: number;
    details?: string;
  }[];
  /** Agent ID who completed the task */
  agentId?: string;
}

/**
 * Arbiter validation result
 */
export interface ArbiterValidationResult {
  /** Validation success */
  success: boolean;
  /** Is spec valid */
  valid: boolean;
  /** Validation errors */
  errors: Array<{
    field: string;
    message: string;
    severity: "error" | "warning";
  }>;
  /** Validation warnings */
  warnings?: Array<{
    field: string;
    message: string;
  }>;
  /** Suggestions for improvement */
  suggestions?: string[];
  /** CAWS version used */
  cawsVersion: string;
  /** Validation duration */
  durationMs: number;
  /** Orchestration metadata */
  orchestrationContext?: {
    taskId?: string;
    agentId?: string;
    timestamp?: string;
  };
}

/**
 * Task assignment result
 */
export interface TaskAssignmentResult {
  /** Assignment success */
  success: boolean;
  /** Assigned agent ID */
  agentId: string;
  /** Agent name */
  agentName: string;
  /** Assignment reason */
  reason: string;
  /** Agent capabilities matched */
  capabilitiesMatched: string[];
  /** Estimated effort */
  estimatedEffort?: {
    hours: number;
    confidence: number;
  };
  /** Task priority */
  priority: "low" | "medium" | "high" | "critical";
}

/**
 * Progress monitoring result
 */
export interface ProgressMonitoringResult {
  /** Task ID */
  taskId: string;
  /** Current status */
  status: "pending" | "in_progress" | "completed" | "blocked";
  /** Budget usage */
  budgetUsage: {
    files: {
      current: number;
      limit: number;
      percentage: number;
    };
    loc: {
      current: number;
      limit: number;
      percentage: number;
    };
  };
  /** Active alerts */
  alerts: Array<{
    severity: "info" | "warning" | "critical";
    message: string;
    threshold?: number;
  }>;
  /** Acceptance criteria progress */
  acceptanceCriteria: Array<{
    id: string;
    status: "pending" | "in_progress" | "completed";
    testsWritten?: number;
    testsPassing?: number;
    coverage?: number;
  }>;
  /** Overall progress percentage */
  overallProgress: number;
  /** Time tracking */
  timeTracking?: {
    started: string;
    estimated: string;
    remaining?: string;
  };
}

/**
 * Arbiter verdict result
 */
export interface ArbiterVerdictResult {
  /** Verdict decision */
  decision: "approved" | "rejected" | "conditional";
  /** Task ID */
  taskId: string;
  /** Agent ID */
  agentId: string;
  /** Overall quality score */
  qualityScore: number; // 0-100
  /** Quality gates summary */
  qualityGates: {
    total: number;
    passed: number;
    failed: number;
    details: Array<{
      gate: string;
      passed: boolean;
      score?: number;
      message: string;
    }>;
  };
  /** Budget compliance */
  budgetCompliance: {
    filesWithinBudget: boolean;
    locWithinBudget: boolean;
    waiversUsed: string[];
  };
  /** Recommendations */
  recommendations?: string[];
  /** Required actions (if conditional) */
  requiredActions?: string[];
  /** Verdict timestamp */
  timestamp: string;
}
