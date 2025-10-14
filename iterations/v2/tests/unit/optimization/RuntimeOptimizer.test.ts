/**
 * @fileoverview Unit tests for RuntimeOptimizer
 *
 * @author @darianrosebrook
 */

import { RuntimeOptimizer } from "@/optimization/RuntimeOptimizer";
import { afterEach, beforeEach, describe, expect, it } from "@jest/globals";

describe("RuntimeOptimizer", () => {
  let optimizer: RuntimeOptimizer;

  beforeEach(() => {
    optimizer = new RuntimeOptimizer({
      enabled: true,
      collectionIntervalMs: 1000,
      analysisWindowMs: 5000,
    });
  });

  afterEach(async () => {
    await optimizer.stop();
  });

  describe("Initialization", () => {
    it("should initialize optimizer", async () => {
      await optimizer.initialize();

      const status = optimizer.getHealthStatus();
      expect(status).toBeDefined();
    });

    it("should start monitoring", async () => {
      await optimizer.initialize();
      await optimizer.start();

      const status = optimizer.getHealthStatus();
      expect(status.isRunning).toBe(true);
    });

    it("should stop monitoring", async () => {
      await optimizer.initialize();
      await optimizer.start();
      await optimizer.stop();

      const status = optimizer.getHealthStatus();
      expect(status.isRunning).toBe(false);
    });
  });

  describe("Configuration", () => {
    it("should get configuration", () => {
      const config = optimizer.getConfig();

      expect(config.enabled).toBe(true);
      expect(config.collectionIntervalMs).toBe(1000);
    });

    it("should update configuration", () => {
      optimizer.updateConfig({
        collectionIntervalMs: 2000,
        enableCacheOptimization: false,
      });

      const config = optimizer.getConfig();
      expect(config.collectionIntervalMs).toBe(2000);
      expect(config.enableCacheOptimization).toBe(false);
    });
  });

  describe("Analysis", () => {
    it("should perform analysis", async () => {
      await optimizer.initialize();

      const analysis = await optimizer.analyze();

      expect(analysis).toBeDefined();
      expect(analysis.timestamp).toBeInstanceOf(Date);
      expect(analysis.bottlenecks).toBeDefined();
      expect(analysis.recommendations).toBeDefined();
      expect(analysis.healthScore).toBeGreaterThanOrEqual(0);
      expect(analysis.healthScore).toBeLessThanOrEqual(100);
    });

    it("should track analysis history", async () => {
      await optimizer.initialize();

      await optimizer.analyze();
      await optimizer.analyze();

      const history = optimizer.getAnalysisHistory(10);
      expect(history.length).toBeGreaterThanOrEqual(2);
    });
  });

  describe("Health Status", () => {
    it("should report health status", () => {
      const status = optimizer.getHealthStatus();

      expect(status.isRunning).toBeDefined();
      expect(status.metricsCollected).toBeGreaterThanOrEqual(0);
      expect(status.bottlenecksDetected).toBeGreaterThanOrEqual(0);
    });
  });

  describe("Cache Statistics", () => {
    it("should get cache statistics", async () => {
      await optimizer.initialize();

      const stats = await optimizer.getCacheStatistics();

      expect(Array.isArray(stats)).toBe(true);
    });
  });

  describe("Performance Trends", () => {
    it("should get performance trends", async () => {
      await optimizer.initialize();

      const trends = await optimizer.getPerformanceTrends();

      expect(Array.isArray(trends)).toBe(true);
    });
  });

  describe("Edge Cases", () => {
    it("should handle disabled optimizer", async () => {
      const disabledOptimizer = new RuntimeOptimizer({
        enabled: false,
      });

      await disabledOptimizer.initialize();
      await disabledOptimizer.start();

      const status = disabledOptimizer.getHealthStatus();
      expect(status.isRunning).toBe(false);

      await disabledOptimizer.stop();
    });

    it("should handle multiple start calls", async () => {
      await optimizer.initialize();
      await optimizer.start();
      await optimizer.start(); // Should not throw

      await optimizer.stop();
    });

    it("should handle analysis without initialization", async () => {
      const analysis = await optimizer.analyze();

      expect(analysis).toBeDefined();
    });
  });
});
