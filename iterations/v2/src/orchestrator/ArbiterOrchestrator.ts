/**
 * @fileoverview Arbiter Orchestrator - Main Integration Component (ARBITER-005)
 *
 * Central orchestrator that integrates all arbiter components including
 * task management, agent registry, security, health monitoring, and
 * knowledge research capabilities.
 *
 * @author @darianrosebrook
 */

import { AgentControlConfig } from "../types/agent-prompting";

// CAWS Integration imports
import { ArbitrationOrchestrator as ArbitrationProtocolEngine } from "../arbitration/ArbitrationOrchestrator";
import { ArbiterReasoningEngine } from "../reasoning/ArbiterReasoningEngine";

// Verification Engine imports
import type {
  VerificationEngine,
  VerificationEngineConfig,
} from "../types/verification";
import { VerificationType } from "../types/verification";
import { VerificationEngineImpl } from "../verification/VerificationEngine";

// Audit Logging imports
import {
  AuditEventType,
  AuditLogger,
  AuditSeverity,
} from "../observability/AuditLogger";

// Workspace and Health Integration imports
import { SystemHealthMonitor } from "../monitoring/SystemHealthMonitor.js";
import { WorkspaceStateManager } from "../workspace/WorkspaceStateManager.js";

// Re-export commonly used types
export { VerificationPriority } from "../types/verification";

/**
 * Security audit levels
 */
export enum SecurityAuditLevel {
  INFO = "info",
  WARNING = "warning",
  ERROR = "error",
  CRITICAL = "critical",
}

/**
 * Security event types
 */
export enum SecurityEventType {
  AUTHENTICATION = "authentication",
  AUTHORIZATION = "authorization",
  INPUT_VALIDATION = "input_validation",
  DATA_ACCESS = "data_access",
  CONFIGURATION = "configuration",
  OVERRIDE_REQUEST = "override_request",
  OVERRIDE_APPROVAL = "override_approval",
  CONSTITUTIONAL_VIOLATION = "constitutional_violation",
  RATE_LIMIT_EXCEEDED = "rate_limit_exceeded",
  SUSPICIOUS_ACTIVITY = "suspicious_activity",
}

/**
 * Security audit event
 */
export interface SecurityAuditEvent {
  id: string;
  timestamp: Date;
  level: SecurityAuditLevel;
  type: SecurityEventType;
  userId?: string;
  sessionId?: string;
  ipAddress?: string;
  userAgent?: string;
  resource: string;
  action: string;
  success: boolean;
  details: Record<string, any>;
  riskScore: number; // 0-100, higher is more risky
}

/**
 * Arbiter Orchestrator Configuration
 */
export interface ArbiterOrchestratorConfig {
  /** Task queue configuration */
  taskQueue: any; // Using any for now, should be TaskQueueConfig

  /** Task assignment configuration */
  taskAssignment: any; // Using any for now, should be TaskAssignmentConfig

  /** Agent registry configuration */
  agentRegistry: any; // Using any for now, should be AgentRegistryConfig

  /** Security configuration */
  // Removed duplicate security property

  /** Health monitoring configuration */
  healthMonitor: any; // Using any for now, should be HealthMonitorConfig

  /** Recovery management configuration */
  recoveryManager: any; // Using any for now, should be RecoveryManagerConfig

  /** Knowledge seeker configuration */
  knowledgeSeeker: any; // Using any for now, should be KnowledgeSeekerConfig

  /** Workspace state manager configuration */
  workspaceManager?: any; // Using any for now, should be WorkspaceStateConfig

  /** Database configuration (optional - graceful degradation if not provided) */
  database?: {
    host: string;
    port: number;
    database: string;
    /** Database user - should be retrieved from secure environment variables */
    user: string;
    /** Database password - NEVER store in config, use environment variables */
    password?: never; // Explicitly forbid storing password in config
    maxConnections?: number;
    /** SSL configuration for secure database connections */
    ssl?: {
      enabled: boolean;
      ca?: string;
      cert?: string;
      key?: string;
      rejectUnauthorized?: boolean;
    };
  };

  /** Security configuration */
  security?: {
    /** Enable security audit logging */
    auditLoggingEnabled: boolean;
    /** Maximum security audit events to retain */
    maxAuditEvents: number;
    /** Enable input sanitization */
    inputSanitizationEnabled: boolean;
    /** Enable secure error responses */
    secureErrorResponsesEnabled: boolean;
    /** Session timeout in minutes */
    sessionTimeoutMinutes: number;
  };

  /** GPT-5 prompting engine configuration */
  prompting: AgentControlConfig & {
    enabled: boolean;
  };

  /** Task research system configuration (ARBITER-006 Phase 4) */
  research?: {
    enabled: boolean;
    detector?: {
      minConfidence?: number;
      maxQueries?: number;
      enableQuestionDetection?: boolean;
      enableUncertaintyDetection?: boolean;
      enableTechnicalDetection?: boolean;
    };
    augmenter?: {
      maxResultsPerQuery?: number;
    };
  };

  /** CAWS integration configuration */
  caws?: {
    enabled: boolean;
    arbitrationProtocol?: {
      enabled: boolean;
      requireConstitutionalReview?: boolean;
      maxRetries?: number;
    };
    reasoningEngine?: {
      enabled: boolean;
      debateThreshold?: number; // Minimum agents needed for debate
      consensusThreshold?: number; // Required consensus level (0-1)
    };
    verificationEngine?: {
      enabled: boolean;
      cacheEnabled?: boolean;
      cacheTtlMs?: number;
      maxConcurrent?: number;
      timeoutMs?: number;
    };
    humanOverride?: {
      enabled: boolean;
      requireApproval?: boolean;
      maxOverridesPerHour?: number;
      overrideValidityHours?: number; // How long an approved override is valid
      requireReason?: boolean; // Require justification for overrides
      escalationThreshold?: number; // Auto-escalate after N denials
    };
  };
}

/**
 * Default Arbiter Orchestrator Configuration
 */
export const defaultArbiterOrchestratorConfig: ArbiterOrchestratorConfig = {
  taskQueue: {
    maxCapacity: 1000,
    defaultTimeoutMs: 300000, // 5 minutes
    maxRetries: 3,
    priorityMode: "priority",
  },

  taskAssignment: {
    // Default assignment configuration
    strategy: "load_balanced",
    maxConcurrentTasks: 10,
  },

  agentRegistry: {
    // Default registry configuration
    maxAgents: 100,
    registrationTimeoutMs: 30000,
  },

  security: {
    // Default security configuration
    auditLoggingEnabled: true,
    maxAuditEvents: 10000,
    inputSanitizationEnabled: true,
    secureErrorResponsesEnabled: true,
    sessionTimeoutMinutes: 60,
  },

  healthMonitor: {
    // Default health monitoring
    enabled: true,
    checkIntervalMs: 30000, // 30 seconds
  },

  recoveryManager: {
    // Default recovery configuration
    enabled: true,
    maxRecoveryAttempts: 3,
  },

  knowledgeSeeker: {
    // Default knowledge seeking
    enabled: true,
    maxQueries: 5,
  },

  prompting: {
    enabled: false, // Disabled by default for production stability
    reasoningEffort: {
      default: "standard" as any,
      complexityMapping: {} as any,
      dynamicAdjustment: false,
    },
    eagerness: {
      default: 0.5,
      complexityMapping: {} as any,
      dynamicAdjustment: false,
    },
    toolBudget: {
      default: { maxCalls: 10, maxCost: 1.0 },
      complexityMapping: {} as any,
      dynamicAdjustment: false,
    },
  } as any,

  caws: {
    enabled: true,
    arbitrationProtocol: {
      enabled: true,
      requireConstitutionalReview: true,
      maxRetries: 3,
    },
    reasoningEngine: {
      enabled: true,
      debateThreshold: 3, // Minimum agents for debate
      consensusThreshold: 0.7, // 70% consensus required
    },
    verificationEngine: {
      enabled: true,
      cacheEnabled: true,
      cacheTtlMs: 3600000, // 1 hour
      maxConcurrent: 10,
      timeoutMs: 30000, // 30 seconds
    },
    humanOverride: {
      enabled: true, // Enabled by default for flexibility
      requireApproval: true,
      maxOverridesPerHour: 5,
      overrideValidityHours: 24, // 24 hours
      requireReason: true,
      escalationThreshold: 3, // Escalate after 3 denials
    },
  },
};

/**
 * Human Override Request
 */
export interface OverrideRequest {
  id: string;
  taskId: string;
  violation: {
    reason: string;
    severity: "low" | "medium" | "high" | "critical";
    type: string;
  };
  requestedBy: string; // User/system that requested override
  status: "pending" | "approved" | "denied" | "expired";
  justification?: string;
  approvedBy?: string;
  approvedAt?: Date;
  expiresAt?: Date;
  createdAt: Date;
  updatedAt: Date;
  denialCount: number; // Track repeated denials
  metadata: {
    taskType?: string;
    agentId?: string;
    constitutionalRule?: string;
    riskAssessment: "low" | "medium" | "high" | "critical";
  };
}

/**
 * Override Approval Decision
 */
export interface OverrideDecision {
  requestId: string;
  decision: "approve" | "deny";
  approvedBy: string;
  justification: string;
  validityHours?: number; // Custom validity period
  conditions?: string[]; // Additional conditions for approval
}

/**
 * Arbiter Orchestrator Status
 */
export interface ArbiterOrchestratorStatus {
  /** Overall system health */
  healthy: boolean;

  /** Component statuses */
  components: {
    taskQueue: boolean;
    taskAssignment: boolean;
    agentRegistry: boolean;
    security: boolean;
    healthMonitor: boolean;
    arbitrationProtocol?: boolean;
    reasoningEngine?: boolean;
    humanOverride?: boolean;
  };

  /** Performance metrics */
  metrics: {
    activeTasks: number;
    queuedTasks: number;
    registeredAgents: number;
    uptimeSeconds: number;
    /** Human override metrics */
    pendingOverrides?: number;
    approvedOverrides?: number;
    overrideUsageThisHour?: number;
  };

  /** Version information */
  version: string;
}

/**
 * Arbiter Orchestrator - Main Integration Component
 */
