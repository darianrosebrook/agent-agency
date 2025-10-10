/**
 * @fileoverview Security Layer for Agent Registry (ARBITER-001)
 *
 * Provides authentication, authorization, input validation, and multi-tenant isolation
 * for the agent registry system.
 *
 * @author @darianrosebrook
 */

import {
  AgentProfile,
  AgentId,
  PerformanceMetrics,
  AgentQuery,
} from "../types/agent-registry";

/**
 * Security Context for operations
 */
export interface SecurityContext {
  /** Tenant identifier for multi-tenant isolation */
  tenantId: string;

  /** User identifier */
  userId: string;

  /** User roles */
  roles: string[];

  /** Request timestamp */
  requestedAt: Date;

  /** Request identifier for audit logging */
  requestId: string;

  /** IP address for rate limiting */
  ipAddress?: string;
}

/**
 * Security Configuration
 */
export interface SecurityConfig {
  /** Enable authentication */
  authenticationEnabled: boolean;

  /** Enable authorization */
  authorizationEnabled: boolean;

  /** Enable multi-tenant isolation */
  multiTenantEnabled: boolean;

  /** Enable audit logging */
  auditLoggingEnabled: boolean;

  /** Enable rate limiting */
  rateLimitingEnabled: boolean;

  /** Rate limit: requests per minute */
  rateLimitPerMinute: number;

  /** Allowed roles for agent registration */
  allowedRegistrationRoles: string[];

  /** Allowed roles for agent modification */
  allowedModificationRoles: string[];

  /** Allowed roles for agent deletion */
  allowedDeletionRoles: string[];
}

/**
 * Security Audit Entry
 */
export interface AuditEntry {
  id: string;
  tenantId: string;
  userId: string;
  operation: string;
  resource: string;
  resourceId: string;
  timestamp: Date;
  success: boolean;
  errorMessage?: string;
  metadata: Record<string, any>;
}

/**
 * Rate Limit Tracker
 */
interface RateLimitEntry {
  count: number;
  resetAt: Date;
}

/**
 * Security Error
 */
export class SecurityError extends Error {
  constructor(
    message: string,
    public code: string,
    public context?: SecurityContext
  ) {
    super(message);
    this.name = "SecurityError";
  }
}

/**
 * Agent Registry Security Layer
 *
 * Enforces authentication, authorization, input validation, and multi-tenant isolation.
 */
export class AgentRegistrySecurity {
  private config: SecurityConfig;
  private auditLog: AuditEntry[] = [];
  private rateLimits: Map<string, RateLimitEntry> = new Map();

  constructor(config: Partial<SecurityConfig> = {}) {
    this.config = {
      authenticationEnabled: true,
      authorizationEnabled: true,
      multiTenantEnabled: true,
      auditLoggingEnabled: true,
      rateLimitingEnabled: true,
      rateLimitPerMinute: 100,
      allowedRegistrationRoles: ["admin", "agent-manager"],
      allowedModificationRoles: ["admin", "agent-manager", "orchestrator"],
      allowedDeletionRoles: ["admin"],
      ...config,
    };
  }

  /**
   * Authenticate request and create security context
   */
  authenticateRequest(
    token: string,
    requestId: string,
    ipAddress?: string
  ): SecurityContext {
    if (!this.config.authenticationEnabled) {
      return this.createAnonymousContext(requestId);
    }

    // TODO: Implement actual token validation (JWT, OAuth, etc.)
    // For now, parse a simple token format: "tenant:user:roles"
    const parts = Buffer.from(token, "base64").toString("utf8").split(":");

    if (parts.length < 3) {
      throw new SecurityError(
        "Invalid authentication token",
        "INVALID_TOKEN"
      );
    }

    return {
      tenantId: parts[0],
      userId: parts[1],
      roles: parts[2].split(","),
      requestedAt: new Date(),
      requestId,
      ipAddress,
    };
  }

  /**
   * Authorize registration operation
   */
  authorizeRegistration(context: SecurityContext): void {
    if (!this.config.authorizationEnabled) {
      return;
    }

    const hasPermission = context.roles.some((role) =>
      this.config.allowedRegistrationRoles.includes(role)
    );

    if (!hasPermission) {
      this.logAuditEntry({
        ...this.createAuditEntry(context, "register_agent", "agent", "unknown"),
        success: false,
        errorMessage: "Insufficient permissions",
      });

      throw new SecurityError(
        "Insufficient permissions to register agents",
        "UNAUTHORIZED",
        context
      );
    }
  }

