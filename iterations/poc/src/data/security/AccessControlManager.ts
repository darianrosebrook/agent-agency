/**
 * @fileoverview Access Control Manager
 * @author @darianrosebrook
 *
 * Provides sophisticated access control beyond Row Level Security.
 * Implements attribute-based access control (ABAC) and policy enforcement.
 */

import { Logger } from "../../utils/Logger";

export interface AccessPolicy {
  id: string;
  name: string;
  description: string;
  effect: "allow" | "deny";
  principals: string[]; // User/Agent IDs or wildcards
  resources: string[]; // Resource patterns (e.g., "agents:*", "tasks:read")
  actions: string[]; // Actions (e.g., "read", "write", "delete")
  conditions?: AccessCondition[];
  priority: number;
  enabled: boolean;
}

export interface AccessCondition {
  type: "time" | "ip" | "role" | "attribute" | "relationship";
  operator: "equals" | "contains" | "regex" | "greater" | "less" | "between";
  attribute: string;
  value: any;
}

export interface AccessRequest {
  principal: string; // User/Agent ID
  resource: string; // Resource being accessed
  action: string; // Action being performed
  tenantId: string;
  context?: {
    ip?: string;
    timestamp?: Date;
    userAgent?: string;
    attributes?: Record<string, any>;
    relationships?: Record<string, any>;
  };
}

export interface AccessDecision {
  allowed: boolean;
  policy?: AccessPolicy;
  reason: string;
  evaluatedPolicies: number;
  executionTime: number;
}

export interface AccessControlConfig {
  enableAccessControl: boolean;
  defaultEffect: "allow" | "deny";
  policyEvaluationMode: "first-match" | "all-match";
  auditLogging: boolean;
  rateLimiting: {
    enabled: boolean;
    windowMs: number;
    maxRequests: number;
  };
}

export class AccessControlManager {
  private policies: Map<string, AccessPolicy> = new Map();
  private config: AccessControlConfig;
  private logger: Logger;
  private rateLimitCache: Map<string, { count: number; resetTime: number }> =
    new Map();

  constructor(config: Partial<AccessControlConfig> = {}, logger?: Logger) {
    this.config = {
      enableAccessControl: true,
      defaultEffect: "deny",
      policyEvaluationMode: "first-match",
      auditLogging: true,
      rateLimiting: {
        enabled: false,
        windowMs: 60000, // 1 minute
        maxRequests: 100,
      },
      ...config,
    };

    this.logger = logger || new Logger("AccessControlManager");

    // Initialize with default policies
    this.initializeDefaultPolicies();
  }

  /**
   * Initialize default access control policies
   */
  private initializeDefaultPolicies(): void {
    // Default deny-all policy (catch-all)
    const defaultDenyPolicy: AccessPolicy = {
      id: "default-deny",
      name: "Default Deny",
      description: "Deny all access by default",
      effect: "deny",
      principals: ["*"],
      resources: ["*"],
      actions: ["*"],
      priority: 0,
      enabled: true,
    };

    // Allow tenant admin access
    const tenantAdminPolicy: AccessPolicy = {
      id: "tenant-admin",
      name: "Tenant Admin Access",
      description: "Allow tenant administrators full access",
      effect: "allow",
      principals: ["role:tenant-admin"],
      resources: ["tenants:*", "agents:*", "tasks:*", "experiences:*"],
      actions: ["*"],
      priority: 100,
      enabled: true,
    };

    // Allow users to access their own data
    const userAccessPolicy: AccessPolicy = {
      id: "user-self-access",
      name: "User Self Access",
      description: "Allow users to access their own resources",
      effect: "allow",
      principals: ["*"],
      resources: ["agents:${principal}", "tasks:${principal}"],
      actions: ["read", "write", "update"],
      conditions: [
        {
          type: "relationship",
          operator: "equals",
          attribute: "resource.owner",
          value: "${principal}",
        },
      ],
      priority: 50,
      enabled: true,
    };

    this.addPolicy(defaultDenyPolicy);
    this.addPolicy(tenantAdminPolicy);
    this.addPolicy(userAccessPolicy);
  }

  /**
   * Add an access control policy
   */
  addPolicy(policy: AccessPolicy): void {
    this.policies.set(policy.id, policy);
    this.logger.info(
      `Added access control policy: ${policy.name} (${policy.id})`
    );
  }

  /**
   * Remove an access control policy
   */
  removePolicy(policyId: string): boolean {
    const removed = this.policies.delete(policyId);
    if (removed) {
      this.logger.info(`Removed access control policy: ${policyId}`);
    }
    return removed;
  }