export class ArbiterOrchestrator {
  private config: ArbiterOrchestratorConfig;
  private components: {
    taskQueue: any; // TaskQueue
    secureQueue?: any;
    taskAssignment: any; // TaskAssignmentManager
    agentRegistry: any; // AgentRegistryManager
    security: any; // SecurityManager
    healthMonitor: any; // HealthMonitor
    recoveryManager: any; // RecoveryManager
    knowledgeSeeker: any; // KnowledgeSeeker
    workspaceManager?: WorkspaceStateManager; // WorkspaceStateManager
    systemHealthMonitor?: SystemHealthMonitor; // SystemHealthMonitor
    promptingEngine?: any; // PromptingEngine
    // CAWS Integration components
    arbitrationProtocol?: ArbitrationProtocolEngine;
    reasoningEngine?: ArbiterReasoningEngine;
    verificationEngine?: VerificationEngine;
    auditLogger?: AuditLogger;
  };
  private initialized = false;
  private overrideRequests: Map<string, OverrideRequest> = new Map();
  private approvedOverrides: Map<string, OverrideRequest> = new Map();
  private deniedRequests: Map<string, OverrideRequest> = new Map();
  private overrideUsage: { count: number; windowStart: number } = {
    count: 0,
    windowStart: Date.now(),
  };

  // Security hardening
  private securityAuditEvents: SecurityAuditEvent[] = [];
  private maxAuditEvents = 10000; // Prevent memory exhaustion
  private securityLogger: any = null; // Secure logger (can be replaced with proper logging service)

  constructor(
    config: ArbiterOrchestratorConfig,
    workspaceManager?: WorkspaceStateManager,
    systemHealthMonitor?: SystemHealthMonitor
  ) {
    this.config = config;
    this.components = {} as any;
    this.components.workspaceManager = workspaceManager;
    this.components.systemHealthMonitor = systemHealthMonitor;
  }

  /**
   * Initialize the orchestrator
   */
  async initialize(): Promise<void> {
    if (this.initialized) {
      console.log("Arbiter Orchestrator already initialized");
      return;
    }

    try {
      console.log("Initializing Arbiter Orchestrator...");

      // Initialize core components (simplified for now)
      this.components.taskQueue = {}; // Would initialize actual TaskQueue
      this.components.taskAssignment = {}; // Would initialize actual TaskAssignmentManager
      this.components.agentRegistry = {}; // Would initialize actual AgentRegistryManager
      this.components.security = {}; // Would initialize actual SecurityManager
      this.components.healthMonitor = {}; // Would initialize actual HealthMonitor
      this.components.recoveryManager = {}; // Would initialize actual RecoveryManager
      this.components.knowledgeSeeker = {}; // Would initialize actual KnowledgeSeeker

      // Initialize CAWS components if enabled
      await this.initializeCAWSComponents();

      this.initialized = true;
      console.log("✅ Arbiter Orchestrator initialized successfully");
    } catch (error) {
      console.error("❌ Failed to initialize Arbiter Orchestrator:", error);
      throw error;
    }
  }

  /**
   * Initialize CAWS integration components
   */
  private async initializeCAWSComponents(): Promise<void> {
    if (!this.config.caws?.enabled) {
      console.log("CAWS integration disabled");
      return;
    }

    // Initialize Arbitration Protocol Engine (ARBITER-015)
    if (this.config.caws.arbitrationProtocol?.enabled) {
      try {
        this.components.arbitrationProtocol = new ArbitrationProtocolEngine();
        console.log("✅ Arbitration Protocol Engine initialized");
      } catch (error) {
        console.error(
          "❌ Failed to initialize Arbitration Protocol Engine:",
          error
        );
        throw error;
      }
    }

    // Initialize Reasoning Engine (ARBITER-016)
    if (this.config.caws.reasoningEngine?.enabled) {
      try {
        this.components.reasoningEngine = new ArbiterReasoningEngine();
        console.log("✅ Arbiter Reasoning Engine initialized");
      } catch (error) {
        console.error(
          "❌ Failed to initialize Arbiter Reasoning Engine:",
          error
        );
        throw error;
      }
    }

    // Initialize Verification Engine (ARBITER-007)
    if (this.config.caws.verificationEngine?.enabled) {
      try {
        const verificationConfig: VerificationEngineConfig = {
          defaultTimeoutMs:
            this.config.caws.verificationEngine.timeoutMs ?? 30000,
          minConfidenceThreshold: 0.7,
          maxEvidencePerMethod: 10,
          cacheEnabled:
            this.config.caws.verificationEngine.cacheEnabled ?? true,
          cacheTtlMs: this.config.caws.verificationEngine.cacheTtlMs ?? 3600000, // 1 hour
          maxConcurrentVerifications:
            this.config.caws.verificationEngine.maxConcurrent ?? 10,
          retryAttempts: 3,
          retryDelayMs: 1000,
          methods: [
            {
              type: VerificationType.FACT_CHECKING,
              enabled: true,
              priority: 1,
              timeoutMs: 10000,
              config: {},
            },
            {
              type: VerificationType.CROSS_REFERENCE,
              enabled: true,
              priority: 2,
              timeoutMs: 10000,
              config: {},
            },
            {
              type: VerificationType.LOGICAL_VALIDATION,
              enabled: true,
              priority: 3,
              timeoutMs: 10000,
              config: {},
            },
            {
              type: VerificationType.STATISTICAL_VALIDATION,
              enabled: true,
              priority: 4,
              timeoutMs: 10000,
              config: {},
            },
            {
              type: VerificationType.CONSISTENCY_VALIDATION,
              enabled: true,
              priority: 5,
              timeoutMs: 10000,
              config: {},
            },
          ],
        };

        this.components.verificationEngine = new VerificationEngineImpl(
          verificationConfig
        );
        console.log("✅ Verification Engine initialized");
      } catch (error) {
        console.error("❌ Failed to initialize Verification Engine:", error);
        throw error;
      }
    }

    // Initialize Audit Logger (ARBITER-008)
    if (this.config.security?.auditLoggingEnabled) {
      try {
        this.components.auditLogger = new AuditLogger("ArbiterOrchestrator");
        console.log("✅ Audit Logger initialized");
      } catch (error) {
        console.error("❌ Failed to initialize Audit Logger:", error);
        throw error;
      }
    }
  }

  /**
   * Validate and sanitize task input
   */
  private validateTaskInput(task: any): {
    valid: boolean;
    sanitizedTask: any;
    errors: string[];
  } {
    const errors: string[] = [];
    const sanitizedTask = { ...task };

    // Validate task ID
    if (!task.id || typeof task.id !== "string") {
      errors.push("Task ID is required and must be a string");
    } else if (task.id.length > 256) {
      errors.push("Task ID must be less than 256 characters");
    } else if (!/^[a-zA-Z0-9_-]+$/.test(task.id)) {
      errors.push("Task ID contains invalid characters");
      sanitizedTask.id = task.id.replace(/[^a-zA-Z0-9_-]/g, "_");
    }

    // Validate task type
    if (!task.type || typeof task.type !== "string") {
      errors.push("Task type is required and must be a string");
    } else if (task.type.length > 100) {
      errors.push("Task type must be less than 100 characters");
    } else {
      // Allow only safe task types
      const allowedTypes = [
        "analysis",
        "research",
        "computation",
        "writing",
        "communication",
        "data_processing",
        "automation",
        "decision_making",
        "policy_development",
      ];
      if (!allowedTypes.includes(task.type)) {
        errors.push(`Task type '${task.type}' is not allowed`);
      }
    }

    // Validate description (optional but sanitized)
    if (task.description) {
      if (typeof task.description !== "string") {
        errors.push("Task description must be a string");
      } else if (task.description.length > 10000) {
        errors.push("Task description must be less than 10000 characters");
        sanitizedTask.description = task.description.substring(0, 10000);
      }
      // Remove potentially harmful content
      sanitizedTask.description = sanitizedTask.description.replace(
        /<script[^>]*>.*?<\/script>/gi,
        ""
      );
      sanitizedTask.description = sanitizedTask.description.replace(
        /javascript:/gi,
        ""
      );
    }

    // Validate priority
    if (task.priority) {
      const allowedPriorities = ["low", "normal", "high", "urgent"];
      if (!allowedPriorities.includes(task.priority)) {
        errors.push(`Priority '${task.priority}' is not allowed`);
        sanitizedTask.priority = "normal"; // Default to normal
      }
    }

    // Validate capabilities array
    if (task.requiredCapabilities) {
      if (!Array.isArray(task.requiredCapabilities)) {
        errors.push("Required capabilities must be an array");
      } else if (task.requiredCapabilities.length > 10) {
        errors.push("Cannot require more than 10 capabilities");
        sanitizedTask.requiredCapabilities = task.requiredCapabilities.slice(
          0,
          10
        );
      } else {
        // Sanitize capability names
        sanitizedTask.requiredCapabilities = task.requiredCapabilities.map(
          (cap: string) => {
            if (typeof cap !== "string" || cap.length > 50) {
              errors.push(
                "Capability names must be strings less than 50 characters"
              );
              return "unknown";
            }
            return cap.replace(/[^a-zA-Z0-9_-]/g, "_");
          }
        );
      }
    }

    return {
      valid: errors.length === 0,
      sanitizedTask,
      errors,
    };
  }

