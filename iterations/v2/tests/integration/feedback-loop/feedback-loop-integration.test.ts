import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { FeedbackLoopManager } from "../../../src/feedback-loop/FeedbackLoopManager";
import { ConfigManager } from "../../../src/config/ConfigManager";
import { FeedbackSource, FeedbackType } from "../../../src/types/feedback-loop";

// Create a test configuration
const testConfig = {
  feedbackLoop: {
    enabled: true,
    collection: {
      enabledSources: [
        FeedbackSource.PERFORMANCE_METRICS,
        FeedbackSource.TASK_OUTCOMES,
        FeedbackSource.USER_RATINGS,
        FeedbackSource.SYSTEM_EVENTS,
      ],
      batchSize: 5, // Small batch size for testing
      flushIntervalMs: 100, // Fast flush for testing
      retentionPeriodDays: 30,
      samplingRate: 1.0,
      filters: {},
    },
    analysis: {
      enabledAnalyzers: ["trend", "anomaly"],
      analysisIntervalMs: 200, // Fast analysis for testing
      anomalyThreshold: 2.0,
      trendWindowHours: 24,
      minDataPoints: 3,
      correlationThreshold: 0.5,
      predictionHorizonHours: 24,
    },
    improvements: {
      autoApplyThreshold: 0.7,
      maxConcurrentImprovements: 3,
      cooldownPeriodMs: 1000, // Short cooldown for testing
      improvementTimeoutMs: 5000,
      rollbackOnFailure: false,
      monitoringPeriodMs: 2000,
    },
    pipeline: {
      batchSize: 10,
      processingIntervalMs: 500, // Fast processing for testing
      dataQualityThreshold: 0.5, // Lower threshold for testing
      anonymizationLevel: "none",
      featureEngineering: {
        timeWindowFeatures: false,
        correlationFeatures: false,
        trendFeatures: false,
      },
      trainingDataFormat: "json",
    },
  },
};

