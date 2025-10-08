/**
 * Tenant Isolator - Multi-tenant data isolation and access control
 *
 * This component provides secure tenant isolation for the multi-tenant memory system,
 * ensuring that tenant data cannot leak between projects while allowing controlled
 * cross-tenant learning and intelligence sharing.
 *
 * @author @darianrosebrook
 */

import type {
  AccessRestriction,
  AuditEntry,
  IsolationResult,
  SharingCondition,
  TenantConfig,
  TenantContext,
  TenantPermissions,
} from "../types/index.js";
import { Logger } from "../utils/Logger";

/**
 * TenantIsolator - Core component for multi-tenant data isolation
 */
export class TenantIsolator {
  private logger: Logger;
  private tenantConfigs: Map<string, TenantConfig> = new Map();
  private activeContexts: Map<string, TenantContext> = new Map();
  private auditLogs: AuditEntry[] = [];

  constructor(logger?: Logger) {
    this.logger = logger || new Logger("TenantIsolator");
  }

  /**
   * Register a new tenant with isolation configuration
   */
  async registerTenant(config: TenantConfig): Promise<void> {
    // Validate tenant configuration
    await this.validateTenantConfig(config);

    // Store tenant configuration
    this.tenantConfigs.set(config.tenantId, config);

    // Create initial tenant context
    const context: TenantContext = {
      tenantId: config.tenantId,
      projectId: config.projectId,
      isolationLevel: config.isolationLevel,
      permissions: this.calculatePermissions(config),
      metadata: {
        registeredAt: new Date(),
        configVersion: "1.0",
      },
      createdAt: new Date(),
      lastAccessed: new Date(),
    };

    this.activeContexts.set(config.tenantId, context);

    this.logger.info(`Tenant registered: ${config.tenantId}`, {
      projectId: config.projectId,
      isolationLevel: config.isolationLevel,
    });

    // Audit the registration
    await this.auditOperation(
      config.tenantId,
      "tenant_register",
      "system",
      undefined,
      true,
      {
        config,
      }
    );
  }

  /**
   * Validate tenant access for a specific operation
   */
  async validateTenantAccess(
    tenantId: string,
    operation: string,
    resourceType?: string,
    resourceId?: string
  ): Promise<IsolationResult<boolean>> {
    const context = this.activeContexts.get(tenantId);
    const config = this.tenantConfigs.get(tenantId);

    if (!context || !config) {
      const reason = `Tenant not found: ${tenantId}`;
      this.logger.warn(reason);
      return {
        data: false,
        allowed: false,
        reason,
        auditLog: await this.auditOperation(
          tenantId,
          operation,
          resourceType || "unknown",
          resourceId,
          false,
          {
            error: "tenant_not_found",
          }
        ),
      };
    }

    // Update last accessed
    context.lastAccessed = new Date();
    this.activeContexts.set(tenantId, context);

    // Check basic permissions
    const hasPermission = this.checkOperationPermission(
      context.permissions,
      operation
    );
    if (!hasPermission) {
      const reason = `Operation not permitted: ${operation}`;
      return {
        data: false,
        allowed: false,
        reason,
        auditLog: await this.auditOperation(
          tenantId,
          operation,
          resourceType || "unknown",
          resourceId,
          false,
          {
            error: "permission_denied",
          }
        ),
      };
    }

    // Check resource-specific access if provided
    if (resourceType) {
      const resourceAccess = this.checkResourceAccess(
        config,
        tenantId,
        resourceType,
        resourceId,
        operation
      );
      if (!resourceAccess.allowed) {
        return {
          data: false,
          allowed: false,
          reason: resourceAccess.reason,
          auditLog: await this.auditOperation(
            tenantId,
            operation,
            resourceType,
            resourceId,
            false,
            {
              error: "resource_access_denied",
              details: resourceAccess.reason,
            }
          ),
        };
      }
    }

    // Check isolation level constraints
    const isolationCheck = this.checkIsolationConstraints(config, operation);
    if (!isolationCheck.allowed) {
      return {
        data: false,
        allowed: false,
        reason: isolationCheck.reason,
        auditLog: await this.auditOperation(
          tenantId,
          operation,
          resourceType || "unknown",
          resourceId,
          false,
          {
            error: "isolation_violation",
            details: isolationCheck.reason,
          }
        ),
      };
    }

    // All checks passed
    return {
      data: true,
      allowed: true,
      auditLog: await this.auditOperation(
        tenantId,
        operation,
        resourceType || "unknown",
        resourceId,
        true,
        {}
      ),
    };
  }

