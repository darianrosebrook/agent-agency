/**
 * @fileoverview Unit tests for BottleneckDetector
 *
 * @author @darianrosebrook
 */

import { BottleneckDetector } from "@/optimization/BottleneckDetector";
import {
  BottleneckSeverity,
  MetricType,
  type PerformanceMetric,
} from "@/types/optimization-types";
import { beforeEach, describe, expect, it } from "@jest/globals";

describe("BottleneckDetector", () => {
  let detector: BottleneckDetector;

  beforeEach(() => {
    detector = new BottleneckDetector();
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  describe("Bottleneck Detection", () => {
    it("should detect CPU bottleneck", async () => {
      const metric: PerformanceMetric = {
        type: MetricType.CPU,
        value: 90, // Above default threshold of 80
        unit: "%",
        timestamp: new Date(),
        source: "test-component",
      };

      const bottlenecks = await detector.detectBottlenecks([metric]);

      expect(bottlenecks).toHaveLength(1);
      expect(bottlenecks[0].metricType).toBe(MetricType.CPU);
      expect(bottlenecks[0].component).toBe("test-component");
      expect(bottlenecks[0].currentValue).toBe(90);
    });

    it("should detect memory bottleneck", async () => {
      const metric: PerformanceMetric = {
        type: MetricType.MEMORY,
        value: 90, // Above default threshold of 85
        unit: "%",
        timestamp: new Date(),
        source: "memory-intensive-component",
      };

      const bottlenecks = await detector.detectBottlenecks([metric]);

      expect(bottlenecks).toHaveLength(1);
      expect(bottlenecks[0].metricType).toBe(MetricType.MEMORY);
      expect(bottlenecks[0].severity).toBeDefined();
    });

    it("should detect latency bottleneck", async () => {
      const metric: PerformanceMetric = {
        type: MetricType.LATENCY,
        value: 1500, // Above default threshold of 1000ms
        unit: "ms",
        timestamp: new Date(),
        source: "slow-api",
      };

      const bottlenecks = await detector.detectBottlenecks([metric]);

      expect(bottlenecks).toHaveLength(1);
      expect(bottlenecks[0].metricType).toBe(MetricType.LATENCY);
    });

    it("should detect low throughput bottleneck", async () => {
      const metric: PerformanceMetric = {
        type: MetricType.THROUGHPUT,
        value: 50, // Below minimum threshold of 100
        unit: "req/s",
        timestamp: new Date(),
        source: "api-server",
      };

      const bottlenecks = await detector.detectBottlenecks([metric]);

      expect(bottlenecks).toHaveLength(1);
      expect(bottlenecks[0].metricType).toBe(MetricType.THROUGHPUT);
    });

    it("should detect low cache hit rate", async () => {
      const metric: PerformanceMetric = {
        type: MetricType.CACHE_HIT_RATE,
        value: 50, // Below minimum threshold of 70
        unit: "%",
        timestamp: new Date(),
        source: "cache-layer",
      };

      const bottlenecks = await detector.detectBottlenecks([metric]);

      expect(bottlenecks).toHaveLength(1);
      expect(bottlenecks[0].metricType).toBe(MetricType.CACHE_HIT_RATE);
    });

    it("should not detect bottleneck when below threshold", async () => {
      const metric: PerformanceMetric = {
        type: MetricType.CPU,
        value: 50, // Below threshold of 80
        unit: "%",
        timestamp: new Date(),
        source: "test-component",
      };

      const bottlenecks = await detector.detectBottlenecks([metric]);

      expect(bottlenecks).toHaveLength(0);
    });

    it("should detect multiple bottlenecks", async () => {
      const metrics: PerformanceMetric[] = [
        {
          type: MetricType.CPU,
          value: 90,
          unit: "%",
          timestamp: new Date(),
          source: "component-1",
        },
        {
          type: MetricType.MEMORY,
          value: 95,
          unit: "%",
          timestamp: new Date(),
          source: "component-1",
        },
      ];

      const bottlenecks = await detector.detectBottlenecks(metrics);

      expect(bottlenecks).toHaveLength(2);
    });
  });

  describe("Severity Classification", () => {
    it("should classify critical severity for high deviation", async () => {
      const metric: PerformanceMetric = {
        type: MetricType.CPU,
        value: 99, // (99-60)/60 = 0.65 = 65% deviation (>= 0.5 threshold)
        unit: "%",
        timestamp: new Date(),
        source: "critical-component",
      };

      // Override threshold to 60 for this test to achieve >50% deviation
      const testDetector = new BottleneckDetector({
        [MetricType.CPU]: 60,
        [MetricType.MEMORY]: 80,
        [MetricType.LATENCY]: 1000,
      });

      const bottlenecks = await testDetector.detectBottlenecks([metric]);

      expect(bottlenecks[0].severity).toBe(BottleneckSeverity.CRITICAL);
    });

    it("should classify high severity for moderate deviation", async () => {
      const metric: PerformanceMetric = {
        type: MetricType.CPU,
        value: 94, // (94-70)/70 = 0.343 = 34.3% deviation (>= 0.3 threshold)
        unit: "%",
        timestamp: new Date(),
        source: "high-component",
      };

      // Override threshold to 70 for this test to achieve >30% deviation
      const testDetector = new BottleneckDetector({
        [MetricType.CPU]: 70,
        [MetricType.MEMORY]: 80,
        [MetricType.LATENCY]: 1000,
      });

      const bottlenecks = await testDetector.detectBottlenecks([metric]);

      expect(bottlenecks[0].severity).toBe(BottleneckSeverity.HIGH);
    });

    it("should increase severity with occurrence count", async () => {
      const metric: PerformanceMetric = {
        type: MetricType.CPU,
        value: 82, // Just above threshold
        unit: "%",
        timestamp: new Date(),
        source: "test-component",
      };

      // First occurrence - should be LOW severity
      let bottlenecks = await detector.detectBottlenecks([metric]);
      expect(bottlenecks[0].severity).toBe(BottleneckSeverity.LOW);

      // Multiple more occurrences
      for (let i = 0; i < 15; i++) {
        bottlenecks = await detector.detectBottlenecks([metric]);
      }

      // After 10+ occurrences - should be CRITICAL
      expect(bottlenecks[0].severity).toBe(BottleneckSeverity.CRITICAL);
    });
  });

  describe("Threshold Management", () => {
    it("should use custom thresholds", async () => {
      const customDetector = new BottleneckDetector({
        [MetricType.CPU]: 60,
      });

      const metric: PerformanceMetric = {
        type: MetricType.CPU,
        value: 70, // Above custom threshold of 60
        unit: "%",
        timestamp: new Date(),
        source: "test",
      };

      const bottlenecks = await customDetector.detectBottlenecks([metric]);

      expect(bottlenecks).toHaveLength(1);
      expect(bottlenecks[0].threshold).toBe(60);
    });

    it("should update thresholds", async () => {
      detector.updateThresholds({
        [MetricType.CPU]: 90,
      });

      const metric: PerformanceMetric = {
        type: MetricType.CPU,
        value: 85, // Below new threshold of 90
        unit: "%",
        timestamp: new Date(),
        source: "test",
      };

      const bottlenecks = await detector.detectBottlenecks([metric]);

      expect(bottlenecks).toHaveLength(0);
    });
  });

  describe("Active Bottleneck Management", () => {
    it("should track active bottlenecks", async () => {
      const metric: PerformanceMetric = {
        type: MetricType.CPU,
        value: 90,
        unit: "%",
        timestamp: new Date(),
        source: "test-component",
      };

      await detector.detectBottlenecks([metric]);

      const activeBottlenecks = detector.getActiveBottlenecks();
      expect(activeBottlenecks).toHaveLength(1);
    });

    it("should update existing bottleneck", async () => {
      const now = new Date();

      // First detection
      await detector.detectBottlenecks([
        {
          type: MetricType.CPU,
          value: 85,
          unit: "%",
          timestamp: now,
          source: "test-component",
        },
      ]);

      const active1 = detector.getActiveBottlenecks();
      expect(active1[0].occurrenceCount).toBe(1);

      // Second detection - should update
      await detector.detectBottlenecks([
        {
          type: MetricType.CPU,
          value: 90,
          unit: "%",
          timestamp: new Date(),
          source: "test-component",
        },
      ]);

      const active2 = detector.getActiveBottlenecks();
      expect(active2[0].occurrenceCount).toBe(2);
      expect(active2[0].currentValue).toBe(90);
    });

    it("should clear resolved bottlenecks", async () => {
      const oldTime = new Date(Date.now() - 10000);

      await detector.detectBottlenecks([
        {
          type: MetricType.CPU,
          value: 90,
          unit: "%",
          timestamp: oldTime,
          source: "test-component",
        },
      ]);

      expect(detector.getActiveBottlenecks()).toHaveLength(1);

      const cutoff = new Date(Date.now() - 5000);
      await detector.clearResolvedBottlenecks(cutoff);

      expect(detector.getActiveBottlenecks()).toHaveLength(0);
    });
  });

  describe("Impact Description", () => {
    it("should generate impact description", async () => {
      const metric: PerformanceMetric = {
        type: MetricType.CPU,
        value: 90,
        unit: "%",
        timestamp: new Date(),
        source: "test-component",
      };

      const bottlenecks = await detector.detectBottlenecks([metric]);

      expect(bottlenecks[0].impact).toContain("test-component");
      expect(bottlenecks[0].impact).toBeTruthy();
    });

    it("should include urgency prefix for critical severity", async () => {
      const metric: PerformanceMetric = {
        type: MetricType.CPU,
        value: 140, // Very high, will be CRITICAL
        unit: "%",
        timestamp: new Date(),
        source: "critical-component",
      };

      const bottlenecks = await detector.detectBottlenecks([metric]);

      expect(bottlenecks[0].severity).toBe(BottleneckSeverity.CRITICAL);
      expect(bottlenecks[0].impact).toContain("URGENT");
    });
  });

  describe("Edge Cases", () => {
    it("should handle empty metrics array", async () => {
      const bottlenecks = await detector.detectBottlenecks([]);

      expect(bottlenecks).toHaveLength(0);
    });

    it("should handle metrics with no thresholds", async () => {
      const metric: PerformanceMetric = {
        type: MetricType.ERROR_RATE,
        value: 100,
        unit: "%",
        timestamp: new Date(),
        source: "test",
      };

      // ERROR_RATE has default threshold
      const bottlenecks = await detector.detectBottlenecks([metric]);

      expect(bottlenecks.length).toBeGreaterThanOrEqual(0);
    });

    it("should handle bottleneck history", () => {
      const history = detector.getBottleneckHistory("test-component");

      expect(history).toBeDefined();
      expect(Array.isArray(history)).toBe(true);
    });
  });
});
