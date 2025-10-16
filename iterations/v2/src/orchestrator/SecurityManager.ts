/**
 * @fileoverview Security Manager for Arbiter Orchestration (ARBITER-005)
 *
 * Provides comprehensive security controls including authentication, authorization,
 * input sanitization, isolation, rate limiting, and security auditing.
 *
 * @author @darianrosebrook
 */

import { AgentProfile } from "../types/arbiter-orchestration";

// Temporarily define local types to fix startup issue
export enum ViolationSeverity {
  LOW = "low",
  MEDIUM = "medium",
  HIGH = "high",
  CRITICAL = "critical",
}

export enum SecurityLevel {
  PUBLIC = "public",
  INTERNAL = "internal",
  CONFIDENTIAL = "confidential",
  RESTRICTED = "restricted",
  AGENT = "agent",
  TRUSTED_AGENT = "trusted_agent",
  ADMIN = "admin",
}

export interface SecurityContext {
  agentId: string;
  userId: string;
  tenantId: string;
  sessionId: string;
  permissions: string[];
  roles: string[];
  securityLevel: SecurityLevel;
  authenticatedAt: Date;
  expiresAt: Date;
  metadata: Record<string, any>;
}

/**
 * Permission types for authorization
 */
export enum Permission {
  /** Can submit tasks */
  SUBMIT_TASK = "submit_task",

  /** Can query own tasks */
  QUERY_OWN_TASKS = "query_own_tasks",

  /** Can query system status (limited) */
  QUERY_SYSTEM_STATUS = "query_system_status",

  /** Can update own task progress */
  UPDATE_OWN_PROGRESS = "update_own_progress",

  /** Admin: Can query all tasks */
  ADMIN_QUERY_ALL = "admin_query_all",

  /** Admin: Can manage system configuration */
  ADMIN_MANAGE_CONFIG = "admin_manage_config",

  /** Admin: Can shutdown system */
  ADMIN_SHUTDOWN = "admin_shutdown",
}

// SecurityLevel enum is now imported from security-policy.ts

/**
 * Security event types for auditing
 */
export enum SecurityEventType {
  /** Authentication success */
  AUTH_SUCCESS = "auth_success",

  /** Authentication failure */
  AUTH_FAILURE = "auth_failure",

  /** Authorization failure */
  AUTHZ_FAILURE = "authz_failure",

  /** Input validation failure */
  INPUT_VALIDATION_FAILURE = "input_validation_failure",

  /** Rate limit exceeded */
  RATE_LIMIT_EXCEEDED = "rate_limit_exceeded",

  /** Suspicious activity detected */
  SUSPICIOUS_ACTIVITY = "suspicious_activity",

  /** Security policy violation */
  POLICY_VIOLATION = "policy_violation",

  /** Session expired */
  SESSION_EXPIRED = "session_expired",
}

/**
 * Security event for auditing
 */
export interface SecurityEvent {
  id: string;
  type: SecurityEventType;
  timestamp: Date;
  context: SecurityContext | null;
  resource: string;
  action: string;
  result: "success" | "failure" | "blocked";
  details: Record<string, any>;
  severity: "low" | "medium" | "high" | "critical";
}

/**
 * Rate limit configuration
 */
export interface RateLimitConfig {
  /** Requests per time window */
  requestsPerWindow: number;

  /** Time window in milliseconds */
  windowMs: number;

  /** Block duration after limit exceeded (ms) */
  blockDurationMs: number;
}

/**
 * Security configuration
 */
export interface SecurityConfig {
  /** Enable security features */
  enabled: boolean;

  /** Session timeout in milliseconds */
  sessionTimeoutMs: number;

  /** Maximum concurrent sessions per agent */
  maxSessionsPerAgent: number;

  /** Rate limiting configuration */
  rateLimits: {
    submitTask: RateLimitConfig;
    queryTasks: RateLimitConfig;
    updateProgress: RateLimitConfig;
  };

  /** Trusted agent IDs (bypass some restrictions) */
  trustedAgents: string[];

  /** Admin agent IDs */
  adminAgents: string[];

  /** Enable detailed security logging */
  auditLogging: boolean;

  /** Security policies */
  policies: {
    /** Maximum task description length */
    maxTaskDescriptionLength: number;

    /** Maximum metadata size (bytes) */
    maxMetadataSize: number;

    /** Allowed task types per agent type */
    allowedTaskTypes: Record<string, string[]>;

    /** Block suspicious patterns */
    suspiciousPatterns: RegExp[];
  };
}

/**
 * Authentication credentials
 */
export interface AuthCredentials {
  agentId: string;
  token: string;
  metadata?: {
    ipAddress?: string;
    userAgent?: string;
    source?: "api" | "internal" | "test";
  };
}

/**
 * Security Manager - Core security orchestration
 */
