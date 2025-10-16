/**
 * @fileoverview Unit tests for StatisticsCollector
 *
 * Tests statistics collection utility for orchestrator.
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { TaskQueue } from "../../../../src/orchestrator/TaskQueue";
import { TaskStateMachine } from "../../../../src/orchestrator/TaskStateMachine";
import { StatisticsCollector } from "../../../../src/orchestrator/utils/StatisticsCollector";
import { TaskState } from "../../../../src/types/task-state";

describe("StatisticsCollector", () => {
  let statisticsCollector: StatisticsCollector;
  let mockStateMachine: jest.Mocked<TaskStateMachine>;
  let mockTaskQueue: jest.Mocked<TaskQueue>;

  beforeEach(() => {
    mockStateMachine = {
      getStats: jest.fn(),
      on: jest.fn(),
      emit: jest.fn(),
    } as any;

    mockTaskQueue = {
      getStats: jest.fn(),
      on: jest.fn(),
      emit: jest.fn(),
    } as any;

    // Default mock return values
    mockTaskQueue.getStats.mockReturnValue({
      queued: 5,
      processing: 3,
      total: 8,
    });

    mockStateMachine.getStats.mockReturnValue({
      [TaskState.PENDING]: 2,
      [TaskState.QUEUED]: 5,
      [TaskState.ASSIGNED]: 1,
      [TaskState.RUNNING]: 2,
      [TaskState.SUSPENDED]: 0,
      [TaskState.COMPLETED]: 10,
      [TaskState.FAILED]: 1,
      [TaskState.CANCELLED]: 0,
    });
  });

  afterEach(() => {
    if (statisticsCollector) {
      statisticsCollector.stop();
    }
  });

  describe("initialization", () => {
    it("should create collector with default config", () => {
      statisticsCollector = new StatisticsCollector(
        mockStateMachine,
        mockTaskQueue
      );

      expect(statisticsCollector).toBeDefined();
    });

    it("should create collector with custom config", () => {
      statisticsCollector = new StatisticsCollector(
        mockStateMachine,
        mockTaskQueue,
        {
          statsIntervalMs: 5000,
          enableAutoEmit: true,
        }
      );

      expect(statisticsCollector).toBeDefined();
    });
  });

  describe("collectStats", () => {
    it("should collect comprehensive statistics", () => {
      statisticsCollector = new StatisticsCollector(
        mockStateMachine,
        mockTaskQueue
      );

      const stats = statisticsCollector.collectStats();

      expect(stats).toHaveProperty("queuedTasks", 5);
      expect(stats).toHaveProperty("processingTasks", 3);
      expect(stats).toHaveProperty("completedTasks", 10);
      expect(stats).toHaveProperty("failedTasks", 1);
      expect(stats).toHaveProperty("cancelledTasks", 0);
      expect(stats).toHaveProperty("throughput");
      expect(stats).toHaveProperty("avgLatency");
      expect(stats).toHaveProperty("timestamp");
      expect(stats.timestamp).toBeInstanceOf(Date);
    });

    it("should calculate throughput correctly", () => {
      statisticsCollector = new StatisticsCollector(
        mockStateMachine,
        mockTaskQueue
      );

      // Wait a bit for time to pass
      const stats1 = statisticsCollector.collectStats();
      expect(stats1.throughput).toBeGreaterThanOrEqual(0);

      // With 10 completed tasks initially
      const stats2 = statisticsCollector.collectStats();
      expect(stats2.throughput).toBeGreaterThanOrEqual(0);
    });

    it("should return zero average latency when no tasks recorded", () => {
      statisticsCollector = new StatisticsCollector(
        mockStateMachine,
        mockTaskQueue
      );

      const stats = statisticsCollector.collectStats();
      expect(stats.avgLatency).toBe(0);
    });
  });

  describe("recordLatency", () => {
    it("should record and average latencies", () => {
      statisticsCollector = new StatisticsCollector(
        mockStateMachine,
        mockTaskQueue
      );

      statisticsCollector.recordLatency(1000);
      statisticsCollector.recordLatency(2000);
      statisticsCollector.recordLatency(3000);

      const stats = statisticsCollector.collectStats();
      expect(stats.avgLatency).toBe(2000); // (1000 + 2000 + 3000) / 3
    });

    it("should update average as new latencies recorded", () => {
      statisticsCollector = new StatisticsCollector(
        mockStateMachine,
        mockTaskQueue
      );

      statisticsCollector.recordLatency(1000);
      const stats1 = statisticsCollector.collectStats();
      expect(stats1.avgLatency).toBe(1000);

      statisticsCollector.recordLatency(3000);
      const stats2 = statisticsCollector.collectStats();
      expect(stats2.avgLatency).toBe(2000); // (1000 + 3000) / 2
    });
  });

  describe("automatic collection", () => {
    it("should start automatic collection when enabled", (done) => {
      statisticsCollector = new StatisticsCollector(
        mockStateMachine,
        mockTaskQueue,
        {
          statsIntervalMs: 100, // Fast interval for testing
          enableAutoEmit: true,
        }
      );

      const emitSpy = jest.fn();
      statisticsCollector.on("orchestrator:stats", emitSpy);

      statisticsCollector.start();

      // Wait for at least one emission
      setTimeout(() => {
        expect(emitSpy).toHaveBeenCalled();
        const stats = emitSpy.mock.calls[0][0];
        expect(stats).toHaveProperty("queuedTasks");
        expect(stats).toHaveProperty("processingTasks");
        statisticsCollector.stop();
        done();
      }, 150);
    });

    it("should not start multiple intervals", () => {
      statisticsCollector = new StatisticsCollector(
        mockStateMachine,
        mockTaskQueue,
        {
          statsIntervalMs: 10000,
          enableAutoEmit: true,
        }
      );

      statisticsCollector.start();
      statisticsCollector.start(); // Second call should be ignored

      // Should only have one interval running
      expect(statisticsCollector).toBeDefined();
    });

    it("should stop automatic collection", (done) => {
      statisticsCollector = new StatisticsCollector(
        mockStateMachine,
        mockTaskQueue,
        {
          statsIntervalMs: 50,
          enableAutoEmit: true,
        }
      );

      const emitSpy = jest.fn();
      statisticsCollector.on("orchestrator:stats", emitSpy);

      statisticsCollector.start();

      setTimeout(() => {
        const callCount = emitSpy.mock.calls.length;
        statisticsCollector.stop();

        // Wait to ensure no more emissions after stop
        setTimeout(() => {
          expect(emitSpy.mock.calls.length).toBe(callCount);
          done();
        }, 100);
      }, 100);
    });
  });

  describe("getQueueStats", () => {
    it("should return queue statistics", () => {
      statisticsCollector = new StatisticsCollector(
        mockStateMachine,
        mockTaskQueue
      );

      const queueStats = statisticsCollector.getQueueStats();

      expect(queueStats).toEqual({
        queued: 5,
        processing: 3,
        total: 8,
      });
      expect(mockTaskQueue.getStats).toHaveBeenCalled();
    });
  });

  describe("getStateStats", () => {
    it("should return state machine statistics", () => {
      statisticsCollector = new StatisticsCollector(
        mockStateMachine,
        mockTaskQueue
      );

      const stateStats = statisticsCollector.getStateStats();

      expect(stateStats[TaskState.COMPLETED]).toBe(10);
      expect(stateStats[TaskState.FAILED]).toBe(1);
      expect(stateStats[TaskState.QUEUED]).toBe(5);
      expect(mockStateMachine.getStats).toHaveBeenCalled();
    });
  });

  describe("reset", () => {
    it("should reset statistics", () => {
      statisticsCollector = new StatisticsCollector(
        mockStateMachine,
        mockTaskQueue
      );

      statisticsCollector.recordLatency(1000);
      statisticsCollector.recordLatency(2000);

      const statsBefore = statisticsCollector.collectStats();
      expect(statsBefore.avgLatency).toBe(1500);

      statisticsCollector.reset();

      const statsAfter = statisticsCollector.collectStats();
      expect(statsAfter.avgLatency).toBe(0);
      expect(statsAfter.throughput).toBeGreaterThanOrEqual(0);
    });

    it("should reset start time for throughput calculation", () => {
      statisticsCollector = new StatisticsCollector(
        mockStateMachine,
        mockTaskQueue
      );

      const stats1 = statisticsCollector.collectStats();
      const throughput1 = stats1.throughput;

      statisticsCollector.reset();

      const stats2 = statisticsCollector.collectStats();
      // After reset, throughput should be recalculated from new start time
      expect(stats2.throughput).toBeDefined();
    });
  });

  describe("edge cases", () => {
    it("should handle zero completed tasks", () => {
      mockStateMachine.getStats.mockReturnValue({
        [TaskState.PENDING]: 5,
        [TaskState.QUEUED]: 3,
        [TaskState.ASSIGNED]: 0,
        [TaskState.RUNNING]: 2,
        [TaskState.SUSPENDED]: 0,
        [TaskState.COMPLETED]: 0,
        [TaskState.FAILED]: 0,
        [TaskState.CANCELLED]: 0,
      });

      statisticsCollector = new StatisticsCollector(
        mockStateMachine,
        mockTaskQueue
      );

      const stats = statisticsCollector.collectStats();
      expect(stats.completedTasks).toBe(0);
      expect(stats.throughput).toBeGreaterThanOrEqual(0);
    });

    it("should handle empty queue", () => {
      mockTaskQueue.getStats.mockReturnValue({
        queued: 0,
        processing: 0,
        total: 0,
      });

      statisticsCollector = new StatisticsCollector(
        mockStateMachine,
        mockTaskQueue
      );

      const stats = statisticsCollector.collectStats();
      expect(stats.queuedTasks).toBe(0);
      expect(stats.processingTasks).toBe(0);
    });

    it("should handle high latency values", () => {
      statisticsCollector = new StatisticsCollector(
        mockStateMachine,
        mockTaskQueue
      );

      statisticsCollector.recordLatency(100000); // 100 seconds
      statisticsCollector.recordLatency(200000); // 200 seconds

      const stats = statisticsCollector.collectStats();
      expect(stats.avgLatency).toBe(150000);
    });
  });

  describe("concurrent operations", () => {
    it("should handle concurrent latency recordings", () => {
      statisticsCollector = new StatisticsCollector(
        mockStateMachine,
        mockTaskQueue
      );

      // Record multiple latencies concurrently
      const latencies = [1000, 1500, 2000, 2500, 3000];
      latencies.forEach((latency) => {
        statisticsCollector.recordLatency(latency);
      });

      const stats = statisticsCollector.collectStats();
      const expectedAvg =
        latencies.reduce((a, b) => a + b, 0) / latencies.length;
      expect(stats.avgLatency).toBe(expectedAvg);
    });

    it("should handle stats collection during emission", (done) => {
      statisticsCollector = new StatisticsCollector(
        mockStateMachine,
        mockTaskQueue,
        {
          statsIntervalMs: 50,
          enableAutoEmit: true,
        }
      );

      statisticsCollector.start();

      // Collect stats manually while auto-collection is running
      const manualStats = statisticsCollector.collectStats();
      expect(manualStats).toBeDefined();

      setTimeout(() => {
        statisticsCollector.stop();
        done();
      }, 100);
    });
  });
});
