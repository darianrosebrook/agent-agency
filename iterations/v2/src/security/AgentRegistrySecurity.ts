/**
 * Agent Registry Security Layer
 *
 * Implements authentication, authorization, input validation, and audit logging
 * for the Agent Registry Manager (ARBITER-001).
 *
 * @author @darianrosebrook
 */

import { AgentProfile } from "../types/agent-registry.js";
import { Logger } from "../utils/Logger.js";

export interface SecurityContext {
  tenantId: string;
  userId: string;
  roles: string[];
  permissions: string[];
  sessionId: string;
  ipAddress?: string;
  userAgent?: string;
}

export interface ValidationResult {
  valid: boolean;
  errors: string[];
  sanitized?: any;
}

export interface AuditEvent {
  id: string;
  timestamp: Date;
  eventType: AuditEventType;
  actor: {
    tenantId: string;
    userId: string;
    sessionId: string;
  };
  resource: {
    type: "agent";
    id: string;
  };
  action: AuditAction;
  details: Record<string, any>;
  result: "success" | "failure";
  errorMessage?: string;
  ipAddress?: string;
  userAgent?: string;
}

export enum AuditEventType {
  AGENT_REGISTRATION = "agent_registration",
  AGENT_UPDATE = "agent_update",
  AGENT_DELETION = "agent_deletion",
  AGENT_QUERY = "agent_query",
  PERFORMANCE_UPDATE = "performance_update",
  SECURITY_VIOLATION = "security_violation",
  AUTHENTICATION_FAILURE = "authentication_failure",
  AUTHORIZATION_FAILURE = "authorization_failure",
}

export enum AuditAction {
  CREATE = "create",
  READ = "read",
  UPDATE = "update",
  DELETE = "delete",
  QUERY = "query",
}

export interface SecurityConfig {
  enableAuditLogging: boolean;
  enableInputValidation: boolean;
  enableAuthorization: boolean;
  maxAuditEvents: number;
  auditRetentionDays: number;
  allowedTenantIds?: string[];
  blockedUserIds?: string[];
  rateLimitWindowMs: number;
  rateLimitMaxRequests: number;
}

/**
 * Default security configuration
 */
const DEFAULT_SECURITY_CONFIG: SecurityConfig = {
  enableAuditLogging: true,
  enableInputValidation: true,
  enableAuthorization: true,
  maxAuditEvents: 10000,
  auditRetentionDays: 90,
  rateLimitWindowMs: 60000, // 1 minute
  rateLimitMaxRequests: 100,
};

/**
 * Agent Registry Security Manager
 *
 * Provides comprehensive security controls including:
 * - Input validation and sanitization
 * - Authentication and authorization
 * - Multi-tenant isolation
 * - Audit logging and monitoring
 * - Rate limiting and abuse prevention
 */
export class AgentRegistrySecurity {
  private logger: Logger;
  private config: SecurityConfig;
  private auditEvents: AuditEvent[] = [];
  private rateLimitCache = new Map<
    string,
    { count: number; resetTime: number }
  >();

  constructor(config: Partial<SecurityConfig> = {}) {
    this.config = { ...DEFAULT_SECURITY_CONFIG, ...config };
    this.logger = new Logger("AgentRegistrySecurity");
  }

  /**
   * Authenticate security context
   */
  async authenticate(token: string): Promise<SecurityContext | null> {
    try {
      // TODO: Implement JWT or API key validation
      // For now, accept any token and create basic context
      if (!token || token.length < 10) {
        await this.logAuditEvent({
          id: this.generateId(),
          timestamp: new Date(),
          eventType: AuditEventType.AUTHENTICATION_FAILURE,
          actor: {
            tenantId: "unknown",
            userId: "unknown",
            sessionId: "unknown",
          },
          resource: { type: "agent", id: "unknown" },
          action: AuditAction.READ,
          details: { reason: "Invalid token format" },
          result: "failure",
          errorMessage: "Invalid token format",
        });
        return null;
      }

      // Mock authentication - in production, validate JWT/API key
      const context: SecurityContext = {
        tenantId: this.extractTenantFromToken(token) || "default-tenant",
        userId: this.extractUserFromToken(token) || "anonymous",
        roles: ["agent-registry-user"],
        permissions: ["agent:read", "agent:create", "agent:update"],
        sessionId: this.generateId(),
        ipAddress: "unknown",
        userAgent: "unknown",
      };

      return context;
    } catch (error) {
      this.logger.error("Authentication failed:", error);
      return null;
    }
  }