  /**
   * Authorize modification operation
   */
  authorizeModification(context: SecurityContext): void {
    if (!this.config.authorizationEnabled) {
      return;
    }

    const hasPermission = context.roles.some((role) =>
      this.config.allowedModificationRoles.includes(role)
    );

    if (!hasPermission) {
      throw new SecurityError(
        "Insufficient permissions to modify agents",
        "UNAUTHORIZED",
        context
      );
    }
  }

  /**
   * Authorize deletion operation
   */
  authorizeDeletion(context: SecurityContext): void {
    if (!this.config.authorizationEnabled) {
      return;
    }

    const hasPermission = context.roles.some((role) =>
      this.config.allowedDeletionRoles.includes(role)
    );

    if (!hasPermission) {
      throw new SecurityError(
        "Insufficient permissions to delete agents",
        "UNAUTHORIZED",
        context
      );
    }
  }

  /**
   * Check rate limit
   */
  checkRateLimit(context: SecurityContext): void {
    if (!this.config.rateLimitingEnabled) {
      return;
    }

    const key = `${context.tenantId}:${context.userId}`;
    const now = new Date();
    const limit = this.rateLimits.get(key);

    if (limit && limit.resetAt > now) {
      limit.count++;

      if (limit.count > this.config.rateLimitPerMinute) {
        throw new SecurityError(
          `Rate limit exceeded: ${this.config.rateLimitPerMinute} requests per minute`,
          "RATE_LIMIT_EXCEEDED",
          context
        );
      }
    } else {
      this.rateLimits.set(key, {
        count: 1,
        resetAt: new Date(now.getTime() + 60000), // 1 minute from now
      });
    }
  }

  /**
   * Validate agent profile data
   */
  validateAgentProfile(agent: Partial<AgentProfile>): void {
    // ID validation
    if (!agent.id || typeof agent.id !== "string") {
      throw new SecurityError(
        "Agent ID is required and must be a string",
        "INVALID_INPUT"
      );
    }

    if (agent.id.length > 100) {
      throw new SecurityError(
        "Agent ID too long (max 100 characters)",
        "INVALID_INPUT"
      );
    }

    // Name validation
    if (!agent.name || typeof agent.name !== "string") {
      throw new SecurityError(
        "Agent name is required and must be a string",
        "INVALID_INPUT"
      );
    }

    if (agent.name.length > 200) {
      throw new SecurityError(
        "Agent name too long (max 200 characters)",
        "INVALID_INPUT"
      );
    }

    // Sanitize string inputs
    agent.id = this.sanitizeString(agent.id);
    agent.name = this.sanitizeString(agent.name);

    // Validate capabilities
    if (agent.capabilities) {
      if (agent.capabilities.taskTypes.length > 50) {
        throw new SecurityError(
          "Too many task types (max 50)",
          "INVALID_INPUT"
        );
      }

      if (agent.capabilities.languages.length > 50) {
        throw new SecurityError(
          "Too many languages (max 50)",
          "INVALID_INPUT"
        );
      }

      if (agent.capabilities.specializations.length > 50) {
        throw new SecurityError(
          "Too many specializations (max 50)",
          "INVALID_INPUT"
        );
      }
    }
  }

  /**
   * Validate performance metrics
   */
  validatePerformanceMetrics(metrics: PerformanceMetrics): void {
    if (metrics.qualityScore < 0 || metrics.qualityScore > 1) {
      throw new SecurityError(
        "Quality score must be between 0 and 1",
        "INVALID_INPUT"
      );
    }

    if (metrics.latencyMs < 0 || metrics.latencyMs > 300000) {
      throw new SecurityError(
        "Latency must be between 0 and 300000ms (5 minutes)",
        "INVALID_INPUT"
      );
    }

    if (metrics.tokensUsed !== undefined && (metrics.tokensUsed < 0 || metrics.tokensUsed > 1000000)) {
      throw new SecurityError(
        "Tokens used must be between 0 and 1,000,000",
        "INVALID_INPUT"
      );
    }
  }

  /**
   * Validate query parameters
   */
  validateQuery(query: AgentQuery): void {
    if (query.maxUtilization !== undefined) {
      if (query.maxUtilization < 0 || query.maxUtilization > 100) {
        throw new SecurityError(
          "Max utilization must be between 0 and 100",
          "INVALID_INPUT"
        );
      }
    }

    if (query.minSuccessRate !== undefined) {
      if (query.minSuccessRate < 0 || query.minSuccessRate > 1) {
        throw new SecurityError(
          "Min success rate must be between 0 and 1",
          "INVALID_INPUT"
        );
      }
    }

    if (query.languages && query.languages.length > 20) {
      throw new SecurityError(
        "Too many languages in query (max 20)",
        "INVALID_INPUT"
      );
    }
  }

