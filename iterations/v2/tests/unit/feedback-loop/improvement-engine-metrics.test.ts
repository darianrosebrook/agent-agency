/**
 * ImprovementEngine Metrics Querying Tests
 *
 * Tests for the ImprovementEngine's real metrics querying functionality
 * replacing the previous random simulation approach.
 *
 * @author @darianrosebrook
 */

import { configManager } from "@/config/ConfigManager";
import { ImprovementEngine } from "@/feedback-loop/ImprovementEngine";
import { MetricsCollector } from "@/monitoring/MetricsCollector";
import { FeedbackRecommendation } from "@/types/feedback-loop";

describe("ImprovementEngine Metrics Querying", () => {
  let improvementEngine: ImprovementEngine;
  let metricsCollector: MetricsCollector;

  beforeEach(() => {
    metricsCollector = new MetricsCollector();
    improvementEngine = new ImprovementEngine(configManager, metricsCollector);
  });

  describe("monitorImprovementEffects with real metrics", () => {
    it("should query real metrics instead of using random simulation", async () => {
      // Create a mock recommendation
      const recommendation: FeedbackRecommendation = {
        id: "test-recommendation-1",
        type: "system_configuration",
        action: {
          targetEntity: "test-service",
          operation: "config_update",
          parameters: { maxConnections: 100 },
        },
        expectedImpact: {
          metric: "throughput",
          improvementPercent: 25,
          timeToEffect: "immediate",
        },
        implementationStatus: "implemented",
        updatedAt: new Date(Date.now() - 1800000).toISOString(), // 1.5 hours ago
        priority: "medium",
        description: "Increase max connections to reduce response time",
        riskAssessment: {
          riskLevel: "low",
          rollbackPlan: "revert config",
          monitoringRequired: true,
        },
      };

      // Add to improvement history to simulate implemented recommendation
      (improvementEngine as any).improvementHistory.push(recommendation);

      // Mock the metrics collector to return realistic data
      const mockBeforeMetrics = [
        {
          timestamp: new Date(Date.now() - 2000000),
          cpuUsage: 75,
          memoryUsage: 80,
          availableMemoryMB: 2000,
          totalMemoryMB: 10000,
          diskUsage: 80,
          availableDiskGB: 200,
          loadAverage: [1.5, 1.2, 1.0] as [number, number, number],
          networkIO: { bytesInPerSecond: 1000, bytesOutPerSecond: 2000 },
          throughput: 100, // Low throughput before
        },
      ];

      const mockAfterMetrics = [
        {
          timestamp: new Date(Date.now() - 1800000),
          cpuUsage: 70,
          memoryUsage: 75,
          availableMemoryMB: 2500,
          totalMemoryMB: 10000,
          diskUsage: 75,
          availableDiskGB: 250,
          loadAverage: [1.2, 1.1, 1.0] as [number, number, number],
          networkIO: { bytesInPerSecond: 1200, bytesOutPerSecond: 2200 },
          throughput: 125, // Improved throughput (25% increase: (125-100)/100 = 25%)
        },
      ];

      // Mock the getHistoricalMetrics method
      jest
        .spyOn(metricsCollector, "getHistoricalMetrics")
        .mockResolvedValueOnce(mockBeforeMetrics)
        .mockResolvedValueOnce(mockAfterMetrics);

      // Monitor the improvement
      const result = await improvementEngine.monitorImprovementEffects(
        recommendation
      );

      // Verify the result is based on real metrics calculation
      expect(result.monitoringComplete).toBe(true);
      expect(result.actualImpact).toBeDefined();
      expect(result.actualImpact).toBeCloseTo(25, 1); // Should be close to 25% improvement
      expect(result.effective).toBe(true); // Should be effective since 25% > 50% of expected 25%
    });

    it("should handle insufficient metrics data gracefully", async () => {
      const recommendation: FeedbackRecommendation = {
        id: "test-recommendation-2",
        type: "system_configuration",
        action: {
          targetEntity: "test-service",
          operation: "config_update",
          parameters: { maxConnections: 100 },
        },
        expectedImpact: {
          metric: "response_time_ms",
          improvementPercent: 25,
          timeToEffect: "immediate",
        },
        implementationStatus: "implemented",
        updatedAt: new Date(Date.now() - 1800000).toISOString(),
        priority: "medium",
        description: "Increase max connections to reduce response time",
        riskAssessment: {
          riskLevel: "low",
          rollbackPlan: "revert config",
          monitoringRequired: true,
        },
      };

      (improvementEngine as any).improvementHistory.push(recommendation);

      // Mock empty metrics data
      jest
        .spyOn(metricsCollector, "getHistoricalMetrics")
        .mockResolvedValueOnce([]) // No before metrics
        .mockResolvedValueOnce([]); // No after metrics

      const result = await improvementEngine.monitorImprovementEffects(
        recommendation
      );

      expect(result.monitoringComplete).toBe(true);
      expect(result.effective).toBe(false); // Should be false due to insufficient data
      expect(result.actualImpact).toBeUndefined();
    });

    it("should handle metrics collection errors gracefully", async () => {
      const recommendation: FeedbackRecommendation = {
        id: "test-recommendation-3",
        type: "system_configuration",
        action: {
          targetEntity: "test-service",
          operation: "config_update",
          parameters: { maxConnections: 100 },
        },
        expectedImpact: {
          metric: "response_time_ms",
          improvementPercent: 25,
          timeToEffect: "immediate",
        },
        implementationStatus: "implemented",
        updatedAt: new Date(Date.now() - 1800000).toISOString(),
        priority: "medium",
        description: "Increase max connections to reduce response time",
        riskAssessment: {
          riskLevel: "low",
          rollbackPlan: "revert config",
          monitoringRequired: true,
        },
      };

      (improvementEngine as any).improvementHistory.push(recommendation);

      // Mock metrics collection error
      jest
        .spyOn(metricsCollector, "getHistoricalMetrics")
        .mockRejectedValue(new Error("Database connection failed"));

      const result = await improvementEngine.monitorImprovementEffects(
        recommendation
      );

      expect(result.monitoringComplete).toBe(true);
      expect(result.effective).toBe(false); // Should be false due to error
      expect(result.actualImpact).toBeUndefined();
    });

    it("should calculate improvement for different metric types", async () => {
      const recommendation: FeedbackRecommendation = {
        id: "test-recommendation-4",
        type: "system_configuration",
        action: {
          targetEntity: "test-service",
          operation: "config_update",
          parameters: { maxConnections: 100 },
        },
        expectedImpact: {
          metric: "cpu_usage", // Different target metric
          improvementPercent: 30,
          timeToEffect: "immediate",
        },
        implementationStatus: "implemented",
        updatedAt: new Date(Date.now() - 1800000).toISOString(),
        priority: "medium",
        description: "Optimize CPU usage",
        riskAssessment: {
          riskLevel: "low",
          rollbackPlan: "revert config",
          monitoringRequired: true,
        },
      };

      (improvementEngine as any).improvementHistory.push(recommendation);

      const mockBeforeMetrics = [
        {
          timestamp: new Date(Date.now() - 2000000),
          cpuUsage: 90, // High CPU usage before
          memoryUsage: 80,
          availableMemoryMB: 2000,
          totalMemoryMB: 10000,
          diskUsage: 80,
          availableDiskGB: 200,
          loadAverage: [1.5, 1.2, 1.0] as [number, number, number],
          networkIO: { bytesInPerSecond: 1000, bytesOutPerSecond: 2000 },
          cpu_usage: 90, // High CPU usage before
        },
      ];

      const mockAfterMetrics = [
        {
          timestamp: new Date(Date.now() - 1800000),
          cpuUsage: 63, // Reduced CPU usage (30% improvement)
          memoryUsage: 75,
          availableMemoryMB: 2500,
          totalMemoryMB: 10000,
          diskUsage: 75,
          availableDiskGB: 250,
          loadAverage: [1.2, 1.1, 1.0] as [number, number, number],
          networkIO: { bytesInPerSecond: 1200, bytesOutPerSecond: 2200 },
          cpu_usage: 63, // Reduced CPU usage (30% improvement)
        },
      ];

      jest
        .spyOn(metricsCollector, "getHistoricalMetrics")
        .mockResolvedValueOnce(mockBeforeMetrics)
        .mockResolvedValueOnce(mockAfterMetrics);

      const result = await improvementEngine.monitorImprovementEffects(
        recommendation
      );

      expect(result.monitoringComplete).toBe(true);
      expect(result.actualImpact).toBeDefined();
      expect(result.actualImpact).toBeCloseTo(-30, 1); // Should be close to -30% (CPU reduction is improvement)
      expect(result.effective).toBe(false); // Should be false since -30% is not > 50% of expected 30%
    });

    it("should handle zero baseline values gracefully", async () => {
      const recommendation: FeedbackRecommendation = {
        id: "test-recommendation-5",
        type: "system_configuration",
        action: {
          targetEntity: "test-service",
          operation: "config_update",
          parameters: { maxConnections: 100 },
        },
        expectedImpact: {
          metric: "response_time_ms",
          improvementPercent: 25,
          timeToEffect: "immediate",
        },
        implementationStatus: "implemented",
        updatedAt: new Date(Date.now() - 1800000).toISOString(),
        priority: "medium",
        description: "Increase max connections to reduce response time",
        riskAssessment: {
          riskLevel: "low",
          rollbackPlan: "revert config",
          monitoringRequired: true,
        },
      };

      (improvementEngine as any).improvementHistory.push(recommendation);

      // Mock metrics with zero baseline
      const mockBeforeMetrics = [
        {
          timestamp: new Date(Date.now() - 2000000),
          cpuUsage: 75,
          memoryUsage: 80,
          availableMemoryMB: 2000,
          totalMemoryMB: 10000,
          diskUsage: 80,
          availableDiskGB: 200,
          loadAverage: [1.5, 1.2, 1.0] as [number, number, number],
          networkIO: { bytesInPerSecond: 1000, bytesOutPerSecond: 2000 },
          response_time_ms: 0, // Zero baseline
        },
      ];

      const mockAfterMetrics = [
        {
          timestamp: new Date(Date.now() - 1800000),
          cpuUsage: 70,
          memoryUsage: 75,
          availableMemoryMB: 2500,
          totalMemoryMB: 10000,
          diskUsage: 75,
          availableDiskGB: 250,
          loadAverage: [1.2, 1.1, 1.0] as [number, number, number],
          networkIO: { bytesInPerSecond: 1200, bytesOutPerSecond: 2200 },
          response_time_ms: 375,
        },
      ];

      jest
        .spyOn(metricsCollector, "getHistoricalMetrics")
        .mockResolvedValueOnce(mockBeforeMetrics)
        .mockResolvedValueOnce(mockAfterMetrics);

      const result = await improvementEngine.monitorImprovementEffects(
        recommendation
      );

      expect(result.monitoringComplete).toBe(true);
      expect(result.actualImpact).toBe(0); // Should be 0 due to zero baseline
      expect(result.effective).toBe(false); // Should be false since impact is 0
    });
  });

  describe("calculateAverageMetric helper method", () => {
    it("should calculate average correctly", () => {
      const metrics = [
        { response_time_ms: 500 },
        { response_time_ms: 400 },
        { response_time_ms: 300 },
      ];

      const average = (improvementEngine as any).calculateAverageMetric(
        metrics,
        "response_time_ms"
      );
      expect(average).toBe(400); // (500 + 400 + 300) / 3
    });

    it("should handle empty metrics array", () => {
      const average = (improvementEngine as any).calculateAverageMetric(
        [],
        "response_time_ms"
      );
      expect(average).toBe(0);
    });

    it("should filter out invalid values", () => {
      const metrics = [
        { response_time_ms: 500 },
        { response_time_ms: null },
        { response_time_ms: "invalid" },
        { response_time_ms: 400 },
        { response_time_ms: NaN },
      ];

      const average = (improvementEngine as any).calculateAverageMetric(
        metrics,
        "response_time_ms"
      );
      expect(average).toBe(450); // (500 + 400) / 2
    });
  });
});
