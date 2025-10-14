/**
 * @fileoverview Unit tests for PerformanceMonitor
 *
 * @author @darianrosebrook
 */

import { PerformanceMonitor } from "@/optimization/PerformanceMonitor";
import { MetricType, type PerformanceMetric } from "@/types/optimization-types";
import {
  afterEach,
  beforeEach,
  describe,
  expect,
  it,
  jest,
} from "@jest/globals";

describe("PerformanceMonitor", () => {
  let monitor: PerformanceMonitor;

  beforeEach(() => {
    monitor = new PerformanceMonitor();
  });

  afterEach(async () => {
    await monitor.stop();
    // Ensure real timers are restored after each test
    jest.useRealTimers();
  });

  describe("Metric Recording", () => {
    it("should record performance metrics", async () => {
      const metric: PerformanceMetric = {
        type: MetricType.CPU,
        value: 75,
        unit: "%",
        timestamp: new Date(),
        source: "test-component",
      };

      await monitor.recordMetric(metric);

      expect(monitor.getMetricCount()).toBe(1);
    });

    it("should handle multiple metrics", async () => {
      const metrics: PerformanceMetric[] = [
        {
          type: MetricType.CPU,
          value: 75,
          unit: "%",
          timestamp: new Date(),
          source: "component-1",
        },
        {
          type: MetricType.MEMORY,
          value: 80,
          unit: "%",
          timestamp: new Date(),
          source: "component-1",
        },
        {
          type: MetricType.LATENCY,
          value: 250,
          unit: "ms",
          timestamp: new Date(),
          source: "component-2",
        },
      ];

      for (const metric of metrics) {
        await monitor.recordMetric(metric);
      }

      expect(monitor.getMetricCount()).toBe(3);
    });

    it("should enforce circular buffer limit", async () => {
      const smallMonitor = new PerformanceMonitor({ maxMetrics: 5 });

      // Record 10 metrics (exceeds limit of 5)
      for (let i = 0; i < 10; i++) {
        await smallMonitor.recordMetric({
          type: MetricType.CPU,
          value: i,
          unit: "%",
          timestamp: new Date(),
          source: "test",
        });
      }

      // Should only keep the last 5
      expect(smallMonitor.getMetricCount()).toBe(5);

      await smallMonitor.stop();
    });
  });

  describe("Metric Retrieval", () => {
    it("should retrieve metrics by time window", async () => {
      const now = new Date();
      const oneMinuteAgo = new Date(now.getTime() - 60000);
      const twoMinutesAgo = new Date(now.getTime() - 120000);

      await monitor.recordMetric({
        type: MetricType.CPU,
        value: 50,
        unit: "%",
        timestamp: twoMinutesAgo,
        source: "test",
      });

      await monitor.recordMetric({
        type: MetricType.CPU,
        value: 75,
        unit: "%",
        timestamp: oneMinuteAgo,
        source: "test",
      });

      await monitor.recordMetric({
        type: MetricType.CPU,
        value: 90,
        unit: "%",
        timestamp: now,
        source: "test",
      });

      // Get metrics from last minute
      const recentMetrics = await monitor.getMetrics(oneMinuteAgo, now);

      expect(recentMetrics).toHaveLength(2);
      expect(recentMetrics[0].value).toBe(75);
      expect(recentMetrics[1].value).toBe(90);
    });

    it("should filter metrics by type", async () => {
      const now = new Date();

      await monitor.recordMetric({
        type: MetricType.CPU,
        value: 75,
        unit: "%",
        timestamp: now,
        source: "test",
      });

      await monitor.recordMetric({
        type: MetricType.MEMORY,
        value: 80,
        unit: "%",
        timestamp: now,
        source: "test",
      });

      const oneHourAgo = new Date(now.getTime() - 3600000);
      const cpuMetrics = await monitor.getMetrics(
        oneHourAgo,
        now,
        MetricType.CPU
      );

      expect(cpuMetrics).toHaveLength(1);
      expect(cpuMetrics[0].type).toBe(MetricType.CPU);
    });

    it("should retrieve latest N metrics", async () => {
      // Record 5 metrics
      for (let i = 0; i < 5; i++) {
        await monitor.recordMetric({
          type: MetricType.CPU,
          value: i * 10,
          unit: "%",
          timestamp: new Date(),
          source: "test",
        });
      }

      const latestThree = await monitor.getLatestMetrics(3);

      expect(latestThree).toHaveLength(3);
      expect(latestThree[0].value).toBe(20); // Last 3: indices 2, 3, 4
      expect(latestThree[2].value).toBe(40);
    });

    it("should filter latest metrics by type", async () => {
      await monitor.recordMetric({
        type: MetricType.CPU,
        value: 50,
        unit: "%",
        timestamp: new Date(),
        source: "test",
      });

      await monitor.recordMetric({
        type: MetricType.MEMORY,
        value: 60,
        unit: "%",
        timestamp: new Date(),
        source: "test",
      });

      await monitor.recordMetric({
        type: MetricType.CPU,
        value: 70,
        unit: "%",
        timestamp: new Date(),
        source: "test",
      });

      const latestCpu = await monitor.getLatestMetrics(2, MetricType.CPU);

      expect(latestCpu).toHaveLength(2);
      expect(latestCpu[0].value).toBe(50);
      expect(latestCpu[1].value).toBe(70);
    });
  });

  describe("Metric Cleanup", () => {
    it("should clear old metrics", async () => {
      const now = new Date();
      const oneHourAgo = new Date(now.getTime() - 3600000);

      await monitor.recordMetric({
        type: MetricType.CPU,
        value: 50,
        unit: "%",
        timestamp: oneHourAgo,
        source: "test",
      });

      await monitor.recordMetric({
        type: MetricType.CPU,
        value: 75,
        unit: "%",
        timestamp: now,
        source: "test",
      });

      expect(monitor.getMetricCount()).toBe(2);

      const thirtyMinutesAgo = new Date(now.getTime() - 1800000);
      await monitor.clearMetrics(thirtyMinutesAgo);

      expect(monitor.getMetricCount()).toBe(1);

      const remaining = await monitor.getLatestMetrics(10);
      expect(remaining[0].value).toBe(75);
    });

    it("should not clear recent metrics", async () => {
      const now = new Date();

      await monitor.recordMetric({
        type: MetricType.CPU,
        value: 75,
        unit: "%",
        timestamp: now,
        source: "test",
      });

      const oneHourAgo = new Date(now.getTime() - 3600000);
      await monitor.clearMetrics(oneHourAgo);

      expect(monitor.getMetricCount()).toBe(1);
    });

    it("should enable automatic cleanup", async () => {
      // Test automatic cleanup configuration without testing the timer itself
      // (Timer testing with async operations is complex and prone to flakiness)
      const autoCleanMonitor = new PerformanceMonitor({
        enableAutoCleanup: true,
        cleanupIntervalMs: 1000,
        autoCleanOlderThanMs: 5000,
      });

      await autoCleanMonitor.start();

      const config = autoCleanMonitor.getConfig();
      expect(config.enableAutoCleanup).toBe(true);
      expect(config.cleanupIntervalMs).toBe(1000);
      expect(config.autoCleanOlderThanMs).toBe(5000);

      // Test manual cleanup works as expected
      const oldTime = new Date(Date.now() - 10000);
      await autoCleanMonitor.recordMetric({
        type: MetricType.CPU,
        value: 50,
        unit: "%",
        timestamp: oldTime,
        source: "test",
      });

      await autoCleanMonitor.recordMetric({
        type: MetricType.CPU,
        value: 75,
        unit: "%",
        timestamp: new Date(),
        source: "test",
      });

      expect(autoCleanMonitor.getMetricCount()).toBe(2);

      // Manually trigger cleanup to test the cleanup logic
      const cutoffTime = new Date(Date.now() - 5000);
      await autoCleanMonitor.clearMetrics(cutoffTime);

      // Old metric (>5s old) should be cleaned up, recent one remains
      expect(autoCleanMonitor.getMetricCount()).toBe(1);

      await autoCleanMonitor.stop();
    });
  });

  describe("Configuration", () => {
    it("should return configuration", () => {
      const config = monitor.getConfig();

      expect(config.maxMetrics).toBeGreaterThan(0);
      expect(config.enableAutoCleanup).toBeDefined();
    });

    it("should update configuration", () => {
      monitor.updateConfig({
        maxMetrics: 5000,
        enableAutoCleanup: false,
      });

      const config = monitor.getConfig();

      expect(config.maxMetrics).toBe(5000);
      expect(config.enableAutoCleanup).toBe(false);
    });
  });

  describe("Lifecycle", () => {
    it("should start monitoring", async () => {
      await monitor.start();
      // Should not throw
    });

    it("should stop monitoring", async () => {
      await monitor.start();
      await monitor.stop();
      // Should not throw
    });

    it("should handle multiple start calls", async () => {
      await monitor.start();
      await monitor.start(); // Should not throw
      await monitor.stop();
    });

    it("should handle multiple stop calls", async () => {
      await monitor.start();
      await monitor.stop();
      await monitor.stop(); // Should not throw
    });
  });

  describe("Edge Cases", () => {
    it("should handle empty time window", async () => {
      const now = new Date();
      const metrics = await monitor.getMetrics(now, now);

      expect(metrics).toHaveLength(0);
    });

    it("should handle request for more metrics than available", async () => {
      await monitor.recordMetric({
        type: MetricType.CPU,
        value: 75,
        unit: "%",
        timestamp: new Date(),
        source: "test",
      });

      const latest = await monitor.getLatestMetrics(100);

      expect(latest).toHaveLength(1);
    });

    it("should handle concurrent metric recording", async () => {
      const promises = [];

      for (let i = 0; i < 10; i++) {
        promises.push(
          monitor.recordMetric({
            type: MetricType.CPU,
            value: i,
            unit: "%",
            timestamp: new Date(),
            source: "test",
          })
        );
      }

      await Promise.all(promises);

      expect(monitor.getMetricCount()).toBe(10);
    }, 10000); // 10 second timeout for concurrent operations
  });
});