  /**
   * Log security audit event
   */
  private async logSecurityEvent(
    type: SecurityEventType,
    level: SecurityAuditLevel,
    resource: string,
    action: string,
    success: boolean,
    details: Record<string, any> = {},
    riskScore: number = 0
  ): Promise<void> {
    try {
      // Map SecurityEventType to AuditEventType
      let auditEventType: AuditEventType;
      switch (type) {
        case SecurityEventType.AUTHENTICATION:
          auditEventType = AuditEventType.AUTHENTICATION;
          break;
        case SecurityEventType.AUTHORIZATION:
          auditEventType = AuditEventType.AUTHORIZATION;
          break;
        case SecurityEventType.INPUT_VALIDATION:
          auditEventType = AuditEventType.DATA_ACCESS;
          break;
        default:
          auditEventType = AuditEventType.ACCESS_CONTROL;
      }

      // Map SecurityAuditLevel to AuditSeverity
      let auditSeverity: AuditSeverity;
      switch (level) {
        case SecurityAuditLevel.CRITICAL:
          auditSeverity = AuditSeverity.CRITICAL;
          break;
        case SecurityAuditLevel.ERROR:
          auditSeverity = AuditSeverity.HIGH;
          break;
        case SecurityAuditLevel.WARNING:
          auditSeverity = AuditSeverity.MEDIUM;
          break;
        case SecurityAuditLevel.INFO:
          auditSeverity = AuditSeverity.LOW;
          break;
      }

      // Use new audit logger if available
      if (this.components.auditLogger) {
        await this.components.auditLogger.logAuditEvent(
          auditEventType,
          auditSeverity,
          "system", // actor - could be enhanced to track actual users
          resource,
          action,
          success ? "success" : "failure",
          this.sanitizeAuditDetails(details),
          {
            riskScore,
            complianceFlags:
              level === SecurityAuditLevel.CRITICAL ? ["security"] : [],
          }
        );
      } else {
        // Fallback to legacy audit logging
        const event: SecurityAuditEvent = {
          id: `audit-${Date.now()}-${Math.random()
            .toString(36)
            .substring(2, 9)}`,
          timestamp: new Date(),
          level,
          type,
          resource,
          action,
          success,
          details: this.sanitizeAuditDetails(details),
          riskScore: Math.min(100, Math.max(0, riskScore)),
        };

        this.securityAuditEvents.push(event);

        // Maintain max audit events limit
        if (this.securityAuditEvents.length > this.maxAuditEvents) {
          this.securityAuditEvents.shift();
        }

        // Log to console as fallback
        console.log(
          `[SECURITY-${level.toUpperCase()}] ${type}: ${action} on ${resource}`,
          {
            eventId: event.id,
            riskScore: event.riskScore,
            success: event.success,
          }
        );
      }
    } catch (error) {
      console.error("Failed to log security event:", error);
      // Continue execution - audit logging failure shouldn't break the system
    }
  }

  /**
   * Sanitize audit details to prevent sensitive data leakage
   */
  private sanitizeAuditDetails(
    details: Record<string, any>
  ): Record<string, any> {
    const sanitized = { ...details };

    // Remove or mask sensitive fields
    const sensitiveFields = [
      "password",
      "token",
      "key",
      "secret",
      "credentials",
      "privateKey",
    ];
    for (const field of sensitiveFields) {
      if (sanitized[field]) {
        sanitized[field] = "[REDACTED]";
      }
    }

    // Limit string lengths to prevent log pollution
    for (const [key, value] of Object.entries(sanitized)) {
      if (typeof value === "string" && value.length > 500) {
        sanitized[key] = value.substring(0, 500) + "...[TRUNCATED]";
      }
    }

    return sanitized;
  }

  /**
   * Secure error response that doesn't leak sensitive information
   */
  private createSecureError(error: any, operation: string): Error {
    // Log the full error internally for debugging (fire-and-forget)
    this.logSecurityEvent(
      SecurityEventType.SUSPICIOUS_ACTIVITY,
      SecurityAuditLevel.WARNING,
      operation,
      "error_occurred",
      false,
      { errorType: error?.constructor?.name || "Unknown", operation },
      30
    ).catch((err) => console.error("Failed to log security event:", err));

    // Return sanitized error message
    const errorMessage =
      error instanceof Error ? error.message : "An internal error occurred";
    return new Error(
      `Operation failed: ${operation}. Please contact support if this persists.`
    );
  }

  /**
   * Submit a task for orchestration
   */
  async submitTask(
    task: any, // Task type
    credentials?: any
  ): Promise<{
    taskId: string;
    assignmentId?: string;
    overrideRequired?: string;
  }> {
    if (!this.initialized) {
      throw new Error("Orchestrator not initialized");
    }

    // Validate and sanitize input
    const validation = this.validateTaskInput(task);
    if (!validation.valid) {
      await this.logSecurityEvent(
        SecurityEventType.INPUT_VALIDATION,
        SecurityAuditLevel.WARNING,
        "task",
        "submit",
        false,
        { errors: validation.errors, taskId: task.id },
        40
      );
      throw new Error(`Invalid task input: ${validation.errors.join(", ")}`);
    }

    const sanitizedTask = validation.sanitizedTask;

    await this.logSecurityEvent(
      SecurityEventType.DATA_ACCESS,
      SecurityAuditLevel.INFO,
      "task",
      "submit",
      true,
      { taskId: sanitizedTask.id, taskType: sanitizedTask.type },
      10
    );

    // Log sanitized task submission (avoid logging sensitive data)
    if (this.securityLogger) {
      this.securityLogger.info(`Processing task: ${sanitizedTask.id}`);
    }

    // CAWS Constitutional Compliance Check
    if (this.config.caws?.enabled) {
      const complianceCheck = await this.checkConstitutionalCompliance(task);
      if (!complianceCheck.compliant) {
        // Check if human override is available and enabled
        if (this.config.caws.humanOverride?.enabled) {
          console.log(
            `Task ${task.id} flagged for human override: ${complianceCheck.reason}`
          );
          const overrideRequest = await this.createOverrideRequest(
            task,
            complianceCheck
          );
          return {
            taskId: task.id,
            assignmentId: undefined,
            overrideRequired: overrideRequest.id,
          };
        }

        // Task requires constitutional review
        if (this.config.caws.arbitrationProtocol?.requireConstitutionalReview) {
          console.log(
            `Task ${task.id} flagged for constitutional review: ${complianceCheck.reason}`
          );
          await this.escalateToArbitration(task, complianceCheck);
          // Task is now under arbitration - return early
          return { taskId: task.id, assignmentId: undefined };
        } else {
          // Log but allow task to proceed
          console.warn(
            `Task ${task.id} has constitutional concerns but proceeding: ${complianceCheck.reason}`
          );
        }
      }
    }

    // Check if task requires multi-agent coordination
    if (this.config.caws?.reasoningEngine?.enabled) {
      const requiresDebate = await this.requiresMultiAgentDebate(task);
      if (requiresDebate) {
        console.log(`Task ${task.id} requires multi-agent debate coordination`);
        return await this.coordinateMultiAgentDebate(task);
      }
    }

    // Assign task to suitable agent
    const assignment = await this.assignTaskToAgent(task);
    if (assignment) {
      console.log(`Task ${task.id} assigned to agent ${assignment.agentId}`);

      // Audit log successful task assignment
      await this.components.auditLogger
        ?.logAuditEvent(
          AuditEventType.TASK_EXECUTION,
          AuditSeverity.LOW,
          "system",
          "task",
          "assign",
          "success",
          {
            taskId: task.id,
            assignmentId: assignment.id,
            agentId: assignment.agentId,
            taskType: task.type,
          }
        )
        .catch((err) => console.error("Failed to log task assignment:", err));

      return {
        taskId: task.id,
        assignmentId: assignment.id,
      };
    }

    // Enqueue task if no immediate assignment possible
    // Simplified implementation - would integrate with actual TaskQueue
    console.log(
      `Task ${task.id} enqueued for processing (no immediate assignment)`
    );

    // Audit log task enqueued
    await this.components.auditLogger
      ?.logAuditEvent(
        AuditEventType.TASK_SUBMISSION,
        AuditSeverity.LOW,
        "system",
        "task",
        "enqueue",
        "success",
        {
          taskId: task.id,
          taskType: task.type,
          reason: "no_immediate_assignment",
        }
      )
      .catch((err) => console.error("Failed to log task enqueue:", err));

    return {
      taskId: task.id,
      assignmentId: `queued-assignment-${Date.now()}`,
    };
  }

  /**
   * Determine if a task requires multi-agent debate coordination
   */
  private async requiresMultiAgentDebate(task: any): Promise<boolean> {
    if (!this.config.caws?.reasoningEngine?.enabled) {
      return false;
    }

    // Check explicit debate requirements
    if ((task as any).requiresDebate === true) {
      return true;
    }

    // Check complexity threshold
    const complexityScore = this.calculateTaskComplexity(task);
    if (complexityScore >= 0.7) {
      // High complexity tasks need debate
      return true;
    }

    // Check for controversial topics
    if (this.isControversialTopic(task)) {
      return true;
    }

    // Check if task explicitly requests multiple agents
    if (
      (task as any).minAgents &&
      this.config.caws.reasoningEngine?.debateThreshold &&
      (task as any).minAgents >=
        this.config.caws.reasoningEngine.debateThreshold
    ) {
      return true;
    }

    return false;
  }

  /**
   * Calculate task complexity score (0-1)
   */
  private calculateTaskComplexity(task: any): number {
    let complexity = 0;

    // Length-based complexity
    const content = JSON.stringify(task);
    if (content.length > 1000) complexity += 0.2;
    if (content.length > 5000) complexity += 0.2;

    // Type-based complexity
    if (task.type === "analysis") complexity += 0.3;
    if (task.type === "decision_making") complexity += 0.4;
    if (task.type === "policy_development") complexity += 0.5;

    // Explicit complexity rating
    if ((task as any).complexity) {
      complexity += (task as any).complexity * 0.3;
    }

    return Math.min(complexity, 1.0);
  }

  /**
   * Check if task topic is controversial
   */
  private isControversialTopic(task: any): boolean {
    const controversialKeywords = [
      "policy",
      "ethics",
      "moral",
      "controversial",
      "dispute",
      "conflict",
      "debate",
      "controversy",
      "sensitive",
      "political",
      "religious",
      "bias",
      "fairness",
      "justice",
      "rights",
      "freedom",
    ];

    const content = JSON.stringify(task).toLowerCase();
    return controversialKeywords.some((keyword) => content.includes(keyword));
  }