  /**
   * Authorize action on resource
   */
  async authorize(
    context: SecurityContext,
    action: AuditAction,
    resourceType: "agent",
    resourceId: string,
    resource?: Partial<AgentProfile>
  ): Promise<boolean> {
    try {
      // Check if tenant is blocked
      if (this.config.blockedUserIds?.includes(context.userId)) {
        await this.logSecurityViolation(
          context,
          action,
          resourceType,
          resourceId,
          "User is blocked"
        );
        return false;
      }

      // Check tenant isolation - users can only access their own tenant's resources
      if (resource && this.isCrossTenantAccess(context, resource)) {
        await this.logSecurityViolation(
          context,
          action,
          resourceType,
          resourceId,
          "Cross-tenant access attempt"
        );
        return false;
      }

      // Check rate limiting
      if (!this.checkRateLimit(context)) {
        await this.logSecurityViolation(
          context,
          action,
          resourceType,
          resourceId,
          "Rate limit exceeded"
        );
        return false;
      }

      // Check permissions based on action
      const requiredPermission = this.getRequiredPermission(
        action,
        resourceType
      );
      if (!context.permissions.includes(requiredPermission)) {
        await this.logAuthorizationFailure(
          context,
          action,
          resourceType,
          resourceId,
          requiredPermission
        );
        return false;
      }

      return true;
    } catch (error) {
      this.logger.error("Authorization failed:", error);
      return false;
    }
  }

  /**
   * Validate and sanitize input data
   */
  validateAgentData(data: Partial<AgentProfile>): ValidationResult {
    const errors: string[] = [];
    const sanitized: Partial<AgentProfile> = { ...data };

    // Validate required fields
    if (!data.id || typeof data.id !== "string" || data.id.length === 0) {
      errors.push("Agent ID is required and must be a non-empty string");
    } else if (data.id.length > 255) {
      errors.push("Agent ID must be 255 characters or less");
    } else {
      // Sanitize ID - only allow alphanumeric, dash, underscore
      sanitized.id = data.id.replace(/[^a-zA-Z0-9_-]/g, "");
    }

    if (!data.name || typeof data.name !== "string" || data.name.length === 0) {
      errors.push("Agent name is required and must be a non-empty string");
    } else if (data.name.length > 255) {
      errors.push("Agent name must be 255 characters or less");
    }

    if (!data.modelFamily || typeof data.modelFamily !== "string") {
      errors.push("Model family is required and must be a string");
    } else {
      // Validate against allowed model families
      const allowedFamilies = [
        "gpt-4",
        "claude-3",
        "claude-3.5",
        "gemini-pro",
        "llama-3",
        "mixtral",
      ];
      if (!allowedFamilies.includes(data.modelFamily)) {
        errors.push(
          `Model family must be one of: ${allowedFamilies.join(", ")}`
        );
      }
    }

    // Validate capabilities structure
    if (data.capabilities) {
      if (typeof data.capabilities !== "object") {
        errors.push("Capabilities must be an object");
      } else {
        const caps = data.capabilities as any;
        if (!Array.isArray(caps.taskTypes)) {
          errors.push("Capabilities.taskTypes must be an array");
        }
        if (!Array.isArray(caps.languages)) {
          errors.push("Capabilities.languages must be an array");
        }
        if (!Array.isArray(caps.specializations)) {
          errors.push("Capabilities.specializations must be an array");
        }

        // Validate task types
        if (caps.taskTypes) {
          const validTaskTypes = [
            "code-editing",
            "research",
            "code-review",
            "documentation",
            "testing",
            "debugging",
            "refactoring",
            "api-design",
          ];
          for (const taskType of caps.taskTypes) {
            if (!validTaskTypes.includes(taskType)) {
              errors.push(`Invalid task type: ${taskType}`);
            }
          }
        }

        // Validate languages
        if (caps.languages) {
          const validLanguages = [
            "TypeScript",
            "JavaScript",
            "Python",
            "Rust",
            "Go",
            "Java",
            "C++",
            "C#",
          ];
          for (const language of caps.languages) {
            if (!validLanguages.includes(language)) {
              errors.push(`Invalid language: ${language}`);
            }
          }
        }
      }
    }

    return {
      valid: errors.length === 0,
      errors,
      sanitized: errors.length === 0 ? sanitized : undefined,
    };
  }