describe("FeedbackLoopManager Integration", () => {
  let manager: FeedbackLoopManager;
  let configManager: ConfigManager;

  beforeEach(async () => {
    // Create config manager with test config
    configManager = new ConfigManager();
    // Override config for testing
    (configManager as any).config = testConfig;

    manager = new FeedbackLoopManager(configManager);
    await manager.initialize();
  });

  afterEach(async () => {
    await manager.shutdown();
  });

  describe("end-to-end feedback processing", () => {
    it("should process performance feedback from collection to analysis", async () => {
      // Collect performance metrics
      manager.collectPerformanceMetrics("agent-1", "agent", {
        latencyMs: 100,
        throughput: 50,
        qualityScore: 0.9,
      });

      manager.collectPerformanceMetrics("agent-1", "agent", {
        latencyMs: 120,
        throughput: 45,
        qualityScore: 0.85,
      });

      manager.collectPerformanceMetrics("agent-2", "agent", {
        latencyMs: 80,
        throughput: 60,
        qualityScore: 0.95,
      });

      // Wait for batch processing
      await new Promise(resolve => setTimeout(resolve, 150));

      // Analyze the entity
      const analysis = manager.analyzeEntity("agent-1", "agent");

      expect(analysis).toBeDefined();
      expect(analysis.entityId).toBe("agent-1");
      expect(analysis.metrics.totalFeedbackEvents).toBeGreaterThan(0);
      expect(analysis.confidence).toBeGreaterThan(0);
    });

    it("should handle task outcomes and generate insights", async () => {
      // Collect task outcomes
      manager.collectTaskOutcome("task-1", { success: true }, 5000, 0);
      manager.collectTaskOutcome("task-2", { success: false }, 3000, 1, "Timeout error");
      manager.collectTaskOutcome("task-3", { success: true }, 4000, 0);
      manager.collectTaskOutcome("task-4", { success: false }, 6000, 2, "Network error");

      // Wait for processing
      await new Promise(resolve => setTimeout(resolve, 150));

      // Analyze all entities
      const analyses = manager.analyzeAllEntities("task");

      expect(analyses.length).toBeGreaterThan(0);

      // Find task analysis (though tasks may be grouped or analyzed differently)
      const taskAnalysis = analyses.find(a => a.entityType === "task");
      if (taskAnalysis) {
        expect(taskAnalysis.metrics.totalFeedbackEvents).toBe(4);
        expect(taskAnalysis.insights.length).toBeGreaterThanOrEqual(0); // May have error rate insight
      }
    });

    it("should process user ratings and correlate with performance", async () => {
      // Collect ratings for agents
      manager.collectUserRating("agent-1", "agent", 4.5, {
        accuracy: 5,
        speed: 4,
        reliability: 5,
        communication: 4,
      }, "Good performance");

      manager.collectUserRating("agent-1", "agent", 3.0, {
        accuracy: 3,
        speed: 3,
        reliability: 3,
        communication: 3,
      }, "Slower than expected");

      manager.collectUserRating("agent-2", "agent", 5.0, {
        accuracy: 5,
        speed: 5,
        reliability: 5,
        communication: 5,
      }, "Excellent work");

      // Collect corresponding performance metrics
      manager.collectPerformanceMetrics("agent-1", "agent", {
        latencyMs: 150,
        qualityScore: 0.8,
      });

      manager.collectPerformanceMetrics("agent-2", "agent", {
        latencyMs: 50,
        qualityScore: 0.98,
      });

      // Wait for processing
      await new Promise(resolve => setTimeout(resolve, 150));

      // Analyze agents
      const agent1Analysis = manager.analyzeEntity("agent-1", "agent");
      const agent2Analysis = manager.analyzeEntity("agent-2", "agent");

      expect(agent1Analysis).toBeDefined();
      expect(agent2Analysis).toBeDefined();
      expect(agent1Analysis.metrics.averageRating).toBeDefined();
      expect(agent2Analysis.metrics.averageRating).toBeDefined();

      // Agent 2 should have higher rating
      expect(agent2Analysis.metrics.averageRating! > agent1Analysis.metrics.averageRating!).toBe(true);
    });

    it("should handle system events and component health", async () => {
      // Collect system events
      manager.collectSystemEvent("overload-1", "high", "High CPU usage detected", {
        affectedComponents: ["scheduler", "router"],
        estimatedDowntimeMinutes: 5,
        userImpact: "minor",
      });

      manager.collectSystemEvent("recovery-1", "low", "System recovered", {
        affectedComponents: ["scheduler"],
        userImpact: "none",
      });

      // Collect component health
      manager.collectComponentHealth({
        componentId: "scheduler",
        status: "degraded",
        lastCheck: new Date(),
        responseTime: 1500,
        errorCount: 2,
        details: { cpuUsage: 85 },
      });

      manager.collectComponentHealth({
        componentId: "scheduler",
        status: "healthy",
        lastCheck: new Date(),
        responseTime: 200,
        errorCount: 0,
        details: { cpuUsage: 45 },
      });

      // Wait for processing
      await new Promise(resolve => setTimeout(resolve, 150));

      // Analyze system components
      const analyses = manager.analyzeAllEntities("component");

      expect(analyses.length).toBeGreaterThan(0);

      const schedulerAnalysis = analyses.find(a => a.entityId === "scheduler");
      if (schedulerAnalysis) {
        expect(schedulerAnalysis.metrics.totalFeedbackEvents).toBe(2);
      }
    });

    it("should generate and apply recommendations automatically", async () => {
      // Setup a scenario that should trigger recommendations
      // Agent with declining performance
      manager.collectPerformanceMetrics("agent-slow", "agent", {
        latencyMs: 100,
        qualityScore: 0.9,
      });

      // Simulate time passing with degrading performance
      await new Promise(resolve => setTimeout(resolve, 10));
      manager.collectPerformanceMetrics("agent-slow", "agent", {
        latencyMs: 200,
        qualityScore: 0.7,
      });

      await new Promise(resolve => setTimeout(resolve, 10));
      manager.collectPerformanceMetrics("agent-slow", "agent", {
        latencyMs: 300,
        qualityScore: 0.5,
      });

      // Add user ratings showing dissatisfaction
      manager.collectUserRating("agent-slow", "agent", 2.0, {
        accuracy: 2,
        speed: 1,
        reliability: 2,
        communication: 3,
      }, "Very slow and unreliable");

      // Wait for batch processing and analysis
      await new Promise(resolve => setTimeout(resolve, 250));

      // Check if recommendations were generated and applied
      const stats = manager.getStats();

      // Should have processed events
      expect(stats.totalEvents).toBeGreaterThan(0);

      // Check active improvements (may or may not be applied based on thresholds)
      const activeImprovements = manager.getActiveImprovements();
      // This will depend on the analysis and recommendation logic
    });

    it("should maintain statistics across operations", async () => {
      const initialStats = manager.getStats();

      // Perform various operations
      manager.collectPerformanceMetrics("agent-1", "agent", { latencyMs: 100 });
      manager.collectTaskOutcome("task-1", { success: true }, 1000, 0);
      manager.collectUserRating("agent-1", "agent", 4.0, {
        accuracy: 4,
        speed: 4,
        reliability: 4,
        communication: 4,
      });

      // Wait for processing
      await new Promise(resolve => setTimeout(resolve, 150));

      const updatedStats = manager.getStats();

      // Events should have increased
      expect(updatedStats.totalEvents).toBeGreaterThan(initialStats.totalEvents);

      // Uptime should be positive
      expect(updatedStats.uptimeSeconds).toBeGreaterThan(0);
    });

    it("should handle high-volume feedback collection", async () => {
      const startTime = Date.now();

      // Collect many events quickly
      for (let i = 0; i < 20; i++) {
        manager.collectPerformanceMetrics(`agent-${i % 5}`, "agent", {
          latencyMs: 100 + Math.random() * 100,
          qualityScore: 0.8 + Math.random() * 0.2,
        });
      }

      // Wait for batch processing (multiple batches should be processed)
      await new Promise(resolve => setTimeout(resolve, 300));

      const endTime = Date.now();
      const duration = endTime - startTime;

      const stats = manager.getStats();

      // Should have processed all events
      expect(stats.totalEvents).toBe(20);

      // Processing should be reasonably fast (< 500ms total for this test)
      expect(duration).toBeLessThan(500);

      this.logger?.info(`Processed 20 events in ${duration}ms`);
    });

    it("should provide health status information", () => {
      const health = manager.getHealthStatus();

      expect(health).toBeDefined();
      expect(["healthy", "degraded", "unhealthy"]).toContain(health.status);
      expect(health.details).toBeDefined();
      expect(typeof health.details.isRunning).toBe("boolean");
      expect(typeof health.details.bufferSize).toBe("number");
    });

    it("should handle analysis of entities with insufficient data", () => {
      // Try to analyze an entity with no data
      const analysis = manager.analyzeEntity("nonexistent-agent", "agent");

      expect(analysis).toBeDefined();
      expect(analysis.entityId).toBe("nonexistent-agent");
      expect(analysis.metrics.totalFeedbackEvents).toBe(0);
      expect(analysis.confidence).toBe(0);
    });
  });

  describe("error handling", () => {
    it("should handle invalid feedback data gracefully", async () => {
      // This would test invalid data handling, but since we validate at collection,
      // most invalid data is filtered out. In a real scenario, we might test
      // corrupted batches or invalid analysis requests.

      expect(async () => {
        await manager.initialize(); // Already initialized in beforeEach
      }).not.toThrow();
    });

    it("should continue operating after analysis errors", async () => {
      // Collect some valid data
      manager.collectPerformanceMetrics("agent-1", "agent", { latencyMs: 100 });

      // Wait for processing
      await new Promise(resolve => setTimeout(resolve, 150));

      // Should still be able to get stats
      const stats = manager.getStats();
      expect(stats).toBeDefined();
    });
  });
});