  /**
   * Get tenant-scoped data with isolation
   */
  async getTenantData<T>(
    tenantId: string,
    dataFetcher: () => Promise<T>,
    operation: string = "read"
  ): Promise<IsolationResult<T>> {
    const accessCheck = await this.validateTenantAccess(tenantId, operation);

    if (!accessCheck.allowed) {
      return {
        data: null,
        allowed: false,
        reason: accessCheck.reason,
        auditLog: accessCheck.auditLog,
      };
    }

    try {
      const data = await dataFetcher();
      return {
        data,
        allowed: true,
        auditLog: await this.auditOperation(
          tenantId,
          operation,
          "data",
          undefined,
          true,
          {
            dataSize: this.estimateDataSize(data),
          }
        ),
      };
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : String(error);
      const reason = `Data fetch failed: ${errorMessage}`;
      this.logger.error(reason, { tenantId, operation, error });
      return {
        data: null,
        allowed: false,
        reason,
        auditLog: await this.auditOperation(
          tenantId,
          operation,
          "data",
          undefined,
          false,
          {
            error: errorMessage,
          }
        ),
      };
    }
  }

  /**
   * Store tenant-scoped data with isolation
   */
  async storeTenantData<T>(
    tenantId: string,
    data: T,
    dataStorer: (data: T) => Promise<void>,
    operation: string = "write"
  ): Promise<IsolationResult<void>> {
    const accessCheck = await this.validateTenantAccess(tenantId, operation);

    if (!accessCheck.allowed) {
      return {
        data: null,
        allowed: false,
        reason: accessCheck.reason,
        auditLog: accessCheck.auditLog,
      };
    }

    try {
      await dataStorer(data);
      return {
        data: undefined,
        allowed: true,
        auditLog: await this.auditOperation(
          tenantId,
          operation,
          "data",
          undefined,
          true,
          {
            dataSize: this.estimateDataSize(data),
          }
        ),
      };
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : String(error);
      const reason = `Data storage failed: ${errorMessage}`;
      this.logger.error(reason, { tenantId, operation, error });
      return {
        data: null,
        allowed: false,
        reason,
        auditLog: await this.auditOperation(
          tenantId,
          operation,
          "data",
          undefined,
          false,
          {
            error: errorMessage,
          }
        ),
      };
    }
  }

  /**
   * Check if cross-tenant sharing is allowed
   */
  async canShareWithTenant(
    sourceTenantId: string,
    targetTenantId: string,
    resourceType: string,
    resourceId?: string
  ): Promise<IsolationResult<boolean>> {
    const sourceConfig = this.tenantConfigs.get(sourceTenantId);
    const targetConfig = this.tenantConfigs.get(targetTenantId);

    if (!sourceConfig || !targetConfig) {
      const reason = "One or both tenants not found";
      return {
        data: false,
        allowed: false,
        reason,
      };
    }

    // Check sharing rules
    const sharingRule = sourceConfig.sharingRules.find(
      (rule) => rule.targetTenant === targetTenantId
    );

    if (!sharingRule) {
      const reason = `No sharing rule defined for ${targetTenantId}`;
      return {
        data: false,
        allowed: false,
        reason,
      };
    }

    // Check if resource type is allowed
    if (!sharingRule.resourceTypes.includes(resourceType)) {
      const reason = `Resource type ${resourceType} not allowed for sharing`;
      return {
        data: false,
        allowed: false,
        reason,
      };
    }

    // Check sharing conditions
    const conditionsMet = this.evaluateSharingConditions(
      sharingRule.conditions,
      {
        sourceTenant: sourceTenantId,
        targetTenant: targetTenantId,
        resourceType,
        resourceId,
      }
    );

    if (!conditionsMet) {
      const reason = "Sharing conditions not met";
      return {
        data: false,
        allowed: false,
        reason,
      };
    }

    return {
      data: true,
      allowed: true,
    };
  }

  /**
   * Get audit logs for a tenant
   */
  getAuditLogs(tenantId: string, limit: number = 100): AuditEntry[] {
    return this.auditLogs
      .filter((log) => log.tenantId === tenantId)
      .sort((a, b) => b.timestamp.getTime() - a.timestamp.getTime())
      .slice(0, limit);
  }

  /**
   * Get tenant context
   */
  getTenantContext(tenantId: string): TenantContext | null {
    return this.activeContexts.get(tenantId) || null;
  }

  /**
   * List all registered tenants
   */
  listTenants(): string[] {
    return Array.from(this.tenantConfigs.keys());
  }

  // Private helper methods

  private async validateTenantConfig(config: TenantConfig): Promise<void> {
    if (!config.tenantId || !config.projectId) {
      throw new Error("Tenant ID and Project ID are required");
    }

    if (!["strict", "shared", "federated"].includes(config.isolationLevel)) {
      throw new Error("Invalid isolation level");
    }

    // Check for duplicate tenant ID
    if (this.tenantConfigs.has(config.tenantId)) {
      throw new Error(`Tenant ${config.tenantId} already exists`);
    }
  }