  /**
   * Validate performance metrics
   */
  validatePerformanceMetrics(metrics: any): ValidationResult {
    const errors: string[] = [];

    if (typeof metrics.success !== "boolean") {
      errors.push("Success must be a boolean");
    }

    if (
      typeof metrics.qualityScore !== "number" ||
      metrics.qualityScore < 0 ||
      metrics.qualityScore > 1
    ) {
      errors.push("Quality score must be a number between 0 and 1");
    }

    if (typeof metrics.latencyMs !== "number" || metrics.latencyMs < 0) {
      errors.push("Latency must be a non-negative number");
    }

    if (metrics.taskType && typeof metrics.taskType !== "string") {
      errors.push("Task type must be a string");
    }

    if (metrics.tokensUsed && typeof metrics.tokensUsed !== "number") {
      errors.push("Tokens used must be a number");
    }

    return {
      valid: errors.length === 0,
      errors,
    };
  }

  /**
   * Log audit event
   */
  async logAuditEvent(event: AuditEvent): Promise<void> {
    if (!this.config.enableAuditLogging) return;

    try {
      this.auditEvents.push(event);

      // Maintain audit event limit
      if (this.auditEvents.length > this.config.maxAuditEvents) {
        this.auditEvents = this.auditEvents.slice(-this.config.maxAuditEvents);
      }

      // Log security violations immediately
      if (event.eventType === AuditEventType.SECURITY_VIOLATION) {
        this.logger.warn("Security violation detected:", {
          actor: event.actor,
          action: event.action,
          resource: event.resource,
          details: event.details,
          ipAddress: event.ipAddress,
        });
      }

      this.logger.debug("Audit event logged", {
        eventType: event.eventType,
        result: event.result,
      });
    } catch (error) {
      this.logger.error("Failed to log audit event:", error);
    }
  }

  /**
   * Get audit events for a resource
   */
  getAuditEvents(resourceId: string, limit: number = 50): AuditEvent[] {
    return this.auditEvents
      .filter((event) => event.resource.id === resourceId)
      .sort((a, b) => b.timestamp.getTime() - a.timestamp.getTime())
      .slice(0, limit);
  }

  /**
   * Check if access is cross-tenant
   */
  private isCrossTenantAccess(
    _context: SecurityContext,
    _resource: Partial<AgentProfile>
  ): boolean {
    // TODO: Implement tenant extraction from resource
    // For now, assume all resources are in the same tenant as the context
    return false;
  }

  /**
   * Check rate limiting
   */
  private checkRateLimit(context: SecurityContext): boolean {
    const key = `${context.tenantId}:${context.userId}`;
    const now = Date.now();
    const windowData = this.rateLimitCache.get(key);

    if (!windowData || now > windowData.resetTime) {
      // New window
      this.rateLimitCache.set(key, {
        count: 1,
        resetTime: now + this.config.rateLimitWindowMs,
      });
      return true;
    }

    if (windowData.count >= this.config.rateLimitMaxRequests) {
      return false;
    }

    windowData.count++;
    return true;
  }