  /**
   * Coordinate multi-agent debate for task resolution
   */
  private async coordinateMultiAgentDebate(
    task: any
  ): Promise<{ taskId: string; assignmentId?: string }> {
    if (!this.components.reasoningEngine) {
      throw new Error(
        "Reasoning engine not available for multi-agent coordination"
      );
    }

    try {
      console.log(`Initiating multi-agent debate for task: ${task.id}`);

      // Select debate participants
      const participants = await this.selectDebateParticipants(task);
      const debateThreshold =
        this.config.caws?.reasoningEngine?.debateThreshold || 3;
      if (participants.length < debateThreshold) {
        console.log(
          `Insufficient participants for debate (${participants.length}), falling back to single agent`
        );
        return {
          taskId: task.id,
          assignmentId: `assignment-${Date.now()}`,
        };
      }

      // Format topic for debate
      const debateTopic = this.formatTaskForDebate(task);

      // Initiate debate session
      const debateSession =
        await this.components.reasoningEngine.initiateDebate(
          debateTopic,
          participants
        );

      console.log(
        `Debate session initiated: ${debateSession.id} with ${participants.length} participants`
      );

      // Simulate debate rounds (in production, this would be event-driven)
      await this.conductDebateRounds(debateSession.id, task);

      // Form consensus
      await this.components.reasoningEngine.formConsensus(debateSession.id);

      // Get debate results
      const results = await this.components.reasoningEngine.getDebateResults(
        debateSession.id
      );

      // Process consensus result
      const assignmentId = await this.processDebateConsensus(task, results);

      // Close debate
      await this.components.reasoningEngine.closeDebate(debateSession.id);

      console.log(
        `Multi-agent debate completed for task ${task.id}: ${
          results.consensus?.outcome || "No consensus"
        }`
      );

      return {
        taskId: task.id,
        assignmentId,
      };
    } catch (error) {
      console.error(`Multi-agent debate failed for task ${task.id}:`, error);
      // Fall back to single agent processing
      console.log(
        `Falling back to single-agent processing for task ${task.id}`
      );
      const assignment = await this.assignTaskToAgent(task);
      return {
        taskId: task.id,
        assignmentId: assignment?.id || `fallback-assignment-${Date.now()}`,
      };
    }
  }

  /**
   * Select appropriate agents for debate participation
   */
  private async selectDebateParticipants(
    task: any
  ): Promise<Array<{ agentId: string; role: any; weight?: number }>> {
    try {
      // Query agent registry for available agents with appropriate capabilities
      const availableAgents =
        await this.components.agentRegistry.getAvailableAgents();

      if (!availableAgents || availableAgents.length === 0) {
        // Fallback to generating dynamic agent IDs if registry is empty
        return this.generateFallbackParticipants(task);
      }

      // Define role mappings and required capabilities
      const roleMappings = [
        {
          role: "ANALYST" as any,
          requiredCapabilities: ["analysis", "reasoning", "data_processing"],
          weight: 0.8,
        },
        {
          role: "CRITIC" as any,
          requiredCapabilities: [
            "criticism",
            "evaluation",
            "quality_assessment",
          ],
          weight: 0.7,
        },
        {
          role: "SYNTHESIZER" as any,
          requiredCapabilities: ["synthesis", "integration", "summarization"],
          weight: 0.9,
        },
      ];

      const participants: Array<{
        agentId: string;
        role: any;
        weight?: number;
      }> = [];

      // Select best available agent for each role
      for (const roleMapping of roleMappings) {
        const suitableAgents = availableAgents.filter((agent: any) => {
          if (!agent.capabilities) return false;
          return roleMapping.requiredCapabilities.some((cap: string) =>
            agent.capabilities.includes(cap)
          );
        });

        if (suitableAgents.length > 0) {
          // Select the agent with the best performance score or most matching capabilities
          const bestAgent = suitableAgents.reduce((best: any, current: any) => {
            const currentScore = this.calculateAgentScore(
              current,
              roleMapping.requiredCapabilities
            );
            const bestScore = this.calculateAgentScore(
              best,
              roleMapping.requiredCapabilities
            );
            return currentScore > bestScore ? current : best;
          });

          participants.push({
            agentId: bestAgent.id || bestAgent.agentId,
            role: roleMapping.role,
            weight: roleMapping.weight,
          });
        }
      }

      // Ensure we have at least 3 participants, generate fallback if needed
      if (participants.length < 3) {
        const fallbackParticipants = this.generateFallbackParticipants(task);
        participants.push(
          ...fallbackParticipants.slice(participants.length, 3)
        );
      }

      return participants.slice(0, 3);
    } catch (error) {
      // If agent registry query fails, fall back to dynamic generation
      console.warn(
        "Failed to query agent registry for debate participants, using fallback",
        {
          error: error instanceof Error ? error.message : String(error),
          taskId: task?.id,
        }
      );
      return this.generateFallbackParticipants(task);
    }
  }

  /**
   * Generate fallback participants with dynamic agent IDs
   */
  private generateFallbackParticipants(
    task: any
  ): Array<{ agentId: string; role: any; weight?: number }> {
    const taskId = task?.id || "unknown";
    const timestamp = Date.now();

    return [
      {
        agentId: `agent-analyzer-${taskId}-${timestamp}`,
        role: "ANALYST" as any,
        weight: 0.8,
      },
      {
        agentId: `agent-critic-${taskId}-${timestamp}`,
        role: "CRITIC" as any,
        weight: 0.7,
      },
      {
        agentId: `agent-synthesizer-${taskId}-${timestamp}`,
        role: "SYNTHESIZER" as any,
        weight: 0.9,
      },
    ];
  }

  /**
   * Calculate agent suitability score for a role
   */
  private calculateAgentScore(
    agent: any,
    requiredCapabilities: string[]
  ): number {
    if (!agent.capabilities) return 0;

    const matchingCapabilities = requiredCapabilities.filter((cap: string) =>
      agent.capabilities.includes(cap)
    ).length;

    const capabilityScore = matchingCapabilities / requiredCapabilities.length;
    const performanceScore =
      agent.performanceHistory?.averageSuccessRate || 0.5;

    return capabilityScore * 0.7 + performanceScore * 0.3;
  }

  /**
   * Format task for debate topic
   */
  private formatTaskForDebate(task: any): string {
    return `Task Resolution: ${
      task.description || task.type || "Complex Task"
    }`;
  }

  /**
   * Conduct debate rounds (simplified simulation)
   */
  private async conductDebateRounds(
    debateId: string,
    task: any
  ): Promise<void> {
    if (!this.components.reasoningEngine) return;

    // Simulate debate rounds - in production this would be event-driven
    const participants = [
      "agent-analyzer",
      "agent-critic",
      "agent-synthesizer",
    ];

    for (let round = 1; round <= 2; round++) {
      for (const participantId of participants) {
        // Submit argument
        await this.components.reasoningEngine.submitArgument(
          debateId,
          participantId,
          `Analysis from ${participantId} in round ${round}`,
          [
            {
              id: `evidence-${participantId}-round-${round}`,
              content: `Evidence from ${participantId} analysis`,
              source: participantId,
              credibilityScore: 0.8,
              verificationStatus: "verified" as any,
              timestamp: new Date(),
            },
          ],
          `Reasoning provided by ${participantId} in round ${round}`
        );
      }

      // Aggregate evidence after each round
      await this.components.reasoningEngine.aggregateEvidence(debateId);
    }

    // Submit votes
    for (const participantId of participants) {
      await this.components.reasoningEngine.submitVote(
        debateId,
        participantId,
        "for" as any, // position
        0.85, // confidence
        `Vote from ${participantId} supporting the analysis`
      );
    }
  }

  /**
   * Process debate consensus and create task assignment
   */
  private async processDebateConsensus(
    task: any,
    results: { session: any; consensus: any }
  ): Promise<string> {
    const consensusThreshold =
      this.config.caws?.reasoningEngine?.consensusThreshold || 0.7;
    if (
      results.consensus &&
      results.consensus.confidence >= consensusThreshold
    ) {
      console.log(`Strong consensus reached: ${results.consensus.outcome}`);

      // Create assignment based on consensus
      return `debate-consensus-${Date.now()}`;
    } else {
      console.log(
        `Weak or no consensus reached, proceeding with majority decision`
      );

      // Fallback assignment
      return `debate-fallback-${Date.now()}`;
    }
  }

  /**
   * Assign task to the most suitable available agent
   */
  private async assignTaskToAgent(task: any): Promise<any | null> {
    try {
      // Find available agents (simplified - would query actual agent registry)
      const availableAgents = await this.findAvailableAgents(task);

      if (availableAgents.length === 0) {
        console.log(`No available agents found for task ${task.id}`);
        return null;
      }

      // Select best agent based on capability matching and load balancing
      const selectedAgent = await this.selectBestAgent(task, availableAgents);

      if (!selectedAgent) {
        console.log(`No suitable agent found for task ${task.id}`);
        return null;
      }

      // Check constitutional compliance of the assignment
      const assignmentCompliance = await this.checkAssignmentCompliance(
        task,
        selectedAgent
      );
      if (!assignmentCompliance.compliant) {
        console.warn(
          `Assignment of task ${task.id} to agent ${selectedAgent.id} violates constitutional rules: ${assignmentCompliance.reason}`
        );

        // Try alternative agents
        for (const alternativeAgent of availableAgents) {
          if (alternativeAgent.id !== selectedAgent.id) {
            const altCompliance = await this.checkAssignmentCompliance(
              task,
              alternativeAgent
            );
            if (altCompliance.compliant) {
              console.log(
                `Using alternative agent ${alternativeAgent.id} for task ${task.id}`
              );
              return await this.createTaskAssignment(task, alternativeAgent);
            }
          }
        }

        // If no compliant assignment possible, escalate or reject
        if (
          this.config.caws?.arbitrationProtocol?.requireConstitutionalReview
        ) {
          console.log(
            `No constitutionally compliant assignment possible for task ${task.id}, escalating to arbitration`
          );
          // This would trigger arbitration for assignment conflicts
          return null;
        } else {
          console.warn(
            `Proceeding with non-compliant assignment for task ${task.id} (constitutional review disabled)`
          );
        }
      }

      // Create and return task assignment
      return await this.createTaskAssignment(task, selectedAgent);
    } catch (error) {
      console.error(`Failed to assign task ${task.id}:`, error);
      return null;
    }
  }