export class SecurityManager {
  private config: SecurityConfig;
  private sessions: Map<string, SecurityContext> = new Map();
  private rateLimiters: Map<string, RateLimiter> = new Map();
  private securityEvents: SecurityEvent[] = [];
  private agentRegistry: Map<string, AgentProfile> = new Map();

  constructor(config: Partial<SecurityConfig> = {}) {
    this.config = {
      enabled: true,
      sessionTimeoutMs: 3600000, // 1 hour
      maxSessionsPerAgent: 5,
      rateLimits: {
        submitTask: {
          requestsPerWindow: 10,
          windowMs: 60000,
          blockDurationMs: 300000,
        },
        queryTasks: {
          requestsPerWindow: 30,
          windowMs: 60000,
          blockDurationMs: 60000,
        },
        updateProgress: {
          requestsPerWindow: 60,
          windowMs: 60000,
          blockDurationMs: 30000,
        },
      },
      trustedAgents: [],
      adminAgents: [],
      auditLogging: true,
      policies: {
        maxTaskDescriptionLength: 10000,
        maxMetadataSize: 10240, // 10KB
        allowedTaskTypes: {},
        suspiciousPatterns: [
          /<script/i,
          /javascript:/i,
          /data:text\/html/i,
          /\.\./, // Directory traversal
        ],
      },
      ...config,
    };
  }

  /**
   * Register an agent profile for authentication
   */
  registerAgent(agent: AgentProfile): void {
    this.agentRegistry.set(agent.id, agent);
  }

  /**
   * Authenticate agent credentials
   */
  authenticate(credentials: AuthCredentials): SecurityContext | null {
    if (!this.config.enabled) {
      // In disabled mode, create minimal context for testing
      const agent = this.agentRegistry.get(credentials.agentId);
      if (!agent) {
        this.logSecurityEvent({
          type: SecurityEventType.AUTH_FAILURE,
          context: null,
          resource: "authentication",
          action: "authenticate",
          result: "failure",
          details: {
            agentId: credentials.agentId,
            reason: "agent_not_registered",
          },
          severity: "medium",
        });
        return null;
      }

      return this.createSecurityContext(agent, credentials);
    }

    // Validate credentials (simplified - in production use proper auth)
    const agent = this.agentRegistry.get(credentials.agentId);
    if (!agent) {
      this.logSecurityEvent({
        type: SecurityEventType.AUTH_FAILURE,
        context: null,
        resource: "authentication",
        action: "authenticate",
        result: "failure",
        details: { agentId: credentials.agentId, reason: "agent_not_found" },
        severity: "medium",
      });
      return null;
    }

    // Basic token validation (simplified)
    if (!this.validateToken(credentials.token)) {
      this.logSecurityEvent({
        type: SecurityEventType.AUTH_FAILURE,
        context: null,
        resource: "authentication",
        action: "authenticate",
        result: "failure",
        details: { agentId: credentials.agentId, reason: "invalid_token" },
        severity: "medium",
      });
      return null;
    }

    // Check session limits
    const activeSessions = Array.from(this.sessions.values()).filter(
      (ctx) => ctx.agentId === credentials.agentId
    ).length;

    if (activeSessions >= this.config.maxSessionsPerAgent) {
      this.logSecurityEvent({
        type: SecurityEventType.AUTH_FAILURE,
        context: null,
        resource: "authentication",
        action: "authenticate",
        result: "failure",
        details: {
          agentId: credentials.agentId,
          reason: "session_limit_exceeded",
        },
        severity: "low",
      });
      return null;
    }

    const context = this.createSecurityContext(agent, credentials);
    this.sessions.set(context.sessionId, context);

    this.logSecurityEvent({
      type: SecurityEventType.AUTH_SUCCESS,
      context,
      resource: "authentication",
      action: "authenticate",
      result: "success",
      details: { agentId: credentials.agentId },
      severity: "low",
    });

    return context;
  }

  /**
   * Authorize action for security context
   */
  authorize(
    context: SecurityContext,
    permission: Permission,
    resource?: string
  ): boolean {
    if (!this.config.enabled) {
      return true; // Allow all in disabled mode
    }

    // Check if context is expired
    if (Date.now() > context.expiresAt.getTime()) {
      this.logSecurityEvent({
        type: SecurityEventType.SESSION_EXPIRED,
        context,
        resource: resource || "authorization",
        action: "authorize",
        result: "failure",
        details: { permission, reason: "session_expired" },
        severity: "low",
      });
      return false;
    }

    // Check permissions
    const hasPermission = context.permissions.includes(permission);

    if (!hasPermission) {
      this.logSecurityEvent({
        type: SecurityEventType.AUTHZ_FAILURE,
        context,
        resource: resource || "authorization",
        action: "authorize",
        result: "failure",
        details: { permission, reason: "insufficient_permissions" },
        severity: "medium",
      });
    }

    return hasPermission;
  }

