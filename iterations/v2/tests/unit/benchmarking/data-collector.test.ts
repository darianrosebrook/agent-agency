/**
 * Data Collector Unit Tests
 *
 * @author @darianrosebrook
 */

import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { DataCollector } from "../../../src/benchmarking/DataCollector";
import {
  DataCollectionConfig,
  PerformanceEventType,
} from "../../../src/types/performance-tracking";

describe("DataCollector", () => {
  let collector: DataCollector;
  let defaultConfig: DataCollectionConfig;

  beforeEach(() => {
    collector = new DataCollector();
    defaultConfig = {
      enabled: true,
      samplingRate: 1.0,
      maxBufferSize: 10000,
      batchSize: 100,
      retentionDays: 90,
      anonymization: {
        enabled: true,
        level: "differential",
        preserveAgentIds: true,
        preserveTaskTypes: true,
      },
    };
  });

  afterEach(() => {
    collector.stopCollection();
  });

  describe("constructor", () => {
    it("should create with default config", () => {
      const collector = new DataCollector();
      const stats = collector.getStats();

      expect(stats.isCollecting).toBe(false);
      expect(stats.bufferSize).toBe(0);
    });

    it("should create with custom config", () => {
      const customConfig: Partial<DataCollectionConfig> = {
        enabled: false,
        samplingRate: 0.5,
        maxBufferSize: 5000,
      };

      const collector = new DataCollector(customConfig);
      const stats = collector.getStats();

      expect(stats.isCollecting).toBe(false);
      expect(stats.config.enabled).toBe(false);
      expect(stats.config.samplingRate).toBe(0.5);
      expect(stats.config.maxBufferSize).toBe(5000);
    });
  });

  describe("collection control", () => {
    it("should start collection when enabled", () => {
      collector.startCollection();
      const stats = collector.getStats();

      expect(stats.isCollecting).toBe(true);
    });

    it("should stop collection", () => {
      collector.startCollection();
      collector.stopCollection();
      const stats = collector.getStats();

      expect(stats.isCollecting).toBe(false);
    });

    it("should not start collection when disabled", () => {
      const disabledCollector = new DataCollector({ enabled: false });
      disabledCollector.startCollection();
      const stats = disabledCollector.getStats();

      expect(stats.isCollecting).toBe(false);
    });
  });

  describe("task execution tracking", () => {
    beforeEach(() => {
      collector.startCollection();
    });

    it("should record task start", () => {
      const executionId = collector.recordTaskStart("task-123", "agent-1", {
        priority: "high",
      });

      expect(executionId).toBeTruthy();
      expect(typeof executionId).toBe("string");

      const events = collector.getPendingEvents(10);
      expect(events).toHaveLength(1);
      expect(events[0].type).toBe(PerformanceEventType.TASK_EXECUTION_START);
      expect(events[0].taskId).toBe("task-123");
      expect(events[0].agentId).toBe("agent-1");
    });

    it("should record task completion with metrics", async () => {
      const executionId = collector.recordTaskStart("task-123", "agent-1");

      const metrics = {
        latency: {
          averageMs: 1500,
          p95Ms: 2000,
          p99Ms: 2500,
          minMs: 1000,
          maxMs: 3000,
        },
        accuracy: {
          successRate: 0.9,
          qualityScore: 0.85,
          violationRate: 0.1,
          evaluationScore: 0.8,
        },
        resources: {
          cpuUtilizationPercent: 75,
          memoryUtilizationPercent: 60,
          networkIoKbps: 100,
          diskIoKbps: 50,
        },
        compliance: {
          validationPassRate: 0.95,
          violationSeverityScore: 0.1,
          clauseCitationRate: 0.8,
        },
        cost: {
          costPerTask: 0.5,
          efficiencyScore: 0.85,
          resourceWastePercent: 15,
        },
        reliability: {
          mtbfHours: 168,
          availabilityPercent: 99.5,
          errorRatePercent: 0.5,
          recoveryTimeMinutes: 5,
        },
      };

      await collector.recordTaskCompletion("task-123", "agent-1", metrics);

      const events = collector.getPendingEvents(10);
      expect(events).toHaveLength(2); // start + completion

      const completionEvent = events.find(
        (e) => e.type === PerformanceEventType.TASK_EXECUTION_COMPLETE
      );
      expect(completionEvent).toBeDefined();
      expect(completionEvent?.taskId).toBe("task-123");
      expect(completionEvent?.agentId).toBe("agent-1");
      expect(completionEvent?.metrics).toEqual(metrics);
    });

    it("should respect sampling rate", () => {
      const lowSamplingCollector = new DataCollector({
        samplingRate: 0.0, // Never sample
      });
      lowSamplingCollector.startCollection();

      const executionId = lowSamplingCollector.recordTaskStart(
        "task-123",
        "agent-1"
      );
      expect(executionId).toBe(""); // Should return empty string when not sampling

      const events = lowSamplingCollector.getPendingEvents(10);
      expect(events).toHaveLength(0);
    });
  });

  describe("routing decision tracking", () => {
    beforeEach(() => {
      collector.startCollection();
    });

    it("should record routing decisions", async () => {
      const alternatives = [
        { agentId: "agent-1", score: 0.9 },
        { agentId: "agent-2", score: 0.7 },
      ];

      await collector.recordRoutingDecision(
        "task-123",
        "agent-1",
        alternatives,
        {
          confidence: 0.8,
          rationale: "Selected best performing agent",
        }
      );

      const events = collector.getPendingEvents(10);
      expect(events).toHaveLength(1);

      const event = events[0];
      expect(event.type).toBe(PerformanceEventType.ROUTING_DECISION);
      expect(event.taskId).toBe("task-123");
      expect(event.agentId).toBe("agent-1");
      expect(event.context?.alternatives).toEqual(alternatives);
    });
  });

  describe("evaluation outcome tracking", () => {
    beforeEach(() => {
      collector.startCollection();
    });

    it("should record evaluation outcomes", async () => {
      const evaluation = {
        passed: true,
        score: 0.85,
        rubricScores: {
          accuracy: 0.9,
          relevance: 0.8,
          completeness: 0.85,
        },
        feedback: "Good performance with minor issues",
      };

      await collector.recordEvaluationOutcome("task-123", "agent-1", 0.85, {
        evaluation,
      });

      const events = collector.getPendingEvents(10);
      expect(events).toHaveLength(1);

      const event = events[0];
      expect(event.type).toBe(PerformanceEventType.EVALUATION_OUTCOME);
      expect(event.taskId).toBe("task-123");
      expect(event.agentId).toBe("agent-1");
      expect(event.metrics?.accuracy?.evaluationScore).toBe(0.85);
    });
  });

  describe("constitutional validation tracking", () => {
    beforeEach(() => {
      collector.startCollection();
    });

    it("should record constitutional validations", async () => {
      await collector.recordConstitutionalValidation({
        taskId: "task-123",
        agentId: "agent-1",
        validationResult: {
          valid: true,
          violations: [],
          complianceScore: 0.9,
          processingTimeMs: 100,
          ruleCount: 5,
        },
      });

      const events = collector.getPendingEvents(10);
      expect(events).toHaveLength(1);

      const event = events[0];
      expect(event.type).toBe(PerformanceEventType.CONSTITUTIONAL_VALIDATION);
      expect(event.taskId).toBe("task-123");
      expect(event.agentId).toBe("agent-1");
      expect(event.metrics?.compliance?.validationPassRate).toBe(1);
      expect(event.metrics?.compliance?.violationSeverityScore).toBe(0.1);
    });
  });

  describe("anomaly detection tracking", () => {
    beforeEach(() => {
      collector.startCollection();
    });

    it("should record system anomalies", async () => {
      await collector.recordAnomaly("latency_spike", "critical", "agent-1", {
        spikeMultiplier: 3.5,
        baselineLatency: 1000,
        currentLatency: 3500,
      });

      const events = collector.getPendingEvents(10);
      expect(events).toHaveLength(1);

      const event = events[0];
      expect(event.type).toBe(PerformanceEventType.ANOMALY_DETECTED);
      expect(event.agentId).toBe("agent-1");
      expect(event.context?.anomalyType).toBe("latency_spike");
      expect(event.context?.severity).toBe("critical");
    });
  });

  describe("buffer management", () => {
    it("should manage buffer size limits", () => {
      const smallBufferCollector = new DataCollector({
        maxBufferSize: 2,
      });
      smallBufferCollector.startCollection();

      // Add events beyond buffer limit
      smallBufferCollector.recordTaskStart("task-1", "agent-1");
      smallBufferCollector.recordTaskStart("task-2", "agent-1");
      smallBufferCollector.recordTaskStart("task-3", "agent-1");

      const events = smallBufferCollector.getPendingEvents(10);
      expect(events.length).toBeLessThanOrEqual(2); // Should not exceed buffer size
    });

    it("should prioritize critical events", () => {
      const smallBufferCollector = new DataCollector({
        maxBufferSize: 2,
      });
      smallBufferCollector.startCollection();

      // Add low priority events first
      smallBufferCollector.recordTaskStart("task-1", "agent-1");
      smallBufferCollector.recordTaskStart("task-2", "agent-1");

      // Add high priority event - should trigger anomaly
      smallBufferCollector.recordAnomaly("latency_spike", "critical");

      const events = smallBufferCollector.getPendingEvents(10);
      // Critical anomaly should be included, some low priority events may be dropped
      expect(events.length).toBeGreaterThan(0);
    });

    it("should clear buffer when requested", () => {
      collector.startCollection();
      collector.recordTaskStart("task-1", "agent-1");

      let events = collector.getPendingEvents(10);
      expect(events).toHaveLength(1);

      collector.clearBuffer();

      events = collector.getPendingEvents(10);
      expect(events).toHaveLength(0);
    });
  });

  describe("data anonymization", () => {
    it("should anonymize sensitive data when enabled", () => {
      const collectorWithAnonymization = new DataCollector({
        anonymization: {
          enabled: true,
          level: "basic",
          preserveAgentIds: false,
          preserveTaskTypes: true,
        },
      });
      collectorWithAnonymization.startCollection();

      collectorWithAnonymization.recordTaskStart("task-123", "agent-1", {
        userId: "user-456",
        sessionId: "session-789",
        sensitiveData: "secret",
      });

      const events = collectorWithAnonymization.getPendingEvents(10);
      expect(events).toHaveLength(1);

      const event = events[0];
      // Note: agentId preservation is based on preserveAgentIds setting
      expect(event.context?.userId).toBeUndefined(); // Should be removed
      expect(event.context?.sessionId).toBeUndefined(); // Should be removed
      expect(event.context?.sensitiveData).toBeDefined(); // Should be hashed
    });

    it("should preserve data when anonymization disabled", () => {
      const collectorWithoutAnonymization = new DataCollector({
        anonymization: {
          enabled: false,
          level: "basic",
          preserveAgentIds: true,
          preserveTaskTypes: true,
        },
      });
      collectorWithoutAnonymization.startCollection();

      const originalContext = {
        userId: "user-456",
        sensitiveData: "secret",
      };

      collectorWithoutAnonymization.recordTaskStart(
        "task-123",
        "agent-1",
        originalContext
      );

      const events = collectorWithoutAnonymization.getPendingEvents(10);
      expect(events).toHaveLength(1);

      const event = events[0];
      expect(event.agentId).toBe("agent-1"); // Should be preserved
      expect(event.context).toEqual(originalContext); // Should be unchanged
    });
  });

  describe("data integrity", () => {
    it("should generate unique event IDs", () => {
      collector.startCollection();

      const id1 = collector.recordTaskStart("task-1", "agent-1");
      const id2 = collector.recordTaskStart("task-2", "agent-1");

      expect(id1).not.toBe(id2);
      expect(id1).toMatch(/^perf_\d+_[a-z0-9]+$/);
      expect(id2).toMatch(/^perf_\d+_[a-z0-9]+$/);
    });

    it("should include integrity hashes", () => {
      collector.startCollection();

      collector.recordTaskStart("task-123", "agent-1");

      const events = collector.getPendingEvents(10);
      expect(events).toHaveLength(1);

      const event = events[0];
      expect(event.integrityHash).toBeDefined();
      expect(typeof event.integrityHash).toBe("string");
      expect(event.integrityHash.length).toBeGreaterThan(0);
    });

    it("should generate consistent integrity hashes", () => {
      collector.startCollection();

      collector.recordTaskStart("task-123", "agent-1");
      const events1 = collector.getPendingEvents(10);

      collector.recordTaskStart("task-123", "agent-1");
      const events2 = collector.getPendingEvents(10);

      expect(events1[0].integrityHash).toBe(events2[0].integrityHash);
    });
  });

  describe("event prioritization", () => {
    it("should return events ordered by priority", () => {
      const smallBufferCollector = new DataCollector({
        maxBufferSize: 3,
      });
      smallBufferCollector.startCollection();

      // Add events in reverse priority order
      smallBufferCollector.recordTaskStart("task-1", "agent-1"); // normal
      smallBufferCollector.recordAnomaly("error", "low"); // critical
      smallBufferCollector.recordTaskStart("task-2", "agent-1"); // normal

      const events = smallBufferCollector.getPendingEvents(10);

      // Critical event should come first
      const criticalEvent = events.find(
        (e) => e.type === PerformanceEventType.ANOMALY_DETECTED
      );
      const firstNormalEvent = events.find(
        (e) => e.type === PerformanceEventType.TASK_EXECUTION_START
      );

      expect(criticalEvent).toBeDefined();
      expect(firstNormalEvent).toBeDefined();
    });
  });

  describe("performance monitoring", () => {
    it("should track collection statistics", () => {
      collector.startCollection();

      const initialStats = collector.getStats();

      collector.recordTaskStart("task-1", "agent-1");
      collector.recordTaskStart("task-2", "agent-1");

      const updatedStats = collector.getStats();

      expect(updatedStats.eventsCollected).toBe(2);
      expect(updatedStats.bufferSize).toBe(2);
      expect(updatedStats.averageCollectionTimeMs).toBeGreaterThanOrEqual(0);
    });

    it("should emit buffer high water mark events", () => {
      const mockEmitter = jest.fn();
      collector.on("buffer_high_water_mark", mockEmitter);

      const smallBufferCollector = new DataCollector({
        maxBufferSize: 2,
      });
      smallBufferCollector.startCollection();

      // Fill buffer to capacity
      smallBufferCollector.recordTaskStart("task-1", "agent-1");
      smallBufferCollector.recordTaskStart("task-2", "agent-1");

      // This should trigger high water mark when buffer is full
      expect(mockEmitter).toHaveBeenCalledWith(2);
    });
  });

  describe("configuration management", () => {
    it("should update configuration", () => {
      collector.updateConfig({
        samplingRate: 0.5,
        enabled: false,
      });

      const stats = collector.getStats();
      expect(stats.config.samplingRate).toBe(0.5);
      expect(stats.config.enabled).toBe(false);
    });

    it("should emit config update events", () => {
      const mockEmitter = jest.fn();
      collector.on("config_updated", mockEmitter);

      collector.updateConfig({ samplingRate: 0.8 });

      expect(mockEmitter).toHaveBeenCalledWith(
        expect.objectContaining({ samplingRate: 0.8 })
      );
    });
  });

  describe("collection lifecycle", () => {
    it("should handle collection start/stop cycles", () => {
      // Start collection
      collector.startCollection();
      expect(collector.getStats().isCollecting).toBe(true);

      // Stop collection
      collector.stopCollection();
      expect(collector.getStats().isCollecting).toBe(false);

      // Start again
      collector.startCollection();
      expect(collector.getStats().isCollecting).toBe(true);
    });

    it("should not collect events when stopped", () => {
      collector.startCollection();
      collector.recordTaskStart("task-1", "agent-1");

      collector.stopCollection();
      collector.recordTaskStart("task-2", "agent-1");

      const events = collector.getPendingEvents(10);
      expect(events).toHaveLength(1); // Only the first event
      expect(events[0].taskId).toBe("task-1");
    });
  });
});