  /**
   * Update an existing policy
   */
  updatePolicy(policyId: string, updates: Partial<AccessPolicy>): boolean {
    const existing = this.policies.get(policyId);
    if (!existing) {
      return false;
    }

    this.policies.set(policyId, { ...existing, ...updates });
    this.logger.info(`Updated access control policy: ${policyId}`);
    return true;
  }

  /**
   * Get a policy by ID
   */
  getPolicy(policyId: string): AccessPolicy | undefined {
    return this.policies.get(policyId);
  }

  /**
   * List all policies
   */
  listPolicies(): AccessPolicy[] {
    return Array.from(this.policies.values()).sort(
      (a, b) => b.priority - a.priority
    );
  }

  /**
   * Evaluate access request against policies
   */
  async evaluateAccess(request: AccessRequest): Promise<AccessDecision> {
    if (!this.config.enableAccessControl) {
      return {
        allowed: true,
        reason: "Access control disabled",
        evaluatedPolicies: 0,
        executionTime: 0,
      };
    }

    const startTime = Date.now();
    const evaluatedPolicies: AccessPolicy[] = [];

    try {
      // Check rate limiting first
      if (this.config.rateLimiting.enabled) {
        const rateLimitCheck = this.checkRateLimit(request);
        if (!rateLimitCheck.allowed) {
          return {
            allowed: false,
            reason: rateLimitCheck.reason,
            evaluatedPolicies: 0,
            executionTime: Date.now() - startTime,
          };
        }
      }

      // Sort policies by priority (highest first)
      const sortedPolicies = Array.from(this.policies.values())
        .filter((p) => p.enabled)
        .sort((a, b) => b.priority - a.priority);

      for (const policy of sortedPolicies) {
        if (this.matchesPolicy(request, policy)) {
          evaluatedPolicies.push(policy);

          // Check conditions if they exist
          if (
            policy.conditions &&
            !this.evaluateConditions(request, policy.conditions)
          ) {
            continue;
          }

          const decision: AccessDecision = {
            allowed: policy.effect === "allow",
            policy,
            reason: `${policy.effect.toUpperCase()}: ${policy.name}`,
            evaluatedPolicies: evaluatedPolicies.length,
            executionTime: Date.now() - startTime,
          };

          // Audit log the decision
          if (this.config.auditLogging) {
            this.logger.info(
              `Access ${decision.allowed ? "allowed" : "denied"}:`,
              {
                principal: request.principal,
                resource: request.resource,
                action: request.action,
                policy: policy.id,
                reason: decision.reason,
              }
            );
          }

          // Return first match for first-match mode
          if (this.config.policyEvaluationMode === "first-match") {
            return decision;
          }

          // For all-match mode, continue evaluating but track the decision
          if (!decision.allowed) {
            // Deny takes precedence in all-match mode
            return decision;
          }
        }
      }

      // No policy matched, use default effect
      const defaultDecision: AccessDecision = {
        allowed: this.config.defaultEffect === "allow",
        reason: `Default ${this.config.defaultEffect} (no policy matched)`,
        evaluatedPolicies: evaluatedPolicies.length,
        executionTime: Date.now() - startTime,
      };

      if (this.config.auditLogging) {
        this.logger.info(
          `Access ${defaultDecision.allowed ? "allowed" : "denied"} (default):`,
          {
            principal: request.principal,
            resource: request.resource,
            action: request.action,
            reason: defaultDecision.reason,
          }
        );
      }

      return defaultDecision;
    } catch (error) {
      this.logger.error("Access control evaluation failed:", error);
      return {
        allowed: false,
        reason: `Evaluation error: ${(error as Error).message}`,
        evaluatedPolicies: evaluatedPolicies.length,
        executionTime: Date.now() - startTime,
      };
    }
  }

  /**
   * Check if request matches a policy
   */
  private matchesPolicy(request: AccessRequest, policy: AccessPolicy): boolean {
    // Check principal
    if (
      !this.matchesPattern(
        request.principal,
        policy.principals,
        request.principal
      )
    ) {
      return false;
    }

    // Check resource
    if (
      !this.matchesPattern(
        request.resource,
        policy.resources,
        request.principal
      )
    ) {
      return false;
    }

    // Check action
    if (
      !this.matchesPattern(request.action, policy.actions, request.principal)
    ) {
      return false;
    }

    return true;
  }

  /**
   * Check if value matches any of the patterns
   */
  private matchesPattern(
    value: string,
    patterns: string[],
    principal: string
  ): boolean {
    return patterns.some((pattern) => {
      if (pattern === "*") {
        return true;
      }

      if (pattern.startsWith("role:")) {
        const role = pattern.substring(5);
        return this.hasRole(value, role);
      }

      if (pattern.includes("${principal}")) {
        const expanded = pattern.replace("${principal}", principal);
        return this.matchesSimplePattern(value, expanded);
      }

      return this.matchesSimplePattern(value, pattern);
    });
  }