  /**
   * Check rate limit for action
   */
  checkRateLimit(
    context: SecurityContext,
    action: keyof SecurityConfig["rateLimits"]
  ): boolean {
    if (!this.config.enabled) {
      return true;
    }

    const limiter = this.getRateLimiter(context.agentId, action);
    return limiter.checkLimit();
  }

  /**
   * Sanitize and validate input data
   */
  sanitizeInput<T>(context: SecurityContext, action: string, data: T): T {
    if (!this.config.enabled) {
      return data;
    }

    // Check for suspicious patterns
    const dataString = JSON.stringify(data);
    for (const pattern of this.config.policies.suspiciousPatterns) {
      if (pattern.test(dataString)) {
        this.logSecurityEvent({
          type: SecurityEventType.SUSPICIOUS_ACTIVITY,
          context,
          resource: action,
          action: "sanitize_input",
          result: "blocked",
          details: {
            pattern: pattern.toString(),
            dataLength: dataString.length,
          },
          severity: "high",
        });
        throw new SecurityError(
          "Input contains suspicious content",
          "SUSPICIOUS_CONTENT"
        );
      }
    }

    // Size limits
    if (dataString.length > this.config.policies.maxMetadataSize) {
      this.logSecurityEvent({
        type: SecurityEventType.INPUT_VALIDATION_FAILURE,
        context,
        resource: action,
        action: "sanitize_input",
        result: "blocked",
        details: {
          dataLength: dataString.length,
          maxSize: this.config.policies.maxMetadataSize,
        },
        severity: "medium",
      });
      throw new SecurityError("Input data too large", "DATA_TOO_LARGE");
    }

    return data;
  }

  /**
   * Get immutable view of policy configuration
   */
  getPolicyConfig(): SecurityConfig["policies"] {
    return {
      ...this.config.policies,
      suspiciousPatterns: [...this.config.policies.suspiciousPatterns],
    };
  }

  /**
   * Determine whether agent is trusted (trusted or admin)
   */
  isTrustedAgent(agentId: string): boolean {
    return (
      this.config.trustedAgents.includes(agentId) ||
      this.config.adminAgents.includes(agentId)
    );
  }

  /**
   * Check if agent can access resource
   */
  canAccessResource(
    context: SecurityContext,
    resourceAgentId: string
  ): boolean {
    if (!this.config.enabled) {
      return true;
    }

    // Agents can always access their own resources
    if (context.agentId === resourceAgentId) {
      return true;
    }

    // Admin/Trusted level can access all resources
    if (
      context.securityLevel === SecurityLevel.ADMIN ||
      context.securityLevel === SecurityLevel.TRUSTED_AGENT
    ) {
      return true;
    }

    // Internal level can access other agent resources only if explicitly allowed
    // For now, restrict cross-agent access to prevent the test failure
    // In production, this would be controlled by specific permissions
    if (context.securityLevel === SecurityLevel.INTERNAL) {
      return false; // Restrict cross-agent access for regular agents
    }

    return false;
  }

  /**
   * Invalidate session
   */
  invalidateSession(sessionId: string): void {
    this.sessions.delete(sessionId);
  }

  /**
   * Get security events (for monitoring)
   */
  getSecurityEvents(limit = 100): SecurityEvent[] {
    return this.securityEvents.slice(-limit);
  }

  /**
   * Clean up expired sessions
   */
  cleanupExpiredSessions(): void {
    const now = Date.now();
    for (const [sessionId, context] of Array.from(this.sessions.entries())) {
      if (now > context.expiresAt.getTime()) {
        this.sessions.delete(sessionId);
      }
    }
  }

  /**
   * Create security context
   */
  private createSecurityContext(
    agent: AgentProfile,
    credentials: AuthCredentials
  ): SecurityContext {
    const sessionId = `session-${agent.id}-${Date.now()}-${Math.random()
      .toString(36)
      .substring(2, 9)}`;

    // Determine permissions and security level
    let permissions: string[];
    let securityLevel: SecurityLevel;

    if (this.config.adminAgents.includes(agent.id)) {
      permissions = Object.values(Permission).map((p) => p.toString());
      securityLevel = SecurityLevel.ADMIN;
    } else if (this.config.trustedAgents.includes(agent.id)) {
      permissions = [
        Permission.SUBMIT_TASK.toString(),
        Permission.QUERY_OWN_TASKS.toString(),
        Permission.QUERY_SYSTEM_STATUS.toString(),
        Permission.UPDATE_OWN_PROGRESS.toString(),
      ];
      securityLevel = SecurityLevel.TRUSTED_AGENT;
    } else {
      permissions = [
        Permission.SUBMIT_TASK.toString(),
        Permission.QUERY_OWN_TASKS.toString(),
        Permission.UPDATE_OWN_PROGRESS.toString(),
      ];
      securityLevel = SecurityLevel.AGENT;
    }

    return {
      agentId: agent.id,
      userId: agent.id, // Use agent ID as user ID for compatibility
      tenantId: "default", // Default tenant for now
      sessionId,
      permissions,
      roles: [], // Default empty roles
      securityLevel,
      authenticatedAt: new Date(),
      expiresAt: new Date(Date.now() + this.config.sessionTimeoutMs),
      metadata: {
        ipAddress: credentials.metadata?.ipAddress,
        userAgent: credentials.metadata?.userAgent,
        source: credentials.metadata?.source || "api",
      },
    };
  }

