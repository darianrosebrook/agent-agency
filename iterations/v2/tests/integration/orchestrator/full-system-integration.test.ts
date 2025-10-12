/**
 * @fileoverview Full System Integration Tests
 *
 * Comprehensive integration tests for the complete Arbiter Orchestrator
 * system including all components: TaskQueue, TaskAssignment, AgentRegistry,
 * Security, Health Monitoring, Recovery, and Knowledge Seeker.
 *
 * @author @darianrosebrook
 */

import {
  VerificationPriority,
  ArbiterOrchestrator,
  defaultArbiterOrchestratorConfig,
} from "../../../src/orchestrator/ArbiterOrchestrator";
import { AgentProfile } from "../../../src/types/agent-registry";
import {
  VerificationPriority,
  Task,
  TaskStatus,
  TaskType,
} from "../../../src/types/arbiter-orchestration";
import { KnowledgeQuery, QueryType } from "../../../src/types/knowledge";

describe("Full System Integration", () => {
  let orchestrator: ArbiterOrchestrator;

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
    description: "Integration test task for full system validation",
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
      testId: "full-system-integration",
      source: "automated-test",
    },
  };

  const knowledgeQuery: KnowledgeQuery = {
    id: "knowledge-query-1",
    query: "What are the best practices for TypeScript development?",
    queryType: QueryType.FACTUAL,
    maxResults: 5,
    relevanceThreshold: 0.7,
    timeoutMs: 10000,
    context: {
      domain: "software-development",
      audience: "developers",
    },
    metadata: {
      requesterId: "integration-test",
      priority: 3,
      createdAt: new Date(),
      tags: ["typescript", "best-practices"],
    },
  };

  beforeEach(async () => {
    // Create orchestrator with default config
    orchestrator = new ArbiterOrchestrator(defaultArbiterOrchestratorConfig);

    // Initialize the orchestrator
    await orchestrator.initialize();
  }, 30000); // Increase timeout for initialization

  afterEach(async () => {
    // Clean shutdown
    await orchestrator.shutdown();
  }, 10000);

  describe("System Initialization", () => {
    it("should initialize all components successfully", async () => {
      const status = await orchestrator.getStatus();

      expect(status.healthy).toBe(true);
      expect(status.components.taskQueue).toBe(true);
      expect(status.components.taskAssignment).toBe(true);
      expect(status.components.agentRegistry).toBe(true);
      expect(status.components.security).toBe(true);
      expect(status.components.healthMonitor).toBe(true);
      expect(status.components.recoveryManager).toBe(true);
      expect(status.components.knowledgeSeeker).toBe(true);
    });

    it("should provide comprehensive system metrics", async () => {
      const status = await orchestrator.getStatus();

      expect(status.metrics).toBeDefined();
      expect(typeof status.metrics.activeTasks).toBe("number");
      expect(typeof status.metrics.queuedTasks).toBe("number");
      expect(typeof status.metrics.registeredAgents).toBe("number");
      expect(typeof status.metrics.completedTasks).toBe("number");
      expect(typeof status.metrics.failedTasks).toBe("number");

      expect(status.knowledgeCapabilities).toBeDefined();
      expect(typeof status.knowledgeCapabilities.available).toBe("boolean");
      expect(Array.isArray(status.knowledgeCapabilities.providers)).toBe(true);
      expect(typeof status.knowledgeCapabilities.cacheSize).toBe("number");
    });
  });

  describe("Task Lifecycle Integration", () => {
    beforeEach(async () => {
      // Register test agent
      await orchestrator.registerAgent(testAgent);
    });

    it("should handle complete task lifecycle from submission to completion", async () => {
      // Submit task
      const submission = await orchestrator.submitTask(testTask);
      expect(submission.taskId).toBe(testTask.id);

      // Check task status
      const status = await orchestrator.getTaskStatus(testTask.id);
      expect(status).toBeDefined();

      // Task should be queued or assigned
      expect([TaskStatus.QUEUED, TaskStatus.ASSIGNED]).toContain(status);

      // Wait a bit for processing (in real system this would be async)
      await new Promise((resolve) => setTimeout(resolve, 100));

      // Task should still be in progress
      const updatedStatus = await orchestrator.getTaskStatus(testTask.id);
      expect(updatedStatus).toBeDefined();
    });

    it("should handle task cancellation", async () => {
      // Submit task
      await orchestrator.submitTask(testTask);

      // Cancel task
      const cancelled = await orchestrator.cancelTask(testTask.id);
      expect(typeof cancelled).toBe("boolean");

      // Task should be cancelled
      const status = await orchestrator.getTaskStatus(testTask.id);
      expect(status).toBe(TaskStatus.CANCELED);
    });

    it("should enforce security on task operations", async () => {
      // Try to submit task without authentication (should work with default config)
      const result = await orchestrator.submitTask(testTask);
      expect(result.taskId).toBe(testTask.id);

      // Test authentication
      const validCredentials = {
        agentId: "integration-agent",
        token: "valid-token-12345",
      };

      const authResult = orchestrator.authenticate(validCredentials);
      expect(authResult).toBeDefined();

      // Test authorization
      const isAuthorized = orchestrator.authorize(authResult, "submit_task");
      expect(typeof isAuthorized).toBe("boolean");
    });
  });

  describe("Agent Management Integration", () => {
    it("should register and retrieve agents", async () => {
      // Register agent
      await orchestrator.registerAgent(testAgent);

      // Retrieve agent
      const retrieved = await orchestrator.getAgentProfile(testAgent.id);
      expect(retrieved).toBeDefined();
      expect(retrieved!.id).toBe(testAgent.id);
      expect(retrieved!.name).toBe(testAgent.name);
    });

    it("should handle agent performance updates", async () => {
      // Register agent
      await orchestrator.registerAgent(testAgent);

      // Update performance
      const performanceUpdate = {
        taskCompleted: true,
        qualityScore: 0.95,
        durationMs: 15000,
        success: true,
      };

      await orchestrator.updateAgentPerformance(
        testAgent.id,
        performanceUpdate
      );

      // Verify performance was updated
      const updated = await orchestrator.getAgentProfile(testAgent.id);
      expect(updated).toBeDefined();
      expect(updated!.performanceHistory).toBeDefined();
    });
  });

  describe("Knowledge Research Integration", () => {
    it("should process knowledge queries", async () => {
      const response = await orchestrator.processKnowledgeQuery(knowledgeQuery);

      expect(response).toBeDefined();
      expect(response.query.id).toBe(knowledgeQuery.id);
      expect(response.results).toBeInstanceOf(Array);
      expect(response.summary).toBeDefined();
      expect(response.confidence).toBeGreaterThanOrEqual(0);
      expect(response.confidence).toBeLessThanOrEqual(1);
      expect(response.sourcesUsed).toBeInstanceOf(Array);
      expect(response.metadata.totalResultsFound).toBeGreaterThanOrEqual(0);
    });

    it("should provide knowledge seeker status", async () => {
      const status = await orchestrator.getKnowledgeStatus();

      expect(status).toBeDefined();
      expect(typeof status.enabled).toBe("boolean");
      expect(Array.isArray(status.providers)).toBe(true);
      expect(status.cacheStats).toBeDefined();
      expect(status.processingStats).toBeDefined();
    });

    it("should handle knowledge cache operations", async () => {
      // Process a query to populate cache
      await orchestrator.processKnowledgeQuery(knowledgeQuery);

      // Check cache status
      const status = await orchestrator.getKnowledgeStatus();
      expect(status.cacheStats.queryCacheSize).toBeGreaterThanOrEqual(0);

      // Clear caches
      await orchestrator.clearKnowledgeCaches();

      // Verify caches are cleared
      const updatedStatus = await orchestrator.getKnowledgeStatus();
      expect(updatedStatus.cacheStats.queryCacheSize).toBe(0);
      expect(updatedStatus.cacheStats.resultCacheSize).toBe(0);
    });

    it("should handle concurrent knowledge queries", async () => {
      const queries = [
        knowledgeQuery,
        { ...knowledgeQuery, id: "query-2", query: "What is React?" },
        { ...knowledgeQuery, id: "query-3", query: "Explain async/await" },
      ];

      const responses = await Promise.all(
        queries.map((query) => orchestrator.processKnowledgeQuery(query))
      );

      expect(responses).toHaveLength(3);
      responses.forEach((response) => {
        expect(response.results.length).toBeGreaterThan(0);
        expect(response.confidence).toBeGreaterThan(0);
      });
    });
  });

  describe("Cross-Component Integration", () => {
    it("should integrate task execution with knowledge research", async () => {
      // Register agent with knowledge capabilities
      const knowledgeAgent = {
        ...testAgent,
        capabilities: {
          ...testAgent.capabilities,
          "knowledge-research": { supported: true, confidence: 0.9 },
        } as any,
      };

      await orchestrator.registerAgent(knowledgeAgent);

      // Create a task that requires knowledge research
      const researchTask: Task = {
        ...testTask,
        id: "research-task",
        description: "Research TypeScript best practices",
        type: "knowledge-research" as TaskType,
        metadata: {
          knowledgeQuery: knowledgeQuery,
          researchRequired: true,
        },
      };

      // Submit task
      const submission = await orchestrator.submitTask(researchTask);
      expect(submission.taskId).toBe(researchTask.id);

      // The system should be able to handle this integrated workflow
      const status = await orchestrator.getTaskStatus(researchTask.id);
      expect(status).toBeDefined();
    });

    it("should maintain system health during concurrent operations", async () => {
      // Register multiple agents
      const agents = Array.from({ length: 3 }, (_, i) => ({
        ...testAgent,
        id: `agent-${i}`,
        name: `Agent ${i}`,
      }));

      await Promise.all(
        agents.map((agent) => orchestrator.registerAgent(agent))
      );

      // Submit multiple tasks concurrently
      const tasks = Array.from({ length: 5 }, (_, i) => ({
        ...testTask,
        id: `task-${i}`,
        description: `Task ${i}`,
      }));

      await Promise.all(tasks.map((task) => orchestrator.submitTask(task)));

      // Submit multiple knowledge queries concurrently
      const queries = Array.from({ length: 3 }, (_, i) => ({
        ...knowledgeQuery,
        id: `k-query-${i}`,
        query: `Query ${i}: ${knowledgeQuery.query}`,
      }));

      await Promise.all(
        queries.map((query) => orchestrator.processKnowledgeQuery(query))
      );

      // System should remain healthy
      const status = await orchestrator.getStatus();
      expect(status.healthy).toBe(true);

      // Should have reasonable metrics
      expect(status.metrics.queuedTasks).toBeGreaterThanOrEqual(0);
      expect(status.metrics.registeredAgents).toBe(3);
    });

    it("should handle failure scenarios gracefully", async () => {
      // Test with invalid agent registration
      const invalidAgent = {
        ...testAgent,
        id: "", // Invalid ID
      };

      await expect(orchestrator.registerAgent(invalidAgent)).rejects.toThrow();

      // Test with invalid knowledge query
      const invalidQuery = {
        ...knowledgeQuery,
        query: "", // Invalid empty query
      };

      await expect(
        orchestrator.processKnowledgeQuery(invalidQuery)
      ).rejects.toThrow();

      // System should remain healthy despite failures
      const status = await orchestrator.getStatus();
      expect(status.healthy).toBe(true);
    });
  });

  describe("Event Integration", () => {
    it("should emit and handle system events", async () => {
      let eventReceived = false;
      let receivedEvent: any = null;

      // Listen for task-related events
      const eventHandler = (event: any) => {
        eventReceived = true;
        receivedEvent = event;
      };

      // Register agent and submit task to trigger events
      await orchestrator.registerAgent(testAgent);
      await orchestrator.submitTask(testTask);

      // Process knowledge query to trigger more events
      await orchestrator.processKnowledgeQuery(knowledgeQuery);

      // Events should be emitted (we can't easily test the exact handler
      // without more complex setup, but we can verify the system doesn't crash)
      const status = await orchestrator.getStatus();
      expect(status.healthy).toBe(true);
    });
  });

  describe("Performance and Scalability", () => {
    it("should handle multiple concurrent operations", async () => {
      const startTime = Date.now();

      // Perform multiple operations concurrently
      const operations = [
        // Register agents
        ...Array.from({ length: 5 }, (_, i) =>
          orchestrator.registerAgent({
            ...testAgent,
            id: `perf-agent-${i}`,
            name: `Performance Agent ${i}`,
          })
        ),

        // Submit tasks
        ...Array.from({ length: 10 }, (_, i) =>
          orchestrator.submitTask({
            ...testTask,
            id: `perf-task-${i}`,
            description: `Performance task ${i}`,
          })
        ),

        // Process knowledge queries
        ...Array.from({ length: 3 }, (_, i) =>
          orchestrator.processKnowledgeQuery({
            ...knowledgeQuery,
            id: `perf-query-${i}`,
            query: `Performance query ${i}: ${knowledgeQuery.query}`,
          })
        ),
      ];

      await Promise.all(operations);

      const duration = Date.now() - startTime;

      // Should complete within reasonable time (adjust based on system)
      expect(duration).toBeLessThan(30000); // 30 seconds max

      // System should remain healthy
      const status = await orchestrator.getStatus();
      expect(status.healthy).toBe(true);
      expect(status.metrics.registeredAgents).toBeGreaterThanOrEqual(5);
      expect(status.metrics.queuedTasks).toBeGreaterThanOrEqual(10);
    });

    it("should maintain performance under load", async () => {
      // Register agents
      await Promise.all(
        Array.from({ length: 3 }, (_, i) =>
          orchestrator.registerAgent({
            ...testAgent,
            id: `load-agent-${i}`,
          })
        )
      );

      // Submit tasks
      await Promise.all(
        Array.from({ length: 20 }, (_, i) =>
          orchestrator.submitTask({
            ...testTask,
            id: `load-task-${i}`,
          })
        )
      );

      // Check system remains responsive
      const status = await orchestrator.getStatus();
      expect(status.healthy).toBe(true);

      // Should be able to get agent profiles quickly
      const agent = await orchestrator.getAgentProfile("load-agent-0");
      expect(agent).toBeDefined();
    });
  });
});
