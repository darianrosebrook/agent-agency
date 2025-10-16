/**
 * Tenant Isolator
 *
 * Manages tenant isolation and access control for multi-tenant memory operations.
 *
 * @author @darianrosebrook
 */

import { TenantAccessResult, TenantConfig } from "../types/memory.js";
import { Logger } from "../utils/Logger.js";

export class TenantIsolator {
  private logger: Logger;
  private tenantConfigs: Map<string, TenantConfig> = new Map();
  private tenantEvents: Map<string, any[]> = new Map();

  constructor(logger?: Logger) {
    this.logger = logger || new Logger("TenantIsolator");
  }

  /**
   * Register a tenant configuration
   */
  registerTenant(tenantId: string, config: TenantConfig): void {
    this.tenantConfigs.set(tenantId, config);
    this.logger.debug(`Registered tenant: ${tenantId}`, {
      isolationLevel: config.isolationLevel,
    });
  }

  /**
   * Validate tenant access to a resource/operation
   */
  async validateTenantAccess(
    tenantId: string,
    operation: string,
    _resource: string
  ): Promise<TenantAccessResult> {
    const config = this.tenantConfigs.get(tenantId);

    if (!config) {
      return {
        allowed: false,
        reason: `Tenant ${tenantId} not registered`,
      };
    }

    // Check operation-specific permissions
    if (operation === "federate" && config.isolationLevel !== "federated") {
      return {
        allowed: false,
        reason: `Tenant isolation level '${config.isolationLevel}' does not allow federation`,
      };
    }

    // Basic validation passed
    return {
      allowed: true,
      reason: "",
    };
  }

  /**
   * Get tenant configuration
   */
  getTenantConfig(tenantId: string): TenantConfig | undefined {
    return this.tenantConfigs.get(tenantId);
  }

  /**
   * Check if tenant has access to perform an operation
   */
  checkAccess(
    tenantId: string,
    operation: string
  ): { allowed: boolean; reason?: string } {
    const config = this.tenantConfigs.get(tenantId);
    if (!config) {
      return { allowed: false, reason: "Tenant not registered" };
    }

    // Check operation-specific permissions
    switch (operation) {
      case "read":
        return { allowed: true };
      case "write":
        return { allowed: config.isolationLevel !== "isolated" };
      case "federate":
        return { allowed: config.isolationLevel === "federated" };
      case "submit-update":
        return { allowed: config.isolationLevel === "federated" };
      default:
        return { allowed: false, reason: `Unknown operation: ${operation}` };
    }
  }

  /**
   * Store a tenant event for auditing
   */
  storeTenantEvent(tenantId: string, event: any): void {
    if (!this.tenantEvents.has(tenantId)) {
      this.tenantEvents.set(tenantId, []);
    }

    const events = this.tenantEvents.get(tenantId)!;
    events.push({
      ...event,
      tenantId,
      timestamp: new Date(),
    });

    // Keep only last 100 events per tenant
    if (events.length > 100) {
      events.shift();
    }
  }

  /**
   * Get recent tenant events
   */
  getTenantEvents(tenantId: string, limit: number = 50): any[] {
    const events = this.tenantEvents.get(tenantId) || [];
    return events.slice(-limit);
  }
}