  /**
   * Get required permission for action
   */
  private getRequiredPermission(
    action: AuditAction,
    resourceType: string
  ): string {
    const permissionMap: Record<AuditAction, string> = {
      [AuditAction.CREATE]: `${resourceType}:create`,
      [AuditAction.READ]: `${resourceType}:read`,
      [AuditAction.UPDATE]: `${resourceType}:update`,
      [AuditAction.DELETE]: `${resourceType}:delete`,
      [AuditAction.QUERY]: `${resourceType}:read`,
    };

    return permissionMap[action] || `${resourceType}:read`;
  }

  /**
   * Log security violation
   */
  private async logSecurityViolation(
    context: SecurityContext,
    action: AuditAction,
    resourceType: string,
    resourceId: string,
    reason: string
  ): Promise<void> {
    await this.logAuditEvent({
      id: this.generateId(),
      timestamp: new Date(),
      eventType: AuditEventType.SECURITY_VIOLATION,
      actor: {
        tenantId: context.tenantId,
        userId: context.userId,
        sessionId: context.sessionId,
      },
      resource: { type: resourceType as "agent", id: resourceId },
      action,
      details: { reason, securityViolation: true },
      result: "failure",
      errorMessage: reason,
      ipAddress: context.ipAddress,
      userAgent: context.userAgent,
    });
  }

  /**
   * Log authorization failure
   */
  private async logAuthorizationFailure(
    context: SecurityContext,
    action: AuditAction,
    resourceType: string,
    resourceId: string,
    requiredPermission: string
  ): Promise<void> {
    await this.logAuditEvent({
      id: this.generateId(),
      timestamp: new Date(),
      eventType: AuditEventType.AUTHORIZATION_FAILURE,
      actor: {
        tenantId: context.tenantId,
        userId: context.userId,
        sessionId: context.sessionId,
      },
      resource: { type: resourceType as "agent", id: resourceId },
      action,
      details: { requiredPermission, missingPermission: true },
      result: "failure",
      errorMessage: `Missing required permission: ${requiredPermission}`,
      ipAddress: context.ipAddress,
      userAgent: context.userAgent,
    });
  }

  /**
   * Extract tenant ID from token (mock implementation)
   */
  private extractTenantFromToken(token: string): string | null {
    // TODO: Decode JWT and extract tenant claim
    // Mock implementation
    if (token.includes("tenant-")) {
      const match = token.match(/tenant-(\w+)/);
      return match ? match[1] : null;
    }
    return "default-tenant";
  }

  /**
   * Extract user ID from token (mock implementation)
   */
  private extractUserFromToken(token: string): string | null {
    // TODO: Decode JWT and extract user claim
    // Mock implementation
    if (token.includes("user-")) {
      const match = token.match(/user-(\w+)/);
      return match ? match[1] : null;
    }
    return "anonymous";
  }

  /**
   * Generate unique ID for audit events
   */
  private generateId(): string {
    return `audit_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * Clean up old audit events
   */
  cleanupAuditEvents(): void {
    const cutoffDate = new Date();
    cutoffDate.setDate(cutoffDate.getDate() - this.config.auditRetentionDays);

    this.auditEvents = this.auditEvents.filter(
      (event) => event.timestamp >= cutoffDate
    );
  }

  /**
   * Get security statistics
   */
  getSecurityStats(): {
    totalAuditEvents: number;
    securityViolations: number;
    authFailures: number;
    authzFailures: number;
    rateLimitHits: number;
  } {
    const violations = this.auditEvents.filter(
      (e) => e.eventType === AuditEventType.SECURITY_VIOLATION
    ).length;
    const authFailures = this.auditEvents.filter(
      (e) => e.eventType === AuditEventType.AUTHENTICATION_FAILURE
    ).length;
    const authzFailures = this.auditEvents.filter(
      (e) => e.eventType === AuditEventType.AUTHORIZATION_FAILURE
    ).length;

    // Count rate limit hits from cache (simplified)
    let rateLimitHits = 0;
    for (const [, data] of this.rateLimitCache) {
      if (data.count >= this.config.rateLimitMaxRequests) {
        rateLimitHits++;
      }
    }

    return {
      totalAuditEvents: this.auditEvents.length,
      securityViolations: violations,
      authFailures,
      authzFailures,
      rateLimitHits,
    };
  }
}
