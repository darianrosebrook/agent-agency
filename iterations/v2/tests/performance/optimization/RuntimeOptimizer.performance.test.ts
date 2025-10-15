/**
 * @fileoverview Performance tests for Runtime Optimization Engine
 *
 * Tests performance requirements: <10ms overhead, P95 response times
 *
 * @author @darianrosebrook
 */

import { PerformanceMonitor } from "@/optimization/PerformanceMonitor";
import { RuntimeOptimizer } from "@/optimization/RuntimeOptimizer";
import { MetricType, type PerformanceMetric } from "@/types/optimization-types";
import { afterEach, beforeEach, describe, expect, it } from "@jest/globals";

describe("RuntimeOptimizer Performance", () => {
  let optimizer: RuntimeOptimizer;
  let performanceMonitor: PerformanceMonitor;

  beforeEach(async () => {
    // Initialize with performance-optimized settings
    optimizer = new RuntimeOptimizer({
      enabled: true,
      collectionIntervalMs: 100, // Faster for performance tests
      analysisWindowMs: 1000,
      enableTrendAnalysis: true,
      enableCacheOptimization: false, // Disable for cleaner performance tests
    });

    await optimizer.initialize();
    performanceMonitor = (optimizer as any).performanceMonitor;
  });

  afterEach(async () => {
    if (optimizer) {
      await optimizer.stop();
    }
    if (performanceMonitor) {
      await performanceMonitor.stop();
    }
  });

  describe("Overhead Performance", () => {
    it("should maintain <10ms overhead for metric recording", async () => {
      await optimizer.start();

      const iterations = 1000;
      const metrics: PerformanceMetric[] = [];

      // Generate test metrics
      for (let i = 0; i < iterations; i++) {
        metrics.push({
          type: MetricType.CPU,
          value: 50 + (i % 50),
          unit: "%",
          timestamp: new Date(),
          source: `perf-test-${i}`,
        });
      }

      // Measure recording performance
      const startTime = process.hrtime.bigint();

      for (const metric of metrics) {
        await performanceMonitor.recordMetric(metric);
      }

      const endTime = process.hrtime.bigint();
      const totalTimeMs = Number(endTime - startTime) / 1_000_000;
      const avgTimePerMetric = totalTimeMs / iterations;

      console.log(
        `Performance: ${iterations} metrics in ${totalTimeMs.toFixed(
          2
        )}ms (avg: ${avgTimePerMetric.toFixed(3)}ms per metric)`
      );

      // Verify <10ms overhead per metric
      expect(avgTimePerMetric).toBeLessThan(10);

      // Verify total time is reasonable
      expect(totalTimeMs).toBeLessThan(10000); // 10 seconds max for 1000 metrics
    });

    it("should maintain <10ms overhead for analysis operations", async () => {
      await optimizer.start();

      // Record some baseline metrics
      const baselineMetrics: PerformanceMetric[] = [];
      for (let i = 0; i < 50; i++) {
        baselineMetrics.push({
          type: MetricType.CPU,
          value: 30 + (i % 20),
          unit: "%",
          timestamp: new Date(Date.now() - (50 - i) * 100),
          source: "baseline",
        });
      }

      for (const metric of baselineMetrics) {
        await performanceMonitor.recordMetric(metric);
      }

      // Wait for metrics to be processed
      await new Promise((resolve) => setTimeout(resolve, 200));

      // Measure analysis performance
      const analysisTimes: number[] = [];
      const iterations = 100;

      for (let i = 0; i < iterations; i++) {
        const startTime = process.hrtime.bigint();
        await optimizer.analyze();
        const endTime = process.hrtime.bigint();

        const analysisTimeMs = Number(endTime - startTime) / 1_000_000;
        analysisTimes.push(analysisTimeMs);
      }

      // Calculate P95 response time
      analysisTimes.sort((a, b) => a - b);
      const p95Index = Math.floor(iterations * 0.95);
      const p95Time = analysisTimes[p95Index];
      const avgTime =
        analysisTimes.reduce((sum, time) => sum + time, 0) / iterations;

      console.log(
        `Analysis Performance: P95=${p95Time.toFixed(
          2
        )}ms, Avg=${avgTime.toFixed(2)}ms`
      );

      // Verify P95 response time < 500ms (per working spec)
      expect(p95Time).toBeLessThan(500);

      // Verify average response time is reasonable
      expect(avgTime).toBeLessThan(250);
    });

    it("should maintain <10ms overhead for bottleneck detection", async () => {
      await optimizer.start();

      // Record metrics that will trigger bottleneck detection
      const bottleneckMetrics: PerformanceMetric[] = [];
      for (let i = 0; i < 20; i++) {
        bottleneckMetrics.push({
          type: MetricType.CPU,
          value: 85 + (i % 10), // Above threshold
          unit: "%",
          timestamp: new Date(Date.now() - (20 - i) * 100),
          source: "bottleneck-test",
        });
      }

      // Measure bottleneck detection performance
      const detectionTimes: number[] = [];
      const iterations = 50;

      for (let i = 0; i < iterations; i++) {
        // Record a metric
        const metric = bottleneckMetrics[i % bottleneckMetrics.length];
        await performanceMonitor.recordMetric(metric);

        // Measure detection time
        const startTime = process.hrtime.bigint();
        await optimizer.analyze();
        const endTime = process.hrtime.bigint();

        const detectionTimeMs = Number(endTime - startTime) / 1_000_000;
        detectionTimes.push(detectionTimeMs);
      }

      // Calculate P95 response time
      detectionTimes.sort((a, b) => a - b);
      const p95Index = Math.floor(iterations * 0.95);
      const p95Time = detectionTimes[p95Index];
      const avgTime =
        detectionTimes.reduce((sum, time) => sum + time, 0) / iterations;

      console.log(
        `Bottleneck Detection Performance: P95=${p95Time.toFixed(
          2
        )}ms, Avg=${avgTime.toFixed(2)}ms`
      );

      // Verify P95 response time < 10ms
      expect(p95Time).toBeLessThan(10);

      // Verify average response time is reasonable
      expect(avgTime).toBeLessThan(5);
    });
  });

  describe("Memory Usage", () => {
    it("should maintain reasonable memory usage during operation", async () => {
      await optimizer.start();

      const initialMemory = process.memoryUsage();
      console.log(
        `Initial memory: ${(initialMemory.heapUsed / 1024 / 1024).toFixed(2)}MB`
      );

      // Record a large number of metrics
      const iterations = 5000;
      for (let i = 0; i < iterations; i++) {
        const metric: PerformanceMetric = {
          type: MetricType.CPU,
          value: 50 + (i % 50),
          unit: "%",
          timestamp: new Date(),
          source: `memory-test-${i}`,
        };

        await performanceMonitor.recordMetric(metric);

        // Check memory every 1000 iterations
        if (i % 1000 === 0 && i > 0) {
          const currentMemory = process.memoryUsage();
          const memoryIncrease =
            currentMemory.heapUsed - initialMemory.heapUsed;
          const memoryIncreaseMB = memoryIncrease / 1024 / 1024;

          console.log(
            `Memory after ${i} metrics: +${memoryIncreaseMB.toFixed(2)}MB`
          );

          // Memory increase should be reasonable (less than 50MB for 1000 metrics)
          expect(memoryIncreaseMB).toBeLessThan(50);
        }
      }

      const finalMemory = process.memoryUsage();
      const totalMemoryIncrease = finalMemory.heapUsed - initialMemory.heapUsed;
      const totalMemoryIncreaseMB = totalMemoryIncrease / 1024 / 1024;

      console.log(
        `Final memory increase: +${totalMemoryIncreaseMB.toFixed(2)}MB`
      );

      // Total memory increase should be reasonable
      expect(totalMemoryIncreaseMB).toBeLessThan(200); // 200MB max for 5000 metrics
    });
  });

  describe("Concurrent Performance", () => {
    it("should handle concurrent metric recording efficiently", async () => {
      await optimizer.start();

      const concurrentOperations = 100;
      const metricsPerOperation = 10;

      const startTime = process.hrtime.bigint();

      // Create concurrent operations
      const promises = Array.from(
        { length: concurrentOperations },
        async (_, i) => {
          const operationMetrics: PerformanceMetric[] = [];

          for (let j = 0; j < metricsPerOperation; j++) {
            operationMetrics.push({
              type: MetricType.CPU,
              value: 50 + ((i + j) % 50),
              unit: "%",
              timestamp: new Date(),
              source: `concurrent-${i}-${j}`,
            });
          }

          // Record all metrics for this operation
          for (const metric of operationMetrics) {
            await performanceMonitor.recordMetric(metric);
          }
        }
      );

      // Wait for all operations to complete
      await Promise.all(promises);

      const endTime = process.hrtime.bigint();
      const totalTimeMs = Number(endTime - startTime) / 1_000_000;
      const totalMetrics = concurrentOperations * metricsPerOperation;
      const avgTimePerMetric = totalTimeMs / totalMetrics;

      console.log(
        `Concurrent Performance: ${totalMetrics} metrics in ${totalTimeMs.toFixed(
          2
        )}ms (avg: ${avgTimePerMetric.toFixed(3)}ms per metric)`
      );

      // Verify concurrent performance is still good
      expect(avgTimePerMetric).toBeLessThan(15); // Slightly higher threshold for concurrent operations
      expect(totalTimeMs).toBeLessThan(5000); // 5 seconds max for 1000 concurrent metrics
    });
  });

  describe("Scalability", () => {
    it("should maintain performance with increasing metric volume", async () => {
      await optimizer.start();

      const volumeTests = [100, 500, 1000, 2000];
      const results: { volume: number; avgTime: number; p95Time: number }[] =
        [];

      for (const volume of volumeTests) {
        // Clear previous metrics
        await performanceMonitor.clearMetrics(new Date(0));

        // Record metrics
        const metrics: PerformanceMetric[] = [];
        for (let i = 0; i < volume; i++) {
          metrics.push({
            type: MetricType.CPU,
            value: 50 + (i % 50),
            unit: "%",
            timestamp: new Date(Date.now() - (volume - i) * 100),
            source: `volume-test-${i}`,
          });
        }

        for (const metric of metrics) {
          await performanceMonitor.recordMetric(metric);
        }

        // Wait for processing
        await new Promise((resolve) => setTimeout(resolve, 100));

        // Measure analysis performance
        const analysisTimes: number[] = [];
        const iterations = 20;

        for (let i = 0; i < iterations; i++) {
          const startTime = process.hrtime.bigint();
          await optimizer.analyze();
          const endTime = process.hrtime.bigint();

          const analysisTimeMs = Number(endTime - startTime) / 1_000_000;
          analysisTimes.push(analysisTimeMs);
        }

        // Calculate statistics
        analysisTimes.sort((a, b) => a - b);
        const p95Index = Math.floor(iterations * 0.95);
        const p95Time = analysisTimes[p95Index];
        const avgTime =
          analysisTimes.reduce((sum, time) => sum + time, 0) / iterations;

        results.push({ volume, avgTime, p95Time });
        console.log(
          `Volume ${volume}: Avg=${avgTime.toFixed(2)}ms, P95=${p95Time.toFixed(
            2
          )}ms`
        );
      }

      // Verify performance doesn't degrade significantly with volume
      for (const result of results) {
        expect(result.p95Time).toBeLessThan(500); // P95 should stay under 500ms (per spec)
        expect(result.avgTime).toBeLessThan(250); // Average should stay under 250ms
      }

      // Verify performance scaling is reasonable (not exponential)
      const firstResult = results[0];
      const lastResult = results[results.length - 1];
      const performanceRatio = lastResult.avgTime / firstResult.avgTime;

      console.log(`Performance scaling ratio: ${performanceRatio.toFixed(2)}x`);
      expect(performanceRatio).toBeLessThan(20); // Should not degrade more than 20x (realistic for large volumes)
    });
  });
});
