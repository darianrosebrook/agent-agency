/**
 * Memory System Integration Tests
 *
 * Tests federated learning and tenant isolation integration including:
 * - Federated learning model aggregation across tenants
 * - Tenant data isolation and security
 * - Learning synchronization and conflict resolution
 * - Performance optimization for multi-tenant environments
 *
 * @author @darianrosebrook
 */

import { FederatedLearningEngine } from "../../src/memory/FederatedLearningEngine";
import { TenantIsolator } from "../../src/memory/TenantIsolator";
import {
  LearningEvent,
  ModelUpdate,
  PrivacyConfig,
  TenantId,
} from "../../src/types/memory";

describe("Memory System Integration", () => {
  let federatedEngine: FederatedLearningEngine;
  let tenantIsolator: TenantIsolator;

  // Test data fixtures
  const createTenant = (
    id: string,
    privacyLevel: "high" | "medium" | "low" = "medium"
  ): TenantId => ({
    id,
    name: `Tenant ${id}`,
    privacyConfig: {
      dataIsolation: privacyLevel === "high",
      aggregationAllowed: privacyLevel !== "high",
      modelSharing: privacyLevel === "low",
      auditLogging: true,
    } as PrivacyConfig,
  });

  const createModelUpdate = (
    tenantId: TenantId,
    accuracy: number = 0.85,
    sampleCount: number = 1000
  ): ModelUpdate => ({
    tenantId: tenantId.id,
    modelVersion: "1.0.0",
    weights: new Map([
      ["layer1", new Float32Array([0.1, 0.2, 0.3])],
      ["layer2", new Float32Array([0.4, 0.5, 0.6])],
    ]),
    gradients: new Map([
      ["layer1", new Float32Array([0.01, 0.02, 0.03])],
      ["layer2", new Float32Array([0.04, 0.05, 0.06])],
    ]),
    metadata: {
      accuracy,
      sampleCount,
      trainingTime: 120,
      epoch: 10,
    },
  });

  const createLearningEvent = (
    tenantId: TenantId,
    eventType: LearningEvent["type"] = "model_update",
    data: any = {}
  ): LearningEvent => ({
    id: `event-${Date.now()}`,
    tenantId: tenantId.id,
    type: eventType,
    timestamp: new Date(),
    data,
  });

  beforeEach(async () => {
    tenantIsolator = new TenantIsolator();
    federatedEngine = new FederatedLearningEngine(tenantIsolator);

    await federatedEngine.initialize();
  });

  afterEach(async () => {
    await federatedEngine.shutdown();
  });

  describe("Federated Learning Engine Integration", () => {
    it("should aggregate model updates from multiple tenants", async () => {
      const tenant1 = createTenant("tenant-1", "low");
      const tenant2 = createTenant("tenant-2", "low");
      const tenant3 = createTenant("tenant-3", "low");

      // Register tenants
      await tenantIsolator.registerTenant(tenant1);
      await tenantIsolator.registerTenant(tenant2);
      await tenantIsolator.registerTenant(tenant3);

      // Create model updates with different accuracies
      const update1 = createModelUpdate(tenant1, 0.82, 800);
      const update2 = createModelUpdate(tenant2, 0.88, 1200);
      const update3 = createModelUpdate(tenant3, 0.85, 1000);

      // Submit updates
      await federatedEngine.submitModelUpdate(update1);
      await federatedEngine.submitModelUpdate(update2);
      await federatedEngine.submitModelUpdate(update3);

      // Trigger aggregation
      const aggregatedModel = await federatedEngine.aggregateModels();

      expect(aggregatedModel).toBeDefined();
      expect(aggregatedModel.weights.size).toBeGreaterThan(0);
      expect(aggregatedModel.metadata.tenantCount).toBe(3);
      expect(aggregatedModel.metadata.averageAccuracy).toBeGreaterThan(0.8);
    });

    it("should respect tenant privacy configurations", async () => {
      const publicTenant = createTenant("public-tenant", "low");
      const privateTenant = createTenant("private-tenant", "high");

      await tenantIsolator.registerTenant(publicTenant);
      await tenantIsolator.registerTenant(privateTenant);

      const publicUpdate = createModelUpdate(publicTenant);
      const privateUpdate = createModelUpdate(privateTenant);

      // Submit updates
      await federatedEngine.submitModelUpdate(publicUpdate);
      await federatedEngine.submitModelUpdate(privateUpdate);

      // Aggregation should only include public tenant
      const aggregatedModel = await federatedEngine.aggregateModels();

      expect(aggregatedModel.metadata.tenantCount).toBe(1); // Only public tenant
      expect(aggregatedModel.contributingTenants).toContain(publicTenant.id);
      expect(aggregatedModel.contributingTenants).not.toContain(
        privateTenant.id
      );
    });

    it("should handle model synchronization across tenants", async () => {
      const tenant1 = createTenant("sync-tenant-1", "low");
      const tenant2 = createTenant("sync-tenant-2", "low");

      await tenantIsolator.registerTenant(tenant1);
      await tenantIsolator.registerTenant(tenant2);

      // Tenant 1 submits update first
      const update1 = createModelUpdate(tenant1, 0.9, 1500);
      await federatedEngine.submitModelUpdate(update1);

      // Aggregate to create global model
      const globalModel1 = await federatedEngine.aggregateModels();

      // Tenant 2 should be able to synchronize with the global model
      const syncResult = await federatedEngine.synchronizeTenant(
        tenant2.id,
        globalModel1
      );

      expect(syncResult.success).toBe(true);
      expect(syncResult.modelVersion).toBe(globalModel1.version);

      // Tenant 2 can now contribute
      const update2 = createModelUpdate(tenant2, 0.87, 1300);
      await federatedEngine.submitModelUpdate(update2);

      // New aggregation should include both tenants
      const globalModel2 = await federatedEngine.aggregateModels();
      expect(globalModel2.metadata.tenantCount).toBe(2);
    });

    it("should detect and handle conflicting model updates", async () => {
      const tenant1 = createTenant("conflict-tenant-1", "low");
      const tenant2 = createTenant("conflict-tenant-2", "low");

      await tenantIsolator.registerTenant(tenant1);
      await tenantIsolator.registerTenant(tenant2);

      // Create conflicting updates (same model version but different weights)
      const update1 = createModelUpdate(tenant1, 0.85);
      update1.weights.set("layer1", new Float32Array([0.1, 0.2, 0.3])); // Version 1.0.0

      const update2 = createModelUpdate(tenant2, 0.85);
      update2.weights.set("layer1", new Float32Array([0.3, 0.2, 0.1])); // Different weights, same version

      await federatedEngine.submitModelUpdate(update1);

      // Second update should trigger conflict detection
      const conflictResult = await federatedEngine.submitModelUpdate(update2);

      expect(conflictResult.hasConflict).toBe(true);
      expect(conflictResult.conflictType).toBe("version_conflict");

      // Aggregation should handle the conflict
      const aggregatedModel = await federatedEngine.aggregateModels();
      expect(aggregatedModel).toBeDefined();
      // Should use conflict resolution strategy (e.g., latest timestamp wins)
    });
  });

  describe("Tenant Isolation Integration", () => {
    it("should maintain strict data isolation between tenants", async () => {
      const tenant1 = createTenant("isolated-tenant-1", "high");
      const tenant2 = createTenant("isolated-tenant-2", "high");

      await tenantIsolator.registerTenant(tenant1);
      await tenantIsolator.registerTenant(tenant2);

      // Each tenant submits their own data
      const event1 = createLearningEvent(tenant1, "model_update", {
        accuracy: 0.85,
      });
      const event2 = createLearningEvent(tenant2, "model_update", {
        accuracy: 0.9,
      });

      await tenantIsolator.storeTenantEvent(tenant1.id, event1);
      await tenantIsolator.storeTenantEvent(tenant2.id, event2);

      // Verify isolation - tenant1 cannot access tenant2's data
      const tenant1Events = await tenantIsolator.getTenantEvents(tenant1.id);
      const tenant2Events = await tenantIsolator.getTenantEvents(tenant2.id);

      expect(tenant1Events.length).toBe(1);
      expect(tenant2Events.length).toBe(1);
      expect(tenant1Events[0].tenantId).toBe(tenant1.id);
      expect(tenant2Events[0].tenantId).toBe(tenant2.id);

      // Cross-tenant access should be blocked
      await expect(
        tenantIsolator.getTenantEvents(tenant2.id, {
          requesterTenantId: tenant1.id,
        })
      ).rejects.toThrow("Access denied");
    });

    it("should support configurable privacy levels", async () => {
      const openTenant = createTenant("open-tenant", "low");
      const mediumTenant = createTenant("medium-tenant", "medium");
      const secureTenant = createTenant("secure-tenant", "high");

      await tenantIsolator.registerTenant(openTenant);
      await tenantIsolator.registerTenant(mediumTenant);
      await tenantIsolator.registerTenant(secureTenant);

      // Test aggregation permissions
      const canOpenAggregate = await tenantIsolator.canParticipateInAggregation(
        openTenant.id
      );
      const canMediumAggregate =
        await tenantIsolator.canParticipateInAggregation(mediumTenant.id);
      const canSecureAggregate =
        await tenantIsolator.canParticipateInAggregation(secureTenant.id);

      expect(canOpenAggregate).toBe(true); // Low privacy allows aggregation
      expect(canMediumAggregate).toBe(true); // Medium privacy allows aggregation
      expect(canSecureAggregate).toBe(false); // High privacy blocks aggregation

      // Test model sharing permissions
      const canOpenShare = await tenantIsolator.canShareModel(openTenant.id);
      const canMediumShare = await tenantIsolator.canShareModel(
        mediumTenant.id
      );
      const canSecureShare = await tenantIsolator.canShareModel(
        secureTenant.id
      );

      expect(canOpenShare).toBe(true); // Low privacy allows sharing
      expect(canMediumShare).toBe(false); // Medium privacy blocks sharing
      expect(canSecureShare).toBe(false); // High privacy blocks sharing
    });

    it("should audit tenant operations", async () => {
      const tenant = createTenant("audited-tenant", "medium");

      await tenantIsolator.registerTenant(tenant);

      // Perform some operations
      const event = createLearningEvent(tenant, "model_update");
      await tenantIsolator.storeTenantEvent(tenant.id, event);

      const _retrievedEvents = await tenantIsolator.getTenantEvents(tenant.id);

      // Check audit trail
      const auditLog = await tenantIsolator.getTenantAuditLog(tenant.id);

      expect(auditLog.length).toBeGreaterThan(0);
      expect(auditLog.some((entry) => entry.operation === "store_event")).toBe(
        true
      );
      expect(
        auditLog.some((entry) => entry.operation === "retrieve_events")
      ).toBe(true);

      // All audit entries should be for the correct tenant
      expect(auditLog.every((entry) => entry.tenantId === tenant.id)).toBe(
        true
      );
    });

    it("should handle tenant lifecycle operations", async () => {
      const tenant = createTenant("lifecycle-tenant", "medium");

      // Register tenant
      await tenantIsolator.registerTenant(tenant);

      // Verify tenant exists
      const existsBefore = await tenantIsolator.tenantExists(tenant.id);
      expect(existsBefore).toBe(true);

      // Store some data
      const event = createLearningEvent(tenant, "model_update");
      await tenantIsolator.storeTenantEvent(tenant.id, event);

      // Verify data exists
      const eventsBefore = await tenantIsolator.getTenantEvents(tenant.id);
      expect(eventsBefore.length).toBe(1);

      // Deactivate tenant
      await tenantIsolator.deactivateTenant(tenant.id);

      // Data should still exist but be inaccessible
      const eventsAfter = await tenantIsolator.getTenantEvents(tenant.id);
      expect(eventsAfter.length).toBe(0); // Deactivated tenants return empty results

      // Reactivate tenant
      await tenantIsolator.reactivateTenant(tenant.id);

      // Data should be accessible again
      const eventsReactivated = await tenantIsolator.getTenantEvents(tenant.id);
      expect(eventsReactivated.length).toBe(1);
    });
  });

  describe("Performance and Scalability", () => {
    it("should handle high-frequency learning events", async () => {
      const tenant = createTenant("perf-tenant", "medium");
      await tenantIsolator.registerTenant(tenant);

      const eventCount = 100;
      const startTime = Date.now();

      // Submit many events rapidly
      const events = [];
      for (let i = 0; i < eventCount; i++) {
        const event = createLearningEvent(tenant, "model_update", { index: i });
        events.push(tenantIsolator.storeTenantEvent(tenant.id, event));
      }

      await Promise.all(events);

      const duration = Date.now() - startTime;

      // Should complete within reasonable time (under 2 seconds)
      expect(duration).toBeLessThan(2000);

      // All events should be stored
      const storedEvents = await tenantIsolator.getTenantEvents(tenant.id);
      expect(storedEvents.length).toBe(eventCount);
    });

    it("should scale with multiple concurrent tenants", async () => {
      const tenantCount = 10;
      const tenants: TenantId[] = [];

      // Register multiple tenants
      for (let i = 0; i < tenantCount; i++) {
        const tenant = createTenant(`scale-tenant-${i}`, "low");
        tenants.push(tenant);
        await tenantIsolator.registerTenant(tenant);
      }

      const startTime = Date.now();

      // Each tenant submits updates concurrently
      const operations = tenants.map(async (tenant) => {
        const update = createModelUpdate(
          tenant,
          0.8 + Math.random() * 0.15,
          1000
        );
        await federatedEngine.submitModelUpdate(update);

        const event = createLearningEvent(tenant, "model_update");
        await tenantIsolator.storeTenantEvent(tenant.id, event);
      });

      await Promise.all(operations);

      const duration = Date.now() - startTime;

      // Should complete within reasonable time (under 5 seconds)
      expect(duration).toBeLessThan(5000);

      // Aggregation should work across all tenants
      const aggregatedModel = await federatedEngine.aggregateModels();
      expect(aggregatedModel.metadata.tenantCount).toBe(tenantCount);

      // Each tenant should have their events
      for (const tenant of tenants) {
        const events = await tenantIsolator.getTenantEvents(tenant.id);
        expect(events.length).toBe(1);
      }
    });

    it("should maintain performance under memory pressure", async () => {
      const tenant = createTenant("memory-tenant", "medium");
      await tenantIsolator.registerTenant(tenant);

      // Simulate memory pressure by creating many large events
      const largeEvents = [];
      for (let i = 0; i < 50; i++) {
        const largeData = {
          weights: new Array(1000).fill(0).map(() => Math.random()),
          gradients: new Array(1000).fill(0).map(() => Math.random()),
          metadata: { size: "large" },
        };
        const event = createLearningEvent(tenant, "model_update", largeData);
        largeEvents.push(tenantIsolator.storeTenantEvent(tenant.id, event));
      }

      const startTime = Date.now();
      await Promise.all(largeEvents);
      const storeDuration = Date.now() - startTime;

      // Should handle large data efficiently
      expect(storeDuration).toBeLessThan(3000);

      // Retrieval should also be efficient
      const retrieveStartTime = Date.now();
      const events = await tenantIsolator.getTenantEvents(tenant.id);
      const retrieveDuration = Date.now() - retrieveStartTime;

      expect(events.length).toBe(50);
      expect(retrieveDuration).toBeLessThan(1000);
    });
  });

  describe("Federated Learning Optimization", () => {
    it("should optimize aggregation for performance", async () => {
      const tenants = Array.from({ length: 5 }, (_, i) =>
        createTenant(`opt-tenant-${i}`, "low")
      );

      // Register tenants
      await Promise.all(
        tenants.map((tenant) => tenantIsolator.registerTenant(tenant))
      );

      // Submit model updates with timing measurement
      const submitStartTime = Date.now();
      const updates = tenants.map((tenant) => createModelUpdate(tenant));
      await Promise.all(
        updates.map((update) => federatedEngine.submitModelUpdate(update))
      );
      const submitDuration = Date.now() - submitStartTime;

      // Aggregate with timing measurement
      const aggregateStartTime = Date.now();
      const aggregatedModel = await federatedEngine.aggregateModels();
      const aggregateDuration = Date.now() - aggregateStartTime;

      // Performance assertions
      expect(submitDuration).toBeLessThan(1000); // Fast submission
      expect(aggregateDuration).toBeLessThan(2000); // Fast aggregation
      expect(aggregatedModel.metadata.tenantCount).toBe(5);

      // Quality assertions
      expect(aggregatedModel.metadata.averageAccuracy).toBeGreaterThan(0.8);
      expect(aggregatedModel.weights.size).toBeGreaterThan(0);
    });

    it("should handle partial failures in aggregation", async () => {
      const goodTenant = createTenant("good-tenant", "low");
      const badTenant = createTenant("bad-tenant", "low");

      await tenantIsolator.registerTenant(goodTenant);
      await tenantIsolator.registerTenant(badTenant);

      // Submit good update
      const goodUpdate = createModelUpdate(goodTenant, 0.9, 1500);
      await federatedEngine.submitModelUpdate(goodUpdate);

      // Submit bad update (simulate corruption)
      const badUpdate = createModelUpdate(badTenant, 0.9, 1500);
      badUpdate.weights = new Map(); // Empty weights = corrupted

      await expect(
        federatedEngine.submitModelUpdate(badUpdate)
      ).rejects.toThrow("Invalid model update");

      // Aggregation should still work with good tenant only
      const aggregatedModel = await federatedEngine.aggregateModels();

      expect(aggregatedModel.metadata.tenantCount).toBe(1);
      expect(aggregatedModel.contributingTenants).toContain(goodTenant.id);
      expect(aggregatedModel.contributingTenants).not.toContain(badTenant.id);
    });

    it("should support incremental learning updates", async () => {
      const tenant = createTenant("incremental-tenant", "low");
      await tenantIsolator.registerTenant(tenant);

      // Submit initial model
      const initialUpdate = createModelUpdate(tenant, 0.75, 500);
      await federatedEngine.submitModelUpdate(initialUpdate);

      let aggregatedModel = await federatedEngine.aggregateModels();
      expect(aggregatedModel.metadata.averageAccuracy).toBe(0.75);

      // Submit improved model
      const improvedUpdate = createModelUpdate(tenant, 0.85, 1000);
      await federatedEngine.submitModelUpdate(improvedUpdate);

      aggregatedModel = await federatedEngine.aggregateModels();
      expect(aggregatedModel.metadata.averageAccuracy).toBe(0.85);

      // Should track improvement over time
      const learningHistory = await federatedEngine.getLearningHistory(
        tenant.id
      );
      expect(learningHistory.length).toBe(2);
      expect(learningHistory[0].accuracy).toBeLessThan(
        learningHistory[1].accuracy
      );
    });
  });

  describe("Security and Compliance", () => {
    it("should enforce data sovereignty for high-privacy tenants", async () => {
      const sovereignTenant = createTenant("sovereign-tenant", "high");

      await tenantIsolator.registerTenant(sovereignTenant);

      // High-privacy tenant should not participate in federation
      const canAggregate = await tenantIsolator.canParticipateInAggregation(
        sovereignTenant.id
      );
      expect(canAggregate).toBe(false);

      // Data should be completely isolated
      const event = createLearningEvent(sovereignTenant, "model_update");
      await tenantIsolator.storeTenantEvent(sovereignTenant.id, event);

      // Other tenants should not be able to access this data
      const otherTenant = createTenant("other-tenant", "low");
      await tenantIsolator.registerTenant(otherTenant);

      await expect(
        tenantIsolator.getTenantEvents(sovereignTenant.id, {
          requesterTenantId: otherTenant.id,
        })
      ).rejects.toThrow("Access denied");
    });

    it("should maintain audit trails for all operations", async () => {
      const auditedTenant = createTenant("audited-tenant", "medium");

      await tenantIsolator.registerTenant(auditedTenant);

      // Perform various operations
      const event1 = createLearningEvent(auditedTenant, "model_update");
      const event2 = createLearningEvent(auditedTenant, "model_validation");

      await tenantIsolator.storeTenantEvent(auditedTenant.id, event1);
      await tenantIsolator.storeTenantEvent(auditedTenant.id, event2);

      const _events = await tenantIsolator.getTenantEvents(auditedTenant.id);

      // Audit log should capture all operations
      const auditLog = await tenantIsolator.getTenantAuditLog(auditedTenant.id);

      expect(auditLog.length).toBeGreaterThan(2); // At least registration + 2 stores + 1 retrieval
      expect(auditLog.every((entry) => entry.timestamp instanceof Date)).toBe(
        true
      );
      expect(
        auditLog.every((entry) => entry.tenantId === auditedTenant.id)
      ).toBe(true);
    });

    it("should handle tenant data cleanup securely", async () => {
      const cleanupTenant = createTenant("cleanup-tenant", "medium");

      await tenantIsolator.registerTenant(cleanupTenant);

      // Store data
      const event = createLearningEvent(cleanupTenant, "model_update");
      await tenantIsolator.storeTenantEvent(cleanupTenant.id, event);

      const update = createModelUpdate(cleanupTenant);
      await federatedEngine.submitModelUpdate(update);

      // Verify data exists
      const eventsBefore = await tenantIsolator.getTenantEvents(
        cleanupTenant.id
      );
      expect(eventsBefore.length).toBe(1);

      // Completely remove tenant
      await tenantIsolator.deleteTenant(cleanupTenant.id);

      // Tenant should no longer exist
      const exists = await tenantIsolator.tenantExists(cleanupTenant.id);
      expect(exists).toBe(false);

      // All data should be securely deleted
      await expect(
        tenantIsolator.getTenantEvents(cleanupTenant.id)
      ).rejects.toThrow("Tenant not found");

      // Federated engine should no longer have tenant's contributions
      const aggregatedModel = await federatedEngine.aggregateModels();
      expect(aggregatedModel.contributingTenants).not.toContain(
        cleanupTenant.id
      );
    });
  });
});

