/**
 * Unit tests for MultiTenantMemoryManager
 *
 * @author @darianrosebrook
 */

import { MultiTenantMemoryManager } from "../../../src/memory/MultiTenantMemoryManager";
import { Logger } from "../../../src/utils/Logger";

describe("MultiTenantMemoryManager", () => {
  let memoryManager: MultiTenantMemoryManager;
  let mockLogger: Logger;

  const testConfig = {
    tenantIsolation: {
      enabled: true,
      defaultIsolationLevel: "shared" as const,
      auditLogging: true,
      maxTenants: 10,
    },
    contextOffloading: {
      enabled: true,
      maxContextSize: 10000,
      compressionThreshold: 0.8,
      relevanceThreshold: 0.7,
      embeddingDimensions: 384,
    },
    federatedLearning: {
      enabled: false,
      privacyLevel: "basic" as const,
      aggregationFrequency: 3600000,
      minParticipants: 3,
    },
    performance: {
      cacheEnabled: true,
      cacheSize: 1000,
      batchProcessing: false,
      asyncOperations: true,
    },
  };

  beforeEach(() => {
    mockLogger = {
      info: jest.fn(),
      warn: jest.fn(),
      error: jest.fn(),
      debug: jest.fn(),
    } as any;

    memoryManager = new MultiTenantMemoryManager(testConfig, mockLogger);
  });

  describe("tenant registration", () => {
    it("should register a tenant successfully", async () => {
      const tenantConfig = {
        tenantId: "test-tenant",
        projectId: "test-project",
        isolationLevel: "shared" as const,
        accessPolicies: [
          {
            resourceType: "memory",
            accessLevel: "write",
            allowedTenants: ["test-tenant"],
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

      const result = await memoryManager.registerTenant(tenantConfig);

      expect(result.success).toBe(true);
      expect(result.tenantId).toBe("test-tenant");
      expect(result.operationId).toContain("register_tenant");
      expect(result.performance.duration).toBeGreaterThanOrEqual(0);
    });

    it("should reject invalid tenant configuration", async () => {
      const invalidConfig = {
        tenantId: "",
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

      const result = await memoryManager.registerTenant(invalidConfig);

      expect(result.success).toBe(false);
      expect(result.error).toContain("Tenant ID and Project ID are required");
    });
  });

  describe("experience storage", () => {
    beforeEach(async () => {
      const tenantConfig = {
        tenantId: "storage-test-tenant",
        projectId: "test-project",
        isolationLevel: "shared" as const,
        accessPolicies: [
          {
            resourceType: "memory" as const,
            accessLevel: "write" as const,
            allowedTenants: ["storage-test-tenant"],
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

      await memoryManager.registerTenant(tenantConfig);
    });

    it("should store experience successfully", async () => {
      const experience = {
        memoryId: "test-experience-1",
        relevanceScore: 0.85,
        contextMatch: {
          similarityScore: 0.9,
          keywordMatches: ["test", "experience"],
          semanticMatches: ["learning", "memory"],
          temporalAlignment: 0.8,
        },
        reasoningPath: {
          steps: [],
          confidence: 0.8,
          depth: 2,
        },
        temporalRelevance: {
          recencyScore: 0.9,
          frequencyScore: 0.7,
          trendAlignment: 0.8,
          decayFactor: 0.1,
        },
        content: {
          taskType: "learning",
          outcome: "success",
          lessons: ["test lesson"],
        },
      };

      const result = await memoryManager.storeExperience(
        "storage-test-tenant",
        experience
      );

      expect(result.success).toBe(true);
      expect(result.data).toBeDefined();
      expect(result.tenantId).toBe("storage-test-tenant");
      expect(result.performance.duration).toBeGreaterThanOrEqual(0);
    });

    it("should reject storage for non-existent tenant", async () => {
      const experience = {
        memoryId: "test-experience-2",
        relevanceScore: 0.8,
        contextMatch: {
          similarityScore: 0.8,
          keywordMatches: ["test"],
          semanticMatches: ["test"],
          temporalAlignment: 0.7,
        },
        content: {},
      };

      const result = await memoryManager.storeExperience(
        "non-existent-tenant",
        experience
      );

      expect(result.success).toBe(false);
      expect(result.error).toContain("Tenant not found");
    });
  });

  describe("contextual memory retrieval", () => {
    beforeEach(async () => {
      const tenantConfig = {
        tenantId: "retrieval-test-tenant",
        projectId: "test-project",
        isolationLevel: "shared" as const,
        accessPolicies: [
          {
            resourceType: "memory",
            accessLevel: "read",
            allowedTenants: ["retrieval-test-tenant"],
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

      await memoryManager.registerTenant(tenantConfig);
    });

    it("should retrieve memories successfully", async () => {
      const queryContext = {
        taskId: "test-query",
        agentId: "retrieval-test-tenant",
        type: "memory_retrieval",
        description: "Test memory retrieval",
        requirements: ["test"],
        constraints: {},
        metadata: {},
      };

      const result = await memoryManager.getContextualMemories(
        "retrieval-test-tenant",
        queryContext
      );

      expect(result.success).toBe(true);
      expect(Array.isArray(result.data)).toBe(true);
      expect(result.tenantId).toBe("retrieval-test-tenant");
      expect(result.performance.duration).toBeGreaterThanOrEqual(0);
    });

    it("should reject retrieval for non-existent tenant", async () => {
      const queryContext = {
        taskId: "test-query",
        agentId: "non-existent",
        type: "memory_retrieval",
        description: "Test query",
        requirements: [],
        constraints: {},
        metadata: {},
      };

      const result = await memoryManager.getContextualMemories(
        "non-existent-tenant",
        queryContext
      );

      expect(result.success).toBe(false);
      expect(result.error).toContain("Tenant not found");
    });
  });

  describe("context offloading", () => {
    beforeEach(async () => {
      const tenantConfig = {
        tenantId: "offload-test-tenant",
        projectId: "test-project",
        isolationLevel: "shared" as const,
        accessPolicies: [
          {
            resourceType: "memory",
            accessLevel: "write",
            allowedTenants: ["offload-test-tenant"],
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

      await memoryManager.registerTenant(tenantConfig);
    });

    it("should offload context successfully", async () => {
      const context = {
        taskId: "test-context-offload",
        agentId: "offload-test-tenant",
        type: "complex_task",
        description:
          "A very long description that should trigger context offloading due to its length and complexity. This description contains multiple sentences and concepts that would benefit from summarization and compression.",
        requirements: [
          "requirement1",
          "requirement2",
          "requirement3",
          "requirement4",
          "requirement5",
        ],
        constraints: {
          timeLimit: 300000,
          memoryLimit: 1000000,
          priority: "high",
        },
        metadata: {
          complexity: 0.9,
          importance: "critical",
        },
      };

      const result = await memoryManager.offloadContext(
        "offload-test-tenant",
        context
      );

      expect(result.success).toBe(true);
      expect(result.data).toBeDefined();
      expect(result.data?.tenantId).toBe("offload-test-tenant");
      expect(result.performance.offloaded).toBe(true);
    });
  });

  describe("federated insights", () => {
    it("should return empty insights when federated learning is disabled", async () => {
      const context = {
        taskId: "federated-test",
        agentId: "test-agent",
        type: "federated_query",
        description: "Test federated insights",
        requirements: [],
        constraints: {},
        metadata: {},
      };

      const result = await memoryManager.getFederatedInsights(
        "test-tenant",
        context
      );

      expect(result.insights).toEqual([]);
      expect(result.confidence).toBe(0);
      expect(result.sourceTenants).toEqual([]);
      expect(result.privacyPreserved).toBe(true);
    });
  });

  describe("system health", () => {
    it("should return system health metrics", async () => {
      const health = await memoryManager.getSystemHealth();

      expect(typeof health.tenants).toBe("number");
      expect(typeof health.activeOperations).toBe("number");
      expect(typeof health.cacheSize).toBe("number");
      expect(typeof health.offloadedContexts).toBe("number");
      expect(typeof health.federatedParticipants).toBe("number");
    });
  });

  describe("maintenance", () => {
    it("should perform maintenance without errors", async () => {
      await expect(memoryManager.performMaintenance()).resolves.not.toThrow();
    });
  });
});