  /**
   * Simple pattern matching (supports wildcards)
   */
  private matchesSimplePattern(value: string, pattern: string): boolean {
    if (pattern === "*") {
      return true;
    }

    // Convert pattern to regex
    const regexPattern = pattern.replace(/\*/g, ".*").replace(/\?/g, ".");

    const regex = new RegExp(`^${regexPattern}$`);
    return regex.test(value);
  }

  /**
   * Check if principal has a specific role
   */
  private hasRole(principal: string, role: string): boolean {
    // In a real implementation, this would check against a user/role database
    // For now, we'll do simple pattern matching
    return principal.includes(`role:${role}`) || principal === role;
  }

  /**
   * Evaluate policy conditions
   */
  private evaluateConditions(
    request: AccessRequest,
    conditions: AccessCondition[]
  ): boolean {
    return conditions.every((condition) => {
      const attributeValue = this.getAttributeValue(
        request,
        condition.attribute
      );

      // Special handling for time conditions
      if (condition.type === "time" && attributeValue instanceof Date) {
        const hour = attributeValue.getUTCHours();

        switch (condition.operator) {
          case "equals":
            return hour === Number(condition.value);
          case "greater":
            return hour > Number(condition.value);
          case "less":
            return hour < Number(condition.value);
          case "between": {
            const [min, max] = condition.value;
            return hour >= min && hour <= max;
          }
          default:
            return false;
        }
      }

      // General condition handling
      switch (condition.operator) {
        case "equals":
          return attributeValue === condition.value;
        case "contains":
          return Array.isArray(attributeValue)
            ? attributeValue.includes(condition.value)
            : String(attributeValue).includes(String(condition.value));
        case "regex": {
          const regex = new RegExp(condition.value);
          return regex.test(String(attributeValue));
        }
        case "greater":
          return Number(attributeValue) > Number(condition.value);
        case "less":
          return Number(attributeValue) < Number(condition.value);
        case "between": {
          const [min, max] = condition.value;
          const numValue = Number(attributeValue);
          return numValue >= min && numValue <= max;
        }
        default:
          return false;
      }
    });
  }

  /**
   * Get attribute value from request context
   */
  private getAttributeValue(request: AccessRequest, attribute: string): any {
    // Handle variable substitution
    if (attribute === "${principal}") {
      return request.principal;
    }

    if (attribute === "${tenant}") {
      return request.tenantId;
    }

    // Check context attributes
    if (request.context?.attributes) {
      if (attribute in request.context.attributes) {
        return request.context.attributes[attribute];
      }
    }

    // Check relationships
    if (request.context?.relationships) {
      if (attribute in request.context.relationships) {
        return request.context.relationships[attribute];
      }
    }

    // Default attributes
    switch (attribute) {
      case "timestamp":
        return request.context?.timestamp || new Date();
      case "ip":
        return request.context?.ip;
      case "userAgent":
        return request.context?.userAgent;
      default:
        return undefined;
    }
  }

  /**
   * Check rate limiting
   */
  private checkRateLimit(request: AccessRequest): {
    allowed: boolean;
    reason: string;
  } {
    const key = `${request.principal}:${request.tenantId}`;
    const now = Date.now();
    const _windowStart = now - this.config.rateLimiting.windowMs; // Not used but kept for clarity

    let rateLimit = this.rateLimitCache.get(key);

    if (!rateLimit || rateLimit.resetTime < now) {
      rateLimit = {
        count: 0,
        resetTime: now + this.config.rateLimiting.windowMs,
      };
    }

    rateLimit.count++;

    if (rateLimit.count > this.config.rateLimiting.maxRequests) {
      return {
        allowed: false,
        reason: `Rate limit exceeded: ${rateLimit.count}/${this.config.rateLimiting.maxRequests} requests in ${this.config.rateLimiting.windowMs}ms`,
      };
    }

    this.rateLimitCache.set(key, rateLimit);
    return { allowed: true, reason: "" };
  }

  /**
   * Get access control status
   */
  getStatus(): {
    enabled: boolean;
    policyCount: number;
    defaultEffect: string;
    rateLimitingEnabled: boolean;
    auditLoggingEnabled: boolean;
  } {
    return {
      enabled: this.config.enableAccessControl,
      policyCount: this.policies.size,
      defaultEffect: this.config.defaultEffect,
      rateLimitingEnabled: this.config.rateLimiting.enabled,
      auditLoggingEnabled: this.config.auditLogging,
    };
  }

  /**
   * Clear rate limit cache (for testing/admin purposes)
   */
  clearRateLimitCache(): void {
    this.rateLimitCache.clear();
    this.logger.info("Rate limit cache cleared");
  }
}