  /**
   * Enforce tenant isolation
   */
  scopeToTenant(
    agentId: AgentId,
    context: SecurityContext
  ): AgentId {
    if (!this.config.multiTenantEnabled) {
      return agentId;
    }

    // Ensure agent ID is scoped to tenant
    const scopedId = `${context.tenantId}:${agentId}`;
    return scopedId as AgentId;
  }

  /**
   * Extract agent ID from scoped ID
   */
  unscopeAgentId(scopedId: AgentId): string {
    if (!this.config.multiTenantEnabled) {
      return scopedId;
    }

    const parts = scopedId.split(":");
    return parts.length > 1 ? parts[1] : scopedId;
  }

  /**
   * Verify agent belongs to tenant
   */
  verifyTenantOwnership(
    agentId: AgentId,
    context: SecurityContext
  ): void {
    if (!this.config.multiTenantEnabled) {
      return;
    }

    if (!agentId.startsWith(`${context.tenantId}:`)) {
      throw new SecurityError(
        "Agent does not belong to your tenant",
        "UNAUTHORIZED",
        context
      );
    }
  }

  /**
   * Log audit entry
   */
  logAuditEntry(entry: AuditEntry): void {
    if (!this.config.auditLoggingEnabled) {
      return;
    }

    this.auditLog.push(entry);

    // TODO: Persist to database or external audit system
    console.log(`[AUDIT] ${entry.operation} on ${entry.resource}:${entry.resourceId} by ${entry.userId} - ${entry.success ? "SUCCESS" : "FAILURE"}`);
  }

  /**
   * Get audit log
   */
  getAuditLog(
    tenantId?: string,
    limit: number = 100
  ): AuditEntry[] {
    let filtered = this.auditLog;

    if (tenantId) {
      filtered = filtered.filter((entry) => entry.tenantId === tenantId);
    }

    return filtered.slice(-limit);
  }

