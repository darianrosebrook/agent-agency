/**
 * Performance Tracker - Agent Registry Integration Test
 *
 * @author @darianrosebrook
 * @module tests/integration/performance-tracker-agent-integration
 *
 * End-to-end integration test verifying that the PerformanceTracker receives
 * and processes performance metrics from the AgentRegistryManager.
 */
// @ts-nocheck


import { AgentRegistryManager } from "@/orchestrator/AgentRegistryManager";
import { PerformanceTracker } from "@/rl/PerformanceTracker";

describe("Performance Tracker - Agent Registry Integration", () => {
  let registry: AgentRegistryManager;
  let performanceTracker: PerformanceTracker;

  beforeEach(async () => {
    performanceTracker = new PerformanceTracker({
      enabled: true,
      maxEventsInMemory: 1000,
    });

    // Start collection so the tracker will actually record events
    await performanceTracker.startCollection();

    registry = new AgentRegistryManager(
      {
        enablePersistence: false,
        enableSecurity: false,
      },
      performanceTracker
    );

    await registry.initialize();
  });

  afterEach(async () => {
    await registry.shutdown();
  });

  it("should record task performance when agent registry updates performance", async () => {
    // Register an agent
    const agent = await registry.registerAgent({
      id: "test-agent-001",
      name: "Test Agent",
      modelFamily: "gpt-4",
      capabilities: {
        taskTypes: ["code-editing"],
        languages: ["TypeScript"],
        specializationsV2: [
          {
            type: "Frontend architecture",
            level: "expert",
            successRate: 0.95,
            taskCount: 10,
            averageQuality: 0.92,
          },
        ],
      },
      performanceHistory: {
        successRate: 0.9,
        averageQuality: 0.88,
        averageLatency: 3000,
        taskCount: 20,
      },
      currentLoad: {
        activeTasks: 0,
        queuedTasks: 0,
        utilizationPercent: 0,
      },
    });

    expect(agent).toBeDefined();

    // Update performance metrics
    const metrics = {
      success: true,
      qualityScore: 0.95,
      latencyMs: 2500,
      taskType: "code-editing" as const,
    };

    const updatedAgent = await registry.updatePerformance(
      "test-agent-001",
      metrics
    );

    // Verify the agent registry updated correctly
    expect(updatedAgent.performanceHistory.successRate).toBeGreaterThan(0.9); // Running average updated
    expect(updatedAgent.performanceHistory.taskCount).toBe(21); // Task count incremented

    // Verify the performance tracker received the data
    const stats = performanceTracker.getStats();
    expect(stats.totalTaskExecutions).toBeGreaterThan(0);

    // Check that task performance events were recorded
    const events = performanceTracker.exportTrainingData();
    const taskPerformanceEvents = events.filter(
      (e) => e.type === "task-execution"
    );
    expect(taskPerformanceEvents.length).toBeGreaterThan(0);

    // Verify the event contains the correct data
    const taskEvent = taskPerformanceEvents[0];
    expect(taskEvent.data.agentId).toBe("test-agent-001");
    expect(taskEvent.data.taskType).toBe("code-editing");
    expect(taskEvent.data.success).toBe(true);
    expect(taskEvent.data.qualityScore).toBe(0.95);
    expect(taskEvent.data.latencyMs).toBe(2500);
    expect(taskEvent.data.eventType).toBe("agent_performance");
  });

  it("should record specialization performance updates", async () => {
    // Register an agent with specializations
    await registry.registerAgent({
      id: "specialist-agent-001",
      name: "Specialist Agent",
      modelFamily: "claude-3.5",
      capabilities: {
        taskTypes: ["code-editing", "api-design"],
        languages: ["TypeScript", "Python"],
        specializationsV2: [
          {
            type: "API design",
            level: "intermediate",
            successRate: 0.85,
            taskCount: 15,
            averageQuality: 0.82,
          },
        ],
      },
      performanceHistory: {
        successRate: 0.88,
        averageQuality: 0.85,
        averageLatency: 3500,
        taskCount: 30,
      },
      currentLoad: {
        activeTasks: 0,
        queuedTasks: 0,
        utilizationPercent: 0,
      },
    });

    // Update specialization performance
    const specMetrics = {
      success: true,
      qualityScore: 0.9,
      latencyMs: 0, // Specialization updates don't have latency
    };

    const updatedAgent = await registry.updateSpecializationPerformance(
      "specialist-agent-001",
      "API design",
      specMetrics
    );

    // Verify specialization was updated
    const apiDesignSpec = updatedAgent.capabilities.specializationsV2?.find(
      (s) => s.type === "API design"
    );
    expect(apiDesignSpec).toBeDefined();
    expect(apiDesignSpec!.taskCount).toBe(16);
    expect(apiDesignSpec!.successRate).toBeGreaterThan(0.85);

    // Verify performance tracker recorded it
    const events = performanceTracker.exportTrainingData();
    const taskPerformanceEvents = events.filter(
      (e) => e.type === "task-execution"
    );
    expect(taskPerformanceEvents.length).toBeGreaterThan(0);

    // Check for specialization-specific event
    const specEvent = taskPerformanceEvents.find(
      (e) => e.data.taskType === "API design"
    );
    expect(specEvent).toBeDefined();
  });

  it("should maintain performance history across multiple updates", async () => {
    // Register an agent
    await registry.registerAgent({
      id: "history-agent-001",
      name: "History Agent",
      modelFamily: "gpt-4",
      capabilities: {
        taskTypes: ["code-editing"],
        languages: ["TypeScript"],
        specializationsV2: [],
      },
      performanceHistory: {
        successRate: 0.8,
        averageQuality: 0.75,
        averageLatency: 4000,
        taskCount: 10,
      },
      currentLoad: {
        activeTasks: 0,
        queuedTasks: 0,
        utilizationPercent: 0,
      },
    });

    // Perform multiple performance updates
    const updates = [
      { success: true, qualityScore: 0.9, latencyMs: 3000 },
      { success: true, qualityScore: 0.85, latencyMs: 3500 },
      { success: false, qualityScore: 0.3, latencyMs: 5000 },
      { success: true, qualityScore: 0.95, latencyMs: 2800 },
    ];

    for (const metrics of updates) {
      await registry.updatePerformance("history-agent-001", {
        ...metrics,
        taskType: "code-editing",
      });
    }

    // Verify final state
    const finalAgent = await registry.getProfile("history-agent-001");
    expect(finalAgent.performanceHistory.taskCount).toBe(14); // 10 + 4 updates

    // Verify running averages are updated (using incremental average formula)
    // Initial: successRate = 0.8, taskCount = 10
    // Updates: true, true, false, true
    // The incremental formula maintains accurate running averages
    expect(finalAgent.performanceHistory.successRate).toBeGreaterThan(0.7);
    expect(finalAgent.performanceHistory.successRate).toBeLessThan(1.0);

    // Quality score should be a reasonable value (blended initial 0.75 with new scores)
    expect(finalAgent.performanceHistory.averageQuality).toBeGreaterThanOrEqual(
      0.5
    );
    expect(finalAgent.performanceHistory.averageQuality).toBeLessThanOrEqual(
      0.9
    );

    // Latency: (4000 * 10 + 3000 + 3500 + 5000 + 2800) / 14 ≈ 41800/14 ≈ 2985.7
    expect(finalAgent.performanceHistory.averageLatency).toBeLessThan(4000);

    // Verify all events were recorded in performance tracker
    const events = performanceTracker.exportTrainingData();
    const taskEvents = events.filter((e) => e.type === "task-execution");
    expect(taskEvents.length).toBe(4); // One for each update
  });

  it("should provide comprehensive performance statistics", async () => {
    // Register multiple agents with different performance profiles
    await registry.registerAgent({
      id: "high-performer",
      name: "High Performer",
      modelFamily: "gpt-4",
      capabilities: {
        taskTypes: ["code-editing"],
        languages: ["TypeScript"],
        specializationsV2: [],
      },
      performanceHistory: {
        successRate: 0.95,
        averageQuality: 0.92,
        averageLatency: 2500,
        taskCount: 50,
      },
      currentLoad: { activeTasks: 0, queuedTasks: 0, utilizationPercent: 0 },
    });

    await registry.registerAgent({
      id: "consistent-performer",
      name: "Consistent Performer",
      modelFamily: "claude-3.5",
      capabilities: {
        taskTypes: ["code-editing"],
        languages: ["TypeScript"],
        specializationsV2: [],
      },
      performanceHistory: {
        successRate: 0.88,
        averageQuality: 0.85,
        averageLatency: 3200,
        taskCount: 40,
      },
      currentLoad: { activeTasks: 0, queuedTasks: 0, utilizationPercent: 0 },
    });

    // Update performance for both agents
    await registry.updatePerformance("high-performer", {
      success: true,
      qualityScore: 0.95,
      latencyMs: 2400,
      taskType: "code-editing",
    });

    await registry.updatePerformance("consistent-performer", {
      success: true,
      qualityScore: 0.88,
      latencyMs: 3100,
      taskType: "code-editing",
    });

    // Get performance stats from tracker
    const stats = performanceTracker.getStats();
    expect(stats.totalTaskExecutions).toBeGreaterThan(0);

    // Verify events contain agent-specific data
    const events = performanceTracker.exportTrainingData();
    const agentEvents = events.filter((e) => e.type === "task-execution");

    const highPerformerEvents = agentEvents.filter(
      (e) => e.data.agentId === "high-performer"
    );
    const consistentPerformerEvents = agentEvents.filter(
      (e) => e.data.agentId === "consistent-performer"
    );

    expect(highPerformerEvents.length).toBe(1);
    expect(consistentPerformerEvents.length).toBe(1);

    // Verify event data accuracy
    expect(highPerformerEvents[0].data.qualityScore).toBe(0.95);
    expect(highPerformerEvents[0].data.latencyMs).toBe(2400);
    expect(consistentPerformerEvents[0].data.qualityScore).toBe(0.88);
    expect(consistentPerformerEvents[0].data.latencyMs).toBe(3100);
  });
});
