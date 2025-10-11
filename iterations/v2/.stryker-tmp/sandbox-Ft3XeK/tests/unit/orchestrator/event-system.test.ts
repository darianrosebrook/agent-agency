/**
 * @fileoverview Tests for Event System (ARBITER-005)
 *
 * @author @darianrosebrook
 */
// @ts-nocheck


import {
  EventEmitter,
  events,
  EventSeverity,
} from "../../../src/orchestrator/EventEmitter";
import {
  EventTypes,
  TaskDequeuedEvent,
  TaskEnqueuedEvent,
} from "../../../src/orchestrator/OrchestratorEvents";

describe("EventEmitter", () => {
  let emitter: EventEmitter;

  beforeEach(() => {
    emitter = new EventEmitter({ enabled: true });
  });

  afterEach(() => {
    emitter.clear();
    emitter.shutdown();
  });

  describe("Event Emission", () => {
    it("should emit events to registered handlers", async () => {
      const handler = jest.fn();
      emitter.on(EventTypes.TASK_ENQUEUED, handler);

      const event: TaskEnqueuedEvent = {
        id: "test-event-1",
        type: EventTypes.TASK_ENQUEUED,
        timestamp: new Date(),
        severity: EventSeverity.INFO,
        source: "TestComponent",
        taskId: "task-123",
        priority: 5,
        queueDepth: 10,
      };

      await emitter.emit(event);

      expect(handler).toHaveBeenCalledWith(event);
      expect(handler).toHaveBeenCalledTimes(1);
    });

    it("should support multiple handlers for the same event type", async () => {
      const handler1 = jest.fn();
      const handler2 = jest.fn();

      emitter.on(EventTypes.TASK_ENQUEUED, handler1);
      emitter.on(EventTypes.TASK_ENQUEUED, handler2);

      const event: TaskEnqueuedEvent = {
        id: "test-event-1",
        type: EventTypes.TASK_ENQUEUED,
        timestamp: new Date(),
        severity: EventSeverity.INFO,
        source: "TestComponent",
        taskId: "task-123",
        priority: 5,
        queueDepth: 10,
      };

      await emitter.emit(event);

      expect(handler1).toHaveBeenCalledWith(event);
      expect(handler2).toHaveBeenCalledWith(event);
    });

    it("should handle async event handlers", async () => {
      const handler = jest.fn().mockResolvedValue(undefined);
      emitter.on(EventTypes.TASK_ENQUEUED, handler);

      const event: TaskEnqueuedEvent = {
        id: "test-event-1",
        type: EventTypes.TASK_ENQUEUED,
        timestamp: new Date(),
        severity: EventSeverity.INFO,
        source: "TestComponent",
        taskId: "task-123",
        priority: 5,
        queueDepth: 10,
      };

      await emitter.emit(event);

      expect(handler).toHaveBeenCalledWith(event);
    });

    it("should store events in memory", async () => {
      const event: TaskEnqueuedEvent = {
        id: "test-event-1",
        type: EventTypes.TASK_ENQUEUED,
        timestamp: new Date(),
        severity: EventSeverity.INFO,
        source: "TestComponent",
        taskId: "task-123",
        priority: 5,
        queueDepth: 10,
      };

      await emitter.emit(event);

      const storedEvents = emitter.getEvents();
      expect(storedEvents).toHaveLength(1);
      expect(storedEvents[0]).toEqual(event);
    });

    it("should limit stored events to max capacity", async () => {
      // Create emitter with small capacity
      const smallEmitter = new EventEmitter({
        enabled: true,
        storage: {
          maxEvents: 2,
          retentionMs: 24 * 60 * 60 * 1000,
          persistentStorage: false,
        },
      });

      // Emit 3 events
      for (let i = 1; i <= 3; i++) {
        const event: TaskEnqueuedEvent = {
          id: `test-event-${i}`,
          type: EventTypes.TASK_ENQUEUED,
          timestamp: new Date(),
          severity: EventSeverity.INFO,
          source: "TestComponent",
          taskId: `task-${i}`,
          priority: 5,
          queueDepth: i,
        };
        await smallEmitter.emit(event);
      }

      const storedEvents = smallEmitter.getEvents();
      expect(storedEvents).toHaveLength(2); // Should only keep last 2
      expect(storedEvents[0].taskId).toBe("task-2");
      expect(storedEvents[1].taskId).toBe("task-3");
    });
  });

  describe("Event Filtering", () => {
    beforeEach(async () => {
      // Emit various events
      const events = [
        {
          id: "event-1",
          type: EventTypes.TASK_ENQUEUED,
          timestamp: new Date(),
          severity: EventSeverity.INFO,
          source: "TaskQueue",
          taskId: "task-1",
          priority: 5,
          queueDepth: 10,
        },
        {
          id: "event-2",
          type: EventTypes.TASK_DEQUEUED,
          timestamp: new Date(),
          severity: EventSeverity.WARN,
          source: "TaskQueue",
          taskId: "task-1",
          queueDepth: 9,
          waitTimeMs: 5000,
        },
        {
          id: "event-3",
          type: EventTypes.TASK_ENQUEUED,
          timestamp: new Date(),
          severity: EventSeverity.ERROR,
          source: "TaskAssignment",
          taskId: "task-2",
          priority: 3,
          queueDepth: 8,
        },
      ];

      for (const event of events) {
        await emitter.emit(event as TaskEnqueuedEvent | TaskDequeuedEvent);
      }
    });

    it("should filter events by type", () => {
      const enqueuedEvents = emitter.getEvents({
        types: [EventTypes.TASK_ENQUEUED],
      });
      expect(enqueuedEvents).toHaveLength(2);
      expect(
        enqueuedEvents.every((e) => e.type === EventTypes.TASK_ENQUEUED)
      ).toBe(true);
    });

    it("should filter events by severity", () => {
      const errorEvents = emitter.getEvents({
        severities: [EventSeverity.ERROR],
      });
      expect(errorEvents).toHaveLength(1);
      expect(errorEvents[0].severity).toBe(EventSeverity.ERROR);
    });

    it("should filter events by source", () => {
      const assignmentEvents = emitter.getEvents({
        sources: ["TaskAssignment"],
      });
      expect(assignmentEvents).toHaveLength(1);
      expect(assignmentEvents[0].source).toBe("TaskAssignment");
    });

    it("should filter events by task ID", () => {
      const task1Events = emitter.getEvents({
        taskIds: ["task-1"],
      });
      expect(task1Events).toHaveLength(2);
      expect(task1Events.every((e) => e.taskId === "task-1")).toBe(true);
    });

    it("should support custom filter functions", () => {
      const highPriorityEvents = emitter.getEvents({
        customFilter: (event) => (event as any).priority >= 5,
      });
      expect(highPriorityEvents).toHaveLength(1);
      expect((highPriorityEvents[0] as any).priority).toBe(5);
    });

    it("should combine multiple filters", () => {
      const filteredEvents = emitter.getEvents({
        types: [EventTypes.TASK_ENQUEUED],
        severities: [EventSeverity.INFO, EventSeverity.ERROR],
        sources: ["TaskQueue"],
      });
      expect(filteredEvents).toHaveLength(1);
      expect(filteredEvents[0].taskId).toBe("task-1");
    });
  });

  describe("Event Statistics", () => {
    beforeEach(async () => {
      const event: TaskEnqueuedEvent = {
        id: "test-event-1",
        type: EventTypes.TASK_ENQUEUED,
        timestamp: new Date(),
        severity: EventSeverity.INFO,
        source: "TaskQueue",
        taskId: "task-123",
        priority: 5,
        queueDepth: 10,
      };

      await emitter.emit(event);
    });

    it("should provide event statistics", () => {
      const stats = emitter.getStats();

      expect(stats.totalEvents).toBe(1);
      expect(stats.eventsByType.get(EventTypes.TASK_ENQUEUED)).toBe(1);
      expect(stats.eventsBySeverity.get(EventSeverity.INFO)).toBe(1);
      expect(stats.eventsBySource.get("TaskQueue")).toBe(1);
      expect(
        stats.handlersByType.get(EventTypes.TASK_ENQUEUED)
      ).toBeUndefined(); // No handlers registered
    });
  });

  describe("Global Event Emitter", () => {
    it("should provide global event emission", async () => {
      const handler = jest.fn();
      events.on(EventTypes.TASK_ENQUEUED, handler);

      const event: TaskEnqueuedEvent = {
        id: "global-event-1",
        type: EventTypes.TASK_ENQUEUED,
        timestamp: new Date(),
        severity: EventSeverity.INFO,
        source: "GlobalTest",
        taskId: "global-task",
        priority: 1,
        queueDepth: 5,
      };

      await events.emit(event);

      expect(handler).toHaveBeenCalledWith(event);

      // Cleanup
      events.off(EventTypes.TASK_ENQUEUED, handler);
    });
  });

  describe("Disabled Emitter", () => {
    it("should not emit events when disabled", async () => {
      const disabledEmitter = new EventEmitter({ enabled: false });
      const handler = jest.fn();

      disabledEmitter.on(EventTypes.TASK_ENQUEUED, handler);

      const event: TaskEnqueuedEvent = {
        id: "disabled-event",
        type: EventTypes.TASK_ENQUEUED,
        timestamp: new Date(),
        severity: EventSeverity.INFO,
        source: "DisabledTest",
        taskId: "disabled-task",
        priority: 1,
        queueDepth: 0,
      };

      await disabledEmitter.emit(event);

      expect(handler).not.toHaveBeenCalled();
      expect(disabledEmitter.getEvents()).toHaveLength(0);
    });
  });

  afterAll(() => {
    // Clean up global event emitter to prevent Jest from hanging
    events.shutdown();
  });
});