  private calculatePermissions(config: TenantConfig): TenantPermissions {
    const basePermissions: TenantPermissions = {
      canRead: true,
      canWrite: true,
      canShare: config.isolationLevel !== "strict",
      canFederate: config.isolationLevel === "federated",
      allowedOperations: ["read", "write"],
      resourceLimits: {},
    };

    if (config.isolationLevel === "shared") {
      basePermissions.allowedOperations.push("share");
    }

    if (config.isolationLevel === "federated") {
      basePermissions.allowedOperations.push("share", "federate");
    }

    return basePermissions;
  }

  private checkOperationPermission(
    permissions: TenantPermissions,
    operation: string
  ): boolean {
    return (
      permissions.allowedOperations.includes(operation) ||
      (operation === "read" && permissions.canRead) ||
      (operation === "write" && permissions.canWrite) ||
      (operation === "share" && permissions.canShare) ||
      (operation === "federate" && permissions.canFederate)
    );
  }

  private checkResourceAccess(
    config: TenantConfig,
    tenantId: string,
    resourceType: string,
    resourceId?: string,
    operation?: string
  ): { allowed: boolean; reason?: string } {
    const policy = config.accessPolicies.find(
      (p) => p.resourceType === resourceType
    );
    if (!policy) {
      return { allowed: false, reason: `No access policy for ${resourceType}` };
    }

    // Check if operation is allowed
    if (
      !policy.allowedTenants.includes(tenantId) &&
      !policy.allowedTenants.includes("*")
    ) {
      return {
        allowed: false,
        reason: `Tenant not allowed for ${resourceType}`,
      };
    }

    // Check if operation matches access level
    if (operation && !this.checkAccessLevel(policy.accessLevel, operation)) {
      return {
        allowed: false,
        reason: `Operation '${operation}' not permitted by access level '${policy.accessLevel}'`,
      };
    }

    // Check restrictions
    for (const restriction of policy.restrictions) {
      if (!this.checkRestriction(restriction, { tenantId, resourceId })) {
        return { allowed: false, reason: restriction.description };
      }
    }

    return { allowed: true };
  }

  private checkAccessLevel(accessLevel: string, operation: string): boolean {
    switch (accessLevel) {
      case "read":
        return operation === "read";
      case "write":
        return operation === "read" || operation === "write";
      case "share":
        return (
          operation === "read" || operation === "write" || operation === "share"
        );
      case "federate":
        return (
          operation === "read" ||
          operation === "write" ||
          operation === "share" ||
          operation === "federate"
        );
      default:
        return false;
    }
  }

  private checkRestriction(
    restriction: AccessRestriction,
    context: any
  ): boolean {
    switch (restriction.type) {
      case "time_based":
        const now = new Date();
        const startTime = new Date(restriction.value.start);
        const endTime = new Date(restriction.value.end);
        return now >= startTime && now <= endTime;

      case "data_sensitivity":
        // Implement sensitivity checks based on your data classification
        return true; // Placeholder

      case "usage_limit":
        // Implement usage tracking and limits
        return true; // Placeholder

      default:
        return false;
    }
  }

  private checkIsolationConstraints(
    config: TenantConfig,
    operation: string
  ): { allowed: boolean; reason?: string } {
    switch (config.isolationLevel) {
      case "strict":
        if (["share", "federate"].includes(operation)) {
          return {
            allowed: false,
            reason: "Strict isolation does not allow sharing",
          };
        }
        break;

      case "shared":
        if (operation === "federate") {
          return {
            allowed: false,
            reason: "Shared isolation does not allow federation",
          };
        }
        break;

      case "federated":
        // All operations allowed
        break;
    }

    return { allowed: true };
  }

  private evaluateSharingConditions(
    conditions: SharingCondition[],
    context: any
  ): boolean {
    return conditions.every((condition) => {
      // Implement condition evaluation logic
      // This is a placeholder - you'd implement specific logic for each condition type
      return true;
    });
  }

  private async auditOperation(
    tenantId: string,
    operation: string,
    resourceType: string,
    resourceId: string | undefined,
    success: boolean,
    details: Record<string, any>
  ): Promise<AuditEntry> {
    const entry: AuditEntry = {
      tenantId,
      operation,
      resourceType,
      resourceId,
      timestamp: new Date(),
      success,
      details,
    };

    this.auditLogs.push(entry);

    // Keep only last 10000 entries to prevent memory issues
    if (this.auditLogs.length > 10000) {
      this.auditLogs = this.auditLogs.slice(-5000);
    }

    return entry;
  }

  private estimateDataSize(data: any): number {
    try {
      return JSON.stringify(data).length;
    } catch {
      return 0;
    }
  }
}
