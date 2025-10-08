/**
 * Unit tests for TenantIsolator
 *
 * @author @darianrosebrook
 */

import { TenantIsolator } from "../../../src/memory/TenantIsolator";
import type { TenantConfig } from "../../../src/types";
import { Logger } from "../../../src/utils/Logger";

describe("TenantIsolator", () => {
  let tenantIsolator: TenantIsolator;
  let mockLogger: Logger;

  beforeEach(() => {
    mockLogger = {
      info: jest.fn(),
      warn: jest.fn(),
      error: jest.fn(),
      debug: jest.fn(),
    } as any;

    tenantIsolator = new TenantIsolator(mockLogger);
  });

  describe("registerTenant", () => {
    it("should register a tenant successfully", async () => {
      const config: TenantConfig = {
        tenantId: "test-tenant",
        projectId: "test-project",
        isolationLevel: "strict" as const,
        accessPolicies: [],
        sharingRules: [],
        dataRetention: {
          defaultRetentionDays: 30,
          archivalPolicy: "delete" as const,
          complianceRequirements: [],
          backupFrequency: "weekly" as const,
        },
        encryptionEnabled: false,
        auditLogging: true,
      };

      await tenantIsolator.registerTenant(config);

      const context = tenantIsolator.getTenantContext("test-tenant");
      expect(context).toBeDefined();
      expect(context?.tenantId).toBe("test-tenant");
      expect(context?.projectId).toBe("test-project");
      expect(context?.isolationLevel).toBe("strict");
    });

    it("should reject duplicate tenant registration", async () => {
      const config: TenantConfig = {
        tenantId: "duplicate-tenant",
        projectId: "test-project",
        isolationLevel: "shared" as const,
        accessPolicies: [],
        sharingRules: [],
        dataRetention: {
          defaultRetentionDays: 30,
          archivalPolicy: "delete" as const,
          complianceRequirements: [],
          backupFrequency: "weekly" as const,
        },
        encryptionEnabled: false,
        auditLogging: true,
      };

      await tenantIsolator.registerTenant(config);

      await expect(tenantIsolator.registerTenant(config)).rejects.toThrow(
        "Tenant duplicate-tenant already exists"
      );
    });
  });

  describe("validateTenantAccess", () => {
    beforeEach(async () => {
      const config: TenantConfig = {
        tenantId: "access-test-tenant",
        projectId: "test-project",
        isolationLevel: "shared" as const,
        accessPolicies: [
          {
            resourceType: "memory" as const,
            accessLevel: "read" as const,
            allowedTenants: ["access-test-tenant"],
            restrictions: [],
            conditions: [],
          },
        ],
        sharingRules: [],
        dataRetention: {
          defaultRetentionDays: 30,
          archivalPolicy: "delete" as const,
          complianceRequirements: [],
          backupFrequency: "weekly" as const,
        },
        encryptionEnabled: false,
        auditLogging: true,
      };

      await tenantIsolator.registerTenant(config);
    });

    it("should allow valid tenant access", async () => {
      const result = await tenantIsolator.validateTenantAccess(
        "access-test-tenant",
        "read"
      );

      expect(result.allowed).toBe(true);
      expect(result.data).toBe(true);
    });

    it("should deny access for non-existent tenant", async () => {
      const result = await tenantIsolator.validateTenantAccess(
        "non-existent-tenant",
        "read"
      );

      expect(result.allowed).toBe(false);
      expect(result.reason).toContain("Tenant not found");
    });

    it("should allow access for authorized operation", async () => {
      const result = await tenantIsolator.validateTenantAccess(
        "access-test-tenant",
        "write"
      );

      expect(result.allowed).toBe(true);
      expect(result.data).toBe(true);
    });

    it("should deny access for federated operation in shared isolation", async () => {
      const result = await tenantIsolator.validateTenantAccess(
        "access-test-tenant",
        "federate"
      );

      expect(result.allowed).toBe(false);
      expect(result.reason).toContain("Operation not permitted");
    });
  });

  describe("getTenantData", () => {
    beforeEach(async () => {
      const config = {
        tenantId: "data-test-tenant",
        projectId: "test-project",
        isolationLevel: "strict" as const,
        accessPolicies: [],
        sharingRules: [],
        dataRetention: {
          defaultRetentionDays: 30,
          archivalPolicy: "delete" as const,
          complianceRequirements: [],
          backupFrequency: "weekly" as const,
        },
        encryptionEnabled: false,
        auditLogging: true,
      };

      await tenantIsolator.registerTenant(config);
    });

    it("should successfully get tenant data", async () => {
      const mockDataFetcher = jest.fn().mockResolvedValue("test data");

      const result = await tenantIsolator.getTenantData(
        "data-test-tenant",
        mockDataFetcher,
        "read"
      );

      expect(result.allowed).toBe(true);
      expect(result.data).toBe("test data");
      expect(mockDataFetcher).toHaveBeenCalledTimes(1);
    });

    it("should handle data fetch errors", async () => {
      const mockDataFetcher = jest
        .fn()
        .mockRejectedValue(new Error("Database connection failed"));

      const result = await tenantIsolator.getTenantData(
        "data-test-tenant",
        mockDataFetcher,
        "read"
      );

      expect(result.allowed).toBe(false);
      expect(result.reason).toContain("Data fetch failed");
      expect(result.data).toBeNull();
    });
  });

  describe("canShareWithTenant", () => {
    beforeEach(async () => {
      // Register source tenant
      await tenantIsolator.registerTenant({
        tenantId: "source-tenant",
        projectId: "project-a",
        isolationLevel: "shared" as const,
        accessPolicies: [],
        sharingRules: [
          {
            targetTenant: "target-tenant",
            resourceTypes: ["memory"],
            conditions: [],
            anonymizationLevel: "none" as const,
          },
        ],
        dataRetention: {
          defaultRetentionDays: 30,
          archivalPolicy: "delete" as const,
          complianceRequirements: [],
          backupFrequency: "weekly" as const,
        },
        encryptionEnabled: false,
        auditLogging: true,
      });

      // Register target tenant
      await tenantIsolator.registerTenant({
        tenantId: "target-tenant",
        projectId: "project-b",
        isolationLevel: "shared" as const,
        accessPolicies: [],
        sharingRules: [],
        dataRetention: {
          defaultRetentionDays: 30,
          archivalPolicy: "delete" as const,
          complianceRequirements: [],
          backupFrequency: "weekly" as const,
        },
        encryptionEnabled: false,
        auditLogging: true,
      });
    });

    it("should allow sharing when rule exists", async () => {
      const result = await tenantIsolator.canShareWithTenant(
        "source-tenant",
        "target-tenant",
        "memory"
      );

      expect(result.allowed).toBe(true);
      expect(result.data).toBe(true);
    });

    it("should deny sharing when no rule exists", async () => {
      const result = await tenantIsolator.canShareWithTenant(
        "target-tenant",
        "source-tenant",
        "memory"
      );

      expect(result.allowed).toBe(false);
      expect(result.reason).toContain("No sharing rule defined");
    });
  });

  describe("audit logging", () => {
    beforeEach(async () => {
      const config: TenantConfig = {
        tenantId: "audit-test-tenant",
        projectId: "test-project",
        isolationLevel: "strict" as const,
        accessPolicies: [],
        sharingRules: [],
        dataRetention: {
          defaultRetentionDays: 30,
          archivalPolicy: "delete" as const,
          complianceRequirements: [],
          backupFrequency: "weekly" as const,
        },
        encryptionEnabled: false,
        auditLogging: true,
      };

      await tenantIsolator.registerTenant(config);
    });

    it("should log successful operations", async () => {
      await tenantIsolator.validateTenantAccess("audit-test-tenant", "read");

      const auditLogs = tenantIsolator.getAuditLogs("audit-test-tenant");
      const readLogs = auditLogs.filter((log) => log.operation === "read");
      expect(readLogs.length).toBeGreaterThan(0);

      const readLog = readLogs[0];
      expect(readLog.tenantId).toBe("audit-test-tenant");
      expect(readLog.operation).toBe("read");
      expect(readLog.success).toBe(true);
    });

    it("should log failed operations", async () => {
      // Create a separate tenant isolator instance for this test
      const testIsolator = new TenantIsolator(mockLogger);
      await testIsolator.registerTenant({
        tenantId: "strict-test-tenant",
        projectId: "test-project",
        isolationLevel: "strict" as const,
        accessPolicies: [],
        sharingRules: [],
        dataRetention: {
          defaultRetentionDays: 30,
          archivalPolicy: "delete" as const,
          complianceRequirements: [],
          backupFrequency: "weekly" as const,
        },
        encryptionEnabled: false,
        auditLogging: true,
      });

      await testIsolator.validateTenantAccess("strict-test-tenant", "share");

      const auditLogs = testIsolator.getAuditLogs("strict-test-tenant");
      const failedLogs = auditLogs.filter((log) => !log.success);
      expect(failedLogs.length).toBeGreaterThan(0);

      const failedLog = failedLogs[0];
      expect(failedLog.tenantId).toBe("strict-test-tenant");
      expect(failedLog.operation).toBe("share");
      expect(failedLog.success).toBe(false);
    });
  });
});