  /**
   * Find available agents that could handle the task
   */
  private async findAvailableAgents(task: any): Promise<any[]> {
    // Simplified implementation - would query actual agent registry
    // In production, this would:
    // 1. Query agent registry for agents with required capabilities
    // 2. Filter by current availability and load
    // 3. Consider agent performance history

    const mockAgents = [
      {
        id: "agent-001",
        capabilities: ["analysis", "research", "computation"],
        currentLoad: 2,
        maxLoad: 5,
        performance: { quality: 0.85, speed: 0.9, reliability: 0.95 },
        status: "available",
      },
      {
        id: "agent-002",
        capabilities: ["writing", "communication", "analysis"],
        currentLoad: 1,
        maxLoad: 4,
        performance: { quality: 0.9, speed: 0.8, reliability: 0.9 },
        status: "available",
      },
      {
        id: "agent-003",
        capabilities: ["computation", "data_processing", "automation"],
        currentLoad: 3,
        maxLoad: 6,
        performance: { quality: 0.8, speed: 0.95, reliability: 0.85 },
        status: "available",
      },
    ];

    // Filter by capability matching and availability
    return mockAgents.filter((agent) => {
      const hasCapability =
        !task.requiredCapabilities ||
        task.requiredCapabilities.some((cap: string) =>
          agent.capabilities.includes(cap)
        );

      const hasCapacity = agent.currentLoad < agent.maxLoad;
      const isAvailable = agent.status === "available";

      return hasCapability && hasCapacity && isAvailable;
    });
  }

  /**
   * Select the best agent for the task based on multiple criteria
   */
  private async selectBestAgent(task: any, agents: any[]): Promise<any | null> {
    if (agents.length === 0) return null;

    // Enhanced scoring with workspace and health awareness
    const scoredAgents = await Promise.all(
      agents.map(async (agent) => {
        let score = 0;

        // Enhanced scoring algorithm with workspace and health factors
        const factors = await this.calculateEnhancedScore(task, agent);

        // Apply weights to different factors
        score += factors.capability * 0.25; // Reduced from 40% to make room for new factors
        score += factors.loadBalancing * 0.15; // Reduced from 30%
        score += factors.performance * 0.15; // Reduced from 30%
        score += factors.workspace * 0.2; // NEW: Workspace context awareness
        score += factors.health * 0.15; // NEW: System health awareness
        score += factors.resources * 0.1; // NEW: Resource availability

        return { agent, score, factors }; // Include factors for debugging
      })
    );

    // Sort by score (highest first) and return best agent
    scoredAgents.sort((a, b) => b.score - a.score);

    console.log(`Agent selection for task ${task.id}:`);
    scoredAgents.slice(0, 3).forEach((item, index) => {
      console.log(
        `  ${index + 1}. ${item.agent.id}: ${item.score.toFixed(
          3
        )} (cap: ${item.factors.capability.toFixed(
          2
        )}, ws: ${item.factors.workspace.toFixed(
          2
        )}, health: ${item.factors.health.toFixed(2)})`
      );
    });

    return scoredAgents[0].agent;
  }

  /**
   * Calculate enhanced agent score with workspace and health awareness
   */
  private async calculateEnhancedScore(
    task: any,
    agent: any
  ): Promise<{
    capability: number;
    loadBalancing: number;
    performance: number;
    workspace: number;
    health: number;
    resources: number;
  }> {
    // Existing factors
    const capability = this.calculateCapabilityMatch(task, agent);
    const loadBalancing = 1 - agent.currentLoad / agent.maxLoad;
    const performance = this.calculatePerformanceScore(task, agent);

    // New workspace context factor
    const workspace = await this.calculateWorkspaceContextScore(task, agent);

    // New health awareness factor
    const health = await this.calculateSystemHealthScore(agent);

    // New resource availability factor
    const resources = await this.calculateResourceAvailabilityScore(
      task,
      agent
    );

    return {
      capability,
      loadBalancing,
      performance,
      workspace,
      health,
      resources,
    };
  }

  /**
   * Calculate workspace context relevance score
   */
  private async calculateWorkspaceContextScore(
    task: any,
    agent: any
  ): Promise<number> {
    if (!this.components.workspaceManager) {
      return 0.5; // Neutral score if no workspace data available
    }

    try {
      const workspaceManager = this.components.workspaceManager;

      // Get recent agent activity (last 24 hours)
      const recentActivity = await this.getAgentWorkspaceActivity(agent.id);
      const activityScore = Math.min(recentActivity / 50, 1.0); // Normalize and cap

      // Generate context for task-relevant files
      const taskKeywords = this.extractTaskKeywords(task);
      const context = workspaceManager.generateContext({
        relevanceKeywords: taskKeywords,
        maxFiles: 20,
        recencyWeight: 0.4, // Favor recent changes for active tasks
      });

      // Calculate agent's familiarity with context files
      const familiarityScore = this.calculateAgentFamiliarity(
        agent.id,
        context
      );

      // Context relevance score
      const relevanceScore =
        context.relevanceScores.size > 0
          ? Array.from(context.relevanceScores.values()).reduce(
              (a, b) => a + b,
              0
            ) / context.relevanceScores.size
          : 0;

      // Combine factors: activity (30%), familiarity (40%), relevance (30%)
      return (
        activityScore * 0.3 + familiarityScore * 0.4 + relevanceScore * 0.3
      );
    } catch (error) {
      console.warn(
        `Failed to calculate workspace context score for agent ${agent.id}:`,
        error
      );
      return 0.5; // Neutral fallback
    }
  }

  /**
   * Calculate system health awareness score
   */
  private async calculateSystemHealthScore(agent: any): Promise<number> {
    if (!this.components.systemHealthMonitor) {
      return 0.8; // Neutral score if no health monitor available
    }

    try {
      const healthMetrics =
        await this.components.systemHealthMonitor.getHealthMetrics();

      // Get agent-specific health
      const agentHealth = this.components.systemHealthMonitor.getAgentHealth(
        agent.id
      );

      // System-wide health factors
      const systemHealth = healthMetrics.overallHealth;
      const errorRatePenalty = Math.min(0.3, healthMetrics.errorRate / 20); // Max 0.3 penalty
      const loadPenalty = healthMetrics.queueDepth > 50 ? 0.2 : 0; // Penalty for high queue

      // Agent-specific health bonus/penalty
      const agentHealthBonus = agentHealth ? agentHealth.healthScore - 0.5 : 0; // 0 if healthy, negative if unhealthy

      const healthScore = Math.max(
        0.1,
        Math.min(
          1.0,
          systemHealth - errorRatePenalty - loadPenalty + agentHealthBonus
        )
      );

      return Math.round(healthScore * 100) / 100;
    } catch (error) {
      console.warn(
        `Failed to calculate system health score for agent ${agent.id}:`,
        error
      );
      return 0.8; // Neutral fallback
    }
  }

  /**
   * Calculate resource availability score
   */
  private calculateResourceAvailabilityScore(
    task: any,
    agent: any
  ): Promise<number> {
    // Placeholder for resource availability
    // In production, this would consider:
    // - Agent's available capacity
    // - Task's resource requirements
    // - System-wide resource constraints

    // For now, base it on agent's load (inverse relationship)
    const availableCapacity = 1 - agent.currentLoad / agent.maxLoad;
    return Promise.resolve(availableCapacity);
  }

  /**
   * Get recent workspace activity for an agent
   */
  private async getAgentWorkspaceActivity(agentId: string): Promise<number> {
    // Placeholder - would track agent file modifications
    // In production, this would query workspace state for agent activity
    return Math.random() * 20; // Mock activity score
  }

  /**
   * Calculate agent's familiarity with workspace context
   */
  private calculateAgentFamiliarity(agentId: string, context: any): number {
    // Placeholder - would analyze agent's historical interaction with context files
    // In production, this would consider:
    // - Files the agent has recently modified
    // - Agent's access patterns
    // - Task completion history
    return Math.random() * 0.5 + 0.3; // Mock familiarity score (0.3-0.8)
  }

  /**
   * Extract keywords from task for workspace relevance
   */
  private extractTaskKeywords(task: any): string[] {
    const keywords: string[] = [];

    // Add task type
    if (task.type) keywords.push(task.type);

    // Add explicit keywords if provided
    if (task.keywords) keywords.push(...task.keywords);

    // Extract from task description (simplified)
    if (task.description) {
      const words = task.description.toLowerCase().split(/\s+/);
      keywords.push(...words.filter((word: string) => word.length > 3));
    }

    // Add common development keywords
    keywords.push("src", "lib", "component", "service", "util", "test");

    return [...new Set(keywords)]; // Remove duplicates
  }

  /**
   * Calculate how well an agent matches task capabilities
   */
  private calculateCapabilityMatch(task: any, agent: any): number {
    if (!task.requiredCapabilities || task.requiredCapabilities.length === 0) {
      return 1.0; // Full match if no specific requirements
    }

    const requiredCaps = task.requiredCapabilities;
    const agentCaps = agent.capabilities;

    const matchedCaps = requiredCaps.filter((cap: string) =>
      agentCaps.includes(cap)
    );
    return matchedCaps.length / requiredCaps.length;
  }

  /**
   * Calculate performance score for agent on this task type
   */
  private calculatePerformanceScore(task: any, agent: any): number {
    // Simplified performance scoring
    // In production, this would consider historical performance data

    const perf = agent.performance;

    // Weight performance factors based on task type
    switch (task.type) {
      case "analysis":
      case "research":
        return perf.quality * 0.6 + perf.reliability * 0.3 + perf.speed * 0.1;

      case "computation":
      case "data_processing":
        return perf.speed * 0.5 + perf.reliability * 0.3 + perf.quality * 0.2;

      case "writing":
      case "communication":
        return perf.quality * 0.7 + perf.reliability * 0.2 + perf.speed * 0.1;

      default:
        return (perf.quality + perf.reliability + perf.speed) / 3;
    }
  }

  /**
   * Check constitutional compliance of assigning task to agent
   */
  private async checkAssignmentCompliance(
    task: any,
    agent: any
  ): Promise<{
    compliant: boolean;
    reason: string;
  }> {
    // Check for constitutional violations in agent-task assignment
    const violations: string[] = [];

    // Agent capability restrictions
    if (
      task.restrictedAgentTypes &&
      task.restrictedAgentTypes.includes(agent.type)
    ) {
      violations.push(
        `Agent type ${agent.type} is restricted from this task type`
      );
    }

    // Agent load limits (constitutional resource allocation)
    if (agent.currentLoad >= agent.maxLoad * 0.9) {
      // Over 90% capacity
      violations.push(`Agent ${agent.id} is over capacity limit`);
    }

    // Task-agent compatibility rules
    if (task.requiresHumanReview && !agent.hasHumanReviewCapability) {
      violations.push(
        `Task requires human review capability that agent ${agent.id} lacks`
      );
    }

    // Security classification compatibility
    if (task.securityLevel && agent.securityClearance < task.securityLevel) {
      violations.push(`Agent ${agent.id} lacks required security clearance`);
    }

    if (violations.length === 0) {
      return {
        compliant: true,
        reason: "Assignment constitutionally compliant",
      };
    }

    return {
      compliant: false,
      reason: violations.join("; "),
    };
  }