  /**
   * Create audit entry
   */
  private createAuditEntry(
    context: SecurityContext,
    operation: string,
    resource: string,
    resourceId: string
  ): Omit<AuditEntry, "success"> {
    return {
      id: `audit-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      tenantId: context.tenantId,
      userId: context.userId,
      operation,
      resource,
      resourceId,
      timestamp: new Date(),
      metadata: {
        requestId: context.requestId,
        roles: context.roles,
        ipAddress: context.ipAddress,
      },
    };
  }

  /**
   * Create anonymous context for non-authenticated mode
   */
  private createAnonymousContext(requestId: string): SecurityContext {
    return {
      tenantId: "default",
      userId: "anonymous",
      roles: ["public"],
      requestedAt: new Date(),
      requestId,
    };
  }

  /**
   * Sanitize string input to prevent injection attacks
   */
  private sanitizeString(input: string): string {
    // Remove null bytes
    let sanitized = input.replace(/\0/g, "");

    // Trim whitespace
    sanitized = sanitized.trim();

    // Remove control characters
    // eslint-disable-next-line no-control-regex
    sanitized = sanitized.replace(/[\x00-\x1F\x7F]/g, "");

    return sanitized;
  }
}

/**
 * Secure Agent Registry Wrapper
 *
 * Wraps AgentRegistryManager with security enforcement.
 */
export class SecureAgentRegistry {
  private security: AgentRegistrySecurity;

  constructor(
    private registry: any, // AgentRegistryManager
    securityConfig?: Partial<SecurityConfig>
  ) {
    this.security = new AgentRegistrySecurity(securityConfig);
  }

  /**
   * Register agent with security checks
   */
  async registerAgent(
    agent: AgentProfile,
    context: SecurityContext
  ): Promise<AgentProfile> {
    // Rate limiting
    this.security.checkRateLimit(context);

    // Authorization
    this.security.authorizeRegistration(context);

    // Input validation
    this.security.validateAgentProfile(agent);

    // Tenant scoping
    const scopedAgent = {
      ...agent,
      id: this.security.scopeToTenant(agent.id, context),
    };

    try {
      const result = await this.registry.registerAgent(scopedAgent);

      // Audit log
      this.security.logAuditEntry({
        ...this.createBaseAuditEntry(context, "register_agent", "agent", agent.id),
        success: true,
      });

      return result;
    } catch (error) {
      // Audit log failure
      this.security.logAuditEntry({
        ...this.createBaseAuditEntry(context, "register_agent", "agent", agent.id),
        success: false,
        errorMessage: error instanceof Error ? error.message : String(error),
      });

      throw error;
    }
  }

  /**
   * Get agent with tenant isolation
   */
  async getAgent(
    agentId: AgentId,
    context: SecurityContext
  ): Promise<AgentProfile | null> {
    // Rate limiting
    this.security.checkRateLimit(context);

    // Tenant scoping
    const scopedId = this.security.scopeToTenant(agentId, context);

    const result = await this.registry.getProfile(scopedId);

    // Audit log
    this.security.logAuditEntry({
      ...this.createBaseAuditEntry(context, "get_agent", "agent", agentId),
      success: true,
    });

    return result;
  }

  /**
   * Query agents with tenant isolation
   */
  async queryAgents(
    query: AgentQuery,
    context: SecurityContext
  ): Promise<AgentProfile[]> {
    // Rate limiting
    this.security.checkRateLimit(context);

    // Input validation
    this.security.validateQuery(query);

    // Get agents (registry will handle capability filtering)
    const results = await this.registry.getAgentsByCapability(query);

    // Filter by tenant
    const tenantResults = results.filter((result: any) =>
      result.agent.id.startsWith(`${context.tenantId}:`)
    );

    // Audit log
    this.security.logAuditEntry({
      ...this.createBaseAuditEntry(context, "query_agents", "agent", "query"),
      success: true,
      metadata: {
        requestId: context.requestId,
        roles: context.roles,
        ipAddress: context.ipAddress,
        queryParams: query,
        resultCount: tenantResults.length,
      },
    });

    return tenantResults.map((r: any) => r.agent);
  }

  /**
   * Update performance with security checks
   */
  async updatePerformance(
    agentId: AgentId,
    metrics: PerformanceMetrics,
    context: SecurityContext
  ): Promise<AgentProfile> {
    // Rate limiting
    this.security.checkRateLimit(context);

    // Authorization
    this.security.authorizeModification(context);

    // Input validation
    this.security.validatePerformanceMetrics(metrics);

    // Tenant verification
    const scopedId = this.security.scopeToTenant(agentId, context);
    this.security.verifyTenantOwnership(scopedId, context);

    try {
      const result = await this.registry.updatePerformance(scopedId, metrics);

      // Audit log
      this.security.logAuditEntry({
        ...this.createBaseAuditEntry(context, "update_performance", "agent", agentId),
        success: true,
        metadata: {
          requestId: context.requestId,
          roles: context.roles,
          ipAddress: context.ipAddress,
          metrics,
        },
      });

      return result;
    } catch (error) {
      // Audit log failure
      this.security.logAuditEntry({
        ...this.createBaseAuditEntry(context, "update_performance", "agent", agentId),
        success: false,
        errorMessage: error instanceof Error ? error.message : String(error),
      });

      throw error;
    }
  }

  /**
   * Unregister agent with security checks
   */
  async unregisterAgent(
    agentId: AgentId,
    context: SecurityContext
  ): Promise<boolean> {
    // Rate limiting
    this.security.checkRateLimit(context);

    // Authorization
    this.security.authorizeDeletion(context);

    // Tenant verification
    const scopedId = this.security.scopeToTenant(agentId, context);
    this.security.verifyTenantOwnership(scopedId, context);

    try {
      const result = await this.registry.unregisterAgent(scopedId);

      // Audit log
      this.security.logAuditEntry({
        ...this.createBaseAuditEntry(context, "unregister_agent", "agent", agentId),
        success: true,
      });

      return result;
    } catch (error) {
      // Audit log failure
      this.security.logAuditEntry({
        ...this.createBaseAuditEntry(context, "unregister_agent", "agent", agentId),
        success: false,
        errorMessage: error instanceof Error ? error.message : String(error),
      });

      throw error;
    }
  }

  /**
   * Get audit log for tenant
   */
  getAuditLog(context: SecurityContext, limit?: number): AuditEntry[] {
    return this.security.getAuditLog(context.tenantId, limit);
  }

  /**
   * Create base audit entry
   */
  private createBaseAuditEntry(
    context: SecurityContext,
    operation: string,
    resource: string,
    resourceId: string
  ): Omit<AuditEntry, "success"> {
    return {
      id: `audit-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      tenantId: context.tenantId,
      userId: context.userId,
      operation,
      resource,
      resourceId,
      timestamp: new Date(),
      metadata: {
        requestId: context.requestId,
        roles: context.roles,
        ipAddress: context.ipAddress,
      },
    };
  }
}

