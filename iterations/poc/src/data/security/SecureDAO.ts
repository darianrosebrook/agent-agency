/**
 * @fileoverview Secure Data Access Object Base Class
 * @author @darianrosebrook
 *
 * Base class for DAOs that require encryption and access control.
 * Provides secure data operations with automatic encryption/decryption.
 */

import { Logger } from "../../utils/Logger";
import { DataLayer } from "../DataLayer";
import { BaseDAO } from "../dao/BaseDAO";
import { QueryOptions } from "../types";
import {
  AccessControlManager,
  AccessDecision,
  AccessRequest,
} from "./AccessControlManager";
import { EncryptionManager } from "./EncryptionManager";

export interface SecurityConfig {
  encryptionEnabled: boolean;
  accessControlEnabled: boolean;
  sensitiveFields: string[];
  resourceType: string;
  requireTenantIsolation: boolean;
}

export interface SecureOperationResult<T = any> {
  success: boolean;
  data?: T;
  error?: string;
  accessGranted: boolean;
  accessDecision?: AccessDecision;
  encrypted: boolean;
}

export abstract class SecureDAO<
  T extends { id: string; tenantId: string; createdAt: Date; updatedAt: Date }
> extends BaseDAO<T> {
  protected encryptionManager?: EncryptionManager;
  protected accessControlManager?: AccessControlManager;
  protected securityConfig: SecurityConfig;
  protected logger: Logger;

  constructor(
    dataLayer: DataLayer,
    tableName: string,
    entityName: string,
    securityConfig: SecurityConfig,
    encryptionManager?: EncryptionManager,
    accessControlManager?: AccessControlManager,
    logger?: Logger
  ) {
    super(dataLayer, tableName, entityName, logger);
    this.securityConfig = securityConfig;
    this.encryptionManager = encryptionManager;
    this.accessControlManager = accessControlManager;
    this.logger = logger || new Logger(`SecureDAO:${tableName}`);
  }

  /**
   * Secure create operation with access control and encryption
   */
  async secureCreate(
    data: Partial<T>,
    principal: string,
    tenantId: string,
    context?: AccessRequest["context"]
  ): Promise<SecureOperationResult<T>> {
    try {
      // Check access control
      const accessRequest: AccessRequest = {
        principal,
        resource: `${this.securityConfig.resourceType}:*`,
        action: "create",
        tenantId,
        context,
      };

      const accessDecision = await this.checkAccess(accessRequest);
      if (!accessDecision.allowed) {
        return {
          success: false,
          error: `Access denied: ${accessDecision.reason}`,
          accessGranted: false,
          accessDecision,
          encrypted: false,
        };
      }

      // Prepare data for storage
      let processedData = { ...data } as Partial<T>;

      // Encrypt sensitive fields
      if (this.securityConfig.encryptionEnabled && this.encryptionManager) {
        processedData = (await this.encryptionManager.encryptFields(
          processedData as Record<string, any>,
          this.securityConfig.sensitiveFields
        )) as Partial<T>;
      }

      // Add tenant isolation if required
      if (this.securityConfig.requireTenantIsolation) {
        (processedData as any).tenant_id = tenantId;
      }

      // Perform the actual create operation
      const result = await this.create(
        processedData as Omit<T, "id" | "createdAt" | "updatedAt">
      );

      return {
        success: result.success,
        data: result.data,
        error: result.error,
        accessGranted: true,
        accessDecision,
        encrypted: this.securityConfig.encryptionEnabled,
      };
    } catch (error) {
      this.logger.error("Secure create operation failed:", error);
      return {
        success: false,
        error: `Create operation failed: ${(error as Error).message}`,
        accessGranted: false,
        encrypted: false,
      };
    }
  }

  /**
   * Secure read operation with access control and decryption
   */
  async secureRead(
    id: string,
    principal: string,
    tenantId: string,
    context?: AccessRequest["context"]
  ): Promise<SecureOperationResult<T>> {
    try {
      // Check access control
      const accessRequest: AccessRequest = {
        principal,
        resource: `${this.securityConfig.resourceType}:${id}`,
        action: "read",
        tenantId,
        context,
      };

      const accessDecision = await this.checkAccess(accessRequest);
      if (!accessDecision.allowed) {
        return {
          success: false,
          error: `Access denied: ${accessDecision.reason}`,
          accessGranted: false,
          accessDecision,
          encrypted: false,
        };
      }

      // Perform the actual read operation
      const result = await this.findById(id, tenantId);

      if (!result.success || !result.data) {
        return {
          success: false,
          data: undefined,
          error: result.error,
          accessGranted: true,
          accessDecision,
          encrypted: false,
        };
      }

      // Decrypt sensitive fields
      let processedData = result.data;
      if (this.securityConfig.encryptionEnabled && this.encryptionManager) {
        processedData = (await this.encryptionManager.decryptFields(
          processedData as Record<string, any>,
          this.securityConfig.sensitiveFields
        )) as T;
      }

      return {
        success: true,
        data: processedData,
        accessGranted: true,
        accessDecision,
        encrypted: this.securityConfig.encryptionEnabled,
      };
    } catch (error) {
      this.logger.error("Secure read operation failed:", error);
      return {
        success: false,
        error: `Read operation failed: ${(error as Error).message}`,
        accessGranted: false,
        encrypted: false,
      };
    }
  }

  /**
   * Secure update operation with access control and encryption
   */
  async secureUpdate(
    id: string,
    data: Partial<T>,
    principal: string,
    tenantId: string,
    context?: AccessRequest["context"]
  ): Promise<SecureOperationResult<T>> {
    try {
      // Check access control
      const accessRequest: AccessRequest = {
        principal,
        resource: `${this.securityConfig.resourceType}:${id}`,
        action: "update",
        tenantId,
        context,
      };

      const accessDecision = await this.checkAccess(accessRequest);
      if (!accessDecision.allowed) {
        return {
          success: false,
          error: `Access denied: ${accessDecision.reason}`,
          accessGranted: false,
          accessDecision,
          encrypted: false,
        };
      }

      // Prepare data for storage
      let processedData = { ...data } as Partial<T>;

      // Encrypt sensitive fields
      if (this.securityConfig.encryptionEnabled && this.encryptionManager) {
        processedData = (await this.encryptionManager.encryptFields(
          processedData as Record<string, any>,
          this.securityConfig.sensitiveFields
        )) as Partial<T>;
      }

      // Perform the actual update operation
      const result = await this.update(id, tenantId, processedData);

      return {
        success: result.success,
        data: result.data,
        error: result.error,
        accessGranted: true,
        accessDecision,
        encrypted: this.securityConfig.encryptionEnabled,
      };
    } catch (error) {
      this.logger.error("Secure update operation failed:", error);
      return {
        success: false,
        error: `Update operation failed: ${(error as Error).message}`,
        accessGranted: false,
        encrypted: false,
      };
    }
  }

  /**
   * Secure delete operation with access control
   */
  async secureDelete(
    id: string,
    principal: string,
    tenantId: string,
    context?: AccessRequest["context"]
  ): Promise<SecureOperationResult<boolean>> {
    try {
      // Check access control
      const accessRequest: AccessRequest = {
        principal,
        resource: `${this.securityConfig.resourceType}:${id}`,
        action: "delete",
        tenantId,
        context,
      };

      const accessDecision = await this.checkAccess(accessRequest);
      if (!accessDecision.allowed) {
        return {
          success: false,
          error: `Access denied: ${accessDecision.reason}`,
          accessGranted: false,
          accessDecision,
          encrypted: false,
        };
      }

      // Perform the actual delete operation
      const result = await this.delete(id, tenantId);

      return {
        success: result.success,
        data: result.success,
        error: result.error,
        accessGranted: true,
        accessDecision,
        encrypted: false, // Delete doesn't involve encryption
      };
    } catch (error) {
      this.logger.error("Secure delete operation failed:", error);
      return {
        success: false,
        error: `Delete operation failed: ${(error as Error).message}`,
        accessGranted: false,
        encrypted: false,
      };
    }
  }

  /**
   * Secure query operation with access control and decryption
   */
  async secureQuery(
    conditions: Record<string, any>,
    principal: string,
    tenantId: string,
    options?: { limit?: number; offset?: number },
    context?: AccessRequest["context"]
  ): Promise<SecureOperationResult<T[]>> {
    try {
      // Check access control for query operation
      const accessRequest: AccessRequest = {
        principal,
        resource: `${this.securityConfig.resourceType}:*`,
        action: "query",
        tenantId,
        context,
      };

      const accessDecision = await this.checkAccess(accessRequest);
      if (!accessDecision.allowed) {
        return {
          success: false,
          error: `Access denied: ${accessDecision.reason}`,
          accessGranted: false,
          accessDecision,
          encrypted: false,
        };
      }

      // Add tenant isolation to conditions if required
      const queryConditions = { ...conditions };
      if (this.securityConfig.requireTenantIsolation) {
        queryConditions.tenant_id = tenantId;
      }

      // Perform the actual query operation
      const queryOptions: QueryOptions = {
        timeout: options?.limit ? options.limit * 1000 : undefined,
        cache: false,
      };
      const result = await this.findMany(
        queryConditions as Partial<T>,
        queryOptions
      );

      if (!result.success || !result.data) {
        return {
          success: false,
          data: [],
          error: result.error,
          accessGranted: true,
          accessDecision,
          encrypted: false,
        };
      }

      // Decrypt sensitive fields for each record
      let processedData = result.data;
      if (this.securityConfig.encryptionEnabled && this.encryptionManager) {
        processedData = (await Promise.all(
          result.data.map(
            async (record: T) =>
              await this.encryptionManager!.decryptFields(
                record as Record<string, any>,
                this.securityConfig.sensitiveFields
              )
          )
        )) as T[];
      }

      return {
        success: true,
        data: processedData,
        accessGranted: true,
        accessDecision,
        encrypted: this.securityConfig.encryptionEnabled,
      };
    } catch (error) {
      this.logger.error("Secure query operation failed:", error);
      return {
        success: false,
        error: `Query operation failed: ${(error as Error).message}`,
        accessGranted: false,
        encrypted: false,
      };
    }
  }

  /**
   * Check access control for an operation
   */
  private async checkAccess(request: AccessRequest): Promise<AccessDecision> {
    if (
      !this.securityConfig.accessControlEnabled ||
      !this.accessControlManager
    ) {
      // Access control disabled, allow by default
      return {
        allowed: true,
        reason: "Access control disabled",
        evaluatedPolicies: 0,
        executionTime: 0,
      };
    }

    return await this.accessControlManager.evaluateAccess(request);
  }

  /**
   * Get security configuration
   */
  getSecurityConfig(): SecurityConfig {
    return { ...this.securityConfig };
  }

  /**
   * Update security configuration
   */
  updateSecurityConfig(config: Partial<SecurityConfig>): void {
    this.securityConfig = { ...this.securityConfig, ...config };
    this.logger.info(`Updated security configuration for ${this.tableName}`);
  }
}