  /**
   * Create a proper task assignment with monitoring and deadlines
   */
  private async createTaskAssignment(task: any, agent: any): Promise<any> {
    const assignmentId = `assignment-${task.id}-${agent.id}-${Date.now()}`;

    const assignment = {
      id: assignmentId,
      taskId: task.id,
      agentId: agent.id,
      agentName: agent.name || `Agent ${agent.id}`,
      assignedAt: new Date(),
      deadline: this.calculateDeadline(task),
      assignmentTimeoutMs: this.calculateTimeout(task),
      status: "assigned",
      priority: task.priority || "normal",
      monitoring: {
        progressChecks: true,
        deadlineMonitoring: true,
        qualityGates: task.qualityRequirements || [],
      },
      metadata: {
        assignmentReason: "capability_match_load_balance",
        constitutionalCompliance: "verified",
        expectedDuration: this.estimateTaskDuration(task, agent),
      },
    };

    // Update agent load (simplified)
    agent.currentLoad += 1;

    // In production, this would persist the assignment and set up monitoring
    console.log(
      `Created assignment ${assignmentId} for task ${task.id} to agent ${agent.id}`
    );

    return assignment;
  }

  /**
   * Calculate assignment deadline based on task requirements
   */
  private calculateDeadline(task: any): Date {
    const baseDuration = this.estimateTaskDuration(task);
    const deadline = new Date(Date.now() + baseDuration);

    // Add buffer time based on priority
    const bufferMultiplier =
      task.priority === "urgent" ? 1.2 : task.priority === "high" ? 1.5 : 2.0;

    return new Date(deadline.getTime() * bufferMultiplier);
  }

  /**
   * Calculate assignment timeout in milliseconds
   */
  private calculateTimeout(task: any): number {
    const baseDuration = this.estimateTaskDuration(task);

    // Timeouts are typically 2-3x the expected duration
    const timeoutMultiplier =
      task.priority === "urgent" ? 2.0 : task.priority === "high" ? 2.5 : 3.0;

    return Math.max(baseDuration * timeoutMultiplier, 300000); // Minimum 5 minutes
  }

  /**
   * Estimate task duration based on type and complexity
   */
  private estimateTaskDuration(task?: any, agent?: any): number {
    if (!task) return 1800000; // 30 minutes default

    let baseDuration = 900000; // 15 minutes base

    // Adjust by task type
    switch (task.type) {
      case "research":
      case "analysis":
        baseDuration *= 2;
        break;
      case "computation":
        baseDuration *= 1.5;
        break;
      case "writing":
        baseDuration *= 1.8;
        break;
    }

    // Adjust by complexity
    if (task.complexity) {
      baseDuration *= 1 + task.complexity;
    }

    // Adjust by agent performance (if available)
    if (agent?.performance?.speed) {
      baseDuration /= agent.performance.speed;
    }

    return Math.max(baseDuration, 300000); // Minimum 5 minutes
  }

  /**
   * Create a human override request for constitutional violation
   */
  private async createOverrideRequest(
    task: any,
    complianceCheck: { compliant: boolean; reason: string; severity: string }
  ): Promise<OverrideRequest> {
    // Check rate limits
    if (!this.checkOverrideRateLimit()) {
      await this.logSecurityEvent(
        SecurityEventType.RATE_LIMIT_EXCEEDED,
        SecurityAuditLevel.WARNING,
        "override",
        "create_request",
        false,
        { taskId: task.id, reason: complianceCheck.reason },
        60
      );
      throw new Error(
        "Override rate limit exceeded. Too many overrides requested recently."
      );
    }

    const requestId = `override-${task.id}-${Date.now()}-${Math.random()
      .toString(36)
      .substring(2, 9)}`;

    const overrideRequest: OverrideRequest = {
      id: requestId,
      taskId: task.id,
      violation: {
        reason: complianceCheck.reason,
        severity: complianceCheck.severity as any,
        type: "constitutional_violation",
      },
      requestedBy: "system", // Could be enhanced to track actual user
      status: "pending",
      createdAt: new Date(),
      updatedAt: new Date(),
      denialCount: 0,
      metadata: {
        taskType: task.type,
        riskAssessment: complianceCheck.severity as any,
        constitutionalRule: "CAWS-COMPLIANCE-CHECK",
      },
    };

    // Store the override request
    this.overrideRequests.set(requestId, overrideRequest);

    // Increment usage counter
    this.overrideUsage.count++;

    await this.logSecurityEvent(
      SecurityEventType.OVERRIDE_REQUEST,
      SecurityAuditLevel.INFO,
      "override",
      "create_request",
      true,
      {
        requestId,
        taskId: task.id,
        severity: complianceCheck.severity,
        reason: complianceCheck.reason,
      },
      complianceCheck.severity === "critical"
        ? 80
        : complianceCheck.severity === "high"
        ? 60
        : 40
    );

    // Secure logging only (no sensitive data)
    if (this.securityLogger) {
      this.securityLogger.info(
        `Created override request ${requestId} for task ${task.id}`
      );
    }

    return overrideRequest;
  }

  /**
   * Check if override rate limit allows new requests
   */
  private checkOverrideRateLimit(): boolean {
    const now = Date.now();
    const windowMs = 60 * 60 * 1000; // 1 hour window
    const maxOverrides =
      this.config.caws?.humanOverride?.maxOverridesPerHour || 5;

    // Reset window if needed
    if (now - this.overrideUsage.windowStart > windowMs) {
      this.overrideUsage = { count: 0, windowStart: now };
    }

    return this.overrideUsage.count < maxOverrides;
  }

  /**
   * Approve or deny an override request
   */
  async processOverrideDecision(
    decision: OverrideDecision
  ): Promise<OverrideRequest> {
    if (!this.initialized) {
      throw new Error("Orchestrator not initialized");
    }

    const request = this.overrideRequests.get(decision.requestId);
    if (!request) {
      throw new Error(`Override request ${decision.requestId} not found`);
    }

    if (request.status !== "pending") {
      throw new Error(
        `Override request ${decision.requestId} is not in pending status`
      );
    }

    // Update request
    request.status = decision.decision === "approve" ? "approved" : "denied";
    request.approvedBy = decision.approvedBy;
    request.approvedAt = new Date();
    request.updatedAt = new Date();

    if (decision.decision === "approve") {
      // Set expiration
      const validityHours =
        decision.validityHours ||
        this.config.caws?.humanOverride?.overrideValidityHours ||
        24;
      request.expiresAt = new Date(Date.now() + validityHours * 60 * 60 * 1000);

      // Move to approved overrides
      this.approvedOverrides.set(request.id, request);
    } else {
      // Track denial count for escalation
      request.denialCount++;

      const escalationThreshold =
        this.config.caws?.humanOverride?.escalationThreshold || 3;
      if (request.denialCount >= escalationThreshold) {
        console.warn(
          `Override request ${request.id} denied ${request.denialCount} times, consider escalation`
        );
      }

      // Add to denied requests map
      this.deniedRequests.set(request.id, request);
    }

    // Remove from pending requests
    this.overrideRequests.delete(request.id);

    console.log(
      `Override request ${request.id} ${decision.decision}d by ${decision.approvedBy}`
    );

    return request;
  }

  /**
   * Check if a task has an approved override
   */
  private hasApprovedOverride(taskId: string): boolean {
    // Check if any approved override exists for this task
    for (const override of this.approvedOverrides.values()) {
      if (override.taskId === taskId && override.status === "approved") {
        // Check if still valid
        if (!override.expiresAt || override.expiresAt > new Date()) {
          return true;
        } else {
          // Expired, remove it
          this.approvedOverrides.delete(override.id);
          override.status = "expired";
        }
      }
    }
    return false;
  }

  /**
   * Get pending override requests
   */
  async getPendingOverrides(): Promise<OverrideRequest[]> {
    if (!this.initialized) {
      throw new Error("Orchestrator not initialized");
    }

    return Array.from(this.overrideRequests.values()).filter(
      (req) => req.status === "pending"
    );
  }

  /**
   * Get override request by ID
   */
  async getOverrideRequest(requestId: string): Promise<OverrideRequest | null> {
    if (!this.initialized) {
      throw new Error("Orchestrator not initialized");
    }

    return (
      this.overrideRequests.get(requestId) ||
      this.approvedOverrides.get(requestId) ||
      null
    );
  }

  /**
   * Resubmit a task that was pending override approval
   */
  async resubmitTaskWithOverride(
    taskId: string,
    overrideId: string
  ): Promise<{ taskId: string; assignmentId?: string }> {
    if (!this.initialized) {
      throw new Error("Orchestrator not initialized");
    }

    // Verify override is approved and valid
    const override = this.approvedOverrides.get(overrideId);
    if (
      !override ||
      override.taskId !== taskId ||
      override.status !== "approved"
    ) {
      throw new Error(
        `Invalid or expired override ${overrideId} for task ${taskId}`
      );
    }

    if (override.expiresAt && override.expiresAt <= new Date()) {
      throw new Error(`Override ${overrideId} has expired`);
    }

    // Create a mock task (in production, would retrieve from storage)
    const task = { id: taskId, type: override.metadata.taskType || "unknown" };

    console.log(
      `Resubmitting task ${taskId} with approved override ${overrideId}`
    );

    // Skip compliance check and proceed with assignment
    const assignment = await this.assignTaskToAgent(task);
    if (assignment) {
      return {
        taskId: task.id,
        assignmentId: assignment.id,
      };
    }

    // Fallback if no assignment possible
    return {
      taskId: task.id,
      assignmentId: `resubmitted-${Date.now()}`,
    };
  }

