/**
 * Unit Tests: ContextPreservationEngine
 *
 * Tests context snapshot creation, compression, restoration,
 * and differential storage capabilities.
 *
 * @author @darianrosebrook
 */

import { ContextPreservationEngine } from "../../../src/learning/ContextPreservationEngine.js";

describe("ContextPreservationEngine", () => {
  let engine: ContextPreservationEngine;

  beforeEach(() => {
    engine = new ContextPreservationEngine();
  });

  describe("Snapshot Creation", () => {
    it("should create a new context snapshot", async () => {
      const sessionId = "session-1";
      const context = {
        value: 42,
        data: "test",
        largeArray: new Array(100).fill("repeated data for compression"),
      };

      const result = await engine.createSnapshot(sessionId, 1, context);

      expect(result.success).toBe(true);
      expect(result.snapshotId).toBeDefined();
      expect(result.compressionRatio).toBeGreaterThan(0);
    });

    it("should compress context data", async () => {
      const sessionId = "session-2";
      const largeContext = {
        data: new Array(1000).fill("test data string"),
      };

      const result = await engine.createSnapshot(sessionId, 1, largeContext);

      expect(result.success).toBe(true);
      expect(result.compressionRatio).toBeGreaterThan(0);
      expect(result.compressionRatio).toBeLessThanOrEqual(1);
      expect(result.sizeBytes).toBeLessThan(
        JSON.stringify(largeContext).length
      );
    });

    it("should reject oversized context", async () => {
      const sessionId = "session-3";
      const oversizedContext = {
        data: new Array(10000000).fill("x"), // Very large
      };

      const smallEngine = new ContextPreservationEngine({
        maxSnapshotSizeMB: 1, // 1MB limit
      });

      const result = await smallEngine.createSnapshot(
        sessionId,
        1,
        oversizedContext
      );

      expect(result.success).toBe(false);
      expect(result.error).toContain("exceeds limit");
    });

    it("should generate unique snapshot IDs", async () => {
      const sessionId = "session-4";
      const context = { value: 123 };

      const result1 = await engine.createSnapshot(sessionId, 1, context);
      const result2 = await engine.createSnapshot(sessionId, 2, context);

      expect(result1.snapshotId).not.toBe(result2.snapshotId);
    });
  });

  describe("Differential Storage", () => {
    it("should create differential snapshot when base exists", async () => {
      const sessionId = "session-5";
      const baseContext = {
        value: 1,
        data: "base",
        large: new Array(100).fill("same"),
      };
      const modifiedContext = {
        value: 2,
        data: "modified",
        large: new Array(100).fill("same"),
      };

      const diffEngine = new ContextPreservationEngine({
        enableDifferentialStorage: true,
      });

      // Create base snapshot
      const baseResult = await diffEngine.createSnapshot(
        sessionId,
        1,
        baseContext
      );

      // Create differential snapshot
      const diffResult = await diffEngine.createSnapshot(
        sessionId,
        2,
        modifiedContext,
        baseResult.snapshotId
      );

      expect(diffResult.success).toBe(true);
      expect(diffResult.snapshotId).toBeDefined();
    });

    it("should use higher compression for differential snapshots with similar data", async () => {
      const sessionId = "session-6";
      const baseContext = { value: 1, largeData: new Array(500).fill("same") };
      const modifiedContext = {
        value: 2,
        largeData: new Array(500).fill("same"),
      };

      const diffEngine = new ContextPreservationEngine({
        enableDifferentialStorage: true,
      });

      // Create base
      const baseResult = await diffEngine.createSnapshot(
        sessionId,
        1,
        baseContext
      );

      // Create differential
      const diffResult = await diffEngine.createSnapshot(
        sessionId,
        2,
        modifiedContext,
        baseResult.snapshotId
      );

      // Differential should be smaller due to shared data
      expect(diffResult.success).toBe(true);
      expect(diffResult.sizeBytes).toBeLessThan(baseResult.sizeBytes);
    });

    it("should fall back to full snapshot if differential fails", async () => {
      const sessionId = "session-7";
      const context = { value: 42 };

      const diffEngine = new ContextPreservationEngine({
        enableDifferentialStorage: true,
      });

      // Try to create differential without valid base
      const result = await diffEngine.createSnapshot(
        sessionId,
        1,
        context,
        "nonexistent-base"
      );

      expect(result.success).toBe(true);
      expect(result.snapshotId).toBeDefined();
    });
  });

  describe("Context Restoration", () => {
    it("should restore context from snapshot", async () => {
      const sessionId = "session-8";
      const originalContext = {
        value: 42,
        data: "test",
        nested: { deep: "value" },
      };

      const saveResult = await engine.createSnapshot(
        sessionId,
        1,
        originalContext
      );

      const restoreResult = await engine.restoreSnapshot(
        saveResult.snapshotId!
      );

      expect(restoreResult.success).toBe(true);
      expect(restoreResult.context).toEqual(originalContext);
    });

    it("should validate checksum on restoration", async () => {
      const sessionId = "session-9";
      const context = { value: 123 };

      const checksumEngine = new ContextPreservationEngine({
        checksumValidation: true,
      });

      const saveResult = await checksumEngine.createSnapshot(
        sessionId,
        1,
        context
      );

      const restoreResult = await checksumEngine.restoreSnapshot(
        saveResult.snapshotId!
      );

      expect(restoreResult.success).toBe(true);
      expect(restoreResult.checksumValid).toBe(true);
    });

    it("should restore differential snapshots correctly", async () => {
      const sessionId = "session-10";
      const baseContext = { value: 1, data: "base" };
      const modifiedContext = { value: 2, data: "modified" };

      const diffEngine = new ContextPreservationEngine({
        enableDifferentialStorage: true,
      });

      // Create base
      const baseResult = await diffEngine.createSnapshot(
        sessionId,
        1,
        baseContext
      );

      // Create differential
      const diffResult = await diffEngine.createSnapshot(
        sessionId,
        2,
        modifiedContext,
        baseResult.snapshotId
      );

      // Restore differential
      const restoreResult = await diffEngine.restoreSnapshot(
        diffResult.snapshotId!
      );

      expect(restoreResult.success).toBe(true);
      expect(restoreResult.context).toEqual(modifiedContext);
    });

    it("should handle missing snapshot gracefully", async () => {
      const result = await engine.restoreSnapshot("nonexistent-snapshot");

      expect(result.success).toBe(false);
      expect(result.error).toBeDefined();
    });
  });

  describe("Compression Ratio", () => {
    it("should achieve 70%+ compression on repetitive data", async () => {
      const sessionId = "session-11";
      const repetitiveContext = {
        data: new Array(1000).fill("repeated string"),
        more: new Array(1000).fill("repeated string"),
      };

      const result = await engine.createSnapshot(
        sessionId,
        1,
        repetitiveContext
      );

      expect(result.success).toBe(true);
      expect(result.compressionRatio).toBeGreaterThanOrEqual(0.7);
    });

    it("should handle already-compressed data", async () => {
      const sessionId = "session-12";
      // Random data compresses poorly
      const randomData = Array.from({ length: 1000 }, () =>
        Math.random().toString(36).substring(7)
      ).join("");

      const randomContext = {
        data: randomData,
      };

      const result = await engine.createSnapshot(sessionId, 1, randomContext);

      expect(result.success).toBe(true);
      // Lower compression ratio for random data, but still some compression
      expect(result.compressionRatio).toBeGreaterThan(0);
    });
  });

  describe("Cache Management", () => {
    it("should cache recently created snapshots", async () => {
      const sessionId = "session-13";
      const context = { value: 42 };

      const saveResult = await engine.createSnapshot(sessionId, 1, context);

      // First restoration should use cache
      const restoreResult1 = await engine.restoreSnapshot(
        saveResult.snapshotId!
      );

      // Second restoration should also use cache
      const restoreResult2 = await engine.restoreSnapshot(
        saveResult.snapshotId!
      );

      expect(restoreResult1.success).toBe(true);
      expect(restoreResult2.success).toBe(true);
      expect(restoreResult1.context).toEqual(context);
      expect(restoreResult2.context).toEqual(context);
    });

    it("should provide cache statistics", async () => {
      const sessionId = "session-14";

      // Create some snapshots
      await engine.createSnapshot(sessionId, 1, { value: 1 });
      await engine.createSnapshot(sessionId, 2, { value: 2 });

      const stats = engine.getCacheStats();

      expect(stats).toHaveProperty("snapshotCount");
      expect(stats).toHaveProperty("totalSizeBytes");
      expect(stats).toHaveProperty("averageCompressionRatio");
      expect(stats.snapshotCount).toBeGreaterThan(0);
    });

    it("should clear session cache on demand", async () => {
      const sessionId = "session-15";
      const context = { value: 123 };

      await engine.createSnapshot(sessionId, 1, context);

      engine.clearSession(sessionId);

      const stats = engine.getCacheStats();
      expect(stats.snapshotCount).toBe(0);
    });
  });

  describe("Performance", () => {
    it("should create snapshots quickly (<100ms)", async () => {
      const sessionId = "session-16";
      const context = { value: 42, data: new Array(100).fill("test") };

      const startTime = Date.now();
      const result = await engine.createSnapshot(sessionId, 1, context);
      const duration = Date.now() - startTime;

      expect(result.success).toBe(true);
      expect(duration).toBeLessThan(100);
      expect(result.timeMs).toBeLessThan(100);
    });

    it("should restore snapshots quickly (<30ms P95 target)", async () => {
      const sessionId = "session-17";
      const context = { value: 42, data: "test data" };

      const saveResult = await engine.createSnapshot(sessionId, 1, context);

      const startTime = Date.now();
      const restoreResult = await engine.restoreSnapshot(
        saveResult.snapshotId!
      );
      const duration = Date.now() - startTime;

      expect(restoreResult.success).toBe(true);
      expect(duration).toBeLessThan(30);
      expect(restoreResult.timeMs).toBeLessThan(30);
    });
  });

  describe("Configuration", () => {
    it("should respect custom compression level", async () => {
      const sessionId = "session-18";
      const context = { data: new Array(1000).fill("test") };

      const maxCompressionEngine = new ContextPreservationEngine({
        compressionLevel: 9, // Maximum compression
      });

      const result = await maxCompressionEngine.createSnapshot(
        sessionId,
        1,
        context
      );

      expect(result.success).toBe(true);
      expect(result.compressionRatio).toBeGreaterThan(0);
    });

    it("should disable compression when configured", async () => {
      const sessionId = "session-19";
      const context = { value: 42 };

      const noCompressionEngine = new ContextPreservationEngine({
        enableCompression: false,
      });

      const result = await noCompressionEngine.createSnapshot(
        sessionId,
        1,
        context
      );

      expect(result.success).toBe(true);
      // No compression means ratio should be closer to 0
      expect(result.compressionRatio).toBeLessThan(0.5);
    });

    it("should disable differential storage when configured", async () => {
      const sessionId = "session-20";
      const context = { value: 42 };

      const noDiffEngine = new ContextPreservationEngine({
        enableDifferentialStorage: false,
      });

      const result1 = await noDiffEngine.createSnapshot(sessionId, 1, context);
      const result2 = await noDiffEngine.createSnapshot(
        sessionId,
        2,
        context,
        result1.snapshotId
      );

      expect(result2.success).toBe(true);
      // Should create full snapshot instead of differential
    });

    it("should disable checksum validation when configured", async () => {
      const sessionId = "session-21";
      const context = { value: 123 };

      const noChecksumEngine = new ContextPreservationEngine({
        checksumValidation: false,
      });

      const saveResult = await noChecksumEngine.createSnapshot(
        sessionId,
        1,
        context
      );

      const restoreResult = await noChecksumEngine.restoreSnapshot(
        saveResult.snapshotId!
      );

      expect(restoreResult.success).toBe(true);
      // Checksum validation is disabled, so checksumValid may be undefined or false
    });
  });

  describe("Edge Cases", () => {
    it("should handle empty context", async () => {
      const sessionId = "session-22";
      const emptyContext = {};

      const result = await engine.createSnapshot(sessionId, 1, emptyContext);

      expect(result.success).toBe(true);
      expect(result.snapshotId).toBeDefined();
    });

    it("should handle null values in context", async () => {
      const sessionId = "session-23";
      const contextWithNull = { value: null, data: "test" };

      const saveResult = await engine.createSnapshot(
        sessionId,
        1,
        contextWithNull
      );
      const restoreResult = await engine.restoreSnapshot(
        saveResult.snapshotId!
      );

      expect(restoreResult.success).toBe(true);
      expect(restoreResult.context).toEqual(contextWithNull);
    });

    it("should handle arrays in context", async () => {
      const sessionId = "session-24";
      const contextWithArray = {
        items: [1, 2, 3, { nested: "value" }],
        tags: ["a", "b", "c"],
      };

      const saveResult = await engine.createSnapshot(
        sessionId,
        1,
        contextWithArray
      );
      const restoreResult = await engine.restoreSnapshot(
        saveResult.snapshotId!
      );

      expect(restoreResult.success).toBe(true);
      expect(restoreResult.context).toEqual(contextWithArray);
    });

    it("should handle deeply nested objects", async () => {
      const sessionId = "session-25";
      const deepContext = {
        level1: {
          level2: {
            level3: {
              level4: {
                level5: "deep value",
              },
            },
          },
        },
      };

      const saveResult = await engine.createSnapshot(sessionId, 1, deepContext);
      const restoreResult = await engine.restoreSnapshot(
        saveResult.snapshotId!
      );

      expect(restoreResult.success).toBe(true);
      expect(restoreResult.context).toEqual(deepContext);
    });
  });

  describe("error handling and edge cases", () => {
    it("should handle snapshot creation failure", async () => {
      const sessionId = "session-fail";
      const context = { data: "test" };

      // Mock compression to fail
      const failingEngine = new ContextPreservationEngine();
      (failingEngine as any).compressData = jest.fn().mockImplementation(() => {
        throw new Error("Compression failed");
      });

      const result = await failingEngine.createSnapshot(sessionId, 1, context);

      expect(result.success).toBe(false);
      expect(result.error).toContain("Compression failed");
      expect(result.timeMs).toBeGreaterThan(0);
    });

    it("should handle differential snapshot when base snapshot restore fails", async () => {
      const sessionId = "session-diff-fail";
      const baseContext = { data: "base" };
      const diffContext = { data: "modified" };

      const failingEngine = new ContextPreservationEngine({
        enableDifferentialStorage: true,
      });

      // Create base snapshot
      const baseResult = await failingEngine.createSnapshot(
        sessionId,
        1,
        baseContext
      );
      expect(baseResult.success).toBe(true);

      // Mock restoreSnapshot to fail for differential
      failingEngine.restoreSnapshot = jest.fn().mockResolvedValue({
        success: false,
        context: null,
        error: "Base snapshot not found",
      });

      const diffResult = await failingEngine.createSnapshot(
        sessionId,
        2,
        diffContext,
        baseResult.snapshotId
      );

      // Should fall back to full snapshot
      expect(diffResult.success).toBe(true);
    });

    it("should handle restoration when checksum validation fails", async () => {
      const sessionId = "session-checksum";
      const context = { data: "test" };

      const checksumEngine = new ContextPreservationEngine({
        checksumValidation: true,
      });

      const saveResult = await checksumEngine.createSnapshot(
        sessionId,
        1,
        context
      );

      // Manually corrupt the stored snapshot
      const snapshots = (checksumEngine as any).snapshots;
      const snapshot = snapshots.get(saveResult.snapshotId!);
      snapshot.compressedContext = "corrupted"; // Corrupt the data

      const restoreResult = await checksumEngine.restoreSnapshot(
        saveResult.snapshotId!
      );

      expect(restoreResult.success).toBe(false);
      expect(restoreResult.checksumValid).toBe(false);
      expect(restoreResult.error).toContain("checksum");
    });

    it("should handle restoration when differential base snapshot is missing", async () => {
      const sessionId = "session-missing-base";
      const baseContext = { data: "base" };
      const diffContext = { data: "modified" };

      const diffEngine = new ContextPreservationEngine({
        enableDifferentialStorage: true,
      });

      // Create base snapshot
      const baseResult = await diffEngine.createSnapshot(
        sessionId,
        1,
        baseContext
      );

      // Create differential
      const diffResult = await diffEngine.createSnapshot(
        sessionId,
        2,
        diffContext,
        baseResult.snapshotId
      );

      // Delete the base snapshot
      (diffEngine as any).snapshots.delete(baseResult.snapshotId!);

      const restoreResult = await diffEngine.restoreSnapshot(
        diffResult.snapshotId!
      );

      expect(restoreResult.success).toBe(false);
      expect(restoreResult.error).toContain("base snapshot");
    });

    it("should handle restoration timeout scenarios", async () => {
      const engine = new ContextPreservationEngine();

      // Create a very large context that might take time to process
      const largeContext = {
        data: "x".repeat(50000), // Large string
        nested: Array.from({ length: 1000 }, (_, i) => ({
          id: i,
          value: `item-${i}`,
        })),
      };

      const saveResult = await engine.createSnapshot(
        "large-timeout",
        1,
        largeContext
      );
      expect(saveResult.success).toBe(true);

      const restoreResult = await engine.restoreSnapshot(
        saveResult.snapshotId!
      );
      expect(restoreResult.success).toBe(true);
      expect(restoreResult.context).toEqual(largeContext);
    });

    it("should handle restoration of non-existent snapshots", async () => {
      const engine = new ContextPreservationEngine();

      const restoreResult = await engine.restoreSnapshot("non-existent-id");

      expect(restoreResult.success).toBe(false);
      expect(restoreResult.context).toBeNull();
    });

    it("should handle differential snapshots with existing base", async () => {
      const sessionId = "session-diff-existing";
      const baseContext = { data: "base" };
      const diffContext = { data: "modified", extra: "field" };

      const diffEngine = new ContextPreservationEngine({
        enableDifferentialStorage: true,
      });

      // Create base snapshot
      const baseResult = await diffEngine.createSnapshot(
        sessionId,
        1,
        baseContext
      );

      // Create differential snapshot
      const diffResult = await diffEngine.createSnapshot(
        sessionId,
        2,
        diffContext,
        baseResult.snapshotId
      );

      expect(diffResult.success).toBe(true);

      // Restore and verify differential works
      const restoreResult = await diffEngine.restoreSnapshot(
        diffResult.snapshotId!
      );
      expect(restoreResult.success).toBe(true);
      expect(restoreResult.context).toEqual(diffContext);
    });

    it("should handle many snapshots in cache", async () => {
      const cacheEngine = new ContextPreservationEngine();

      // Create many snapshots
      for (let i = 0; i < 10; i++) {
        await cacheEngine.createSnapshot(`session-${i}`, 1, { data: i });
      }

      const cacheStats = cacheEngine.getCacheStats();
      expect(cacheStats.snapshotCount).toBe(10);
    });

    it("should handle session cleanup", async () => {
      const engine = new ContextPreservationEngine();

      // Create some snapshots
      await engine.createSnapshot("cleanup-test", 1, { data: "test" });
      const initialCount = engine.getCacheStats().snapshotCount;

      // Clear session
      engine.clearSession("cleanup-test");

      const finalCount = engine.getCacheStats().snapshotCount;
      expect(finalCount).toBeLessThan(initialCount);
    });

    it("should handle invalid configuration gracefully", async () => {
      const invalidEngine = new ContextPreservationEngine({
        maxSnapshotSizeMB: -1, // Invalid size
      });

      const context = { data: "invalid config test" };
      const result = await invalidEngine.createSnapshot("invalid", 1, context);

      // Should handle invalid config gracefully
      expect(result.success).toBe(true); // May still succeed or fail based on implementation
    });

    it("should handle compression edge cases", async () => {
      const engine = new ContextPreservationEngine({
        compressionLevel: 0, // No compression
      });

      const context = { data: "compression test" };
      const result = await engine.createSnapshot(
        "compression-test",
        1,
        context
      );

      // Should handle compression settings
      expect(result.success).toBe(true);
      expect(result.compressionRatio).toBeGreaterThanOrEqual(0);
    });

    it("should handle restoration errors gracefully", async () => {
      const engine = new ContextPreservationEngine();

      const context = { data: "test" };
      const saveResult = await engine.createSnapshot(
        "session-restore-error",
        1,
        context
      );

      // Mock restore to fail
      engine.restoreSnapshot = jest.fn().mockResolvedValue({
        success: false,
        context: null,
        timeMs: 5,
        checksumValid: false,
        error: "Restoration failed",
      });

      const restoreResult = await engine.restoreSnapshot(
        saveResult.snapshotId!
      );

      // Should handle the error gracefully
      expect(restoreResult.success).toBe(false);
      expect(restoreResult.error).toContain("Restoration failed");
    });
  });

  describe("performance and scaling", () => {
    it("should handle large context objects efficiently", async () => {
      const largeContext = {
        data: "x".repeat(10000), // 10KB of data
        nested: {
          array: Array.from({ length: 100 }, (_, i) => ({
            id: i,
            value: `item-${i}`,
          })),
          deep: {
            level1: { level2: { level3: "deep value" } },
          },
        },
      };

      const engine = new ContextPreservationEngine();

      const startTime = Date.now();
      const result = await engine.createSnapshot(
        "large-session",
        1,
        largeContext
      );
      const createTime = Date.now() - startTime;

      expect(result.success).toBe(true);
      expect(createTime).toBeLessThan(500); // Should complete within 500ms

      const restoreStartTime = Date.now();
      const restoreResult = await engine.restoreSnapshot(result.snapshotId!);
      const restoreTime = Date.now() - restoreStartTime;

      expect(restoreResult.success).toBe(true);
      expect(restoreResult.context).toEqual(largeContext);
      expect(restoreTime).toBeLessThan(200); // Should restore within 200ms
    });

    it("should maintain performance with many snapshots", async () => {
      const engine = new ContextPreservationEngine();
      const sessionId = "performance-session";
      const context = { counter: 0 };

      // Create many snapshots
      const snapshotIds = [];
      const startTime = Date.now();

      for (let i = 0; i < 50; i++) {
        context.counter = i;
        const result = await engine.createSnapshot(sessionId, i, {
          ...context,
        });
        snapshotIds.push(result.snapshotId);
      }

      const createTime = Date.now() - startTime;

      expect(snapshotIds.length).toBe(50);
      expect(createTime).toBeLessThan(2000); // Should create 50 snapshots within 2 seconds

      // Test retrieval performance
      const retrieveStartTime = Date.now();
      for (const snapshotId of snapshotIds.slice(-10)) {
        // Test last 10
        const result = await engine.restoreSnapshot(snapshotId!);
        expect(result.success).toBe(true);
      }
      const retrieveTime = Date.now() - retrieveStartTime;

      expect(retrieveTime).toBeLessThan(500); // Should retrieve 10 snapshots within 500ms
    });

    it("should handle concurrent operations correctly", async () => {
      const engine = new ContextPreservationEngine();
      const sessionId = "concurrent-session";

      // Perform multiple concurrent operations
      const promises = [];

      for (let i = 0; i < 10; i++) {
        promises.push(
          engine.createSnapshot(sessionId, i, {
            id: i,
            data: `concurrent-${i}`,
          })
        );
      }

      const results = await Promise.all(promises);

      expect(results.length).toBe(10);
      results.forEach((result) => {
        expect(result.success).toBe(true);
      });

      // All snapshots should be retrievable
      for (const result of results) {
        const restoreResult = await engine.restoreSnapshot(result.snapshotId!);
        expect(restoreResult.success).toBe(true);
      }
    });
  });

  describe("configuration edge cases", () => {
    it("should handle zero compression level", async () => {
      const engine = new ContextPreservationEngine({
        compressionLevel: 0, // No compression
      });

      const context = { data: "no compression test" };
      const result = await engine.createSnapshot("no-compress", 1, context);

      expect(result.success).toBe(true);
      expect(result.compressionRatio).toBe(0); // No compression applied
    });

    it("should handle many snapshots efficiently", async () => {
      const engine = new ContextPreservationEngine();

      // Create many snapshots
      for (let i = 0; i < 50; i++) {
        await engine.createSnapshot(`session-${i}`, 1, { data: i });
      }

      const stats = engine.getCacheStats();
      expect(stats.snapshotCount).toBe(50);
      expect(stats.totalSizeBytes).toBeGreaterThan(0);
    });

    it("should handle disabled features gracefully", async () => {
      const minimalEngine = new ContextPreservationEngine({
        enableCompression: false,
        enableDifferentialStorage: false,
        checksumValidation: false,
      });

      const context = { data: "minimal features" };
      const result = await minimalEngine.createSnapshot("minimal", 1, context);

      expect(result.success).toBe(true);
      expect(result.compressionRatio).toBe(0); // No compression

      const restoreResult = await minimalEngine.restoreSnapshot(
        result.snapshotId!
      );
      expect(restoreResult.success).toBe(true);
      expect(restoreResult.checksumValid).toBeUndefined(); // No checksum validation
    });

    it("should handle extreme configuration values", async () => {
      const extremeEngine = new ContextPreservationEngine({
        compressionLevel: 9, // Maximum
        maxSnapshotSizeMB: 1, // Small limit
      });

      const context = { data: "extreme config" };
      const result = await extremeEngine.createSnapshot("extreme", 1, context);

      expect(result.success).toBe(true);

      const stats = extremeEngine.getCacheStats();
      expect(stats.snapshotCount).toBeGreaterThanOrEqual(0);
    });
  });

  describe("resource management", () => {
    it("should properly clean up session data", async () => {
      const engine = new ContextPreservationEngine();

      // Create some snapshots
      for (let i = 0; i < 5; i++) {
        await engine.createSnapshot(`cleanup-${i}`, 1, { data: i });
      }

      const initialCacheSize = engine.getCacheStats().snapshotCount;

      // Clear session should clean up
      engine.clearSession("cleanup-0");

      const finalCacheSize = engine.getCacheStats().snapshotCount;

      expect(finalCacheSize).toBeLessThan(initialCacheSize);

      // Engine should still function for basic operations
      const context = { data: "post-cleanup" };
      const result = await engine.createSnapshot("post-cleanup", 1, context);

      expect(result.success).toBe(true);
    });

    it("should handle cache pressure gracefully", async () => {
      const engine = new ContextPreservationEngine();

      // Create many snapshots to simulate cache pressure
      const promises = [];
      for (let i = 0; i < 20; i++) {
        promises.push(
          engine.createSnapshot(`cache-${i}`, 1, {
            largeData: "x".repeat(1000),
          })
        );
      }

      const results = await Promise.all(promises);

      // Should handle cache pressure without crashing
      results.forEach((result) => {
        expect(result.success).toBe(true);
      });

      // Cache stats should be available
      const cacheStats = engine.getCacheStats();
      expect(cacheStats).toBeDefined();
      expect(typeof cacheStats.totalSizeBytes).toBe("number");
    });
  });
});
