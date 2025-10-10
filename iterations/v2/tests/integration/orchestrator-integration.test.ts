/**
 * @fileoverview Integration Tests for Orchestrator Components (ARBITER-005)
 *
 * Tests the integration between TaskQueue, TaskAssignment, SecurityManager,
 * and EventEmitter to ensure they work together correctly.
 *
 * @author @darianrosebrook
 */

import { afterEach, beforeEach, describe, expect, it } from "@jest/globals";
import { EventEmitter, events } from "../../../src/orchestrator/EventEmitter";
import { EventTypes } from "../../../src/orchestrator/OrchestratorEvents";
import { SecurityManager } from "../../../src/orchestrator/SecurityManager";
import { TaskAssignmentManager } from "../../../src/orchestrator/TaskAssignment";
import { TaskQueue } from "../../../src/orchestrator/TaskQueue";
import {
  AgentProfile,
  RoutingDecision,
  Task,
  TaskType,
} from "../../../src/types/arbiter-orchestration";

describe("Orchestrator Integration", () => {
  let taskQueue: TaskQueue;
  let assignmentManager: TaskAssignmentManager;
  let securityManager: SecurityManager;
  let eventEmitter: EventEmitter;

  // Test data
  const testAgent: AgentProfile = {
    id: "integration-agent",
    name: "Integration Test Agent",
    modelFamily: "gpt-4" as any,
    capabilities: {
      "code-editing": { supported: true, confidence: 0.9 },
      analysis: { supported: true, confidence: 0.8 },
    } as any,
    performanceHistory: [] as any,
    currentLoad: {
      activeTasks: 0,
      queueDepth: 0,
      memoryUsage: 0,
      cpuUsage: 0,
      lastUpdated: new Date(),
    } as any,
    registeredAt: new Date().toISOString(),
    lastActiveAt: new Date().toISOString(),
  };

  const testTask: Task = {
    id: "integration-task-1",
    description: "Integration test task",
    type: "code-editing" as TaskType,
    priority: 5,
    timeoutMs: 30000,
    attempts: 0,
    maxAttempts: 3,
    requiredCapabilities: {
      "code-editing": { supported: true, confidence: 0.8 },
    } as any,
    budget: {
      maxFiles: 10,
      maxLoc: 100,
    },
    createdAt: new Date(),
    metadata: {
      testId: "integration-test",
      source: "automated-test",
    },
  };

  beforeEach(async () => {
    // Setup components
    eventEmitter = new EventEmitter({ enabled: true });
    securityManager = new SecurityManager({
      enabled: true,
      trustedAgents: [],
      adminAgents: [],
    });
    securityManager.registerAgent(testAgent);

    taskQueue = new TaskQueue({
      maxCapacity: 100,
      defaultTimeoutMs: 30000,
      maxRetries: 3,
      priorityMode: "priority",
      persistenceEnabled: false,
      securityManager,
    });

    assignmentManager = new TaskAssignmentManager({
      acknowledgmentTimeoutMs: 5000,
      maxAssignmentDurationMs: 30000,
      autoReassignmentEnabled: true,
      maxReassignmentAttempts: 2,
      progressCheckIntervalMs: 5000,
      persistenceEnabled: false,
    });

    // Initialize components
    await taskQueue.initialize();
    await assignmentManager.initialize();
  });

  afterEach(async () => {
    // Cleanup
    await taskQueue.shutdown();
    await assignmentManager.shutdown();
    eventEmitter.clear();
    eventEmitter.shutdown();

    // Clear any remaining event handlers
    events.clear();
  });

  describe("Task Lifecycle Integration", () => {
    it("should handle complete task lifecycle with security and events", async () => {
      // Setup event tracking
      const eventsReceived: any[] = [];
      events.on("task.enqueued", (event) => eventsReceived.push(event));
      events.on("task.dequeued", (event) => eventsReceived.push(event));

      // 1. Authenticate agent
      const credentials = {
        agentId: "integration-agent",
        token: "valid-token-12345",
      };
      const context = securityManager.authenticate(credentials);
      expect(context).toBeTruthy();

      // 2. Enqueue task with authentication
      const secureQueue = new (
        await import("../../../src/orchestrator/TaskQueue")
      ).SecureTaskQueue(taskQueue, securityManager);

      await secureQueue.enqueue(testTask, credentials);

      // 3. Verify task was enqueued with events
      expect(eventsReceived).toHaveLength(1);
      expect(eventsReceived[0].type).toBe(EventTypes.TASK_ENQUEUED);
      expect(eventsReceived[0].taskId).toBe(testTask.id);

      // 4. Check queue state
      const queueSize = await taskQueue.size();
      expect(queueSize).toBe(1);

      // 5. Peek at task
      const peekedTask = await taskQueue.peek();
      expect(peekedTask?.id).toBe(testTask.id);

      // 6. Dequeue task
      const dequeuedTask = await taskQueue.dequeue();
      expect(dequeuedTask?.id).toBe(testTask.id);

      // 7. Verify dequeue event
      expect(eventsReceived).toHaveLength(2);
      expect(eventsReceived[1].type).toBe(EventTypes.TASK_DEQUEUED);
      expect(eventsReceived[1].taskId).toBe(testTask.id);

      // 8. Create task assignment
      const routingDecision: RoutingDecision = {
        id: "routing-decision-1",
        selectedAgent: testAgent,
        confidence: 0.9,
        strategy: "capability-match",
        reason: "Best capability match for code editing",
        timestamp: new Date(),
        alternatives: [],
      };

      const assignment = assignmentManager.createAssignment(
        dequeuedTask!,
        routingDecision
      );

      expect(assignment.task.id).toBe(testTask.id);
      expect(assignment.agent.id).toBe(testAgent.id);

      // 9. Acknowledge assignment
      const acknowledged = assignmentManager.acknowledgeAssignment(
        assignment.id
      );
      expect(acknowledged).toBe(true);

      // 10. Update progress
      const progressUpdated = assignmentManager.updateProgress(
        assignment.id,
        0.5,
        "running",
        { phase: "implementation" }
      );
      expect(progressUpdated).toBe(true);

      // 11. Complete assignment
      const result = {
        success: true,
        output: "Task completed successfully",
        qualityScore: 0.95,
        executionTimeMs: 15000,
      };

      assignmentManager.completeAssignment(assignment.id, result);

      // 12. Verify final state
      const finalQueueSize = await taskQueue.size();
      expect(finalQueueSize).toBe(0);

      // 13. Check security events
      const securityEvents = securityManager.getSecurityEvents();
      expect(
        securityEvents.some((e) => e.type === "security.authenticated")
      ).toBe(true);

      // 14. Verify all events were emitted
      const allEvents = eventEmitter.getEvents();
      expect(allEvents.length).toBeGreaterThan(0);
    });

    it("should handle security violations properly", async () => {
      // Try to enqueue without authentication
      const invalidCredentials = {
        agentId: "unknown-agent",
        token: "invalid-token",
      };

      const secureQueue = new (
        await import("../../../src/orchestrator/TaskQueue")
      ).SecureTaskQueue(taskQueue, securityManager);

      await expect(
        secureQueue.enqueue(testTask, invalidCredentials)
      ).rejects.toThrow();

      // Verify security events were logged
      const securityEvents = securityManager.getSecurityEvents();
      expect(
        securityEvents.some((e) => e.type === "security.auth_failed")
      ).toBe(true);

      // Verify task was not enqueued
      const queueSize = await taskQueue.size();
      expect(queueSize).toBe(0);
    });

    it("should handle input validation failures", async () => {
      const invalidTask = {
        id: "", // Invalid: empty ID
        description: "Test task",
        type: "invalid-type" as any,
        priority: 15, // Invalid: > 10
      };

      const credentials = {
        agentId: "integration-agent",
        token: "valid-token-12345",
      };

      await expect(
        taskQueue.enqueueWithCredentials(invalidTask as any, credentials)
      ).rejects.toThrow();

      // Verify task was not enqueued
      const queueSize = await taskQueue.size();
      expect(queueSize).toBe(0);
    });

    it("should handle concurrent operations safely", async () => {
      const credentials = {
        agentId: "integration-agent",
        token: "valid-token-12345",
      };

      // Start multiple concurrent enqueue operations
      const enqueuePromises = [];
      for (let i = 1; i <= 5; i++) {
        const concurrentTask: Task = {
          ...testTask,
          id: `concurrent-task-${i}`,
        };
        enqueuePromises.push(
          taskQueue.enqueueWithCredentials(concurrentTask, credentials)
        );
      }

      // All should succeed without race conditions
      await Promise.all(enqueuePromises);

      // Verify all tasks were enqueued
      const queueSize = await taskQueue.size();
      expect(queueSize).toBe(5);

      // Dequeue all tasks
      for (let i = 0; i < 5; i++) {
        const dequeued = await taskQueue.dequeue();
        expect(dequeued).toBeTruthy();
      }

      // Verify queue is empty
      const finalSize = await taskQueue.size();
      expect(finalSize).toBe(0);
    });

    it("should provide comprehensive event visibility", async () => {
      // Setup comprehensive event tracking
      const eventCounts: Record<string, number> = {};
      const eventHandler = (event: any) => {
        eventCounts[event.type] = (eventCounts[event.type] || 0) + 1;
      };

      // Listen to all task events
      events.onMultiple(
        [EventTypes.TASK_ENQUEUED, EventTypes.TASK_DEQUEUED],
        eventHandler
      );

      const credentials = {
        agentId: "integration-agent",
        token: "valid-token-12345",
      };

      // Perform task operations
      await taskQueue.enqueueWithCredentials(testTask, credentials);
      await taskQueue.dequeue();

      // Verify events were emitted
      expect(eventCounts[EventTypes.TASK_ENQUEUED]).toBe(1);
      expect(eventCounts[EventTypes.TASK_DEQUEUED]).toBe(1);

      // Check event filtering
      const enqueuedEvents = events.getEvents({
        types: [EventTypes.TASK_ENQUEUED],
        taskIds: [testTask.id],
      });
      expect(enqueuedEvents).toHaveLength(1);
      expect(enqueuedEvents[0].taskId).toBe(testTask.id);

      // Check event statistics
      const stats = events.getStats();
      expect(stats.totalEvents).toBeGreaterThanOrEqual(2);
      expect(stats.eventsByType.get(EventTypes.TASK_ENQUEUED)).toBe(1);
      expect(stats.eventsByType.get(EventTypes.TASK_DEQUEUED)).toBe(1);
    });
  });

  describe("System Health Integration", () => {
    it("should maintain system health metrics", async () => {
      // This would test integration with HealthMonitor
      // For now, just verify the components can be initialized together
      expect(taskQueue).toBeDefined();
      expect(assignmentManager).toBeDefined();
      expect(securityManager).toBeDefined();
      expect(eventEmitter).toBeDefined();

      // Verify all components are in ready state
      const queueSize = await taskQueue.size();
      expect(queueSize).toBe(0);

      const securityEvents = securityManager.getSecurityEvents();
      expect(Array.isArray(securityEvents)).toBe(true);

      const systemEvents = eventEmitter.getEvents();
      expect(Array.isArray(systemEvents)).toBe(true);
    });
  });
});