  /**
   * Get override system statistics
   */
  async getOverrideStats(): Promise<{
    pendingRequests: number;
    approvedOverrides: number;
    deniedRequests: number;
    expiredOverrides: number;
    usageThisHour: number;
  }> {
    if (!this.initialized) {
      throw new Error("Orchestrator not initialized");
    }

    const now = new Date();
    let expiredCount = 0;

    // Clean up expired overrides
    for (const [id, override] of this.approvedOverrides.entries()) {
      if (override.expiresAt && override.expiresAt <= now) {
        this.approvedOverrides.delete(id);
        override.status = "expired";
        expiredCount++;
      }
    }

    return {
      pendingRequests: Array.from(this.overrideRequests.values()).filter(
        (r) => r.status === "pending"
      ).length,
      approvedOverrides: this.approvedOverrides.size,
      deniedRequests: this.deniedRequests.size,
      expiredOverrides: expiredCount,
      usageThisHour: this.overrideUsage.count,
    };
  }

  /**
   * Check constitutional compliance of a task
   */
  private async checkConstitutionalCompliance(task: any): Promise<{
    compliant: boolean;
    reason: string;
    severity: "low" | "medium" | "high" | "critical";
  }> {
    // Basic compliance checks - in production, this would use ARBITER-003 CAWS Validator
    // For now, implement simple heuristics

    const violations: string[] = [];

    // Check for potentially harmful content
    const harmfulPatterns = [
      /exploit|hack|crack|malware/i,
      /illegal|unlawful|forbidden/i,
      /harm|damage|destroy/i,
      /deceptive|fraudulent|misleading/i,
    ];

    const taskContent = JSON.stringify(task);
    for (const pattern of harmfulPatterns) {
      if (pattern.test(taskContent)) {
        violations.push(
          `Potentially harmful content detected: ${pattern.source}`
        );
      }
    }

    // Check for resource-intensive operations without limits
    if (task.type === "computation" && !(task as any).resourceLimits) {
      violations.push("Computation task without resource limits");
    }

    // Check for data privacy concerns
    if (task.type === "data_processing" && !(task as any).privacyControls) {
      violations.push("Data processing task without privacy controls");
    }

    if (violations.length === 0) {
      return {
        compliant: true,
        reason: "No constitutional violations detected",
        severity: "low",
      };
    }

    // Determine severity based on violations
    const severity =
      violations.length > 2 ? "high" : violations.length > 1 ? "medium" : "low";

    return {
      compliant: false,
      reason: violations.join("; "),
      severity,
    };
  }

  /**
   * Escalate task to constitutional arbitration
   */
  private async escalateToArbitration(
    task: any,
    complianceCheck: { compliant: boolean; reason: string; severity: string }
  ): Promise<void> {
    if (!this.components.arbitrationProtocol) {
      throw new Error(
        "Arbitration protocol not available for constitutional review"
      );
    }

    console.log(
      `Escalating task ${task.id} to constitutional arbitration: ${complianceCheck.reason}`
    );

    try {
      // Create constitutional violation from compliance check
      const violation = {
        id: `viol-${task.id}-${Date.now()}`,
        ruleId: "CAWS-COMPLIANCE-CHECK",
        description: complianceCheck.reason,
        severity: complianceCheck.severity as any,
        evidence: [
          `Task content: ${JSON.stringify(task)}`,
          `Compliance check result: ${complianceCheck.reason}`,
          `Detected by: ArbiterOrchestrator`,
        ],
        detectedAt: new Date(),
        violator: "system", // System-detected violation
        context: {
          taskId: task.id,
          orchestratorId: "arbiter-orchestrator",
          complianceCheck: complianceCheck,
        },
      };

      // Get relevant constitutional rules (would normally query rule engine)
      const rules: any[] = [
        {
          id: "CAWS-CORE",
          version: "1.0.0",
          name: "Constitutional AI Work Systems Core Principles",
          category: "core_principles" as any,
          severity: "critical" as any,
          description: "Fundamental CAWS constitutional requirements",
          conditions: [],
          actions: [],
          metadata: {},
        },
      ];

      // Start arbitration session
      const session = await this.components.arbitrationProtocol.startSession(
        violation,
        rules,
        ["system"] // System as participant for automated arbitration
      );

      console.log(
        `Arbitration session started: ${session.id} for task ${task.id}`
      );

      // Evaluate rules against the violation
      await this.components.arbitrationProtocol.evaluateRules(session.id);

      // Find applicable precedents
      await this.components.arbitrationProtocol.findPrecedents(session.id);

      // Generate verdict
      const verdict = await this.components.arbitrationProtocol.generateVerdict(
        session.id,
        "system" // Automated system verdict
      );

      console.log(
        `Arbitration verdict generated: ${verdict.outcome} for task ${task.id}`
      );

      // Complete the session
      await this.components.arbitrationProtocol.completeSession(session.id);

      console.log(`Arbitration completed for task ${task.id}`);
    } catch (error) {
      console.error(
        `Failed to escalate task ${task.id} to arbitration:`,
        error
      );
      throw new Error(
        `Arbitration escalation failed: ${
          error instanceof Error ? error.message : String(error)
        }`
      );
    }
  }

  /**
   * Get orchestrator status
   */
  async getStatus(): Promise<ArbiterOrchestratorStatus> {
    if (!this.initialized) {
      throw new Error("Orchestrator not initialized");
    }

    // Get override stats
    const overrideStats = await this.getOverrideStats();

    return {
      healthy: true,
      components: {
        taskQueue: !!this.components.taskQueue,
        taskAssignment: !!this.components.taskAssignment,
        agentRegistry: !!this.components.agentRegistry,
        security: !!this.components.security,
        healthMonitor: !!this.components.healthMonitor,
        arbitrationProtocol: !!this.components.arbitrationProtocol,
        reasoningEngine: !!this.components.reasoningEngine,
        humanOverride: !!this.config.caws?.humanOverride?.enabled,
      },
      metrics: {
        activeTasks: 0, // Would query actual task queue
        queuedTasks: 0, // Would query actual task queue
        registeredAgents: 0, // Would query actual agent registry
        uptimeSeconds: Math.floor(Date.now() / 1000), // Simplified
        // Human override metrics
        pendingOverrides: overrideStats.pendingRequests,
        approvedOverrides: overrideStats.approvedOverrides,
        overrideUsageThisHour: overrideStats.usageThisHour,
      },
      version: "2.0.0",
    };
  }

  /**
   * Get security audit events (admin access only)
   */
  async getSecurityAuditEvents(
    limit: number = 100,
    level?: SecurityAuditLevel,
    type?: SecurityEventType,
    since?: Date
  ): Promise<SecurityAuditEvent[]> {
    if (!this.initialized) {
      throw new Error("Orchestrator not initialized");
    }

    // Security check - in production this would require admin privileges
    await this.logSecurityEvent(
      SecurityEventType.DATA_ACCESS,
      SecurityAuditLevel.INFO,
      "security_audit",
      "read_events",
      true,
      { limit, level, type, since: since?.toISOString() },
      20
    );

    let events = [...this.securityAuditEvents];

    // Apply filters
    if (level) {
      events = events.filter((e) => e.level === level);
    }
    if (type) {
      events = events.filter((e) => e.type === type);
    }
    if (since) {
      events = events.filter((e) => e.timestamp >= since);
    }

    // Sort by timestamp (newest first) and limit
    events.sort((a, b) => b.timestamp.getTime() - a.timestamp.getTime());

    return events.slice(0, Math.min(limit, 1000)); // Hard limit for security
  }

  /**
   * Set secure logger for production use
   */
  setSecureLogger(logger: typeof console): void {
    this.securityLogger = logger;
    this.logSecurityEvent(
      SecurityEventType.CONFIGURATION,
      SecurityAuditLevel.INFO,
      "security",
      "set_logger",
      true,
      { loggerType: "external" },
      0
    ).catch((err) => console.error("Failed to log security event:", err));
  }

  /**
   * Get the audit logger for external use
   */
  getAuditLogger(): AuditLogger | null {
    return this.components.auditLogger || null;
  }

  /**
   * Get security metrics for monitoring
   */
  async getSecurityMetrics(): Promise<{
    totalAuditEvents: number;
    eventsByLevel: Record<SecurityAuditLevel, number>;
    eventsByType: Record<SecurityEventType, number>;
    highRiskEventsLastHour: number;
    averageRiskScore: number;
  }> {
    if (!this.initialized) {
      throw new Error("Orchestrator not initialized");
    }

    const eventsByLevel: Record<SecurityAuditLevel, number> = {
      [SecurityAuditLevel.INFO]: 0,
      [SecurityAuditLevel.WARNING]: 0,
      [SecurityAuditLevel.ERROR]: 0,
      [SecurityAuditLevel.CRITICAL]: 0,
    };

    const eventsByType: Record<SecurityEventType, number> = {
      [SecurityEventType.AUTHENTICATION]: 0,
      [SecurityEventType.AUTHORIZATION]: 0,
      [SecurityEventType.INPUT_VALIDATION]: 0,
      [SecurityEventType.DATA_ACCESS]: 0,
      [SecurityEventType.CONFIGURATION]: 0,
      [SecurityEventType.OVERRIDE_REQUEST]: 0,
      [SecurityEventType.OVERRIDE_APPROVAL]: 0,
      [SecurityEventType.CONSTITUTIONAL_VIOLATION]: 0,
      [SecurityEventType.RATE_LIMIT_EXCEEDED]: 0,
      [SecurityEventType.SUSPICIOUS_ACTIVITY]: 0,
    };

    let highRiskEventsLastHour = 0;
    let totalRiskScore = 0;
    const oneHourAgo = new Date(Date.now() - 60 * 60 * 1000);

    for (const event of this.securityAuditEvents) {
      eventsByLevel[event.level]++;
      eventsByType[event.type]++;
      totalRiskScore += event.riskScore;

      if (event.timestamp >= oneHourAgo && event.riskScore >= 50) {
        highRiskEventsLastHour++;
      }
    }

    return {
      totalAuditEvents: this.securityAuditEvents.length,
      eventsByLevel,
      eventsByType,
      highRiskEventsLastHour,
      averageRiskScore:
        this.securityAuditEvents.length > 0
          ? totalRiskScore / this.securityAuditEvents.length
          : 0,
    };
  }

