/**
 * Unit tests for FederatedLearningEngine
 *
 * Tests federated learning functionality with privacy preservation.
 *
 * @author @darianrosebrook
 */

import { FederatedLearningEngine } from "../../../src/memory/FederatedLearningEngine";
import { TenantIsolator } from "../../../src/memory/TenantIsolator";

describe("FederatedLearningEngine", () => {
  let engine: FederatedLearningEngine;
  let mockTenantIsolator: jest.Mocked<TenantIsolator>;

  beforeEach(() => {
    // Mock TenantIsolator
    mockTenantIsolator = {
      validateTenantAccess: jest.fn(),
    } as any;

    engine = new FederatedLearningEngine(
      {
        enabled: true,
        privacyLevel: "basic",
        aggregationFrequency: 60000, // 1 minute
        minParticipants: 2,
        maxParticipants: 10,
        privacyBudget: 1.0,
        aggregationMethod: "weighted",
        learningRate: 0.01,
        convergenceThreshold: 0.95,
      },
      mockTenantIsolator
    );
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  describe("participant registration", () => {
    it("should register tenant with federated isolation level", async () => {
      mockTenantIsolator.validateTenantAccess.mockResolvedValue({
        allowed: true,
        reason: "",
      });

      const result = await engine.registerParticipant("tenant-1", {
        isolationLevel: "federated",
        maxMemorySize: 1000,
        allowFederation: true,
      });

      expect(result).toBe(true);
    });

    it("should reject tenant without federated isolation level", async () => {
      const result = await engine.registerParticipant("tenant-1", {
        isolationLevel: "strict",
        maxMemorySize: 1000,
        allowFederation: false,
      });

      expect(result).toBe(false);
    });
  });

  describe("system health", () => {
    it("should return system health metrics", async () => {
      const health = await engine.getSystemHealth();

      expect(health).toHaveProperty("activeSessions");
      expect(health).toHaveProperty("registeredParticipants");
      expect(health).toHaveProperty("pendingAggregations");
      expect(health).toHaveProperty("totalInsightsShared");
      expect(health).toHaveProperty("averagePrivacyScore");
    });
  });

  describe("maintenance", () => {
    it("should perform maintenance without errors", async () => {
      await expect(engine.performMaintenance()).resolves.not.toThrow();
    });
  });
});