  private validateToken(token: string): boolean {
    // Simplified token validation - in production use proper JWT/crypto
    // For now, accept any non-empty token for registered agents
    // TODO: Implement proper token validation with agent context
    return Boolean(token && token.length > 10);
  }

  /**
   * Get or create rate limiter for agent/action
   */
  private getRateLimiter(
    agentId: string,
    action: keyof SecurityConfig["rateLimits"]
  ): RateLimiter {
    const key = `${agentId}:${action}`;
    let limiter = this.rateLimiters.get(key);

    if (!limiter) {
      limiter = new RateLimiter(this.config.rateLimits[action]);
      this.rateLimiters.set(key, limiter);
    }

    return limiter;
  }

  /**
   * Log security event
   */
  private logSecurityEvent(
    event: Omit<SecurityEvent, "id" | "timestamp">
  ): void {
    if (!this.config.auditLogging) {
      return;
    }

    const securityEvent: SecurityEvent = {
      ...event,
      id: `sec-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`,
      timestamp: new Date(),
    };

    this.securityEvents.push(securityEvent);

    // Keep only last 1000 events to prevent memory leaks
    if (this.securityEvents.length > 1000) {
      this.securityEvents = this.securityEvents.slice(-500);
    }

    // Log critical security events
    if (event.severity === "critical" || event.severity === "high") {
      console.error(`🚨 SECURITY EVENT: ${event.type}`, {
        context: event.context?.agentId,
        resource: event.resource,
        action: event.action,
        result: event.result,
        details: event.details,
      });
    }
  }
}

/**
 * Rate limiter implementation
 */
class RateLimiter {
  private config: RateLimitConfig;
  private requests: number[] = [];
  private blockedUntil: number = 0;

  constructor(config: RateLimitConfig) {
    this.config = config;
  }

  checkLimit(): boolean {
    const now = Date.now();

    // Check if still blocked
    if (now < this.blockedUntil) {
      return false;
    }

    // Remove old requests outside the window
    this.requests = this.requests.filter(
      (timestamp) => now - timestamp < this.config.windowMs
    );

    // Check if under limit
    if (this.requests.length < this.config.requestsPerWindow) {
      this.requests.push(now);
      return true;
    }

    // Block for duration
    this.blockedUntil = now + this.config.blockDurationMs;
    return false;
  }

  getRemainingRequests(): number {
    const now = Date.now();
    this.requests = this.requests.filter(
      (timestamp) => now - timestamp < this.config.windowMs
    );
    return Math.max(0, this.config.requestsPerWindow - this.requests.length);
  }

  getBlockedUntil(): number {
    return this.blockedUntil;
  }
}

/**
 * Security error class
 */
export class SecurityError extends Error {
  constructor(message: string, public code: string, public details?: any) {
    super(message);
    this.name = "SecurityError";
  }
}

/**
 * Security middleware for protecting operations
 */
export class SecurityMiddleware {
  private securityManager: SecurityManager;

  constructor(securityManager: SecurityManager) {
    this.securityManager = securityManager;
  }

  /**
   * Protect an operation with security checks
   */
  async protect<T>(
    credentials: AuthCredentials,
    permission: Permission,
    action: keyof SecurityConfig["rateLimits"],
    operation: (context: SecurityContext) => Promise<T>
  ): Promise<T> {
    // Authenticate
    const context = this.securityManager.authenticate(credentials);
    if (!context) {
      throw new SecurityError("Authentication failed", "AUTH_FAILED");
    }

    // Authorize
    if (!this.securityManager.authorize(context, permission)) {
      throw new SecurityError("Authorization failed", "AUTHZ_FAILED");
    }

    // Rate limit
    if (!this.securityManager.checkRateLimit(context, action)) {
      throw new SecurityError("Rate limit exceeded", "RATE_LIMITED");
    }

    // Execute operation
    try {
      return await operation(context);
    } catch (error) {
      // Log operation failure
      console.error(`Operation failed for agent ${context.agentId}:`, error);
      throw error;
    }
  }
}