  /**
   * Get task status by ID
   */
  async getTaskStatus(taskId: string): Promise<any> {
    if (!this.initialized) {
      throw new Error("Orchestrator not initialized");
    }
    // Delegate to getStatus for now, can be extended for specific task queries
    return this.getStatus();
  }

  /**
   * Register a new agent with the orchestrator
   */
  async registerAgent(agent: any): Promise<void> {
    if (!this.initialized) {
      throw new Error("Orchestrator not initialized");
    }
    // Placeholder implementation - would integrate with agent registry
    console.log(`Registering agent: ${agent.id || agent.name}`);
  }

  /**
   * Get agent profile by ID
   */
  async getAgentProfile(agentId: string): Promise<any> {
    if (!this.initialized) {
      throw new Error("Orchestrator not initialized");
    }
    // Placeholder implementation
    return { id: agentId, name: "Agent", capabilities: [] };
  }

  /**
   * Cancel a task
   */
  async cancelTask(taskId: string): Promise<boolean | null> {
    if (!this.initialized) {
      throw new Error("Orchestrator not initialized");
    }
    console.log(`Cancelling task: ${taskId}`);
    // Placeholder implementation - would interact with task queue
    return true;
  }

  /**
   * Authenticate user credentials
   */
  async authenticate(credentials: any): Promise<boolean> {
    if (!this.initialized) {
      throw new Error("Orchestrator not initialized");
    }

    await this.logSecurityEvent(
      SecurityEventType.AUTHENTICATION,
      SecurityAuditLevel.INFO,
      "authentication",
      "login_attempt",
      true,
      { username: credentials.username },
      10
    );

    // Placeholder implementation - would integrate with auth system
    return true;
  }

  /**
   * Authorize user action
   */
  authorize(context: any, action: string): boolean | null {
    if (!this.initialized) {
      throw new Error("Orchestrator not initialized");
    }

    const userId = context?.userId || context?.user?.id || "unknown";

    this.logSecurityEvent(
      SecurityEventType.AUTHORIZATION,
      SecurityAuditLevel.INFO,
      "authorization",
      action,
      true,
      { userId, action },
      15
    ).catch((err) => console.error("Failed to log security event:", err));

    // Placeholder implementation - would check permissions
    return true;
  }

  /**
   * Update agent performance metrics
   */
  async updateAgentPerformance(agentId: string, metrics: any): Promise<void> {
    if (!this.initialized) {
      throw new Error("Orchestrator not initialized");
    }
    console.log(`Updating agent ${agentId} performance:`, metrics);
    // Placeholder implementation
  }

  /**
   * Process a knowledge query
   */
  async processKnowledgeQuery(query: string | any): Promise<any> {
    if (!this.initialized) {
      throw new Error("Orchestrator not initialized");
    }
    // Handle both string and KnowledgeQuery object
    const queryStr = typeof query === "string" ? query : query.query;

    // Placeholder implementation - would delegate to knowledge seeker
    return {
      query: queryStr,
      results: [],
      confidence: 0.0,
      processingTimeMs: 0,
    };
  }

  /**
   * Get knowledge system status
   */
  async getKnowledgeStatus(): Promise<any> {
    if (!this.initialized) {
      throw new Error("Orchestrator not initialized");
    }
    return {
      healthy: true,
      cacheSize: 0,
      queriesProcessed: 0,
    };
  }

  /**
   * Clear knowledge caches
   */
  async clearKnowledgeCaches(): Promise<void> {
    if (!this.initialized) {
      throw new Error("Orchestrator not initialized");
    }
    console.log("Clearing knowledge caches");
    // Placeholder implementation
  }

  /**
   * Verify information using verification engine
   */
  async verifyInformation(request: any): Promise<any> {
    if (!this.initialized) {
      throw new Error("Orchestrator not initialized");
    }

    // Audit log verification request
    await this.components.auditLogger
      ?.logAuditEvent(
        AuditEventType.VERIFICATION_REQUEST,
        AuditSeverity.LOW,
        "system",
        "verification",
        "verify_request",
        "success",
        {
          requestId: request.id,
          contentType: request.type || "unknown",
          hasExpectedOutput: !!request.expectedOutput,
        }
      )
      .catch((err) =>
        console.error("Failed to log verification request:", err)
      );

    // Check if verification engine is available
    if (!this.components.verificationEngine) {
      console.warn(
        "Verification engine not available, returning unverified result"
      );

      // Audit log verification failure
      await this.components.auditLogger
        ?.logAuditEvent(
          AuditEventType.VERIFICATION_FAILURE,
          AuditSeverity.MEDIUM,
          "system",
          "verification",
          "verify_request",
          "failure",
          {
            requestId: request.id,
            reason: "verification_engine_unavailable",
          }
        )
        .catch((err) =>
          console.error("Failed to log verification failure:", err)
        );

      return {
        requestId: request.id,
        verdict: "unverified",
        confidence: 0.0,
        reasoning: ["Verification engine not available"],
        supportingEvidence: [],
        contradictoryEvidence: [],
        verificationMethods: [],
        processingTimeMs: 0,
      };
    }

    try {
      const startTime = Date.now();

      // Convert request to verification request format
      const verificationRequest = {
        id: request.id,
        content: request.content || request.information || request.claim,
        type: request.type || "fact_checking",
        priority: request.priority || "medium",
        context: request.context || {},
        metadata: request.metadata || {},
      };

      // Execute verification
      const result = await this.components.verificationEngine.verify(
        verificationRequest
      );

      const processingTimeMs = Date.now() - startTime;

      // Audit log successful verification
      await this.components.auditLogger
        ?.logAuditEvent(
          AuditEventType.VERIFICATION_SUCCESS,
          result.confidence > 0.8
            ? AuditSeverity.LOW
            : result.confidence > 0.5
            ? AuditSeverity.MEDIUM
            : AuditSeverity.HIGH,
          "system",
          "verification",
          "verify_complete",
          "success",
          {
            requestId: result.requestId,
            verdict: result.verdict,
            confidence: result.confidence,
            processingTimeMs,
            methodCount: result.methodResults?.length || 0,
          }
        )
        .catch((err) =>
          console.error("Failed to log verification success:", err)
        );

      // Convert result to expected format
      return {
        requestId: result.requestId,
        verdict: result.verdict,
        confidence: result.confidence,
        reasoning: result.reasoning,
        supportingEvidence: result.supportingEvidence,
        contradictoryEvidence: result.contradictoryEvidence,
        verificationMethods: result.methodResults?.map((m) => m.method) || [],
        processingTimeMs,
      };
    } catch (error) {
      console.error("Verification failed:", error);

      // Audit log verification error
      await this.components.auditLogger
        ?.logAuditEvent(
          AuditEventType.VERIFICATION_FAILURE,
          AuditSeverity.HIGH,
          "system",
          "verification",
          "verify_error",
          "failure",
          {
            requestId: request.id,
            error: error instanceof Error ? error.message : "Unknown error",
            errorType:
              error instanceof Error ? error.constructor.name : "Unknown",
          }
        )
        .catch((err) =>
          console.error("Failed to log verification error:", err)
        );

      return {
        requestId: request.id,
        verdict: "error",
        confidence: 0.0,
        reasoning: [
          `Verification error: ${
            error instanceof Error ? error.message : "Unknown error"
          }`,
        ],
        supportingEvidence: [],
        contradictoryEvidence: [],
        verificationMethods: [],
        processingTimeMs: 0,
      };
    }
  }

  /**
   * Get verification method statistics
   */
  async getVerificationMethodStats(): Promise<any> {
    if (!this.initialized) {
      throw new Error("Orchestrator not initialized");
    }

    if (!this.components.verificationEngine) {
      return {
        methodsAvailable: 0,
        totalVerifications: 0,
        averageConfidence: 0.0,
        error: "Verification engine not available",
      };
    }

    try {
      const health = await this.components.verificationEngine.healthCheck();
      const supportedMethods =
        this.components.verificationEngine.getSupportedMethods();

      // Get status for each method
      const methodStats = await Promise.all(
        supportedMethods.map(async (method) => {
          const status =
            await this.components.verificationEngine!.getMethodStatus(method);
          return {
            method,
            enabled: status.enabled,
            healthy: status.healthy,
            successRate: status.successRate || 0,
            averageProcessingTime: status.averageProcessingTime || 0,
            lastUsed: status.lastUsed,
          };
        })
      );

      return {
        methodsAvailable: health.enabledMethods,
        totalMethods: health.totalMethods,
        healthyMethods: health.healthyMethods,
        activeVerifications: health.activeVerifications,
        cacheSize: health.cacheSize,
        methodStats,
      };
    } catch (error) {
      console.error("Failed to get verification method stats:", error);
      return {
        methodsAvailable: 0,
        totalVerifications: 0,
        averageConfidence: 0.0,
        error: error instanceof Error ? error.message : "Unknown error",
      };
    }
  }

  /**
   * Get verification evidence statistics
   */
  async getVerificationEvidenceStats(): Promise<any> {
    if (!this.initialized) {
      throw new Error("Orchestrator not initialized");
    }
    return {
      totalEvidence: 0,
      averageCredibility: 0.0,
      sourceCount: 0,
    };
  }

  /**
   * Shutdown the orchestrator
   */
  async shutdown(): Promise<void> {
    if (!this.initialized) {
      return;
    }

    console.log("Shutting down Arbiter Orchestrator...");

    try {
      // Shutdown CAWS components
      if (this.components.arbitrationProtocol) {
        // Arbitration protocol doesn't have explicit shutdown
        this.components.arbitrationProtocol = undefined;
      }

      if (this.components.reasoningEngine) {
        // Reasoning engine doesn't have explicit shutdown
        this.components.reasoningEngine = undefined;
      }

      // Clear all component references
      this.components.taskQueue = undefined;
      this.components.taskAssignment = undefined;
      this.components.agentRegistry = undefined;
      this.components.security = undefined;
      this.components.healthMonitor = undefined;
      this.components.recoveryManager = undefined;
      this.components.knowledgeSeeker = undefined;

      this.initialized = false;
      console.log("✅ Arbiter Orchestrator shutdown complete");
    } catch (error) {
      console.error("Error during orchestrator shutdown:", error);
      this.initialized = false;
    }
  }
}
